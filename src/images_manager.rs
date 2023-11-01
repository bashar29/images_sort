use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::directories;
use crate::exif;
use crate::exif::ExifData;
use crate::exif::ExifError;
use crate::global_configuration::GlobalConfiguration;
use crate::reporting::Reporting;
use eyre::Result;
use indicatif::ProgressBar;

pub fn sort_images_in_dir(
    dir: &std::path::Path,
    target_dir: &std::path::Path,
    unsorted_images_dir: &std::path::Path,
    configuration: &GlobalConfiguration
) -> Result<()> {
    log::trace!("sort_images_of_dir in {:?}", dir);

    let files = directories::get_files_from_dir(dir)?;
    let bar = ProgressBar::new(files.len().try_into().unwrap());
    for file in files {
        let r_exif_data = exif::get_exif_data(&file);
        match r_exif_data {
            Ok(exif_data) => match sort_image_from_exif_data(&file, exif_data, target_dir, configuration) {
                Ok(()) => {
                    log::trace!("Image {:?} processed...", file);
                    Reporting::image_processed_sorted();
                }
                Err(e) => {
                    log::error!("Error {:?} when processing image {:?} ...", e, file);
                    Reporting::error_on_image();
                    eprintln!("Error {} when processing image {:?} ...", e, file)
                }
            },
            Err(e) => match e {
                ExifError::IO(io) => {
                    log::error!("Error {:?} when processing image {:?} ...", io, file);
                    Reporting::error_on_image();
                    eprintln!("Error {} when processing image {:?} ...", io, file)
                }
                ExifError::NotImageFile(s) => {
                    log::warn!("{} is not an image. {}", file.display(), s)
                }
                ExifError::Decoding(s) => {
                    log::error!("Error {:?} when decoding exif_data of file {:?}", s, file);
                    match copy_unsorted_image_in_specific_dir(&file, unsorted_images_dir) {
                        Ok(()) => {
                            Reporting::image_processed_unsorted();
                            log::trace!(
                                "Image {:?} processed (no Exif Data -> copied in unsorted dir)...",
                                file
                            )
                        }
                        Err(e) => {
                            log::error!("Error {:?} when processing image {:?} ...", e, file);
                            Reporting::error_on_image();
                            eprintln!("Error {} when processing image {:?} ...", e, file)
                        }
                    }
                }
                ExifError::NoExifData => {
                    log::warn!("Warning: {:?} when getting exif_data of file {:?}", e, file);
                    match copy_unsorted_image_in_specific_dir(&file, unsorted_images_dir) {
                        Ok(()) => {
                            Reporting::image_processed_unsorted();
                            log::trace!(
                                "Image {:?} processed (no Exif Data -> copied in unsorted dir)...",
                                file
                            )
                        }
                        Err(e) => {
                            log::error!("Error {:?} when processing image {:?} ...", e, file);
                            Reporting::error_on_image();
                            eprintln!("Error {} when processing image {:?} ...", e, file)
                        }
                    }
                }
            },
        }
        bar.inc(1);
    }
    Ok(())
}

fn sort_image_from_exif_data(
    file: &std::path::Path,
    exif_data: ExifData,
    target_dir: &std::path::Path,
    configuration: &GlobalConfiguration
) -> Result<()> {
    log::trace!(
        "sort_image_from_exif_data file: {:?} exif_data: {:?}",
        file,
        exif_data
    );
    let new_directory_path = std::path::Path::new(exif_data.year_month.get());
    let new_directory_path_buf = directories::create_subdir(target_dir, new_directory_path)?;
    let new_directory_path = std::path::Path::new(exif_data.place.get());
    let mut new_directory_path_buf =
        directories::create_subdir(new_directory_path_buf.as_path(), new_directory_path)?;
    
    if *configuration.use_device() {
        let new_directory_path = std::path::Path::new(exif_data.device.get());
        new_directory_path_buf =
            directories::create_subdir(new_directory_path_buf.as_path(), new_directory_path)?;
    }


    let p = new_directory_path_buf.as_path();
    // unwrap() is ok here, the file have been checked as a file before
    let pb = p.join(std::path::Path::new(&file.file_name().unwrap()));
    let checked = check_for_duplicate_and_rename(pb.as_path())?;
    
    if let Some(deduplicate_path_name) = checked {
        fs::copy(file, deduplicate_path_name.as_path())?;
    } else {
        fs::copy(file, pb.as_path())?;
    }

    Ok(())
}

fn copy_unsorted_image_in_specific_dir(
    file: &std::path::Path,
    unsorted_dir: &std::path::Path,
) -> Result<()> {
    log::trace!(
        "copy_unsorted_image_in_specific_dir file: {:?}, unsorted_dir: {:?}",
        file,
        unsorted_dir
    );
    let mut copied_filename = String::from(unsorted_dir.to_str().unwrap());
    copied_filename.push_str(file.to_str().unwrap());
    log::debug!("file: {:?} ; copied filename: {:?}", file, copied_filename);
    let copied_file = std::path::Path::new(&copied_filename);
    fs::DirBuilder::new()
        .recursive(true)
        .create(copied_file.parent().unwrap())?;
    
    fs::copy(file, copied_file)?;
   
    Ok(())
}

/// verify if there is already a file pointed by the path. If so, return a new path
fn check_for_duplicate_and_rename(file: &Path) -> Result<Option<PathBuf>> {
    log::trace!("check_for_duplicate_and_rename {:?}", file);
    if file.is_dir() {
        log::error!("Error when checking for duplication in target directory");
        eprintln!("Error when checking for duplication in target directory");
        return Err(eyre::eyre!("Error when checking for duplication in target directory"));
    }
    
    let test = file.try_exists()?;
    
    if test == true {
        
        let path: &Path = file.as_ref();
        let mut new_path = path.to_owned();
        let mut new_filename = String::new();
        new_filename.push_str(&file.file_stem().unwrap().to_string_lossy());
        new_filename.push_str("_duplicate_");
        new_filename.push_str(&rand::Rng::gen_range(&mut rand::thread_rng(), 100..255).to_string());

        new_path.set_file_name(new_filename);
        if let Some(ext) = path.extension() {
            new_path.set_extension(ext);
        }
        
        Ok(Some(new_path))
    } else {
        Ok(None)
    }
}

//TODO Tests
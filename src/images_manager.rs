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
    configuration: &GlobalConfiguration,
) -> Result<()> {
    log::trace!("sort_images_of_dir in {:?}", dir);

    let files = directories::get_files_from_dir(dir)?;
    let bar = ProgressBar::new(files.len().try_into().unwrap());
    for file in files {
        let r_exif_data = exif::get_exif_data(&file);
        match r_exif_data {
            Ok(exif_data) => {
                match sort_image_from_exif_data(&file, &exif_data, target_dir, configuration) {
                    Ok(()) => {
                        log::trace!("Image {:?} processed...", file);
                        Reporting::image_processed_sorted();
                    }
                    Err(e) => {
                        log::error!("Error {:?} when processing image {:?} ...", e, file);
                        Reporting::error_on_image();
                        eprintln!("Error {} when processing image {:?} ...", e, file)
                    }
                }
            }
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

// TODO use configuration in place of target_dir?
fn sort_image_from_exif_data(
    file: &std::path::Path,
    exif_data: &ExifData,
    target_dir: &std::path::Path,
    configuration: &GlobalConfiguration,
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
    let p = unsorted_dir.join(file);
    fs::DirBuilder::new()
        .recursive(true)
        .create(p.as_path().parent().unwrap())?;

    log::debug!("file: {:?} to: {:?}", file, p.as_path());
    fs::copy(file, p.as_path())?;

    Ok(())
}

/// verify if there is already a file pointed by the path. If so, return a new path
fn check_for_duplicate_and_rename(file: &Path) -> Result<Option<PathBuf>> {
    log::trace!("check_for_duplicate_and_rename {:?}", file);
    if file.is_dir() {
        log::error!("Error when checking for duplication in target directory");
        eprintln!("Error when checking for duplication in target directory");
        return Err(eyre::eyre!(
            "Error when checking for duplication in target directory"
        ));
    }

    let test = file.try_exists()?;

    if test == true {
        let path: &Path = file.as_ref();
        let mut new_path = path.to_owned();
        let mut new_filename = String::new();
        new_filename.push_str(&file.file_stem().unwrap().to_string_lossy());
        new_filename.push_str("_duplicate_");
        new_filename.push_str(&rand::Rng::gen_range(&mut rand::thread_rng(), 100..4096).to_string());

        new_path.set_file_name(new_filename);
        if let Some(ext) = path.extension() {
            new_path.set_extension(ext);
        }

        Ok(Some(new_path))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use crate::exif::Directory;

    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_check_for_duplicate_and_rename() {
        init();
        let current_dir = std::env::current_dir().unwrap();
        std::fs::create_dir("./test_check_dir").unwrap();
        let path = std::path::Path::new("./test_check_dir/foo.txt");
        fs::write(path, "Lorem ipsum").unwrap();

        let result = check_for_duplicate_and_rename(path).unwrap();
        assert!(result.is_some());

        let path_2 = std::path::Path::new("./test_check_dir/foo_2.txt");
        let result = check_for_duplicate_and_rename(path_2).unwrap();
        assert!(result.is_none());

        // ensure we are in the good directory before cleanup
        assert_eq!(current_dir, std::env::current_dir().unwrap());
        // cleanup
        std::fs::remove_dir_all("./test_check_dir").unwrap();
    }

    #[test]
    fn test_copy_unsorted_image_in_specific_dir() {
        init();
        let current_dir = std::env::current_dir().unwrap();
        let dir = std::path::Path::new("./test_cp_unsorted");
        std::fs::create_dir(dir).unwrap();
        let file = std::path::Path::new("foo_test.txt");
        fs::write(file, "Lorem ipsum").unwrap();

        copy_unsorted_image_in_specific_dir(file, dir).unwrap();
        let copied_file = std::path::Path::new("./test_cp_unsorted/foo_test.txt");
        assert!(copied_file.exists());

        // ensure we are in the good directory before cleanup
        assert_eq!(current_dir, std::env::current_dir().unwrap());
        // cleanup
        std::fs::remove_dir_all(dir).unwrap();
        std::fs::remove_file(file).unwrap();
    }

    #[test]
    fn test_sort_image_from_exif_data() {
        init();
        let current_dir = std::env::current_dir().unwrap();
        let dir_target = std::path::Path::new("./test_sort_image");
        std::fs::create_dir(dir_target).unwrap();

        let mut configuration = GlobalConfiguration::new();
        *configuration.use_device_mut() = false;
        *configuration.source_directory_mut() = PathBuf::from("./");
        *configuration.dest_directory_mut() = PathBuf::from(dir_target);

        let exif_data = ExifData {
            year_month: Directory::parse(String::from("2023 10")),
            gps_lat: 0.0,
            gps_long: 0.0,
            place: Directory::parse(String::from("Null_Island")),
            device: Directory::parse(String::from("Nikkon")),
        };

        sort_image_from_exif_data(
            Path::new("./data_4_tests/DSCN0025.jpg"),
            &exif_data,
            dir_target,
            &configuration,
        )
        .unwrap();
        let copied_file = std::path::Path::new("./test_sort_image/2023 10/Null_Island/DSCN0025.jpg");
        assert!(copied_file.exists());

        *configuration.use_device_mut() = true;

        sort_image_from_exif_data(
            Path::new("./data_4_tests/DSCN0025.jpg"),
            &exif_data,
            dir_target,
            &configuration,
        )
        .unwrap();
        let copied_file =
            std::path::Path::new("./test_sort_image/2023 10/Null_Island/Nikkon/DSCN0025.jpg");
        assert!(copied_file.exists());

        // ensure we are in the good directory before cleanup
        assert_eq!(current_dir, std::env::current_dir().unwrap());
        // cleanup
        std::fs::remove_dir_all(dir_target).unwrap();
    }

    #[test]
    fn test_sort_images_in_dir() {
        init();
        let current_dir = std::env::current_dir().unwrap(); 
        let mut configuration = GlobalConfiguration::new();
        *configuration.use_device_mut() = false;
        *configuration.source_directory_mut() = PathBuf::from("./");
        *configuration.dest_directory_mut() = PathBuf::from("test_sort_images");

        let source_dir = std::path::Path::new("data_4_tests");
        let dir_target = std::path::Path::new("test_sort_images");
        let unsorted_dir = std::path::Path::new("test_sort_images/unsorted");
        
        sort_images_in_dir(source_dir, dir_target, unsorted_dir, &configuration).unwrap();
        assert_eq!(4, fs::read_dir("test_sort_images/2008 10/Arezzo")
            .unwrap()
            .map(|r| r.unwrap().path())
            .filter(|r| r.is_file()).collect::<Vec<PathBuf>>().len());
        
        // ensure we are in the good directory before cleanup
        assert_eq!(current_dir, std::env::current_dir().unwrap());
        // cleanup
        std::fs::remove_dir_all(dir_target).unwrap();
    }

}

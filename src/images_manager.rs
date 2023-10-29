use std::fs;

use crate::directories;
use crate::exif;
use crate::exif::ExifData;
use eyre::Result;

pub fn sort_images_in_dir(
    dir: &std::path::Path,
    target_dir: &std::path::Path,
    unsorted_images_dir: &std::path::Path,
) -> Result<()> {
    log::trace!("sort_images_of_dir in {:?}", dir);

    let files = directories::get_files_from_dir(dir)?;

    for file in files {
        let r_exif_data = exif::get_exif_data(&file);
        match r_exif_data {
            Ok(exif_data) => match sort_image_from_exif_data(&file, exif_data, target_dir) {
                Ok(()) => log::trace!("Image {:?} processed...", file),
                Err(e) => log::error!("Error {} when processing image {:?} ...", e, file),
            },
            Err(e) => {
                log::warn!("Error {:?} when getting exif_data of file {:?}", e, file);
                match copy_unsorted_image_in_specific_dir(&file, unsorted_images_dir) {
                    Ok(()) => log::trace!(
                        "Image {:?} processed (no Exif Data -> copied in unsorted dir)...",
                        file
                    ),
                    Err(e) => log::error!("Error {} when processing image {:?} ...", e, file),
                }
            }
        }
    }
    Ok(())
}

fn sort_image_from_exif_data(
    file: &std::path::Path,
    exif_data: ExifData,
    target_dir: &std::path::Path,
) -> Result<()> {
    log::trace!(
        "sort_image_from_exif_data file: {:?} exif_data: {:?}",
        file,
        exif_data
    );
    let new_directory_path = std::path::Path::new(exif_data.year_month.get());
    let new_directory_path_buf = directories::create_subdir(target_dir, new_directory_path)?;
    let new_directory_path = std::path::Path::new(exif_data.place.get());
    let new_directory_path_buf =
        directories::create_subdir(new_directory_path_buf.as_path(), new_directory_path)?;
    let new_directory_path = std::path::Path::new(exif_data.device.get());
    let new_directory_path_buf =
        directories::create_subdir(new_directory_path_buf.as_path(), new_directory_path)?;

    let mut new_path_name: String = String::from(new_directory_path_buf.to_str().unwrap());
    new_path_name.push('/');
    new_path_name.push_str(file.file_name().unwrap().to_str().unwrap());

    fs::copy(file, std::path::Path::new(&new_path_name))?;

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
    log::debug!("copied filename: {:?}", copied_filename);
    copied_filename.push_str(file.to_str().unwrap());
    log::debug!("file: {:?} ; copied filename: {:?}", file, copied_filename);
    let copied_file = std::path::Path::new(&copied_filename);
    fs::DirBuilder::new()
        .recursive(true)
        .create(copied_file.parent().unwrap())?;
    fs::copy(file, copied_file)?;
    Ok(())
}
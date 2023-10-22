use clap::Parser;
use exif_images::ExifData;
use std::fs;

mod directories;
mod exif_images;
mod place_finder;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Source Directory (where the photos to manage are)
    #[arg(short, long)]
    source_dir: String,
    /// Destination Directory (where to copy the sorted images). Default : in the source directory
    #[arg(short, long)]
    dest_dir: Option<String>,
}

// TODO Args : dest dir
// TODO number of images in directories / number of images processed

fn main() {
    env_logger::init();
    let args = Args::parse();
    log::info!("Launching image_sort -- args : {:?}", args);

    log::info!("Screening all directories in source directory ...");
    let top_directory = &args.source_dir;
    let top_directory = std::path::Path::new(top_directory);
    let mut all_directories = directories::get_subdirectories_recursive(top_directory).unwrap();
    all_directories.push(std::path::PathBuf::from(top_directory));

    log::info!("Create target directory ...");

    let target = match args.dest_dir {
        Some(dest_dir) => directories::create_sorted_images_dir(std::path::Path::new(&dest_dir)),
        None => directories::create_sorted_images_dir(&top_directory),
    };

    let target = match target {
        Ok(path) => {
            log::info!("target directory is {:?}...", &path);
            path
        }
        Err(e) => {
            log::error!("target directory creation failed : {:?}", e);
            return;
        }
    };

    let unsorted_dir = directories::create_unsorted_images_dir(&target).unwrap();

    // initialisation of reverse geocoder (static variable to avoid multiple loading of data)
    let _ = place_finder::LocationsWrapper::init().unwrap();
    let _ = place_finder::ReverseGeocoderWrapper::init().unwrap();

    for dir in &all_directories {
        log::debug!("{:?}", dir);
        match sort_images_of_dir(dir, &target, &unsorted_dir) {
            Err(e) => log::error!("Error {} when processing images in {:?}.", e, dir),
            _ => log::info!("Images in {:?} processed...", dir),
        }
    }
}

fn sort_images_of_dir(
    dir: &std::path::Path,
    target_dir: &std::path::Path,
    unsorted_images_dir: &std::path::Path,
) -> Result<(), anyhow::Error> {
    log::trace!("sort_images_of_dir in {:?}", dir);

    let files = directories::get_files_from_dir(dir)?;

    for file in files {
        let r_exif_data = exif_images::get_exif_data(&file);
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
) -> Result<(), anyhow::Error> {
    log::trace!(
        "sort_image_from_exif_data file: {:?} exif_data: {:?}",
        file,
        exif_data
    );
    let new_directory_path = std::path::Path::new(exif_data.year_month.get());
    let new_directory_path_buf = directories::create_subdir(target_dir, new_directory_path)?;
    let new_directory_path = std::path::Path::new(exif_data.place.get());
    let new_directory_path_buf =
        directories::create_subdir(&new_directory_path_buf.as_path(), new_directory_path)?;
    let new_directory_path = std::path::Path::new(exif_data.device.get());
    let new_directory_path_buf =
        directories::create_subdir(&new_directory_path_buf.as_path(), new_directory_path)?;

    let mut new_path_name: String = String::from(new_directory_path_buf.to_str().unwrap());
    new_path_name.push('/');
    new_path_name.push_str(file.file_name().unwrap().to_str().unwrap());

    fs::copy(file, std::path::Path::new(&new_path_name))?;

    Ok(())
}

fn copy_unsorted_image_in_specific_dir(
    file: &std::path::Path,
    unsorted_dir: &std::path::Path,
) -> Result<(), anyhow::Error> {
    log::trace!("copy_unsorted_image_in_specific_dir file: {:?}, unsorted_dir: {:?}", file, unsorted_dir);
    let mut copied_filename = String::from(unsorted_dir.to_str().unwrap());
    log::debug!("copied filename: {:?}", copied_filename);
    copied_filename.push_str(file.to_str().unwrap());
    log::debug!("file: {:?} ; copied filename: {:?}", file, copied_filename);
    let copied_file = std::path::Path::new(&copied_filename);
    fs::DirBuilder::new().recursive(true).create(&copied_file.parent().unwrap())?;
    fs::copy(file, copied_file)?;
    Ok(())
}

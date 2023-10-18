use std::fs;
use clap::Parser;
use images::ExifData;

mod directories;
mod images;
mod reverse_gps;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Directory
    #[arg(short, long)]
    dir: String,
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    log::info!("Launching image_sort -- args : {:?}", args);

    log::info!("Screening directories ...");
    let top_directory = &args.dir;
    let target_directory = std::path::Path::new(top_directory);
    let mut all_directories = directories::get_subdirectories_recursive(target_directory).unwrap();
    all_directories.push(std::path::PathBuf::from(top_directory));

    log::info!("Create target directory ...");
    let target = directories::create_sorted_images_dir(&target_directory);

    match target {
        Ok(path) => log::info!("target directory is {:?}...", &path),
        Err(e) => {
            log::error!("target directory creation failed : {:?}", e);
            return;
        }
    }

    let _ = reverse_gps::LocationsWrapper::init().unwrap();
    let _ = reverse_gps::ReverseGeocoderWrapper::init().unwrap();

    for dir in &all_directories {
        log::debug!("{:?}", dir);
        match sort_images_of_dir(dir, &target_directory) {
            Err(e) => log::error!("Error {} when processing images in {:?}.", e, dir),
            _ => log::info!("Images in {:?} processed...", dir),
        }
    }
}

fn sort_images_of_dir(dir: &std::path::Path, target_dir: &std::path::Path) -> Result<(), anyhow::Error> {
    log::trace!("sort_images_of_dir in {:?}", dir);

    let files = directories::get_files_from_dir(dir)?;

    for file in files {
        let r_exif_data = images::get_exif_data(&file);
        match r_exif_data {
            Ok(exif_data) => match sort_image_from_exif_data(&file, exif_data, target_dir) {
                Ok(()) => log::trace!("Image {:?} processed...", file),
                Err(e) => log::error!("Error {} when processing image {:?} ...", e, file),
            },
            Err(e) => {
                log::error!("Error {} when getting exif_data of file {:?}", e, file)
            }
        }
    }
    Ok(())
}

fn sort_image_from_exif_data(
    file: &std::path::Path,
    exif_data: ExifData,
    target_dir: &std::path::Path
) -> Result<(), anyhow::Error> {
    log::trace!(
        "sort_image_from_exif_data file: {:?} exif_data: {:?}",
        file,
        exif_data
    );
    let new_directory_path = std::path::Path::new(&exif_data.year);
    let new_directory_path_buf = directories::create_subdir(target_dir, new_directory_path)?;
    let new_directory_path = std::path::Path::new(&exif_data.place);
    let new_directory_path_buf = directories::create_subdir(&new_directory_path_buf.as_path(), new_directory_path)?;
    let new_directory_path = std::path::Path::new(&exif_data.device);
    let new_directory_path_buf = directories::create_subdir(&new_directory_path_buf.as_path(), new_directory_path)?;
    
    let mut new_path_name: String = String::from(new_directory_path_buf.to_str().unwrap());
    new_path_name.push_str(file.file_name().unwrap().to_str().unwrap());

    fs::copy(file, std::path::Path::new(&new_path_name))?;

    Ok(())
}

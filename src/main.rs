use clap::Parser;

mod directories;
mod exif;
mod images_manager;
mod place_finder;
mod reporting;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Source Directory (where the photos to manage are)
    #[arg(short, long)]
    source_dir: String,
    /// Destination Directory (where to copy the sorted images). Default : in the source directory
    #[arg(short, long)]
    dest_dir: Option<String>,
    /// Use Device (Camera Model) to sort
    #[arg(short, long)]
    use_device: Option<bool>,// TODO use it
}


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
        None => directories::create_sorted_images_dir(top_directory),
    };

    let target = match target {
        Ok(path) => {
            log::info!("target directory is {:?}...", &path);
            path
        }
        Err(e) => {
            log::error!(
                "target directory creation failed : {:?}, ending execution",
                e
            );
            return;
        }
    };

    let unsorted_dir = directories::create_unsorted_images_dir(&target).unwrap();

    // initialisation of reverse geocoder (static variable to avoid multiple loading of data)
    //place_finder::LocationsWrapper::init();
    //place_finder::ReverseGeocoderWrapper::init();

    for dir in &all_directories {
        log::debug!("{:?}", dir);
        match images_manager::sort_images_in_dir(dir, &target, &unsorted_dir) {
            Err(e) => log::error!(
                "Unexpected error {} when processing images in {:?}.",
                e,
                dir
            ),
            _ => log::info!("Images in {:?} processed...", dir),
        }
    }
}

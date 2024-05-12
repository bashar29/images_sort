use clap::Parser;
use indicatif::ProgressBar;

use crate::{global_configuration::GlobalConfiguration, reporting::Reporting};

mod directories;
mod exif;
mod global_configuration;
mod images_manager;
mod place_finder;
mod reporting;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Source Directory (where are the photos to sort)
    #[arg(short, long)]
    source_dir: String,
    /// Destination Directory (where to copy the sorted images). Default : in the current directory
    #[arg(short, long, default_value_t = '.'.to_string())]
    dest_dir: String,
    /// Use Device (Camera Model) as a key to sort
    #[arg(short, long)]
    use_device: Option<bool>,
}

fn main() {
    env_logger::init();
    let mut configuration = GlobalConfiguration::new();
    let args = Args::parse();
    log::info!("Launching image_sort -- args : {:?}", args);

    println!("Screening all directories in source directory ...");

    let top_directory = &args.source_dir;
    let top_directory = std::path::Path::new(top_directory);
    *configuration.source_directory_mut() = top_directory.to_path_buf();

    // match args.dest_dir {
    //     Some(dest_dir) => *configuration.dest_directory_mut() = std::path::PathBuf::from(dest_dir),
    //     None => *configuration.dest_directory_mut() = configuration.source_directory().clone(),
    // }
    *configuration.dest_directory_mut() = std::path::PathBuf::from(&args.dest_dir);

    if let Some(d) = args.use_device {
        *configuration.use_device_mut() = d;
    }

    let mut all_directories =
        match directories::get_subdirectories_recursive(configuration.source_directory_as_path()) {
            Ok(d) => d,
            Err(e) => {
                log::error!(
                    "Error {:?} when listing all subdirectories from {:?}",
                    e,
                    top_directory
                );
                eprintln!(
                    "Error : {} when listing all subdirectories from {}",
                    e,
                    top_directory.display()
                );
                std::process::exit(1)
            }
        };

    all_directories.push(configuration.source_directory().clone());

    println!("Create target directory ...");

    let sorted_dir =
        match directories::create_sorted_images_dir(configuration.dest_directory_as_path()) {
            Ok(path) => path,
            Err(e) => {
                log::error!(
                    "Error when creating the target directory : {:?}, ending execution",
                    e
                );
                eprintln!("Error when creating the target directory : {}, ending execution", e);
                std::process::exit(1)
            }
        };
    *configuration.sorted_images_directory_mut() = sorted_dir;

    let unsorted_dir =
        directories::create_unsorted_images_dir(configuration.sorted_images_directory_as_path())
            .unwrap();
    *configuration.unsorted_images_directory_mut() = unsorted_dir;

    println!("Sorting images ...");

    let bar = ProgressBar::new(all_directories.len().try_into().unwrap());
    for dir in &all_directories {
        log::debug!("{:?}", dir);
        match images_manager::sort_images_in_dir(
            dir,
            &configuration,
        ) {
            Err(e) => {
                log::error!(
                    "Unexpected error {:?} when processing images in {:?}.",
                    e,
                    dir
                );
                eprintln!(
                    "Unexpected error {} when processing images in {:?}.",
                    e, dir
                )
            }
            _ => {
                Reporting::directory_processed();
                println!("Images in {:?} processed...", dir);
            }
        }
        bar.inc(1);
    }
    Reporting::print_reporting();
}

use clap::Parser;

use crate::reporting::Reporting;

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
    use_device: Option<bool>, // TODO use it
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    log::info!("Launching image_sort -- args : {:?}", args);

    println!("Screening all directories in source directory ...");

    let top_directory = &args.source_dir;
    let top_directory = std::path::Path::new(top_directory);
    //let mut all_directories = directories::get_subdirectories_recursive(top_directory).unwrap();
    let mut all_directories = match directories::get_subdirectories_recursive(top_directory) {
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

    all_directories.push(std::path::PathBuf::from(top_directory));

    println!("Create target directory ...");

    let target = match args.dest_dir {
        Some(dest_dir) => directories::create_sorted_images_dir(std::path::Path::new(&dest_dir)),
        None => directories::create_sorted_images_dir(top_directory),
    };

    let target = match target {
        Ok(path) => path,
        Err(e) => {
            log::error!(
                "target directory creation failed : {:?}, ending execution",
                e
            );
            eprintln!("target directory creation failed : {}, ending execution", e);
            std::process::exit(1)
        }
    };

    let unsorted_dir = directories::create_unsorted_images_dir(&target).unwrap();

    println!("Sorting images ...");

    for dir in &all_directories {
        log::debug!("{:?}", dir);
        match images_manager::sort_images_in_dir(dir, &target, &unsorted_dir) {
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
    }
    Reporting::print_reporting();
}

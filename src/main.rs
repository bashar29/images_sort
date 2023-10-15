use clap::Parser;
use anyhow::anyhow;

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
    let path = std::path::Path::new(top_directory);
    let mut all_directories = directories::get_subdirectories_recursive(path).unwrap();
    all_directories.push(std::path::PathBuf::from(top_directory));

    log::info!("Create target directory ...");
    let target = directories::create_sorted_images_dir(&path);
    
    match target {
        Ok(path) => log::info!("target directory is {:?}...", &path),
        Err(e) => {
            log::error!("target directory creation failed : {:?}", e);
            return 
        }
    }
    
    let _ = reverse_gps::LocationsWrapper::init().unwrap();
    let _ = reverse_gps::ReverseGeocoderWrapper::init().unwrap();
    
    for dir in &all_directories {
        log::debug!("{:?}", dir);
    
    }
    
}

fn sort_images_of_dir(dir: &std::path::Path) -> Result<(), anyhow::Error> {
    log::trace!("sort_images_of_dir in {:?}", dir);

    let files = directories::get_files_from_dir(dir);
    
    if let Err(e) = files {
        log::error!("failed to get files in dir {:?} : {:?}", dir, e);
        return Err(anyhow!("failed to get files. Error : {}", e)); 
    }

    for file in files {


    }

    Ok(())
}
use clap::Parser;

mod directories;
mod images;

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
    
    for dir in &all_directories {
        log::info!("{:?}", dir);
    }
}

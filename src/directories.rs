use std::{
    fs::{self, DirBuilder},
    io,
    path::{Path, PathBuf},
};

pub fn get_subdirectories_recursive(top_directory: &Path) -> Result<Vec<PathBuf>, io::Error> {
    log::trace!("{:?}", top_directory);
    let directories: Vec<PathBuf> = Vec::new();
    let sub_dir = get_subdirectories(top_directory)?;
    let mut directories = [directories, sub_dir.clone()].concat();
    for d in &sub_dir {
        directories.append(&mut get_subdirectories_recursive(d.as_path())?);
    }

    Ok(directories)
}

fn get_subdirectories(top_directory: &Path) -> Result<Vec<PathBuf>, io::Error> {
    log::trace!("{:?}", top_directory);
    Ok(fs::read_dir(top_directory)?
        .into_iter()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap().path())
        .filter(|r| r.is_dir())
        .collect())
}

pub fn create_sorted_images_dir(top_directory: &Path) -> Result<PathBuf, io::Error> {
    log::trace!("{:?}", top_directory);
    let now = chrono::Local::now();
    log::debug!("now == {}",now);
    //let suffix = now.format("%Y%m%d-%H%M%S").to_string();
    let suffix = now.format("%Y%m%d-%H").to_string();
    let dirname = format!("Sorted_Images-{}", suffix);
    log::info!("new directory name : {}", dirname);
    let path = top_directory.join(dirname);
    DirBuilder::new().recursive(false).create(&path)?;
    Ok(path)
}

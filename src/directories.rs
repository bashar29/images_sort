use std::{
    fs::{self, DirBuilder},
    io,
    path::{Path, PathBuf},
};

pub fn get_subdirectories_recursive(top_directory: &Path) -> Result<Vec<PathBuf>, io::Error> {
    log::trace!("get_subdirectories_recursive of {:?}", top_directory);
    let directories: Vec<PathBuf> = Vec::new();
    let sub_dir = get_subdirectories(top_directory)?;
    let mut directories = [directories, sub_dir.clone()].concat();
    for d in &sub_dir {
        directories.append(&mut get_subdirectories_recursive(d.as_path())?);
    }

    Ok(directories)
}

fn get_subdirectories(top_directory: &Path) -> Result<Vec<PathBuf>, io::Error> {
    log::trace!("get_subdirectories of {:?}", top_directory);
    Ok(fs::read_dir(top_directory)?
        .into_iter()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap().path())
        .filter(|r| r.is_dir())
        .collect())
}

pub fn create_sorted_images_dir(top_directory: &Path) -> Result<PathBuf, io::Error> {
    log::trace!("create_sorted_images_dir in {:?}", top_directory);
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

pub fn create_subdir(parent_directory: &Path, sub_dir: &Path) -> Result<PathBuf, io::Error> {
    log::trace!("create_subdir in {:?}", parent_directory);
    let new_dir = parent_directory.join(sub_dir);
    DirBuilder::new().recursive(true).create(&new_dir)?;
    // TODO : check behaviour when dir already exists
    Ok(new_dir)
}

pub fn get_files_from_dir(dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    log::trace!("get_images_from_dir in {:?}", dir);
    Ok(fs::read_dir(dir)?
        .into_iter()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap().path())
        .filter(|r| r.is_file())
        .collect())
}

#[cfg(test)]
mod tests {
    //use super::*;
    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_create_subdir() {
        init();
    }
}

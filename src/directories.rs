//! # directories
//!
//! Functions to manage interactions with the filesystem.
use std::{
    fs::{self, DirBuilder},
    io,
    path::{Path, PathBuf},
};
use eyre::Result;

// TODO : check IO Error bubbling

const SORTED_IMAGES_DIRNAME_PREFIX: &str = "Images-";
const UNSORTED_IMAGES_SUBDIR_NAME: &str = "Unsorted/";

/// Get all subdirectories of a directory, recursively dig in all directories
pub fn get_subdirectories_recursive(top_directory: &Path) -> Result<Vec<PathBuf>> {
    log::trace!("get_subdirectories_recursive of {:?}", top_directory);
    let directories: Vec<PathBuf> = Vec::new();
    let sub_dir = get_subdirectories(top_directory)?;
    let mut directories = [directories, sub_dir.clone()].concat();
    for d in &sub_dir {
        directories.append(&mut get_subdirectories_recursive(d.as_path())?);
    }

    Ok(directories)
}

fn get_subdirectories(top_directory: &Path) -> Result<Vec<PathBuf>> {
    log::trace!("get_subdirectories of {:?}", top_directory);
    Ok(fs::read_dir(top_directory)?
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap().path())
        .filter(|r| r.is_dir())
        .collect())
}

/// Create the directory where the sorted images will be copied.
/// The name will embed info of the timestamp of the creation.
pub fn create_sorted_images_dir(top_directory: &Path) -> Result<PathBuf> {
    log::trace!("create_sorted_images_dir in {:?}", top_directory);
    let now = chrono::Local::now();
    log::debug!("now == {}", now);
    //let suffix = now.format("%Y%m%d-%H%M%S").to_string();
    let suffix = now.format("%Y%m%d-%H%M%S").to_string();
    let dirname = format!("{}{}", SORTED_IMAGES_DIRNAME_PREFIX, suffix);
    log::info!("new directory name : {}", dirname);
    let path = top_directory.join(dirname);
    log::debug!("path of target directory to be created : {:?}", path);
    DirBuilder::new().recursive(false).create(&path)?;
    Ok(path)
}

/// Create the directory where images that couldn't be sorted (because they lack of EXIF Data)
/// will be copied
pub fn create_unsorted_images_dir(parent_directory: &Path) -> Result<PathBuf> {
    log::trace!("create_unsorted_images_dir in {:?}", parent_directory);
    let unsorted_images_dir = parent_directory.join(std::path::Path::new(&String::from(
        UNSORTED_IMAGES_SUBDIR_NAME,
    )));
    DirBuilder::new()
        .recursive(true)
        .create(&unsorted_images_dir)?;
    Ok(unsorted_images_dir)
}

pub fn create_subdir(parent_directory: &Path, sub_dir: &Path) -> Result<PathBuf> {
    log::trace!("create_subdir in {:?}", parent_directory);
    let new_dir = parent_directory.join(sub_dir);
    DirBuilder::new().recursive(true).create(&new_dir)?;
    // TODO : check behaviour when dir already exists
    Ok(new_dir)
}

/// Return a Vec containing all FILES contained in a directory
pub fn get_files_from_dir(dir: &Path) -> Result<Vec<PathBuf>> {
    log::trace!("get_images_from_dir in {:?}", dir);
    Ok(fs::read_dir(dir)?
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap().path())
        .filter(|r| r.is_file())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_create_subdir() {
        init();
        assert_eq!(Path::new("./new_dir").try_exists().unwrap(), false);
        let result = create_subdir(
            std::path::Path::new(&String::from("./")),
            std::path::Path::new(&String::from("new_dir")),
        );
        let dir = result.unwrap();
        assert!(dir.is_dir());

        std::fs::remove_dir(dir.as_path()).unwrap();
    }

    #[test]
    fn test_get_files_from_dir() {
        init();
        let current_dir = std::env::current_dir().unwrap();
        std::fs::create_dir("./this_dir").unwrap();
        std::fs::File::create("./this_dir/foo1.txt").unwrap();
        std::fs::File::create("./this_dir/foo2.txt").unwrap();
        std::fs::File::create("./this_dir/foo3.txt").unwrap();
        let files = get_files_from_dir(std::path::Path::new(&String::from("./this_dir"))).unwrap();
        assert_eq!(files.len(), 3);

        // ensure we are in the good directory before cleaning this_dir.
        assert_eq!(current_dir, std::env::current_dir().unwrap());
        std::fs::remove_dir_all("./this_dir").unwrap();
    }
}

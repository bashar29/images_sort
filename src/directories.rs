use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub fn get_subdirectories_recursive(top_directory: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let directories: Vec<PathBuf> = Vec::new();
    let sub_dir = get_subdirectories(top_directory)?;
    let mut directories = [directories, sub_dir.clone()].concat();
    for d in &sub_dir {
        directories.append(&mut get_subdirectories_recursive(d.as_path())?);
    }

    Ok(directories)
}

fn get_subdirectories(top_directory: &Path) -> Result<Vec<PathBuf>, io::Error> {
    log::debug!("{:?}", top_directory);
    Ok(fs::read_dir(top_directory)?
        .into_iter()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap().path())
        .filter(|r| r.is_dir())
        .collect())
}

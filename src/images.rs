// fn sort_images(top_directory: &Path) -> Result<Vec<PathBuf>, io::Error> {
//     log::debug!("{:?}", top_directory);
//     Ok(fs::read_dir(top_directory)?
//         .into_iter()
//         .filter(|r| r.is_ok())
//         .map(|r| r.unwrap().path())
//         .filter(|r| r.is_dir())
//         .collect())
// }

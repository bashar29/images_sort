use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use crate::directories;
use crate::exif;
use crate::exif::ExifData;
use crate::exif::ExifError;
use crate::global_configuration::GlobalConfiguration;
use crate::performance::{PerformanceMetrics, Timer};
use crate::reporting::Reporting;
use eyre::Result;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

pub fn sort_images_in_dir(
    dir: &std::path::Path,
    configuration: &GlobalConfiguration,
) -> Result<()> {
    log::trace!("sort_images_of_dir in {:?}", dir);

    let files = directories::get_files_from_dir(dir)?;
    let bar = ProgressBar::new(files.len().try_into().unwrap());
    bar.set_style(
        ProgressStyle::default_bar()
            .template("  {spinner:.blue} [{bar:30.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("━━╾─"),
    );

    // Wrap progress bar in Arc for sharing across threads
    let bar = Arc::new(bar);

    // Configure rayon thread pool to use moderate parallelism (good for NAS HDD)
    // Limit to 4 threads to avoid disk thrashing
    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .ok(); // Ignore error if already initialized

    // Process files in parallel
    files.par_iter().for_each(|file| {
        bar.set_message(format!("{}", file.file_name().unwrap_or_default().to_string_lossy()));

        let r_exif_data = exif::get_exif_data(file);
        match r_exif_data {
            Ok(exif_data) => {
                // Collect statistics
                Reporting::add_place(exif_data.place.get().to_string());
                Reporting::add_device(exif_data.device.get().to_string());
                Reporting::update_date_range(exif_data.year_month.get());

                match sort_image_from_exif_data(file, &exif_data, configuration) {
                    Ok(()) => {
                        log::trace!("Image {:?} processed...", file);
                        Reporting::image_processed_sorted();
                    }
                    Err(e) => {
                        log::error!("Error {:?} when processing image {:?} ...", e, file);
                        Reporting::error_on_image();
                        Reporting::add_error(file.clone(), format!("{}", e));
                        eprintln!("Error {} when processing image {:?} ...", e, file)
                    }
                }
            }
            Err(e) => match e {
                ExifError::IO(io) => {
                    log::error!("Error {:?} when processing image {:?} ...", io, file);
                    Reporting::error_on_image();
                    Reporting::add_error(file.clone(), format!("IO error: {}", io));
                    eprintln!("Error {} when processing image {:?} ...", io, file)
                }
                ExifError::NotImageFile(s) => {
                    log::warn!("{} is not an image. {}", file.display(), s)
                }
                ExifError::Decoding(s) => {
                    log::error!("Error {:?} when decoding exif_data of file {:?}", s, file);
                    match copy_unsorted_image_in_specific_dir(file, configuration.unsorted_images_directory_as_path()) {
                        Ok(()) => {
                            Reporting::image_processed_unsorted();
                            log::trace!(
                                "Image {:?} processed (no Exif Data -> copied in unsorted dir)...",
                                file
                            )
                        }
                        Err(e) => {
                            log::error!("Error {:?} when processing image {:?} ...", e, file);
                            Reporting::error_on_image();
                            Reporting::add_error(file.clone(), format!("{}", e));
                            eprintln!("Error {} when processing image {:?} ...", e, file)
                        }
                    }
                }
                ExifError::NoExifData => {
                    log::warn!("Warning: {:?} when getting exif_data of file {:?}", e, file);
                    match copy_unsorted_image_in_specific_dir(file, configuration.unsorted_images_directory_as_path()) {
                        Ok(()) => {
                            Reporting::image_processed_unsorted();
                            log::trace!(
                                "Image {:?} processed (no Exif Data -> copied in unsorted dir)...",
                                file
                            )
                        }
                        Err(e) => {
                            log::error!("Error {:?} when processing image {:?} ...", e, file);
                            Reporting::error_on_image();
                            Reporting::add_error(file.clone(), format!("{}", e));
                            eprintln!("Error {} when processing image {:?} ...", e, file)
                        }
                    }
                }
            },
        }
        bar.inc(1);
    });

    bar.finish_and_clear();
    Ok(())
}

fn sort_image_from_exif_data(
    file: &std::path::Path,
    exif_data: &ExifData,
    configuration: &GlobalConfiguration,
) -> Result<()> {
    log::trace!(
        "sort_image_from_exif_data file: {:?} exif_data: {:?}",
        file,
        exif_data
    );
    let new_directory_path = std::path::Path::new(exif_data.year_month.get());
    let new_directory_path_buf = directories::create_subdir(configuration.sorted_images_directory_as_path(), new_directory_path)?;
    let new_directory_path = std::path::Path::new(exif_data.place.get());
    let mut new_directory_path_buf =
        directories::create_subdir(new_directory_path_buf.as_path(), new_directory_path)?;

    if *configuration.use_device() {
        let new_directory_path = std::path::Path::new(exif_data.device.get());
        new_directory_path_buf =
            directories::create_subdir(new_directory_path_buf.as_path(), new_directory_path)?;
    }

    let p = new_directory_path_buf.as_path();
    // unwrap() is ok here, the file have been checked as a file before
    let pb = p.join(std::path::Path::new(&file.file_name().unwrap()));
    let checked = check_for_duplicate_and_rename(pb.as_path())?;

    if let Some(deduplicate_path_name) = checked {
        copy_file_with_metrics(file, deduplicate_path_name.as_path())?;
    } else {
        copy_file_with_metrics(file, pb.as_path())?;
    }

    Ok(())
}

fn copy_unsorted_image_in_specific_dir(
    file: &std::path::Path,
    unsorted_dir: &std::path::Path,
) -> Result<()> {
    log::trace!(
        "copy_unsorted_image_in_specific_dir file: {:?}, unsorted_dir: {:?}",
        file,
        unsorted_dir
    );
    let p = unsorted_dir.join(file);
    fs::DirBuilder::new()
        .recursive(true)
        .create(p.as_path().parent().unwrap())?;

    log::debug!("file: {:?} to: {:?}", file, p.as_path());
    copy_file_with_metrics(file, p.as_path())?;

    Ok(())
}

/// Copy a file and record performance metrics (time and bytes)
fn copy_file_with_metrics(from: &Path, to: &Path) -> Result<u64> {
    let timer = Timer::new();

    // Perform the copy
    let bytes_copied = fs::copy(from, to)?;

    // Record metrics
    PerformanceMetrics::record_file_copy(timer.elapsed(), bytes_copied);

    Ok(bytes_copied)
}

/// verify if there is already a file pointed by the path. If so, return a new path
fn check_for_duplicate_and_rename(file: &Path) -> Result<Option<PathBuf>> {
    log::trace!("check_for_duplicate_and_rename {:?}", file);
    if file.is_dir() {
        log::error!("Error when checking for duplication in target directory");
        eprintln!("Error when checking for duplication in target directory");
        return Err(eyre::eyre!(
            "Error when checking for duplication in target directory"
        ));
    }

    // If the file doesn't exist, no need to rename
    if !file.try_exists()? {
        return Ok(None);
    }

    let path: &Path = file.as_ref();
    let stem = file.file_stem().unwrap().to_string_lossy();
    let ext = path.extension();

    // Try to find a unique name by generating random numbers and checking existence
    for attempt in 0..1000 {
        let random_num = rand::Rng::random_range(&mut rand::rng(), 100..100000);
        let new_filename = format!("{}_duplicate_{}", stem, random_num);

        let mut new_path = path.with_file_name(new_filename);
        if let Some(e) = ext {
            new_path.set_extension(e);
        }

        // Check that the new path doesn't exist
        if !new_path.try_exists()? {
            log::debug!("Found unique name after {} attempts: {:?}", attempt + 1, new_path);
            Reporting::duplicate_renamed();
            return Ok(Some(new_path));
        }
    }

    Err(eyre::eyre!(
        "Unable to find a unique filename after 1000 attempts for {:?}",
        file
    ))
}

#[cfg(test)]
mod tests {
    use crate::exif::Directory;

    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_check_for_duplicate_and_rename() {
        init();
        let current_dir = std::env::current_dir().unwrap();
        std::fs::create_dir("./test_check_dir").unwrap();

        // Test 1: File exists -> should return Some with a new unique path
        let path = std::path::Path::new("./test_check_dir/foo.txt");
        fs::write(path, "Lorem ipsum").unwrap();

        let result = check_for_duplicate_and_rename(path).unwrap();
        assert!(result.is_some(), "Should return Some when file exists");
        let new_path = result.unwrap();
        assert!(!new_path.exists(), "New path should not exist yet");
        assert!(new_path.to_string_lossy().contains("_duplicate_"), "Should contain '_duplicate_'");
        assert_eq!(new_path.extension(), path.extension(), "Should preserve extension");

        // Test 2: File doesn't exist -> should return None
        let path_2 = std::path::Path::new("./test_check_dir/foo_2.txt");
        let result = check_for_duplicate_and_rename(path_2).unwrap();
        assert!(result.is_none(), "Should return None when file doesn't exist");

        // Test 3: Multiple duplicates should generate different names
        let mut generated_paths = std::collections::HashSet::new();
        for i in 0..10 {
            let result = check_for_duplicate_and_rename(path).unwrap();
            assert!(result.is_some(), "Iteration {}: Should return Some", i);
            let new_path = result.unwrap();

            // Verify uniqueness
            assert!(!generated_paths.contains(&new_path),
                "Iteration {}: Generated path {:?} should be unique", i, new_path);
            assert!(!new_path.exists(),
                "Iteration {}: New path {:?} should not exist", i, new_path);

            generated_paths.insert(new_path.clone());

            // Create the file to simulate collision for next iteration
            fs::write(&new_path, format!("Duplicate {}", i)).unwrap();
        }

        // Test 4: Verify all generated paths are different
        assert_eq!(generated_paths.len(), 10, "Should have generated 10 unique paths");

        // ensure we are in the good directory before cleanup
        assert_eq!(current_dir, std::env::current_dir().unwrap());
        // cleanup
        std::fs::remove_dir_all("./test_check_dir").unwrap();
    }

    #[test]
    fn test_copy_unsorted_image_in_specific_dir() {
        init();
        let current_dir = std::env::current_dir().unwrap();
        let dir = std::path::Path::new("./test_cp_unsorted");
        std::fs::create_dir(dir).unwrap();
        let file = std::path::Path::new("foo_test.txt");
        fs::write(file, "Lorem ipsum").unwrap();

        copy_unsorted_image_in_specific_dir(file, dir).unwrap();
        let copied_file = std::path::Path::new("./test_cp_unsorted/foo_test.txt");
        assert!(copied_file.exists());

        // ensure we are in the good directory before cleanup
        assert_eq!(current_dir, std::env::current_dir().unwrap());
        // cleanup
        std::fs::remove_dir_all(dir).unwrap();
        std::fs::remove_file(file).unwrap();
    }

    #[test]
    fn test_sort_image_from_exif_data() {
        init();
        let current_dir = std::env::current_dir().unwrap();
        let dir_target = std::path::Path::new("./test_sort_image");
        std::fs::create_dir(dir_target).unwrap();

        let mut configuration = GlobalConfiguration::new();
        *configuration.use_device_mut() = false;
        *configuration.source_directory_mut() = PathBuf::from("./");
        *configuration.sorted_images_directory_mut() = PathBuf::from(dir_target);

        let exif_data = ExifData {
            year_month: Directory::parse(String::from("2023 10")),
            gps_lat: 0.0,
            gps_long: 0.0,
            place: Directory::parse(String::from("Null_Island")),
            device: Directory::parse(String::from("Nikkon")),
        };

        sort_image_from_exif_data(
            Path::new("./data_4_tests/DSCN0025.jpg"),
            &exif_data,
            &configuration,
        )
        .unwrap();
        let copied_file =
            std::path::Path::new("./test_sort_image/2023 10/Null_Island/DSCN0025.jpg");
        assert!(copied_file.exists());

        *configuration.use_device_mut() = true;

        sort_image_from_exif_data(
            Path::new("./data_4_tests/DSCN0025.jpg"),
            &exif_data,
            &configuration,
        )
        .unwrap();
        let copied_file =
            std::path::Path::new("./test_sort_image/2023 10/Null_Island/Nikkon/DSCN0025.jpg");
        assert!(copied_file.exists());

        // ensure we are in the good directory before cleanup
        assert_eq!(current_dir, std::env::current_dir().unwrap());
        // cleanup
        std::fs::remove_dir_all(dir_target).unwrap();
    }

    #[test]
    fn test_sort_images_in_dir() {
        init();
        let current_dir = std::env::current_dir().unwrap();
        let mut configuration = GlobalConfiguration::new();
        *configuration.use_device_mut() = false;
        *configuration.source_directory_mut() = PathBuf::from("./");
        *configuration.sorted_images_directory_mut() = PathBuf::from("test_sort_images");

        let source_dir = std::path::Path::new("data_4_tests");
        *configuration.unsorted_images_directory_mut() = PathBuf::from("test_sort_images/unsorted");

        sort_images_in_dir(source_dir, &configuration).unwrap();
        assert_eq!(
            4,
            fs::read_dir("test_sort_images/2008 10/Arezzo")
                .unwrap()
                .map(|r| r.unwrap().path())
                .filter(|r| r.is_file())
                .collect::<Vec<PathBuf>>()
                .len()
        );

        // ensure we are in the good directory before cleanup
        assert_eq!(current_dir, std::env::current_dir().unwrap());
        // cleanup
        std::fs::remove_dir_all(configuration.sorted_images_directory_as_path()).unwrap();
    }
}

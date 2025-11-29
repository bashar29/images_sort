use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct GlobalConfiguration {
    use_device: bool,
    source_directory: PathBuf,
    dest_directory: PathBuf,
    sorted_images_directory: PathBuf,
    unsorted_images_directory: PathBuf,
    not_images_directory: PathBuf,
}

impl GlobalConfiguration {
    pub fn new() -> GlobalConfiguration {
        GlobalConfiguration {
            use_device: true,
            source_directory: PathBuf::new(),
            dest_directory: PathBuf::new(),
            sorted_images_directory: PathBuf::new(),
            unsorted_images_directory: PathBuf::new(),
            not_images_directory: PathBuf::new(),
        }
    }

    pub fn use_device(&self) -> &bool {
        &self.use_device
    }

    pub fn use_device_mut(&mut self) -> &mut bool {
        &mut self.use_device
    }

    pub fn source_directory(&self) -> &PathBuf {
        &self.source_directory
    }

    pub fn source_directory_as_path(&self) -> &Path {
        self.source_directory.as_path()
    }

    pub fn source_directory_mut(&mut self) -> &mut PathBuf {
        &mut self.source_directory
    }

    pub fn _dest_directory(&self) -> &PathBuf {
        &self.dest_directory
    }

    pub fn dest_directory_as_path(&self) -> &Path {
        self.dest_directory.as_path()
    }

    pub fn dest_directory_mut(&mut self) -> &mut PathBuf {
        &mut self.dest_directory
    }

    pub fn _sorted_images_directory(&self) -> &PathBuf {
        &self.sorted_images_directory
    }

    pub fn sorted_images_directory_as_path(&self) -> &Path {
        self.sorted_images_directory.as_path()
    }

    pub fn sorted_images_directory_mut(&mut self) -> &mut PathBuf {
        &mut self.sorted_images_directory
    }

    pub fn _unsorted_images_directory(&self) -> &PathBuf {
        &self.unsorted_images_directory
    }

    pub fn unsorted_images_directory_as_path(&self) -> &Path {
        self.unsorted_images_directory.as_path()
    }

    pub fn unsorted_images_directory_mut(&mut self) -> &mut PathBuf {
        &mut self.unsorted_images_directory
    }

    pub fn _not_images_directory(&self) -> &PathBuf {
        &self.not_images_directory
    }

    pub fn not_images_directory_as_path(&self) -> &Path {
        self.not_images_directory.as_path()
    }

    pub fn not_images_directory_mut(&mut self) -> &mut PathBuf {
        &mut self.not_images_directory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_global_configuration() {
        init();
        let mut conf = GlobalConfiguration::new();
        assert_eq!(conf.use_device(), &true);
        let b = conf.use_device_mut();
        *b = false;
        assert_eq!(conf.use_device(), &false);
    }
}

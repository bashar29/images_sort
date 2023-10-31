use std::sync::RwLock;

pub struct Reporting {
    nb_directories: u32,
    nb_images: u32,
    nb_sorted_images: u32,
    nb_unsorted_images: u32,
    nb_error_on_images: u32,
}

// anti-pattern???
static REPORTING_WRAPPER: RwLock<Reporting> = RwLock::new(Reporting {
    nb_directories: 0,
    nb_images: 0,
    nb_sorted_images: 0,
    nb_unsorted_images: 0,
    nb_error_on_images: 0,
});

impl Reporting {
    pub fn image_processed_sorted() {
        let mut r = REPORTING_WRAPPER.write().unwrap();
        r.nb_images += 1;
        r.nb_sorted_images += 1;
    }

    pub fn image_processed_unsorted() {
        let mut r = REPORTING_WRAPPER.write().unwrap();
        r.nb_images += 1;
        r.nb_unsorted_images += 1;
    }

    pub fn directory_processed() {
        let mut r = REPORTING_WRAPPER.write().unwrap();
        r.nb_directories += 1;
    }

    pub fn error_on_image() {
        let mut r = REPORTING_WRAPPER.write().unwrap();
        r.nb_error_on_images += 1;
    }

    pub fn print_reporting() {
        let r = REPORTING_WRAPPER.read().unwrap();
        println!("number of directories processed : {}", r.nb_directories);
        println!("number of images processed : {}", r.nb_images);
        println!("number of images sorted : {}", r.nb_sorted_images);
        println!("number of images not sorted : {}", r.nb_unsorted_images);
        println!(
            "number of error when dealing with an image : {}",
            r.nb_error_on_images
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_reporting() {
        init();
        Reporting::directory_processed();
        Reporting::error_on_image();
        Reporting::error_on_image();
        for _ in 0..20 {
            Reporting::image_processed_sorted();
        }
        for _ in 0..10 {
            Reporting::image_processed_unsorted();
        }
        Reporting::error_on_image();

        // /!\ REPORTING_WRAPPER is static ; if another test use it to access and modify
        // the Reporting object, then the assert_eq!() of this test (or of the another test) will failed.
        // So : keep only one test usinf REPORTING_WRAPPER !!!
        assert_eq!(1, REPORTING_WRAPPER.read().unwrap().nb_directories);
        assert_eq!(3, REPORTING_WRAPPER.read().unwrap().nb_error_on_images);
        assert_eq!(10, REPORTING_WRAPPER.read().unwrap().nb_unsorted_images);
        assert_eq!(20, REPORTING_WRAPPER.read().unwrap().nb_sorted_images);
    }
}
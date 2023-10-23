use once_cell::sync::OnceCell;

static REPORTING_WRAPPER: OnceCell<Reporting> = OnceCell::new();

pub struct Reporting {
    nb_images: u32,
    nb_sorted_images: u32,
    nb_unsorted_images: u32,
}

impl Reporting {

    pub fn init() -> () {
        match REPORTING_WRAPPER.set(Self {
            nb_images: 0,
            nb_sorted_images: 0,
            nb_unsorted_images: 0
        }) {
            Err(_) => {
                log::warn!("Error initializing REPORTING_WRAPPER (already filled)");
                ()
            }
            _ => () 
        }
    }

    pub fn get() -> &'static Reporting {
        REPORTING_WRAPPER.get().unwrap()
    }

    pub fn image_processed_sorted(&mut self) {
        self.nb_images += 1;
        self.nb_sorted_images += 1;
    }
    
    pub fn image_processed_unsorted(&mut self) {
        self.nb_images += 1;
        self.nb_unsorted_images += 1;
    }

    pub fn print_reporting(&self) {
        println!("number of images processed : {}", self.nb_images);
        println!("number of images sorted : {}", self.nb_sorted_images);
        println!("number of images not sorted : {}", self.nb_unsorted_images);
    }

}
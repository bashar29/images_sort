use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::RwLock;
use std::time::Instant;

// Atomic counters for thread-safe increments without contention
static NB_DIRECTORIES: AtomicU32 = AtomicU32::new(0);
static NB_IMAGES: AtomicU32 = AtomicU32::new(0);
static NB_SORTED_IMAGES: AtomicU32 = AtomicU32::new(0);
static NB_UNSORTED_IMAGES: AtomicU32 = AtomicU32::new(0);
static NB_ERROR_ON_IMAGES: AtomicU32 = AtomicU32::new(0);
static NB_DUPLICATES_RENAMED: AtomicU32 = AtomicU32::new(0);

// Complex data structures that still need RwLock
pub struct Reporting {
    start_time: Option<Instant>,
    places_found: HashMap<String, u32>,
    devices_found: HashSet<String>,
    errors_details: Vec<(PathBuf, String)>,
    oldest_date: Option<String>,
    newest_date: Option<String>,
}

impl Default for Reporting {
    fn default() -> Self {
        Self {
            start_time: None,
            places_found: HashMap::new(),
            devices_found: HashSet::new(),
            errors_details: Vec::new(),
            oldest_date: None,
            newest_date: None,
        }
    }
}

// TODO anti-pattern to have a static variable?
static REPORTING_WRAPPER: Lazy<RwLock<Reporting>> = Lazy::new(|| RwLock::new(Reporting::default()));

impl Reporting {
    pub fn start_timer() {
        let mut r = REPORTING_WRAPPER.write().unwrap();
        r.start_time = Some(Instant::now());
    }

    pub fn image_processed_sorted() {
        NB_IMAGES.fetch_add(1, Ordering::Relaxed);
        NB_SORTED_IMAGES.fetch_add(1, Ordering::Relaxed);
    }

    pub fn image_processed_unsorted() {
        NB_IMAGES.fetch_add(1, Ordering::Relaxed);
        NB_UNSORTED_IMAGES.fetch_add(1, Ordering::Relaxed);
    }

    pub fn directory_processed() {
        NB_DIRECTORIES.fetch_add(1, Ordering::Relaxed);
    }

    pub fn error_on_image() {
        NB_ERROR_ON_IMAGES.fetch_add(1, Ordering::Relaxed);
    }

    pub fn duplicate_renamed() {
        NB_DUPLICATES_RENAMED.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_place(place: String) {
        let mut r = REPORTING_WRAPPER.write().unwrap();
        *r.places_found.entry(place).or_insert(0) += 1;
    }

    pub fn add_device(device: String) {
        let mut r = REPORTING_WRAPPER.write().unwrap();
        r.devices_found.insert(device);
    }

    pub fn add_error(file: PathBuf, reason: String) {
        let mut r = REPORTING_WRAPPER.write().unwrap();
        r.errors_details.push((file, reason));
    }

    pub fn update_date_range(date: &str) {
        let mut r = REPORTING_WRAPPER.write().unwrap();

        // Update oldest
        if r.oldest_date.is_none() || r.oldest_date.as_ref().map(|s| s.as_str()) > Some(date) {
            r.oldest_date = Some(date.to_string());
        }

        // Update newest
        if r.newest_date.is_none() || r.newest_date.as_ref().map(|s| s.as_str()) < Some(date) {
            r.newest_date = Some(date.to_string());
        }
    }

    pub fn _reset() {
        // Reset atomic counters
        NB_DIRECTORIES.store(0, Ordering::Relaxed);
        NB_IMAGES.store(0, Ordering::Relaxed);
        NB_SORTED_IMAGES.store(0, Ordering::Relaxed);
        NB_UNSORTED_IMAGES.store(0, Ordering::Relaxed);
        NB_ERROR_ON_IMAGES.store(0, Ordering::Relaxed);
        NB_DUPLICATES_RENAMED.store(0, Ordering::Relaxed);

        // Reset complex structures
        let mut r = REPORTING_WRAPPER.write().unwrap();
        r.start_time = None;
        r.places_found.clear();
        r.devices_found.clear();
        r.errors_details.clear();
        r.oldest_date = None;
        r.newest_date = None;
    }

    pub fn print_reporting() {
        let r = REPORTING_WRAPPER.read().unwrap();

        // Read atomic counters
        let nb_directories = NB_DIRECTORIES.load(Ordering::Relaxed);
        let nb_images = NB_IMAGES.load(Ordering::Relaxed);
        let nb_sorted_images = NB_SORTED_IMAGES.load(Ordering::Relaxed);
        let nb_unsorted_images = NB_UNSORTED_IMAGES.load(Ordering::Relaxed);
        let nb_error_on_images = NB_ERROR_ON_IMAGES.load(Ordering::Relaxed);
        let nb_duplicates_renamed = NB_DUPLICATES_RENAMED.load(Ordering::Relaxed);

        // Calculate execution time
        let duration = r.start_time.map(|start| start.elapsed());
        let duration_str = match duration {
            Some(d) => {
                let secs = d.as_secs();
                if secs < 60 {
                    format!("{}s", secs)
                } else {
                    format!("{}m {}s", secs / 60, secs % 60)
                }
            }
            None => "N/A".to_string(),
        };

        // Calculate percentages
        let sorted_pct = if nb_images > 0 {
            (nb_sorted_images as f64 / nb_images as f64) * 100.0
        } else {
            0.0
        };
        let unsorted_pct = if nb_images > 0 {
            (nb_unsorted_images as f64 / nb_images as f64) * 100.0
        } else {
            0.0
        };
        let error_pct = if nb_images > 0 {
            (nb_error_on_images as f64 / nb_images as f64) * 100.0
        } else {
            0.0
        };

        println!("╔═══════════════════════════════════════════════════════════╗");
        println!("║              📸 Image Sorting Report                      ║");
        println!("╠═══════════════════════════════════════════════════════════╣");
        println!("║ ⏱️  Execution time         : {:<29}║", duration_str);
        println!("║ 📁 Directories processed   : {:<29}║", nb_directories);
        println!("║ 🖼️  Images processed        : {:<29}║", nb_images);
        println!("║                                                            ║");
        println!("║ ✅ Successfully sorted     : {} ({:.1}%){:>17}║",
            nb_sorted_images, sorted_pct, "");
        println!("║ ⚠️  Unsorted (no EXIF)     : {} ({:.1}%){:>17}║",
            nb_unsorted_images, unsorted_pct, "");
        println!("║ 🔁 Duplicates renamed      : {:<29}║", nb_duplicates_renamed);
        println!("║ ❌ Errors                  : {} ({:.1}%){:>17}║",
            nb_error_on_images, error_pct, "");

        // Display locations statistics
        if !r.places_found.is_empty() {
            println!("║                                                            ║");
            println!("║ 🌍 Locations discovered    : {:<29}║", r.places_found.len());

            // Get top 5 locations
            let mut places_vec: Vec<_> = r.places_found.iter().collect();
            places_vec.sort_by(|a, b| b.1.cmp(a.1));
            let top_places: Vec<String> = places_vec
                .iter()
                .take(5)
                .map(|(place, count)| format!("{} ({})", place, count))
                .collect();

            if !top_places.is_empty() {
                println!("║    Top: {:<48}║", top_places.join(", "));
            }
        }

        // Display devices
        if !r.devices_found.is_empty() {
            println!("║                                                            ║");
            println!("║ 📷 Devices found           : {:<29}║", r.devices_found.len());
            let devices: Vec<String> = r.devices_found.iter().take(5).cloned().collect();
            if !devices.is_empty() {
                let devices_str = devices.join(", ");
                let devices_display = if devices_str.len() > 48 {
                    format!("{}...", &devices_str[..45])
                } else {
                    devices_str
                };
                println!("║    {:<52}║", devices_display);
            }
        }

        // Display date range
        if r.oldest_date.is_some() && r.newest_date.is_some() {
            println!("║                                                            ║");
            let date_range = format!(
                "{} → {}",
                r.oldest_date.as_ref().unwrap(),
                r.newest_date.as_ref().unwrap()
            );
            println!("║ 📅 Date range              : {:<29}║", date_range);
        }

        println!("╚═══════════════════════════════════════════════════════════╝");

        // Display error details if any
        if !r.errors_details.is_empty() && r.errors_details.len() <= 10 {
            println!();
            println!("⚠️  Error details:");
            for (file, reason) in &r.errors_details {
                println!("  • {}: {}", file.display(), reason);
            }
        } else if r.errors_details.len() > 10 {
            println!();
            println!("⚠️  {} errors occurred (showing first 10):", r.errors_details.len());
            for (file, reason) in r.errors_details.iter().take(10) {
                println!("  • {}: {}", file.display(), reason);
            }
        }
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
        // /!\ REPORTING_WRAPPER is static ; if another test use it to access and modify
        // the Reporting object, then the assert_eq!() of this test (or of the another test) will failed.
        // reset() to minimize the risk but it's still possible a write occurs from another test during tests execution
        // to avoid this : cargo test -- --test-threads=1
        Reporting::_reset();
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

        // /!\ Atomic counters are static ; if another test uses them
        // then the assert_eq!() of this test (or of the another test) may fail.
        // reset() to minimize the risk but it's still possible a write occurs from another test during tests execution
        // to avoid this : cargo test -- --test-threads=1
        assert_eq!(1, NB_DIRECTORIES.load(Ordering::Relaxed));
        assert_eq!(3, NB_ERROR_ON_IMAGES.load(Ordering::Relaxed));
        assert_eq!(10, NB_UNSORTED_IMAGES.load(Ordering::Relaxed));
        assert_eq!(20, NB_SORTED_IMAGES.load(Ordering::Relaxed));
    }
}

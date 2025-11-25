//! Performance tracking and profiling
//!
//! This module provides tools to measure and report performance metrics
//! for image processing operations.

use once_cell::sync::Lazy;
use std::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    // Operation counts
    pub exif_reads: u32,
    pub geocoding_lookups: u32,
    pub geocoding_cache_hits: u32,
    pub file_copies: u32,
    pub directory_creations: u32,

    // Time measurements
    pub total_exif_time: Duration,
    pub total_geocoding_time: Duration,
    pub total_file_copy_time: Duration,
    pub total_directory_creation_time: Duration,

    // File size stats
    pub total_bytes_copied: u64,
}

static PERF_METRICS: Lazy<RwLock<PerformanceMetrics>> =
    Lazy::new(|| RwLock::new(PerformanceMetrics::default()));

impl PerformanceMetrics {
    /// Record an EXIF read operation
    pub fn record_exif_read(duration: Duration) {
        let mut metrics = PERF_METRICS.write().unwrap();
        metrics.exif_reads += 1;
        metrics.total_exif_time += duration;
    }

    /// Record a geocoding lookup
    pub fn record_geocoding(duration: Duration, cache_hit: bool) {
        let mut metrics = PERF_METRICS.write().unwrap();
        metrics.geocoding_lookups += 1;
        metrics.total_geocoding_time += duration;
        if cache_hit {
            metrics.geocoding_cache_hits += 1;
        }
    }

    /// Record a file copy operation
    pub fn record_file_copy(duration: Duration, bytes: u64) {
        let mut metrics = PERF_METRICS.write().unwrap();
        metrics.file_copies += 1;
        metrics.total_file_copy_time += duration;
        metrics.total_bytes_copied += bytes;
    }

    /// Record a directory creation
    pub fn record_directory_creation(duration: Duration) {
        let mut metrics = PERF_METRICS.write().unwrap();
        metrics.directory_creations += 1;
        metrics.total_directory_creation_time += duration;
    }

    /// Reset all metrics (used for testing)
    #[allow(dead_code)]
    pub fn _reset() {
        let mut metrics = PERF_METRICS.write().unwrap();
        *metrics = PerformanceMetrics::default();
    }

    /// Print performance report
    pub fn print_report() {
        let metrics = PERF_METRICS.read().unwrap();

        println!();
        println!("╔═══════════════════════════════════════════════════════════╗");
        println!("║              ⚡ Performance Report                        ║");
        println!("╠═══════════════════════════════════════════════════════════╣");

        // EXIF operations
        if metrics.exif_reads > 0 {
            let avg_exif = metrics.total_exif_time.as_millis() / metrics.exif_reads as u128;
            println!("║ 📖 EXIF reads              : {:<29}║", metrics.exif_reads);
            println!("║    Total time              : {:<29}║",
                format!("{:.2}s", metrics.total_exif_time.as_secs_f64()));
            println!("║    Average per read        : {:<29}║",
                format!("{}ms", avg_exif));
        }

        // Geocoding operations
        if metrics.geocoding_lookups > 0 {
            let avg_geocoding = metrics.total_geocoding_time.as_millis() / metrics.geocoding_lookups as u128;
            let cache_hit_rate = (metrics.geocoding_cache_hits as f64 / metrics.geocoding_lookups as f64) * 100.0;
            println!("║                                                            ║");
            println!("║ 🌍 Geocoding lookups       : {:<29}║", metrics.geocoding_lookups);
            println!("║    Cache hits              : {} ({:.1}%){:>17}║",
                metrics.geocoding_cache_hits, cache_hit_rate, "");
            println!("║    Total time              : {:<29}║",
                format!("{:.2}s", metrics.total_geocoding_time.as_secs_f64()));
            println!("║    Average per lookup      : {:<29}║",
                format!("{}ms", avg_geocoding));
        }

        // File copy operations
        if metrics.file_copies > 0 {
            let avg_copy = metrics.total_file_copy_time.as_millis() / metrics.file_copies as u128;
            let total_mb = metrics.total_bytes_copied as f64 / (1024.0 * 1024.0);
            let throughput = total_mb / metrics.total_file_copy_time.as_secs_f64();
            println!("║                                                            ║");
            println!("║ 📁 File copies             : {:<29}║", metrics.file_copies);
            println!("║    Total size              : {:<29}║",
                format!("{:.2} MB", total_mb));
            println!("║    Total time              : {:<29}║",
                format!("{:.2}s", metrics.total_file_copy_time.as_secs_f64()));
            println!("║    Average per file        : {:<29}║",
                format!("{}ms", avg_copy));
            println!("║    Throughput              : {:<29}║",
                format!("{:.2} MB/s", throughput));
        }

        // Directory operations
        if metrics.directory_creations > 0 {
            let avg_mkdir = metrics.total_directory_creation_time.as_millis() / metrics.directory_creations as u128;
            println!("║                                                            ║");
            println!("║ 📂 Directory creations     : {:<29}║", metrics.directory_creations);
            println!("║    Total time              : {:<29}║",
                format!("{:.2}s", metrics.total_directory_creation_time.as_secs_f64()));
            println!("║    Average per mkdir       : {:<29}║",
                format!("{}ms", avg_mkdir));
        }

        // Time breakdown
        println!("║                                                            ║");
        println!("║ ⏱️  Time breakdown:                                        ║");
        let total_measured = metrics.total_exif_time
            + metrics.total_geocoding_time
            + metrics.total_file_copy_time
            + metrics.total_directory_creation_time;

        if total_measured.as_millis() > 0 {
            let exif_pct = (metrics.total_exif_time.as_secs_f64() / total_measured.as_secs_f64()) * 100.0;
            let geo_pct = (metrics.total_geocoding_time.as_secs_f64() / total_measured.as_secs_f64()) * 100.0;
            let copy_pct = (metrics.total_file_copy_time.as_secs_f64() / total_measured.as_secs_f64()) * 100.0;
            let mkdir_pct = (metrics.total_directory_creation_time.as_secs_f64() / total_measured.as_secs_f64()) * 100.0;

            println!("║    EXIF reading            : {:<29}║", format!("{:.1}%", exif_pct));
            println!("║    Geocoding               : {:<29}║", format!("{:.1}%", geo_pct));
            println!("║    File copying            : {:<29}║", format!("{:.1}%", copy_pct));
            println!("║    Directory creation      : {:<29}║", format!("{:.1}%", mkdir_pct));
        }

        println!("╚═══════════════════════════════════════════════════════════╝");
    }
}

/// Helper struct to automatically measure operation duration
pub struct Timer {
    start: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

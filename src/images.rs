use anyhow;
use exif::{Exif, In, Tag};
use regex::Regex;
use std::path::Path;

use crate::reverse_gps;

#[derive(Debug)]
pub struct ExifData {
    pub year_month: Directory,
    pub gps_lat: String,
    pub gps_long: String,
    pub place: Directory,
    pub device: Directory,
}

#[derive(Debug)]
pub struct Directory(String);
impl Directory {
    pub fn parse(s: String) -> Directory {
        log::trace!("parse {}", s);
        let re = Regex::new(r"[^\w]").unwrap();
        let clean_string = re.replace_all(&s, "");
        log::debug!("clean_string = {}", clean_string);
        Self(clean_string.to_string())
    }

    pub fn get(&self) -> &String {
        &self.0
    }
}

pub fn get_exif_data(path: &Path) -> Result<ExifData, anyhow::Error> {
    log::trace!("get_exif_data of {:?}", &path);
    let file = std::fs::File::open(path)?;
    let mut bufreader = std::io::BufReader::new(file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader)?;

    let exif_data = analyze_exif_data(exif)?;

    // TODO convert GPS to place

    Ok(exif_data)
}

fn analyze_exif_data(exif: Exif) -> Result<ExifData, anyhow::Error> {
    log::trace!("analyze_exif_data ...");

    let mut exif_data = ExifData {
        year_month: Directory::parse(String::from("Unknown")),
        place: Directory::parse(String::from("Unknown")),
        device: Directory::parse(String::from("Unknown")),
        gps_lat: String::from("Unknown"),
        gps_long: String::from("Unknown"),
    };

    let date_time = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY);
    if let Some(timestamp) = date_time {
        log::debug!("EXIF DateTimeOriginal = {}", timestamp.display_value());
        let timestamp_value = timestamp.display_value().to_string();
        exif_data.year_month = Directory::parse(String::from(&timestamp_value[0..7]));
    } else {
        // TODO exploit DateTimeDigitized
        log::warn!("EXIF DateTimeOriginal tag is missing");
    }

    let device = exif.get_field(Tag::Model, In::PRIMARY);
    if let Some(model) = device {
        log::debug!("EXIF Model = {}", model.display_value());
        exif_data.device = Directory::parse(String::from(model.display_value().to_string()));
    } else {
        log::warn!("EXIF Model tag is missing");
    }

    let lat = exif.get_field(Tag::GPSLatitude, In::PRIMARY);
    if let Some(lat) = lat {
        log::debug!("EXIF GPSLatitude = {}", lat.display_value());
        exif_data.gps_lat = lat.display_value().to_string();
    } else {
        log::warn!("EXIF GPSLatitude tag is missing");
    }

    let long = exif.get_field(Tag::GPSLongitude, In::PRIMARY);
    if let Some(long) = long {
        log::debug!("EXIF GPSLongitude = {}", long.display_value());
        exif_data.gps_long = long.display_value().to_string();
    } else {
        log::warn!("EXIF GPSLongitude tag is missing");
    }

    let place = reverse_gps::find_place(&exif_data.gps_lat, &exif_data.gps_long);
    if let Some(place) = place {
        log::debug!("EXIF Place from reverse geocoding = {}", place);
        exif_data.place = Directory::parse(place);
    } else {
        log::warn!("EXIF no place found");
    }

    Ok(exif_data)
}

// DateTimeOriginal
// DateTimeDigitized
// Model
// GPSLatitude
// GPSLongitude
// https://developers.google.com/maps/documentation/geocoding/requests-reverse-geocoding?hl=fr

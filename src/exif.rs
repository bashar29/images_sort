//! # exif_images
//!
//! Getting the exif data needed to sort the images.
//!

use crate::place_finder;
use exif::{Exif, Field, In, Tag, Value};
use regex::Regex;
use std::path::Path;

#[derive(Debug)]
pub struct ExifData {
    pub year_month: Directory,
    pub gps_lat: f64,
    pub gps_long: f64,
    pub place: Directory,
    pub device: Directory,
}

#[derive(thiserror::Error, Debug)]
pub enum ExifError {
    #[error("No Exif Data in the image")]
    NoExifData,
    #[error("The file processed is not an image {0}")]
    NotImageFile(String),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Coords {0} can't be processed")]
    Decoding(String),
}

/// Directory Struct to ensure that only authorized characters in directories names.
///
/// # Examples
/// ```
/// let dir = Directory::parse(String::from("Cool @name"));
/// assert_eq!(dir.get(), "Cool  name");
///
/// ```
#[derive(Debug, PartialEq)]
pub struct Directory(String);
impl Directory {
    pub fn parse(s: String) -> Directory {
        log::trace!("parse {}", s);
        let re = Regex::new(r"[^\w]").unwrap();
        let clean_string = re.replace_all(&s, " ");
        log::debug!("clean_string = {}", clean_string);
        Self(clean_string.to_string())
    }

    pub fn get(&self) -> &String {
        &self.0
    }
}

/// get the exif data needed to sort the file
pub fn get_exif_data(path: &Path) -> Result<ExifData, ExifError> {
    log::trace!("get_exif_data of {:?}", &path);
    let file = std::fs::File::open(path)?;
    let mut bufreader = std::io::BufReader::new(file);
    let exifreader = exif::Reader::new();

    //let exif = exifreader.read_from_container(&mut bufreader)?;
    let exif = match exifreader.read_from_container(&mut bufreader) {
        Ok(exif) => exif,
        Err(e) => match e {
            exif::Error::Io(io) => return Err(ExifError::IO(io)),
            exif::Error::InvalidFormat(s) => return Err(ExifError::NotImageFile(s.to_string())),
            _ => return Err(ExifError::NoExifData),
        },
    };

    let exif_data = analyze_exif_data(exif)?;

    Ok(exif_data)
}

fn analyze_exif_data(exif: Exif) -> Result<ExifData, ExifError> {
    log::trace!("analyze_exif_data ...");

    let mut exif_data = ExifData {
        year_month: Directory::parse(String::from("Unknown Date")),
        place: Directory::parse(String::from("Unknown Place")),
        device: Directory::parse(String::from("Unknown Device")),
        gps_lat: 0.0,
        gps_long: 0.0,
    };

    let date_time_original = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY);
    let date_time_digitized = exif.get_field(Tag::DateTimeDigitized, In::PRIMARY);
    if let Some(timestamp) = analyze_exif_datetime(date_time_original) {
        exif_data.year_month = timestamp;
    } else {
        log::warn!("EXIF DateTimeOriginal tag is missing - trying DateTimeDigitized");
        if let Some(timestamp) = analyze_exif_datetime(date_time_digitized) {
            exif_data.year_month = timestamp;
        } else {
            log::warn!("both EXIF DateTimeOriginal and DateTimeDigitized tag are missing");
        }
    }

    let device = exif.get_field(Tag::Model, In::PRIMARY);
    if let Some(model) = device {
        log::debug!("EXIF Model = {}", model.display_value());
        exif_data.device = Directory::parse(model.display_value().to_string());
    } else {
        log::warn!("EXIF Model tag is missing");
    }

    // https://exiftool.org/TagNames/GPS.html
    let lat = exif.get_field(Tag::GPSLatitude, In::PRIMARY);
    let lat_ref = exif.get_field(Tag::GPSLatitudeRef, In::PRIMARY);

    exif_data.gps_lat = analyze_exif_lat_long(lat, lat_ref)?;

    let long = exif.get_field(Tag::GPSLongitude, In::PRIMARY);
    let long_ref = exif.get_field(Tag::GPSLongitudeRef, In::PRIMARY);

    exif_data.gps_long = analyze_exif_lat_long(long, long_ref)?;

    if exif_data.gps_lat != 0.0 || exif_data.gps_long != 0.0 {
        let place = place_finder::find_place(exif_data.gps_lat, exif_data.gps_long);
        if let Some(place) = place {
            log::debug!("EXIF Place from reverse geocoding = {}", place);
            exif_data.place = Directory::parse(place);
        } else {
            log::warn!("EXIF no place found");
            exif_data.place = Directory::parse(String::from("Unknown Place"));
        }
    } else {
        exif_data.place = Directory::parse(String::from("Null_Island"));
        // https://fr.wikipedia.org/wiki/Null_Island
    }

    Ok(exif_data)
}

fn analyze_exif_datetime(date_time: Option<&Field>) -> Option<Directory> {
    log::trace!("analyze_exif_datetime {:?}", date_time);
    if let Some(timestamp) = date_time {
        log::debug!("EXIF DateTime*** = {}", timestamp.display_value());
        let timestamp_value = timestamp.display_value().to_string();
        Some(Directory::parse(String::from(&timestamp_value[0..7])))
    } else {
        None
    }
}

/// analyse field GPSLatitude / GPSLongitude and GPSLatitudeRef / GPSLongitudeRef and return
/// a f64 value that represent the latitude in decimal degree.
/// If not possible to get the latitude, return 0.0 (latitude of Null Island)
fn analyze_exif_lat_long(l: Option<&Field>, l_ref: Option<&Field>) -> Result<f64, ExifError> {
    if let Some(l) = l {
        log::debug!("EXIF GPSL*** = {}", l.display_value());
        let latitude_or_longitude = match &l.value {
            Value::Rational(vec_rationals) => {
                //let l = place_finder::convert_deg_min_sec_to_decimal_deg(vec_rationals)?;
                let l = match place_finder::convert_deg_min_sec_to_decimal_deg(vec_rationals) {
                    Ok(l) => l,
                    Err(place_finder::PlaceFinderError::Decode(coords)) => {
                        return Err(ExifError::Decoding(coords))
                    }
                };

                match l_ref {
                    Some(v) => {
                        log::debug!("EXIF GPSL***Ref = {}", v.display_value());
                        if v.display_value().to_string() == "N"
                            || v.display_value().to_string() == "E"
                        {
                            l
                        } else {
                            -1.0 * l
                        }
                    }
                    None => 0.0,
                }
            }
            _ => 0.0,
        };
        Ok(latitude_or_longitude)
    } else {
        log::warn!("EXIF GPSLatitude tag is missing");
        Ok(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_get_exif_data() {
        init();
        let path = std::path::Path::new("DSCN0025.jpg");
        let exif_data = get_exif_data(path).unwrap();
        log::debug!("{:?}", exif_data);
        assert_eq!(
            exif_data.year_month,
            Directory::parse("2008 10".to_string())
        );
        assert_eq!(exif_data.place, Directory::parse("Arezzo".to_string()));
        assert_eq!(
            exif_data.device,
            Directory::parse(" COOLPIX P6000 ".to_string())
        );
    }
}

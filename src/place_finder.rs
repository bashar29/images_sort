//! reverse_gps
//! leverage reverse_geocoder crate to get the nearest place (town) of the GPS data we got from an image.

use exif::Rational;
use once_cell::sync::OnceCell;
use reverse_geocoder::{Locations, ReverseGeocoder};

// two static variables, to avoid loading data for each image we are dealing with.
static LOCATIONS_WRAPPER: OnceCell<LocationsWrapper> = OnceCell::new();
static REVERSE_GEOCODER_WRAPPER: OnceCell<ReverseGeocoderWrapper> = OnceCell::new();

pub struct ReverseGeocoderWrapper<'a> {
    pub reverse_geocoder: ReverseGeocoder<'a>,
}

pub struct LocationsWrapper {
    pub locations: Locations,
}

impl LocationsWrapper {
    pub fn init() -> Result<(), anyhow::Error> {
        match LOCATIONS_WRAPPER.set(Self {
            locations: Locations::from_memory(),
        }) {
            Err(_e) => {
                log::warn!("Error initializing Locations - OnceCell was not empty");
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn get_locations_wrapper() -> &'static LocationsWrapper {
        LOCATIONS_WRAPPER.get().unwrap()
    }
}

impl ReverseGeocoderWrapper<'static> {
    pub fn init() -> Result<(), anyhow::Error> {
        match REVERSE_GEOCODER_WRAPPER.set(Self {
            reverse_geocoder: ReverseGeocoder::new(
                &LocationsWrapper::get_locations_wrapper().locations,
            ),
        }) {
            Err(_e) => {
                log::warn!("Error initializing Reverse Geocoder - OnceCell was not empty");
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn get_geocoder_wrapper() -> &'static ReverseGeocoderWrapper<'static> {
        REVERSE_GEOCODER_WRAPPER.get().unwrap()
    }
}

pub fn find_place(lat: f64, long: f64) -> Option<String> {
    log::trace!("find_place {} {}", lat, long);

    let coords = (lat, long);

    let search_result = ReverseGeocoderWrapper::get_geocoder_wrapper()
        .reverse_geocoder
        .search(coords)?;
    log::debug!("Distance {}", search_result.distance);
    log::debug!("Record {}", search_result.record);
    Some(String::from(&search_result.record.name))
}

// conversion
// https://www.fcc.gov/media/radio/dms-decimal
// https://www.rapidtables.com/convert/number/degrees-minutes-seconds-to-degrees.html
pub fn convert_deg_min_sec_to_decimal_deg(coord: &Vec<Rational>) -> Result<f64, anyhow::Error> {
    log::trace!("convert_deg_min_sec_to_decimal_deg {:?}", coord);
    let deg = coord.get(0).ok_or_else(|| 0);
    let min = coord.get(1).ok_or_else(|| 0);
    let sec = coord.get(2).ok_or_else(|| 0);

    let m = min.unwrap().to_f64() / 60.0;
    let s = sec.unwrap().to_f64() / 3600.0;

    Ok(deg.unwrap().to_f64() + m + s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use exif::Rational;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    // load reverse_geocoder data
    fn load_data() {
        let _ = LocationsWrapper::init().unwrap();
        let _ = ReverseGeocoderWrapper::init().unwrap();
    }

    #[test]
    fn test_convert_deg_min_sec_to_decimal_deg() {
        init();
        let deg: Rational = Rational { num: 48, denom: 1 };
        let min: Rational = Rational { num: 5, denom: 1 };
        let sec: Rational = Rational { num: 92, denom: 2 };
        assert_eq!(
            convert_deg_min_sec_to_decimal_deg(&vec![deg, min, sec]).unwrap(),
            48.096111111111114
        );
    }

    #[test]
    fn test_find_place() {
        init();
        load_data();
        let lat = 48.083328;
        let long = -1.68333;
        let rennes = find_place(lat, long);
        assert_eq!(rennes.unwrap(), String::from("Rennes"));

        let lat = 38.7208429;
        let long = -9.1525689;
        let lisbonne = find_place(lat, long);
        assert_eq!(lisbonne.unwrap(), String::from("Lisbon"));

        let lat = -20.8798761;
        let long = 55.4440519;
        let saint_denis = find_place(lat, long);
        assert_eq!(saint_denis.unwrap(), String::from("Saint-Denis"));
    }
}

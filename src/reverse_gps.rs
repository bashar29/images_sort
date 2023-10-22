use exif::Rational;
use once_cell::sync::OnceCell;
use reverse_geocoder::{Locations, ReverseGeocoder};

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
            Err(_e) => anyhow::bail!("Error initializing Locations"),
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
            Err(_e) => anyhow::bail!("Error initializing Reverse Geocoder"),
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

    #[test]
    fn test_convert_deg_min_sec_to_decimal_deg() {
        let deg: Rational = Rational { num: 48, denom: 1 };
        let min: Rational = Rational { num: 5, denom: 1 };
        let sec: Rational = Rational { num: 92, denom: 2 };
        assert_eq!(
            convert_deg_min_sec_to_decimal_deg(&vec![deg, min, sec]).unwrap(),
            48.096111111111114
        );
    }
}

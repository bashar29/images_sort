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

pub fn find_place(lat: &str, long: &str) -> Option<String> {
    log::trace!("find_place {} {}", lat, long);

    let lat_float = match lat.parse::<f64>() {
        Ok(f) => f,
        _ => return Some(String::from("Unknown")),
    };
    let long_float = match long.parse::<f64>() {
        Ok(f) => f,
        _ => return Some(String::from("Unknown")),
    };

    let coords = (lat_float, long_float);

    let search_result = ReverseGeocoderWrapper::get_geocoder_wrapper()
        .reverse_geocoder
        .search(coords)?;
    log::debug!("Distance {}", search_result.distance);
    log::debug!("Record {}", search_result.record);
    Some(String::from(&search_result.record.name))
}

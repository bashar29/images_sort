use reverse_geocoder::{Locations, ReverseGeocoder, SearchResult};

pub fn init_reverse_geocoder() -> ReverseGeocoder<'static> {
    log::trace!("init_reverse_geocoder");
    let geocoder = ReverseGeocoder::new(&Locations::from_memory());
    geocoder
}

pub fn find_place(reverse_geocoder: &ReverseGeocoder, lat: &str, long: &str) -> Option<String> {
    log::trace!("find_place {} {}", lat, long);
    let coords = (lat.parse().unwrap(), long.parse().unwrap());
    let search_result = reverse_geocoder.search(coords)?;
    log::debug!("Distance {}", search_result.distance);
    log::debug!("Record {}", search_result.record);
    Some(search_result.record.name)
}
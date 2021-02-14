use std::io::{ErrorKind, Error};
mod satellite;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let res = ureq::get("https://celestrak.com/NORAD/elements/gp.php")
    .query("GROUP", "starlink")
    .query("FORMAT", "json")
    .call();
  if res.error() {
    Err(Box::new(Error::new(ErrorKind::Other, format!(
      "network error {}: {}",
      res.status(),
      res.into_string().unwrap_or_default(),
    ))))
  } else {
    if let Ok(sats) = satellite::propagate(&res.into_json_deserialize()?) {
      for sat in &sats {
        println!("Name: {}", sat.get_name());
        println!("Timestamp: {}", sat.get_timestamp_s());
        println!(
          "LAT: {}, LON: {}, ALT: {}",
          sat.get_geodetic_position().lat_deg,
          sat.get_geodetic_position().lon_deg,
          sat.get_geodetic_position().alt_km
        );
      }
      let mut observer = satellite::Observer::new(
        [0.0, 0.0, 0.0],
        chrono::Utc::now().naive_utc().timestamp(),
      );
      satellite::update_observer_satellites(&mut observer, &sats);
      for sat in observer.get_satellites_in_los() {
        println!(
          "NAME: {}, AZ: {} EL: {} RA: {}",
          sat.get_satellite_info().get_name(),
          sat.get_position().azimuth_deg,
          sat.get_position().elevation_deg,
          sat.get_position().range_km,
        )
      }
    }
    Ok(())
  }
}

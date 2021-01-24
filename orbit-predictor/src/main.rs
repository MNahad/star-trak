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
      res.into_string()?
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
    }
    Ok(())
  }
}

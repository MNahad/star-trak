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
    let mut elements_group = res.into_json_deserialize::<Vec<sgp4::Elements>>()?;
    let elements_group = elements_group
      .drain(0..)
      .map(|el| satellite::Elements::new(el))
      .collect();
    let sats = satellite::propagate(&elements_group);
    for sat in &sats {
      println!("Name: {}", sat.get_data().get_name());
      println!("Timestamp: {}", sat.get_timestamp_s());
      println!(
        "LAT: {}, LON: {}, ALT: {}",
        sat.get_latitude_deg(),
        sat.get_longitude_deg(),
        sat.get_altitude_km(),
      );
    }
    let mut observer = satellite::Observer::new(0.0, 0.0, 0.0);
    satellite::update_observer_satellites(&mut observer, &sats);
    for sat in observer.get_satellites_in_range() {
      println!(
        "NAME: {}, AZ: {} EL: {} RA: {}",
        sat.get_data().get_data().get_name(),
        sat.get_azimuth_deg(),
        sat.get_elevation_deg(),
        sat.get_range_km(),
      )
    }
    Ok(())
  }
}

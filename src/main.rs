mod propagator;
use propagator::{Sgp4Data, Propagator, Observer};
use std::thread;
use std::time::Duration;

fn main() {
  let res = ureq::get("https://celestrak.com/NORAD/elements/gp.php")
    .query("GROUP", "starlink")
    .query("FORMAT", "json")
    .call();
  if !res.error() {
    if let Ok(mut elements_group) = res.into_json_deserialize::<Vec<sgp4::Elements>>() {
      let sgp4_data = elements_group
        .drain(0..)
        .filter_map(|elements| {
          Sgp4Data::from_elements(elements)
        })
        .collect();
      let mut propagator = Propagator::from_sgp4_data(&sgp4_data);
      let mut observer = Observer::from_coords(
        0.0,
        0.0,
        0.0,
        propagator.get_satellites().len() / 2
      );
      loop {
        propagator.propagate();
        propagator.update_observer_satellites(&mut observer);
        for sat in propagator.get_satellites() {
          println!("Name: {}", sat.get_name());
          println!("Timestamp: {}", sat.get_time());
          println!(
            "LAT: {}, LON: {}, ALT: {}",
            sat.get_latitude_deg(),
            sat.get_longitude_deg(),
            sat.get_altitude_km(),
          );
        }
        println!("SATS: {}", propagator.get_satellites().len());
        for sat in observer.get_satellites_in_range() {
          println!(
            "NAME: {} CAT: {} OBJ: {} AZ: {} EL: {} RA: {}",
            sat.get_name(),
            sat.get_norad_cat_id(),
            sat.get_object_id(),
            sat.get_azimuth_deg(),
            sat.get_elevation_deg(),
            sat.get_range_km(),
          );
        }
        println!("RANGED: {}", observer.get_satellites_in_range().len());
        thread::sleep(Duration::from_millis(1000));
      }
    }
  }
}

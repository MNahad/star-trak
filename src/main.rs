use serde_json;
use std::io;
use std::io::prelude::*;
use std::env;
use std::thread;
use std::time::Duration;

fn main() {
  let mut data = String::new();
  io::stdin().read_line(&mut data).unwrap_or_default();
  let mut output = io::stdout();
  let args: Vec<String> = env::args().collect();
  let (lat, lon, alt) = (
    args[1].parse::<f64>().unwrap_or_default(),
    args[2].parse::<f64>().unwrap_or_default(),
    args[3].parse::<f64>().unwrap_or_default(),
  );
  let mut propagator = star_trak::init(
    serde_json::from_str(&data).unwrap_or_default(),
    [lat, lon, alt],
  );
  loop {
    star_trak::update(&mut propagator);
    for (idx, sat) in propagator.get_satellites().iter().enumerate() {
      output.write(&format!(
        "NAME: {} LAT: {} LON: {} ALT: {}",
        propagator.get_sat_data(idx).get_name(),
        sat.get_position().lat_deg,
        sat.get_position().lon_deg,
        sat.get_position().alt_km,
      ).into_bytes()).unwrap_or_default();
    }
    thread::sleep(Duration::from_millis(1000));
  }
}

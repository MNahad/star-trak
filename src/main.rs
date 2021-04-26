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
  let args_len = args.len();
  let duration_ms = if args_len >= 2 { args[1].parse::<u64>().unwrap_or(1000) } else { 1000 };
  let coords = if args_len >= 5 {
    [
      args[2].parse::<f64>().unwrap_or_default(),
      args[3].parse::<f64>().unwrap_or_default(),
      args[4].parse::<f64>().unwrap_or_default(),
    ]
  } else { [0.0, 0.0, 0.0] };
  let mut propagator = star_trak::init(
    serde_json::from_str(&data).unwrap_or_default(),
    coords,
  );
  loop {
    star_trak::update(&mut propagator);
    output.write(&format!(
      "{}\n",
      propagator.get_satellites().len(),
    ).into_bytes()).unwrap_or_default();
    for (idx, sat) in propagator.get_satellites().iter().enumerate() {
      let sat_data = propagator.get_sat_data(idx);
      let position = sat.get_position();
      output.write(&format!(
        "{},{},{},{},{},{}\n",
        sat_data.get_object_id(),
        sat_data.get_norad_cat_id(),
        sat_data.get_name(),
        position.lat_deg,
        position.lon_deg,
        position.alt_km,
      ).into_bytes()).unwrap_or_default();
    }
    output.write(&format!(
      "{}\n",
      propagator.get_observer().get_ranged_satellites().len(),
    ).into_bytes()).unwrap_or_default();
    for (key, sat) in propagator.get_observer().get_ranged_satellites().iter() {
      let sat_data = &propagator.get_sgp4_data()[*key];
      let position = sat.get_position();
      output.write(&format!(
        "{},{},{},{},{},{}\n",
        sat_data.get_object_id(),
        sat_data.get_norad_cat_id(),
        sat_data.get_name(),
        position.azimuth_deg,
        position.elevation_deg,
        position.range_km,
      ).into_bytes()).unwrap_or_default();
    }
    thread::sleep(Duration::from_millis(duration_ms));
  }
}

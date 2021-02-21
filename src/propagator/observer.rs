use super::transforms::{Coords, Geodetic, Horizontal};
use super::state::State;
use super::satellite::{Satellite, BaseSat};
use std::collections::HashMap;

pub struct Observer<'a, 'b> {
  pub(super) observer_state: State,
  satellites_in_range: HashMap<u64, RangedSat<'a, 'b>>,
}

impl<'a, 'b> Observer<'a, 'b> {
  pub fn new(lat_deg: f64, lon_deg: f64, alt_km: f64) -> Self {
    Observer {
      observer_state: State {
        position: Coords::Geodetic(Geodetic {
          lat_deg,
          lon_deg,
          alt_km,
        }),
        velocity: None,
        time: 0,
      },
      satellites_in_range: HashMap::new(),
    }
  }
  pub fn get_position(&self) -> &Geodetic {
    let position: Option<&Geodetic>;
    if let Coords::Geodetic(coords) = &self.observer_state.position {
      position = Some(&coords);
    } else {
      position = None;
    }
    position.unwrap_or_else(|| &Geodetic { lat_deg: 0.0, lon_deg: 0.0, alt_km: 0.0 })
  }
  pub fn get_satellites_in_range(&self) -> Vec<&RangedSat> {
    self.satellites_in_range.values().collect()
  }
  pub fn upsert_satellite_in_range(&mut self, sat: RangedSat<'a, 'b>) -> () {
    self.satellites_in_range.insert(sat.linked_data.get_norad_id(), sat);
  }
  pub fn delete_satellite_in_range(&mut self, id: u64) -> () {
    self.satellites_in_range.remove(&id);
  }
}

pub type RangedSat<'a, 'b> = Satellite<'b, BaseSat<'a>>;

impl<'a, 'b> RangedSat<'a, 'b> {
  pub fn get_norad_id(&self) -> u64 {
    self.linked_data.get_norad_id()
  }
  pub fn get_name(&self) -> &str {
    self.linked_data.get_name()
  }
  pub fn get_azimuth_deg(&self) -> f64 {
    self.get_coord(|coords| coords.azimuth_deg)
  }
  pub fn get_elevation_deg(&self) -> f64 {
    self.get_coord(|coords| coords.elevation_deg)
  }
  pub fn get_range_km(&self) -> f64 {
    self.get_coord(|coords| coords.range_km)
  }
  fn get_coord<F: Fn(&Horizontal) -> f64>(&self, cb: F) -> f64 {
    let coord: Option<f64>;
    if let Coords::Horizontal(coords) = &self.state.position {
      coord = Some(cb(coords));
    } else {
      coord = None;
    }
    coord.unwrap_or_default()
  }
}

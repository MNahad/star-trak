use super::{State, Geodetic, Horizontal};
use std::collections::HashMap;

pub struct RangedSatellite {
  pub(super) vector: State<Horizontal>,
  pub(super) linked_data_idx: usize,
}

impl RangedSatellite {
  pub fn get_position(&self) -> &Horizontal {
    &self.vector.position
  }
}

pub struct Observer {
  pub(super) state: State<Geodetic>,
  satellites_in_range: HashMap<usize, RangedSatellite>,
}

impl Observer {
  pub fn from_coords(lat_deg: f64, lon_deg: f64, alt_km: f64, max_sats: usize) -> Self {
    Observer {
      state: State {
        position: Geodetic {
          lat_deg,
          lon_deg,
          alt_km,
        },
        velocity: None,
        time: 0,
      },
      satellites_in_range: HashMap::with_capacity(max_sats),
    }
  }
  pub fn get_ranged_satellites(&self) -> &HashMap<usize, RangedSatellite> {
    &self.satellites_in_range
  }
  pub(super) fn upsert_ranged_satellite(&mut self, sat: RangedSatellite) -> () {
    self.satellites_in_range.insert(sat.linked_data_idx, sat);
  }
  pub(super) fn delete_ranged_satellite(&mut self, id: usize) -> () {
    self.satellites_in_range.remove(&id);
  }
  pub(super) fn update_observer(&mut self, lat_deg: f64, lon_deg: f64, alt_km: f64) -> () {
    self.state.position.lat_deg = lat_deg;
    self.state.position.lon_deg = lon_deg;
    self.state.position.alt_km = alt_km;
  }
}

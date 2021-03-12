use super::state::{State, Geodetic, Horizontal};
use std::collections::HashMap;

pub struct Observer {
  pub(super) state: State<Geodetic>,
  satellites_in_range: HashMap<u64, RangedSatellite>,
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
  pub fn get_position(&self) -> &Geodetic {
    &self.state.position
  }
  pub fn get_satellites_in_range(&self) -> Vec<&RangedSatellite> {
    self.satellites_in_range.values().collect()
  }
  pub fn upsert_satellite_in_range(&mut self, sat: RangedSatellite) -> () {
    self.satellites_in_range.insert(sat.get_norad_cat_id(), sat);
  }
  pub fn delete_satellite_in_range(&mut self, id: u64) -> () {
    self.satellites_in_range.remove(&id);
  }
}

pub struct RangedSatellite {
  pub(super) norad_cat_id: u64,
  pub(super) object_id: String,
  pub(super) name: String,
  pub(super) vector: Horizontal,
}

impl RangedSatellite {
  pub fn get_norad_cat_id(&self) -> u64 {
    self.norad_cat_id
  }
  pub fn get_object_id(&self) -> &str {
    &self.object_id
  }
  pub fn get_name(&self) -> &str {
    &self.name
  }
  pub fn get_azimuth_deg(&self) -> f64 {
    self.vector.azimuth_deg
  }
  pub fn get_elevation_deg(&self) -> f64 {
    self.vector.elevation_deg
  }
  pub fn get_range_km(&self) -> f64 {
    self.vector.range_km
  }
}

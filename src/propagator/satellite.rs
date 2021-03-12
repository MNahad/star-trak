use super::state::{State, Geodetic, Cartesian};
use super::Sgp4Data;

pub struct Satellite<'a> {
  pub(super) state: State<Geodetic>,
  pub(super) state_auxiliary: Option<State<Cartesian>>,
  pub(super) linked_data: &'a Sgp4Data,
}

impl<'a> Satellite<'a> {
  pub fn get_norad_cat_id(&self) -> u64 {
    self.linked_data.get_norad_cat_id()
  }
  pub fn get_object_id(&self) -> &str {
    self.linked_data.get_object_id()
  }
  pub fn get_name(&self) -> &str {
    self.linked_data.get_name()
  }
  pub fn get_latitude_deg(&self) -> f64 {
    self.state.position.lat_deg
  }
  pub fn get_longitude_deg(&self) -> f64 {
    self.state.position.lon_deg
  }
  pub fn get_altitude_km(&self) -> f64 {
    self.state.position.alt_km
  }
  pub fn get_x(&self) -> f64 {
    self.get_aux_coord(|coords| coords.x)
  }
  pub fn get_y(&self) -> f64 {
    self.get_aux_coord(|coords| coords.y)
  }
  pub fn get_z(&self) -> f64 {
    self.get_aux_coord(|coords| coords.z)
  }
  pub fn get_time(&self) -> i64 {
    self.state.time
  }
  fn get_aux_coord<F: Fn(&Cartesian) -> f64>(&self, extract_coord: F) -> f64 {
    match &self.state_auxiliary {
      Some(state) => extract_coord(&state.position),
      None => 0.0,
    }
  }
}

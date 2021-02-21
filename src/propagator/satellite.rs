use super::transforms::{Coords, Geodetic, Cartesian};
use super::state::State;
use super::Elements;

pub struct Satellite<'a, T> {
  pub(super) state: State,
  pub(super) state_auxiliary: Option<State>,
  pub(super) linked_data: &'a T,
}

pub type BaseSat<'a> = Satellite<'a, Elements>;

impl<'a> BaseSat<'a> {
  pub fn get_norad_id(&self) -> u64 {
    self.linked_data.get_norad_id()
  }
  pub fn get_name(&self) -> &str {
    self.linked_data.get_name()
  }
  pub fn get_latitude_deg(&self) -> f64 {
    self.get_coord(|coords| coords.lat_deg)
  }
  pub fn get_longitude_deg(&self) -> f64 {
    self.get_coord(|coords| coords.lon_deg)
  }
  pub fn get_altitude_km(&self) -> f64 {
    self.get_coord(|coords| coords.alt_km)
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
  fn get_coord<F: Fn(&Geodetic) -> f64>(&self, cb: F) -> f64 {
    let coord: Option<f64>;
    if let Coords::Geodetic(coords) = &self.state.position {
      coord = Some(cb(coords));
    } else {
      coord = None;
    }
    coord.unwrap_or_default()
  }
  fn get_aux_coord<F: Fn(&Cartesian) -> f64>(&self, cb: F) -> f64 {
    let coord: Option<f64>;
    if let Some(state) = &self.state_auxiliary {
      if let Coords::Cartesian(coords) = &state.position {
        coord = Some(cb(coords));
      } else {
        coord = None;
      }
      coord.unwrap_or_default()
    } else {
      0.0
    }
  }
}

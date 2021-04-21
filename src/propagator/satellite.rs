use super::{State, Geodetic, Cartesian};

pub struct Satellite {
  pub(super) state: State<Geodetic>,
  pub(super) state_auxiliary: Option<State<Cartesian>>,
  pub(super) linked_data_idx: usize,
}

impl Satellite {
  pub fn get_position(&self) -> &Geodetic {
    &self.state.position
  }
  pub fn get_position_auxiliary(&self) -> &Cartesian {
    match &self.state_auxiliary {
      Some(state) => &state.position,
      None => &Cartesian { x: 0.0, y: 0.0, z: 0.0 },
    }
  }
}

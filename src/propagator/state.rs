use super::transforms::{Coords};

pub struct State {
  pub(super) position: Coords,
  pub(super) velocity: Option<Coords>,
  pub(super) time: i64,
}

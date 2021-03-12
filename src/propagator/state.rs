pub struct State<T> {
  pub(super) position: T,
  pub(super) velocity: Option<T>,
  pub(super) time: i64,
}

pub struct Geodetic {
  pub(super) lat_deg: f64,
  pub(super) lon_deg: f64,
  pub(super) alt_km: f64,
}

pub struct Horizontal {
  pub(super) azimuth_deg: f64,
  pub(super) elevation_deg: f64,
  pub(super) range_km: f64,
}

pub struct Cartesian {
  pub(super) x: f64,
  pub(super) y: f64,
  pub(super) z: f64,
}

use serde::Serialize;

#[derive(Clone, Copy, Serialize)]
pub struct State<T> {
  pub position: T,
  pub velocity: Option<T>,
  pub time: i64,
}

#[derive(Clone, Copy, Serialize)]
pub struct Geodetic {
  pub lat_deg: f64,
  pub lon_deg: f64,
  pub alt_km: f64,
}

#[derive(Clone, Copy, Serialize)]
pub struct Horizontal {
  pub azimuth_deg: f64,
  pub elevation_deg: f64,
  pub range_km: f64,
}

pub struct Cartesian {
  pub x: f64,
  pub y: f64,
  pub z: f64,
}

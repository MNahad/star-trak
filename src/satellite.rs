mod transforms;
use transforms::{Geodetic, Horizontal, Cartesian};
use chrono::{Datelike, Timelike};
use std::collections::HashMap;

struct Coords([f64; 3]);

impl Geodetic for Coords {
  fn new(lat_deg: f64, lon_deg: f64, alt_km: f64) -> Self {
    Coords([lat_deg, lon_deg, alt_km])
  }
  fn get_lat_deg(&self) -> f64 {
    self.0[0]
  }
  fn get_lon_deg(&self) -> f64 {
    self.0[1]
  }
  fn get_alt_km(&self) -> f64 {
    self.0[2]
  }
  fn set(&mut self, lat_deg: f64, lon_deg: f64, alt_km: f64) {
    self.0 = [lat_deg, lon_deg, alt_km];
  }
}

impl Horizontal for Coords {
  fn new(azimuth_deg: f64, elevation_deg: f64, range_km: f64) -> Self {
    Coords([azimuth_deg, elevation_deg, range_km])
  }
  fn get_azimuth_deg(&self) -> f64 {
    self.0[0]
  }
  fn get_elevation_deg(&self) -> f64 {
    self.0[1]
  }
  fn get_range_km(&self) -> f64 {
    self.0[2]
  }
  fn set(&mut self, azimuth_deg: f64, elevation_deg: f64, range_km: f64) {
    self.0 = [azimuth_deg, elevation_deg, range_km];
  }
}

impl Cartesian for Coords {
  fn new(x: f64, y: f64, z: f64) -> Self {
    Coords([x, y, z])
  }
  fn get_x(&self) -> f64 {
    self.0[0]
  }
  fn get_y(&self) -> f64 {
    self.0[1]
  }
  fn get_z(&self) -> f64 {
    self.0[2]
  }
  fn set(&mut self, x: f64, y: f64, z: f64) {
    self.0 = [x, y, z];
  }
}

struct State {
  position: Coords,
  timestamp_s: i64,
}

pub struct Satellite<'a, T> {
  state: State,
  data: &'a T,
}

impl<'a, T> Satellite<'a, T> {
  pub fn get_latitude_deg(&self) -> f64 {
    self.state.position.get_lat_deg()
  }

  pub fn get_longitude_deg(&self) -> f64 {
    self.state.position.get_lon_deg()
  }

  pub fn get_altitude_km(&self) -> f64 {
    self.state.position.get_alt_km()
  }

  pub fn get_azimuth_deg(&self) -> f64 {
    self.state.position.get_azimuth_deg()
  }

  pub fn get_elevation_deg(&self) -> f64 {
    self.state.position.get_elevation_deg()
  }

  pub fn get_range_km(&self) -> f64 {
    self.state.position.get_range_km()
  }

  pub fn get_timestamp_s(&self) -> i64 {
    self.state.timestamp_s
  }

  pub fn get_data(&self) -> &T {
    self.data
  }

  fn update_position(&mut self, position: Coords) -> () {
    self.state.position = position;
  }

  fn update_timestamp(&mut self, timestamp: i64) -> () {
    self.state.timestamp_s = timestamp;
  }
}

type BaseSat<'a> = Satellite<'a, Elements>;
type RangedSat<'a, 'b> = Satellite<'b, BaseSat<'a>>;

pub struct Observer<'a, 'b> {
  observer_state: State,
  satellites_in_range: HashMap<u64, RangedSat<'a, 'b>>,
}

impl<'a, 'b> Observer<'a, 'b> {
  pub fn new(lat: f64, lng: f64, alt: f64) -> Self {
    Observer {
      observer_state: State {
        position: Geodetic::new(lat, lng, alt),
        timestamp_s: 0,
      },
      satellites_in_range: HashMap::new(),
    }
  }
  pub fn get_satellites_in_range(&self) -> Vec<&RangedSat> {
    self.satellites_in_range.values().collect()
  }
  fn upsert_satellite_in_range(&mut self, sat: RangedSat<'a, 'b>) -> () {
    self.satellites_in_range.insert(sat.data.data.get_norad_id(), sat);
  }
  fn delete_satellite_in_range(&mut self, id: u64) -> () {
    self.satellites_in_range.remove(&id);
  }
  fn update_timestamp(&mut self, timestamp: i64) -> () {
    self.observer_state.timestamp_s = timestamp;
  }
}

pub struct Elements(sgp4::Elements);

impl Elements {
  pub fn new(elements: sgp4::Elements) -> Self {
    Elements(elements)
  }
  pub fn get_norad_id(&self) -> u64 {
    self.0.norad_id
  }
  pub fn get_name(&self) -> &str {
    if let Some(name) = self.0.object_name.as_ref() {
      name
    } else {
      "NAME"
    }
  }
  pub fn get_international_designator(&self) -> &str {
    if let Some(designator) = self.0.international_designator.as_ref() {
      designator
    } else {
      "DESIGNATOR"
    }
  }
}

pub fn propagate<'a>(elements_group: &'a Vec<Elements>) -> Vec<BaseSat<'a>> {
  let mut predictions: Vec<BaseSat> = Vec::new();
  for elements in elements_group {
    if let Ok(constants) = sgp4::Constants::from_elements(&elements.0) {
      let timestamp = chrono::Utc::now().naive_utc();
      let elapsed_ms = timestamp.signed_duration_since(elements.0.datetime).num_milliseconds();
      if let Ok(prediction) = constants.propagate((elapsed_ms as f64) / (1000.0 * 60.0)) {
        let position_eci_km: Coords = Cartesian::new(
          prediction.position[0],
          prediction.position[1],
          prediction.position[2],
        );
        let position_geodetic = transforms::eci_to_geodetic(
          &position_eci_km,
          sgp4::iau_epoch_to_sidereal_time(
            elements.0.epoch() + ((elapsed_ms as f64) / (31_557_600.0 * 1000.0))
          )
        );
        predictions.push(BaseSat {
          state: State {
            position: position_geodetic,
            timestamp_s: timestamp.timestamp(),
          },
          data: elements,
        });
      }
    }
  }
  predictions
}

pub fn update_observer_satellites<'a, 'b>(
  observer: &mut Observer<'a, 'b>,
  sats: &'b Vec<BaseSat<'a>>,
) -> () {
  let timestamp = chrono::Utc::now().naive_utc();
  let gmst = sgp4::iau_epoch_to_sidereal_time(epoch(&timestamp));
  let observer_eci: Coords = transforms::geodetic_to_eci(&observer.observer_state.position, gmst);
  let observer_position = [observer_eci.get_x(), observer_eci.get_y(), observer_eci.get_z()];
  for sat in sats {
    let position = [
      sat.state.position.get_lat_deg(),
      sat.state.position.get_lon_deg(),
      sat.state.position.get_alt_km(),
    ];
    let range_eci: Coords = Cartesian::new(
      position[0] - observer_position[0],
      position[1] - observer_position[1],
      position[2] - observer_position[2],
    );
    let range_enu: Coords = transforms::eci_to_topocentric_enu(
      &range_eci,
      observer_position[0],
      observer_position[1],
      gmst,
    );
    let range_aer = transforms::enu_to_aer(&range_enu);
    if range_enu.get_z() > 0.0 {
      observer.upsert_satellite_in_range(RangedSat {
        state: State {
          position: range_aer,
          timestamp_s: sat.state.timestamp_s,
        },
        data: sat,
      });
    } else {
      observer.delete_satellite_in_range(sat.data.get_norad_id());
    }
  }
  observer.update_timestamp(timestamp.timestamp());
}

// Based on
// https://crates.io/crates/sgp4
fn epoch(datetime: &chrono::NaiveDateTime) -> f64 {
  (
    367 * datetime.year() as i32
    - (7 * (datetime.year() as i32 + (datetime.month() as i32 + 9) / 12)) / 4
    + 275 * datetime.month() as i32 / 9
    + datetime.day() as i32
    - 730531
  ) as f64 / 365.25
  + (datetime.num_seconds_from_midnight() as i32 - 43200) as f64 / (24.0 * 60.0 * 60.0 * 365.25)
  + (datetime.nanosecond() as f64) / (24.0 * 60.0 * 60.0 * 1e9 * 365.25)
}

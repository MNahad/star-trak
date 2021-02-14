mod transforms;
use transforms::{Geodetic, Horizontal, Cartesian};
use chrono::{Datelike, Timelike};
use std::collections::HashMap;

pub struct Satellite<'a> {
  timing: Timing,
  coords: Coords,
  elements: &'a sgp4::Elements,
}

impl<'a> Satellite<'a> {
  pub fn get_norad_id(&self) -> u64 {
    self.elements.norad_id
  }
  pub fn get_name(&self) -> &str {
    if let Some(name) = self.elements.object_name.as_ref() {
      name
    } else {
      "NAME"
    }
  }
  pub fn get_international_designator(&self) -> &str {
    if let Some(designator) = self.elements.international_designator.as_ref() {
      designator
    } else {
      "DESIGNATOR"
    }
  }
  pub fn get_geodetic_position(&self) -> &Geodetic {
    &self.coords.position_geodetic
  }
  pub fn get_timestamp_s(&self) -> i64 {
    self.timing.timestamp_s
  }
  fn update_coords(&mut self, coords: Coords) -> () {
    self.coords = coords;
  }
  fn update_timing(&mut self, timing: Timing) -> () {
    self.timing = timing;
  }
}

pub struct Observer<'a, 'b> {
  observer_position: Geodetic,
  observer_timestamp_s: i64,
  satellites_in_los: HashMap<u64, RangedSatellite<'a, 'b>>,
}

impl<'a, 'b> Observer<'a, 'b> {
  pub fn new(lat_lng_alt: [f64; 3], timestamp_s: i64) -> Self {
    Observer {
      observer_position: Geodetic {
        lat_deg: lat_lng_alt[0],
        lon_deg: lat_lng_alt[1],
        alt_km: lat_lng_alt[2],
      },
      observer_timestamp_s: timestamp_s,
      satellites_in_los: HashMap::new(),
    }
  }
  pub fn get_satellites_in_los(&self) -> Vec<&RangedSatellite> {
    self.satellites_in_los.values().collect()
  }
  fn upsert_satellite_in_los(&mut self, sat: RangedSatellite<'a, 'b>) -> () {
    self.satellites_in_los.insert(sat.satellite.get_norad_id(), sat);
  }
  fn delete_satellite_in_los(&mut self, id: u64) -> () {
    self.satellites_in_los.remove(&id);
  }
  fn update_timestamp(&mut self, timestamp: i64) -> () {
    self.observer_timestamp_s = timestamp;
  }
}

pub struct RangedSatellite<'a, 'b> {
  satellite_position: Horizontal,
  satellite: &'b Satellite<'a>,
}

impl<'a, 'b> RangedSatellite<'a, 'b> {
  pub fn get_position(&self) -> &Horizontal {
    &self.satellite_position
  }
  pub fn get_satellite_info(&self) -> &Satellite {
    self.satellite
  }
}

struct Coords {
  position_geodetic: Geodetic,
  position_eci_km: Cartesian,
  velocity_eci_km_s: Cartesian,
}

struct Timing {
  timestamp_s: i64,
  elapsed_since_epoch_ms: i64,
}

pub fn propagate(
  elements_group: &Vec<sgp4::Elements>,
) -> Result<Vec<Satellite>, Box<dyn std::error::Error>> {
  let mut predictions: Vec<Satellite> = Vec::new();
  for elements in elements_group {
    if let Ok(constants) = sgp4::Constants::from_elements(elements) {
      let timestamp = chrono::Utc::now().naive_utc();
      let elapsed_ms = timestamp.signed_duration_since(elements.datetime).num_milliseconds();
      if let Ok(prediction) = constants.propagate((elapsed_ms as f64) / (1000.0 * 60.0)) {
        let position_eci_km = Cartesian {
          x: prediction.position[0],
          y: prediction.position[1],
          z: prediction.position[2],
        };
        let velocity_eci_km_s = Cartesian {
          x: prediction.velocity[0],
          y: prediction.velocity[1],
          z: prediction.velocity[2],
        };
        let position_geodetic = transforms::eci_to_geodetic(
          &position_eci_km,
          sgp4::iau_epoch_to_sidereal_time(
            elements.epoch() + ((elapsed_ms as f64) / (31_557_600.0 * 1000.0))
          )
        );
        predictions.push(Satellite {
          coords: Coords {
            position_eci_km,
            velocity_eci_km_s,
            position_geodetic,
          },
          timing: Timing {
            timestamp_s: timestamp.timestamp(),
            elapsed_since_epoch_ms: elapsed_ms,
          },
          elements,
        });
      }
    }
  }
  Ok(predictions)
}

pub fn update_observer_satellites<'a, 'b>(
  observer: &mut Observer<'a, 'b>,
  sats: &'b Vec<Satellite<'a>>
) -> () {
  let timestamp = chrono::Utc::now().naive_utc();
  let gmst = sgp4::iau_epoch_to_sidereal_time(epoch(&timestamp));
  let observer_eci = transforms::geodetic_to_eci(&observer.observer_position, gmst);
  for sat in sats {
    let range_eci = Cartesian {
      x: sat.coords.position_eci_km.x - observer_eci.x,
      y: sat.coords.position_eci_km.y - observer_eci.y,
      z: sat.coords.position_eci_km.z - observer_eci.z,
    };
    let range_enu = transforms::eci_to_topocentric_enu(
      &range_eci,
      observer.observer_position.lat_deg,
      observer.observer_position.lon_deg,
      gmst,
    );
    let range_aer = transforms::enu_to_aer(&range_enu);
    if range_enu.z > 0.0 {
      observer.upsert_satellite_in_los(RangedSatellite {
        satellite_position: range_aer,
        satellite: sat,
      });
    } else {
      observer.delete_satellite_in_los(sat.get_norad_id());
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

mod state;
mod satellite;
mod observer;
mod transforms;
pub use observer::Observer;
use observer::RangedSat;
use satellite::BaseSat;
use state::State;
use transforms::{Coords, Cartesian};
use chrono::{Datelike, Timelike};

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
        let position_eci_km = Cartesian {
          x: prediction.position[0],
          y: prediction.position[1],
          z: prediction.position[2],
        };
        let state_geodetic = transforms::eci_to_geodetic(
          &position_eci_km,
          sgp4::iau_epoch_to_sidereal_time(
            elements.0.epoch() + ((elapsed_ms as f64) / (31_557_600.0 * 1000.0))
          ),
        );
        predictions.push(BaseSat {
          state: State {
            position: Coords::Geodetic(state_geodetic),
            velocity: None,
            time: timestamp.timestamp(),
          },
          state_auxiliary: Some(State {
            position: Coords::Cartesian(position_eci_km),
            velocity: Some(Coords::Cartesian(Cartesian {
              x: prediction.velocity[0],
              y: prediction.velocity[1],
              z: prediction.velocity[2],
            })),
            time: timestamp.timestamp(),
          }),
          linked_data: elements,
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
  let observer_eci = transforms::geodetic_to_eci(observer.get_position(), gmst);
  for sat in sats {
    let range_eci = Cartesian {
      x: sat.get_x() - observer_eci.x,
      y: sat.get_y() - observer_eci.y,
      z: sat.get_z() - observer_eci.z,
    };
    let range_enu = transforms::eci_to_topocentric_enu(
      &range_eci,
      observer.get_position().lat_deg,
      observer.get_position().lon_deg,
      gmst,
    );
    if range_enu.z > 0.0 {
      let range_aer = transforms::enu_to_aer(&range_enu);
      observer.upsert_satellite_in_range(RangedSat {
        state: State {
          position: Coords::Horizontal(range_aer),
          velocity: None,
          time: timestamp.timestamp()
        },
        state_auxiliary: None,
        linked_data: sat,
      });
    } else {
      observer.delete_satellite_in_range(sat.get_norad_id());
    }
  }
  observer.observer_state.time = timestamp.timestamp();
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

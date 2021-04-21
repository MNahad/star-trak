mod satellite;
mod observer;
mod transforms;

use super::state::{State, Geodetic, Cartesian, Horizontal};
use satellite::Satellite;
use observer::{Observer, RangedSatellite};
use chrono::{Utc, Datelike, Timelike, TimeZone};

pub struct Sgp4Data(sgp4::Elements, sgp4::Constants<'static>);

impl Sgp4Data {
  pub fn from_elements(elements: sgp4::Elements) -> Option<Self> {
    match sgp4::Constants::from_elements(&elements) {
      Ok(constants) => Some(Sgp4Data(elements, constants)),
      _ => None,
    }
  }
  pub fn get_name(&self) -> &str {
    match self.0.object_name.as_ref() {
      Some(name) => name,
      _ => "UNKNOWN",
    }
  }
  pub fn get_object_id(&self) -> &str {
    match self.0.international_designator.as_ref() {
      Some(designator) => designator,
      _ => "UNKNOWN",
    }
  }
}

pub struct Propagator(Vec<Satellite>, Observer, Vec<Sgp4Data>);

impl Propagator {
  pub fn from_data(sgp4_data: Vec<Sgp4Data>, observer_coords: [f64; 3]) -> Self {
    Propagator(
      sgp4_data.iter().enumerate().map(|(idx, _)| Satellite {
        state: State {
          position: Geodetic { lat_deg: 0.0, lon_deg: 0.0, alt_km: 0.0 },
          velocity: None,
          time: 0,
        },
        state_auxiliary: None,
        linked_data_idx: idx,
      }).collect(),
      Observer::from_coords(
        observer_coords[0],
        observer_coords[1],
        observer_coords[2],
        sgp4_data.len() / 4,
      ),
      sgp4_data,
    )
  }
  pub fn get_satellites(&self) -> &Vec<Satellite> {
    &self.0
  }
  pub fn get_observer(&self) -> &Observer {
    &self.1
  }
  pub fn get_sat_data(&self, idx: usize) -> &Sgp4Data {
    &self.2[self.0[idx].linked_data_idx]
  }
  pub fn propagate(&mut self) -> () {
    for sat in self.0.iter_mut() {
      let Sgp4Data(elements, constants) = &self.2[sat.linked_data_idx];
      let timestamp = Utc::now().naive_utc();
      let elapsed_ms = timestamp.signed_duration_since(elements.datetime).num_milliseconds();
      if let Ok(prediction) = constants.propagate((elapsed_ms as f64) / (1000.0 * 60.0)) {
        let position_eci_km = Cartesian {
          x: prediction.position[0],
          y: prediction.position[1],
          z: prediction.position[2],
        };
        let position_geodetic = transforms::eci_to_geodetic(
          &position_eci_km,
          sgp4::iau_epoch_to_sidereal_time(
            elements.epoch() + ((elapsed_ms as f64) / (31_557_600.0 * 1000.0))
          ),
        );
        sat.state = State {
          position: position_geodetic,
          velocity: None,
          time: timestamp.timestamp(),
        };
        sat.state_auxiliary = Some(State {
          position: position_eci_km,
          velocity: Some(Cartesian {
            x: prediction.velocity[0],
            y: prediction.velocity[1],
            z: prediction.velocity[2],
          }),
          time: timestamp.timestamp(),
        });
      }
    }
  }
  pub fn update_observer_satellites(&mut self) -> () {
    let Propagator(satellites, observer, _) = self;
    for sat in satellites.iter() {
      let timestamp = Utc.timestamp(sat.state.time, 0).naive_utc();
      let gmst = sgp4::iau_epoch_to_sidereal_time(epoch(&timestamp));
      let observer_eci = transforms::geodetic_to_eci(&observer.state.position, gmst);
      let range_eci = Cartesian {
        x: sat.get_position_auxiliary().x - observer_eci.x,
        y: sat.get_position_auxiliary().y - observer_eci.y,
        z: sat.get_position_auxiliary().z - observer_eci.z,
      };
      let range_enu = transforms::eci_to_topocentric_enu(
        &range_eci,
        observer.state.position.lat_deg,
        observer.state.position.lon_deg,
        gmst,
      );
      if range_enu.z > 0.0 {
        let range_aer = transforms::enu_to_aer(&range_enu);
        observer.upsert_ranged_satellite(RangedSatellite {
          vector: State {
            position: range_aer,
            velocity: None,
            time: timestamp.timestamp(),
          },
          linked_data_idx: sat.linked_data_idx,
        });
      } else {
        observer.delete_ranged_satellite(sat.linked_data_idx);
      }
    }
    observer.state.time = Utc::now().naive_utc().timestamp();
  }
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

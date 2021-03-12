mod state;
mod satellite;
mod observer;
mod transforms;

pub use observer::Observer;
use observer::RangedSatellite;
use satellite::Satellite;
use state::{State, Geodetic, Cartesian};
use chrono::{Datelike, Timelike};

pub struct Sgp4Data(sgp4::Elements, sgp4::Constants<'static>);

impl Sgp4Data {
  pub fn from_elements(elements: sgp4::Elements) -> Option<Self> {
    match sgp4::Constants::from_elements(&elements) {
      Ok(constants) => Some(Sgp4Data(elements, constants)),
      _ => None,
    }
  }
  pub fn get_norad_cat_id(&self) -> u64 {
    self.0.norad_id
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

pub struct Propagator<'a>(Vec<Satellite<'a>>);

impl<'a> Propagator<'a> {
  pub fn from_sgp4_data(sgp4_data: &'a Vec<Sgp4Data>) -> Self {
    Propagator(sgp4_data.iter().map(|data| Satellite {
      state: State {
        position: Geodetic { lat_deg: 0.0, lon_deg: 0.0, alt_km: 0.0 },
        velocity: None,
        time: 0,
      },
      state_auxiliary: None,
      linked_data: data,
    }).collect())
  }
  pub fn get_satellites(&self) -> &Vec<Satellite<'a>> {
    &self.0
  }
  pub fn propagate(&mut self) -> () {
    for sat in self.0.iter_mut() {
      let Sgp4Data(elements, constants) = sat.linked_data;
      let timestamp = chrono::Utc::now().naive_utc();
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
  pub fn update_observer_satellites(&self, observer: &mut Observer) -> () {
    let timestamp = chrono::Utc::now().naive_utc();
    let gmst = sgp4::iau_epoch_to_sidereal_time(epoch(&timestamp));
    let observer_eci = transforms::geodetic_to_eci(observer.get_position(), gmst);
    for sat in self.0.iter() {
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
        observer.upsert_satellite_in_range(RangedSatellite {
          norad_cat_id: sat.get_norad_cat_id(),
          object_id: sat.get_object_id().to_owned(),
          name: sat.get_name().to_owned(),
          vector: range_aer,
        });
      } else {
        observer.delete_satellite_in_range(sat.get_norad_cat_id());
      }
    }
    observer.state.time = timestamp.timestamp();
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

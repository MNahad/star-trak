mod state;
mod transforms;

use chrono::{Datelike, TimeZone, Timelike, Utc};
use sgp4::Elements;
pub use state::State;
use state::{Constellation, Observers, RangedConstellation};

pub enum StateType {
    PositionGeodetic,
    PositionAER(usize),
    VelocityENU(usize),
}

pub struct Engine(Constellation, Observers);

impl Engine {
    pub fn from_data(elements_vec: Vec<Elements>, observers_states: Vec<State>) -> Self {
        let constellation = Constellation::from_elements(elements_vec);
        let constellation_len = constellation.sgp4_data.len();
        Engine(
            constellation,
            Observers::from_states(observers_states, constellation_len),
        )
    }
    pub fn update_observer_states(&mut self, observer_states: Vec<State>) -> () {
        let current_len = self.1.states.len();
        let new_len = observer_states.len();
        let constellation_len = self.0.sgp4_data.len();
        for (i, &state) in observer_states.iter().enumerate() {
            if i >= current_len {
                break;
            }
            self.1.states[i] = state;
        }
        if new_len < current_len {
            self.1.states.truncate(new_len);
            self.1.constellations.truncate(new_len);
        } else if new_len > current_len {
            self.1.states.extend(observer_states[current_len..].iter());
            for _ in observer_states[current_len..].iter() {
                self.1
                    .constellations
                    .push(RangedConstellation::new(constellation_len));
            }
        }
    }
    pub fn propagate(&mut self) -> () {
        for (i, (elements, constants)) in self.0.sgp4_data.iter().enumerate() {
            let timestamp = Utc::now().naive_utc();
            let elapsed_ms = timestamp
                .signed_duration_since(elements.datetime)
                .num_milliseconds();
            if let Ok(prediction) = constants.propagate((elapsed_ms as f64) / (1000.0 * 60.0)) {
                let position_eci_km = State(
                    prediction.position[0],
                    prediction.position[1],
                    prediction.position[2],
                );
                let velocity_eci_km = State(
                    prediction.velocity[0],
                    prediction.velocity[1],
                    prediction.velocity[2],
                );
                let position_geodetic = transforms::eci_to_geodetic(
                    &position_eci_km,
                    sgp4::iau_epoch_to_sidereal_time(
                        elements.epoch() + ((elapsed_ms as f64) / (31_557_600.0 * 1000.0)),
                    ),
                );
                self.0.positions_eci[i] = position_eci_km;
                self.0.velocities_eci[i] = velocity_eci_km;
                self.0.positions_geodetic[i] = position_geodetic;
                self.0.times[i] = timestamp.timestamp();
            }
        }
    }
    pub fn update_observers_satellites(&mut self) -> () {
        let Constellation {
            positions_eci,
            velocities_eci,
            times,
            ..
        } = &self.0;
        for (observer_counter, observer_state) in self.1.states.iter().enumerate() {
            let RangedConstellation {
                positions_aer,
                velocities_enu,
            } = &mut self.1.constellations[observer_counter];
            for satellite_counter in 0..positions_aer.len() {
                let sat_position_eci = &positions_eci[satellite_counter];
                let sat_velocity_eci = &velocities_eci[satellite_counter];
                let update_time = times[satellite_counter];
                let (position_aer, velocity_enu) = Engine::get_topocentric_state(
                    observer_state,
                    sat_position_eci,
                    sat_velocity_eci,
                    update_time,
                );
                positions_aer[satellite_counter] = position_aer;
                velocities_enu[satellite_counter] = velocity_enu;
            }
        }
    }
    pub fn get_norad_ids(&self) -> Vec<u64> {
        self.0
            .sgp4_data
            .iter()
            .map(|data| data.0.norad_id)
            .collect()
    }
    pub fn get_states(&self, state_type: StateType) -> &Vec<State> {
        match state_type {
            StateType::PositionGeodetic => &self.0.positions_geodetic,
            StateType::PositionAER(observer_number) => {
                &self.1.constellations[observer_number].positions_aer
            }
            StateType::VelocityENU(observer_number) => {
                &self.1.constellations[observer_number].velocities_enu
            }
        }
    }
    pub fn get_timestamps(&self) -> &Vec<i64> {
        &self.0.times
    }
    fn get_topocentric_state(
        observer_state: &State,
        sat_position_eci: &State,
        sat_velocity_eci: &State,
        update_time: i64,
    ) -> (State, State) {
        let timestamp = Utc.timestamp(update_time, 0).naive_utc();
        let gmst = sgp4::iau_epoch_to_sidereal_time(epoch(&timestamp));
        let observer_eci = transforms::geodetic_to_eci(observer_state, gmst);
        let disp_eci = State(
            sat_position_eci.0 - observer_eci.0,
            sat_position_eci.1 - observer_eci.1,
            sat_position_eci.2 - observer_eci.2,
        );
        let position_enu =
            transforms::eci_to_topocentric_enu(&disp_eci, observer_state.0, observer_state.1, gmst);
        let velocity_enu = transforms::eci_to_topocentric_enu(
            sat_velocity_eci,
            observer_state.0,
            observer_state.1,
            gmst,
        );
        let position_aer = transforms::enu_to_aer(&position_enu);
        (position_aer, velocity_enu)
    }
}

/// Based on
/// https://crates.io/crates/sgp4
fn epoch(datetime: &chrono::NaiveDateTime) -> f64 {
    (367 * datetime.year() as i32
        - (7 * (datetime.year() as i32 + (datetime.month() as i32 + 9) / 12)) / 4
        + 275 * datetime.month() as i32 / 9
        + datetime.day() as i32
        - 730531) as f64
        / 365.25
        + (datetime.num_seconds_from_midnight() as i32 - 43200) as f64
            / (24.0 * 60.0 * 60.0 * 365.25)
        + (datetime.nanosecond() as f64) / (24.0 * 60.0 * 60.0 * 1e9 * 365.25)
}

mod engine;

use engine::{Engine, State, StateType};
use sgp4::Elements;
#[cfg(feature = "js-api")]
use wasm_bindgen::prelude::*;

pub fn init(
    elements_group: Vec<Elements>,
    (observer_lat, observer_lon, observer_alt): (f64, f64, f64),
) -> Engine {
    Engine::from_data(
        elements_group,
        vec![State(observer_lat, observer_lon, observer_alt)],
    )
}

pub fn update_observer_state(engine: &mut Engine, (lat, lon, alt): (f64, f64, f64)) -> () {
    engine.update_observer_states(vec![State(lat, lon, alt)])
}

pub fn update(engine: &mut Engine) -> () {
    engine.propagate();
    engine.update_observers_satellites();
}

pub fn get_constellation_geodetic_states(engine: &Engine) -> &Vec<State> {
    engine.get_states(StateType::PositionGeodetic)
}

pub fn get_observer_constellations(engine: &Engine) -> (&Vec<State>, &Vec<State>) {
    (
        engine.get_states(StateType::PositionAER(0)),
        engine.get_states(StateType::VelocityENU(0)),
    )
}

#[cfg(feature = "js-api")]
#[wasm_bindgen]
pub struct Service(Engine);

#[cfg(feature = "js-api")]
#[wasm_bindgen]
impl Service {
    #[wasm_bindgen(constructor)]
    pub fn new(data: &str, observer_lat: f64, observer_lon: f64, observer_alt: f64) -> Self {
        let elements_group = serde_json::from_str::<Vec<Elements>>(data).expect("INVALID DATA");
        Service(init(
            elements_group,
            (observer_lat, observer_lon, observer_alt),
        ))
    }
    pub fn get_norad_ids(&self) -> Vec<u64> {
        self.0.get_norad_ids()
    }
    pub fn update(&mut self) -> Vec<f64> {
        update(&mut self.0);
        let geodetic_states = get_constellation_geodetic_states(&self.0);
        let (ranged_positions, ranged_velocities) = get_observer_constellations(&self.0);
        geodetic_states
            .iter()
            .chain(ranged_positions.iter())
            .chain(ranged_velocities.iter())
            .flat_map(|&State(val1, val2, val3)| [val1, val2, val3])
            .collect()
    }
    pub fn update_observer(&mut self, lat_deg: f64, lon_deg: f64, alt_km: f64) -> () {
        update_observer_state(&mut self.0, (lat_deg, lon_deg, alt_km));
    }
}

mod state;
mod propagator;

use propagator::{Propagator, Sgp4Data};
#[cfg(feature = "js-api")]
use state::{Geodetic, Horizontal};
#[cfg(feature = "js-api")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "js-api")]
#[wasm_bindgen]
pub struct Service(Propagator);

#[cfg(feature = "js-api")]
#[wasm_bindgen]
impl Service {
  #[wasm_bindgen(constructor)]
  pub fn new(data: &str, observer_lat: f32, observer_lon: f32, observer_alt: f32) -> Self {
    let elements_group = serde_json::from_str::<Vec<sgp4::Elements>>(data).expect("INVALID DATA");
    Service(init(elements_group, [observer_lat.into(), observer_lon.into(), observer_alt.into()]))
  }
  pub fn update(&mut self) -> JsValue {
    update(&mut self.0);
    JsValue::from_serde(&(
      self.0.get_satellites().iter().map(
        |sat| (*sat.get_position(), self.0.get_sat_data(sat.get_linked_idx()).get_name()),
      ).collect::<Vec<(Geodetic, &str)>>(),
      self.0.get_observer().get_ranged_satellites().iter().map(
        |(_, sat)| (*sat.get_position(), self.0.get_sat_data(sat.get_linked_idx()).get_name()),
      ).collect::<Vec<(Horizontal, &str)>>(),
    )).expect("CANNOT SERIALISE")
  }
  pub fn update_observer(&mut self, lat_deg: f32, lon_deg: f32, alt_km: f32) -> () {
    update_observer(&mut self.0, lat_deg.into(), lon_deg.into(), alt_km.into());
  }
}

pub fn init(mut elements_group: Vec<sgp4::Elements>, observer_coords: [f64; 3]) -> Propagator {
  let sgp4_data = elements_group
    .drain(0..)
    .filter_map(|elements| {
      Sgp4Data::from_elements(elements)
    })
    .collect();
  Propagator::from_data(sgp4_data, observer_coords)
}

pub fn update(propagator: &mut Propagator) -> () {
  propagator.propagate();
  propagator.update_observer_satellites();
}

pub fn update_observer(
  propagator: &mut Propagator,
  lat_deg: f64,
  lon_deg: f64,
  alt_km: f64,
) -> () {
  propagator.update_observer(lat_deg, lon_deg, alt_km);
}

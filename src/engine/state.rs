use sgp4::{Constants, Elements};
use std::fmt::{Display, Error, Formatter};
use std::result::Result;

#[derive(Copy, Clone)]
pub struct State(pub f64, pub f64, pub f64);

impl Display for State {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "({:.6},{:.6},{:.6})", self.0, self.1, self.2)
    }
}

pub struct Constellation {
    pub(super) sgp4_data: Vec<(Elements, Constants<'static>)>,
    pub(super) positions_geodetic: Vec<State>,
    pub(super) positions_eci: Vec<State>,
    pub(super) velocities_eci: Vec<State>,
    pub(super) times: Vec<i64>,
}

impl Constellation {
    pub fn from_elements(mut elements_vec: Vec<Elements>) -> Self {
        let sgp4_data = elements_vec
            .drain(0..)
            .filter_map(|elements| match Constants::from_elements(&elements) {
                Ok(constants) => Some((elements, constants)),
                _ => None,
            })
            .collect::<Vec<(Elements, Constants<'static>)>>();
        let len = sgp4_data.len();
        Constellation {
            sgp4_data,
            positions_geodetic: new_state_vector(len),
            positions_eci: new_state_vector(len),
            velocities_eci: new_state_vector(len),
            times: vec![0; len],
        }
    }
}

pub struct Observers {
    pub(super) states: Vec<State>,
    pub(super) constellations: Vec<RangedConstellation>,
}

impl Observers {
    pub fn from_states(observers_states: Vec<State>, constellation_len: usize) -> Self {
        let len = observers_states.len();
        Observers {
            states: observers_states,
            constellations: vec![
                RangedConstellation {
                    positions_aer: new_state_vector(constellation_len),
                    velocities_enu: new_state_vector(constellation_len),
                };
                len
            ],
        }
    }
}

#[derive(Clone)]
pub struct RangedConstellation {
    pub(super) positions_aer: Vec<State>,
    pub(super) velocities_enu: Vec<State>,
}

impl RangedConstellation {
    pub fn new(len: usize) -> Self {
        RangedConstellation {
            positions_aer: new_state_vector(len),
            velocities_enu: new_state_vector(len),
        }
    }
}

fn new_state_vector(len: usize) -> Vec<State> {
    vec![State(0.0, 0.0, 0.0); len]
}

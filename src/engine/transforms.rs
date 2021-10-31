use super::State;
use std::f64::consts::PI;

pub fn eci_to_geodetic(&State(x, y, z): &State, gmst: f64) -> State {
    let theta = y.atan2(x);
    let theta = if theta < 0.0 { theta + 2.0 * PI } else { theta };
    let lambda_e = theta - gmst;
    let lambda_e = if lambda_e > PI {
        lambda_e - 2.0 * PI
    } else {
        lambda_e
    };
    let lambda_e = if lambda_e < -PI {
        lambda_e + 2.0 * PI
    } else {
        lambda_e
    };
    let lon_deg = lambda_e.to_degrees();
    let r_km = (x.powi(2) + y.powi(2)).sqrt();
    let (lat_deg, alt_km) = compute_geodetic_coords_2d(r_km, z);
    State(lat_deg, lon_deg, alt_km)
}

pub fn geodetic_to_eci(&State(lat_deg, lon_deg, alt_km): &State, gmst: f64) -> State {
    const A: f64 = 6378137.0;
    const E_SQ: f64 = 0.00669437999014;
    let cos_phi = lat_deg.to_radians().cos();
    let sin_phi = lat_deg.to_radians().sin();
    let cc = A / (1.0 - E_SQ * sin_phi.powi(2)).sqrt();
    let theta = gmst + lon_deg.to_radians();
    State(
        ((cc + alt_km * 1000.0) * cos_phi * theta.cos()) * 0.001,
        ((cc + alt_km * 1000.0) * cos_phi * theta.sin()) * 0.001,
        ((cc * (1.0 - E_SQ) + alt_km * 1000.0) * sin_phi) * 0.001,
    )
}

pub fn eci_to_topocentric_enu(
    &State(x, y, z): &State,
    lat_deg: f64,
    lon_deg: f64,
    gmst: f64,
) -> State {
    let theta = gmst + lon_deg.to_radians();
    let s_lat = lat_deg.to_radians().sin();
    let s_lon = theta.sin();
    let c_lat = lat_deg.to_radians().cos();
    let c_lon = theta.cos();
    let m: [[f64; 3]; 3] = [
        [-s_lon, c_lon, 0.0],
        [-s_lat * c_lon, -s_lat * s_lon, c_lat],
        [c_lat * c_lon, c_lat * s_lon, s_lat],
    ];
    State(
        m[0][0] * x + m[0][1] * y + m[0][2] * z,
        m[1][0] * x + m[1][1] * y + m[1][2] * z,
        m[2][0] * x + m[2][1] * y + m[2][2] * z,
    )
}

pub fn enu_to_aer(&State(x, y, z): &State) -> State {
    let range_km = (x.powi(2) + y.powi(2) + z.powi(2)).sqrt();
    State(
        (x).atan2(y).to_degrees(),
        (z / range_km).asin().to_degrees(),
        range_km,
    )
}

/// Refer to
/// J. Zhu, "Conversion of Earth-centered Earth-fixed coordinates to geodetic coordinates,"
/// IEEE Transactions on Aerospace and Electronic Systems, vol 30, pp 957-961, 1994.
fn compute_geodetic_coords_2d(r_km: f64, z_km: f64) -> (f64, f64) {
    const A_SQ: f64 = 40680631590769.0;
    const B_SQ: f64 = 40408299984087.0;
    const E_SQ: f64 = 0.00669437999014;
    const E_TWO_SQ: f64 = 0.00673949674228;
    let r = r_km * 1000.0;
    let r_sq = r.powi(2);
    let z = z_km * 1000.0;
    let z_sq = z.powi(2);
    let ee_sq = A_SQ - B_SQ;
    let ff = 54.0 * B_SQ * z_sq;
    let gg = r_sq + ((1.0 - E_SQ) * z_sq) - (E_SQ * ee_sq);
    let cc = (E_SQ.powi(2) * ff * r_sq) / gg.powi(3);
    let ss = (1.0 + cc + (cc.powi(2) + 2.0 * cc).sqrt()).cbrt();
    let pp = ff / (3.0 * (ss + ss.recip() + 1.0).powi(2) * gg.powi(2));
    let qq = (1.0 + 2.0 * E_SQ.powi(2) * pp).sqrt();
    let r_o = ((-(pp * E_SQ * r)) / (1.0 + qq))
        + ((0.5 * A_SQ * (1.0 + qq.recip()))
            - ((pp * (1.0 - E_SQ) * z_sq) / (qq * (1.0 + qq)))
            - (0.5 * pp * r_sq))
            .sqrt();
    let uu = ((r - E_SQ * r_o).powi(2) + z_sq).sqrt();
    let vv = ((r - E_SQ * r_o).powi(2) + (1.0 - E_SQ) * z_sq).sqrt();
    let z_o = B_SQ * z / (A_SQ.sqrt() * vv);
    let lat_deg = (z + E_TWO_SQ * z_o).atan2(r).to_degrees();
    let alt_km = uu * (1.0 - z_o / z) * 0.001;
    (lat_deg, alt_km)
}

#[cfg(test)]
mod tests {
    use super::State;

    fn assert_eq(
        &State(left_1, left_2, left_3): &State,
        &State(right_1, right_2, right_3): &State,
    ) {
        assert!((left_1 - right_1).abs() <= 1e-6);
        assert!((left_2 - right_2).abs() <= 1e-6);
        assert!((left_3 - right_3).abs() <= 1e-6);
    }

    #[test]
    fn it_converts_eci_to_geodetic() {
        let eci = State(-1040.847789, 6686.458279, 3670.305288);
        let gmst = 3.132033;
        let geodetic = State(28.608389, -80.604333, 1325.000000);
        let geo_result = super::eci_to_geodetic(&eci, gmst);
        assert_eq(&geo_result, &geodetic);
    }

    #[test]
    fn it_converts_geodetic_to_eci() {
        let geodetic = State(28.608389, -80.604333, 1325.000000);
        let gmst = 3.132033;
        let eci = State(-1040.847789, 6686.458279, 3670.305288);
        let eci_result = super::geodetic_to_eci(&geodetic, gmst);
        assert_eq(&eci_result, &eci);
    }

    #[test]
    fn it_converts_eci_to_aer() {
        let eci = State(
            -1040.847789 - 2558.531817,
            6686.458279 - 4639.454865,
            3670.305288 - 3539.150874,
        );
        let observer_lat_lng = [33.920700, -118.327800];
        let gmst = 3.132033;
        let aer = State(88.913626, 1.635970, 4142.820054);
        let aer_result = super::enu_to_aer(&super::eci_to_topocentric_enu(
            &eci,
            observer_lat_lng[0],
            observer_lat_lng[1],
            gmst,
        ));
        assert_eq(&aer_result, &aer);
    }
}

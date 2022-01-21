use serde_json;
use std::env;
use std::fmt::Display;
use std::io;
use std::io::prelude::*;
use std::io::Stdout;
use std::thread;
use std::time::Duration;

fn write<T: Display>(output: &mut Stdout, data: T) -> () {
    output
        .write(&format!("{}", data).into_bytes())
        .unwrap_or_default();
}

fn write_vec<T: Copy + Display>(output: &mut Stdout, vec: &Vec<T>) -> () {
    write(output, "[");
    for (i, &data) in vec.iter().enumerate() {
        write(output, data);
        if i < vec.len() - 1 {
            write(output, ",");
        }
    }
    write(output, "]");
}

fn main() {
    let mut data = String::new();
    io::stdin().read_line(&mut data).unwrap_or_default();
    let mut output = io::stdout();
    let args: Vec<String> = env::args().collect();
    let args_len = args.len();
    let duration_ms = if args_len >= 2 {
        args[1].parse::<u64>().unwrap_or(1000)
    } else {
        1000
    };
    let coords = if args_len >= 5 {
        (
            args[2].parse::<f64>().unwrap_or_default(),
            args[3].parse::<f64>().unwrap_or_default(),
            args[4].parse::<f64>().unwrap_or_default(),
        )
    } else {
        (0.0, 0.0, 0.0)
    };
    let mut engine = star_trak::init(serde_json::from_str(&data).unwrap_or_default(), coords);
    let norad_ids = engine.get_norad_ids();
    write(&mut output, "L");
    write(&mut output, norad_ids.len());
    write(&mut output, "IDS");
    write_vec(&mut output, &norad_ids);
    write(&mut output, "\n");
    write(
        &mut output,
        "[POS_GEO:DEG_DEG_KM|REL_POS_AER:DEG_DEG_KM|REL_VEL_ENU:KM_KM_KM|TIME:S]",
    );
    write(&mut output, "\n");
    loop {
        star_trak::update(&mut engine);
        let geodetic_positions = star_trak::get_constellation_geodetic_positions(&engine);
        let (ranged_positions, ranged_velocities) = star_trak::get_observer_constellations(&engine);
        let timestamps = engine.get_timestamps();
        write(&mut output, "POS_GEO");
        write_vec(&mut output, geodetic_positions);
        write(&mut output, "REL_POS_AER");
        write_vec(&mut output, ranged_positions);
        write(&mut output, "REL_VEL_ENU");
        write_vec(&mut output, ranged_velocities);
        write(&mut output, "TIME");
        write_vec(&mut output, timestamps);
        write(&mut output, "\n");
        thread::sleep(Duration::from_millis(duration_ms));
    }
}

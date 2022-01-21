# Star Trak
## A WebAssembly-ready SGP4 realtime satellite tracker Rust crate

`star_trak` is Rust crate that tracks satellites of interest in realtime from General Perturbations (GP) orbital data.

This crate uses the [`sgp4`](https://crates.io/crates/sgp4) Rust crate to parse and extract the propagator constants for each satellite in the GP data. It then uses the crate along with the constants to propagate each satellite's orbital state from epoch to current date and time.

This crate auto-transforms the state data so that it is output with respect to both a Geodetic latitude-longitude-altitude reference frame, and a Topocentric azimuth-elevation-range reference frame relative to some observer.

This crate supports multiple compilation targets. It can be compiled into a native binary, or a WebAssembly module. It also optionally exposes a TypeScript API via [`wasm-bindgen`](https://crates.io/crates/wasm-bindgen) so that it can be consumed by TypeScript / JavaScript source code.

Head to [Star Trak PWA](https://github.com/mnahad/star-trak-pwa) for the official reference implementation.

# Prerequisites

## Required

- `cargo` ([`rustup`](https://www.rust-lang.org/tools/install) is recommended)

## Optional

- A WebAssembly runtime ([`wasmtime`](https://github.com/bytecodealliance/wasmtime) is recommended)
- [`wasm-pack`](https://github.com/rustwasm/wasm-pack) to create a TypeScript-compatible WebAssembly package

# Quickstart for native binary and standalone WebAssembly targets

## Build

```sh
# Select the correct target triple
# examples ...
rustup target add wasm32-unknown-unknown # for wasm
rustup target add x86_64-apple-darwin # for MacOS on x86_64

# Build
cargo build
```

## Run

```sh
# Get the GP data
curl -o gp.json "https://celestrak.com/NORAD/elements/gp.php?GROUP=starlink&FORMAT=json"

# Run the executable and pipe in the data via STDIN
cat gp.json | cargo run

# Optional args can be passed in
# i.e. cargo run [interval-ms observer-lat-deg observer-lng-deg observer-alt-km]
# examples ...
cat gp.json | cargo run 5000 # for a custom update interval of 5000 milliseconds
cat gp.json | cargo run 1000 33.920700 -118.327800 # for an update interval of 1000 milliseconds and a custom observer position

# For WebAssembly, execute the wasm runtime and pipe in the data
# examples ...
cat gp.json | wasmtime run ./path/to/star_trak.wasm
cat gp.json | wasmtime run ./path/to/star_trak.wasm 500 33.920700 -118.327800 0.0
```

# Quickstart for WebAssembly module consumed by TypeScript / JavaScript projects

## Build

```sh
wasm-pack build -- --features js-api
```

## Run

```typescript
// Define variables
const gpJson = await (
  await fetch("https://celestrak.com/NORAD/elements/gp.php?GROUP=starlink&FORMAT=json")
).json();
const observerCoords = [0.0, 0.0, 0.0];

import("./path/to/star-trak.js").then(({ Service }) => {
  // Create new Service object with GP data and observer coordinates
  const service = new Service(JSON.stringify(gpJson), ...observerCoords);
  setInterval(() => {
    // Update satellite states
    service.update();
    // Get states
    const geodeticPositions = service.get_constellation_geodetic_positions();
    const rangedPositions = service.get_ranged_positions();
    const rangedVelocities = service.get_ranged_velocities();
    // ...
  }, 1000);
  // Optional: Extract NORAD IDs from GP data
  const ids = service.get_norad_ids();
  // Optional: Update observer's coordinates
  const newObserverCoords = [0.0, 0.0, 0.0];
  service.update_observer(...newObserverCoords);
  // ...
});
```

# API Reference

## Crate library API

_Note that `engine` refers to the internal constellation engine module that powers this crate, and `sgp4` refers to the external `sgp4` Rust crate._

```rust
pub fn init(
    elements_group: Vec<sgp4::Elements>,
    (observer_lat, observer_lon, observer_alt): (f64, f64, f64),
) -> engine::Engine
```

Creates an instance of the constellation engine from a vector of GP Elements structs as per the `sgp4` crate. Also takes in an observer position.

```rust
pub fn update(engine: &mut engine::Engine) -> ()
```

Method to update the engine's satellite constellation. All states (including states relative to an observer) are propagated to their values at the device's current date and time.

```rust
pub fn update_observer_state(engine: &mut engine::Engine, (lat, lon, alt): (f64, f64, f64)) -> ()
```

Method to set engine's observer state with new position values.

```rust
pub fn get_constellation_geodetic_positions(engine: &engine::Engine) -> &Vec<engine::State>
```

Method to get all satellite positions in Earth-Centred Earth-Fixed (ECEF) geodetic format. Returns a reference to a vector of tuples (with each tuple in the format latitude degrees, longitude degrees, altitude km).

```rust
pub fn get_observer_constellations(
    engine: &engine::Engine,
) -> (&Vec<engine::State>, &Vec<engine::State>)
```

Method to get all constellation data relative to the observer. Returns a tuple of 2 references. The 1st reference is to a vector of relative positions in the topocentric Azimuth-Elevation-Range format (as a tuple of azimuth degrees, elevation degrees, range km). The 2nd reference is to a vector of relative velocities in topocentric East-North-Up format (as a tuple of east km, north km, up km).

```rust
impl engine::Engine {
    pub fn get_norad_ids(&self) -> Vec<u64>
    pub fn get_timestamps(&self) -> &Vec<i64>
}
```

The `engine::Engine` struct has some useful public methods that can be called. `get_norad_ids` returns a vector of the NORAD IDs, and `get_timestamps` returns a reference to a vector of the timestamps (as seconds relative to 1 January 1970).

## Executable API

### Parameters (all optional)

`[interval-ms observer-lat-deg observer-lng-deg observer-alt-km]`

- `interval-ms` The period between successive state computations (default 1000 ms)
- `observer-lat-deg` The latitude of an optional observer (default 0.0 degrees)
- `observer-lng-deg` The longitude of an optional observer (default 0.0 degrees)
- `observer-alt-km` The altitude of an optional observer (default 0.0 km)

### Input via STDIN

A serialised GP array in OMM (Orbit Mean-elements Message) [1] format encoded as JSON.

### Output via STDOUT

_Note: The following uses `<INTEGER>` to denote an integer value and `<DECIMAL>` to denote a decimal value. An ellipsis, such as in `<INTEGER...INTEGER>`, denotes a comma-separated ("`,`"-separated) list of values. Parentheses, such as in `<(DECIMAL,DECIMAL,DECIMAL)>`, denote a comma-separated tuple of values enclosed in the parentheses characters `(` and `)`._

The output message format consists of __2 header lines__ followed by __a recurring line of state data__. Each line is separated by a newline character (`\n`). The recurring line is output every interval.

__Header line 1__ is of the format:

`L<INTEGER>IDS[<INTEGER...INTEGER>]`

where the section beginning with the `L` character indicates the fixed length of all output arrays (also indicating the number of satellites in the constellation), and the section beginning with the characters `IDS` contains the array of all the NORAD IDs enclosed by square bracket characters.

__Header line 2__ prints the following string of characters:

`[POS_GEO:DEG_DEG_KM|REL_POS_AER:DEG_DEG_KM|REL_VEL_ENU:KM_KM_KM|TIME:S]`

and this header line indicates the format returned by the subsequent recurring lines.

- `POS_GEO:DEG_DEG_KM` - Array of position states in ECEF geodetic format (output as a tuple of latitude degrees, longitude degrees, altitude km).
- `REL_POS_AER:DEG_DEG_KM` - Array of relative positions with respect to an observer in topocentric Azimuth-Elevation-Range format (output as a tuple of azimuth degrees, elevation degrees, range km).
- `REL_VEL_ENU:KM_KM_KM` - Array of relative velocities with respect to an observer in topocentric East-North-Up format (output as a tuple of east km, north km, up km).
- `TIME:S` - Array of timestamps (output as seconds relative to 1 January 1970).

__Each recurring line__ is of the format:

`POS_GEO[<(DECIMAL,DECIMAL,DECIMAL)...(DECIMAL,DECIMAL,DECIMAL)>]REL_POS_AER[<(DECIMAL,DECIMAL,DECIMAL)...(DECIMAL,DECIMAL,DECIMAL)>]REL_VEL_ENU[<(DECIMAL,DECIMAL,DECIMAL)...(DECIMAL,DECIMAL,DECIMAL)>]TIME[<INTEGER...INTEGER>]`

and to help with parsing, each section starts with an indicator string matching the string in the header line (e.g. `POS_GEO`) and each array is enclosed by square bracket characters.

## TypeScript / JavaScript API

In order to use the TypeScript / JavaScript API, the `js-api` feature flag needs to be enabled.

```typescript
export class Service {
  /**
  * @param {string} data - stringified GP json array in OMM format
  * @param {number} observer_lat - Observer latitude in degrees
  * @param {number} observer_lon - Observer longitude in degrees
  * @param {number} observer_alt - Observer altitude in km
  */
  constructor(data: string, observer_lat: number, observer_lon: number, observer_alt: number);
  /**
  * @returns {BigUint64Array}
  * Gets an array of NORAD IDs.
  */
  get_norad_ids(): BigUint64Array;
  /**
  * Method to update the engine's satellite constellation.
  * All states (including states relative to an observer) are propagated to their values at
  * the device's current date and time.
  */
  update(): void;
  /**
  * @returns {Float64Array}
  * Method to get all satellite positions in Earth-Centred Earth-Fixed (ECEF) geodetic format.
  * Returns a array of 3 * N values, where N is the number of satellites in the constellation.
  * Each set of 3 values is in the format (latitude degrees, longitude degrees, altitude km).
  */
  get_constellation_geodetic_positions(): Float64Array;
  /**
  * @returns {Float64Array}
  * Method to get positions relative to the observer in the topocentric Azimuth-Elevation-Range format.
  * Returns a array of 3 * N values, where N is the number of satellites in the constellation.
  * Each set of 3 values is in the format (azimuth degrees, elevation degrees, range km).
  */
  get_ranged_positions(): Float64Array;
  /**
  * @returns {Float64Array}
  * Method to get velocities relative to the observer in the topocentric East-North-Up format.
  * Returns a array of 3 * N values, where N is the number of satellites in the constellation.
  * Each set of 3 values is in the format (east km, north km, up km).
  */
  get_ranged_velocities(): Float64Array;
  /**
  * @param {number} lat_deg - Observer latitude in degrees
  * @param {number} lon_deg - Observer longitude in degrees
  * @param {number} alt_km - Observer altitude in km
  * Updates observer position.
  */
  update_observer(lat_deg: number, lon_deg: number, alt_km: number): void;
}
```

# References

[1] Consultative Committee for Space Data Systems, "Recommendations for Space Data System Standards - Orbit Data Messages," CCSDS 502.0-B-2 [Online], 2009. Available: https://public.ccsds.org/Pubs/502x0b2c1e2.pdf

[2] D. A. Vallado, P. Crawford, R. Hujsak, T. S. Kelso, "Revisiting Spacetrack Report #3: Rev 2," American Institute of Aeronautics and Astronautics AIAA 2006-6753-Rev2 [Online], 2006. Available: https://celestrak.com/publications/AIAA/2006-6753/AIAA-2006-6753-Rev2.pdf

[3] T. S. Kelso, "Orbital Coordinate Systems, Part I," Satellite Times, 2, no. 1, pp 80-81 [Online], 1995. Available: https://celestrak.com/columns/v02n01/

[4] T. S. Kelso, "Orbital Coordinate Systems, Part II," Satellite Times, 2, no. 2, pp 78-79 [Online], 1995. Available: https://celestrak.com/columns/v02n02/

[5] T. S. Kelso, "Orbital Coordinate Systems, Part III," Satellite Times, 2, no. 3, pp 78-79 [Online], 1996. Available: https://celestrak.com/columns/v02n03/

[6] J. Zhu, "Conversion of Earth-centered Earth-fixed coordinates to geodetic coordinates," IEEE Transactions on Aerospace and Electronic Systems, vol 30, pp 957-961 [Online], 1994. Available: https://ieeexplore.ieee.org/document/303772

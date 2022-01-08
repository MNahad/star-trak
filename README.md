# Star Trak
## A WebAssembly-ready SGP4 realtime satellite tracker Rust crate

`star_trak` is Rust crate that tracks satellites of interest in realtime from General Pertubations (GP) orbital data.

This crate uses the [`sgp4`](https://crates.io/crates/sgp4) Rust crate to parse and extract the propagator constants for each satellite in the GP data. It then uses the crate along with the constants to propagate each satellite's orbital state from epoch to current date and time.

This crate auto-transforms the state data so that it is output with respect to both a Geodetic latitude-longitude-altitude reference frame, and a Topocentric azimuth-elevation-range reference frame relative to some observer.

This crate supports multiple compilation targets. It can be compiled into a native binary, or a WebAssembly module. It also optionally exposes a TypeScript API via [`wasm-bindgen`](https://crates.io/crates/wasm-bindgen) so that it can be consumed by TypeScript / JavaScript source code.

## Prerequisites

### Required

- `cargo` ([`rustup`](https://www.rust-lang.org/tools/install) is recommended)

### Optional

- A WebAssembly runtime ([`wasmtime`](https://github.com/bytecodealliance/wasmtime) is recommended)
- [`wasm-pack`](https://github.com/rustwasm/wasm-pack) to create a TypeScript-compatible WebAssembly package

## Quick start for native binary and standalone WebAssembly targets

### Build

```sh
# Select the correct target triple
# examples ...
rustup target add wasm32-unknown-unknown # for wasm
rustup target add x86_64-apple-darwin # for MacOS on x86_64

# Build
cargo build
```

### Run

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

## Quickstart for WebAssembly module consumed by TypeScript / JavaScript projects

### Build

```sh
wasm-pack build -- --features js-api
```

### Run

```typescript
// Define variables
// const gpJson = ...
// const observerCoords = [0.0, 0.0, 0.0];

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

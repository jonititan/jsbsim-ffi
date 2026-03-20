# jsbsim-ffi

Rust FFI bindings for [JSBSim](https://github.com/JSBSim-Team/jsbsim), an open-source flight dynamics model (FDM). Provides a safe Rust wrapper around `FGFDMExec` via a thin C shim.

## Prerequisites

1. **Build & install JSBSim** (requires C++17 compiler, cmake, pkg-config):
   ```bash
   git clone https://github.com/JSBSim-Team/jsbsim.git
   cd jsbsim && mkdir build && cd build
   cmake -DCMAKE_INSTALL_PREFIX=/usr/local -DBUILD_SHARED_LIBS=ON ..
   make -j$(nproc) && sudo make install
   sudo ldconfig /usr/local/lib
   ```

2. **Verify pkg-config** finds JSBSim:
   ```bash
   pkg-config --cflags --libs JSBSim
   ```
   If not found, add `export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH`.

3. **Update `build.rs`** — change the `jsbsim_src` path to your local JSBSim source `src/` directory.

## API

```rust
use jsbsim_ffi::Sim;

let mut sim = Sim::new("/path/to/jsbsim");   // root with aircraft/, engine/, systems/, scripts/
sim.load_model("c172x");                      // load aircraft by name
sim.load_script("scripts/c1723.xml");         // or load a scenario script
sim.set_property("ic/h-sl-ft", 5000.0);       // set initial conditions
sim.run_ic();                                  // initialise
sim.set_dt(0.01);                              // optional: set timestep
sim.run();                                     // advance one step
let alt = sim.get_property("position/h-sl-ft"); // read properties
```

`Sim` implements `Drop` — the C++ `FGFDMExec` is destroyed automatically.

## Examples

```bash
cargo run --example simple   # scripted Cessna 172 flight
cargo run --example fly      # interactive keyboard flight (W/S pitch, A/D roll, Q/E yaw, ↑/↓ throttle)
```

## Testing

Tests require `JSBSIM_ROOT` pointing to a JSBSim directory containing `aircraft/`, `engine/`, `systems/`, and `scripts/`. JSBSim is not thread-safe, so use `--test-threads=1`:

```bash
JSBSIM_ROOT=/path/to/jsbsim cargo test -- --test-threads=1
```

The integration tests in `tests/jsbsim_tests.rs` are ported from [JSBSim's Python test suite](https://github.com/JSBSim-Team/jsbsim/tree/master/tests) and cover model loading, script execution, initial conditions, atmosphere model validation, property round-trips, hold-down, timestep control, and physics sanity checks.

## Project Structure

```
jsbsim-ffi/
├── build.rs                 # compiles C++ wrapper, links JSBSim
├── c_wrapper/
│   ├── jsbsim_wrapper.h     # C FFI declarations
│   └── jsbsim_wrapper.cpp   # C++ FGFDMExec wrapper
├── src/lib.rs               # safe Rust API
├── examples/
│   ├── simple.rs            # scripted flight
│   └── fly.rs               # interactive flight
└── tests/
    ├── test_smoke.rs         # basic linkage/smoke tests
    └── jsbsim_tests.rs       # integration tests (from JSBSim test suite)
```

## Troubleshooting

| Problem | Fix |
|---|---|
| `libJSBSim.so.1: cannot open shared object file` | `sudo ldconfig /usr/local/lib` |
| `JSBSim not found via pkg-config!` | `export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH` |
| `Could not open file: Path "aircraft/..."` | Pass correct root dir to `Sim::new()` containing `aircraft/`, `engine/`, etc. |

## License

MIT

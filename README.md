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
sim.load_ic("reset01.xml", true);             // load IC file from aircraft dir
sim.set_property("ic/h-sl-ft", 5000.0);       // set initial conditions
sim.run_ic();                                  // initialise
sim.set_dt(0.01);                              // optional: set timestep
sim.run();                                     // advance one step
let alt = sim.get_property("position/h-sl-ft"); // read properties
```

`Sim` implements `Drop` — the C++ `FGFDMExec` is destroyed automatically.

### Full method list

| Category | Method | Description |
|---|---|---|
| **Lifecycle** | `Sim::new(root_dir)` | Create FDM instance |
| | `Sim::get_version()` | JSBSim version string |
| **Loading** | `load_model(name)` | Load aircraft (e.g. `"c172x"`) |
| | `load_script(path)` | Load scenario script |
| | `load_ic(file, use_aircraft_path)` | Load IC from XML file |
| **Simulation** | `run_ic()` | Initialize from ICs |
| | `run()` | Advance one timestep |
| | `set_dt(s)` / `get_dt()` | Set/get timestep |
| | `get_sim_time()` | Current sim time (seconds) |
| | `reset_to_initial_conditions(mode)` | Reset sim (0=state, 1=reload) |
| **Hold/Resume** | `hold()` / `resume()` / `holding()` | Pause/resume simulation |
| | `enable_increment_then_hold(n)` | Run N steps then auto-hold |
| **Integration** | `suspend_integration()` / `resume_integration()` | Freeze/unfreeze physics |
| **Trim** | `do_trim(mode)` | Trim aircraft (`trim::LONGITUDINAL`, `FULL`, `GROUND`, etc.) |
| **Properties** | `get_property(path)` / `set_property(path, val)` | Read/write properties |
| | `has_property(path)` | Check if property exists |
| | `query_property_catalog(filter)` | Search property tree |
| | `print_property_catalog()` | Dump all properties to stdout |
| **Output** | `set_output_directive(file)` | Add output XML directive |
| | `enable_output()` / `disable_output()` | Toggle data output |
| | `set_output_filename(n, file)` | Set output channel filename |
| **Paths** | `set_aircraft_path(p)` / `set_engine_path(p)` / `set_systems_path(p)` | Configure search paths |
| **Debug** | `set_debug_level(n)` | 0=silent, higher=verbose |
| | `get_model_name()` | Name of loaded aircraft |

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

# jsbsim-ffi

Rust FFI bindings for [JSBSim](https://github.com/JSBSim-Team/jsbsim), an open-source flight dynamics model (FDM) library written in C++.

This crate provides a safe Rust wrapper around JSBSim's `FGFDMExec` class via a thin C shim, letting you load aircraft models, run simulation scripts, and read/write flight properties from Rust.

## Prerequisites

### 1. Install JSBSim from source

```bash
git clone https://github.com/JSBSim-Team/jsbsim.git
cd jsbsim
mkdir build && cd build
cmake -DCMAKE_INSTALL_PREFIX=/usr/local -DBUILD_SHARED_LIBS=ON ..
make -j$(nproc)
sudo make install
```

### 2. Register the shared library

After installation, make sure the linker can find `libJSBSim.so`:

```bash
sudo ldconfig /usr/local/lib
```

Or, if you prefer not to use `sudo`, set the environment variable before running:

```bash
export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH
```

### 3. Verify pkg-config can find JSBSim

The build script uses `pkg-config` to locate JSBSim headers and libraries:

```bash
pkg-config --cflags --libs JSBSim
```

This should output something like `-I/usr/local/include/JSBSim -L/usr/local/lib -lJSBSim`. If it doesn't, ensure `/usr/local/lib/pkgconfig` is in your `PKG_CONFIG_PATH`:

```bash
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH
```

### 4. Build tools

You need a C++ compiler (g++ or clang++) with C++17 support, plus the standard Rust toolchain:

- **Rust** ≥ 1.56 (edition 2021)
- **C++ compiler** with C++17 support
- **pkg-config**
- **cmake** (for building JSBSim itself)

On Debian/Ubuntu:

```bash
sudo apt install build-essential pkg-config cmake
```

## Configuration

The `build.rs` script has one hardcoded path that you may need to change:

```rust
let jsbsim_src = "/home/joni/jsbsim/src";
```

Change this to point to your local JSBSim **source tree** `src/` directory. This is needed because the installed headers at `/usr/local/include/JSBSim/` are sufficient for most use, but the C++ wrapper compiles against the full source headers for maximum compatibility.

> **Tip:** If you installed JSBSim and the installed headers are sufficient (they usually are), you can remove the `jsbsim_src` include line and rely solely on pkg-config's include paths.

## JSBSim data files

JSBSim needs access to aircraft models, engine definitions, and simulation scripts at runtime. These are found in the JSBSim source repository:

```
jsbsim/
├── aircraft/     ← aircraft model definitions
├── engine/       ← engine definitions
├── systems/      ← systems definitions (autopilot, etc.)
└── scripts/      ← simulation scenario scripts
```

When creating a `Sim` instance, pass the path to your JSBSim root directory (the one containing `aircraft/`, `engine/`, `systems/`, and `scripts/`):

```rust
let mut sim = Sim::new("/path/to/jsbsim");
```

## Usage

Add this crate as a dependency (local path or git):

```toml
[dependencies]
jsbsim-ffi = { path = "../jsbsim-ffi" }
```

### API

| Method | Description |
|---|---|
| `Sim::new(root_dir)` | Create a new JSBSim FDM instance with the given root directory |
| `sim.load_model(name)` | Load an aircraft model by name (e.g., `"c172x"`) |
| `sim.load_script(path)` | Load a JSBSim script XML file (relative to root dir) |
| `sim.run_ic()` | Initialize and run initial conditions |
| `sim.run()` | Advance the simulation by one time step |
| `sim.set_dt(seconds)` | Set the simulation time step in seconds |
| `sim.get_property(name)` | Read a property value (e.g., `"position/h-agl-ft"`) |
| `sim.set_property(name, value)` | Set a property value (e.g., `"fcs/throttle-cmd-norm"`) |

The `Sim` struct implements `Drop`, so the underlying C++ `FGFDMExec` object is automatically destroyed when the Rust struct goes out of scope.

### Example

```rust
use jsbsim_ffi::Sim;

fn main() {
    let mut sim = Sim::new("/path/to/jsbsim");

    if !sim.load_script("scripts/c1723.xml") {
        eprintln!("Failed to load script!");
        return;
    }

    if !sim.run_ic() {
        eprintln!("Failed to run initial conditions!");
        return;
    }

    println!("JSBSim running (Cessna 172)...");

    for step in 0..2000 {
        sim.run();

        if step % 400 == 0 {
            let t   = sim.get_property("simulation/sim-time-sec");
            let alt = sim.get_property("position/h-agl-ft");
            let ias = sim.get_property("velocities/vc-kts");

            println!("t={:.1}s | Altitude: {:.0} ft | IAS: {:.1} kts", t, alt, ias);
        }
    }
}
```

Run the included examples:

```bash
# Simple scripted simulation (non-interactive)
LD_LIBRARY_PATH=/usr/local/lib cargo run --example simple

# Interactive flight — fly with your keyboard!
LD_LIBRARY_PATH=/usr/local/lib cargo run --example fly
```

### Interactive flight example (`fly`)

The `fly` example lets you fly a Cessna 172 in real-time using keyboard controls. The aircraft starts airborne at 3000 ft, 100 kts, heading north.

**Controls:**

| Key | Action |
|---|---|
| `W` / `S` | Pitch down / up (elevator) |
| `A` / `D` | Roll left / right (aileron) |
| `Q` / `E` | Yaw left / right (rudder) |
| `↑` / `↓` | Increase / decrease throttle |
| `Space` | Center all flight controls |
| `B` | Toggle parking brake |
| `Esc` | Quit |

A live HUD displays airspeed, altitude, heading, pitch/roll angles, throttle position, and other instruments.

### Common JSBSim properties

Here are some frequently used property paths:

| Property | Description |
|---|---|
| `simulation/sim-time-sec` | Simulation time in seconds |
| `position/h-agl-ft` | Altitude above ground level (feet) |
| `position/h-sl-ft` | Altitude above sea level (feet) |
| `position/lat-geod-deg` | Geodetic latitude (degrees) |
| `position/long-gc-deg` | Geocentric longitude (degrees) |
| `velocities/vc-kts` | Calibrated airspeed (knots) |
| `velocities/vtrue-kts` | True airspeed (knots) |
| `velocities/mach` | Mach number |
| `velocities/v-down-fps` | Vertical speed (ft/s, positive=down) |
| `attitude/phi-rad` | Roll angle (radians) |
| `attitude/theta-rad` | Pitch angle (radians) |
| `attitude/psi-rad` | Heading (radians) |
| `fcs/throttle-cmd-norm` | Throttle command (0.0–1.0) |
| `fcs/aileron-cmd-norm` | Aileron command (-1.0–1.0) |
| `fcs/elevator-cmd-norm` | Elevator command (-1.0–1.0) |
| `fcs/rudder-cmd-norm` | Rudder command (-1.0–1.0) |
| `propulsion/engine/thrust-lbs` | Engine thrust (pounds) |

For a complete list of properties, see the [JSBSim reference manual](https://jsbsim-team.github.io/jsbsim-reference-manual/).

## Project structure

```
jsbsim-ffi/
├── build.rs                      # Build script: compiles C++ wrapper, links JSBSim
├── Cargo.toml
├── c_wrapper/
│   ├── jsbsim_wrapper.h          # C header declaring the FFI interface
│   └── jsbsim_wrapper.cpp        # C++ implementation wrapping FGFDMExec
├── examples/
│   ├── fly.rs                    # Example: interactive keyboard flight
│   └── simple.rs                 # Example: run a Cessna 172 script
├── src/
│   └── lib.rs                    # Safe Rust API wrapping the C FFI
└── README.md
```

## Troubleshooting

**`libJSBSim.so.1: cannot open shared object file`**

The dynamic linker can't find the JSBSim shared library. Fix with:
```bash
sudo ldconfig /usr/local/lib
# or
export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH
```

**`JSBSim not found via pkg-config!`**

Make sure JSBSim is installed and pkg-config can find it:
```bash
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH
pkg-config --exists JSBSim && echo "Found" || echo "Not found"
```

**`Could not open file: Path "aircraft/..."`**

The root directory passed to `Sim::new()` must contain the `aircraft/`, `engine/`, `systems/`, and `scripts/` directories. Point it to your JSBSim source checkout.

## License

MIT

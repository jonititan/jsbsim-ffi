//! Batch-mode wind field example — Cessna 172 in time-varying wind.
//!
//! Demonstrates how to update JSBSim's wind field each timestep from Rust,
//! as discussed in:
//!   - <https://github.com/JSBSim-Team/jsbsim/discussions/1130>
//!   - <https://github.com/JSBSim-Team/jsbsim/issues/74>
//!
//! The simulation runs a Cessna 172 in level flight while a wind field that
//! evolves over time is injected every step via `set_property`.  The wind
//! model includes:
//!
//!   1. **Altitude-dependent base wind** — logarithmic wind profile (stronger
//!      at altitude, calm near the surface).
//!   2. **Slowly rotating direction** — the wind direction veers over time.
//!   3. **Pseudo-turbulence** — multi-frequency sine perturbations.
//!
//! Run with:
//!
//! ```sh
//! JSBSIM_ROOT=/path/to/jsbsim cargo run --example wind_batch
//! ```

use jsbsim_ffi::Sim;

// ── Wind model ──────────────────────────────────────────────────────────

/// Compute wind components (North, East, Down) in ft/s given sim state.
///
/// This is a simple procedural model you can replace with a lookup table,
/// a weather API, CFD output, etc.
fn compute_wind(time_s: f64, altitude_agl_ft: f64) -> (f64, f64, f64) {
    // -- 1. Logarithmic wind profile --
    // Reference wind at 1000 ft AGL.  Surface roughness length ~ 0.1 ft.
    let ref_alt = 1000.0_f64;
    let roughness = 0.1_f64;
    let ref_speed_fps = 30.0; // ~18 kts at 1000 ft

    let alt = altitude_agl_ft.max(1.0); // avoid log(0)
    let profile = (alt / roughness).ln() / (ref_alt / roughness).ln();
    let base_speed = ref_speed_fps * profile.max(0.0);

    // -- 2. Slowly veering direction (full rotation in 600 s) --
    let veer_rate = std::f64::consts::TAU / 600.0; // rad/s
    let wind_from_rad = veer_rate * time_s;

    // Decompose into NED.  "Wind from" means the wind blows *towards* the
    // opposite direction, so north-component = -speed * cos(from).
    let wind_north = -base_speed * wind_from_rad.cos();
    let wind_east = -base_speed * wind_from_rad.sin();

    // -- 3. Pseudo-turbulence (sum of sines at different frequencies) --
    let turb_amplitude = 5.0; // ft/s peak
    let turb_n = turb_amplitude
        * (0.7 * (0.3 * time_s).sin()
            + 0.5 * (0.73 * time_s + 1.0).sin()
            + 0.3 * (1.87 * time_s + 2.5).sin());
    let turb_e = turb_amplitude
        * (0.6 * (0.37 * time_s + 0.5).sin()
            + 0.4 * (0.91 * time_s + 1.7).sin()
            + 0.3 * (2.13 * time_s + 3.1).sin());
    let turb_d =
        turb_amplitude * 0.4 * (0.5 * (0.41 * time_s).sin() + 0.5 * (1.17 * time_s + 0.8).sin());

    (wind_north + turb_n, wind_east + turb_e, turb_d)
}

// ── Main ────────────────────────────────────────────────────────────────

fn main() {
    let root = std::env::var("JSBSIM_ROOT").unwrap_or_else(|_| {
        eprintln!(
            "JSBSIM_ROOT is not set.  Point it at a directory containing\n\
             aircraft/, engine/, systems/ subdirectories.\n\
             \n\
             Example:\n\
             \n\
                 JSBSIM_ROOT=/path/to/jsbsim cargo run --example wind_batch\n"
        );
        std::process::exit(1);
    });

    // ── Create sim & load aircraft ──────────────────────────────────────
    let mut sim = Sim::new(&root);
    sim.set_debug_level(0); // suppress JSBSim chatter

    if !sim.load_model("c172x") {
        eprintln!("Failed to load aircraft model 'c172x'!");
        return;
    }

    // ── Initial conditions: level flight at 3 000 ft, heading north ─────
    sim.set_property("ic/h-sl-ft", 3000.0);
    sim.set_property("ic/vc-kts", 100.0);
    sim.set_property("ic/psi-true-deg", 0.0);
    sim.set_property("ic/lat-geod-deg", 37.62);
    sim.set_property("ic/long-gc-deg", -122.38);
    sim.set_property("ic/gamma-deg", 0.0);

    if !sim.run_ic() {
        eprintln!("Failed to initialise conditions!");
        return;
    }

    // Engine running, mixture rich, gear up
    sim.set_property("propulsion/engine/set-running", 1.0);
    sim.set_property("fcs/mixture-cmd-norm", 1.0);
    sim.set_property("gear/gear-cmd-norm", 0.0);
    sim.set_property("fcs/throttle-cmd-norm", 0.65);

    // ── Print header ────────────────────────────────────────────────────
    println!("Wind-Field Batch Simulation — Cessna 172");
    println!("=========================================");
    println!(
        "{:>7}  {:>7}  {:>7}  {:>8}  {:>8}  {:>8}  {:>7}  {:>7}  {:>7}  {:>6}",
        "Time(s)",
        "Alt(ft)",
        "IAS(kt)",
        "Hdg(°)",
        "GS(kt)",
        "TAS(kt)",
        "Wn(fps)",
        "We(fps)",
        "Wd(fps)",
        "Wmag"
    );
    println!("{}", "-".repeat(95));

    // ── Simulation loop ─────────────────────────────────────────────────
    let total_steps = 120 * 60; // 60 seconds at 120 Hz
    let print_interval = 120 * 5; // print every 5 seconds

    for step in 0..total_steps {
        let time = sim.get_sim_time();
        let alt_agl = sim.get_property("position/h-agl-ft");

        // ── Compute and inject wind for this timestep ───────────────────
        let (wn, we, wd) = compute_wind(time, alt_agl);
        sim.set_property("atmosphere/wind-north-fps", wn);
        sim.set_property("atmosphere/wind-east-fps", we);
        sim.set_property("atmosphere/wind-down-fps", wd);

        // ── Advance one step ────────────────────────────────────────────
        sim.run();

        // ── Report ──────────────────────────────────────────────────────
        if step % print_interval == 0 {
            let alt_msl = sim.get_property("position/h-sl-ft");
            let ias = sim.get_property("velocities/vc-kts");
            let hdg = sim.get_property("attitude/psi-deg");
            let gs_n = sim.get_property("velocities/v-north-fps");
            let gs_e = sim.get_property("velocities/v-east-fps");
            let gs = (gs_n * gs_n + gs_e * gs_e).sqrt() * 0.592_484; // fps→kts
            let tas = sim.get_property("velocities/vt-fps") * 0.592_484;
            let wmag = (wn * wn + we * we + wd * wd).sqrt();

            // Read the total wind (steady + gust + turb) as seen by JSBSim
            let _tw_n = sim.get_property("atmosphere/total-wind-north-fps");
            let _tw_e = sim.get_property("atmosphere/total-wind-east-fps");

            println!(
                "{:>7.1}  {:>7.0}  {:>7.1}  {:>8.1}  {:>8.1}  {:>8.1}  {:>7.1}  {:>7.1}  {:>7.1}  {:>6.1}",
                time, alt_msl, ias, hdg, gs, tas, wn, we, wd, wmag
            );
        }
    }

    println!("{}", "-".repeat(95));
    println!(
        "Simulation complete. Wind was updated every timestep ({} steps).",
        total_steps
    );
}

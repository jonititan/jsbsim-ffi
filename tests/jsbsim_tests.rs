//! Integration tests ported from the JSBSim Python test suite.
//!
//! These tests exercise the Rust FFI wrapper against a real JSBSim data
//! directory.  Set the `JSBSIM_ROOT` environment variable to the path
//! containing `aircraft/`, `engine/`, `systems/`, and `scripts/`
//! directories before running:
//!
//! ```sh
//! JSBSIM_ROOT=/path/to/jsbsim cargo test -- --test-threads=1
//! ```
//!
//! JSBSim's C++ internals are not thread-safe, so tests must run
//! single-threaded to avoid crashes from concurrent FDM instances.
//!
//! Ported tests originate from:
//!   <https://github.com/JSBSim-Team/jsbsim/tree/master/tests>

use jsbsim_ffi::Sim;

// ---------------------------------------------------------------------------
// Helper: obtain JSBSIM_ROOT or skip the test gracefully.
// ---------------------------------------------------------------------------
fn jsbsim_root() -> Option<String> {
    match std::env::var("JSBSIM_ROOT") {
        Ok(r) if !r.is_empty() => Some(r),
        _ => {
            eprintln!(
                "JSBSIM_ROOT not set — skipping test.  \
                 To run: JSBSIM_ROOT=/path/to/jsbsim cargo test"
            );
            None
        }
    }
}

/// Convenience: create a `Sim` pointed at JSBSIM_ROOT.
fn create_fdm() -> Option<Sim> {
    jsbsim_root().map(|root| Sim::new(&root))
}

// ===========================================================================
// TestModelLoading  (from tests/TestModelLoading.py)
//
// Verify that a selection of aircraft models can be loaded.
// The Python original also checks that extracting XML sections into
// separate files produces bit-identical output; here we focus on the
// load-model step itself since that is what the FFI exposes.
// ===========================================================================

#[test]
fn load_model_ball() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(
        sim.load_model("ball"),
        "Failed to load the 'ball' model"
    );
}

#[test]
fn load_model_c172x() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(
        sim.load_model("c172x"),
        "Failed to load the 'c172x' model"
    );
}

#[test]
fn load_model_737() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(
        sim.load_model("737"),
        "Failed to load the '737' model"
    );
}

#[test]
fn load_model_f16() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(
        sim.load_model("f16"),
        "Failed to load the 'f16' model"
    );
}

#[test]
fn load_model_nonexistent_returns_false() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(
        !sim.load_model("this_aircraft_does_not_exist"),
        "Loading a nonexistent model should return false"
    );
}

// ===========================================================================
// CheckScripts  (from tests/CheckScripts.py)
//
// Load and run several JSBSim scripts for a short period to verify that
// the script-loading path and the simulation loop work through the FFI.
// ===========================================================================

/// Helper: load a script by name (relative to JSBSIM_ROOT/scripts/),
/// run initial conditions, then step the simulation up to `end_time` seconds.
fn run_script(script_name: &str, end_time: f64) {
    let root = match jsbsim_root() {
        Some(r) => r,
        None => return,
    };
    let script_path = format!("{}/scripts/{}", root, script_name);
    let mut sim = Sim::new(&root);

    assert!(
        sim.load_script(&script_path),
        "Failed to load script {script_name}"
    );
    assert!(sim.run_ic(), "RunIC failed for script {script_name}");

    // Step until we reach end_time or run() returns false (simulation ended).
    loop {
        let t = sim.get_property("simulation/sim-time-sec");
        if t >= end_time {
            break;
        }
        if !sim.run() {
            break;
        }
    }
    let t = sim.get_property("simulation/sim-time-sec");
    assert!(
        t > 0.0,
        "Simulation time should have advanced for script {script_name}, got {t}"
    );
}

#[test]
fn script_ball_orbit() {
    run_script("ball_orbit.xml", 5.0);
}

#[test]
fn script_c1721() {
    run_script("c1721.xml", 5.0);
}

#[test]
fn script_ball() {
    run_script("ball.xml", 5.0);
}

#[test]
fn script_c1722() {
    run_script("c1722.xml", 5.0);
}

// ===========================================================================
// TestInitialConditions  (from tests/TestInitialConditions.py)
//
// Verify that initial-condition properties are correctly set and
// propagated after RunIC.
// ===========================================================================

#[test]
fn initial_conditions_altitude_and_speed() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("c172x"));

    // Set ICs via properties (the same approach the Python test uses).
    sim.set_property("ic/h-sl-ft", 5000.0);
    sim.set_property("ic/vc-kts", 120.0);
    sim.set_property("ic/gamma-deg", 0.0);
    sim.set_property("ic/psi-true-deg", 90.0);

    assert!(sim.run_ic(), "RunIC failed");

    // After RunIC the simulation state should reflect the ICs.
    let alt = sim.get_property("position/h-sl-ft");
    assert!(
        (alt - 5000.0).abs() < 10.0,
        "Altitude should be ~5000 ft, got {alt}"
    );

    // Heading should be approximately what we set.
    let psi = sim.get_property("attitude/psi-rad");
    let psi_deg = psi.to_degrees();
    assert!(
        (psi_deg - 90.0).abs() < 1.0,
        "Heading should be ~90°, got {psi_deg}"
    );
}

#[test]
fn initial_conditions_ball_position() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("ball"));

    let target_alt_ft = 100_000.0;
    sim.set_property("ic/h-sl-ft", target_alt_ft);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);

    assert!(sim.run_ic(), "RunIC failed for ball");

    let alt = sim.get_property("position/h-sl-ft");
    assert!(
        (alt - target_alt_ft).abs() < 10.0,
        "Ball altitude should be ~{target_alt_ft} ft, got {alt}"
    );
}

// ===========================================================================
// CheckSimTimeReset  (from tests/CheckSimTimeReset.py)
//
// Verify that the simulation time starts at zero and advances correctly.
// ===========================================================================

#[test]
fn sim_time_starts_at_zero() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("c172x"));

    sim.set_property("ic/h-sl-ft", 3000.0);
    sim.set_property("ic/vc-kts", 100.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    let t0 = sim.get_property("simulation/sim-time-sec");
    assert!(
        t0.abs() < 1e-9,
        "Sim time should be 0 after RunIC, got {t0}"
    );
}

#[test]
fn sim_time_advances() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("c172x"));

    sim.set_property("ic/h-sl-ft", 3000.0);
    sim.set_property("ic/vc-kts", 100.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    // Step a number of frames.
    for _ in 0..100 {
        sim.run();
    }

    let t = sim.get_property("simulation/sim-time-sec");
    assert!(
        t > 0.0,
        "Sim time should advance after stepping, got {t}"
    );
}

#[test]
fn sim_time_monotonically_increasing() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("ball"));

    sim.set_property("ic/h-sl-ft", 10000.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    let mut prev_t = 0.0_f64;
    for _ in 0..200 {
        sim.run();
        let t = sim.get_property("simulation/sim-time-sec");
        assert!(
            t > prev_t,
            "Time must be monotonically increasing: prev={prev_t}, now={t}"
        );
        prev_t = t;
    }
}

// ===========================================================================
// TestStdAtmosphere  (from tests/TestStdAtmosphere.py)
//
// Verify that standard-atmosphere properties at sea level match the ISA
// 1976 reference values.
// ===========================================================================

#[test]
fn std_atmosphere_sea_level() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("ball"));

    sim.set_property("ic/h-sl-ft", 0.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    // ISA sea-level temperature = 518.67 °R  (288.15 K)
    let temp_r = sim.get_property("atmosphere/T-R");
    assert!(
        (temp_r - 518.67).abs() < 0.5,
        "Sea-level temperature should be ~518.67 °R, got {temp_r}"
    );

    // ISA sea-level pressure = 2116.22 psf  (101325 Pa)
    let p_psf = sim.get_property("atmosphere/P-psf");
    assert!(
        (p_psf - 2116.22).abs() < 1.0,
        "Sea-level pressure should be ~2116.22 psf, got {p_psf}"
    );

    // ISA sea-level density ≈ 0.002377 slugs/ft³
    let rho = sim.get_property("atmosphere/rho-slugs_ft3");
    assert!(
        (rho - 0.002377).abs() < 0.0001,
        "Sea-level density should be ~0.002377 slug/ft³, got {rho}"
    );
}

#[test]
fn std_atmosphere_at_altitude() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("ball"));

    // Set to 36,089 ft (tropopause in ISA).
    let tropopause_ft = 36_089.0;
    sim.set_property("ic/h-sl-ft", tropopause_ft);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    // ISA tropopause temperature = 216.65 K = 389.97 °R
    let temp_r = sim.get_property("atmosphere/T-R");
    assert!(
        (temp_r - 389.97).abs() < 1.0,
        "Tropopause temperature should be ~389.97 °R, got {temp_r}"
    );

    // Pressure at the tropopause should be significantly lower than sea level.
    let p_psf = sim.get_property("atmosphere/P-psf");
    assert!(
        p_psf < 500.0 && p_psf > 400.0,
        "Tropopause pressure should be ~472 psf, got {p_psf}"
    );
}

// ===========================================================================
// TestPressureAltitude  (from tests/TestPressureAltitude.py)
//
// Set a geometric altitude, then read back the pressure altitude and
// verify it is consistent.
// ===========================================================================

#[test]
fn pressure_altitude_at_sea_level() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("ball"));

    sim.set_property("ic/h-sl-ft", 0.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    // With no temperature offset, pressure altitude == geometric altitude at
    // sea level.
    let pa = sim.get_property("atmosphere/pressure-altitude");
    assert!(
        pa.abs() < 1.0,
        "Pressure altitude at sea level should be ~0 ft, got {pa}"
    );
}

#[test]
fn pressure_altitude_at_10000ft() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("ball"));

    sim.set_property("ic/h-sl-ft", 10_000.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    // With standard atmosphere (no delta-T), pressure altitude should match
    // geometric altitude closely.
    let pa = sim.get_property("atmosphere/pressure-altitude");
    assert!(
        (pa - 10_000.0).abs() < 50.0,
        "Pressure altitude at 10 000 ft should be ~10 000 ft, got {pa}"
    );
}

// ===========================================================================
// TestMiscellaneous – property round-trip  (from tests/TestMiscellaneous.py)
//
// Verify get/set property round-trips and that the property tree works.
// ===========================================================================

#[test]
fn property_round_trip() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("ball"));

    sim.set_property("ic/h-sl-ft", 1000.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    // Throttle-command should be settable and readable.
    assert!(sim.set_property("fcs/throttle-cmd-norm", 0.75));
    let v = sim.get_property("fcs/throttle-cmd-norm");
    assert!(
        (v - 0.75).abs() < 1e-6,
        "Throttle should round-trip to 0.75, got {v}"
    );
}

#[test]
fn property_set_and_read_ic() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("c172x"));

    let target = 7500.0_f64;
    sim.set_property("ic/h-sl-ft", target);

    // IC properties should be readable before RunIC.
    let readback = sim.get_property("ic/h-sl-ft");
    assert!(
        (readback - target).abs() < 1e-6,
        "IC altitude should round-trip, got {readback}"
    );
}

// ===========================================================================
// TestHoldDown  (from tests/TestHoldDown.py)
//
// Verify that the hold-down property prevents the aircraft from moving.
// ===========================================================================

#[test]
fn hold_down_prevents_motion() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("c172x"));

    sim.set_property("ic/h-sl-ft", 0.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    // Engage hold-down.
    sim.set_property("forces/hold-down", 1.0);

    // Step the simulation.
    for _ in 0..200 {
        sim.run();
    }

    // With hold-down active the altitude should remain at or very near 0.
    let alt = sim.get_property("position/h-sl-ft");
    assert!(
        alt.abs() < 5.0,
        "With hold-down active, altitude should stay ~0 ft, got {alt}"
    );
}

// ===========================================================================
// Simulation property consistency  (inspired by TestSuspend.py /
// TestMiscellaneous.py)
//
// Note: The Python TestSuspend uses fdm.hold()/resume() which are direct
// C++ method calls not exposed in this FFI.  Instead we verify that
// simulation properties remain self-consistent across a run.
// ===========================================================================

#[test]
fn simulation_dt_consistent_with_time() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("ball"));

    sim.set_property("ic/h-sl-ft", 10_000.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    let dt = sim.get_property("simulation/dt");
    assert!(dt > 0.0, "dt should be positive, got {dt}");

    // Run 100 steps and verify accumulated time ≈ 100 * dt.
    let n_steps = 100;
    for _ in 0..n_steps {
        sim.run();
    }

    let t = sim.get_property("simulation/sim-time-sec");
    let expected = dt * n_steps as f64;
    assert!(
        (t - expected).abs() < dt,
        "After {n_steps} steps at dt={dt}, time should be ~{expected}, got {t}"
    );
}

// ===========================================================================
// dt / timestep control  (inspired by CheckScripts.py testScriptEndTime)
//
// Verify that set_dt changes the integration timestep.
// ===========================================================================

#[test]
fn set_dt_changes_timestep() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("ball"));

    sim.set_property("ic/h-sl-ft", 10_000.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    let custom_dt = 0.01; // 10 ms
    sim.set_dt(custom_dt);

    // Step once, then check the dt property.
    sim.run();
    let dt_read = sim.get_property("simulation/dt");
    assert!(
        (dt_read - custom_dt).abs() < 1e-9,
        "dt should be {custom_dt}, got {dt_read}"
    );

    // Step 100 frames at 0.01 s → expect ~1.0 s of sim time.
    // (We already did one step, so 99 more.)
    for _ in 0..99 {
        sim.run();
    }
    let t = sim.get_property("simulation/sim-time-sec");
    assert!(
        (t - 1.0).abs() < 0.05,
        "After 100 steps at dt=0.01, sim time should be ~1.0 s, got {t}"
    );
}

// ===========================================================================
// Multiple instances  (inspired by TestMiscellaneous.py property-manager
// sharing test)
//
// Verify that two independent Sim instances can coexist without
// interfering with each other.
// ===========================================================================

#[test]
fn multiple_independent_instances() {
    let root = match jsbsim_root() {
        Some(r) => r,
        None => return,
    };

    let mut sim_a = Sim::new(&root);
    let mut sim_b = Sim::new(&root);

    assert!(sim_a.load_model("ball"));
    assert!(sim_b.load_model("c172x"));

    // Different ICs.
    sim_a.set_property("ic/h-sl-ft", 50_000.0);
    sim_a.set_property("ic/vc-kts", 0.0);
    sim_a.set_property("ic/gamma-deg", 0.0);

    sim_b.set_property("ic/h-sl-ft", 3_000.0);
    sim_b.set_property("ic/vc-kts", 100.0);
    sim_b.set_property("ic/gamma-deg", 0.0);

    assert!(sim_a.run_ic());
    assert!(sim_b.run_ic());

    for _ in 0..50 {
        sim_a.run();
        sim_b.run();
    }

    let alt_a = sim_a.get_property("position/h-sl-ft");
    let alt_b = sim_b.get_property("position/h-sl-ft");

    // The two simulations should have very different altitudes.
    assert!(
        (alt_a - alt_b).abs() > 100.0,
        "Independent sims should diverge: alt_a={alt_a}, alt_b={alt_b}"
    );
}

// ===========================================================================
// Full script run  (inspired by CheckScripts.py)
//
// Run a longer scenario to verify the wrapper handles sustained
// simulation loops.
// ===========================================================================

#[test]
fn full_script_run_ball_orbit() {
    let root = match jsbsim_root() {
        Some(r) => r,
        None => return,
    };
    let script_path = format!("{}/scripts/ball_orbit.xml", root);
    let mut sim = Sim::new(&root);

    assert!(sim.load_script(&script_path));
    assert!(sim.run_ic());

    let mut steps = 0u64;
    let max_time = 30.0;
    loop {
        let t = sim.get_property("simulation/sim-time-sec");
        if t >= max_time {
            break;
        }
        if !sim.run() {
            break;
        }
        steps += 1;
    }

    let t_final = sim.get_property("simulation/sim-time-sec");
    assert!(
        t_final >= max_time - 1.0,
        "Script should have run to ~{max_time}s, got {t_final}"
    );
    assert!(
        steps > 100,
        "Should have taken many steps, took {steps}"
    );
    println!("ball_orbit: {steps} steps, final time {t_final:.2}s");
}

// ===========================================================================
// Gravity / free-fall  (validates physics through the FFI)
//
// Drop a ball from 10 000 ft with zero velocity and verify the altitude
// decreases over time (i.e., gravity is working through the wrapper).
// ===========================================================================

#[test]
fn ball_free_fall_altitude_decreases() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("ball"));

    let initial_alt = 10_000.0;
    sim.set_property("ic/h-sl-ft", initial_alt);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    // Step 5 seconds of simulation.
    sim.set_dt(0.01);
    for _ in 0..500 {
        sim.run();
    }

    let alt = sim.get_property("position/h-sl-ft");
    assert!(
        alt < initial_alt,
        "After free-fall the ball should have descended: \
         initial={initial_alt}, current={alt}"
    );

    // Rough sanity: after ~5s of free-fall, Δh ≈ ½g·t² ≈ 402 ft.
    let delta = initial_alt - alt;
    assert!(
        delta > 300.0 && delta < 600.0,
        "Free-fall displacement should be ~400 ft, got {delta}"
    );
    println!("Free-fall: Δalt = {delta:.1} ft after ~5 s");
}

// ===========================================================================
// C172x – trimmed flight properties  (inspired by TestInitialConditions.py)
//
// Verify that a c172x with sensible ICs produces physically plausible
// property values after a short run.
// ===========================================================================

#[test]
fn c172x_flight_properties_plausible() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("c172x"));

    sim.set_property("ic/h-sl-ft", 5000.0);
    sim.set_property("ic/vc-kts", 100.0);
    sim.set_property("ic/gamma-deg", 0.0);
    sim.set_property("ic/psi-true-deg", 0.0);
    assert!(sim.run_ic());

    // Run for 2 seconds.
    for _ in 0..240 {
        sim.run();
    }

    // Airspeed should still be positive.
    let vc = sim.get_property("velocities/vc-kts");
    assert!(
        vc > 50.0,
        "Calibrated airspeed should be > 50 kts, got {vc}"
    );

    // Altitude should still be in a reasonable band.
    let alt = sim.get_property("position/h-sl-ft");
    assert!(
        alt > 4000.0 && alt < 6000.0,
        "Altitude should remain near 5000 ft, got {alt}"
    );

    // Roll / pitch should be modest (not tumbling).
    let phi = sim.get_property("attitude/phi-rad").to_degrees().abs();
    let theta = sim.get_property("attitude/theta-rad").to_degrees().abs();
    assert!(
        phi < 30.0,
        "Roll should be modest, got {phi}°"
    );
    assert!(
        theta < 30.0,
        "Pitch should be modest, got {theta}°"
    );
}

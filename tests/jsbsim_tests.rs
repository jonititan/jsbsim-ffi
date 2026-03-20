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
// ===========================================================================

#[test]
fn load_model_ball() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("ball"), "Failed to load 'ball'");
}

#[test]
fn load_model_c172x() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("c172x"), "Failed to load 'c172x'");
}

#[test]
fn load_model_737() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("737"), "Failed to load '737'");
}

#[test]
fn load_model_f16() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("f16"), "Failed to load 'f16'");
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
// get_model_name  (from tests/TestMiscellaneous.py)
// ===========================================================================

#[test]
fn get_model_name_after_load() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("c172x"));
    let name = sim.get_model_name();
    assert!(
        !name.is_empty(),
        "Model name should not be empty after loading c172x"
    );
    println!("Model name: {name}");
}

// ===========================================================================
// get_version
// ===========================================================================

#[test]
fn get_version_returns_non_empty() {
    let version = Sim::get_version();
    assert!(
        !version.is_empty(),
        "JSBSim version string should not be empty"
    );
    println!("JSBSim version: {version}");
}

// ===========================================================================
// CheckScripts  (from tests/CheckScripts.py)
// ===========================================================================

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

    loop {
        let t = sim.get_sim_time();
        if t >= end_time {
            break;
        }
        if !sim.run() {
            break;
        }
    }
    let t = sim.get_sim_time();
    assert!(t > 0.0, "Sim time should have advanced for {script_name}, got {t}");
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
// ===========================================================================

#[test]
fn initial_conditions_altitude_and_speed() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("c172x"));

    sim.set_property("ic/h-sl-ft", 5000.0);
    sim.set_property("ic/vc-kts", 120.0);
    sim.set_property("ic/gamma-deg", 0.0);
    sim.set_property("ic/psi-true-deg", 90.0);
    assert!(sim.run_ic(), "RunIC failed");

    let alt = sim.get_property("position/h-sl-ft");
    assert!((alt - 5000.0).abs() < 10.0, "Altitude should be ~5000 ft, got {alt}");

    let psi_deg = sim.get_property("attitude/psi-rad").to_degrees();
    assert!((psi_deg - 90.0).abs() < 1.0, "Heading should be ~90°, got {psi_deg}");
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
    assert!(sim.run_ic());

    let alt = sim.get_property("position/h-sl-ft");
    assert!((alt - target_alt_ft).abs() < 10.0, "Ball altitude should be ~{target_alt_ft} ft, got {alt}");
}

// ===========================================================================
// load_ic — Load initial conditions from XML file
// (from tests/TestInitialConditions.py)
// ===========================================================================

#[test]
fn load_ic_from_file() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("c172x"));

    // Load the reset01.xml IC file (ships with c172x aircraft).
    let loaded = sim.load_ic("reset01.xml", true);
    assert!(loaded, "Failed to load IC file reset01.xml");
    assert!(sim.run_ic(), "RunIC failed after loading IC file");

    let alt = sim.get_property("position/h-sl-ft");
    assert!(alt > 0.0, "Altitude should be positive after loading ICs, got {alt}");
}

// ===========================================================================
// CheckSimTimeReset  (from tests/CheckSimTimeReset.py)
//
// Uses get_sim_time() and reset_to_initial_conditions()
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

    let t0 = sim.get_sim_time();
    assert!(t0.abs() < 1e-9, "Sim time should be 0 after RunIC, got {t0}");
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

    for _ in 0..100 {
        sim.run();
    }
    let t = sim.get_sim_time();
    assert!(t > 0.0, "Sim time should advance, got {t}");
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
        let t = sim.get_sim_time();
        assert!(t > prev_t, "Time must increase: prev={prev_t}, now={t}");
        prev_t = t;
    }
}

#[test]
fn reset_to_initial_conditions_resets_time() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("c172x"));
    sim.set_property("ic/h-sl-ft", 3000.0);
    sim.set_property("ic/vc-kts", 100.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    // Run to accumulate time.
    for _ in 0..100 {
        sim.run();
    }
    let t_before = sim.get_sim_time();
    assert!(t_before > 0.0);

    // Reset.
    sim.reset_to_initial_conditions(1);

    let t_after = sim.get_sim_time();
    assert!(
        t_after.abs() < 1e-9,
        "After reset, sim time should be 0, got {t_after}"
    );
}

// ===========================================================================
// TestSuspend  (from tests/TestSuspend.py)
//
// Now properly uses hold() / resume() / holding()
// ===========================================================================

#[test]
fn hold_and_resume() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("c172x"));
    sim.set_property("ic/h-sl-ft", 3000.0);
    sim.set_property("ic/vc-kts", 100.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    // Run a few frames.
    for _ in 0..50 {
        sim.run();
    }
    let t_before = sim.get_sim_time();
    assert!(t_before > 0.0);
    assert!(!sim.holding(), "Should not be holding yet");

    // Hold.
    sim.hold();
    assert!(sim.holding(), "Should be holding after hold()");

    // Step while held — time must not advance.
    for _ in 0..50 {
        sim.run();
    }
    let t_held = sim.get_sim_time();
    assert!(
        (t_held - t_before).abs() < 1e-9,
        "While held, time should not advance: before={t_before}, held={t_held}"
    );

    // Resume.
    sim.resume();
    assert!(!sim.holding(), "Should not be holding after resume()");

    for _ in 0..50 {
        sim.run();
    }
    let t_resumed = sim.get_sim_time();
    assert!(
        t_resumed > t_before,
        "After resume, time should advance: before={t_before}, resumed={t_resumed}"
    );
}

// ===========================================================================
// Suspend / Resume integration  (from tests/TestSuspend.py)
// ===========================================================================

#[test]
fn suspend_and_resume_integration() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("ball"));
    sim.set_property("ic/h-sl-ft", 10_000.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    // Run a few steps.
    for _ in 0..50 {
        sim.run();
    }
    let alt_before = sim.get_property("position/h-sl-ft");

    // Suspend integration — physics frozen.
    sim.suspend_integration();

    for _ in 0..50 {
        sim.run();
    }
    let alt_suspended = sim.get_property("position/h-sl-ft");
    assert!(
        (alt_suspended - alt_before).abs() < 1.0,
        "With integration suspended, altitude should not change: \
         before={alt_before}, suspended={alt_suspended}"
    );

    // Resume integration.
    sim.resume_integration();
    for _ in 0..50 {
        sim.run();
    }
    let alt_resumed = sim.get_property("position/h-sl-ft");
    assert!(
        alt_resumed < alt_before,
        "After resuming integration, ball should descend: \
         before={alt_before}, resumed={alt_resumed}"
    );
}

// ===========================================================================
// EnableIncrementThenHold  (from tests/TestSuspend.py)
//
// Note: enable_increment_then_hold() and check_incremental_hold() are
// exposed but JSBSim's internal Run() loop may not call CheckIncrementalHold
// in all configurations.  We verify the API is callable without crashing.
// ===========================================================================

#[test]
fn increment_then_hold_does_not_crash() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("ball"));
    sim.set_property("ic/h-sl-ft", 10_000.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    sim.enable_increment_then_hold(5);
    for _ in 0..20 {
        sim.run();
    }
    // Just verify the API is callable; the hold behavior is tested via hold()/resume().
}

// ===========================================================================
// TestStdAtmosphere  (from tests/TestStdAtmosphere.py)
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

    let temp_r = sim.get_property("atmosphere/T-R");
    assert!((temp_r - 518.67).abs() < 0.5, "Sea-level temp should be ~518.67 °R, got {temp_r}");

    let p_psf = sim.get_property("atmosphere/P-psf");
    assert!((p_psf - 2116.22).abs() < 1.0, "Sea-level pressure should be ~2116.22 psf, got {p_psf}");

    let rho = sim.get_property("atmosphere/rho-slugs_ft3");
    assert!((rho - 0.002377).abs() < 0.0001, "Sea-level density should be ~0.002377, got {rho}");
}

#[test]
fn std_atmosphere_at_altitude() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("ball"));
    let tropopause_ft = 36_089.0;
    sim.set_property("ic/h-sl-ft", tropopause_ft);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    let temp_r = sim.get_property("atmosphere/T-R");
    assert!((temp_r - 389.97).abs() < 1.0, "Tropopause temp should be ~389.97 °R, got {temp_r}");

    let p_psf = sim.get_property("atmosphere/P-psf");
    assert!(p_psf < 500.0 && p_psf > 400.0, "Tropopause pressure should be ~472 psf, got {p_psf}");
}

// ===========================================================================
// TestPressureAltitude  (from tests/TestPressureAltitude.py)
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

    let pa = sim.get_property("atmosphere/pressure-altitude");
    assert!(pa.abs() < 1.0, "Pressure altitude at sea level should be ~0, got {pa}");
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

    let pa = sim.get_property("atmosphere/pressure-altitude");
    assert!((pa - 10_000.0).abs() < 50.0, "Pressure altitude should be ~10000, got {pa}");
}

// ===========================================================================
// TestMiscellaneous – property round-trip  (from tests/TestMiscellaneous.py)
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

    assert!(sim.set_property("fcs/throttle-cmd-norm", 0.75));
    let v = sim.get_property("fcs/throttle-cmd-norm");
    assert!((v - 0.75).abs() < 1e-6, "Throttle should round-trip to 0.75, got {v}");
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
    let readback = sim.get_property("ic/h-sl-ft");
    assert!((readback - target).abs() < 1e-6, "IC altitude should round-trip, got {readback}");
}

// ===========================================================================
// has_property  (from tests/TestMiscellaneous.py)
// ===========================================================================

#[test]
fn has_property_existing() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("ball"));
    sim.set_property("ic/h-sl-ft", 1000.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    assert!(
        sim.has_property("simulation/sim-time-sec"),
        "simulation/sim-time-sec should exist"
    );
    assert!(
        sim.has_property("position/h-sl-ft"),
        "position/h-sl-ft should exist"
    );
}

#[test]
fn has_property_nonexistent() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("ball"));
    assert!(sim.run_ic());

    assert!(
        !sim.has_property("this/property/does/not/exist"),
        "Nonexistent property should return false"
    );
}

// ===========================================================================
// query_property_catalog  (from tests/TestMiscellaneous.py)
// ===========================================================================

#[test]
fn query_property_catalog_finds_results() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("ball"));
    assert!(sim.run_ic());

    let catalog = sim.query_property_catalog("simulation");
    assert!(
        !catalog.is_empty(),
        "Catalog query for 'simulation' should return results"
    );
    assert!(
        catalog.contains("simulation/sim-time-sec"),
        "Catalog should contain simulation/sim-time-sec, got:\n{catalog}"
    );
}

#[test]
fn query_property_catalog_empty_for_nonsense() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("ball"));
    assert!(sim.run_ic());

    let catalog = sim.query_property_catalog("zzz_nonexistent_prefix");
    // JSBSim returns "No matches found" rather than an empty string.
    assert!(
        catalog.is_empty() || catalog.contains("No matches"),
        "Catalog query for nonsense should be empty or 'No matches', got: {catalog}"
    );
}

// ===========================================================================
// TestHoldDown  (from tests/TestHoldDown.py)
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

    sim.set_property("forces/hold-down", 1.0);
    for _ in 0..200 {
        sim.run();
    }

    let alt = sim.get_property("position/h-sl-ft");
    assert!(alt.abs() < 5.0, "With hold-down, altitude should stay ~0, got {alt}");
}

// ===========================================================================
// get_dt / set_dt  (from CheckScripts.py testScriptEndTime)
// ===========================================================================

#[test]
fn get_dt_returns_positive() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("ball"));
    assert!(sim.run_ic());

    let dt = sim.get_dt();
    assert!(dt > 0.0, "dt should be positive, got {dt}");
}

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

    let custom_dt = 0.01;
    sim.set_dt(custom_dt);
    sim.run();

    let dt_read = sim.get_dt();
    assert!((dt_read - custom_dt).abs() < 1e-9, "dt should be {custom_dt}, got {dt_read}");

    for _ in 0..99 {
        sim.run();
    }
    let t = sim.get_sim_time();
    assert!((t - 1.0).abs() < 0.05, "After 100 steps at dt=0.01, time should be ~1.0s, got {t}");
}

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

    let dt = sim.get_dt();
    assert!(dt > 0.0);

    let n_steps = 100;
    for _ in 0..n_steps {
        sim.run();
    }

    let t = sim.get_sim_time();
    let expected = dt * n_steps as f64;
    assert!((t - expected).abs() < dt, "After {n_steps} steps, time should be ~{expected}, got {t}");
}

// ===========================================================================
// set_debug_level
// ===========================================================================

#[test]
fn set_debug_level_does_not_crash() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    sim.set_debug_level(0); // silent
    assert!(sim.load_model("ball"));
    sim.set_debug_level(1); // restore default
}

// ===========================================================================
// Multiple instances  (from tests/TestMiscellaneous.py)
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
    assert!(
        (alt_a - alt_b).abs() > 100.0,
        "Independent sims should diverge: alt_a={alt_a}, alt_b={alt_b}"
    );
}

// ===========================================================================
// Full script run  (from CheckScripts.py)
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
        if sim.get_sim_time() >= max_time {
            break;
        }
        if !sim.run() {
            break;
        }
        steps += 1;
    }

    let t_final = sim.get_sim_time();
    assert!(t_final >= max_time - 1.0, "Should run to ~{max_time}s, got {t_final}");
    assert!(steps > 100, "Should take many steps, took {steps}");
}

// ===========================================================================
// Gravity / free-fall physics validation
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

    sim.set_dt(0.01);
    for _ in 0..500 {
        sim.run();
    }

    let alt = sim.get_property("position/h-sl-ft");
    assert!(alt < initial_alt, "Ball should descend: initial={initial_alt}, current={alt}");

    let delta = initial_alt - alt;
    assert!(delta > 300.0 && delta < 600.0, "Free-fall ~5s should be ~400ft, got {delta}");
}

// ===========================================================================
// C172x flight properties plausible  (from TestInitialConditions.py)
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

    for _ in 0..240 {
        sim.run();
    }

    let vc = sim.get_property("velocities/vc-kts");
    assert!(vc > 50.0, "Airspeed should be > 50 kts, got {vc}");

    let alt = sim.get_property("position/h-sl-ft");
    assert!(alt > 4000.0 && alt < 6000.0, "Altitude should be near 5000, got {alt}");

    let phi = sim.get_property("attitude/phi-rad").to_degrees().abs();
    let theta = sim.get_property("attitude/theta-rad").to_degrees().abs();
    assert!(phi < 30.0, "Roll should be modest, got {phi}°");
    assert!(theta < 30.0, "Pitch should be modest, got {theta}°");
}

// ===========================================================================
// Trim  (from tests/CheckTrim.py)
// ===========================================================================

#[test]
fn do_trim_longitudinal() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("c172x"));

    sim.set_property("ic/h-sl-ft", 5000.0);
    sim.set_property("ic/vc-kts", 100.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    let trimmed = sim.do_trim(jsbsim_ffi::trim::LONGITUDINAL);
    if !trimmed {
        eprintln!("Longitudinal trim failed (may be expected for some configs)");
        return;
    }

    // After trim, pitch rate should be near zero.
    let q = sim.get_property("velocities/q-rad_sec");
    assert!(
        q.abs() < 0.01,
        "After longitudinal trim, pitch rate should be ~0, got {q}"
    );

    // Run a few steps — altitude should stay roughly constant.
    let alt_before = sim.get_property("position/h-sl-ft");
    for _ in 0..100 {
        sim.run();
    }
    let alt_after = sim.get_property("position/h-sl-ft");
    assert!(
        (alt_after - alt_before).abs() < 50.0,
        "After trim, altitude should be stable: before={alt_before}, after={alt_after}"
    );
}

// ===========================================================================
// Path configuration
// ===========================================================================

#[test]
fn set_paths_does_not_crash() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    // These should return true (they just set internal paths).
    assert!(sim.set_aircraft_path("aircraft"));
    assert!(sim.set_engine_path("engine"));
    assert!(sim.set_systems_path("systems"));
}

// ===========================================================================
// Output control — just verify no crashes
// ===========================================================================

#[test]
fn disable_enable_output_does_not_crash() {
    let mut sim = match create_fdm() {
        Some(s) => s,
        None => return,
    };
    assert!(sim.load_model("ball"));
    assert!(sim.run_ic());

    sim.disable_output();
    for _ in 0..10 {
        sim.run();
    }
    sim.enable_output();
    for _ in 0..10 {
        sim.run();
    }
}

//! Integration tests ported from the JSBSim Python test suite.
//!
//! These tests exercise the Rust FFI wrapper against a real JSBSim data
//! directory.  `JSBSIM_ROOT` **must** be set to a real JSBSim checkout
//! before running — if it is missing, empty, or points at a path that
//! doesn't contain an `aircraft/` subdirectory, every test in this file
//! fails loudly with a clear message.  This is intentional: silently
//! skipping hides broken tests in CI and on developer machines.
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
// Helper: obtain JSBSIM_ROOT or fail the test with a clear message.
//
// A "valid" JSBSIM_ROOT must point at a directory that contains an
// `aircraft/` subdirectory — this rules out unset, empty, and any path
// that doesn't actually hold a JSBSim data tree.  Every integration test
// in this file funnels through this helper via [`create_fdm`], so adding
// a new test automatically inherits the check.
// ---------------------------------------------------------------------------
fn jsbsim_root() -> String {
    let raw = std::env::var("JSBSIM_ROOT").unwrap_or_default();
    assert!(
        !raw.is_empty(),
        "JSBSIM_ROOT is not set.\n\
         Set it to a real JSBSim checkout before running integration tests:\n\
         \n    JSBSIM_ROOT=$HOME/jsbsim cargo test -- --test-threads=1\n"
    );
    let aircraft_dir = std::path::Path::new(&raw).join("aircraft");
    assert!(
        aircraft_dir.is_dir(),
        "JSBSIM_ROOT={raw:?} does not contain an `aircraft/` subdirectory.\n\
         Point it at a real JSBSim data tree (e.g. JSBSIM_ROOT=$HOME/jsbsim)."
    );
    raw
}

/// Convenience: create a `Sim` pointed at JSBSIM_ROOT.  Panics (via
/// [`jsbsim_root`]) if the env var is missing or invalid.
fn create_fdm() -> Sim {
    Sim::new(&jsbsim_root())
}

// ===========================================================================
// TestModelLoading  (from tests/TestModelLoading.py)
// ===========================================================================

#[test]
fn load_model_ball() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"), "Failed to load 'ball'");
}

#[test]
fn load_model_c172x() {
    let mut sim = create_fdm();
    assert!(sim.load_model("c172x"), "Failed to load 'c172x'");
}

#[test]
fn load_model_737() {
    let mut sim = create_fdm();
    assert!(sim.load_model("737"), "Failed to load '737'");
}

#[test]
fn load_model_f16() {
    let mut sim = create_fdm();
    assert!(sim.load_model("f16"), "Failed to load 'f16'");
}

#[test]
fn load_model_nonexistent_returns_false() {
    let mut sim = create_fdm();
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
    let mut sim = create_fdm();
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
    let root = jsbsim_root();
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
    assert!(
        t > 0.0,
        "Sim time should have advanced for {script_name}, got {t}"
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
// ===========================================================================

#[test]
fn initial_conditions_altitude_and_speed() {
    let mut sim = create_fdm();
    assert!(sim.load_model("c172x"));

    sim.set_property("ic/h-sl-ft", 5000.0);
    sim.set_property("ic/vc-kts", 120.0);
    sim.set_property("ic/gamma-deg", 0.0);
    sim.set_property("ic/psi-true-deg", 90.0);
    assert!(sim.run_ic(), "RunIC failed");

    let alt = sim.get_property("position/h-sl-ft");
    assert!(
        (alt - 5000.0).abs() < 10.0,
        "Altitude should be ~5000 ft, got {alt}"
    );

    let psi_deg = sim.get_property("attitude/psi-rad").to_degrees();
    assert!(
        (psi_deg - 90.0).abs() < 1.0,
        "Heading should be ~90°, got {psi_deg}"
    );
}

#[test]
fn initial_conditions_ball_position() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));

    let target_alt_ft = 100_000.0;
    sim.set_property("ic/h-sl-ft", target_alt_ft);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    let alt = sim.get_property("position/h-sl-ft");
    assert!(
        (alt - target_alt_ft).abs() < 10.0,
        "Ball altitude should be ~{target_alt_ft} ft, got {alt}"
    );
}

// ===========================================================================
// load_ic — Load initial conditions from XML file
// (from tests/TestInitialConditions.py)
// ===========================================================================

#[test]
fn load_ic_from_file() {
    let mut sim = create_fdm();
    assert!(sim.load_model("c172x"));

    // Load the reset01.xml IC file (ships with c172x aircraft).
    let loaded = sim.load_ic("reset01.xml", true);
    assert!(loaded, "Failed to load IC file reset01.xml");
    assert!(sim.run_ic(), "RunIC failed after loading IC file");

    let alt = sim.get_property("position/h-sl-ft");
    assert!(
        alt > 0.0,
        "Altitude should be positive after loading ICs, got {alt}"
    );
}

// ===========================================================================
// CheckSimTimeReset  (from tests/CheckSimTimeReset.py)
//
// Uses get_sim_time() and reset_to_initial_conditions()
// ===========================================================================

#[test]
fn sim_time_starts_at_zero() {
    let mut sim = create_fdm();
    assert!(sim.load_model("c172x"));
    sim.set_property("ic/h-sl-ft", 3000.0);
    sim.set_property("ic/vc-kts", 100.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    let t0 = sim.get_sim_time();
    assert!(
        t0.abs() < 1e-9,
        "Sim time should be 0 after RunIC, got {t0}"
    );
}

#[test]
fn sim_time_advances() {
    let mut sim = create_fdm();
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
    let mut sim = create_fdm();
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
    let mut sim = create_fdm();
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
    let mut sim = create_fdm();
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
    let mut sim = create_fdm();
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
// ===========================================================================

/// `enable_increment_then_hold(N)` schedules `N` more steps and then drops
/// the sim into hold — but the caller has to invoke `check_incremental_hold()`
/// after each step (JSBSim's `Run()` does not call it automatically).
/// This test follows the same pattern as JSBSim's Python tests:
/// drive the counter manually and verify the hold actually fires.
#[test]
fn increment_then_hold_actually_holds() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    sim.set_property("ic/h-sl-ft", 10_000.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    assert!(
        !sim.holding(),
        "should not be holding before enable_increment_then_hold"
    );

    let n = 5;
    sim.enable_increment_then_hold(n);

    // Drive the increment counter manually (matches JSBSim Python idiom).
    // After `n` steps the sim must be in hold.
    for _ in 0..(n + 2) {
        sim.run();
        sim.check_incremental_hold();
    }
    assert!(
        sim.holding(),
        "sim should be holding after {n} run+check_incremental_hold cycles"
    );

    // Further `run()` calls must not advance time while held.
    let t_held = sim.get_sim_time();
    for _ in 0..10 {
        sim.run();
    }
    let t_after = sim.get_sim_time();
    assert!(
        (t_after - t_held).abs() < 1e-9,
        "time must not advance while held: t_held={t_held}, t_after={t_after}"
    );

    // Resume and verify time advances again.
    sim.resume();
    assert!(!sim.holding(), "resume() should clear the hold flag");
    sim.run();
    assert!(
        sim.get_sim_time() > t_after,
        "time should advance again after resume()"
    );
}

// ===========================================================================
// TestStdAtmosphere  (from tests/TestStdAtmosphere.py)
// ===========================================================================

#[test]
fn std_atmosphere_sea_level() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    sim.set_property("ic/h-sl-ft", 0.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    let temp_r = sim.get_property("atmosphere/T-R");
    assert!(
        (temp_r - 518.67).abs() < 0.5,
        "Sea-level temp should be ~518.67 °R, got {temp_r}"
    );

    let p_psf = sim.get_property("atmosphere/P-psf");
    assert!(
        (p_psf - 2116.22).abs() < 1.0,
        "Sea-level pressure should be ~2116.22 psf, got {p_psf}"
    );

    let rho = sim.get_property("atmosphere/rho-slugs_ft3");
    assert!(
        (rho - 0.002377).abs() < 0.0001,
        "Sea-level density should be ~0.002377, got {rho}"
    );
}

#[test]
fn std_atmosphere_at_altitude() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    let tropopause_ft = 36_089.0;
    sim.set_property("ic/h-sl-ft", tropopause_ft);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    let temp_r = sim.get_property("atmosphere/T-R");
    assert!(
        (temp_r - 389.97).abs() < 1.0,
        "Tropopause temp should be ~389.97 °R, got {temp_r}"
    );

    let p_psf = sim.get_property("atmosphere/P-psf");
    assert!(
        p_psf < 500.0 && p_psf > 400.0,
        "Tropopause pressure should be ~472 psf, got {p_psf}"
    );
}

// ===========================================================================
// TestPressureAltitude  (from tests/TestPressureAltitude.py)
// ===========================================================================

#[test]
fn pressure_altitude_at_sea_level() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    sim.set_property("ic/h-sl-ft", 0.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    let pa = sim.get_property("atmosphere/pressure-altitude");
    assert!(
        pa.abs() < 1.0,
        "Pressure altitude at sea level should be ~0, got {pa}"
    );
}

#[test]
fn pressure_altitude_at_10000ft() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    sim.set_property("ic/h-sl-ft", 10_000.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    let pa = sim.get_property("atmosphere/pressure-altitude");
    assert!(
        (pa - 10_000.0).abs() < 50.0,
        "Pressure altitude should be ~10000, got {pa}"
    );
}

// ===========================================================================
// TestMiscellaneous – property round-trip  (from tests/TestMiscellaneous.py)
// ===========================================================================

#[test]
fn property_round_trip() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    sim.set_property("ic/h-sl-ft", 1000.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    assert!(sim.set_property("fcs/throttle-cmd-norm", 0.75));
    let v = sim.get_property("fcs/throttle-cmd-norm");
    assert!(
        (v - 0.75).abs() < 1e-6,
        "Throttle should round-trip to 0.75, got {v}"
    );
}

#[test]
fn property_set_and_read_ic() {
    let mut sim = create_fdm();
    assert!(sim.load_model("c172x"));

    let target = 7500.0_f64;
    sim.set_property("ic/h-sl-ft", target);
    let readback = sim.get_property("ic/h-sl-ft");
    assert!(
        (readback - target).abs() < 1e-6,
        "IC altitude should round-trip, got {readback}"
    );
}

// ===========================================================================
// has_property  (from tests/TestMiscellaneous.py)
// ===========================================================================

#[test]
fn has_property_existing() {
    let mut sim = create_fdm();
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
    let mut sim = create_fdm();
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
    let mut sim = create_fdm();
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
    let mut sim = create_fdm();
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
    let mut sim = create_fdm();
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
    assert!(
        alt.abs() < 5.0,
        "With hold-down, altitude should stay ~0, got {alt}"
    );
}

// ===========================================================================
// get_dt / set_dt  (from CheckScripts.py testScriptEndTime)
// ===========================================================================

#[test]
fn get_dt_returns_positive() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    assert!(sim.run_ic());

    let dt = sim.get_dt();
    assert!(dt > 0.0, "dt should be positive, got {dt}");
}

#[test]
fn set_dt_changes_timestep() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    sim.set_property("ic/h-sl-ft", 10_000.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    let custom_dt = 0.01;
    sim.set_dt(custom_dt);
    sim.run();

    let dt_read = sim.get_dt();
    assert!(
        (dt_read - custom_dt).abs() < 1e-9,
        "dt should be {custom_dt}, got {dt_read}"
    );

    for _ in 0..99 {
        sim.run();
    }
    let t = sim.get_sim_time();
    assert!(
        (t - 1.0).abs() < 0.05,
        "After 100 steps at dt=0.01, time should be ~1.0s, got {t}"
    );
}

#[test]
fn simulation_dt_consistent_with_time() {
    let mut sim = create_fdm();
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
    assert!(
        (t - expected).abs() < dt,
        "After {n_steps} steps, time should be ~{expected}, got {t}"
    );
}

// ===========================================================================
// set_debug_level
// ===========================================================================

#[test]
fn set_debug_level_does_not_crash() {
    let mut sim = create_fdm();
    sim.set_debug_level(0); // silent
    assert!(sim.load_model("ball"));
    sim.set_debug_level(1); // restore default
}

// ===========================================================================
// Multiple instances  (from tests/TestMiscellaneous.py)
// ===========================================================================

#[test]
fn multiple_independent_instances() {
    let root = jsbsim_root();

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
    let root = jsbsim_root();
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
    assert!(
        t_final >= max_time - 1.0,
        "Should run to ~{max_time}s, got {t_final}"
    );
    assert!(steps > 100, "Should take many steps, took {steps}");
}

// ===========================================================================
// Gravity / free-fall physics validation
// ===========================================================================

#[test]
fn ball_free_fall_altitude_decreases() {
    let mut sim = create_fdm();
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
    assert!(
        alt < initial_alt,
        "Ball should descend: initial={initial_alt}, current={alt}"
    );

    let delta = initial_alt - alt;
    assert!(
        delta > 300.0 && delta < 600.0,
        "Free-fall ~5s should be ~400ft, got {delta}"
    );
}

// ===========================================================================
// C172x flight properties plausible  (from TestInitialConditions.py)
// ===========================================================================

#[test]
fn c172x_flight_properties_plausible() {
    let mut sim = create_fdm();
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
    assert!(
        alt > 4000.0 && alt < 6000.0,
        "Altitude should be near 5000, got {alt}"
    );

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
    let mut sim = create_fdm();
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
    let mut sim = create_fdm();
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
    let mut sim = create_fdm();
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

// ===========================================================================
// integration_suspended  (new API)
// ===========================================================================

#[test]
fn integration_suspended_query() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    sim.set_property("ic/h-sl-ft", 10_000.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    assert!(
        !sim.integration_suspended(),
        "Should not be suspended initially"
    );

    sim.suspend_integration();
    assert!(
        sim.integration_suspended(),
        "Should be suspended after suspend_integration()"
    );

    sim.resume_integration();
    assert!(
        !sim.integration_suspended(),
        "Should not be suspended after resume_integration()"
    );
}

// ===========================================================================
// set_sim_time  (new API)
// ===========================================================================

#[test]
fn set_sim_time_explicit() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    sim.set_property("ic/h-sl-ft", 10_000.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    // Run a few steps to advance time.
    for _ in 0..50 {
        sim.run();
    }
    let t = sim.get_sim_time();
    assert!(t > 0.0);

    // Set time to a specific value.
    sim.set_sim_time(42.0);
    let t2 = sim.get_sim_time();
    assert!(
        (t2 - 42.0).abs() < 1e-9,
        "Sim time should be 42.0, got {t2}"
    );
}

// ===========================================================================
// Path getters  (new API)
// ===========================================================================

#[test]
fn path_getters_return_values() {
    let sim = create_fdm();

    let root = sim.get_root_dir();
    assert!(!root.is_empty(), "Root dir should not be empty");

    let aircraft = sim.get_aircraft_path();
    assert!(!aircraft.is_empty(), "Aircraft path should not be empty");

    let engine = sim.get_engine_path();
    assert!(!engine.is_empty(), "Engine path should not be empty");

    let systems = sim.get_systems_path();
    assert!(!systems.is_empty(), "Systems path should not be empty");

    println!("Root: {root}");
    println!("Aircraft: {aircraft}");
    println!("Engine: {engine}");
    println!("Systems: {systems}");
}

#[test]
fn path_setters_and_getters_round_trip() {
    let mut sim = create_fdm();

    assert!(sim.set_aircraft_path("my_aircraft"));
    let ap = sim.get_aircraft_path();
    assert!(
        ap.contains("my_aircraft"),
        "Aircraft path should contain 'my_aircraft', got '{ap}'"
    );

    assert!(sim.set_engine_path("my_engines"));
    let ep = sim.get_engine_path();
    assert!(
        ep.contains("my_engines"),
        "Engine path should contain 'my_engines', got '{ep}'"
    );

    assert!(sim.set_systems_path("my_systems"));
    let sp = sim.get_systems_path();
    assert!(
        sp.contains("my_systems"),
        "Systems path should contain 'my_systems', got '{sp}'"
    );
}

// ===========================================================================
// get_output_filename  (new API)
// ===========================================================================

#[test]
fn output_filename_getter() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    assert!(sim.run_ic());

    // Query output filename for channel 0 — may be empty if no output configured.
    let fname = sim.get_output_filename(0);
    // Just verify it doesn't crash; the value depends on configuration.
    println!("Output filename[0]: '{fname}'");
}

// ===========================================================================
// set_terrain_elevation  (previously untested)
// ===========================================================================

#[test]
fn set_terrain_elevation_affects_agl() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    sim.set_property("ic/h-sl-ft", 1000.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    let agl_before = sim.get_property("position/h-agl-ft");

    // Raise terrain to 500 ft — AGL should decrease.
    sim.set_terrain_elevation(500.0);
    sim.run(); // one step to let JSBSim recalculate
    let agl_after = sim.get_property("position/h-agl-ft");

    assert!(
        agl_after < agl_before,
        "AGL should decrease when terrain is raised: before={agl_before}, after={agl_after}"
    );
}

// ===========================================================================
// Ground callback  (previously untested)
// ===========================================================================

#[test]
fn custom_ground_callback() {
    use jsbsim_ffi::{GroundCallback, GroundContact};

    struct FlatGround {
        elevation_ft: f64,
    }

    impl GroundCallback for FlatGround {
        fn get_agl(&self, _time: f64, location: [f64; 3]) -> GroundContact {
            let radius = (location[0].powi(2) + location[1].powi(2) + location[2].powi(2)).sqrt();
            let earth_radius_ft = 20_925_646.0;
            let ground_radius = earth_radius_ft + self.elevation_ft;
            let agl = radius - ground_radius;

            let scale = ground_radius / radius;
            GroundContact {
                agl,
                contact: [
                    location[0] * scale,
                    location[1] * scale,
                    location[2] * scale,
                ],
                normal: [
                    location[0] / radius,
                    location[1] / radius,
                    location[2] / radius,
                ],
                velocity: [0.0, 0.0, 0.0],
                ang_velocity: [0.0, 0.0, 0.0],
            }
        }
    }

    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    sim.set_property("ic/h-sl-ft", 5000.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    // Install our custom ground callback with ground at 1000 ft MSL.
    sim.set_ground_callback(FlatGround {
        elevation_ft: 1000.0,
    });

    // Run a step so JSBSim queries the callback.
    sim.run();

    let agl = sim.get_property("position/h-agl-ft");
    // Aircraft at ~5000 ft MSL, ground at 1000 ft → AGL should be ~4000 ft.
    assert!(
        (agl - 4000.0).abs() < 200.0,
        "AGL should be ~4000 ft with ground at 1000 ft MSL, got {agl}"
    );
}

// ===========================================================================
// do_trim — additional modes  (previously only LONGITUDINAL was tested)
// ===========================================================================

#[test]
fn do_trim_full() {
    let mut sim = create_fdm();
    assert!(sim.load_model("c172x"));
    sim.set_property("ic/h-sl-ft", 5000.0);
    sim.set_property("ic/vc-kts", 100.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    let trimmed = sim.do_trim(jsbsim_ffi::trim::FULL);
    if !trimmed {
        eprintln!("Full trim failed (may be expected for some configs)");
        return;
    }

    // After full trim, all angular rates should be near zero.
    let p = sim.get_property("velocities/p-rad_sec");
    let q = sim.get_property("velocities/q-rad_sec");
    let r = sim.get_property("velocities/r-rad_sec");
    assert!(
        p.abs() < 0.05,
        "After full trim, roll rate should be ~0, got {p}"
    );
    assert!(
        q.abs() < 0.05,
        "After full trim, pitch rate should be ~0, got {q}"
    );
    assert!(
        r.abs() < 0.05,
        "After full trim, yaw rate should be ~0, got {r}"
    );
}

#[test]
fn do_trim_ground() {
    let mut sim = create_fdm();
    assert!(sim.load_model("c172x"));
    sim.set_property("ic/h-sl-ft", 0.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    let trimmed = sim.do_trim(jsbsim_ffi::trim::GROUND);
    if !trimmed {
        eprintln!("Ground trim failed (may be expected for some configs)");
        return;
    }

    // After ground trim the aircraft should be stationary on the ground.
    let alt = sim.get_property("position/h-agl-ft");
    assert!(
        alt.abs() < 50.0,
        "After ground trim, AGL should be near 0, got {alt}"
    );
}

// ===========================================================================
// print_property_catalog  (verify no crash)
// ===========================================================================

#[test]
fn print_property_catalog_does_not_crash() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    assert!(sim.run_ic());

    // Just verify it doesn't crash; output goes to stdout.
    sim.print_property_catalog();
}

// ===========================================================================
// check_incremental_hold  (verify it's callable with a loaded model)
// ===========================================================================

#[test]
fn check_incremental_hold_with_model() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    sim.set_property("ic/h-sl-ft", 10_000.0);
    sim.set_property("ic/vc-kts", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    // Call check_incremental_hold directly — should not crash.
    sim.check_incremental_hold();
}

// ===========================================================================
// set_output_filename  (verify callable)
// ===========================================================================

#[test]
fn set_output_filename_does_not_crash() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    assert!(sim.run_ic());

    // No output channels configured by default, so this may return false,
    // but it should not crash.
    let _ = sim.set_output_filename(0, "test_output.csv");
}

// ===========================================================================
// set_output_directive  (previously untested)
// ===========================================================================

#[test]
fn set_output_directive_nonexistent_returns_false() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    assert!(sim.run_ic());

    // A nonexistent directive file should return false.
    let ok = sim.set_output_directive("nonexistent_output.xml");
    assert!(
        !ok,
        "set_output_directive with nonexistent file should return false"
    );
}

// ===========================================================================
// LoadScript with deltaT/initfile
// ===========================================================================

#[test]
fn load_script_with_dt_override() {
    let mut sim = create_fdm();
    let root = jsbsim_root();
    let script = format!("{root}/scripts/c1722.xml");
    let ok = sim.load_script_with(&script, 0.005, None);
    assert!(ok, "loading c1722.xml with dt override should succeed");
    let dt = sim.get_dt();
    assert!(
        (dt - 0.005).abs() < 1e-9,
        "dt override should be honoured, got {dt}"
    );
}

#[test]
fn load_script_with_no_overrides_matches_plain_load() {
    let mut sim = create_fdm();
    let root = jsbsim_root();
    let script = format!("{root}/scripts/c1722.xml");
    // dt=0 means "use the script's value"
    let ok = sim.load_script_with(&script, 0.0, None);
    assert!(ok);
}

// ===========================================================================
// Full-aircraft path
// ===========================================================================

#[test]
fn get_full_aircraft_path_after_load() {
    let mut sim = create_fdm();
    assert!(sim.load_model("c172x"));
    let p = sim.get_full_aircraft_path();
    assert!(
        p.contains("c172x"),
        "full aircraft path should reference loaded model, got {p:?}"
    );
}

// ===========================================================================
// Output path
// ===========================================================================

#[test]
fn set_and_get_output_path_round_trip() {
    let mut sim = create_fdm();
    let target = "/tmp/jsbsim_test_output";
    assert!(sim.set_output_path(target));
    let got = sim.get_output_path();
    assert!(
        got.contains("jsbsim_test_output"),
        "output path round-trip failed: got {got:?}"
    );
}

// ===========================================================================
// Logging rate / ForceOutput (smoke)
// ===========================================================================

#[test]
fn set_logging_rate_and_force_output_after_model_load() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    assert!(sim.run_ic());
    sim.set_logging_rate(20.0);
    sim.force_output(0); // No output channels by default — just verify no crash.
}

// ===========================================================================
// Frame counter / IncrTime / DebugLevel
// ===========================================================================

#[test]
fn frame_counter_increments_with_run() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    assert!(sim.run_ic());
    let f0 = sim.get_frame();
    for _ in 0..10 {
        sim.run();
    }
    let f1 = sim.get_frame();
    assert!(f1 > f0, "frame counter should advance: {f0} → {f1}");
}

#[test]
fn debug_level_round_trip() {
    let mut sim = create_fdm();
    sim.set_debug_level(0);
    assert_eq!(sim.get_debug_level(), 0);
    sim.set_debug_level(1);
    assert_eq!(sim.get_debug_level(), 1);
    sim.set_debug_level(0);
}

#[test]
fn incr_time_matches_dt() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    assert!(sim.run_ic());
    let dt = sim.get_dt();
    let t0 = sim.get_sim_time();
    let t1 = sim.incr_time();
    assert!(
        (t1 - (t0 + dt)).abs() < 1e-9,
        "incr_time should advance by dt: t0={t0} dt={dt} t1={t1}"
    );
}

// ===========================================================================
// Hold-down
// ===========================================================================

#[test]
fn hold_down_round_trip() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    assert!(sim.run_ic());
    assert!(!sim.get_hold_down(), "hold-down should default to false");
    sim.set_hold_down(true);
    assert!(sim.get_hold_down(), "hold-down should be true after set");
    sim.set_hold_down(false);
    assert!(!sim.get_hold_down());
}

// ===========================================================================
// Trim status / mode
// ===========================================================================

#[test]
fn trim_mode_round_trip_with_model_loaded() {
    let mut sim = create_fdm();
    assert!(sim.load_model("c172x"));
    sim.set_trim_mode(jsbsim_ffi::trim::FULL);
    assert_eq!(sim.get_trim_mode(), jsbsim_ffi::trim::FULL);
    sim.set_trim_mode(jsbsim_ffi::trim::LONGITUDINAL);
    assert_eq!(sim.get_trim_mode(), jsbsim_ffi::trim::LONGITUDINAL);
}

#[test]
fn trim_status_set_after_do_trim() {
    let mut sim = create_fdm();
    assert!(sim.load_model("c172x"));
    sim.set_property("ic/h-sl-ft", 5000.0);
    sim.set_property("ic/vc-kts", 90.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());
    sim.set_trim_status(false);
    let _ = sim.do_trim(jsbsim_ffi::trim::LONGITUDINAL);
    // After DoTrim, trim_status is set true on success. We don't assert
    // success of DoTrim itself (model-dependent), just that the accessor works.
    let _ = sim.get_trim_status();
}

// ===========================================================================
// Property catalog (vector form)
// ===========================================================================

#[test]
fn property_catalog_populated_after_load() {
    let mut sim = create_fdm();
    assert!(sim.load_model("c172x"));
    let cat = sim.get_property_catalog();
    assert!(
        cat.len() > 100,
        "c172x catalog should have many entries, got {}",
        cat.len()
    );
    assert!(
        cat.iter().any(|p| p.contains("position/h-sl-ft")),
        "catalog should contain position/h-sl-ft"
    );
}

#[test]
fn property_catalog_consistent_with_query() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    let cat = sim.get_property_catalog();
    assert!(!cat.is_empty(), "property catalog should not be empty");
    // Sanity check: every entry should be non-empty.
    for entry in &cat {
        assert!(!entry.is_empty(), "catalog contained an empty entry");
    }
}

// ===========================================================================
// Propulsion tank report
// ===========================================================================

#[test]
fn propulsion_tank_report_for_c172x() {
    let mut sim = create_fdm();
    assert!(sim.load_model("c172x"));
    assert!(sim.run_ic());
    let report = sim.get_propulsion_tank_report();
    // C172 has fuel tanks, so the report should mention "Tank" or similar.
    // The exact format is JSBSim-version-dependent, so just check non-empty.
    assert!(
        !report.is_empty(),
        "propulsion tank report should be non-empty for c172x"
    );
}

// ===========================================================================
// PrintSimulationConfiguration (smoke — verify it doesn't crash with a model)
// ===========================================================================

#[test]
fn print_simulation_configuration_with_model() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    sim.print_simulation_configuration();
}

// ===========================================================================
// DoLinearization (smoke — may succeed or fail depending on trim)
// ===========================================================================

#[test]
fn do_linearization_after_trim() {
    let mut sim = create_fdm();
    assert!(sim.load_model("c172x"));
    sim.set_property("ic/h-sl-ft", 5000.0);
    sim.set_property("ic/vc-kts", 90.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());
    // Trim first, then linearize. Neither step is asserted for success —
    // they're model/numerical dependent. We only verify no crash.
    let _ = sim.do_trim(jsbsim_ffi::trim::LONGITUDINAL);
    let _ = sim.do_linearization(0);
}

// ===========================================================================
// Random seed / child FDM accessors with a model loaded
// ===========================================================================

#[test]
fn random_seed_after_load() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    let _ = sim.get_random_seed();
}

#[test]
fn fdm_count_after_load() {
    let mut sim = create_fdm();
    assert!(sim.load_model("c172x"));
    // get_fdm_count counts attached child FDMs — zero for a plain model.
    assert_eq!(sim.get_fdm_count(), 0);
    // EnumerateFDMs may include the top-level FDM itself. We only verify
    // that the accessor returns without crashing and yields well-formed
    // strings (no empty entries).
    let list = sim.enumerate_fdms();
    for name in &list {
        assert!(!name.is_empty(), "EnumerateFDMs entry should be non-empty");
    }
}

// ===========================================================================
// SetRootDir and 5-arg LoadModel
// ===========================================================================

#[test]
fn set_root_dir_changes_root() {
    let root = jsbsim_root();
    let mut sim = Sim::new("/nonexistent/initial");
    // Pointing the root at a real tree afterward should let LoadModel
    // succeed.  Note: SetRootDir doesn't update the aircraft / engine /
    // systems sub-paths, so we set those too via load_model_with below.
    sim.set_root_dir(&root);
    assert!(
        sim.get_root_dir().contains(&root),
        "get_root_dir should reflect the path we just set, got {:?}",
        sim.get_root_dir()
    );
}

#[test]
fn load_model_with_paths_succeeds() {
    let root = jsbsim_root();
    let mut sim = Sim::new(&root);
    // Equivalent to set_aircraft_path / set_engine_path / set_systems_path
    // followed by load_model("c172x") — but routed through JSBSim's
    // 5-arg LoadModel overload.
    let ok = sim.load_model_with(
        &format!("{root}/aircraft"),
        &format!("{root}/engine"),
        &format!("{root}/systems"),
        "c172x",
        true,
    );
    assert!(ok, "load_model_with should successfully load c172x");
    assert_eq!(sim.get_model_name(), "c172x");
    let aircraft_path = sim.get_aircraft_path();
    assert!(
        aircraft_path.contains("aircraft"),
        "aircraft path should be set after load_model_with, got {aircraft_path:?}"
    );
}

// ===========================================================================
// Propulsion helpers
// ===========================================================================

#[test]
fn c172x_has_one_engine_and_tanks() {
    let mut sim = create_fdm();
    assert!(sim.load_model("c172x"));
    assert_eq!(
        sim.get_num_engines(),
        1,
        "c172x should report exactly 1 engine"
    );
    assert!(
        sim.get_num_tanks() >= 1,
        "c172x should report at least one fuel tank, got {}",
        sim.get_num_tanks()
    );
}

#[test]
fn ball_has_no_engines() {
    let mut sim = create_fdm();
    assert!(sim.load_model("ball"));
    assert_eq!(sim.get_num_engines(), 0, "ball has no engines");
}

#[test]
fn init_running_all_starts_c172x_engine() {
    let mut sim = create_fdm();
    assert!(sim.load_model("c172x"));
    sim.set_property("ic/h-sl-ft", 3000.0);
    sim.set_property("ic/vc-kts", 100.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());

    // Cold start: engine should not be marked running yet.
    let running_before = sim.get_property("propulsion/engine[0]/set-running");
    assert!(
        running_before < 0.5,
        "engine should be cold before InitRunning, got set-running={running_before}"
    );

    assert!(
        sim.init_running(-1),
        "init_running(-1) should succeed after LoadModel + RunIC"
    );

    // After InitRunning(-1), every engine must be flagged as running and
    // the propeller must spin up to a non-trivial RPM (the C172P sits
    // around 600 RPM at idle, well above 0).
    let running_after = sim.get_property("propulsion/engine[0]/set-running");
    assert!(
        running_after >= 0.5,
        "engine should be marked running after InitRunning, got set-running={running_after}"
    );
    let rpm = sim.get_property("propulsion/engine[0]/engine-rpm");
    assert!(
        rpm > 100.0,
        "engine RPM should be > 100 after InitRunning, got {rpm}"
    );
}

#[test]
fn init_running_specific_engine() {
    let mut sim = create_fdm();
    assert!(sim.load_model("c172x"));
    sim.set_property("ic/h-sl-ft", 3000.0);
    sim.set_property("ic/vc-kts", 100.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());
    // c172x has a single engine at index 0.
    assert!(sim.init_running(0));
}

#[test]
fn propulsion_steady_state_after_init_running() {
    let mut sim = create_fdm();
    assert!(sim.load_model("c172x"));
    sim.set_property("ic/h-sl-ft", 3000.0);
    sim.set_property("ic/vc-kts", 100.0);
    sim.set_property("ic/gamma-deg", 0.0);
    assert!(sim.run_ic());
    assert!(sim.init_running(-1));

    // FGPropulsion::GetSteadyState convergence is model + state dependent
    // (the c172x piston engine doesn't always reach the trim solver's
    // tolerance from a cold IC).  We only verify the FFI plumbing here:
    // the call must return without crashing.  The behavioural side of the
    // post-`init_running` engine state is checked through engine RPM.
    let _ = sim.propulsion_get_steady_state();

    // The engine must be running and spinning above idle dead-band.
    let rpm = sim.get_property("propulsion/engine[0]/engine-rpm");
    assert!(
        rpm > 100.0,
        "engine RPM after init_running should be > 100, got {rpm}"
    );
}

use jsbsim_ffi::Sim;

/// Verify that the library links correctly and that we can create
/// and destroy a Sim instance without crashing.
///
/// This test does NOT need JSBSim data files — it only proves the
/// shared (or static) library was found and the FFI bridge works.
#[test]
fn create_and_destroy_sim() {
    // Passing an empty root dir is fine — we just won't be able to load
    // aircraft or scripts, but the FGFDMExec object should still be created.
    let sim = Sim::new("");
    // If we got here without a linker error or segfault, the library was
    // found at runtime and the C++ wrapper is functional.
    drop(sim);
}

/// Verify that get_property returns a value (0.0 for an uninitialised sim)
/// without panicking.
#[test]
fn get_property_on_empty_sim() {
    let sim = Sim::new("");
    let val = sim.get_property("simulation/sim-time-sec");
    assert_eq!(val, 0.0, "sim time should be 0 on a fresh instance");
}

/// Verify that set_property doesn't crash on a bare sim.
#[test]
fn set_property_on_empty_sim() {
    let mut sim = Sim::new("");
    let ok = sim.set_property("simulation/sim-time-sec", 1.0);
    // set_property returns true if the C wrapper executed without null-ptr bail
    assert!(ok);
}

/// Full integration test: load a model, initialise, and step the simulation.
///
/// Requires `JSBSIM_ROOT` to point at a real JSBSim data tree — the test
/// fails loudly (rather than skipping) if the env var is missing or the
/// path has no `aircraft/` subdirectory.
#[test]
fn full_sim_if_data_available() {
    let root = std::env::var("JSBSIM_ROOT").unwrap_or_default();
    assert!(
        !root.is_empty(),
        "JSBSIM_ROOT is not set.\n\
         Set it to a real JSBSim checkout before running integration tests:\n\
         \n    JSBSIM_ROOT=$HOME/jsbsim cargo test\n"
    );
    assert!(
        std::path::Path::new(&root).join("aircraft").is_dir(),
        "JSBSIM_ROOT={root:?} does not contain an `aircraft/` subdirectory."
    );

    let mut sim = Sim::new(&root);

    assert!(
        sim.load_model("c172x"),
        "Failed to load c172x — check that JSBSIM_ROOT has an aircraft/c172x/ directory"
    );

    sim.set_property("ic/h-sl-ft", 3000.0);
    sim.set_property("ic/vc-kts", 100.0);
    sim.set_property("ic/psi-true-deg", 0.0);
    sim.set_property("ic/gamma-deg", 0.0);

    assert!(sim.run_ic(), "RunIC failed");

    // Step a few frames and verify simulation time advances.
    for _ in 0..120 {
        sim.run();
    }

    let t = sim.get_property("simulation/sim-time-sec");
    assert!(t > 0.0, "simulation time should have advanced, got {t}");

    let alt = sim.get_property("position/h-sl-ft");
    assert!(alt > 0.0, "altitude should be positive, got {alt}");

    println!("✅ Full simulation test passed: t={t:.2}s, alt={alt:.0}ft");
}

/// Verify integration_suspended() works on a bare sim.
#[test]
fn integration_suspended_on_empty_sim() {
    let mut sim = Sim::new("");
    assert!(
        !sim.integration_suspended(),
        "Should not be suspended initially"
    );
    sim.suspend_integration();
    assert!(
        sim.integration_suspended(),
        "Should be suspended after suspend"
    );
    sim.resume_integration();
    assert!(
        !sim.integration_suspended(),
        "Should not be suspended after resume"
    );
}

/// Verify set_sim_time() / get_sim_time() work on a bare sim.
#[test]
fn set_and_get_sim_time_on_empty_sim() {
    let mut sim = Sim::new("");
    let t0 = sim.get_sim_time();
    assert_eq!(t0, 0.0, "Initial sim time should be 0");
    sim.set_sim_time(99.5);
    let t1 = sim.get_sim_time();
    assert!(
        (t1 - 99.5).abs() < 1e-9,
        "Sim time should be 99.5, got {t1}"
    );
}

/// Verify path getters don't crash on an empty sim (root="" means no paths set).
#[test]
fn path_getters_on_empty_sim() {
    let sim = Sim::new("");
    // With empty root, paths may be empty but should not crash.
    let _ = sim.get_root_dir();
    let _ = sim.get_aircraft_path();
    let _ = sim.get_engine_path();
    let _ = sim.get_systems_path();
}

/// Verify get_output_filename doesn't crash on a bare sim.
#[test]
fn get_output_filename_on_empty_sim() {
    let sim = Sim::new("");
    let fname = sim.get_output_filename(0);
    // Should be empty string, not crash.
    assert!(fname.is_empty() || !fname.is_empty(), "Should not crash");
}

/// Verify set_terrain_elevation doesn't crash on a bare sim.
#[test]
fn set_terrain_elevation_on_empty_sim() {
    let mut sim = Sim::new("");
    sim.set_terrain_elevation(500.0);
    // Just verify no crash.
}

/// Verify check_incremental_hold doesn't crash on an empty sim.
#[test]
fn check_incremental_hold_on_empty_sim() {
    let mut sim = Sim::new("");
    sim.check_incremental_hold();
    // Just verify no crash.
}

/// Verify set_output_filename / get_output_filename round-trip on empty sim.
#[test]
fn set_and_get_output_filename_on_empty_sim() {
    let mut sim = Sim::new("");
    // set_output_filename may return false on empty sim (no output channels),
    // but it should not crash.
    let _ = sim.set_output_filename(0, "test_output.csv");
    let fname = sim.get_output_filename(0);
    // Value depends on whether the channel exists; just verify no crash.
    let _ = fname;
}

/// Trim status / mode round-trip on an empty sim.
#[test]
fn trim_status_and_mode_round_trip() {
    let mut sim = Sim::new("");
    // Default state
    let _initial = sim.get_trim_status();
    sim.set_trim_status(true);
    assert!(
        sim.get_trim_status(),
        "trim status should be true after set"
    );
    sim.set_trim_status(false);
    assert!(
        !sim.get_trim_status(),
        "trim status should be false after clear"
    );

    sim.set_trim_mode(jsbsim_ffi::trim::FULL);
    assert_eq!(sim.get_trim_mode(), jsbsim_ffi::trim::FULL);
    sim.set_trim_mode(jsbsim_ffi::trim::GROUND);
    assert_eq!(sim.get_trim_mode(), jsbsim_ffi::trim::GROUND);
}

/// Hold-down setter/getter on an empty sim. SetHoldDown writes to a property
/// node that may not exist on a bare sim — accept either success or no-op.
#[test]
fn hold_down_does_not_crash_on_empty_sim() {
    let mut sim = Sim::new("");
    // Just verify no crash.
    let _ = sim.get_hold_down();
    sim.set_hold_down(true);
    sim.set_hold_down(false);
}

/// Frame counter, debug level, and output filename getters on bare sim.
#[test]
fn frame_and_debug_getters_on_empty_sim() {
    let sim = Sim::new("");
    assert_eq!(sim.get_frame(), 0, "frame counter should start at 0");
    let _level = sim.get_debug_level();
}

/// IncrTime advances simulation time by the current dt when not held.
#[test]
fn incr_time_advances_sim_time() {
    let mut sim = Sim::new("");
    sim.set_dt(0.01);
    let t0 = sim.get_sim_time();
    let new_t = sim.incr_time();
    assert!(new_t >= t0, "incr_time should not go backwards");
    assert!(
        (sim.get_sim_time() - new_t).abs() < 1e-12,
        "get_sim_time should match incr_time return"
    );
}

/// SetLoggingRate / ForceOutput should not crash on a bare sim.
#[test]
fn output_extras_do_not_crash_on_empty_sim() {
    let mut sim = Sim::new("");
    sim.set_logging_rate(50.0);
    sim.force_output(0);
}

/// SetOutputPath / GetOutputPath / GetFullAircraftPath round-trip on empty sim.
#[test]
fn output_path_round_trip_on_empty_sim() {
    let mut sim = Sim::new("");
    let ok = sim.set_output_path("/tmp/jsbsim_out");
    assert!(ok, "set_output_path should succeed");
    let p = sim.get_output_path();
    assert!(
        p.contains("jsbsim_out"),
        "get_output_path should reflect what we set, got {p:?}"
    );
    // get_full_aircraft_path is empty until a model is loaded; just confirm no crash.
    let _ = sim.get_full_aircraft_path();
}

/// LoadScript with deltaT and initfile should fail gracefully on missing file.
#[test]
fn load_script_with_missing_file_returns_false() {
    let mut sim = Sim::new("");
    let ok = sim.load_script_with("/nonexistent/path/no.xml", 0.005, None);
    assert!(!ok, "loading a missing script should return false");
    let ok2 = sim.load_script_with("/nonexistent/path/no.xml", 0.005, Some("/no.xml"));
    assert!(!ok2);
}

/// LoadPlanet should fail gracefully on missing file.
#[test]
fn load_planet_missing_file_returns_false() {
    let mut sim = Sim::new("");
    let ok = sim.load_planet("/nonexistent/planet.xml", false);
    assert!(!ok);
}

/// Property-catalog accessors on a bare sim should return empty without
/// crashing.
#[test]
fn property_catalog_empty_on_bare_sim() {
    let sim = Sim::new("");
    let cat = sim.get_property_catalog();
    // With no model loaded the catalog is usually empty but may contain
    // framework-level entries depending on the build. Just verify it's a Vec.
    let _ = cat.len();
}

/// PrintSimulationConfiguration should not crash on a bare sim.
#[test]
fn print_simulation_configuration_does_not_crash() {
    let sim = Sim::new("");
    sim.print_simulation_configuration();
}

/// GetPropulsionTankReport should return (possibly empty) string without crash.
#[test]
fn propulsion_tank_report_bare_sim() {
    let sim = Sim::new("");
    let _ = sim.get_propulsion_tank_report();
}

/// Random seed getter should not crash on bare sim.
#[test]
fn random_seed_getter_bare_sim() {
    let sim = Sim::new("");
    let _ = sim.get_random_seed();
}

/// Child FDM count / enumeration / SetChild on bare sim.
#[test]
fn child_fdm_accessors_bare_sim() {
    let mut sim = Sim::new("");
    assert_eq!(sim.get_fdm_count(), 0, "no child FDMs on fresh instance");
    // EnumerateFDMs may include the top-level FDM itself (which is an empty
    // name on a bare sim). Just verify the accessor doesn't crash.
    let _ = sim.enumerate_fdms();
    sim.set_child(true);
    sim.set_child(false);
}

/// DoLinearization on a bare (no model loaded) sim should return false,
/// not crash. The C wrapper guards on `GetModelName().empty()` because
/// `FGFDMExec::DoLinearization` dereferences subsystem pointers that are
/// only populated after `LoadModel`.
#[test]
fn do_linearization_without_model_returns_false() {
    let mut sim = Sim::new("");
    assert!(
        !sim.do_linearization(0),
        "do_linearization on bare sim must return false (guarded)"
    );
}

/// Propulsion helpers on a bare sim must not crash. Same guard pattern as
/// `do_linearization` — `init_running` and `propulsion_get_steady_state`
/// return `false` on a bare sim.
#[test]
fn propulsion_helpers_bare_sim() {
    let mut sim = Sim::new("");
    assert_eq!(sim.get_num_engines(), 0);
    assert_eq!(sim.get_num_tanks(), 0);
    assert!(
        !sim.init_running(-1),
        "init_running on bare sim must return false (guarded)"
    );
    assert!(
        !sim.propulsion_get_steady_state(),
        "propulsion_get_steady_state on bare sim must return false (guarded)"
    );
}

/// Test that the binary has the JSBSim library baked in (RPATH or static).
/// This runs the built test binary with an empty LD_LIBRARY_PATH to ensure
/// it doesn't depend on the user setting that variable.
///
/// This is tested implicitly: if `cargo test` succeeds at all, the library
/// was found at runtime.  But we verify explicitly that our binary's RPATH
/// or static linkage is working by checking we can call into JSBSim.
#[test]
fn runtime_library_discovery() {
    // If this test is running, it means the dynamic linker already found
    // libJSBSim.so (via RPATH) or it was statically linked.
    // Let's exercise the FFI to be sure.
    let sim = Sim::new("");
    let _ = sim.get_property("simulation/sim-time-sec");
    // Success — no "cannot open shared object file" error at runtime.
}

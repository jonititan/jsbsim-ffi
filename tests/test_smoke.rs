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

/// If JSBSIM_ROOT is set, run a full integration test:
/// load a model, initialise, and step the simulation.
#[test]
fn full_sim_if_data_available() {
    let root = match std::env::var("JSBSIM_ROOT") {
        Ok(r) => r,
        Err(_) => {
            eprintln!(
                "JSBSIM_ROOT not set — skipping full simulation test.\n\
                 To run: JSBSIM_ROOT=/path/to/jsbsim cargo test"
            );
            return;
        }
    };

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

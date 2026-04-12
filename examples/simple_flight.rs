//! Rust port of JSBSim's `examples/python/SimpleFlight.py`.
//!
//! Loads a Cessna 172x, sets initial conditions for level flight at 5000 ft,
//! runs 600 seconds of simulation, and prints altitude/airspeed every 10
//! seconds.
//!
//! Usage:
//!   JSBSIM_ROOT=/path/to/jsbsim cargo run --example simple_flight

use jsbsim_ffi::Sim;

#[path = "common/mod.rs"]
mod common;

fn main() {
    let root = common::jsbsim_root_or_exit("simple_flight");

    let mut sim = Sim::new(&root);

    // Load the Cessna 172x model.
    if !sim.load_model("c172x") {
        eprintln!("Failed to load 'c172x' model.");
        std::process::exit(1);
    }

    // ── Initial conditions ──────────────────────────────────────────
    sim.set_property("ic/h-sl-ft", 5000.0);
    sim.set_property("ic/vc-kts", 100.0);
    sim.set_property("ic/gamma-deg", 0.0);
    sim.set_property("ic/psi-true-deg", 0.0);

    if !sim.run_ic() {
        eprintln!("RunIC failed.");
        std::process::exit(1);
    }

    // ── Simulation loop ─────────────────────────────────────────────
    let dt = sim.get_dt();
    let end_time = 600.0; // 10 minutes
    let print_interval = 10.0; // print every 10 seconds
    let mut next_print = 0.0;

    println!(
        "{:<10} {:<12} {:<12} {:<12}",
        "Time(s)", "Alt(ft)", "Vc(kts)", "Gamma(deg)"
    );
    println!("{}", "-".repeat(48));

    while sim.get_sim_time() <= end_time {
        if sim.get_sim_time() >= next_print {
            let t = sim.get_sim_time();
            let alt = sim.get_property("position/h-sl-ft");
            let vc = sim.get_property("velocities/vc-kts");
            let gamma = sim.get_property("flight-path/gamma-deg");
            println!("{:<10.1} {:<12.1} {:<12.1} {:<12.3}", t, alt, vc, gamma);
            next_print += print_interval;
        }
        if !sim.run() {
            break;
        }
    }

    println!(
        "\nSimulation complete.  Final time: {:.1}s",
        sim.get_sim_time()
    );
    println!("dt = {dt:.4}s  ({:.0} steps)", sim.get_sim_time() / dt);
}

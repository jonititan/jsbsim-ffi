//! Rust port of JSBSim's `examples/python/script_example.py`.
//!
//! Loads and runs a JSBSim script XML file, printing simulation state
//! every second.  If no script is specified, defaults to `scripts/c1721.xml`.
//!
//! Usage:
//!   JSBSIM_ROOT=/path/to/jsbsim cargo run --example script_example
//!   JSBSIM_ROOT=/path/to/jsbsim cargo run --example script_example -- scripts/ball.xml

use jsbsim_ffi::Sim;

fn main() {
    let root = std::env::var("JSBSIM_ROOT").unwrap_or_else(|_| {
        eprintln!("Set JSBSIM_ROOT to the JSBSim data directory.");
        std::process::exit(1);
    });

    // Optional: script path from command-line argument.
    let args: Vec<String> = std::env::args().collect();
    let script_path = if args.len() > 1 {
        args[1].clone()
    } else {
        format!("{}/scripts/c1721.xml", root)
    };

    let mut sim = Sim::new(&root);

    println!("Loading script: {script_path}");
    if !sim.load_script(&script_path) {
        eprintln!("Failed to load script '{script_path}'.");
        std::process::exit(1);
    }

    if !sim.run_ic() {
        eprintln!("RunIC failed.");
        std::process::exit(1);
    }

    println!("JSBSim version: {}", Sim::get_version());
    println!("dt = {:.4}s", sim.get_dt());
    println!();

    // ── Simulation loop ─────────────────────────────────────────────
    let print_interval = 1.0;
    let mut next_print = 0.0;

    println!(
        "{:<10} {:<12} {:<12} {:<12} {:<12}",
        "Time(s)", "Alt(ft)", "Vc(kts)", "Lat(deg)", "Lon(deg)"
    );
    println!("{}", "-".repeat(60));

    loop {
        let t = sim.get_sim_time();

        if t >= next_print {
            let alt = sim.get_property("position/h-sl-ft");
            let vc = sim.get_property("velocities/vc-kts");
            let lat = sim.get_property("position/lat-gc-deg");
            let lon = sim.get_property("position/long-gc-deg");
            println!(
                "{:<10.2} {:<12.1} {:<12.1} {:<12.6} {:<12.6}",
                t, alt, vc, lat, lon
            );
            next_print += print_interval;
        }

        if !sim.run() {
            break;
        }
    }

    println!("\nScript finished at t = {:.2}s", sim.get_sim_time());
}

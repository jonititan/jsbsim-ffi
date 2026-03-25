//! Rust port of JSBSim's `examples/python/cannonball.py`.
//!
//! Simulates a cannonball launched at a given velocity and angle using
//! the JSBSim "ball" model.  Prints the trajectory until the ball
//! returns to ground level, then reports the range and max altitude.
//!
//! Usage:
//!   JSBSIM_ROOT=/path/to/jsbsim cargo run --example cannonball

use jsbsim_ffi::Sim;

fn main() {
    let root = std::env::var("JSBSIM_ROOT").unwrap_or_else(|_| {
        eprintln!("Set JSBSIM_ROOT to the JSBSim data directory.");
        std::process::exit(1);
    });

    let mut sim = Sim::new(&root);

    if !sim.load_model("ball") {
        eprintln!("Failed to load 'ball' model.");
        std::process::exit(1);
    }

    // ── Launch parameters ───────────────────────────────────────────
    let launch_speed_kts = 2000.0; // knots
    let launch_angle_deg = 45.0; // degrees above horizon
    let launch_alt_ft = 10.0; // just above ground
    let launch_heading_deg = 0.0; // north

    sim.set_property("ic/h-sl-ft", launch_alt_ft);
    sim.set_property("ic/vc-kts", launch_speed_kts);
    sim.set_property("ic/gamma-deg", launch_angle_deg);
    sim.set_property("ic/psi-true-deg", launch_heading_deg);

    if !sim.run_ic() {
        eprintln!("RunIC failed.");
        std::process::exit(1);
    }

    // Use a small timestep for accuracy.
    sim.set_dt(0.01);

    println!("Cannonball launched at {launch_speed_kts} kts, {launch_angle_deg}° above horizon");
    println!("JSBSim version: {}", Sim::get_version());
    println!();

    println!(
        "{:<10} {:<14} {:<14} {:<14}",
        "Time(s)", "Alt(ft)", "Dist(nm)", "Speed(kts)"
    );
    println!("{}", "-".repeat(54));

    let mut max_alt = 0.0_f64;
    let print_interval = 1.0;
    let mut next_print = 0.0;

    loop {
        let t = sim.get_sim_time();
        let alt = sim.get_property("position/h-sl-ft");
        let vc = sim.get_property("velocities/vc-kts");
        // Ground distance approximated via lat/lon (assuming flat earth near origin).
        let lat = sim.get_property("position/lat-gc-deg");
        let lon = sim.get_property("position/long-gc-deg");
        // Rough distance in nautical miles from origin (0,0).
        let dist_nm = ((lat * 60.0).powi(2) + (lon * 60.0).powi(2)).sqrt();

        if alt > max_alt {
            max_alt = alt;
        }

        if t >= next_print {
            println!(
                "{:<10.2} {:<14.1} {:<14.2} {:<14.1}",
                t, alt, dist_nm, vc
            );
            next_print += print_interval;
        }

        // Stop when ball returns to ground (after it has been airborne).
        if t > 1.0 && alt <= 0.0 {
            println!(
                "{:<10.2} {:<14.1} {:<14.2} {:<14.1}  *** IMPACT ***",
                t, alt, dist_nm, vc
            );
            break;
        }

        if !sim.run() {
            break;
        }

        // Safety: stop after 600 seconds.
        if t > 600.0 {
            println!("Timeout — simulation stopped at {t:.1}s");
            break;
        }
    }

    println!("\n--- Results ---");
    println!("Flight time : {:.2} s", sim.get_sim_time());
    println!("Max altitude: {max_alt:.0} ft ({:.0} m)", max_alt * 0.3048);
    let final_lat = sim.get_property("position/lat-gc-deg");
    let final_lon = sim.get_property("position/long-gc-deg");
    let range_nm = ((final_lat * 60.0).powi(2) + (final_lon * 60.0).powi(2)).sqrt();
    println!("Range       : {range_nm:.1} nm ({:.0} m)", range_nm * 1852.0);
}
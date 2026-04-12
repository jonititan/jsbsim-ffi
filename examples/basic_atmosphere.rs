//! Rust port of JSBSim's `examples/python/Basic_Atmosphere.py`.
//!
//! Exercises the standard atmosphere model by stepping through altitudes
//! from sea level to 100,000 ft and printing temperature, pressure, and
//! density at each level.
//!
//! Usage:
//!   JSBSIM_ROOT=/path/to/jsbsim cargo run --example basic_atmosphere

use jsbsim_ffi::Sim;

fn main() {
    let root = std::env::var("JSBSIM_ROOT").unwrap_or_else(|_| {
        eprintln!("Set JSBSIM_ROOT to the JSBSim data directory.");
        std::process::exit(1);
    });

    let mut sim = Sim::new(&root);

    // Load a minimal model — "ball" is lightweight.
    if !sim.load_model("ball") {
        eprintln!("Failed to load 'ball' model.");
        std::process::exit(1);
    }

    println!(
        "{:<12} {:<14} {:<14} {:<14} {:<14}",
        "Alt (ft)", "T (°R)", "T (°F)", "P (psf)", "ρ (sl/ft³)"
    );
    println!("{}", "-".repeat(70));

    // Step through altitudes from 0 to 100,000 ft in 5,000 ft increments.
    let mut alt = 0.0_f64;
    while alt <= 100_000.0 {
        sim.set_property("ic/h-sl-ft", alt);
        sim.set_property("ic/vc-kts", 0.0);
        sim.set_property("ic/gamma-deg", 0.0);
        sim.run_ic();

        let temp_r = sim.get_property("atmosphere/T-R");
        let temp_f = temp_r - 459.67; // Rankine → Fahrenheit
        let p_psf = sim.get_property("atmosphere/P-psf");
        let rho = sim.get_property("atmosphere/rho-slugs_ft3");

        println!(
            "{:<12.0} {:<14.2} {:<14.2} {:<14.2} {:<14.6}",
            alt, temp_r, temp_f, p_psf, rho
        );

        alt += 5_000.0;
    }

    // ── Standard atmosphere layer boundaries ────────────────────────
    println!("\n--- Key altitude checks ---");
    let check_alts = [
        (0.0, "Sea level"),
        (36_089.0, "Tropopause"),
        (65_617.0, "~20 km"),
        (82_021.0, "~25 km"),
    ];

    for (h, label) in &check_alts {
        sim.set_property("ic/h-sl-ft", *h);
        sim.set_property("ic/vc-kts", 0.0);
        sim.set_property("ic/gamma-deg", 0.0);
        sim.run_ic();

        let temp_r = sim.get_property("atmosphere/T-R");
        let p_psf = sim.get_property("atmosphere/P-psf");
        let rho = sim.get_property("atmosphere/rho-slugs_ft3");
        let a_fps = sim.get_property("atmosphere/a-fps"); // speed of sound

        println!(
            "{label:<16} h={h:>8.0} ft  T={temp_r:>7.2} °R  P={p_psf:>8.2} psf  ρ={rho:.6} sl/ft³  a={a_fps:.1} ft/s"
        );
    }
}

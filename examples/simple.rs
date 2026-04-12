use jsbsim_ffi::Sim;

#[path = "common/mod.rs"]
mod common;

fn main() {
    // The JSBSim data root must contain aircraft/, engine/, systems/, scripts/.
    // Set JSBSIM_ROOT to point to your JSBSim source checkout or data install.
    let root = common::jsbsim_root_or_exit("simple");

    let mut sim = Sim::new(&root);

    if !sim.load_script("scripts/c1723.xml") {
        eprintln!("Failed to load script!");
        return;
    }

    if !sim.run_ic() {
        eprintln!("Failed to run initial conditions!");
        return;
    }

    println!("✅ JSBSim running (Cessna 172)...");

    for step in 0..2000 {
        sim.run();

        if step % 400 == 0 {
            let t = sim.get_property("simulation/sim-time-sec");
            let alt = sim.get_property("position/h-agl-ft");
            let ias = sim.get_property("velocities/vc-kts");

            println!(
                "t={:.1}s → Altitude: {:.0} ft | IAS: {:.1} kts",
                t, alt, ias
            );
        }
    }
}

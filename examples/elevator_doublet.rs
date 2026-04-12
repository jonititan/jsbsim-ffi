//! Trim a Bombardier Global 5000, then drive an elevator doublet and
//! record angle of attack vs time.
//!
//! Port of `examples/python/test_pathsim_01_Trim_Elevator_Doublet.ipynb`:
//!   <https://github.com/JSBSim-Team/jsbsim/blob/master/examples/python/test_pathsim_01_Trim_Elevator_Doublet.ipynb>
//!
//! The Python notebook uses [PathSim](https://pathsim.org/) blocks to
//! drive the input.  This Rust port uses a plain step loop because we
//! don't depend on PathSim — the doublet is hard-coded as a piecewise
//! constant function of time.
//!
//! Doublet schedule (added to the trimmed elevator command):
//!
//! | t(s)        | Δ elevator (norm) |
//! | ----------- | ----------------- |
//! | 0  ..< 10   | 0                 |
//! | 10 ..< 11   | -0.1              |
//! | 11 ..< 12   |  0                |
//! | 12 ..< 13   | +0.1              |
//! | 13 ..< 40   |  0                |
//!
//! Run with:
//!
//! ```sh
//! JSBSIM_ROOT=/path/to/jsbsim cargo run --example elevator_doublet
//! ```

use jsbsim_ffi::{trim, Sim};
use plotters::prelude::*;

const AIRCRAFT: &str = "global5000";
const PLOT_PATH: &str = "elevator_doublet.svg";
const FUEL_MAX_LBS: f64 = 8097.63;
const PAYLOAD_LBS: f64 = 15172.0 / 2.0;
const ALTITUDE_FT: f64 = 15_000.0;
const CAS_KTS: f64 = 250.0;

const RUN_TIME_S: f64 = 40.0;
const PRINT_INTERVAL_S: f64 = 0.5;

fn doublet_offset(t: f64) -> f64 {
    if (10.0..11.0).contains(&t) {
        -0.1
    } else if (12.0..13.0).contains(&t) {
        0.1
    } else {
        0.0
    }
}

fn main() {
    let root = std::env::var("JSBSIM_ROOT")
        .expect("JSBSIM_ROOT must be set to a JSBSim data tree (with aircraft/, scripts/, …)");

    let mut sim = Sim::new(&root);
    sim.set_debug_level(0);
    assert!(sim.load_model(AIRCRAFT), "failed to load `{AIRCRAFT}`");
    sim.set_property("propulsion/set-running", -1.0);

    let fuel_per_tank = FUEL_MAX_LBS / 2.0;

    // Initial conditions
    sim.set_property("ic/h-sl-ft", ALTITUDE_FT);
    sim.set_property("ic/vc-kts", CAS_KTS);
    sim.set_property("ic/gamma-deg", 0.0);
    sim.set_property("propulsion/tank[0]/contents-lbs", fuel_per_tank);
    sim.set_property("propulsion/tank[1]/contents-lbs", fuel_per_tank);
    sim.set_property("propulsion/tank[2]/contents-lbs", fuel_per_tank);
    sim.set_property("inertia/pointmass-weight-lbs[0]", PAYLOAD_LBS);

    assert!(sim.run_ic());
    sim.run();

    if !sim.do_trim(trim::FULL) {
        eprintln!("Trim failed — continuing doublet from an untrimmed state.");
    }
    sim.run();

    let trim_elev_norm = sim.get_property("fcs/elevator-pos-norm");
    let trim_alpha = sim.get_property("aero/alpha-deg");
    let trim_cas = sim.get_property("velocities/vc-kts");

    println!("Trim — {AIRCRAFT}");
    println!("  Altitude       : {ALTITUDE_FT:.0} ft");
    println!("  CAS            : {trim_cas:.2} kt");
    println!("  Trim AoA       : {trim_alpha:.4}°");
    println!("  Trim elev (n)  : {trim_elev_norm:.4}");
    println!();
    println!("Doublet schedule: -0.1 norm at t∈[10,11), +0.1 norm at t∈[12,13)");
    println!();
    println!("    t(s)   elev(norm)   AoA(deg)    pitch(deg)");
    println!("  ------   ----------   --------    ----------");

    let dt = sim.get_dt();
    let total_steps = (RUN_TIME_S / dt).round() as u64;
    let print_every = (PRINT_INTERVAL_S / dt).round() as u64;

    let mut times: Vec<f64> = Vec::with_capacity(total_steps as usize);
    let mut cmds: Vec<f64> = Vec::with_capacity(total_steps as usize);
    let mut alphas: Vec<f64> = Vec::with_capacity(total_steps as usize);

    for step in 0..total_steps {
        let t = sim.get_sim_time();
        let cmd = trim_elev_norm + doublet_offset(t);
        sim.set_property("fcs/elevator-cmd-norm", cmd);
        sim.run();
        let alpha = sim.get_property("aero/alpha-deg");

        times.push(t);
        cmds.push(cmd);
        alphas.push(alpha);

        if step.is_multiple_of(print_every) {
            let pitch = sim.get_property("attitude/theta-deg");
            println!("  {t:6.2}   {cmd:10.4}   {alpha:8.4}    {pitch:10.4}");
        }
    }

    println!();
    println!("Final t = {:.2} s", sim.get_sim_time());

    if let Err(e) = render_plot(&times, &cmds, &alphas) {
        eprintln!("plot failed: {e}");
    } else {
        println!("Wrote plot → {PLOT_PATH}");
    }
}

/// Two stacked panels matching the Python notebook: top is the elevator
/// command in % (×100), bottom is the AoA response in degrees, both vs
/// time.  The doublet pulses at t=[10,11) and t=[12,13) are visible as
/// vertical excursions in the top panel and damped oscillations below.
fn render_plot(
    times: &[f64],
    cmds: &[f64],
    alphas: &[f64],
) -> Result<(), Box<dyn std::error::Error>> {
    if times.is_empty() {
        return Err("no samples".into());
    }
    let root = SVGBackend::new(PLOT_PATH, (1000, 700)).into_drawing_area();
    root.fill(&WHITE)?;
    let panels = root.split_evenly((2, 1));

    let t_min = *times.first().unwrap();
    let t_max = *times.last().unwrap();

    // Panel 1: elevator command (×100)
    {
        let pct: Vec<(f64, f64)> = times
            .iter()
            .copied()
            .zip(cmds.iter().map(|c| 100.0 * c))
            .collect();
        let y_min = pct.iter().map(|p| p.1).fold(f64::INFINITY, f64::min) - 5.0;
        let y_max = pct.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max) + 5.0;
        let mut chart = ChartBuilder::on(&panels[0])
            .caption(format!("Elevator Doublet — {AIRCRAFT}"), ("sans-serif", 18))
            .margin(15)
            .x_label_area_size(35)
            .y_label_area_size(60)
            .build_cartesian_2d(t_min..t_max, y_min..y_max)?;
        chart
            .configure_mesh()
            .x_desc("Time (s)")
            .y_desc("Elevator command (%)")
            .draw()?;
        chart.draw_series(LineSeries::new(pct, BLUE.stroke_width(2)))?;
    }

    // Panel 2: AoA
    {
        let y_min = alphas.iter().copied().fold(f64::INFINITY, f64::min) - 0.5;
        let y_max = alphas.iter().copied().fold(f64::NEG_INFINITY, f64::max) + 0.5;
        let mut chart = ChartBuilder::on(&panels[1])
            .margin(15)
            .x_label_area_size(35)
            .y_label_area_size(60)
            .build_cartesian_2d(t_min..t_max, y_min..y_max)?;
        chart
            .configure_mesh()
            .x_desc("Time (s)")
            .y_desc("Angle of attack (deg)")
            .draw()?;
        chart.draw_series(LineSeries::new(
            times.iter().copied().zip(alphas.iter().copied()),
            RED.stroke_width(2),
        ))?;
    }

    root.present()?;
    Ok(())
}

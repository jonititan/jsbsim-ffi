//! Rudder kick / steady-heading sideslip test on a Boeing 737.
//!
//! Port of `examples/python/Rudder Kick.ipynb` from the JSBSim repository:
//!   <https://github.com/JSBSim-Team/jsbsim/blob/master/examples/python/Rudder%20Kick.ipynb>
//!
//! Trims the 737 in level flight at 200 KCAS / 1000 ft, then ramps both
//! aileron and rudder to their maxima over 3 s and runs for 20 s,
//! recording sideslip (β), bank angle (φ), and inceptor positions every
//! 0.5 s.
//!
//! Run with:
//!
//! ```sh
//! JSBSIM_ROOT=/path/to/jsbsim cargo run --example rudder_kick
//! ```

use jsbsim_ffi::{trim, Sim};
use plotters::prelude::*;

#[path = "common/mod.rs"]
mod common;

const AIRCRAFT: &str = "737";
const PLOT_PATH: &str = "rudder_kick.svg";

const ALTITUDE_FT: f64 = 1000.0;
const CAS_KTS: f64 = 200.0;

const RISE_TIME_S: f64 = 3.0;
const RUN_PERIOD_S: f64 = 20.0;
const PRINT_INTERVAL_S: f64 = 0.5;

const AILERON_MAX: f64 = 1.0;
const RUDDER_MAX: f64 = 0.92;

fn main() {
    let root = common::jsbsim_root_or_exit("rudder_kick");

    let mut sim = Sim::new(&root);
    sim.set_debug_level(0);
    assert!(sim.load_model(AIRCRAFT), "failed to load `{AIRCRAFT}`");

    sim.set_property("propulsion/set-running", -1.0);

    // Widen alpha bounds for the trim solver.
    sim.set_property("aero/alpha-max-rad", 12.0_f64.to_radians());
    sim.set_property("aero/alpha-min-rad", (-4.0_f64).to_radians());

    let dt = sim.get_dt();
    let steps_per_print = (PRINT_INTERVAL_S / dt).round() as u64;
    let total_steps = (RUN_PERIOD_S / dt).round() as u64;

    // Per-step inceptor increments to reach max over RISE_TIME_S.
    let d_aileron = AILERON_MAX / (RISE_TIME_S / dt);
    let d_rudder = RUDDER_MAX / (RISE_TIME_S / dt);

    // Initial conditions
    sim.set_property("ic/h-sl-ft", ALTITUDE_FT);
    sim.set_property("ic/vc-kts", CAS_KTS);
    sim.set_property("ic/gamma-deg", 0.0);
    sim.set_property("ic/beta-deg", 0.0);
    assert!(sim.run_ic());

    if !sim.do_trim(trim::FULL) {
        eprintln!("Trim failed — continuing rudder kick from an untrimmed state.");
    }

    println!("Rudder kick — {AIRCRAFT}");
    println!("  IC : altitude {ALTITUDE_FT:.0} ft, CAS {CAS_KTS:.0} kts");
    println!("  dt : {dt:.4} s   ({total_steps} steps over {RUN_PERIOD_S:.0} s)");
    println!();
    println!("    t(s)    aileron   rudder    beta(deg)   bank(deg)");
    println!("  ------   --------  --------  ----------  ----------");

    // Per-step recordings for the plot.
    let mut times: Vec<f64> = Vec::with_capacity(total_steps as usize);
    let mut betas: Vec<f64> = Vec::with_capacity(total_steps as usize);
    let mut bank_angles: Vec<f64> = Vec::with_capacity(total_steps as usize);
    let mut aileron_log: Vec<f64> = Vec::with_capacity(total_steps as usize);
    let mut rudder_log: Vec<f64> = Vec::with_capacity(total_steps as usize);

    for step in 0..total_steps {
        sim.run();

        let t = sim.get_sim_time();
        let beta = sim.get_property("aero/beta-deg");
        let bank = sim.get_property("attitude/phi-deg");
        let aileron = sim.get_property("fcs/aileron-cmd-norm");
        let rudder = sim.get_property("fcs/rudder-cmd-norm");

        // Record every step for the plot.
        times.push(t);
        betas.push(beta);
        bank_angles.push(bank);
        aileron_log.push(aileron);
        rudder_log.push(rudder);

        if step.is_multiple_of(steps_per_print) {
            println!("  {t:6.2}   {aileron:7.4}   {rudder:7.4}   {beta:9.4}   {bank:9.4}");
        }

        // Ramp the inceptors toward their maxima.
        let aileron_cmd = sim.get_property("fcs/aileron-cmd-norm");
        if aileron_cmd < AILERON_MAX {
            sim.set_property("fcs/aileron-cmd-norm", aileron_cmd + d_aileron);
        }
        let rudder_cmd = sim.get_property("fcs/rudder-cmd-norm");
        if rudder_cmd < RUDDER_MAX {
            sim.set_property("fcs/rudder-cmd-norm", rudder_cmd + d_rudder);
        }
    }

    println!();
    println!("Final state: t={:.2} s", sim.get_sim_time());

    if let Err(e) = render_plot(&times, &betas, &bank_angles, &aileron_log, &rudder_log) {
        eprintln!("plot failed: {e}");
    } else {
        println!("Wrote plot → {PLOT_PATH}");
    }
}

/// Twin-axis time-history plot matching the Python notebook: β (deg) on the
/// left axis, inceptor positions on the right axis, time on x.
fn render_plot(
    times: &[f64],
    betas: &[f64],
    bank: &[f64],
    aileron: &[f64],
    rudder: &[f64],
) -> Result<(), Box<dyn std::error::Error>> {
    if times.is_empty() {
        return Err("no samples to plot".into());
    }
    let root = SVGBackend::new(PLOT_PATH, (1000, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let t_min = *times.first().unwrap();
    let t_max = *times.last().unwrap();
    let beta_min = betas
        .iter()
        .chain(bank.iter())
        .copied()
        .fold(f64::INFINITY, f64::min)
        - 2.0;
    let beta_max = betas
        .iter()
        .chain(bank.iter())
        .copied()
        .fold(f64::NEG_INFINITY, f64::max)
        + 2.0;

    let mut chart = ChartBuilder::on(&root)
        .caption(format!("Rudder kick — {AIRCRAFT}"), ("sans-serif", 20))
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .right_y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, beta_min..beta_max)?
        .set_secondary_coord(t_min..t_max, -0.1_f64..1.1);

    chart
        .configure_mesh()
        .x_desc("Time (s)")
        .y_desc("Angle (deg)")
        .draw()?;
    chart
        .configure_secondary_axes()
        .y_desc("Inceptor position (norm)")
        .draw()?;

    chart
        .draw_series(LineSeries::new(
            times.iter().copied().zip(betas.iter().copied()),
            RED.stroke_width(2),
        ))?
        .label("β (sideslip)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED.stroke_width(2)));

    chart
        .draw_series(LineSeries::new(
            times.iter().copied().zip(bank.iter().copied()),
            MAGENTA.stroke_width(2),
        ))?
        .label("φ (bank)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], MAGENTA.stroke_width(2)));

    chart
        .draw_secondary_series(LineSeries::new(
            times.iter().copied().zip(aileron.iter().copied()),
            BLUE.stroke_width(2),
        ))?
        .label("Aileron cmd")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE.stroke_width(2)));

    chart
        .draw_secondary_series(LineSeries::new(
            times.iter().copied().zip(rudder.iter().copied()),
            GREEN.stroke_width(2),
        ))?
        .label("Rudder cmd")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN.stroke_width(2)));

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::LowerRight)
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    root.present()?;
    Ok(())
}

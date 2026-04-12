//! Angle of Attack vs Calibrated Airspeed for a Bombardier Global 5000.
//!
//! Port of `examples/python/AoA vs CAS.ipynb` from the JSBSim repository:
//!   <https://github.com/JSBSim-Team/jsbsim/blob/master/examples/python/AoA%20vs%20CAS.ipynb>
//!
//! Sweeps calibrated airspeed in 10 kt steps from 90–550 kt at a fixed
//! altitude/weight/CG, trims the aircraft at each speed, and prints the
//! resulting angle of attack and elevator deflection.  The Python original
//! plots the same data with matplotlib; the Rust port prints a table.
//!
//! Run with:
//!
//! ```sh
//! JSBSIM_ROOT=/path/to/jsbsim cargo run --example aoa_vs_cas
//! ```

use jsbsim_ffi::{trim, Sim};
use plotters::prelude::*;

const AIRCRAFT: &str = "global5000";
const PLOT_PATH: &str = "aoa_vs_cas.svg";

// Global 5000 fuel tank size (lbm).
const FUEL_MAX_LBS: f64 = 8097.63;

// Mid payload (lbm).
const PAYLOAD_LBS: f64 = 15172.0 / 2.0;

// Trim altitude (ft).
const ALTITUDE_FT: f64 = 15000.0;

// Calibrated airspeed sweep (kts).
const CAS_MIN: i32 = 90;
const CAS_MAX: i32 = 550;
const CAS_STEP: i32 = 10;

fn main() {
    let root = std::env::var("JSBSIM_ROOT")
        .expect("JSBSIM_ROOT must be set to a JSBSim data tree (with aircraft/, scripts/, …)");

    let mut sim = Sim::new(&root);
    sim.set_debug_level(0);
    assert!(
        sim.load_model(AIRCRAFT),
        "failed to load aircraft `{AIRCRAFT}` — does {root}/aircraft/{AIRCRAFT}/ exist?"
    );

    // Warm-start every engine on the aircraft.
    sim.set_property("propulsion/set-running", -1.0);

    let fuel_per_tank = FUEL_MAX_LBS / 2.0;

    println!("Trim sweep — {AIRCRAFT}");
    println!("  Altitude   : {ALTITUDE_FT:.0} ft");
    println!("  Payload    : {PAYLOAD_LBS:.0} lb");
    println!("  Fuel/tank  : {fuel_per_tank:.0} lb (× 3 tanks)");
    println!("  CAS range  : {CAS_MIN}..{CAS_MAX} kt step {CAS_STEP}");
    println!();
    println!("  CAS(kt)   AoA(deg)   Elev(deg)   Elev(norm)");
    println!("  -------   --------   ---------   ----------");

    let mut trimmed = 0;
    let mut failed = 0;
    let mut series: Vec<(f64, f64, f64, f64)> = Vec::new(); // (cas, aoa, elev_deg, elev_norm)

    for cas in (CAS_MIN..CAS_MAX).step_by(CAS_STEP as usize) {
        sim.set_property("ic/h-sl-ft", ALTITUDE_FT);
        sim.set_property("ic/vc-kts", cas as f64);
        sim.set_property("ic/gamma-deg", 0.0);
        sim.set_property("propulsion/tank[0]/contents-lbs", fuel_per_tank);
        sim.set_property("propulsion/tank[1]/contents-lbs", fuel_per_tank);
        sim.set_property("propulsion/tank[2]/contents-lbs", fuel_per_tank);
        sim.set_property("inertia/pointmass-weight-lbs[0]", PAYLOAD_LBS);

        if !sim.run_ic() {
            eprintln!("  CAS={cas:3} kt: run_ic failed");
            failed += 1;
            continue;
        }
        sim.run();

        if !sim.do_trim(trim::FULL) {
            // Trim failure is expected at the edges of the envelope.
            failed += 1;
            continue;
        }

        let cas_actual = sim.get_property("velocities/vc-kts");
        let alpha_deg = sim.get_property("aero/alpha-deg");
        let elev_rad = sim.get_property("fcs/elevator-pos-rad");
        let elev_norm = sim.get_property("fcs/elevator-pos-norm");
        let elev_deg = elev_rad.to_degrees();

        println!("  {cas_actual:7.1}   {alpha_deg:8.3}   {elev_deg:9.3}   {elev_norm:10.4}");
        series.push((cas_actual, alpha_deg, elev_deg, elev_norm));
        trimmed += 1;
    }

    if let Err(e) = render_plot(&series) {
        eprintln!("plot failed: {e}");
    } else {
        println!();
        println!("Wrote plot → {PLOT_PATH}");
    }

    println!();
    println!(
        "Summary: trimmed {trimmed} / {} points ({failed} failures).",
        trimmed + failed
    );
}

/// Three stacked panels matching the Python notebook's matplotlib output:
/// AoA vs CAS, elevator deflection (deg) vs CAS, elevator deflection (norm)
/// vs CAS.  Written as SVG so we don't need any native font deps.
fn render_plot(series: &[(f64, f64, f64, f64)]) -> Result<(), Box<dyn std::error::Error>> {
    if series.is_empty() {
        return Err("no trim points to plot".into());
    }

    let root = SVGBackend::new(PLOT_PATH, (900, 900)).into_drawing_area();
    root.fill(&WHITE)?;
    let panels = root.split_evenly((3, 1));

    // X axis bounds (CAS)
    let cas_min = series.iter().map(|p| p.0).fold(f64::INFINITY, f64::min) - 5.0;
    let cas_max = series.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max) + 5.0;

    // Panel 1: AoA vs CAS
    {
        let mut chart = ChartBuilder::on(&panels[0])
            .caption(
                format!("Trimmed flight conditions — {} (FPA = 0°)", AIRCRAFT),
                ("sans-serif", 18),
            )
            .margin(10)
            .x_label_area_size(35)
            .y_label_area_size(55)
            .build_cartesian_2d(cas_min..cas_max, -5.0_f64..15.0)?;
        chart
            .configure_mesh()
            .x_desc("KCAS (kt)")
            .y_desc("Angle of Attack (deg)")
            .draw()?;
        chart.draw_series(LineSeries::new(
            series.iter().map(|p| (p.0, p.1)),
            BLUE.stroke_width(2),
        ))?;
        chart.draw_series(
            series
                .iter()
                .map(|p| Circle::new((p.0, p.1), 3, BLUE.filled())),
        )?;
    }

    // Panel 2: Elevator deflection (deg) vs CAS
    {
        let elev_min = series.iter().map(|p| p.2).fold(f64::INFINITY, f64::min) - 1.0;
        let elev_max = series.iter().map(|p| p.2).fold(f64::NEG_INFINITY, f64::max) + 1.0;
        let mut chart = ChartBuilder::on(&panels[1])
            .margin(10)
            .x_label_area_size(35)
            .y_label_area_size(55)
            .build_cartesian_2d(cas_min..cas_max, elev_min..elev_max)?;
        chart
            .configure_mesh()
            .x_desc("KCAS (kt)")
            .y_desc("Elevator deflection (deg)")
            .draw()?;
        chart.draw_series(LineSeries::new(
            series.iter().map(|p| (p.0, p.2)),
            RED.stroke_width(2),
        ))?;
        chart.draw_series(
            series
                .iter()
                .map(|p| Circle::new((p.0, p.2), 3, RED.filled())),
        )?;
    }

    // Panel 3: Normalized elevator deflection vs CAS
    {
        let elev_min = series.iter().map(|p| p.3).fold(f64::INFINITY, f64::min) - 0.05;
        let elev_max = series.iter().map(|p| p.3).fold(f64::NEG_INFINITY, f64::max) + 0.05;
        let mut chart = ChartBuilder::on(&panels[2])
            .margin(10)
            .x_label_area_size(35)
            .y_label_area_size(55)
            .build_cartesian_2d(cas_min..cas_max, elev_min..elev_max)?;
        chart
            .configure_mesh()
            .x_desc("KCAS (kt)")
            .y_desc("Elevator deflection (normalized)")
            .draw()?;
        chart.draw_series(LineSeries::new(
            series.iter().map(|p| (p.0, p.3)),
            GREEN.stroke_width(2),
        ))?;
        chart.draw_series(
            series
                .iter()
                .map(|p| Circle::new((p.0, p.3), 3, GREEN.filled())),
        )?;
    }

    root.present()?;
    Ok(())
}

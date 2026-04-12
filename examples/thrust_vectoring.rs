//! Pitch thrust-vector optimisation for a Boeing 737.
//!
//! Port of `examples/python/Thrust Vectoring Analysis.ipynb`:
//!   <https://github.com/JSBSim-Team/jsbsim/blob/master/examples/python/Thrust%20Vectoring%20Analysis.ipynb>
//!
//! Sweeps the pitch thrust-vector angle from 0–10° at two flight
//! conditions (cruise at 30 000 ft / Mach 0.8 and climb at 15 000 ft /
//! 300 KCAS / γ=3°), trimming at each angle.  Records the total thrust
//! and angle of attack, then reports the angle that minimises required
//! thrust — the optimal vector angle.
//!
//! Based on NASA TM-2000-209585 ("Optimal Pitch Thrust-Vector Angle and
//! Benefits for all Flight Regimes").
//!
//! Run with:
//!
//! ```sh
//! JSBSIM_ROOT=/path/to/jsbsim cargo run --example thrust_vectoring
//! ```

use jsbsim_ffi::{trim, Sim};
use plotters::prelude::*;

#[path = "common/mod.rs"]
mod common;

const AIRCRAFT: &str = "737";
const N_ANGLES: usize = 50;
const TV_MIN_DEG: f64 = 0.0;
const TV_MAX_DEG: f64 = 10.0;

#[derive(Clone, Copy)]
enum Speed {
    Mach(f64),
    Cas(f64),
}

struct Condition<'a> {
    title: &'a str,
    plot_filename: &'a str,
    altitude_ft: f64,
    speed: Speed,
    fpa_deg: f64,
}

fn main() {
    let root = common::jsbsim_root_or_exit("thrust_vectoring");

    let mut sim = Sim::new(&root);
    sim.set_debug_level(0);
    assert!(sim.load_model(AIRCRAFT), "failed to load `{AIRCRAFT}`");
    sim.set_property("propulsion/set-running", -1.0);

    let conditions = [
        Condition {
            title: "Cruise — 30 000 ft, Mach 0.8",
            plot_filename: "thrust_vectoring_cruise.svg",
            altitude_ft: 30_000.0,
            speed: Speed::Mach(0.8),
            fpa_deg: 0.0,
        },
        Condition {
            title: "Climb — 15 000 ft, 300 KCAS, γ = 3°",
            plot_filename: "thrust_vectoring_climb.svg",
            altitude_ft: 15_000.0,
            speed: Speed::Cas(300.0),
            fpa_deg: 3.0,
        },
    ];

    for cond in &conditions {
        thrust_vector_sweep(&mut sim, cond);
        println!();
    }
}

fn thrust_vector_sweep(sim: &mut Sim, cond: &Condition) {
    println!("=== {} ===", cond.title);
    println!("  TV(deg)   Thrust(lbf)   Alpha(deg)");
    println!("  -------   -----------   ----------");

    let mut min_thrust = f64::INFINITY;
    let mut min_angle = f64::NAN;
    let mut min_alpha = f64::NAN;
    let mut trimmed = 0;
    let mut samples: Vec<(f64, f64, f64)> = Vec::with_capacity(N_ANGLES + 1); // (tv, thrust, alpha)

    for i in 0..=N_ANGLES {
        let tv_angle = TV_MIN_DEG + (TV_MAX_DEG - TV_MIN_DEG) * (i as f64) / (N_ANGLES as f64);

        // Initial conditions
        sim.set_property("ic/h-sl-ft", cond.altitude_ft);
        match cond.speed {
            Speed::Mach(m) => sim.set_property("ic/mach", m),
            Speed::Cas(v) => sim.set_property("ic/vc-kts", v),
        };
        sim.set_property("ic/gamma-deg", cond.fpa_deg);

        if !sim.run_ic() {
            continue;
        }

        // Set the pitch thrust-vector angle on both engines.
        let tv_rad = tv_angle.to_radians();
        sim.set_property("propulsion/engine[0]/pitch-angle-rad", tv_rad);
        sim.set_property("propulsion/engine[1]/pitch-angle-rad", tv_rad);

        if !sim.do_trim(trim::FULL) {
            continue;
        }

        let thrust_one = sim.get_property("propulsion/engine[0]/thrust-lbs");
        let thrust_total = thrust_one * 2.0; // two engines
        let alpha = sim.get_property("aero/alpha-deg");

        // Print sparsely so the table stays readable.
        if i.is_multiple_of(5) {
            println!("  {tv_angle:7.2}   {thrust_total:11.1}   {alpha:10.4}");
        }

        samples.push((tv_angle, thrust_total, alpha));
        if thrust_total < min_thrust {
            min_thrust = thrust_total;
            min_angle = tv_angle;
            min_alpha = alpha;
        }
        trimmed += 1;
    }

    if trimmed == 0 {
        println!("  (no trim points succeeded)");
    } else {
        println!();
        println!(
            "  Optimum: TV = {min_angle:.2}°  →  thrust = {min_thrust:.1} lbf  (α = {min_alpha:.3}°)"
        );
        println!("  ({trimmed} / {} points trimmed)", N_ANGLES + 1);
        match render_plot(cond, &samples, min_angle, min_thrust) {
            Ok(()) => println!("  Wrote plot → {}", cond.plot_filename),
            Err(e) => eprintln!("  plot failed: {e}"),
        }
    }
}

/// Twin-axis plot: thrust (lbf) on the left, angle of attack (deg) on the
/// right, both vs the pitch thrust-vector angle.  A red dot marks the
/// minimum-thrust point — the optimum vector angle.
fn render_plot(
    cond: &Condition,
    samples: &[(f64, f64, f64)],
    min_angle: f64,
    min_thrust: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    if samples.is_empty() {
        return Err("no samples".into());
    }
    let root = SVGBackend::new(cond.plot_filename, (1000, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let tv_min = samples.first().unwrap().0;
    let tv_max = samples.last().unwrap().0;
    let thrust_lo = samples.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let thrust_hi = samples
        .iter()
        .map(|p| p.1)
        .fold(f64::NEG_INFINITY, f64::max);
    let thrust_pad = (thrust_hi - thrust_lo).max(1.0) * 0.1;
    let alpha_lo = samples.iter().map(|p| p.2).fold(f64::INFINITY, f64::min);
    let alpha_hi = samples
        .iter()
        .map(|p| p.2)
        .fold(f64::NEG_INFINITY, f64::max);
    let alpha_pad = (alpha_hi - alpha_lo).max(0.1) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(cond.title, ("sans-serif", 20))
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(70)
        .right_y_label_area_size(60)
        .build_cartesian_2d(
            tv_min..tv_max,
            (thrust_lo - thrust_pad)..(thrust_hi + thrust_pad),
        )?
        .set_secondary_coord(
            tv_min..tv_max,
            (alpha_lo - alpha_pad)..(alpha_hi + alpha_pad),
        );

    chart
        .configure_mesh()
        .x_desc("Thrust vector angle (deg)")
        .y_desc("Thrust (lbf)")
        .draw()?;
    chart
        .configure_secondary_axes()
        .y_desc("Angle of attack (deg)")
        .draw()?;

    chart
        .draw_series(LineSeries::new(
            samples.iter().map(|p| (p.0, p.1)),
            BLUE.stroke_width(2),
        ))?
        .label("Thrust")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE.stroke_width(2)));

    chart
        .draw_secondary_series(LineSeries::new(
            samples.iter().map(|p| (p.0, p.2)),
            GREEN.stroke_width(2),
        ))?
        .label("AoA")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN.stroke_width(2)));

    // Mark the optimum.
    chart
        .draw_series(std::iter::once(Circle::new(
            (min_angle, min_thrust),
            6,
            RED.filled(),
        )))?
        .label(format!("Min thrust @ {min_angle:.2}°"))
        .legend(|(x, y)| Circle::new((x + 10, y), 5, RED.filled()));

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    root.present()?;
    Ok(())
}

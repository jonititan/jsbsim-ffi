//! Trim envelope sweep for a Boeing 737.
//!
//! Port of `examples/python/Trim Envelope.ipynb`:
//!   <https://github.com/JSBSim-Team/jsbsim/blob/master/examples/python/Trim%20Envelope.ipynb>
//!
//! Sweeps calibrated airspeed (120–460 kt in 10 kt steps) and flight
//! path angle (-10° to +10° in 1° steps) at 15 000 ft and trims at each
//! point.  Reports the throttle and angle of attack required for level
//! trim across the envelope as an ASCII grid (rather than the matplotlib
//! heat-map of the Python original).
//!
//! Run with:
//!
//! ```sh
//! JSBSIM_ROOT=/path/to/jsbsim cargo run --example trim_envelope
//! ```

use jsbsim_ffi::{trim, Sim};
use plotters::prelude::*;

#[path = "common/mod.rs"]
mod common;

const AIRCRAFT: &str = "737";
const PLOT_PATH: &str = "trim_envelope.svg";
const ALTITUDE_FT: f64 = 15_000.0;

const SPEED_MIN: i32 = 120;
const SPEED_MAX: i32 = 460;
const SPEED_STEP: usize = 20; // print column every 20 kt to keep the grid wide enough

const GAMMA_MIN: i32 = -10;
const GAMMA_MAX: i32 = 10;

fn main() {
    let root = common::jsbsim_root_or_exit("trim_envelope");

    let mut sim = Sim::new(&root);
    sim.set_debug_level(0);
    assert!(sim.load_model(AIRCRAFT), "failed to load `{AIRCRAFT}`");
    sim.set_property("propulsion/set-running", -1.0);

    sim.set_property("aero/alpha-max-rad", 12.0_f64.to_radians());
    sim.set_property("aero/alpha-min-rad", (-4.0_f64).to_radians());

    println!("Trim envelope — {AIRCRAFT}");
    println!("  Altitude {ALTITUDE_FT:.0} ft");
    println!("  Speed    {SPEED_MIN}..{SPEED_MAX} kt CAS");
    println!("  γ        {GAMMA_MIN}..{GAMMA_MAX}°");
    println!();

    // Collect results: (speed, gamma) -> (throttle, alpha)
    let mut results: Vec<(i32, i32, f64, f64)> = Vec::new();
    let mut trimmed = 0;
    let mut failed = 0;

    for speed in (SPEED_MIN..SPEED_MAX).step_by(10) {
        for gamma in GAMMA_MIN..GAMMA_MAX {
            sim.set_property("ic/h-sl-ft", ALTITUDE_FT);
            sim.set_property("ic/vc-kts", speed as f64);
            sim.set_property("ic/gamma-deg", gamma as f64);

            if !sim.run_ic() {
                failed += 1;
                continue;
            }
            if sim.do_trim(trim::FULL) {
                let throttle = sim.get_property("fcs/throttle-cmd-norm[0]");
                let alpha = sim.get_property("aero/alpha-deg");
                results.push((speed, gamma, throttle, alpha));
                trimmed += 1;
            } else {
                failed += 1;
            }
        }
    }

    // Print throttle grid: rows are γ (top = high), columns are CAS.
    println!("Throttle [0..1]  (· = trim failed)");
    print_grid(&results, |r| r.2);
    println!();
    println!("Angle of attack (deg)  (· = trim failed)");
    print_grid(&results, |r| r.3);
    println!();
    println!(
        "Summary: trimmed {trimmed} / {} points ({failed} failures).",
        trimmed + failed
    );

    if let Err(e) = render_plot(&results) {
        eprintln!("plot failed: {e}");
    } else {
        println!("Wrote plot → {PLOT_PATH}");
    }
}

/// Two side-by-side scatter panels matching the Python notebook: throttle
/// (left) and AoA (right) vs (CAS, γ).  The colour of each marker encodes
/// the value via a viridis-like colour map.
fn render_plot(results: &[(i32, i32, f64, f64)]) -> Result<(), Box<dyn std::error::Error>> {
    if results.is_empty() {
        return Err("no trim points to plot".into());
    }
    let root = SVGBackend::new(PLOT_PATH, (1400, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    let panels = root.split_evenly((1, 2));

    let throttle_min = results.iter().map(|r| r.2).fold(f64::INFINITY, f64::min);
    let throttle_max = results
        .iter()
        .map(|r| r.2)
        .fold(f64::NEG_INFINITY, f64::max);
    let alpha_min = results.iter().map(|r| r.3).fold(f64::INFINITY, f64::min);
    let alpha_max = results
        .iter()
        .map(|r| r.3)
        .fold(f64::NEG_INFINITY, f64::max);

    draw_panel(
        &panels[0],
        results,
        |r| r.2,
        throttle_min,
        throttle_max,
        "Trim Envelope — Throttle",
        "Throttle [0..1]",
    )?;
    draw_panel(
        &panels[1],
        results,
        |r| r.3,
        alpha_min,
        alpha_max,
        "Trim Envelope — Angle of Attack",
        "AoA (deg)",
    )?;

    root.present()?;
    Ok(())
}

fn draw_panel<DB: DrawingBackend, F: Fn(&(i32, i32, f64, f64)) -> f64>(
    area: &DrawingArea<DB, plotters::coord::Shift>,
    results: &[(i32, i32, f64, f64)],
    field: F,
    val_min: f64,
    val_max: f64,
    title: &str,
    legend_label: &str,
) -> Result<(), Box<dyn std::error::Error>>
where
    DB::ErrorType: 'static,
{
    let x_min = (SPEED_MIN - 20) as f64;
    let x_max = (SPEED_MAX + 20) as f64;
    let y_min = (GAMMA_MIN * 2) as f64;
    let y_max = (GAMMA_MAX * 2) as f64;

    let mut chart = ChartBuilder::on(area)
        .caption(title, ("sans-serif", 18))
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
    chart
        .configure_mesh()
        .x_desc("CAS (kt)")
        .y_desc("γ (deg)")
        .draw()?;

    let span = (val_max - val_min).max(1e-9);
    chart.draw_series(results.iter().map(|r| {
        let v = field(r);
        let t = ((v - val_min) / span).clamp(0.0, 1.0);
        let color = viridis(t);
        Circle::new((r.0 as f64, r.1 as f64), 5, color.filled())
    }))?;

    // A small colour-bar legend in the upper-left of the panel.
    let bar_x0 = x_min + (x_max - x_min) * 0.02;
    let bar_y0 = y_max - (y_max - y_min) * 0.05;
    let bar_w = (x_max - x_min) * 0.30;
    let bar_h = (y_max - y_min) * 0.04;
    let n_steps = 60;
    for i in 0..n_steps {
        let t0 = i as f64 / n_steps as f64;
        let t1 = (i + 1) as f64 / n_steps as f64;
        chart.draw_series(std::iter::once(Rectangle::new(
            [
                (bar_x0 + bar_w * t0, bar_y0 - bar_h),
                (bar_x0 + bar_w * t1, bar_y0),
            ],
            viridis(t0).filled(),
        )))?;
    }
    chart.draw_series(std::iter::once(Text::new(
        format!("{} ({val_min:.2} → {val_max:.2})", legend_label),
        (bar_x0, bar_y0 + (y_max - y_min) * 0.01),
        ("sans-serif", 13),
    )))?;

    Ok(())
}

/// Approximate viridis colour map (5-stop linear interpolation).
fn viridis(t: f64) -> RGBColor {
    let stops = [
        (0.0, (68u8, 1, 84)),  // dark purple
        (0.25, (59, 82, 139)), // blue
        (0.5, (33, 145, 140)), // teal
        (0.75, (94, 201, 98)), // green
        (1.0, (253, 231, 37)), // yellow
    ];
    let t = t.clamp(0.0, 1.0);
    for w in stops.windows(2) {
        let (t0, c0) = w[0];
        let (t1, c1) = w[1];
        if t <= t1 {
            let f = (t - t0) / (t1 - t0).max(1e-12);
            let r = c0.0 as f64 + (c1.0 as f64 - c0.0 as f64) * f;
            let g = c0.1 as f64 + (c1.1 as f64 - c0.1 as f64) * f;
            let b = c0.2 as f64 + (c1.2 as f64 - c0.2 as f64) * f;
            return RGBColor(r as u8, g as u8, b as u8);
        }
    }
    RGBColor(
        stops[stops.len() - 1].1 .0,
        stops[stops.len() - 1].1 .1,
        stops[stops.len() - 1].1 .2,
    )
}

fn print_grid<F: Fn(&(i32, i32, f64, f64)) -> f64>(results: &[(i32, i32, f64, f64)], field: F) {
    // Header row: speeds.
    print!("  γ\\V  ");
    let speeds: Vec<i32> = (SPEED_MIN..SPEED_MAX).step_by(SPEED_STEP).collect();
    for s in &speeds {
        print!("  {s:>5}");
    }
    println!();
    print!("       ");
    for _ in &speeds {
        print!("  -----");
    }
    println!();

    for gamma in (GAMMA_MIN..GAMMA_MAX).rev() {
        print!("  {gamma:>3}  ");
        for s in &speeds {
            match results.iter().find(|r| r.0 == *s && r.1 == gamma) {
                Some(r) => print!("  {:>5.2}", field(r)),
                None => print!("      ·"),
            }
        }
        println!();
    }
}

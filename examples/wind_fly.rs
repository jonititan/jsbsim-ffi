//! Interactive flight with dynamic wind — fly a Cessna 172 in a live wind field.
//!
//! Demonstrates per-timestep wind injection into JSBSim from Rust, as
//! discussed in:
//!   - <https://github.com/JSBSim-Team/jsbsim/discussions/1130>
//!   - <https://github.com/JSBSim-Team/jsbsim/issues/74>
//!
//! The wind model features:
//!   - **Altitude-dependent base wind** (logarithmic profile — stronger aloft).
//!   - **User-adjustable base wind** speed and direction via keyboard.
//!   - **Pseudo-turbulence** overlay that updates every simulation step.
//!
//! Controls:
//!   W / S       → pitch down / up  (elevator)
//!   A / D       → roll left / right (aileron)
//!   Q / E       → yaw left / right  (rudder)
//!   Up / Down   → increase / decrease throttle
//!   Space       → level flight controls (center stick + rudder)
//!   B           → toggle parking brake
//!   J / L       → rotate base wind direction (CCW / CW)
//!   I / K       → increase / decrease base wind speed
//!   T           → toggle turbulence on/off
//!   Esc         → quit
//!
//! Run with:
//!
//! ```sh
//! JSBSIM_ROOT=/path/to/jsbsim cargo run --example wind_fly
//! ```

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{self, ClearType},
};
use jsbsim_ffi::Sim;
use std::io::{stdout, Write};
use std::time::{Duration, Instant};

// ── Simulation constants ────────────────────────────────────────────────
const SIM_DT: f64 = 1.0 / 120.0;
const SIM_STEP_DURATION: Duration = Duration::from_nanos((SIM_DT * 1_000_000_000.0) as u64);
const DISPLAY_EVERY_N_STEPS: u64 = 12; // ~10 Hz display at 120 Hz sim

// Control increments
const ELEVATOR_STEP: f64 = 0.05;
const AILERON_STEP: f64 = 0.05;
const RUDDER_STEP: f64 = 0.05;
const THROTTLE_STEP: f64 = 0.05;
const WIND_SPEED_STEP: f64 = 5.0; // ft/s (~3 kts)
const WIND_DIR_STEP: f64 = 10.0; // degrees

fn clamp(val: f64, min: f64, max: f64) -> f64 {
    val.max(min).min(max)
}

// ── Wind model ──────────────────────────────────────────────────────────

struct WindState {
    /// Base wind speed at reference altitude (ft/s).
    base_speed_fps: f64,
    /// Direction the wind blows FROM (degrees, 0 = north, 90 = east).
    from_direction_deg: f64,
    /// Whether pseudo-turbulence is active.
    turbulence_on: bool,
}

impl WindState {
    fn new() -> Self {
        Self {
            base_speed_fps: 20.0,      // ~12 kts
            from_direction_deg: 270.0, // from the west
            turbulence_on: true,
        }
    }

    /// Compute NED wind components (ft/s) for the current timestep.
    fn compute(&self, time_s: f64, altitude_agl_ft: f64) -> (f64, f64, f64) {
        // -- Logarithmic wind profile --
        let ref_alt = 1000.0_f64;
        let roughness = 0.1_f64;
        let alt = altitude_agl_ft.max(1.0);
        let profile = (alt / roughness).ln() / (ref_alt / roughness).ln();
        let speed = self.base_speed_fps * profile.max(0.0);

        // Decompose into NED ("from" direction → wind blows opposite).
        let from_rad = self.from_direction_deg.to_radians();
        let wn = -speed * from_rad.cos();
        let we = -speed * from_rad.sin();

        // -- Pseudo-turbulence --
        let (tn, te, td) = if self.turbulence_on {
            let a = 4.0_f64; // amplitude ft/s
            let tn = a
                * (0.7 * (0.31 * time_s).sin()
                    + 0.5 * (0.77 * time_s + 1.0).sin()
                    + 0.3 * (1.93 * time_s + 2.5).sin());
            let te = a
                * (0.6 * (0.37 * time_s + 0.5).sin()
                    + 0.4 * (0.89 * time_s + 1.7).sin()
                    + 0.3 * (2.17 * time_s + 3.1).sin());
            let td = a * 0.3 * (0.5 * (0.43 * time_s).sin() + 0.5 * (1.21 * time_s + 0.8).sin());
            (tn, te, td)
        } else {
            (0.0, 0.0, 0.0)
        };

        (wn + tn, we + te, td)
    }
}

// ── Main ────────────────────────────────────────────────────────────────

fn main() {
    let jsbsim_root = std::env::var("JSBSIM_ROOT").unwrap_or_else(|_| {
        eprintln!(
            "JSBSIM_ROOT is not set.  Point it at a directory containing\n\
             aircraft/, engine/, systems/ subdirectories.\n\
             \n\
             Example:\n\
             \n\
                 JSBSIM_ROOT=/path/to/jsbsim cargo run --example wind_fly\n"
        );
        std::process::exit(1);
    });

    // ── Create sim & load aircraft ──────────────────────────────────────
    let mut sim = Sim::new(&jsbsim_root);
    sim.set_debug_level(0);

    if !sim.load_model("c172x") {
        eprintln!("Failed to load aircraft model 'c172x'!");
        return;
    }

    // ── Initial conditions ──────────────────────────────────────────────
    sim.set_property("ic/h-sl-ft", 3000.0);
    sim.set_property("ic/vc-kts", 100.0);
    sim.set_property("ic/psi-true-deg", 0.0);
    sim.set_property("ic/lat-geod-deg", 37.62);
    sim.set_property("ic/long-gc-deg", -122.38);
    sim.set_property("ic/gamma-deg", 0.0);

    if !sim.run_ic() {
        eprintln!("Failed to initialise conditions!");
        return;
    }

    sim.set_property("propulsion/engine/set-running", 1.0);
    sim.set_property("fcs/mixture-cmd-norm", 1.0);
    sim.set_property("gear/gear-cmd-norm", 0.0);

    // ── Control state ───────────────────────────────────────────────────
    let mut throttle = 0.65_f64;
    let mut elevator = 0.0_f64;
    let mut aileron = 0.0_f64;
    let mut rudder = 0.0_f64;
    let mut brake = false;

    // ── Wind state ──────────────────────────────────────────────────────
    let mut wind = WindState::new();

    // ── Terminal setup ──────────────────────────────────────────────────
    terminal::enable_raw_mode().expect("Failed to enable raw mode");
    let mut out = stdout();
    execute!(out, terminal::Clear(ClearType::All), cursor::Hide).ok();

    println!("\r\n  ✈  Interactive Flight with Dynamic Wind — Cessna 172  ✈\r");
    println!("  W/S=Pitch  A/D=Roll  Q/E=Rudder  ↑/↓=Throttle  Space=Level\r");
    println!("  I/K=Wind±  J/L=Wind dir  T=Turb toggle  B=Brake  Esc=Quit\r\n");

    let mut step_count: u64 = 0;

    // ── Simulation loop ─────────────────────────────────────────────────
    'main: loop {
        let frame_start = Instant::now();

        // ── (a) Poll keyboard ───────────────────────────────────────────
        while event::poll(Duration::from_millis(0)).unwrap_or(false) {
            if let Ok(Event::Key(KeyEvent {
                code,
                kind: KeyEventKind::Press,
                ..
            })) = event::read()
            {
                match code {
                    KeyCode::Esc => break 'main,

                    // Flight controls
                    KeyCode::Char('w') | KeyCode::Char('W') => {
                        elevator = clamp(elevator - ELEVATOR_STEP, -1.0, 1.0);
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        elevator = clamp(elevator + ELEVATOR_STEP, -1.0, 1.0);
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        aileron = clamp(aileron - AILERON_STEP, -1.0, 1.0);
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        aileron = clamp(aileron + AILERON_STEP, -1.0, 1.0);
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        rudder = clamp(rudder - RUDDER_STEP, -1.0, 1.0);
                    }
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        rudder = clamp(rudder + RUDDER_STEP, -1.0, 1.0);
                    }
                    KeyCode::Up => throttle = clamp(throttle + THROTTLE_STEP, 0.0, 1.0),
                    KeyCode::Down => throttle = clamp(throttle - THROTTLE_STEP, 0.0, 1.0),
                    KeyCode::Char(' ') => {
                        elevator = 0.0;
                        aileron = 0.0;
                        rudder = 0.0;
                    }
                    KeyCode::Char('b') | KeyCode::Char('B') => brake = !brake,

                    // Wind controls
                    KeyCode::Char('i') | KeyCode::Char('I') => {
                        wind.base_speed_fps = (wind.base_speed_fps + WIND_SPEED_STEP).min(100.0);
                    }
                    KeyCode::Char('k') | KeyCode::Char('K') => {
                        wind.base_speed_fps = (wind.base_speed_fps - WIND_SPEED_STEP).max(0.0);
                    }
                    KeyCode::Char('j') | KeyCode::Char('J') => {
                        wind.from_direction_deg =
                            (wind.from_direction_deg - WIND_DIR_STEP).rem_euclid(360.0);
                    }
                    KeyCode::Char('l') | KeyCode::Char('L') => {
                        wind.from_direction_deg =
                            (wind.from_direction_deg + WIND_DIR_STEP).rem_euclid(360.0);
                    }
                    KeyCode::Char('t') | KeyCode::Char('T') => {
                        wind.turbulence_on = !wind.turbulence_on;
                    }

                    _ => {}
                }
            }
        }

        // ── (b) Copy flight controls to JSBSim ─────────────────────────
        sim.set_property("fcs/throttle-cmd-norm", throttle);
        sim.set_property("fcs/elevator-cmd-norm", elevator);
        sim.set_property("fcs/aileron-cmd-norm", aileron);
        sim.set_property("fcs/rudder-cmd-norm", rudder);
        sim.set_property("fcs/center-brake-cmd-norm", if brake { 1.0 } else { 0.0 });

        // ── (c) Compute and inject wind ─────────────────────────────────
        let time = sim.get_sim_time();
        let alt_agl = sim.get_property("position/h-agl-ft");
        let (wn, we, wd) = wind.compute(time, alt_agl);

        sim.set_property("atmosphere/wind-north-fps", wn);
        sim.set_property("atmosphere/wind-east-fps", we);
        sim.set_property("atmosphere/wind-down-fps", wd);

        // ── (d) Run one sim step ────────────────────────────────────────
        sim.run();
        step_count += 1;

        // ── (e) Display ─────────────────────────────────────────────────
        if step_count.is_multiple_of(DISPLAY_EVERY_N_STEPS) {
            let alt_msl = sim.get_property("position/h-sl-ft");
            let ias = sim.get_property("velocities/vc-kts");
            let vsi = sim.get_property("velocities/h-dot-fps") * 60.0;
            let hdg = sim.get_property("attitude/psi-deg");
            let pitch = sim.get_property("attitude/theta-deg");
            let roll = sim.get_property("attitude/phi-deg");
            let lat = sim.get_property("position/lat-geod-deg");
            let lon = sim.get_property("position/long-gc-deg");
            let mach = sim.get_property("velocities/mach");
            let tas = sim.get_property("velocities/vt-fps") * 0.592_484;
            let rpm = sim.get_property("propulsion/engine/engine-rpm");
            let gear = sim.get_property("gear/gear-cmd-norm");

            // Total wind magnitude as seen by JSBSim
            let tw_n = sim.get_property("atmosphere/total-wind-north-fps");
            let tw_e = sim.get_property("atmosphere/total-wind-east-fps");
            let tw_d = sim.get_property("atmosphere/total-wind-down-fps");
            let tw_mag = (tw_n * tw_n + tw_e * tw_e + tw_d * tw_d).sqrt();
            let tw_mag_kts = tw_mag * 0.592_484;

            // Groundspeed
            let gs_n = sim.get_property("velocities/v-north-fps");
            let gs_e = sim.get_property("velocities/v-east-fps");
            let gs_kts = (gs_n * gs_n + gs_e * gs_e).sqrt() * 0.592_484;

            execute!(out, cursor::MoveTo(0, 4)).ok();

            write!(
                out,
                "  ┌─────────────── Flight Instruments ───────────────┐\r\n"
            )
            .ok();
            write!(
                out,
                "  │  Time: {:>7.1}s         Mach: {:.3}              │\r\n",
                time, mach
            )
            .ok();
            write!(
                out,
                "  │  IAS:  {:>7.1} kts      TAS:  {:>6.1} kts        │\r\n",
                ias, tas
            )
            .ok();
            write!(
                out,
                "  │  GS:   {:>7.1} kts      RPM:  {:.0}           │\r\n",
                gs_kts, rpm
            )
            .ok();
            write!(
                out,
                "  │  ALT:  {:>7.0} ft MSL   VSI:  {:>+7.0} fpm       │\r\n",
                alt_msl, vsi
            )
            .ok();
            write!(
                out,
                "  │  AGL:  {:>7.0} ft        HDG:  {:>5.1}°           │\r\n",
                alt_agl, hdg
            )
            .ok();
            write!(
                out,
                "  │  Pitch: {:>+6.1}°  Roll: {:>+6.1}°               │\r\n",
                pitch, roll
            )
            .ok();
            write!(
                out,
                "  │  Lat: {:>9.4}°   Lon: {:>10.4}°           │\r\n",
                lat, lon
            )
            .ok();
            write!(
                out,
                "  ├─────────────── Wind Field ───────────────────────┤\r\n"
            )
            .ok();
            write!(
                out,
                "  │  Base:  {:>5.1} fps ({:>5.1} kts)  from {:>5.1}°      │\r\n",
                wind.base_speed_fps,
                wind.base_speed_fps * 0.592_484,
                wind.from_direction_deg
            )
            .ok();
            write!(
                out,
                "  │  Total: {:>5.1} fps ({:>5.1} kts)  Turb: {}       │\r\n",
                tw_mag,
                tw_mag_kts,
                if wind.turbulence_on { " ON" } else { "OFF" }
            )
            .ok();
            write!(
                out,
                "  │  Wn: {:>+6.1}  We: {:>+6.1}  Wd: {:>+6.1} fps     │\r\n",
                tw_n, tw_e, tw_d
            )
            .ok();
            write!(
                out,
                "  ├─────────────── Controls ─────────────────────────┤\r\n"
            )
            .ok();
            write!(
                out,
                "  │  Throttle: {:<4.0}%   Elevator: {:>+5.2}             │\r\n",
                throttle * 100.0,
                elevator
            )
            .ok();
            write!(
                out,
                "  │  Aileron:  {:>+5.2}    Rudder:   {:>+5.2}             │\r\n",
                aileron, rudder
            )
            .ok();
            write!(
                out,
                "  │  Gear: {}    Brake: {}                      │\r\n",
                if gear > 0.5 { "DOWN" } else { " UP " },
                if brake { " ON" } else { "OFF" }
            )
            .ok();
            write!(
                out,
                "  └─────────────────────────────────────────────────┘\r\n"
            )
            .ok();
            out.flush().ok();

            if alt_agl < 0.0 {
                write!(out, "\r\n  💥 CRASHED! The aircraft hit the ground.\r\n").ok();
                out.flush().ok();
                std::thread::sleep(Duration::from_secs(2));
                break 'main;
            }
        }

        // ── (f) Real-time pacing ────────────────────────────────────────
        let elapsed = frame_start.elapsed();
        if elapsed < SIM_STEP_DURATION {
            std::thread::sleep(SIM_STEP_DURATION - elapsed);
        }
    }

    // ── Cleanup ─────────────────────────────────────────────────────────
    execute!(out, cursor::Show).ok();
    terminal::disable_raw_mode().ok();
    println!("\r\n  Flight ended. Thanks for flying! ✈\r");
}

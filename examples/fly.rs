//! Interactive flight example — fly a Cessna 172 with your keyboard.
//!
//! This follows the JSBSim cyclic execution pattern:
//!   1. Copy control inputs to JSBSim (set_property)
//!   2. Execute one simulation step (Run)
//!   3. Read state from JSBSim (get_property)
//!   4. Sleep to maintain real-time pacing
//!
//! Controls:
//!   W / S       → pitch down / up  (elevator)
//!   A / D       → roll left / right (aileron)
//!   Q / E       → yaw left / right  (rudder)
//!   Up / Down   → increase / decrease throttle
//!   Space       → level controls (center stick + rudder)
//!   B           → toggle parking brake
//!   Esc         → quit
//!
//! Run with:
//!   JSBSIM_ROOT=/path/to/jsbsim cargo run --example fly

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{self, ClearType},
    cursor,
    execute,
};
use jsbsim_ffi::Sim;
use std::io::{stdout, Write};
use std::time::{Duration, Instant};

// Simulation rate: JSBSim default is 120 Hz
const SIM_DT: f64 = 1.0 / 120.0;
const SIM_STEP_DURATION: Duration = Duration::from_nanos((SIM_DT * 1_000_000_000.0) as u64);

// How often to refresh the display (every N sim steps)
const DISPLAY_EVERY_N_STEPS: u64 = 12; // ~10 Hz display at 120 Hz sim

// Control increments per key press
const ELEVATOR_STEP: f64 = 0.05;
const AILERON_STEP: f64 = 0.05;
const RUDDER_STEP: f64 = 0.05;
const THROTTLE_STEP: f64 = 0.05;

fn clamp(val: f64, min: f64, max: f64) -> f64 {
    val.max(min).min(max)
}

fn main() {
    // ── 1. Create FGFDMExec and load model ──────────────────────────────
    // JSBSIM_ROOT must point to a directory containing aircraft/, engine/,
    // systems/ subdirectories (typically the JSBSim source checkout).
    let jsbsim_root = std::env::var("JSBSIM_ROOT").unwrap_or_else(|_| {
        eprintln!(
            "JSBSIM_ROOT is not set.  Point it at a directory containing\n\
             aircraft/, engine/, systems/ subdirectories.\n\
             \n\
             Example:\n\
             \n\
                 JSBSIM_ROOT=/path/to/jsbsim cargo run --example fly\n"
        );
        std::process::exit(1);
    });

    let mut sim = Sim::new(&jsbsim_root);

    if !sim.load_model("c172x") {
        eprintln!("Failed to load aircraft model 'c172x'!");
        return;
    }

    // ── 2. Set initial conditions ───────────────────────────────────────
    sim.set_property("ic/h-sl-ft", 3000.0);        // altitude MSL
    sim.set_property("ic/vc-kts", 100.0);           // calibrated airspeed
    sim.set_property("ic/psi-true-deg", 0.0);       // heading north
    sim.set_property("ic/lat-geod-deg", 37.62);     // San Francisco area
    sim.set_property("ic/long-gc-deg", -122.38);
    sim.set_property("ic/gamma-deg", 0.0);          // level flight

    // ── 3. RunIC — initialise without integrating ───────────────────────
    if !sim.run_ic() {
        eprintln!("Failed to initialise conditions!");
        return;
    }

    // ── 4. Set initial control inputs ───────────────────────────────────
    sim.set_property("propulsion/engine/set-running", 1.0);
    sim.set_property("fcs/mixture-cmd-norm", 1.0);
    sim.set_property("gear/gear-cmd-norm", 0.0);    // gear up

    // Control state — these are what the user modifies with keyboard
    let mut throttle = 0.65_f64;
    let mut elevator = 0.0_f64;
    let mut aileron  = 0.0_f64;
    let mut rudder   = 0.0_f64;
    let mut brake    = false;

    // ── Terminal setup ──────────────────────────────────────────────────
    terminal::enable_raw_mode().expect("Failed to enable raw mode");
    let mut out = stdout();
    execute!(out, terminal::Clear(ClearType::All), cursor::Hide).ok();

    println!("\r\n  ✈  Interactive JSBSim Flight — Cessna 172  ✈\r");
    println!("  W/S=Pitch  A/D=Roll  Q/E=Rudder  ↑/↓=Throttle  Space=Level  B=Brake  Esc=Quit\r\n");

    let mut step_count: u64 = 0;

    // ── 5. Cyclic execution loop (real-time paced) ──────────────────────
    'main: loop {
        let frame_start = Instant::now();

        // ── (a) Poll keyboard input (non-blocking) ─────────────────────
        while event::poll(Duration::from_millis(0)).unwrap_or(false) {
            if let Ok(Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. })) = event::read() {
                match code {
                    KeyCode::Esc => break 'main,

                    // Pitch: W = push nose down, S = pull up
                    KeyCode::Char('w') | KeyCode::Char('W') => {
                        elevator = clamp(elevator - ELEVATOR_STEP, -1.0, 1.0);
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        elevator = clamp(elevator + ELEVATOR_STEP, -1.0, 1.0);
                    }

                    // Roll: A = left, D = right
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        aileron = clamp(aileron - AILERON_STEP, -1.0, 1.0);
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        aileron = clamp(aileron + AILERON_STEP, -1.0, 1.0);
                    }

                    // Yaw: Q = left rudder, E = right rudder
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        rudder = clamp(rudder - RUDDER_STEP, -1.0, 1.0);
                    }
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        rudder = clamp(rudder + RUDDER_STEP, -1.0, 1.0);
                    }

                    // Throttle
                    KeyCode::Up => {
                        throttle = clamp(throttle + THROTTLE_STEP, 0.0, 1.0);
                    }
                    KeyCode::Down => {
                        throttle = clamp(throttle - THROTTLE_STEP, 0.0, 1.0);
                    }

                    // Level all flight controls
                    KeyCode::Char(' ') => {
                        elevator = 0.0;
                        aileron = 0.0;
                        rudder = 0.0;
                    }

                    // Parking brake toggle
                    KeyCode::Char('b') | KeyCode::Char('B') => {
                        brake = !brake;
                    }

                    _ => {}
                }
            }
        }

        // ── (b) Copy control inputs to JSBSim ──────────────────────────
        sim.set_property("fcs/throttle-cmd-norm", throttle);
        sim.set_property("fcs/elevator-cmd-norm", elevator);
        sim.set_property("fcs/aileron-cmd-norm", aileron);
        sim.set_property("fcs/rudder-cmd-norm", rudder);
        sim.set_property("fcs/center-brake-cmd-norm", if brake { 1.0 } else { 0.0 });

        // ── (c) Execute ONE simulation step ─────────────────────────────
        sim.run();
        step_count += 1;

        // ── (d) Copy state from JSBSim & display ────────────────────────
        if step_count % DISPLAY_EVERY_N_STEPS == 0 {
            let time    = sim.get_property("simulation/sim-time-sec");
            let alt_agl = sim.get_property("position/h-agl-ft");
            let alt_msl = sim.get_property("position/h-sl-ft");
            let ias     = sim.get_property("velocities/vc-kts");
            let vsi     = sim.get_property("velocities/h-dot-fps") * 60.0;
            let hdg     = sim.get_property("attitude/psi-deg");
            let pitch   = sim.get_property("attitude/theta-deg");
            let roll    = sim.get_property("attitude/phi-deg");
            let lat     = sim.get_property("position/lat-geod-deg");
            let lon     = sim.get_property("position/long-gc-deg");
            let mach    = sim.get_property("velocities/mach");
            let rpm     = sim.get_property("propulsion/engine/engine-rpm");
            let gear    = sim.get_property("gear/gear-cmd-norm");

            execute!(out, cursor::MoveTo(0, 3)).ok();

            write!(out, "  ┌─────────────── Flight Instruments ───────────────┐\r\n").ok();
            write!(out, "  │  Time: {:>7.1}s         Mach: {:.3}              │\r\n", time, mach).ok();
            write!(out, "  │  IAS:  {:>7.1} kts      RPM:  {:.0}           │\r\n", ias, rpm).ok();
            write!(out, "  │  ALT:  {:>7.0} ft MSL   VSI:  {:>+7.0} fpm       │\r\n", alt_msl, vsi).ok();
            write!(out, "  │  AGL:  {:>7.0} ft        HDG:  {:>5.1}°           │\r\n", alt_agl, hdg).ok();
            write!(out, "  │  Pitch: {:>+6.1}°  Roll: {:>+6.1}°               │\r\n", pitch, roll).ok();
            write!(out, "  │  Lat: {:>9.4}°   Lon: {:>10.4}°           │\r\n", lat, lon).ok();
            write!(out, "  ├─────────────── Controls ─────────────────────────┤\r\n").ok();
            write!(out, "  │  Throttle: {:<4.0}%   Elevator: {:>+5.2}             │\r\n", throttle * 100.0, elevator).ok();
            write!(out, "  │  Aileron:  {:>+5.2}    Rudder:   {:>+5.2}             │\r\n", aileron, rudder).ok();
            write!(out, "  │  Gear: {}    Brake: {}                      │\r\n",
                if gear > 0.5 { "DOWN" } else { " UP " },
                if brake { " ON" } else { "OFF" }).ok();
            write!(out, "  └─────────────────────────────────────────────────┘\r\n").ok();
            out.flush().ok();

            // Check for crash
            if alt_agl < 0.0 {
                write!(out, "\r\n  💥 CRASHED! The aircraft hit the ground.\r\n").ok();
                out.flush().ok();
                std::thread::sleep(Duration::from_secs(2));
                break 'main;
            }
        }

        // ── (e) Real-time pacing: sleep for remainder of this frame ─────
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

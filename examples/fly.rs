//! Interactive flight example — fly a Cessna 172 with your keyboard.
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
//!   LD_LIBRARY_PATH=/usr/local/lib cargo run --example fly

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{self, ClearType},
    cursor,
    execute,
};
use jsbsim_ffi::Sim;
use std::io::{stdout, Write};
use std::time::{Duration, Instant};

// Control increments per key press
const ELEVATOR_STEP: f64 = 0.05;
const AILERON_STEP: f64 = 0.05;
const RUDDER_STEP: f64 = 0.05;
const THROTTLE_STEP: f64 = 0.05;

fn clamp(val: f64, min: f64, max: f64) -> f64 {
    val.max(min).min(max)
}

fn main() {
    // ── Initialise JSBSim ───────────────────────────────────────────────
    // Change this path to your JSBSim source checkout
    let jsbsim_root = "/home/joni/jsbsim";

    let mut sim = Sim::new(jsbsim_root);

    // Load the Cessna 172 model directly
    if !sim.load_model("c172x") {
        eprintln!("Failed to load aircraft model 'c172x'!");
        return;
    }

    // Set initial conditions: airborne at 3000 ft, 100 kts, heading north
    sim.set_property("ic/h-sl-ft", 3000.0);       // altitude MSL
    sim.set_property("ic/vc-kts", 100.0);          // calibrated airspeed
    sim.set_property("ic/psi-true-deg", 0.0);      // heading north
    sim.set_property("ic/lat-geod-deg", 37.62);    // San Francisco area
    sim.set_property("ic/long-gc-deg", -122.38);
    sim.set_property("ic/gamma-deg", 0.0);         // level flight

    if !sim.run_ic() {
        eprintln!("Failed to initialise conditions!");
        return;
    }

    // Set up for flight: engine running, throttle at cruise
    sim.set_property("propulsion/engine/set-running", 1.0);
    sim.set_property("fcs/throttle-cmd-norm", 0.65);
    sim.set_property("fcs/mixture-cmd-norm", 1.0);
    sim.set_property("gear/gear-cmd-norm", 0.0); // gear up
    sim.set_property("fcs/elevator-cmd-norm", 0.0);
    sim.set_property("fcs/aileron-cmd-norm", 0.0);
    sim.set_property("fcs/rudder-cmd-norm", 0.0);

    // Control state
    let mut throttle = 0.65_f64;
    let mut elevator = 0.0_f64;
    let mut aileron = 0.0_f64;
    let mut rudder = 0.0_f64;
    let mut brake = false;

    // ── Terminal setup ──────────────────────────────────────────────────
    terminal::enable_raw_mode().expect("Failed to enable raw mode");
    let mut out = stdout();
    execute!(out, terminal::Clear(ClearType::All), cursor::Hide).ok();

    let sim_hz = 120; // JSBSim internal rate
    let display_hz = 10; // Screen refresh rate
    let steps_per_display = sim_hz / display_hz;
    let display_interval = Duration::from_millis(1000 / display_hz as u64);
    let mut last_display = Instant::now();

    println!("\r\n  ✈  Interactive JSBSim Flight — Cessna 172  ✈\r");
    println!("  Controls: W/S=Pitch  A/D=Roll  Q/E=Rudder  ↑/↓=Throttle  Space=Level  B=Brake  Esc=Quit\r\n");

    // ── Main loop ───────────────────────────────────────────────────────
    'main: loop {
        // Poll for keyboard input (non-blocking)
        while event::poll(Duration::from_millis(0)).unwrap_or(false) {
            if let Ok(Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. })) = event::read() {
                match code {
                    KeyCode::Esc => break 'main,

                    // Pitch: W = push nose down (negative elevator), S = pull up
                    KeyCode::Char('w') | KeyCode::Char('W') => {
                        elevator = clamp(elevator - ELEVATOR_STEP, -1.0, 1.0);
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        elevator = clamp(elevator + ELEVATOR_STEP, -1.0, 1.0);
                    }

                    // Roll: A = left (negative aileron), D = right
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

                    // Level all controls
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

        // Apply controls to the simulation
        sim.set_property("fcs/throttle-cmd-norm", throttle);
        sim.set_property("fcs/elevator-cmd-norm", elevator);
        sim.set_property("fcs/aileron-cmd-norm", aileron);
        sim.set_property("fcs/rudder-cmd-norm", rudder);
        sim.set_property("fcs/center-brake-cmd-norm", if brake { 1.0 } else { 0.0 });

        // Run simulation steps
        for _ in 0..steps_per_display {
            sim.run();
        }

        // Display HUD at the target refresh rate
        if last_display.elapsed() >= display_interval {
            last_display = Instant::now();

            let time = sim.get_property("simulation/sim-time-sec");
            let alt_agl = sim.get_property("position/h-agl-ft");
            let alt_msl = sim.get_property("position/h-sl-ft");
            let ias = sim.get_property("velocities/vc-kts");
            let vsi = sim.get_property("velocities/h-dot-fps") * 60.0; // convert fps to fpm
            let hdg = sim.get_property("attitude/psi-deg");
            let pitch = sim.get_property("attitude/theta-deg");
            let roll = sim.get_property("attitude/phi-deg");
            let lat = sim.get_property("position/lat-geod-deg");
            let lon = sim.get_property("position/long-gc-deg");
            let mach = sim.get_property("velocities/mach");
            let rpm = sim.get_property("propulsion/engine/engine-rpm");
            let gear = sim.get_property("gear/gear-cmd-norm");

            // Move cursor to line 4 and overwrite
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
            if alt_agl < -10.0 {
                write!(out, "\r\n  💥 CRASHED! The aircraft hit the ground.\r\n").ok();
                out.flush().ok();
                std::thread::sleep(Duration::from_secs(2));
                break 'main;
            }
        }
    }

    // ── Cleanup ─────────────────────────────────────────────────────────
    execute!(out, cursor::Show).ok();
    terminal::disable_raw_mode().ok();
    println!("\r\n  Flight ended. Thanks for flying! ✈\r");
}

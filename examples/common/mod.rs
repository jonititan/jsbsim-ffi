//! Shared helper for the example binaries — resolve `JSBSIM_ROOT` and
//! validate that it points at a real JSBSim data tree.
//!
//! Each example includes this file via `#[path = "common.rs"] mod common;`
//! because cargo treats every file under `examples/` as an independent
//! binary target and there is no implicit `mod` lookup between them.
//!
//! ## Why hard-fail instead of skip
//!
//! It is very easy to copy `JSBSIM_ROOT=/path/to/jsbsim …` from the README
//! verbatim, leaving the placeholder in place.  Without validation,
//! `Sim::new("/path/to/jsbsim")` succeeds (the FGFDMExec constructor just
//! stores the path), then `load_model` fails with a confusing JSBSim
//! error and the example exits with status 0 — a silent failure.
//!
//! [`jsbsim_root_or_exit`] checks both that `JSBSIM_ROOT` is set and that
//! `<root>/aircraft/` is a real directory.  If either check fails it
//! prints an actionable message and exits with status 2 so callers can
//! tell the run was rejected.

#![allow(dead_code)] // Each example only uses a subset of the helpers below.

use std::path::Path;

/// Resolve `JSBSIM_ROOT`, validate it, and return it as a `String`.
///
/// Exits the process with status 2 if:
///   - `JSBSIM_ROOT` is unset or empty, or
///   - `<JSBSIM_ROOT>/aircraft` is not an existing directory.
///
/// `example_name` is interpolated into the error message so the user
/// gets a copy-pasteable command for the specific example they invoked.
pub fn jsbsim_root_or_exit(example_name: &str) -> String {
    let raw = std::env::var("JSBSIM_ROOT").unwrap_or_default();
    if raw.is_empty() {
        eprintln!(
            "JSBSIM_ROOT is not set.\n\
             \n\
             Set it to a real JSBSim checkout containing aircraft/, engine/,\n\
             systems/, and scripts/ subdirectories, then re-run:\n\
             \n    JSBSIM_ROOT=$HOME/jsbsim cargo run --example {example_name}\n"
        );
        std::process::exit(2);
    }
    if !Path::new(&raw).join("aircraft").is_dir() {
        eprintln!(
            "JSBSIM_ROOT={raw:?} does not contain an `aircraft/` subdirectory.\n\
             \n\
             It looks like the env var still has the documentation placeholder\n\
             rather than a real path.  Point it at your JSBSim checkout:\n\
             \n    JSBSIM_ROOT=$HOME/jsbsim cargo run --example {example_name}\n"
        );
        std::process::exit(2);
    }
    raw
}

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[allow(non_camel_case_types)]
pub type JSBSim_FGFDMExec = *mut std::ffi::c_void;

#[link(name = "jsbsim_wrapper")]
extern "C" {
    // Lifecycle
    fn jsbsim_create(root_dir: *const c_char) -> JSBSim_FGFDMExec;
    fn jsbsim_destroy(fdm: JSBSim_FGFDMExec);

    // Loading
    fn jsbsim_load_model(fdm: JSBSim_FGFDMExec, model: *const c_char) -> bool;
    fn jsbsim_load_script(fdm: JSBSim_FGFDMExec, filename: *const c_char) -> bool;
    fn jsbsim_load_ic(fdm: JSBSim_FGFDMExec, filename: *const c_char, use_aircraft_path: bool) -> bool;

    // Simulation control
    fn jsbsim_run_ic(fdm: JSBSim_FGFDMExec) -> bool;
    fn jsbsim_run(fdm: JSBSim_FGFDMExec) -> bool;
    fn jsbsim_set_dt(fdm: JSBSim_FGFDMExec, dt: f64);
    fn jsbsim_get_dt(fdm: JSBSim_FGFDMExec) -> f64;
    fn jsbsim_get_sim_time(fdm: JSBSim_FGFDMExec) -> f64;
    fn jsbsim_reset_to_initial_conditions(fdm: JSBSim_FGFDMExec, mode: i32);

    // Hold / Resume
    fn jsbsim_hold(fdm: JSBSim_FGFDMExec);
    fn jsbsim_resume(fdm: JSBSim_FGFDMExec);
    fn jsbsim_holding(fdm: JSBSim_FGFDMExec) -> bool;
    fn jsbsim_enable_increment_then_hold(fdm: JSBSim_FGFDMExec, steps: i32);
    fn jsbsim_check_incremental_hold(fdm: JSBSim_FGFDMExec);

    // Integration suspend
    fn jsbsim_suspend_integration(fdm: JSBSim_FGFDMExec);
    fn jsbsim_resume_integration(fdm: JSBSim_FGFDMExec);

    // Trim
    fn jsbsim_do_trim(fdm: JSBSim_FGFDMExec, mode: i32) -> bool;

    // Properties
    fn jsbsim_get_property(fdm: JSBSim_FGFDMExec, name: *const c_char) -> f64;
    fn jsbsim_set_property(fdm: JSBSim_FGFDMExec, name: *const c_char, value: f64) -> bool;
    fn jsbsim_has_property(fdm: JSBSim_FGFDMExec, name: *const c_char) -> bool;

    // Property catalog
    fn jsbsim_query_property_catalog(
        fdm: JSBSim_FGFDMExec,
        check: *const c_char,
        buf: *mut c_char,
        buf_len: i32,
    ) -> i32;
    fn jsbsim_print_property_catalog(fdm: JSBSim_FGFDMExec);

    // Output control
    fn jsbsim_set_output_directive(fdm: JSBSim_FGFDMExec, fname: *const c_char) -> bool;
    fn jsbsim_enable_output(fdm: JSBSim_FGFDMExec);
    fn jsbsim_disable_output(fdm: JSBSim_FGFDMExec);
    fn jsbsim_set_output_filename(fdm: JSBSim_FGFDMExec, n: i32, fname: *const c_char) -> bool;

    // Path configuration
    fn jsbsim_set_aircraft_path(fdm: JSBSim_FGFDMExec, path: *const c_char) -> bool;
    fn jsbsim_set_engine_path(fdm: JSBSim_FGFDMExec, path: *const c_char) -> bool;
    fn jsbsim_set_systems_path(fdm: JSBSim_FGFDMExec, path: *const c_char) -> bool;

    // Info / Debug
    fn jsbsim_set_debug_level(fdm: JSBSim_FGFDMExec, level: i32);
    fn jsbsim_get_model_name(fdm: JSBSim_FGFDMExec, buf: *mut c_char, buf_len: i32) -> i32;
    fn jsbsim_get_version(buf: *mut c_char, buf_len: i32) -> i32;
}

/// Helper: read a C string-returning FFI function into a Rust `String`.
fn read_c_string(f: impl Fn(*mut c_char, i32) -> i32) -> String {
    let mut buf = vec![0u8; 256];
    let len = f(buf.as_mut_ptr() as *mut c_char, buf.len() as i32);
    if len <= 0 {
        return String::new();
    }
    // If the buffer was too small, retry with exact size.
    if len as usize >= buf.len() {
        buf.resize(len as usize + 1, 0);
        f(buf.as_mut_ptr() as *mut c_char, buf.len() as i32);
    }
    let cstr = unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char) };
    cstr.to_string_lossy().into_owned()
}

// ── Trim modes (mirrors JSBSim tType enum) ──────────────────────────
/// Trim mode constants for [`Sim::do_trim`].
pub mod trim {
    /// Longitudinal trim (pitch + throttle).
    pub const LONGITUDINAL: i32 = 0;
    /// Full trim (all axes).
    pub const FULL: i32 = 1;
    /// Ground trim.
    pub const GROUND: i32 = 2;
    /// Pullup trim.
    pub const PULLUP: i32 = 3;
    /// Custom trim.
    pub const CUSTOM: i32 = 4;
    /// Turn trim.
    pub const TURN: i32 = 5;
}

pub struct Sim {
    inner: JSBSim_FGFDMExec,
}

impl Sim {
    // ── Lifecycle ────────────────────────────────────────────────────

    /// Create a new JSBSim FDM instance.
    ///
    /// `root_dir` = path to your JSBSim data root (contains `aircraft/`, `engine/`,
    /// `systems/`, and `scripts/` directories).
    pub fn new(root_dir: &str) -> Self {
        let c_root = CString::new(root_dir).expect("root_dir contains null byte");
        let inner = unsafe { jsbsim_create(c_root.as_ptr()) };
        Self { inner }
    }

    // ── Loading ─────────────────────────────────────────────────────

    /// Load an aircraft model by name (e.g., `"c172x"`).
    pub fn load_model(&mut self, model: &str) -> bool {
        let c_model = CString::new(model).expect("model name contains null byte");
        unsafe { jsbsim_load_model(self.inner, c_model.as_ptr()) }
    }

    /// Load a JSBSim script XML file.
    pub fn load_script(&mut self, filename: &str) -> bool {
        let c_file = CString::new(filename).expect("filename contains null byte");
        unsafe { jsbsim_load_script(self.inner, c_file.as_ptr()) }
    }

    /// Load initial conditions from an XML file.
    ///
    /// If `use_aircraft_path` is true, the path is relative to the aircraft directory.
    pub fn load_ic(&mut self, filename: &str, use_aircraft_path: bool) -> bool {
        let c_file = CString::new(filename).expect("filename contains null byte");
        unsafe { jsbsim_load_ic(self.inner, c_file.as_ptr(), use_aircraft_path) }
    }

    // ── Simulation control ──────────────────────────────────────────

    /// Initialize and run initial conditions.
    pub fn run_ic(&mut self) -> bool {
        unsafe { jsbsim_run_ic(self.inner) }
    }

    /// Advance the simulation by one time step.
    pub fn run(&mut self) -> bool {
        unsafe { jsbsim_run(self.inner) }
    }

    /// Set the simulation time step in seconds.
    pub fn set_dt(&mut self, dt: f64) {
        unsafe { jsbsim_set_dt(self.inner, dt) }
    }

    /// Get the current simulation time step in seconds.
    pub fn get_dt(&self) -> f64 {
        unsafe { jsbsim_get_dt(self.inner) }
    }

    /// Get the current simulation time in seconds.
    pub fn get_sim_time(&self) -> f64 {
        unsafe { jsbsim_get_sim_time(self.inner) }
    }

    /// Reset the simulation to its initial conditions.
    ///
    /// `mode`: 0 = reset state only, 1 = also reload the model.
    pub fn reset_to_initial_conditions(&mut self, mode: i32) {
        unsafe { jsbsim_reset_to_initial_conditions(self.inner, mode) }
    }

    // ── Hold / Resume ───────────────────────────────────────────────

    /// Pause the simulation (time stops advancing).
    pub fn hold(&mut self) {
        unsafe { jsbsim_hold(self.inner) }
    }

    /// Resume the simulation from a hold.
    pub fn resume(&mut self) {
        unsafe { jsbsim_resume(self.inner) }
    }

    /// Returns `true` if the simulation is currently on hold.
    pub fn holding(&self) -> bool {
        unsafe { jsbsim_holding(self.inner) }
    }

    /// Run `steps` more time steps, then automatically hold.
    pub fn enable_increment_then_hold(&mut self, steps: i32) {
        unsafe { jsbsim_enable_increment_then_hold(self.inner, steps) }
    }

    /// Check and process an incremental-hold request (called internally by `run()`).
    pub fn check_incremental_hold(&mut self) {
        unsafe { jsbsim_check_incremental_hold(self.inner) }
    }

    // ── Integration suspend ─────────────────────────────────────────

    /// Freeze physics integration (dt becomes 0) while still calling `run()`.
    pub fn suspend_integration(&mut self) {
        unsafe { jsbsim_suspend_integration(self.inner) }
    }

    /// Resume physics integration after a suspend.
    pub fn resume_integration(&mut self) {
        unsafe { jsbsim_resume_integration(self.inner) }
    }

    // ── Trim ────────────────────────────────────────────────────────

    /// Trim the aircraft. Returns `true` on success.
    ///
    /// Use constants from [`trim`] module: `trim::LONGITUDINAL`, `trim::FULL`,
    /// `trim::GROUND`, etc.
    pub fn do_trim(&mut self, mode: i32) -> bool {
        unsafe { jsbsim_do_trim(self.inner, mode) }
    }

    // ── Properties ──────────────────────────────────────────────────

    /// Read a property value by path (e.g., `"position/h-agl-ft"`).
    pub fn get_property(&self, name: &str) -> f64 {
        let c_name = CString::new(name).expect("property name contains null byte");
        unsafe { jsbsim_get_property(self.inner, c_name.as_ptr()) }
    }

    /// Set a property value by path (e.g., `"fcs/throttle-cmd-norm"`, `0.8`).
    pub fn set_property(&mut self, name: &str, value: f64) -> bool {
        let c_name = CString::new(name).expect("property name contains null byte");
        unsafe { jsbsim_set_property(self.inner, c_name.as_ptr(), value) }
    }

    /// Check if a property node exists in the property tree.
    pub fn has_property(&self, name: &str) -> bool {
        let c_name = CString::new(name).expect("property name contains null byte");
        unsafe { jsbsim_has_property(self.inner, c_name.as_ptr()) }
    }

    // ── Property catalog ────────────────────────────────────────────

    /// Search the property catalog for entries matching `check`.
    ///
    /// Returns a newline-separated string of matching property paths.
    pub fn query_property_catalog(&self, check: &str) -> String {
        let c_check = CString::new(check).expect("check string contains null byte");
        // First call to get the length.
        let len = unsafe {
            jsbsim_query_property_catalog(
                self.inner,
                c_check.as_ptr(),
                std::ptr::null_mut(),
                0,
            )
        };
        if len <= 0 {
            return String::new();
        }
        let mut buf = vec![0u8; len as usize + 1];
        unsafe {
            jsbsim_query_property_catalog(
                self.inner,
                c_check.as_ptr(),
                buf.as_mut_ptr() as *mut c_char,
                buf.len() as i32,
            );
        }
        let cstr = unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char) };
        cstr.to_string_lossy().into_owned()
    }

    /// Print the full property catalog to stdout.
    pub fn print_property_catalog(&self) {
        unsafe { jsbsim_print_property_catalog(self.inner) }
    }

    // ── Output control ──────────────────────────────────────────────

    /// Add an output directive from an XML file.
    pub fn set_output_directive(&mut self, fname: &str) -> bool {
        let c_fname = CString::new(fname).expect("fname contains null byte");
        unsafe { jsbsim_set_output_directive(self.inner, c_fname.as_ptr()) }
    }

    /// Enable simulation data output.
    pub fn enable_output(&mut self) {
        unsafe { jsbsim_enable_output(self.inner) }
    }

    /// Disable simulation data output.
    pub fn disable_output(&mut self) {
        unsafe { jsbsim_disable_output(self.inner) }
    }

    /// Set the filename for output channel `n`.
    pub fn set_output_filename(&mut self, n: i32, fname: &str) -> bool {
        let c_fname = CString::new(fname).expect("fname contains null byte");
        unsafe { jsbsim_set_output_filename(self.inner, n, c_fname.as_ptr()) }
    }

    // ── Path configuration ──────────────────────────────────────────

    /// Set the aircraft directory path.
    pub fn set_aircraft_path(&mut self, path: &str) -> bool {
        let c_path = CString::new(path).expect("path contains null byte");
        unsafe { jsbsim_set_aircraft_path(self.inner, c_path.as_ptr()) }
    }

    /// Set the engine directory path.
    pub fn set_engine_path(&mut self, path: &str) -> bool {
        let c_path = CString::new(path).expect("path contains null byte");
        unsafe { jsbsim_set_engine_path(self.inner, c_path.as_ptr()) }
    }

    /// Set the systems directory path.
    pub fn set_systems_path(&mut self, path: &str) -> bool {
        let c_path = CString::new(path).expect("path contains null byte");
        unsafe { jsbsim_set_systems_path(self.inner, c_path.as_ptr()) }
    }

    // ── Info / Debug ────────────────────────────────────────────────

    /// Set the JSBSim debug output level (0 = silent, higher = more verbose).
    pub fn set_debug_level(&mut self, level: i32) {
        unsafe { jsbsim_set_debug_level(self.inner, level) }
    }

    /// Get the name of the currently loaded model (e.g., `"c172x"`).
    pub fn get_model_name(&self) -> String {
        read_c_string(|buf, len| unsafe { jsbsim_get_model_name(self.inner, buf, len) })
    }

    /// Get the JSBSim library version string.
    pub fn get_version() -> String {
        read_c_string(|buf, len| unsafe { jsbsim_get_version(buf, len) })
    }
}

impl Drop for Sim {
    fn drop(&mut self) {
        unsafe {
            jsbsim_destroy(self.inner);
        }
    }
}

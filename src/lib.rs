use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[allow(non_camel_case_types)]
type JSBSim_FGFDMExec = *mut std::ffi::c_void;

/// C function-pointer type matching `jsbsim_get_agl_fn_t` in the wrapper.
type GetAglFn = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    time: f64,
    location: *const f64,
    contact: *mut f64,
    normal: *mut f64,
    velocity: *mut f64,
    ang_velocity: *mut f64,
) -> f64;

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
    fn jsbsim_integration_suspended(fdm: JSBSim_FGFDMExec) -> bool;

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

    // Simulation time setter
    fn jsbsim_set_sim_time(fdm: JSBSim_FGFDMExec, time: f64);

    // Path getters
    fn jsbsim_get_root_dir(fdm: JSBSim_FGFDMExec, buf: *mut c_char, buf_len: i32) -> i32;
    fn jsbsim_get_aircraft_path(fdm: JSBSim_FGFDMExec, buf: *mut c_char, buf_len: i32) -> i32;
    fn jsbsim_get_engine_path(fdm: JSBSim_FGFDMExec, buf: *mut c_char, buf_len: i32) -> i32;
    fn jsbsim_get_systems_path(fdm: JSBSim_FGFDMExec, buf: *mut c_char, buf_len: i32) -> i32;

    // Output filename getter
    fn jsbsim_get_output_filename(fdm: JSBSim_FGFDMExec, n: i32, buf: *mut c_char, buf_len: i32) -> i32;

    // Info / Debug
    fn jsbsim_set_debug_level(fdm: JSBSim_FGFDMExec, level: i32);
    fn jsbsim_get_model_name(fdm: JSBSim_FGFDMExec, buf: *mut c_char, buf_len: i32) -> i32;
    fn jsbsim_get_version(buf: *mut c_char, buf_len: i32) -> i32;

    // Ground callback
    fn jsbsim_set_ground_callback(
        fdm: JSBSim_FGFDMExec,
        get_agl: GetAglFn,
        user_data: *mut std::ffi::c_void,
    );
    fn jsbsim_set_terrain_elevation(fdm: JSBSim_FGFDMExec, elevation_ft: f64);
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

// ── Ground callback types ───────────────────────────────────────────

/// Ground contact data returned by [`GroundCallback::get_agl`].
///
/// All coordinates use JSBSim's Earth-Centered Earth-Fixed (ECEF) frame.
/// Distances are in **feet**, angles in **radians**.
#[derive(Debug, Clone, Copy)]
pub struct GroundContact {
    /// Altitude above ground level (ft).
    pub agl: f64,
    /// Contact point on the terrain surface in ECEF `[x, y, z]` (ft).
    pub contact: [f64; 3],
    /// Unit surface-normal vector at the contact point `[x, y, z]`.
    pub normal: [f64; 3],
    /// Linear velocity of the terrain surface at the contact point `[x, y, z]`
    /// (ft/s).  Usually `[0, 0, 0]` for static terrain.
    pub velocity: [f64; 3],
    /// Angular velocity of the terrain surface at the contact point
    /// `[x, y, z]` (rad/s).  Usually `[0, 0, 0]` for static terrain.
    pub ang_velocity: [f64; 3],
}

/// Trait for providing custom terrain / ground interaction to JSBSim.
///
/// Implement this trait and install it with [`Sim::set_ground_callback`] to
/// feed JSBSim elevation data from your own terrain engine (e.g. heightmaps,
/// mesh terrain, planetary models, etc.).
///
/// # Coordinate frame
///
/// All positions are in JSBSim's Earth-Centered Earth-Fixed (ECEF) frame with
/// distances measured in **feet**.
///
/// # Example
///
/// ```rust,no_run
/// use jsbsim_ffi::{GroundCallback, GroundContact};
///
/// struct FlatEarth {
///     elevation_ft: f64,
/// }
///
/// impl GroundCallback for FlatEarth {
///     fn get_agl(&self, _time: f64, location: [f64; 3]) -> GroundContact {
///         // Simple flat ground at the configured elevation.
///         let radius = (location[0].powi(2)
///                     + location[1].powi(2)
///                     + location[2].powi(2))
///             .sqrt();
///         let earth_radius_ft = 20_925_646.0; // ≈ 6371 km in feet
///         let agl = radius - earth_radius_ft - self.elevation_ft;
///
///         // Contact point: same direction, at ground level.
///         let scale = (earth_radius_ft + self.elevation_ft) / radius;
///         GroundContact {
///             agl,
///             contact: [location[0] * scale,
///                       location[1] * scale,
///                       location[2] * scale],
///             normal:  [location[0] / radius,
///                       location[1] / radius,
///                       location[2] / radius],
///             velocity:     [0.0, 0.0, 0.0],
///             ang_velocity: [0.0, 0.0, 0.0],
///         }
///     }
/// }
/// ```
pub trait GroundCallback {
    /// Compute terrain contact information for a given query position.
    ///
    /// # Arguments
    /// * `time`     – simulation time (s).
    /// * `location` – query position in ECEF `[x, y, z]` (ft).
    ///
    /// # Returns
    /// A [`GroundContact`] describing the terrain surface below `location`.
    fn get_agl(&self, time: f64, location: [f64; 3]) -> GroundContact;
}

/// `extern "C"` trampoline: called by the C++ `FFIGroundCallback` bridge,
/// dispatches into the Rust trait object stored at `user_data`.
///
/// # Safety
///
/// `user_data` must be a valid `*const Box<dyn GroundCallback>` whose
/// pointee is alive for the duration of the call.
unsafe extern "C" fn ground_callback_trampoline(
    user_data: *mut std::ffi::c_void,
    time: f64,
    location: *const f64,
    contact: *mut f64,
    normal: *mut f64,
    velocity: *mut f64,
    ang_velocity: *mut f64,
) -> f64 {
    let cb = &*(user_data as *const Box<dyn GroundCallback>);
    let loc = [*location, *location.add(1), *location.add(2)];

    let result = cb.get_agl(time, loc);

    std::ptr::copy_nonoverlapping(result.contact.as_ptr(), contact, 3);
    std::ptr::copy_nonoverlapping(result.normal.as_ptr(), normal, 3);
    std::ptr::copy_nonoverlapping(result.velocity.as_ptr(), velocity, 3);
    std::ptr::copy_nonoverlapping(result.ang_velocity.as_ptr(), ang_velocity, 3);

    result.agl
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
    /// Heap-stable storage for the Rust ground callback trait object.
    ///
    /// The C++ `FFIGroundCallback` bridge holds a raw pointer (`user_data`)
    /// that points to the inner `Box<dyn GroundCallback>`.  Because this
    /// is behind **two** layers of `Box`, the inner pointer is on the heap
    /// and remains stable even if `Sim` is moved.
    ///
    /// Lifetime invariant: `jsbsim_destroy` (which tears down the C++ bridge)
    /// is called in `Drop::drop` *before* this field is dropped.
    _ground_callback: Option<Box<Box<dyn GroundCallback>>>,
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
        Self {
            inner,
            _ground_callback: None,
        }
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

    /// Set the simulation time directly (seconds).
    pub fn set_sim_time(&mut self, time: f64) {
        unsafe { jsbsim_set_sim_time(self.inner, time) }
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

    /// Returns `true` if physics integration is currently suspended.
    pub fn integration_suspended(&self) -> bool {
        unsafe { jsbsim_integration_suspended(self.inner) }
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

    /// Get the filename for output channel `n`.
    pub fn get_output_filename(&self, n: i32) -> String {
        read_c_string(|buf, len| unsafe { jsbsim_get_output_filename(self.inner, n, buf, len) })
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

    /// Get the root directory path.
    pub fn get_root_dir(&self) -> String {
        read_c_string(|buf, len| unsafe { jsbsim_get_root_dir(self.inner, buf, len) })
    }

    /// Get the aircraft directory path.
    pub fn get_aircraft_path(&self) -> String {
        read_c_string(|buf, len| unsafe { jsbsim_get_aircraft_path(self.inner, buf, len) })
    }

    /// Get the engine directory path.
    pub fn get_engine_path(&self) -> String {
        read_c_string(|buf, len| unsafe { jsbsim_get_engine_path(self.inner, buf, len) })
    }

    /// Get the systems directory path.
    pub fn get_systems_path(&self) -> String {
        read_c_string(|buf, len| unsafe { jsbsim_get_systems_path(self.inner, buf, len) })
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

    // ── Ground callback ─────────────────────────────────────────────

    /// Install a custom ground callback.
    ///
    /// The callback will be invoked by JSBSim every time it needs terrain
    /// information (ground elevation, surface normal, etc.).  This replaces
    /// any previously installed callback (including the built-in default).
    ///
    /// The `Sim` takes ownership of the callback and keeps it alive until
    /// another callback is installed, or the `Sim` is dropped.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use jsbsim_ffi::{Sim, GroundCallback, GroundContact};
    /// struct Flat;
    /// impl GroundCallback for Flat {
    ///     fn get_agl(&self, _t: f64, loc: [f64; 3]) -> GroundContact {
    ///         let r = (loc[0]*loc[0] + loc[1]*loc[1] + loc[2]*loc[2]).sqrt();
    ///         let ground_r = 20_925_646.0; // earth radius in ft
    ///         let s = ground_r / r;
    ///         GroundContact {
    ///             agl: r - ground_r,
    ///             contact: [loc[0]*s, loc[1]*s, loc[2]*s],
    ///             normal:  [loc[0]/r, loc[1]/r, loc[2]/r],
    ///             velocity: [0.0; 3],
    ///             ang_velocity: [0.0; 3],
    ///         }
    ///     }
    /// }
    ///
    /// let mut sim = Sim::new("/path/to/jsbsim");
    /// sim.set_ground_callback(Flat);
    /// ```
    pub fn set_ground_callback<C: GroundCallback + 'static>(&mut self, callback: C) {
        // Double-box: outer Box for heap-stable address, inner Box<dyn> for trait dispatch.
        let inner: Box<dyn GroundCallback> = Box::new(callback);
        let outer = Box::new(inner);

        // The raw pointer to the *inner* Box<dyn GroundCallback> is heap-allocated
        // inside `outer`, so it survives moves of `self`.
        let user_data: *mut std::ffi::c_void =
            &*outer as *const Box<dyn GroundCallback> as *const () as *mut std::ffi::c_void;

        unsafe {
            jsbsim_set_ground_callback(self.inner, ground_callback_trampoline, user_data);
        }

        // Store the outer box to keep the callback alive.
        // Any previously installed callback is dropped here.
        self._ground_callback = Some(outer);
    }

    /// Set the terrain elevation (ft MSL) on the **current** ground callback.
    ///
    /// This is a convenience method that adjusts the built-in default
    /// (sphere-earth) ground callback's terrain height.  It has no effect if
    /// a custom [`GroundCallback`] that ignores `SetTerrainElevation` is
    /// installed.
    pub fn set_terrain_elevation(&mut self, elevation_ft: f64) {
        unsafe { jsbsim_set_terrain_elevation(self.inner, elevation_ft) }
    }
}

// JSBSim's C++ internals are not thread-safe.
// `Sim` is inherently `!Send` and `!Sync` because `inner` is a raw pointer
// (`*mut c_void`), which does not implement `Send` or `Sync`.
// This is the correct behavior — do not add unsafe impls for these traits.

impl Drop for Sim {
    fn drop(&mut self) {
        // Destroy the C++ FDM first – this tears down the FFIGroundCallback
        // bridge that holds a raw pointer into `_ground_callback`.
        unsafe {
            jsbsim_destroy(self.inner);
        }
        // `_ground_callback` is dropped automatically after this, which is
        // safe because the C++ side no longer references it.
    }
}

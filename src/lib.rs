//! Safe Rust bindings for [JSBSim](https://github.com/JSBSim-Team/jsbsim),
//! an open-source flight dynamics model.
//!
//! This crate wraps JSBSim's C++ `FGFDMExec` class
//! ([src/FGFDMExec.h](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h))
//! via a thin C shim (`c_wrapper/jsbsim_wrapper.cpp`), exposing a safe Rust API
//! through the [`Sim`] struct.
//!
//! # JSBSim C++ traceability
//!
//! Every public item in this crate documents which JSBSim C++ class or method
//! drove its creation.  Source references use the path within the
//! [JSBSim repository](https://github.com/JSBSim-Team/jsbsim/tree/master/src).
//!
//! # Usage
//!
//! ```sh
//! JSBSIM_ROOT=/path/to/jsbsim cargo run --example simple
//! ```

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[allow(non_camel_case_types)]
type JSBSim_FGFDMExec = *mut std::ffi::c_void;

/// C function-pointer type matching `jsbsim_get_agl_fn_t` in the wrapper.
///
/// **JSBSim C++ origin:** mirrors the signature of
/// [`FGGroundCallback::GetAGLevel`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/input_output/FGGroundCallback.h).
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
    fn jsbsim_load_model_ex(
        fdm: JSBSim_FGFDMExec,
        aircraft_path: *const c_char,
        engine_path: *const c_char,
        systems_path: *const c_char,
        model: *const c_char,
        add_model_to_path: bool,
    ) -> bool;
    fn jsbsim_load_script(fdm: JSBSim_FGFDMExec, filename: *const c_char) -> bool;
    fn jsbsim_load_script_ex(
        fdm: JSBSim_FGFDMExec,
        filename: *const c_char,
        dt: f64,
        initfile: *const c_char,
    ) -> bool;
    fn jsbsim_load_planet(
        fdm: JSBSim_FGFDMExec,
        filename: *const c_char,
        use_aircraft_path: bool,
    ) -> bool;
    fn jsbsim_load_ic(
        fdm: JSBSim_FGFDMExec,
        filename: *const c_char,
        use_aircraft_path: bool,
    ) -> bool;

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
    fn jsbsim_do_linearization(fdm: JSBSim_FGFDMExec, mode: i32) -> bool;
    fn jsbsim_set_trim_status(fdm: JSBSim_FGFDMExec, status: bool);
    fn jsbsim_get_trim_status(fdm: JSBSim_FGFDMExec) -> bool;
    fn jsbsim_set_trim_mode(fdm: JSBSim_FGFDMExec, mode: i32);
    fn jsbsim_get_trim_mode(fdm: JSBSim_FGFDMExec) -> i32;

    // Reports / child FDMs / seed
    fn jsbsim_print_simulation_configuration(fdm: JSBSim_FGFDMExec);
    fn jsbsim_get_property_catalog_size(fdm: JSBSim_FGFDMExec) -> i32;
    fn jsbsim_get_property_catalog_entry(
        fdm: JSBSim_FGFDMExec,
        i: i32,
        buf: *mut c_char,
        buf_len: i32,
    ) -> i32;
    fn jsbsim_get_propulsion_tank_report(
        fdm: JSBSim_FGFDMExec,
        buf: *mut c_char,
        buf_len: i32,
    ) -> i32;
    fn jsbsim_get_random_seed(fdm: JSBSim_FGFDMExec) -> i32;
    fn jsbsim_get_fdm_count(fdm: JSBSim_FGFDMExec) -> i32;
    fn jsbsim_enumerate_fdms_count(fdm: JSBSim_FGFDMExec) -> i32;
    fn jsbsim_enumerate_fdms_name(
        fdm: JSBSim_FGFDMExec,
        i: i32,
        buf: *mut c_char,
        buf_len: i32,
    ) -> i32;
    fn jsbsim_set_child(fdm: JSBSim_FGFDMExec, is_child: bool);

    // Propulsion helpers
    fn jsbsim_get_num_engines(fdm: JSBSim_FGFDMExec) -> i32;
    fn jsbsim_get_num_tanks(fdm: JSBSim_FGFDMExec) -> i32;
    fn jsbsim_init_running(fdm: JSBSim_FGFDMExec, n: i32) -> bool;
    fn jsbsim_propulsion_get_steady_state(fdm: JSBSim_FGFDMExec) -> bool;

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
    fn jsbsim_force_output(fdm: JSBSim_FGFDMExec, idx: i32);
    fn jsbsim_set_logging_rate(fdm: JSBSim_FGFDMExec, rate_hz: f64);

    // Path configuration
    fn jsbsim_set_aircraft_path(fdm: JSBSim_FGFDMExec, path: *const c_char) -> bool;
    fn jsbsim_set_engine_path(fdm: JSBSim_FGFDMExec, path: *const c_char) -> bool;
    fn jsbsim_set_systems_path(fdm: JSBSim_FGFDMExec, path: *const c_char) -> bool;
    fn jsbsim_set_output_path(fdm: JSBSim_FGFDMExec, path: *const c_char) -> bool;
    fn jsbsim_set_root_dir(fdm: JSBSim_FGFDMExec, path: *const c_char);

    // Simulation time setter
    fn jsbsim_set_sim_time(fdm: JSBSim_FGFDMExec, time: f64);
    fn jsbsim_incr_time(fdm: JSBSim_FGFDMExec) -> f64;
    fn jsbsim_get_frame(fdm: JSBSim_FGFDMExec) -> u32;
    fn jsbsim_get_debug_level(fdm: JSBSim_FGFDMExec) -> i32;
    fn jsbsim_set_hold_down(fdm: JSBSim_FGFDMExec, hold_down: bool);
    fn jsbsim_get_hold_down(fdm: JSBSim_FGFDMExec) -> bool;

    // Path getters
    fn jsbsim_get_root_dir(fdm: JSBSim_FGFDMExec, buf: *mut c_char, buf_len: i32) -> i32;
    fn jsbsim_get_aircraft_path(fdm: JSBSim_FGFDMExec, buf: *mut c_char, buf_len: i32) -> i32;
    fn jsbsim_get_engine_path(fdm: JSBSim_FGFDMExec, buf: *mut c_char, buf_len: i32) -> i32;
    fn jsbsim_get_systems_path(fdm: JSBSim_FGFDMExec, buf: *mut c_char, buf_len: i32) -> i32;
    fn jsbsim_get_output_path(fdm: JSBSim_FGFDMExec, buf: *mut c_char, buf_len: i32) -> i32;
    fn jsbsim_get_full_aircraft_path(fdm: JSBSim_FGFDMExec, buf: *mut c_char, buf_len: i32) -> i32;

    // Output filename getter
    fn jsbsim_get_output_filename(
        fdm: JSBSim_FGFDMExec,
        n: i32,
        buf: *mut c_char,
        buf_len: i32,
    ) -> i32;

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
///
/// **JSBSim C++ origin:** corresponds to the output parameters of
/// [`FGGroundCallback::GetAGLevel()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/input_output/FGGroundCallback.h).
/// The C++ method populates `FGLocation& contact`, `FGColumnVector3& normal`,
/// `FGColumnVector3& v` (velocity), and `FGColumnVector3& w` (angular velocity)
/// and returns the AGL altitude as `double`.  This struct bundles all those
/// outputs into a single Rust value.
#[derive(Debug, Clone, Copy)]
pub struct GroundContact {
    /// Altitude above ground level (ft).
    pub agl: f64,
    /// Contact point on the terrain surface in ECEF `[x, y, z]` (ft).
    ///
    /// **C++ origin:** `FGLocation& contact` parameter of `GetAGLevel()`.
    pub contact: [f64; 3],
    /// Unit surface-normal vector at the contact point `[x, y, z]`.
    ///
    /// **C++ origin:** `FGColumnVector3& normal` parameter of `GetAGLevel()`.
    pub normal: [f64; 3],
    /// Linear velocity of the terrain surface at the contact point `[x, y, z]`
    /// (ft/s).  Usually `[0, 0, 0]` for static terrain.
    ///
    /// **C++ origin:** `FGColumnVector3& v` parameter of `GetAGLevel()`.
    pub velocity: [f64; 3],
    /// Angular velocity of the terrain surface at the contact point
    /// `[x, y, z]` (rad/s).  Usually `[0, 0, 0]` for static terrain.
    ///
    /// **C++ origin:** `FGColumnVector3& w` parameter of `GetAGLevel()`.
    pub ang_velocity: [f64; 3],
}

/// Trait for providing custom terrain / ground interaction to JSBSim.
///
/// Implement this trait and install it with [`Sim::set_ground_callback`] to
/// feed JSBSim elevation data from your own terrain engine (e.g. heightmaps,
/// mesh terrain, planetary models, etc.).
///
/// **JSBSim C++ origin:** abstracts
/// [`JSBSim::FGGroundCallback`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/input_output/FGGroundCallback.h)
/// — an abstract base class with the pure virtual method `GetAGLevel()`.
/// The C++ class is defined in `src/input_output/FGGroundCallback.h` and the
/// default implementation lives in `src/models/FGInertial.cpp`.  Our C wrapper
/// (`FFIGroundCallback` in `jsbsim_wrapper.cpp`) inherits `FGGroundCallback`
/// and delegates to a C function pointer, which in turn trampolines into this
/// Rust trait.
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
    /// **JSBSim C++ origin:**
    /// [`FGGroundCallback::GetAGLevel()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/input_output/FGGroundCallback.h)
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
///
/// **JSBSim C++ origin:** mirrors the `tType` enum from
/// [`src/math/FGTrimmer.h`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/math/FGTrimmer.h),
/// passed to [`FGFDMExec::DoTrim(int)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h).
pub mod trim {
    /// Longitudinal trim (pitch + throttle).
    ///
    /// **C++ value:** `tLongitudinal = 0`
    pub const LONGITUDINAL: i32 = 0;
    /// Full trim (all axes).
    ///
    /// **C++ value:** `tFull = 1`
    pub const FULL: i32 = 1;
    /// Ground trim.
    ///
    /// **C++ value:** `tGround = 2`
    pub const GROUND: i32 = 2;
    /// Pullup trim.
    ///
    /// **C++ value:** `tPullup = 3`
    pub const PULLUP: i32 = 3;
    /// Custom trim.
    ///
    /// **C++ value:** `tCustom = 4`
    pub const CUSTOM: i32 = 4;
    /// Turn trim.
    ///
    /// **C++ value:** `tTurn = 5`
    pub const TURN: i32 = 5;
}

/// Safe wrapper around JSBSim's
/// [`FGFDMExec`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
/// — the top-level flight dynamics model executive.
///
/// **JSBSim C++ origin:**
/// [`JSBSim::FGFDMExec`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
/// is the central class that owns all JSBSim subsystem models (atmosphere,
/// propulsion, aerodynamics, FCS, propagate, etc.) and orchestrates the
/// simulation loop.  This struct holds an opaque pointer to an `FGFDMExec`
/// instance created by the C wrapper.
///
/// `Sim` is `!Send` and `!Sync` because JSBSim's C++ internals are not
/// thread-safe.
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
    ///
    /// **JSBSim C++ origin:**
    /// - [`FGFDMExec::FGFDMExec()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h) — constructor
    /// - [`FGFDMExec::SetRootDir(SGPath)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    /// - [`FGFDMExec::SetAircraftPath(SGPath)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    /// - [`FGFDMExec::SetEnginePath(SGPath)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    /// - [`FGFDMExec::SetSystemsPath(SGPath)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    ///
    /// The C wrapper calls the constructor then sets root, aircraft, engine,
    /// and systems paths from the provided `root_dir`.
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
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::LoadModel(const string& model, bool addModelToPath)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    /// — the single-argument overload that searches the aircraft path.
    pub fn load_model(&mut self, model: &str) -> bool {
        let c_model = CString::new(model).expect("model name contains null byte");
        unsafe { jsbsim_load_model(self.inner, c_model.as_ptr()) }
    }

    /// Load an aircraft model and re-target the aircraft / engine / systems
    /// search paths in a single call.  Equivalent to calling
    /// [`set_aircraft_path`](Self::set_aircraft_path),
    /// [`set_engine_path`](Self::set_engine_path), and
    /// [`set_systems_path`](Self::set_systems_path) followed by
    /// [`load_model`](Self::load_model), but routed through JSBSim's own
    /// 5-argument `LoadModel` overload.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::LoadModel(const SGPath& AircraftPath, const SGPath& EnginePath,
    /// const SGPath& SystemsPath, const string& model, bool addModelToPath)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn load_model_with(
        &mut self,
        aircraft_path: &str,
        engine_path: &str,
        systems_path: &str,
        model: &str,
        add_model_to_path: bool,
    ) -> bool {
        let c_ap = CString::new(aircraft_path).expect("aircraft_path contains null byte");
        let c_ep = CString::new(engine_path).expect("engine_path contains null byte");
        let c_sp = CString::new(systems_path).expect("systems_path contains null byte");
        let c_model = CString::new(model).expect("model name contains null byte");
        unsafe {
            jsbsim_load_model_ex(
                self.inner,
                c_ap.as_ptr(),
                c_ep.as_ptr(),
                c_sp.as_ptr(),
                c_model.as_ptr(),
                add_model_to_path,
            )
        }
    }

    /// Load a JSBSim script XML file.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::LoadScript(const SGPath& Script, double deltaT, const SGPath& initfile)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    /// — loads an XML scenario script that specifies the aircraft, ICs, and
    /// event-driven commands.
    pub fn load_script(&mut self, filename: &str) -> bool {
        let c_file = CString::new(filename).expect("filename contains null byte");
        unsafe { jsbsim_load_script(self.inner, c_file.as_ptr()) }
    }

    /// Load a JSBSim script XML file with overrides for the integration time
    /// step and the initial-conditions file.
    ///
    /// `dt = 0.0` keeps the script's own time step. `initfile = None` keeps
    /// the IC referenced inside the script.
    ///
    /// **JSBSim C++ origin:** the three-argument overload of
    /// [`FGFDMExec::LoadScript`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h).
    pub fn load_script_with(&mut self, filename: &str, dt: f64, initfile: Option<&str>) -> bool {
        let c_file = CString::new(filename).expect("filename contains null byte");
        let c_init = initfile.map(|s| CString::new(s).expect("initfile contains null byte"));
        let init_ptr = c_init
            .as_ref()
            .map(|s| s.as_ptr())
            .unwrap_or(std::ptr::null());
        unsafe { jsbsim_load_script_ex(self.inner, c_file.as_ptr(), dt, init_ptr) }
    }

    /// Load a planet definition XML file.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::LoadPlanet(const SGPath& PlanetPath, bool useAircraftPath)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn load_planet(&mut self, filename: &str, use_aircraft_path: bool) -> bool {
        let c_file = CString::new(filename).expect("filename contains null byte");
        unsafe { jsbsim_load_planet(self.inner, c_file.as_ptr(), use_aircraft_path) }
    }

    /// Load initial conditions from an XML file.
    ///
    /// If `use_aircraft_path` is true, the path is relative to the aircraft directory.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGInitialCondition::Load(const SGPath&, bool)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/initialization/FGInitialCondition.h)
    /// — accessed via `FGFDMExec::GetIC()->Load(...)`.
    pub fn load_ic(&mut self, filename: &str, use_aircraft_path: bool) -> bool {
        let c_file = CString::new(filename).expect("filename contains null byte");
        unsafe { jsbsim_load_ic(self.inner, c_file.as_ptr(), use_aircraft_path) }
    }

    // ── Simulation control ──────────────────────────────────────────

    /// Initialize and run initial conditions.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::RunIC()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    /// — loops JSBSim once without integrating (dt=0) to propagate initial
    /// conditions through all subsystems.
    pub fn run_ic(&mut self) -> bool {
        unsafe { jsbsim_run_ic(self.inner) }
    }

    /// Advance the simulation by one time step.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::Run()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    /// — executes one iteration of all subsystem `Run()` methods, advancing
    /// simulation time by `dt`.  Returns `false` when the simulation is
    /// terminated (e.g. script end, crash, etc.).
    pub fn run(&mut self) -> bool {
        unsafe { jsbsim_run(self.inner) }
    }

    /// Set the simulation time step in seconds.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::Setdt(double delta_t)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn set_dt(&mut self, dt: f64) {
        unsafe { jsbsim_set_dt(self.inner, dt) }
    }

    /// Get the current simulation time step in seconds.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::GetDeltaT()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn get_dt(&self) -> f64 {
        unsafe { jsbsim_get_dt(self.inner) }
    }

    /// Get the current simulation time in seconds.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::GetSimTime()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn get_sim_time(&self) -> f64 {
        unsafe { jsbsim_get_sim_time(self.inner) }
    }

    /// Set the simulation time directly (seconds).
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::Setsim_time(double cur_time)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn set_sim_time(&mut self, time: f64) {
        unsafe { jsbsim_set_sim_time(self.inner, time) }
    }

    /// Increment simulation time by `dt` (when not held). Returns the new
    /// simulation time. Also bumps the frame counter.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::IncrTime()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn incr_time(&mut self) -> f64 {
        unsafe { jsbsim_incr_time(self.inner) }
    }

    /// Get the current frame counter.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::GetFrame()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn get_frame(&self) -> u32 {
        unsafe { jsbsim_get_frame(self.inner) }
    }

    /// Get the current debug level.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::GetDebugLevel()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn get_debug_level(&self) -> i32 {
        unsafe { jsbsim_get_debug_level(self.inner) }
    }

    /// Enable or disable the hold-down flag (`forces/hold-down`).
    ///
    /// Used for hard hold-downs such as rockets on a launch pad with engines
    /// ignited but the vehicle prevented from moving.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::SetHoldDown(bool)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn set_hold_down(&mut self, hold_down: bool) {
        unsafe { jsbsim_set_hold_down(self.inner, hold_down) }
    }

    /// Query the hold-down flag.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::GetHoldDown()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn get_hold_down(&self) -> bool {
        unsafe { jsbsim_get_hold_down(self.inner) }
    }

    /// Reset the simulation to its initial conditions.
    ///
    /// `mode`: 0 = reset state only, 1 = also reload the model.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::ResetToInitialConditions(int mode)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    /// — mode bits: `START_NEW_OUTPUT = 0x1`, `DONT_EXECUTE_RUN_IC = 0x2`.
    pub fn reset_to_initial_conditions(&mut self, mode: i32) {
        unsafe { jsbsim_reset_to_initial_conditions(self.inner, mode) }
    }

    // ── Hold / Resume ───────────────────────────────────────────────

    /// Pause the simulation (time stops advancing).
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::Hold()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    /// — sets `holding = true`.
    pub fn hold(&mut self) {
        unsafe { jsbsim_hold(self.inner) }
    }

    /// Resume the simulation from a hold.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::Resume()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    /// — sets `holding = false`.
    pub fn resume(&mut self) {
        unsafe { jsbsim_resume(self.inner) }
    }

    /// Returns `true` if the simulation is currently on hold.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::Holding()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn holding(&self) -> bool {
        unsafe { jsbsim_holding(self.inner) }
    }

    /// Run `steps` more time steps, then automatically hold.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::EnableIncrementThenHold(int Timesteps)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn enable_increment_then_hold(&mut self, steps: i32) {
        unsafe { jsbsim_enable_increment_then_hold(self.inner, steps) }
    }

    /// Check and process an incremental-hold request (called internally by `run()`).
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::CheckIncrementalHold()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn check_incremental_hold(&mut self) {
        unsafe { jsbsim_check_incremental_hold(self.inner) }
    }

    // ── Integration suspend ─────────────────────────────────────────

    /// Freeze physics integration (dt becomes 0) while still calling `run()`.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::SuspendIntegration()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    /// — saves `dT` and sets it to 0.
    pub fn suspend_integration(&mut self) {
        unsafe { jsbsim_suspend_integration(self.inner) }
    }

    /// Resume physics integration after a suspend.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::ResumeIntegration()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    /// — restores `dT` from `saved_dT`.
    pub fn resume_integration(&mut self) {
        unsafe { jsbsim_resume_integration(self.inner) }
    }

    /// Returns `true` if physics integration is currently suspended.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::IntegrationSuspended()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    /// — returns `dT == 0.0`.
    pub fn integration_suspended(&self) -> bool {
        unsafe { jsbsim_integration_suspended(self.inner) }
    }

    // ── Trim ────────────────────────────────────────────────────────

    /// Trim the aircraft. Returns `true` on success.
    ///
    /// Use constants from [`trim`] module: `trim::LONGITUDINAL`, `trim::FULL`,
    /// `trim::GROUND`, etc.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::DoTrim(int mode)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    /// — creates an `FGTrim` object and runs the trim algorithm.  The mode
    /// values correspond to the `tType` enum.
    pub fn do_trim(&mut self, mode: i32) -> bool {
        unsafe { jsbsim_do_trim(self.inner, mode) }
    }

    /// Set the trim status flag.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::SetTrimStatus(bool)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn set_trim_status(&mut self, status: bool) {
        unsafe { jsbsim_set_trim_status(self.inner, status) }
    }

    /// Get the trim status flag.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::GetTrimStatus()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn get_trim_status(&self) -> bool {
        unsafe { jsbsim_get_trim_status(self.inner) }
    }

    /// Set the stored trim mode (does not invoke the trimmer).
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::SetTrimMode(int)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn set_trim_mode(&mut self, mode: i32) {
        unsafe { jsbsim_set_trim_mode(self.inner, mode) }
    }

    /// Get the stored trim mode.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::GetTrimMode()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn get_trim_mode(&self) -> i32 {
        unsafe { jsbsim_get_trim_mode(self.inner) }
    }

    /// Run the linearization algorithm with the given mode.
    ///
    /// The aircraft must be trimmed first for the resulting state-space
    /// model to be meaningful.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::DoLinearization(int)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn do_linearization(&mut self, mode: i32) -> bool {
        unsafe { jsbsim_do_linearization(self.inner, mode) }
    }

    // ── Reports / child FDMs / seed ─────────────────────────────────

    /// Return the propulsion tank status report.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::GetPropulsionTankReport()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn get_propulsion_tank_report(&self) -> String {
        read_c_string(|buf, len| unsafe { jsbsim_get_propulsion_tank_report(self.inner, buf, len) })
    }

    /// Get the current random seed used for stochastic models.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::SRand() const`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn get_random_seed(&self) -> i32 {
        unsafe { jsbsim_get_random_seed(self.inner) }
    }

    /// Get the number of child FDMs attached to this executive.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::GetFDMCount()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn get_fdm_count(&self) -> i32 {
        unsafe { jsbsim_get_fdm_count(self.inner) }
    }

    /// Enumerate FDM names returned by `FGFDMExec::EnumerateFDMs()`.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::EnumerateFDMs()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn enumerate_fdms(&self) -> Vec<String> {
        let n = unsafe { jsbsim_enumerate_fdms_count(self.inner) };
        if n <= 0 {
            return Vec::new();
        }
        let mut out = Vec::with_capacity(n as usize);
        for i in 0..n {
            out.push(read_c_string(|buf, len| unsafe {
                jsbsim_enumerate_fdms_name(self.inner, i, buf, len)
            }));
        }
        out
    }

    /// Mark this FDM instance as a child of another executive.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::SetChild(bool)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn set_child(&mut self, is_child: bool) {
        unsafe { jsbsim_set_child(self.inner, is_child) }
    }

    // ── Propulsion helpers ──────────────────────────────────────────

    /// Get the number of engines defined for the loaded aircraft.
    ///
    /// Returns 0 on a bare (no-model-loaded) sim.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGPropulsion::GetNumEngines()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/models/FGPropulsion.h)
    /// via `FGFDMExec::GetPropulsion()`.
    pub fn get_num_engines(&self) -> i32 {
        unsafe { jsbsim_get_num_engines(self.inner) }
    }

    /// Get the number of fuel tanks defined for the loaded aircraft.
    ///
    /// Returns 0 on a bare (no-model-loaded) sim.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGPropulsion::GetNumTanks()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/models/FGPropulsion.h)
    /// via `FGFDMExec::GetPropulsion()`.
    pub fn get_num_tanks(&self) -> i32 {
        unsafe { jsbsim_get_num_tanks(self.inner) }
    }

    /// Warm-start engines as running, bypassing the cold-start sequence.
    ///
    /// Pass `-1` to start *all* engines at once, or an engine index `>= 0`
    /// to start a specific engine. Returns `false` if no model is loaded
    /// or if the propulsion subsystem refuses the request.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGPropulsion::InitRunning(int)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/models/FGPropulsion.h)
    /// via `FGFDMExec::GetPropulsion()`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use jsbsim_ffi::Sim;
    /// let mut sim = Sim::new("/path/to/jsbsim");
    /// sim.load_model("c172x");
    /// sim.set_property("ic/h-sl-ft", 3000.0);
    /// sim.set_property("ic/vc-kts", 100.0);
    /// sim.run_ic();
    /// sim.init_running(-1);          // start all engines
    /// assert!(sim.get_num_engines() > 0);
    /// ```
    pub fn init_running(&mut self, n: i32) -> bool {
        unsafe { jsbsim_init_running(self.inner, n) }
    }

    /// Iterate the propulsion model until thrust output is steady.  Used
    /// internally by `do_trim` but can be called directly.
    ///
    /// Returns `false` if no model is loaded or the steady-state loop fails.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGPropulsion::GetSteadyState()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/models/FGPropulsion.h)
    /// via `FGFDMExec::GetPropulsion()`.
    pub fn propulsion_get_steady_state(&mut self) -> bool {
        unsafe { jsbsim_propulsion_get_steady_state(self.inner) }
    }

    // ── Properties ──────────────────────────────────────────────────

    /// Read a property value by path (e.g., `"position/h-agl-ft"`).
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::GetPropertyValue(const string& property)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    /// — delegates to the `FGPropertyManager` tree.
    pub fn get_property(&self, name: &str) -> f64 {
        let c_name = CString::new(name).expect("property name contains null byte");
        unsafe { jsbsim_get_property(self.inner, c_name.as_ptr()) }
    }

    /// Set a property value by path (e.g., `"fcs/throttle-cmd-norm"`, `0.8`).
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::SetPropertyValue(const string& property, double value)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    /// — delegates to the `FGPropertyManager` tree.
    pub fn set_property(&mut self, name: &str, value: f64) -> bool {
        let c_name = CString::new(name).expect("property name contains null byte");
        unsafe { jsbsim_set_property(self.inner, c_name.as_ptr(), value) }
    }

    /// Check if a property node exists in the property tree.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGPropertyManager::HasNode(const string&)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/input_output/FGPropertyManager.h)
    /// — accessed via `FGFDMExec::GetPropertyManager()->HasNode(name)`.
    pub fn has_property(&self, name: &str) -> bool {
        let c_name = CString::new(name).expect("property name contains null byte");
        unsafe { jsbsim_has_property(self.inner, c_name.as_ptr()) }
    }

    // ── Property catalog ────────────────────────────────────────────

    /// Search the property catalog for entries matching `check`.
    ///
    /// Returns a newline-separated string of matching property paths.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::QueryPropertyCatalog(const string& check, const string& end_of_line)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn query_property_catalog(&self, check: &str) -> String {
        let c_check = CString::new(check).expect("check string contains null byte");
        // First call to get the length.
        let len = unsafe {
            jsbsim_query_property_catalog(self.inner, c_check.as_ptr(), std::ptr::null_mut(), 0)
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
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::PrintPropertyCatalog()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn print_property_catalog(&self) {
        unsafe { jsbsim_print_property_catalog(self.inner) }
    }

    /// Print the full simulation configuration (models, paths, etc.) to
    /// stdout.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::PrintSimulationConfiguration()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn print_simulation_configuration(&self) {
        unsafe { jsbsim_print_simulation_configuration(self.inner) }
    }

    /// Get the full property catalog as a `Vec<String>`.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::GetPropertyCatalog()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn get_property_catalog(&self) -> Vec<String> {
        let n = unsafe { jsbsim_get_property_catalog_size(self.inner) };
        if n <= 0 {
            return Vec::new();
        }
        let mut out = Vec::with_capacity(n as usize);
        for i in 0..n {
            let entry = read_c_string(|buf, len| unsafe {
                jsbsim_get_property_catalog_entry(self.inner, i, buf, len)
            });
            out.push(entry);
        }
        out
    }

    // ── Output control ──────────────────────────────────────────────

    /// Add an output directive from an XML file.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::SetOutputDirectives(const SGPath& fname)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    /// — loads an XML output directive that configures data logging channels.
    pub fn set_output_directive(&mut self, fname: &str) -> bool {
        let c_fname = CString::new(fname).expect("fname contains null byte");
        unsafe { jsbsim_set_output_directive(self.inner, c_fname.as_ptr()) }
    }

    /// Enable simulation data output.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::EnableOutput()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    /// — calls `FGOutput::Enable()` on the output subsystem
    /// ([`src/models/FGOutput.h`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/models/FGOutput.h)).
    pub fn enable_output(&mut self) {
        unsafe { jsbsim_enable_output(self.inner) }
    }

    /// Disable simulation data output.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::DisableOutput()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    /// — calls `FGOutput::Disable()` on the output subsystem
    /// ([`src/models/FGOutput.h`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/models/FGOutput.h)).
    pub fn disable_output(&mut self) {
        unsafe { jsbsim_disable_output(self.inner) }
    }

    /// Set the filename for output channel `n`.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::SetOutputFileName(int n, const string& fname)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    /// — delegates to `FGOutput::SetOutputName(n, fname)`.
    pub fn set_output_filename(&mut self, n: i32, fname: &str) -> bool {
        let c_fname = CString::new(fname).expect("fname contains null byte");
        unsafe { jsbsim_set_output_filename(self.inner, n, c_fname.as_ptr()) }
    }

    /// Force a single output object to flush its data once.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::ForceOutput(int idx)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn force_output(&mut self, idx: i32) {
        unsafe { jsbsim_force_output(self.inner, idx) }
    }

    /// Set the global logging rate (Hz) for all output objects.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::SetLoggingRate(double rate)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    /// — delegates to `FGOutput::SetRateHz(rate)`.
    pub fn set_logging_rate(&mut self, rate_hz: f64) {
        unsafe { jsbsim_set_logging_rate(self.inner, rate_hz) }
    }

    /// Get the filename for output channel `n`.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::GetOutputFileName(int n)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    /// — delegates to `FGOutput::GetOutputName(n)`.
    pub fn get_output_filename(&self, n: i32) -> String {
        read_c_string(|buf, len| unsafe { jsbsim_get_output_filename(self.inner, n, buf, len) })
    }

    // ── Path configuration ──────────────────────────────────────────

    /// Set the aircraft directory path.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::SetAircraftPath(const SGPath& path)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn set_aircraft_path(&mut self, path: &str) -> bool {
        let c_path = CString::new(path).expect("path contains null byte");
        unsafe { jsbsim_set_aircraft_path(self.inner, c_path.as_ptr()) }
    }

    /// Set the engine directory path.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::SetEnginePath(const SGPath& path)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn set_engine_path(&mut self, path: &str) -> bool {
        let c_path = CString::new(path).expect("path contains null byte");
        unsafe { jsbsim_set_engine_path(self.inner, c_path.as_ptr()) }
    }

    /// Set the systems directory path.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::SetSystemsPath(const SGPath& path)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn set_systems_path(&mut self, path: &str) -> bool {
        let c_path = CString::new(path).expect("path contains null byte");
        unsafe { jsbsim_set_systems_path(self.inner, c_path.as_ptr()) }
    }

    /// Set the directory where output files will be written.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::SetOutputPath(const SGPath& path)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn set_output_path(&mut self, path: &str) -> bool {
        let c_path = CString::new(path).expect("path contains null byte");
        unsafe { jsbsim_set_output_path(self.inner, c_path.as_ptr()) }
    }

    /// Set the root directory used to resolve relative paths.
    ///
    /// **Note:** SetRootDir does NOT update the aircraft / engine / systems
    /// / output paths.  If you need them re-rooted as well, call the
    /// corresponding setters separately.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::SetRootDir(const SGPath& rootDir)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn set_root_dir(&mut self, path: &str) {
        let c_path = CString::new(path).expect("path contains null byte");
        unsafe { jsbsim_set_root_dir(self.inner, c_path.as_ptr()) }
    }

    /// Get the root directory path.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::GetRootDir()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn get_root_dir(&self) -> String {
        read_c_string(|buf, len| unsafe { jsbsim_get_root_dir(self.inner, buf, len) })
    }

    /// Get the aircraft directory path.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::GetAircraftPath()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn get_aircraft_path(&self) -> String {
        read_c_string(|buf, len| unsafe { jsbsim_get_aircraft_path(self.inner, buf, len) })
    }

    /// Get the engine directory path.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::GetEnginePath()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn get_engine_path(&self) -> String {
        read_c_string(|buf, len| unsafe { jsbsim_get_engine_path(self.inner, buf, len) })
    }

    /// Get the systems directory path.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::GetSystemsPath()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn get_systems_path(&self) -> String {
        read_c_string(|buf, len| unsafe { jsbsim_get_systems_path(self.inner, buf, len) })
    }

    /// Get the output directory path.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::GetOutputPath()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn get_output_path(&self) -> String {
        read_c_string(|buf, len| unsafe { jsbsim_get_output_path(self.inner, buf, len) })
    }

    /// Get the fully-resolved aircraft path (root + aircraft + model).
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::GetFullAircraftPath()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn get_full_aircraft_path(&self) -> String {
        read_c_string(|buf, len| unsafe { jsbsim_get_full_aircraft_path(self.inner, buf, len) })
    }

    // ── Info / Debug ────────────────────────────────────────────────

    /// Set the JSBSim debug output level (0 = silent, higher = more verbose).
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::SetDebugLevel(int level)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn set_debug_level(&mut self, level: i32) {
        unsafe { jsbsim_set_debug_level(self.inner, level) }
    }

    /// Get the name of the currently loaded model (e.g., `"c172x"`).
    ///
    /// **JSBSim C++ origin:**
    /// [`FGFDMExec::GetModelName()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)
    pub fn get_model_name(&self) -> String {
        read_c_string(|buf, len| unsafe { jsbsim_get_model_name(self.inner, buf, len) })
    }

    /// Get the JSBSim library version string.
    ///
    /// **JSBSim C++ origin:**
    /// [`FGJSBBase::GetVersion()`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGJSBBase.h)
    /// — static method on the base class from which all JSBSim classes inherit.
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
    /// **JSBSim C++ origin:**
    /// [`FGInertial::SetGroundCallback(FGGroundCallback*)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/models/FGInertial.h)
    /// — accessed via `FGFDMExec::GetInertial()->SetGroundCallback(...)`.
    /// The default ground callback is a simple sphere-earth model in
    /// [`FGDefaultGroundCallback`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/input_output/FGGroundCallback.h).
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
    ///
    /// **JSBSim C++ origin:**
    /// [`FGInertial::SetTerrainElevation(double)`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/models/FGInertial.h)
    /// — accessed via `FGFDMExec::GetInertial()->SetTerrainElevation(...)`.
    /// Delegates to `FGGroundCallback::SetTerrainElevation()`.
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
        //
        // **JSBSim C++ origin:** `~FGFDMExec()` destructor
        // ([`src/FGFDMExec.h`](https://github.com/JSBSim-Team/jsbsim/blob/master/src/FGFDMExec.h)).
        unsafe {
            jsbsim_destroy(self.inner);
        }
        // `_ground_callback` is dropped automatically after this, which is
        // safe because the C++ side no longer references it.
    }
}

use std::ffi::CString;

#[allow(non_camel_case_types)]
pub type JSBSim_FGFDMExec = *mut std::ffi::c_void;

#[link(name = "jsbsim_wrapper")]
extern "C" {
    fn jsbsim_create(root_dir: *const i8) -> JSBSim_FGFDMExec;
    fn jsbsim_destroy(fdm: JSBSim_FGFDMExec);
    fn jsbsim_load_model(fdm: JSBSim_FGFDMExec, model: *const i8) -> bool;
    fn jsbsim_load_script(fdm: JSBSim_FGFDMExec, filename: *const i8) -> bool;
    fn jsbsim_run_ic(fdm: JSBSim_FGFDMExec) -> bool;
    fn jsbsim_run(fdm: JSBSim_FGFDMExec) -> bool;
    fn jsbsim_set_dt(fdm: JSBSim_FGFDMExec, dt: f64);
    fn jsbsim_get_property(fdm: JSBSim_FGFDMExec, name: *const i8) -> f64;
    fn jsbsim_set_property(fdm: JSBSim_FGFDMExec, name: *const i8, value: f64) -> bool;
}

pub struct Sim {
    inner: JSBSim_FGFDMExec,
}

impl Sim {
    /// Create a new JSBSim FDM instance.
    ///
    /// `root_dir` = path to your JSBSim data root (contains `aircraft/`, `engine/`,
    /// `systems/`, and `scripts/` directories).
    pub fn new(root_dir: &str) -> Self {
        let c_root = CString::new(root_dir).expect("root_dir contains null byte");
        let inner = unsafe { jsbsim_create(c_root.as_ptr()) };
        Self { inner }
    }

    /// Load an aircraft model by name (e.g., `"c172x"`).
    ///
    /// The model is looked up in the `aircraft/` directory under the root dir.
    pub fn load_model(&mut self, model: &str) -> bool {
        let c_model = CString::new(model).expect("model name contains null byte");
        unsafe { jsbsim_load_model(self.inner, c_model.as_ptr()) }
    }

    /// Load a JSBSim script XML file (relative to root dir).
    pub fn load_script(&mut self, filename: &str) -> bool {
        let c_file = CString::new(filename).expect("filename contains null byte");
        unsafe { jsbsim_load_script(self.inner, c_file.as_ptr()) }
    }

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
}

impl Drop for Sim {
    fn drop(&mut self) {
        unsafe { jsbsim_destroy(self.inner); }
    }
}

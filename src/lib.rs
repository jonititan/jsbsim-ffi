use std::ffi::CString;

#[allow(non_camel_case_types)]
pub type JSBSim_FGFDMExec = *mut std::ffi::c_void;

#[link(name = "jsbsim_wrapper")]
extern "C" {
    fn jsbsim_create(root_dir: *const i8) -> JSBSim_FGFDMExec;
    fn jsbsim_destroy(fdm: JSBSim_FGFDMExec);
    fn jsbsim_load_script(fdm: JSBSim_FGFDMExec, filename: *const i8) -> bool;
    fn jsbsim_run_ic(fdm: JSBSim_FGFDMExec) -> bool;
    fn jsbsim_run(fdm: JSBSim_FGFDMExec) -> bool;
    fn jsbsim_get_property(fdm: JSBSim_FGFDMExec, name: *const i8) -> f64;
    fn jsbsim_set_property(fdm: JSBSim_FGFDMExec, name: *const i8, value: f64) -> bool;
}

pub struct Sim {
    inner: JSBSim_FGFDMExec,
}

impl Sim {
    /// `root_dir` = path to your JSBSim source (contains `scripts/`, `aircraft/`, `engine/`, etc.)
    pub fn new(root_dir: &str) -> Self {
        let c_root = CString::new(root_dir).expect("root_dir contains null byte");
        let inner = unsafe { jsbsim_create(c_root.as_ptr()) };
        Self { inner }
    }

    pub fn load_script(&mut self, filename: &str) -> bool {
        let c_file = CString::new(filename).expect("filename contains null byte");
        unsafe { jsbsim_load_script(self.inner, c_file.as_ptr()) }
    }

    pub fn run_ic(&mut self) -> bool {
        unsafe { jsbsim_run_ic(self.inner) }
    }

    pub fn run(&mut self) -> bool {
        unsafe { jsbsim_run(self.inner) }
    }

    pub fn get_property(&self, name: &str) -> f64 {
        let c_name = CString::new(name).expect("property name contains null byte");
        unsafe { jsbsim_get_property(self.inner, c_name.as_ptr()) }
    }

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
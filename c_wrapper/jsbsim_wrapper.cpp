#include "jsbsim_wrapper.h"
#include <FGFDMExec.h>
#include <FGJSBBase.h>
#include <initialization/FGInitialCondition.h>
#include <input_output/FGPropertyManager.h>
#include <simgear/misc/sg_path.hxx>
#include <string>
#include <cstring>
#include <algorithm>

// Helper: safely cast the opaque pointer to FGFDMExec.
static inline JSBSim::FGFDMExec* as_exec(JSBSim_FGFDMExec* fdm) {
    return reinterpret_cast<JSBSim::FGFDMExec*>(fdm);
}

// Helper: copy a std::string into a caller-supplied C buffer.
// Returns the full string length (may exceed buf_len).
static int copy_str(const std::string& src, char* buf, int buf_len) {
    if (buf && buf_len > 0) {
        int n = std::min(static_cast<int>(src.size()), buf_len - 1);
        std::memcpy(buf, src.data(), n);
        buf[n] = '\0';
    }
    return static_cast<int>(src.size());
}

extern "C" {

/* ── Lifecycle ────────────────────────────────────────────────────── */

JSBSim_FGFDMExec* jsbsim_create(const char* root_dir) {
    auto* fdm = new JSBSim::FGFDMExec();
    if (root_dir && *root_dir) {
        fdm->SetRootDir(SGPath(root_dir));
        fdm->SetAircraftPath(SGPath("aircraft"));
        fdm->SetEnginePath(SGPath("engine"));
        fdm->SetSystemsPath(SGPath("systems"));
    }
    return reinterpret_cast<JSBSim_FGFDMExec*>(fdm);
}

void jsbsim_destroy(JSBSim_FGFDMExec* fdm) {
    delete as_exec(fdm);
}

/* ── Loading ──────────────────────────────────────────────────────── */

bool jsbsim_load_model(JSBSim_FGFDMExec* fdm, const char* model) {
    if (!fdm || !model || !*model) return false;
    try {
        return as_exec(fdm)->LoadModel(std::string(model));
    } catch (...) {
        return false;
    }
}

bool jsbsim_load_script(JSBSim_FGFDMExec* fdm, const char* filename) {
    if (!fdm || !filename || !*filename) return false;
    try {
        return as_exec(fdm)->LoadScript(SGPath(std::string(filename)));
    } catch (...) {
        return false;
    }
}

bool jsbsim_load_ic(JSBSim_FGFDMExec* fdm, const char* filename, bool use_aircraft_path) {
    if (!fdm || !filename || !*filename) return false;
    try {
        auto ic = as_exec(fdm)->GetIC();
        if (!ic) return false;
        return ic->Load(SGPath(std::string(filename)), use_aircraft_path);
    } catch (...) {
        return false;
    }
}

/* ── Simulation control ───────────────────────────────────────────── */

bool jsbsim_run_ic(JSBSim_FGFDMExec* fdm) {
    if (!fdm) return false;
    try {
        return as_exec(fdm)->RunIC();
    } catch (...) {
        return false;
    }
}

bool jsbsim_run(JSBSim_FGFDMExec* fdm) {
    if (!fdm) return false;
    try {
        return as_exec(fdm)->Run();
    } catch (...) {
        return false;
    }
}

void jsbsim_set_dt(JSBSim_FGFDMExec* fdm, double dt) {
    if (!fdm) return;
    as_exec(fdm)->Setdt(dt);
}

double jsbsim_get_dt(JSBSim_FGFDMExec* fdm) {
    if (!fdm) return 0.0;
    return as_exec(fdm)->GetDeltaT();
}

double jsbsim_get_sim_time(JSBSim_FGFDMExec* fdm) {
    if (!fdm) return 0.0;
    return as_exec(fdm)->GetSimTime();
}

void jsbsim_reset_to_initial_conditions(JSBSim_FGFDMExec* fdm, int mode) {
    if (!fdm) return;
    try {
        as_exec(fdm)->ResetToInitialConditions(mode);
    } catch (...) {}
}

/* ── Hold / Resume ────────────────────────────────────────────────── */

void jsbsim_hold(JSBSim_FGFDMExec* fdm) {
    if (!fdm) return;
    as_exec(fdm)->Hold();
}

void jsbsim_resume(JSBSim_FGFDMExec* fdm) {
    if (!fdm) return;
    as_exec(fdm)->Resume();
}

bool jsbsim_holding(JSBSim_FGFDMExec* fdm) {
    if (!fdm) return false;
    return as_exec(fdm)->Holding();
}

void jsbsim_enable_increment_then_hold(JSBSim_FGFDMExec* fdm, int steps) {
    if (!fdm) return;
    as_exec(fdm)->EnableIncrementThenHold(steps);
}

void jsbsim_check_incremental_hold(JSBSim_FGFDMExec* fdm) {
    if (!fdm) return;
    as_exec(fdm)->CheckIncrementalHold();
}

/* ── Integration suspend ──────────────────────────────────────────── */

void jsbsim_suspend_integration(JSBSim_FGFDMExec* fdm) {
    if (!fdm) return;
    as_exec(fdm)->SuspendIntegration();
}

void jsbsim_resume_integration(JSBSim_FGFDMExec* fdm) {
    if (!fdm) return;
    as_exec(fdm)->ResumeIntegration();
}

/* ── Trim ─────────────────────────────────────────────────────────── */

bool jsbsim_do_trim(JSBSim_FGFDMExec* fdm, int mode) {
    if (!fdm) return false;
    try {
        as_exec(fdm)->DoTrim(mode);
        return true;
    } catch (...) {
        return false;
    }
}

/* ── Properties ───────────────────────────────────────────────────── */

double jsbsim_get_property(JSBSim_FGFDMExec* fdm, const char* name) {
    if (!fdm || !name) return 0.0;
    return as_exec(fdm)->GetPropertyValue(name);
}

bool jsbsim_set_property(JSBSim_FGFDMExec* fdm, const char* name, double value) {
    if (!fdm || !name) return false;
    as_exec(fdm)->SetPropertyValue(name, value);
    return true;
}

bool jsbsim_has_property(JSBSim_FGFDMExec* fdm, const char* name) {
    if (!fdm || !name) return false;
    auto pm = as_exec(fdm)->GetPropertyManager();
    if (!pm) return false;
    return pm->HasNode(std::string(name));
}

/* ── Property catalog ─────────────────────────────────────────────── */

int jsbsim_query_property_catalog(JSBSim_FGFDMExec* fdm, const char* check,
                                  char* buf, int buf_len) {
    if (!fdm || !check) {
        if (buf && buf_len > 0) buf[0] = '\0';
        return 0;
    }
    std::string result = as_exec(fdm)->QueryPropertyCatalog(std::string(check));
    return copy_str(result, buf, buf_len);
}

void jsbsim_print_property_catalog(JSBSim_FGFDMExec* fdm) {
    if (!fdm) return;
    as_exec(fdm)->PrintPropertyCatalog();
}

/* ── Output control ───────────────────────────────────────────────── */

bool jsbsim_set_output_directive(JSBSim_FGFDMExec* fdm, const char* fname) {
    if (!fdm || !fname || !*fname) return false;
    try {
        return as_exec(fdm)->SetOutputDirectives(SGPath(std::string(fname)));
    } catch (...) {
        return false;
    }
}

void jsbsim_enable_output(JSBSim_FGFDMExec* fdm) {
    if (!fdm) return;
    as_exec(fdm)->EnableOutput();
}

void jsbsim_disable_output(JSBSim_FGFDMExec* fdm) {
    if (!fdm) return;
    as_exec(fdm)->DisableOutput();
}

bool jsbsim_set_output_filename(JSBSim_FGFDMExec* fdm, int n, const char* fname) {
    if (!fdm || !fname) return false;
    try {
        return as_exec(fdm)->SetOutputFileName(n, std::string(fname));
    } catch (...) {
        return false;
    }
}

/* ── Path configuration ───────────────────────────────────────────── */

bool jsbsim_set_aircraft_path(JSBSim_FGFDMExec* fdm, const char* path) {
    if (!fdm || !path) return false;
    return as_exec(fdm)->SetAircraftPath(SGPath(std::string(path)));
}

bool jsbsim_set_engine_path(JSBSim_FGFDMExec* fdm, const char* path) {
    if (!fdm || !path) return false;
    return as_exec(fdm)->SetEnginePath(SGPath(std::string(path)));
}

bool jsbsim_set_systems_path(JSBSim_FGFDMExec* fdm, const char* path) {
    if (!fdm || !path) return false;
    return as_exec(fdm)->SetSystemsPath(SGPath(std::string(path)));
}

/* ── Info / Debug ─────────────────────────────────────────────────── */

void jsbsim_set_debug_level(JSBSim_FGFDMExec* fdm, int level) {
    if (!fdm) return;
    as_exec(fdm)->SetDebugLevel(level);
}

int jsbsim_get_model_name(JSBSim_FGFDMExec* fdm, char* buf, int buf_len) {
    if (!fdm) {
        if (buf && buf_len > 0) buf[0] = '\0';
        return 0;
    }
    const std::string& name = as_exec(fdm)->GetModelName();
    return copy_str(name, buf, buf_len);
}

int jsbsim_get_version(char* buf, int buf_len) {
    const std::string& ver = JSBSim::FGJSBBase::GetVersion();
    return copy_str(ver, buf, buf_len);
}

} // extern "C"

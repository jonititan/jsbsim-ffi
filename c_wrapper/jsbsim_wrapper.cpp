#include "jsbsim_wrapper.h"
#include <FGFDMExec.h>
#include <simgear/misc/sg_path.hxx>
#include <string>

extern "C" {

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
    delete reinterpret_cast<JSBSim::FGFDMExec*>(fdm);
}

bool jsbsim_load_script(JSBSim_FGFDMExec* fdm, const char* filename) {
    if (!fdm || !filename || !*filename) return false;
    auto* exec = reinterpret_cast<JSBSim::FGFDMExec*>(fdm);
    return exec->LoadScript(SGPath(std::string(filename)));
}

bool jsbsim_run_ic(JSBSim_FGFDMExec* fdm) {
    if (!fdm) return false;
    return reinterpret_cast<JSBSim::FGFDMExec*>(fdm)->RunIC();
}

bool jsbsim_run(JSBSim_FGFDMExec* fdm) {
    if (!fdm) return false;
    return reinterpret_cast<JSBSim::FGFDMExec*>(fdm)->Run();
}

double jsbsim_get_property(JSBSim_FGFDMExec* fdm, const char* name) {
    if (!fdm || !name) return 0.0;
    return reinterpret_cast<JSBSim::FGFDMExec*>(fdm)->GetPropertyValue(name);
}

bool jsbsim_set_property(JSBSim_FGFDMExec* fdm, const char* name, double value) {
    if (!fdm || !name) return false;
    auto* exec = reinterpret_cast<JSBSim::FGFDMExec*>(fdm);
    exec->SetPropertyValue(name, value);
    return true;
}

}
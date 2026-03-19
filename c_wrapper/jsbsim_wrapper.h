#ifndef JSBSIM_WRAPPER_H
#define JSBSIM_WRAPPER_H

#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct JSBSim_FGFDMExec JSBSim_FGFDMExec;

JSBSim_FGFDMExec* jsbsim_create(const char* root_dir);
void jsbsim_destroy(JSBSim_FGFDMExec* fdm);

bool jsbsim_load_model(JSBSim_FGFDMExec* fdm, const char* model);
bool jsbsim_load_script(JSBSim_FGFDMExec* fdm, const char* filename);
bool jsbsim_run_ic(JSBSim_FGFDMExec* fdm);
bool jsbsim_run(JSBSim_FGFDMExec* fdm);
void jsbsim_set_dt(JSBSim_FGFDMExec* fdm, double dt);

double jsbsim_get_property(JSBSim_FGFDMExec* fdm, const char* name);
bool jsbsim_set_property(JSBSim_FGFDMExec* fdm, const char* name, double value);

#ifdef __cplusplus
}
#endif
#endif
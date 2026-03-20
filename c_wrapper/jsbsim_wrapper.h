#ifndef JSBSIM_WRAPPER_H
#define JSBSIM_WRAPPER_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct JSBSim_FGFDMExec JSBSim_FGFDMExec;

/* ── Lifecycle ────────────────────────────────────────────────────── */
JSBSim_FGFDMExec* jsbsim_create(const char* root_dir);
void              jsbsim_destroy(JSBSim_FGFDMExec* fdm);

/* ── Loading ──────────────────────────────────────────────────────── */
bool jsbsim_load_model(JSBSim_FGFDMExec* fdm, const char* model);
bool jsbsim_load_script(JSBSim_FGFDMExec* fdm, const char* filename);
bool jsbsim_load_ic(JSBSim_FGFDMExec* fdm, const char* filename, bool use_aircraft_path);

/* ── Simulation control ───────────────────────────────────────────── */
bool jsbsim_run_ic(JSBSim_FGFDMExec* fdm);
bool jsbsim_run(JSBSim_FGFDMExec* fdm);
void jsbsim_set_dt(JSBSim_FGFDMExec* fdm, double dt);
double jsbsim_get_dt(JSBSim_FGFDMExec* fdm);
double jsbsim_get_sim_time(JSBSim_FGFDMExec* fdm);
void jsbsim_reset_to_initial_conditions(JSBSim_FGFDMExec* fdm, int mode);

/* ── Hold / Resume ────────────────────────────────────────────────── */
void jsbsim_hold(JSBSim_FGFDMExec* fdm);
void jsbsim_resume(JSBSim_FGFDMExec* fdm);
bool jsbsim_holding(JSBSim_FGFDMExec* fdm);
void jsbsim_enable_increment_then_hold(JSBSim_FGFDMExec* fdm, int steps);
void jsbsim_check_incremental_hold(JSBSim_FGFDMExec* fdm);

/* ── Integration suspend ──────────────────────────────────────────── */
void jsbsim_suspend_integration(JSBSim_FGFDMExec* fdm);
void jsbsim_resume_integration(JSBSim_FGFDMExec* fdm);

/* ── Trim ─────────────────────────────────────────────────────────── */
bool jsbsim_do_trim(JSBSim_FGFDMExec* fdm, int mode);

/* ── Properties ───────────────────────────────────────────────────── */
double jsbsim_get_property(JSBSim_FGFDMExec* fdm, const char* name);
bool   jsbsim_set_property(JSBSim_FGFDMExec* fdm, const char* name, double value);
bool   jsbsim_has_property(JSBSim_FGFDMExec* fdm, const char* name);

/* ── Property catalog ─────────────────────────────────────────────── */
/*  query_property_catalog writes the result into buf (up to buf_len-1 chars)
    and returns the total length of the catalog string (may exceed buf_len).  */
int  jsbsim_query_property_catalog(JSBSim_FGFDMExec* fdm, const char* check,
                                   char* buf, int buf_len);
void jsbsim_print_property_catalog(JSBSim_FGFDMExec* fdm);

/* ── Output control ───────────────────────────────────────────────── */
bool jsbsim_set_output_directive(JSBSim_FGFDMExec* fdm, const char* fname);
void jsbsim_enable_output(JSBSim_FGFDMExec* fdm);
void jsbsim_disable_output(JSBSim_FGFDMExec* fdm);
bool jsbsim_set_output_filename(JSBSim_FGFDMExec* fdm, int n, const char* fname);

/* ── Path configuration ───────────────────────────────────────────── */
bool jsbsim_set_aircraft_path(JSBSim_FGFDMExec* fdm, const char* path);
bool jsbsim_set_engine_path(JSBSim_FGFDMExec* fdm, const char* path);
bool jsbsim_set_systems_path(JSBSim_FGFDMExec* fdm, const char* path);

/* ── Info / Debug ─────────────────────────────────────────────────── */
void jsbsim_set_debug_level(JSBSim_FGFDMExec* fdm, int level);
/*  get_model_name / get_version write into caller-supplied buf and return
    the actual string length.  */
int  jsbsim_get_model_name(JSBSim_FGFDMExec* fdm, char* buf, int buf_len);
int  jsbsim_get_version(char* buf, int buf_len);

#ifdef __cplusplus
}
#endif
#endif

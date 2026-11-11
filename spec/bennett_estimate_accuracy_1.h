#ifndef BENNETT_ESTIMATE_ACCURACY_1_H
#define BENNETT_ESTIMATE_ACCURACY_1_H

/* Additive bennett.estimate-accuracy/1; requires a launch/0.9 provider
   advertising estimate_accuracy/1. No implementation is supplied here. */
#include <bennett_launch_0_9.h>

#ifdef __cplusplus
extern "C" {
#endif

bennett_status bennett_estimate_run_at_accuracy(
    bennett_program_t* program, const char* region, bennett_token token,
    const char* variant, uint32_t inherited_controls,
    const uint8_t* params_jcs, size_t params_len,
    const uint8_t* aliases_jcs, size_t aliases_len,
    const char* epsilon_decimal, bennett_estimate_t** out);

#ifdef __cplusplus
}
#endif
#endif

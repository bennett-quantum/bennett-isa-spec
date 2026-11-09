/* Do not combine ABI families, including on a repeated include. */
#if defined(BENNETT_LAUNCH_H) || defined(BENNETT_SCHED_H)
#error "Bennett C ABI families 0.8 and 0.9 cannot be mixed in one translation unit"
#endif
#if defined(BENNETT_ABI_FAMILY) && BENNETT_ABI_FAMILY != 9
#error "Bennett 0.9 headers require BENNETT_ABI_FAMILY=9 when a family is specified"
#endif

#ifndef BENNETT_SCHED_0_9_H
#define BENNETT_SCHED_0_9_H

/* Bennett ISA 0.9 §11: sched 0.9. Declarations only.
   Select a matching provider; these names do not version linker symbols.
   See README.md for explicit family and provider selection. */
#include "bennett_launch_0_9.h"

#ifdef __cplusplus
extern "C" {
#endif

/* ISA 0.9 §11. */
typedef struct bennett_sched_request_t bennett_sched_request_t;

/* ISA 0.9 §11. */
typedef struct bennett_sched_response_t bennett_sched_response_t;

/* ISA 0.9 §11. */
bennett_status bennett_sched_request_new(bennett_sched_request_t** out);

/* ISA 0.9 §11. */
bennett_status bennett_sched_request_set_object(bennett_sched_request_t*, const uint8_t*, size_t);

/* ISA 0.9 §11. */
bennett_status bennett_sched_request_set_target(bennett_sched_request_t*, const uint8_t* jcs, size_t);

/* ISA 0.9 §11. */
bennett_status bennett_sched_request_set_region(bennett_sched_request_t*, const char* closure_hash_hex, const char* variant, uint32_t inherited_controls);

/* ISA 0.9 §11. */
bennett_status bennett_sched_request_set_budget(bennett_sched_request_t*, uint64_t steps, uint64_t heap, uint64_t stack, uint64_t depth);

/* ISA 0.9 §11. */
bennett_status bennett_sched_request_set_params(bennett_sched_request_t*, const uint8_t* params_jcs, size_t);

/* ISA 0.9 §11. */
bennett_status bennett_sched_request_set_accuracy(bennett_sched_request_t*, const char* epsilon_decimal);

/* ISA 0.9 §11. */
bennett_status bennett_sched_request_set_capacity(bennett_sched_request_t*, const char* cls, uint64_t peak_qubits);

/* ISA 0.9 §11 C declaration; §10.1 (K91) partition validation. */
bennett_status bennett_sched_request_set_token_aliases(
    bennett_sched_request_t*, const uint8_t* aliases_jcs, size_t aliases_len);

/* ISA 0.9 §11. */
void           bennett_sched_request_free(bennett_sched_request_t*);

/* ISA 0.9 §11. */
bennett_status bennett_sched_schedule(const bennett_sched_request_t*, bennett_sched_response_t** out);

/* ISA 0.9 §11 C declaration: bennett_sched_outcome. */
typedef enum {
    /* ISA 0.9 §11 C enumerator: BENNETT_SCHED_OK. */
    BENNETT_SCHED_OK,
    /* ISA 0.9 §11 C enumerator: BENNETT_SCHED_INFEASIBLE. */
    BENNETT_SCHED_INFEASIBLE,
    /* ISA 0.9 §11 C enumerator: BENNETT_SCHED_UNSUPPORTED. */
    BENNETT_SCHED_UNSUPPORTED
} bennett_sched_outcome;

/* ISA 0.9 §11. */
bennett_sched_outcome bennett_sched_response_outcome(const bennett_sched_response_t*);

/* ISA 0.9 §11. */
const char*    bennett_sched_response_reason(const bennett_sched_response_t*);

/* ISA 0.9 §11. */
const uint8_t* bennett_sched_response_schedule(const bennett_sched_response_t*, size_t* len);

/* ISA 0.9 §11. */
const uint8_t* bennett_sched_response_certificate(const bennett_sched_response_t*, size_t* len);

/* ISA 0.9 §11. */
void           bennett_sched_response_free(bennett_sched_response_t*);

/* ISA 0.9 §10.1: immutable provider-owned JCS bytes last until provider
   unload. Query/manifest validation and negotiation are required before
   alias-aware dispatch; a header inclusion advertises no capability. */
/* ISA 0.9 §10.1 (K91) per-role feature-query C declaration. */
bennett_status bennett_sched_features(const uint8_t** jcs_out, size_t* len_out);

#ifdef __cplusplus
}
#endif

#endif /* BENNETT_SCHED_0_9_H */

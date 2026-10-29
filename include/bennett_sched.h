#ifndef BENNETT_SCHED_H
#define BENNETT_SCHED_H

/* Bennett ISA 0.8 §11: sched 0.8. Declarations only. */
#include "bennett_launch.h"

#ifdef __cplusplus
extern "C" {
#endif

/* §11: "Schedule document (JCS)"; certificates use §12 "Certificate (cert 0.8; JCS)". */
/* §9.1: "Integers that may exceed 2^53 are decimal strings." */
/* §11: "satisfy §6; respect capacity; report every resource; be reproducible from the request plus scheduler version." */

/* §11 C declaration: "typedef struct bennett_sched_request_t bennett_sched_request_t;" */
typedef struct bennett_sched_request_t bennett_sched_request_t;

/* §11 C declaration: "typedef struct bennett_sched_response_t bennett_sched_response_t;" */
typedef struct bennett_sched_response_t bennett_sched_response_t;

/* §11 C declaration: "bennett_status bennett_sched_request_new(bennett_sched_request_t** out);" */
bennett_status bennett_sched_request_new(bennett_sched_request_t** out);

/* §11 C declaration: "bennett_status bennett_sched_request_set_object(bennett_sched_request_t*, const uint8_t*, size_t);" */
bennett_status bennett_sched_request_set_object(bennett_sched_request_t*, const uint8_t*, size_t);

/* §11 C declaration: "bennett_status bennett_sched_request_set_target(bennett_sched_request_t*, const uint8_t* jcs, size_t);" */
bennett_status bennett_sched_request_set_target(bennett_sched_request_t*, const uint8_t* jcs, size_t);

/* §11 C declaration: "bennett_status bennett_sched_request_set_region(bennett_sched_request_t*, const char* closure_hash_hex, const char* variant, uint32_t inherited_controls);" */
bennett_status bennett_sched_request_set_region(bennett_sched_request_t*, const char* closure_hash_hex, const char* variant, uint32_t inherited_controls);

/* §11 C declaration: "bennett_status bennett_sched_request_set_budget(bennett_sched_request_t*, uint64_t steps, uint64_t heap, uint64_t stack, uint64_t depth);" */
bennett_status bennett_sched_request_set_budget(bennett_sched_request_t*, uint64_t steps, uint64_t heap, uint64_t stack, uint64_t depth);

/* §11 C declaration: "bennett_status bennett_sched_request_set_params(bennett_sched_request_t*, const uint8_t* params_jcs, size_t);" */
bennett_status bennett_sched_request_set_params(bennett_sched_request_t*, const uint8_t* params_jcs, size_t);

/* §11 C declaration: "bennett_status bennett_sched_request_set_accuracy(bennett_sched_request_t*, const char* epsilon_decimal);" */
bennett_status bennett_sched_request_set_accuracy(bennett_sched_request_t*, const char* epsilon_decimal);

/* §11 C declaration: "bennett_status bennett_sched_request_set_capacity(bennett_sched_request_t*, const char* cls, uint64_t peak_qubits);" */
bennett_status bennett_sched_request_set_capacity(bennett_sched_request_t*, const char* cls, uint64_t peak_qubits);

/* §11 C declaration: "void bennett_sched_request_free(bennett_sched_request_t*);" */
void bennett_sched_request_free(bennett_sched_request_t*);

/* §11 C declaration: "bennett_status bennett_sched_schedule(const bennett_sched_request_t*, bennett_sched_response_t** out);" */
bennett_status bennett_sched_schedule(const bennett_sched_request_t*, bennett_sched_response_t** out);

/* §11 C declaration: "bennett_sched_outcome"; enumerators follow in normative order. */
typedef enum {
    /* §11 C enumerator: "BENNETT_SCHED_OK". */
    BENNETT_SCHED_OK,
    /* §11 C enumerator: "BENNETT_SCHED_INFEASIBLE". */
    BENNETT_SCHED_INFEASIBLE,
    /* §11 C enumerator: "BENNETT_SCHED_UNSUPPORTED". */
    BENNETT_SCHED_UNSUPPORTED,
} bennett_sched_outcome;

/* §11 C declaration: "bennett_sched_outcome bennett_sched_response_outcome(const bennett_sched_response_t*);" */
bennett_sched_outcome bennett_sched_response_outcome(const bennett_sched_response_t*);

/* §11 C declaration: "const char* bennett_sched_response_reason(const bennett_sched_response_t*);" */
const char* bennett_sched_response_reason(const bennett_sched_response_t*);

/* §11 C declaration: "const uint8_t* bennett_sched_response_schedule(const bennett_sched_response_t*, size_t* len);" */
const uint8_t* bennett_sched_response_schedule(const bennett_sched_response_t*, size_t* len);

/* §11 C declaration: "const uint8_t* bennett_sched_response_certificate(const bennett_sched_response_t*, size_t* len);" */
const uint8_t* bennett_sched_response_certificate(const bennett_sched_response_t*, size_t* len);

/* §11 C declaration: "void bennett_sched_response_free(bennett_sched_response_t*);" */
void bennett_sched_response_free(bennett_sched_response_t*);

#ifdef __cplusplus
}
#endif

#endif /* BENNETT_SCHED_H */

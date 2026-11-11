/* Do not combine ABI families, including on a repeated include. */
#if defined(BENNETT_LAUNCH_H) || defined(BENNETT_SCHED_H)
#error "Bennett C ABI families 0.8 and 0.9 cannot be mixed in one translation unit"
#endif
#if defined(BENNETT_ABI_FAMILY) && BENNETT_ABI_FAMILY != 9
#error "Bennett 0.9 headers require BENNETT_ABI_FAMILY=9 when a family is specified"
#endif

#ifndef BENNETT_LAUNCH_0_9_H
#define BENNETT_LAUNCH_0_9_H

/* Bennett ISA 0.9 §10: launch 0.9. Declarations only.
   Select a matching provider; these names do not version linker symbols.
   See README.md for explicit family and provider selection. */
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ISA 0.9 §10: objects are opaque; returned objects transfer ownership,
   with matching *_free functions (programs use bennett_unload).
   Accessor pointers last until their owner is freed. Host tokens belong
   to the program; device tokens belong to their run (§5.4). */

/* ISA 0.9 §10 C declaration: bennett_status. */
typedef enum {
    /* ISA 0.9 §10 C enumerator: BENNETT_OK=0. */
    BENNETT_OK=0,
    /* ISA 0.9 §10 C enumerator: BENNETT_E_VERSION. */
    BENNETT_E_VERSION,
    /* ISA 0.9 §10 C enumerator: BENNETT_E_PTR_WIDTH. */
    BENNETT_E_PTR_WIDTH,
    /* ISA 0.9 §10 C enumerator: BENNETT_E_DEVICE_MISSING. */
    BENNETT_E_DEVICE_MISSING,
    /* ISA 0.9 §10 C enumerator: BENNETT_E_GATE_UNSUPPORTED. */
    BENNETT_E_GATE_UNSUPPORTED,
    /* ISA 0.9 §10 C enumerator: BENNETT_E_TARGET. */
    BENNETT_E_TARGET,
    /* ISA 0.9 §10 C enumerator: BENNETT_E_BUDGET. */
    BENNETT_E_BUDGET,
    /* ISA 0.9 §10 C enumerator: BENNETT_E_QREG_NOT_CLEAN. */
    BENNETT_E_QREG_NOT_CLEAN,
    /* ISA 0.9 §10 C enumerator: BENNETT_E_INVALID_STATE. */
    BENNETT_E_INVALID_STATE,
    /* ISA 0.9 §10 C enumerator: BENNETT_E_NOT_REVERSIBLE. */
    BENNETT_E_NOT_REVERSIBLE,
    /* ISA 0.9 §10 C enumerator: BENNETT_E_SCHEDULE_INFEASIBLE. */
    BENNETT_E_SCHEDULE_INFEASIBLE,
    /* ISA 0.9 §10 C enumerator: BENNETT_E_ARG. */
    BENNETT_E_ARG,
    /* ISA 0.9 §10 C enumerator: BENNETT_E_NOMEM. */
    BENNETT_E_NOMEM
} bennett_status;

/* ISA 0.9 §10. */
typedef struct bennett_program_t bennett_program_t;

/* ISA 0.9 §10. */
typedef struct bennett_args_t bennett_args_t;

/* ISA 0.9 §10. */
typedef struct bennett_result_t bennett_result_t;

/* ISA 0.9 §10. */
typedef struct bennett_request_t bennett_request_t;

/* ISA 0.9 §10. */
typedef struct bennett_estimate_t bennett_estimate_t;

/* ISA 0.9 §10. */
typedef struct bennett_coverage_t bennett_coverage_t;

/* ISA 0.9 §10. */
typedef struct bennett_tape_iter_t bennett_tape_iter_t;

/* ISA 0.9 §10. */
typedef uint64_t bennett_qreg;

/* ISA 0.9 §10. */
typedef uint64_t bennett_token;

/* ISA 0.9 §10. */
typedef uint64_t bennett_run_id;

/* ISA 0.9 §10 C declaration: bennett_arg_kind. */
typedef enum {
    /* ISA 0.9 §10 C enumerator: BENNETT_ARG_INT. */
    BENNETT_ARG_INT,
    /* ISA 0.9 §10 C enumerator: BENNETT_ARG_F64. */
    BENNETT_ARG_F64,
    /* ISA 0.9 §10 C enumerator: BENNETT_ARG_QREG. */
    BENNETT_ARG_QREG,
    /* ISA 0.9 §10 C enumerator: BENNETT_ARG_TOKEN. */
    BENNETT_ARG_TOKEN
} bennett_arg_kind;

/* ISA 0.9 §10. */
bennett_status bennett_load(const uint8_t* object, size_t len, const uint8_t* target_jcs, size_t tlen, bennett_program_t** out);

/* ISA 0.9 §10. */
bennett_status bennett_unload(bennett_program_t*);

/* ISA 0.9 §10. */
bennett_status bennett_budget(bennett_program_t*, const char* region, uint64_t steps, uint64_t heap, uint64_t stack, uint64_t depth, bennett_token* out);

/* ISA 0.9 §10. */
bennett_status bennett_qreg_new(bennett_program_t*, const char* cls, uint32_t n, bennett_qreg* out);

/* ISA 0.9 §10. */
bennett_status bennett_qreg_prepare(bennett_program_t*, bennett_qreg, const uint8_t* bits, size_t nbits);

/* ISA 0.9 §10. */
bennett_status bennett_qreg_free(bennett_program_t*, bennett_qreg);

/* ISA 0.9 §10. */
bennett_status bennett_args_new(size_t n, bennett_args_t** out);

/* ISA 0.9 §10. */
bennett_status bennett_args_set_int(bennett_args_t*, size_t i, uint32_t width, const uint8_t* le_bytes);

/* ISA 0.9 §10. */
bennett_status bennett_args_set_f64(bennett_args_t*, size_t i, double);

/* ISA 0.9 §10. */
bennett_status bennett_args_set_qreg(bennett_args_t*, size_t i, bennett_qreg);

/* ISA 0.9 §10. */
bennett_status bennett_args_set_token(bennett_args_t*, size_t i, bennett_token);

/* ISA 0.9 §10. */
void           bennett_args_free(bennett_args_t*);

/* ISA 0.9 §10. */
bennett_status bennett_run(bennett_program_t*, const char* cc_entry, const bennett_args_t*, const uint8_t* entropy, size_t entropy_len, bennett_run_id* out);

/* ISA 0.9 §10. */
bennett_status bennett_wait(bennett_program_t*, bennett_run_id, bennett_result_t** result_out, bennett_request_t** request_out);

/* ISA 0.9 §10. */
bennett_status bennett_reply(bennett_program_t*, bennett_run_id, const uint8_t* payload, size_t len);

/* ISA 0.9 §10. */
bennett_status bennett_suspend(bennett_program_t*, bennett_run_id);

/* ISA 0.9 §10. */
bennett_status bennett_resume(bennett_program_t*, bennett_run_id);

/* ISA 0.9 §10. */
bennett_status bennett_reverse(bennett_program_t*, bennett_run_id);

/* ISA 0.9 §10. */
bennett_status bennett_run_close(bennett_program_t*, bennett_run_id);

/* ISA 0.9 §10. */
size_t         bennett_result_nvalues(const bennett_result_t*);

/* ISA 0.9 §10. */
bennett_status bennett_result_get(const bennett_result_t*, size_t i, bennett_arg_kind* kind, uint32_t* width, uint8_t* le_bytes_out, size_t cap);

/* ISA 0.9 §10. */
size_t         bennett_result_nfaults(const bennett_result_t*);

/* ISA 0.9 §10. */
uint64_t bennett_result_fault(const bennett_result_t*, size_t i);

/* ISA 0.9 §10. */
size_t         bennett_result_nmeasurements(const bennett_result_t*);

/* ISA 0.9 §10. */
bennett_status bennett_result_measurement(const bennett_result_t*, size_t i, bennett_qreg* reg, uint8_t* bits_out, size_t cap, size_t* nbits);

/* ISA 0.9 §10. */
uint64_t       bennett_result_tape_length(const bennett_result_t*);

/* ISA 0.9 §10. */
void bennett_result_free(bennett_result_t*);

/* ISA 0.9 §10. */
const char*    bennett_request_service(const bennett_request_t*);

/* ISA 0.9 §10. */
const uint8_t* bennett_request_payload(const bennett_request_t*, size_t* len);

/* ISA 0.9 §10. */
void bennett_request_free(bennett_request_t*);

/* ISA 0.9 §10. */
bennett_status bennett_tape_open(bennett_program_t*, bennett_run_id, bennett_tape_iter_t** out);

/* ISA 0.9 §10. */
bennett_status bennett_tape_next(bennett_tape_iter_t*, const char** dev, const char** op, const uint8_t** args, size_t* alen);

/* ISA 0.9 §10. */
void           bennett_tape_free(bennett_tape_iter_t*);

/* ISA 0.9 §10 C declaration; §10.1 (K12) resolved request. */
bennett_status bennett_estimate_run(bennett_program_t*, const char* region, bennett_token,
                                    const char* variant, uint32_t inherited_controls,
                                    const uint8_t* params_jcs, size_t params_len, bennett_estimate_t** out);

/* ISA 0.9 §10 C declaration; §10.1 (K91) alias-aware request. */
bennett_status bennett_estimate_run_with_token_aliases(
    bennett_program_t*, const char* region, bennett_token,
    const char* variant, uint32_t inherited_controls,
    const uint8_t* params_jcs, size_t params_len,
    const uint8_t* aliases_jcs, size_t aliases_len, bennett_estimate_t** out);

/* ISA 0.9 §10. */
size_t         bennett_estimate_nresources(const bennett_estimate_t*);

/* ISA 0.9 §10. */
bennett_status bennett_estimate_get(const bennett_estimate_t*, size_t i, const char** name, const char** value_decimal);

/* ISA 0.9 §10. */
const uint8_t* bennett_estimate_certificate(const bennett_estimate_t*, size_t* len);

/* ISA 0.9 §10. */
void bennett_estimate_free(bennett_estimate_t*);

/* ISA 0.9 §10. */
bennett_status bennett_coverage_get(bennett_program_t*, bennett_coverage_t** out);

/* ISA 0.9 §10. */
const uint8_t* bennett_coverage_jcs(const bennett_coverage_t*, size_t* len);

/* ISA 0.9 §10. */
void bennett_coverage_free(bennett_coverage_t*);

/* ISA 0.9 §10.1: immutable provider-owned JCS bytes last until provider
   unload. Query/manifest validation and negotiation are required before
   alias-aware dispatch; a header inclusion advertises no capability. */
/* ISA 0.9 §10.1 (K91) per-role feature-query C declaration. */
bennett_status bennett_launch_features(const uint8_t** jcs_out, size_t* len_out);

/* ISA 0.9 §10.1 mixed-launch/1 C declarations. */
typedef struct bennett_mixed_t bennett_mixed_t;

bennett_status bennett_mixed_bind_v1(
    bennett_program_t* program, const uint8_t* container_jcs,
    size_t container_len, uint64_t root_index, bennett_mixed_t** out);

bennett_status bennett_mixed_run_v1(
    bennett_mixed_t* binding, const bennett_args_t* args,
    const uint8_t* entropy, size_t entropy_len,
    const uint8_t* call_jcs, size_t call_len, bennett_run_id* out);

bennett_status bennett_mixed_unbind_v1(bennett_mixed_t* binding);

#ifdef __cplusplus
}
#endif

#endif /* BENNETT_LAUNCH_0_9_H */

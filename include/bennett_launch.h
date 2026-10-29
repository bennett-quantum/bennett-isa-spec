#ifndef BENNETT_LAUNCH_H
#define BENNETT_LAUNCH_H

/* Bennett ISA 0.8 §10: launch 0.8. Declarations only. */
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* §10: "All objects opaque." Only the listed functions touch them (§17 item 5). */
/* §10: "A function that returns an object transfers ownership to the caller, released with the matching *_free." */
/* §10 Model: "Accessor-returned pointers are valid until the owning object is freed." */
/* §10 Model: "Tokens are scoped to the program and destroyed at unload." */
/* §10 States: "A run owns the registers and memory it touches exclusively while not CLOSED." */
/* §10 States: "COMPLETED/FAILED/REVERSED → CLOSED"; "COMPLETED → REVERSED". */
/* §10 States: "there is no reset and no trace-out"; measure consumes the handle (§3.3). */
/* §10 Results: INT = ceil(N/8) LE bytes; F64 = 8 IEEE 754 binary64 LE bytes; QREG/TOKEN = 8-byte LE id. */
/* §10 Results: "width = N" for INT; "width = 64" for F64, QREG and TOKEN. */
/* §12: "Certificate (cert 0.8; JCS)"; §13: "Coverage (coverage 0.8)". */

/* §10 C declaration: "bennett_status"; enumerators follow in normative order. */
typedef enum {
    /* §10 C enumerator: "BENNETT_OK=0". */
    BENNETT_OK=0,
    /* §10 C enumerator: "BENNETT_E_VERSION". */
    BENNETT_E_VERSION,
    /* §10 C enumerator: "BENNETT_E_PTR_WIDTH". */
    BENNETT_E_PTR_WIDTH,
    /* §10 C enumerator: "BENNETT_E_DEVICE_MISSING". */
    BENNETT_E_DEVICE_MISSING,
    /* §10 C enumerator: "BENNETT_E_GATE_UNSUPPORTED". */
    BENNETT_E_GATE_UNSUPPORTED,
    /* §10 C enumerator: "BENNETT_E_TARGET". */
    BENNETT_E_TARGET,
    /* §10 C enumerator: "BENNETT_E_BUDGET". */
    BENNETT_E_BUDGET,
    /* §10 C enumerator: "BENNETT_E_QREG_NOT_CLEAN". */
    BENNETT_E_QREG_NOT_CLEAN,
    /* §10 C enumerator: "BENNETT_E_INVALID_STATE". */
    BENNETT_E_INVALID_STATE,
    /* §10 C enumerator: "BENNETT_E_NOT_REVERSIBLE". */
    BENNETT_E_NOT_REVERSIBLE,
    /* §10 C enumerator: "BENNETT_E_SCHEDULE_INFEASIBLE". */
    BENNETT_E_SCHEDULE_INFEASIBLE,
    /* §10 C enumerator: "BENNETT_E_ARG". */
    BENNETT_E_ARG,
    /* §10 C enumerator: "BENNETT_E_NOMEM". */
    BENNETT_E_NOMEM,
} bennett_status;

/* §10 C declaration: "typedef struct bennett_program_t bennett_program_t;" */
typedef struct bennett_program_t bennett_program_t;

/* §10 C declaration: "typedef struct bennett_args_t bennett_args_t;" */
typedef struct bennett_args_t bennett_args_t;

/* §10 C declaration: "typedef struct bennett_result_t bennett_result_t;" */
typedef struct bennett_result_t bennett_result_t;

/* §10 C declaration: "typedef struct bennett_request_t bennett_request_t;" */
typedef struct bennett_request_t bennett_request_t;

/* §10 C declaration: "typedef struct bennett_estimate_t bennett_estimate_t;" */
typedef struct bennett_estimate_t bennett_estimate_t;

/* §10 C declaration: "typedef struct bennett_coverage_t bennett_coverage_t;" */
typedef struct bennett_coverage_t bennett_coverage_t;

/* §10 C declaration: "typedef struct bennett_tape_iter_t bennett_tape_iter_t;" */
typedef struct bennett_tape_iter_t bennett_tape_iter_t;

/* §10 C declaration: "typedef uint64_t bennett_qreg;" */
typedef uint64_t bennett_qreg;

/* §10 C declaration: "typedef uint64_t bennett_token;" */
typedef uint64_t bennett_token;

/* §10 C declaration: "typedef uint64_t bennett_run_id;" */
typedef uint64_t bennett_run_id;

/* §10 C declaration: "bennett_arg_kind"; enumerators follow in normative order. */
typedef enum {
    /* §10 C enumerator: "BENNETT_ARG_INT". */
    BENNETT_ARG_INT,
    /* §10 C enumerator: "BENNETT_ARG_F64". */
    BENNETT_ARG_F64,
    /* §10 C enumerator: "BENNETT_ARG_QREG". */
    BENNETT_ARG_QREG,
    /* §10 C enumerator: "BENNETT_ARG_TOKEN". */
    BENNETT_ARG_TOKEN,
} bennett_arg_kind;

/* §10 C declaration: "bennett_status bennett_load(const uint8_t* object, size_t len, const uint8_t* target_jcs, size_t tlen, bennett_program_t** out);" */
bennett_status bennett_load(const uint8_t* object, size_t len, const uint8_t* target_jcs, size_t tlen, bennett_program_t** out);

/* §10 Model: "bennett_unload with a run not CLOSED: E_INVALID_STATE." Tokens are destroyed at unload. */
/* §10 C declaration: "bennett_status bennett_unload(bennett_program_t*);" */
bennett_status bennett_unload(bennett_program_t*);

/* §5.4: "bennett_budget validates" heap, stack and depth limits; memoryless limits are zero. */
/* §10 C declaration: "bennett_status bennett_budget(bennett_program_t*, const char* region, uint64_t steps, uint64_t heap, uint64_t stack, uint64_t depth, bennett_token* out);" */
bennett_status bennett_budget(bennett_program_t*, const char* region, uint64_t steps, uint64_t heap, uint64_t stack, uint64_t depth, bennett_token* out);

/* §10 C declaration: "bennett_status bennett_qreg_new(bennett_program_t*, const char* cls, uint32_t n, bennett_qreg* out);" */
bennett_status bennett_qreg_new(bennett_program_t*, const char* cls, uint32_t n, bennett_qreg* out);

/* §10 States: "require a clean register (E_QREG_NOT_CLEAN otherwise)". */
/* §10 C declaration: "bennett_status bennett_qreg_prepare(bennett_program_t*, bennett_qreg, const uint8_t* bits, size_t nbits);" */
bennett_status bennett_qreg_prepare(bennett_program_t*, bennett_qreg, const uint8_t* bits, size_t nbits);

/* §10 States: "require a clean register (E_QREG_NOT_CLEAN otherwise)". */
/* §10 C declaration: "bennett_status bennett_qreg_free(bennett_program_t*, bennett_qreg);" */
bennett_status bennett_qreg_free(bennett_program_t*, bennett_qreg);

/* §10 C declaration: "bennett_status bennett_args_new(size_t n, bennett_args_t** out);" */
bennett_status bennett_args_new(size_t n, bennett_args_t** out);

/* §10 C declaration: "bennett_status bennett_args_set_int(bennett_args_t*, size_t i, uint32_t width, const uint8_t* le_bytes);" */
bennett_status bennett_args_set_int(bennett_args_t*, size_t i, uint32_t width, const uint8_t* le_bytes);

/* §10 C declaration: "bennett_status bennett_args_set_f64(bennett_args_t*, size_t i, double);" */
bennett_status bennett_args_set_f64(bennett_args_t*, size_t i, double);

/* §10 C declaration: "bennett_status bennett_args_set_qreg(bennett_args_t*, size_t i, bennett_qreg);" */
bennett_status bennett_args_set_qreg(bennett_args_t*, size_t i, bennett_qreg);

/* §10 C declaration: "bennett_status bennett_args_set_token(bennett_args_t*, size_t i, bennett_token);" */
bennett_status bennett_args_set_token(bennett_args_t*, size_t i, bennett_token);

/* §10 C declaration: "void bennett_args_free(bennett_args_t*);" */
void bennett_args_free(bennett_args_t*);

/* §10 Model: "Arguments and the entropy tape are copied at bennett_run"; positional kinds and widths must match. */
/* §10 C declaration: "bennett_status bennett_run(bennett_program_t*, const char* cc_entry, const bennett_args_t*, const uint8_t* entropy, size_t entropy_len, bennett_run_id* out);" */
bennett_status bennett_run(bennett_program_t*, const char* cc_entry, const bennett_args_t*, const uint8_t* entropy, size_t entropy_len, bennett_run_id* out);

/* §10 Waiting: "returns OK with exactly one outcome"; request or result is set and the other is NULL. */
/* §10 C declaration: "bennett_status bennett_wait(bennett_program_t*, bennett_run_id, bennett_result_t** result_out, bennett_request_t** request_out);" */
bennett_status bennett_wait(bennett_program_t*, bennett_run_id, bennett_result_t** result_out, bennett_request_t** request_out);

/* §10 Waiting: "bennett_reply answers it and the run is RUNNING again"; hostcall copies a possibly truncated reply. */
/* §10 C declaration: "bennett_status bennett_reply(bennett_program_t*, bennett_run_id, const uint8_t* payload, size_t len);" */
bennett_status bennett_reply(bennett_program_t*, bennett_run_id, const uint8_t* payload, size_t len);

/* §10 C declaration: "bennett_status bennett_suspend(bennett_program_t*, bennett_run_id);" */
/* §10 States: "SUSPENDED ⇄ RUNNING". */
bennett_status bennett_suspend(bennett_program_t*, bennett_run_id);

/* §10 C declaration: "bennett_status bennett_resume(bennett_program_t*, bennett_run_id);" */
/* §10 States: "SUSPENDED ⇄ RUNNING". */
bennett_status bennett_resume(bennett_program_t*, bennett_run_id);

/* §10 States: "Reverse eligibility": COMPLETED, no measure, no later prepare/free/slice/use; else E_NOT_REVERSIBLE. */
/* §10 Invocation tree: "reverses post-order"; R history is LIFO, reversed segments and uncall frames skipped (§7.4). */
/* §10 C declaration: "bennett_status bennett_reverse(bennett_program_t*, bennett_run_id);" */
bennett_status bennett_reverse(bennett_program_t*, bennett_run_id);

/* §10 States: "releases the record; register ids created by the run remain valid unless measured or freed". */
/* §10 C declaration: "bennett_status bennett_run_close(bennett_program_t*, bennett_run_id);" */
bennett_status bennett_run_close(bennett_program_t*, bennett_run_id);

/* §10 C declaration: "size_t bennett_result_nvalues(const bennett_result_t*);" */
size_t bennett_result_nvalues(const bennett_result_t*);

/* §10 Results: "A cap smaller than that byte count is E_ARG and nothing is written." */
/* §10 C declaration: "bennett_status bennett_result_get(const bennett_result_t*, size_t i, bennett_arg_kind* kind, uint32_t* width, uint8_t* le_bytes_out, size_t cap);" */
bennett_status bennett_result_get(const bennett_result_t*, size_t i, bennett_arg_kind* kind, uint32_t* width, uint8_t* le_bytes_out, size_t cap);

/* §10 Results: "a COMPLETED run has none, a FAILED run has at least one and exposes no values". */
/* §10 C declaration: "size_t bennett_result_nfaults(const bennett_result_t*);" */
size_t bennett_result_nfaults(const bennett_result_t*);

/* §10 Results: "codes of §5.4, in order of occurrence". */
/* §10 C declaration: "uint64_t bennett_result_fault(const bennett_result_t*, size_t i);" */
uint64_t bennett_result_fault(const bennett_result_t*, size_t i);

/* §10 C declaration: "size_t bennett_result_nmeasurements(const bennett_result_t*);" */
size_t bennett_result_nmeasurements(const bennett_result_t*);

/* §10 C declaration: "bennett_status bennett_result_measurement(const bennett_result_t*, size_t i, bennett_qreg* reg, uint8_t* bits_out, size_t cap, size_t* nbits);" */
bennett_status bennett_result_measurement(const bennett_result_t*, size_t i, bennett_qreg* reg, uint8_t* bits_out, size_t cap, size_t* nbits);

/* §10 C declaration: "uint64_t bennett_result_tape_length(const bennett_result_t*);" */
uint64_t bennett_result_tape_length(const bennett_result_t*);

/* §10 C declaration: "void bennett_result_free(bennett_result_t*);" */
void bennett_result_free(bennett_result_t*);

/* §10 C declaration: "const char* bennett_request_service(const bennett_request_t*);" */
const char* bennett_request_service(const bennett_request_t*);

/* §10 C declaration: "const uint8_t* bennett_request_payload(const bennett_request_t*, size_t* len);" */
const uint8_t* bennett_request_payload(const bennett_request_t*, size_t* len);

/* §10 C declaration: "void bennett_request_free(bennett_request_t*);" */
void bennett_request_free(bennett_request_t*);

/* §10 C declaration: "bennett_status bennett_tape_open(bennett_program_t*, bennett_run_id, bennett_tape_iter_t** out);" */
bennett_status bennett_tape_open(bennett_program_t*, bennett_run_id, bennett_tape_iter_t** out);

/* §10 Tape: "each an unsigned 64-bit little-endian field"; operands in §7.3 order, alen = 8 × arity. */
/* §10 Tape: "pointers zero-extended from ptr_width". */
/* §10 C declaration: "bennett_status bennett_tape_next(bennett_tape_iter_t*, const char** dev, const char** op, const uint8_t** args, size_t* alen);" */
bennett_status bennett_tape_next(bennett_tape_iter_t*, const char** dev, const char** op, const uint8_t** args, size_t* alen);

/* §10 C declaration: "void bennett_tape_free(bennett_tape_iter_t*);" */
void bennett_tape_free(bennett_tape_iter_t*);

/* §10 C declaration: "bennett_status bennett_estimate_run(bennett_program_t*, const char* region, bennett_token, bennett_estimate_t** out);" */
bennett_status bennett_estimate_run(bennett_program_t*, const char* region, bennett_token, bennett_estimate_t** out);

/* §10 C declaration: "size_t bennett_estimate_nresources(const bennett_estimate_t*);" */
size_t bennett_estimate_nresources(const bennett_estimate_t*);

/* §10 C declaration: "bennett_status bennett_estimate_get(const bennett_estimate_t*, size_t i, const char** name, const char** value_decimal);" */
bennett_status bennett_estimate_get(const bennett_estimate_t*, size_t i, const char** name, const char** value_decimal);

/* §10 C declaration: "const uint8_t* bennett_estimate_certificate(const bennett_estimate_t*, size_t* len);" */
const uint8_t* bennett_estimate_certificate(const bennett_estimate_t*, size_t* len);

/* §10 C declaration: "void bennett_estimate_free(bennett_estimate_t*);" */
void bennett_estimate_free(bennett_estimate_t*);

/* §10 C declaration: "bennett_status bennett_coverage_get(bennett_program_t*, bennett_coverage_t** out);" */
bennett_status bennett_coverage_get(bennett_program_t*, bennett_coverage_t** out);

/* §10 C declaration: "const uint8_t* bennett_coverage_jcs(const bennett_coverage_t*, size_t* len);" */
const uint8_t* bennett_coverage_jcs(const bennett_coverage_t*, size_t* len);

/* §10 C declaration: "void bennett_coverage_free(bennett_coverage_t*);" */
void bennett_coverage_free(bennett_coverage_t*);

#ifdef __cplusplus
}
#endif

#endif /* BENNETT_LAUNCH_H */

# Launch-estimate explicit accuracy, version 1

This is the normative additive `bennett.estimate-accuracy/1` contract for
ISA 0.9 §10.1. Its declaration header is `bennett_estimate_accuracy_1.h`;
compile with both `include/` and `spec/` on the include path. It extends
launch/0.9 without changing any existing exported function's signature.

```c
bennett_status bennett_estimate_run_at_accuracy(
    bennett_program_t* program, const char* region, bennett_token token,
    const char* variant, uint32_t inherited_controls,
    const uint8_t* params_jcs, size_t params_len,
    const uint8_t* aliases_jcs, size_t aliases_len,
    const char* epsilon_decimal, bennett_estimate_t** out);
```

All parameters except epsilon_decimal have precisely the meaning, validation,
copying and ownership rules of §10.1's alias-aware estimate. Every consistent
K91 partition is supported. The new parameter is required, not an optional
default: a NUL-terminated ASCII nonnegative finite decimal string in §11's
bindings.epsilon string form. The provider MUST copy and validate it during
the call and decode its exact rational value, without binary64 conversion or
34-digit resource-display rounding. A null pointer, malformed decimal,
negative value, NaN or infinity is E_ARG before producing an estimate or
effect. Zero is valid when explicitly supplied by the caller. Equivalent
decimal spellings denote the same requested value, as K60 specifies.

The copied value is the request's root accuracy, independently of the
presence or contents of an attached schedule. Reuse that schedule exactly
when every §5.4/§11 binding matches, including equality of bindings.epsilon
to this exact value. Otherwise use §11 defaults and construct the explicit
empty-entries effective schedule required by K12, binding this same root
value as epsilon. Never obtain the value from a stale schedule, a nested
region, a target entry or an inferred exact-gate-only profile.

The effective schedule serializes a default root value as a decimal string
with no exponent, sign, leading integer zeros (except 0) or trailing
fractional zeros; zero is "0". Matching an existing schedule compares exact
values under K60 and retains that document's original bytes and hash.
For omitted gate shares, §17.9 uses this root epsilon divided by its exact
inexact-operation count N; exact native gates use zero, and N=0 performs no
division. All explicit-share, achieved-error, capacity, realization and
certificate obligations remain unchanged. An estimate MUST resolve the full
padded realization for the requested accuracy; an exact-gate-only subset or
a bound over other accuracies cannot substitute for that result.

The certificate binds the effective schedule/hash and its root epsilon with
all existing K12/K13/K91 identities and exact resources. Accuracy is separate
from the args-only params_hash. A checker MUST reconstruct the request's
root value, effective schedule, shares and realization; agreement of display
strings or a producer's resource map is insufficient.

A launch provider may advertise "estimate_accuracy/1" in its immutable
launch/0.9 feature manifest only when it exports this exact function and
implements this entire contract, including all K91 partitions and K12
realization/resource/certificate obligations. Query, ownership and feature
selection follow §10.1. This is a launch extension, not a sched feature or
a change to either base ABI family's existing symbol inventory.

The accuracy policy of the existing entry points without epsilon_decimal
is explicitly pending in §10.1. This extension assigns them no default and
no new rejection rule. The explicit entry point fully specifies requests
with absent, matching and mismatched attached schedules; the pending legacy
policy does not narrow source programs or that capability.

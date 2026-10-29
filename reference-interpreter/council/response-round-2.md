# Editor response: council round 2

Inputs: `out-sol.json` (8 blockers), `out-codex.json` (7 blockers), reviewed
snapshot dc46d07. Member prefixes disambiguate IDs. This response is appendable
for the pending muse second pass. No specification or Python seed edits.

## SOL-R2-B05 — resolved

Coincides with codex R2-B07 below. `src/validation.rs::validate_action_sequence`
checks the exact predicate stack immediately after each qif arm, before any
selector replacement. Malformed bodies return §3.5, never an `expect` panic.
Pin: `r2_validation::qif_arms_cannot_pop_or_leave_predicates` (consumed selector,
extra push, nested balanced positive; parser and hash/check subprocesses).
RED: the exact consumed-selector witness panicked before the fix. GREEN passes.
No new specification question.

## codex R2-B07 — resolved

Same change and test as SOL-R2-B05; both arm boundaries checked.

## SOL-R2-B07 — resolved

Coincides with codex R2-B05. `validation.rs::classical_instruction` requires
Int or Ptr operands for icmp, in addition to equal types. Pin:
`r2_validation::icmp_requires_integer_or_pointer_operands` (token, f64,
qreg[1], parser and hash/check) and
`integer_and_pointer_comparisons_execute_and_reverse` (equal/unequal i8/ptr,
exact i1 result, full history reversal). RED: token comparison was accepted.
No new specification question.

## codex R2-B05 — resolved

Same change and tests as SOL-R2-B07, including the two-token CC witness.

## codex R2-B02 — resolved

`object.rs::validate_row` uses `host_count` for lifted load/store suffixes,
matching both engines. Pin: `r2_validation::lifted_memory_widths_are_canonical_at_load`
mutates sum_db load.i8 → i08 and heap store.i16 → i016 (the actual heap fixture
uses i16, not i8); parser and hash/check reject with §16.1. RED: load.i08 was
accepted. Canonical fixture execution remains covered by the existing suite.
No new specification question.

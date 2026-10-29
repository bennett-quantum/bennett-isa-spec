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

## SOL-R2-B02 — resolved

Coincides with codex R2-B01. `call_discipline.rs::Invocation` stores callee,
token, parsed ins/mem/args, target and fault. `validate_matched_oracle_calls`
never serializes/splits keys, nor discards list parse errors. Pins:
`r1_rules::r2_multi_input_invocation_keys_preserve_tuple_boundaries` (sum2
witness, changes to either input; unchanged pair returns (0,0,5) and reverses),
`r2_codex_changed_second_input_and_two_memory_regions` (codex exact witness,
positive (0,0,4), writes into either supplied region). RED: both reviewers'
changed-input class was accepted before the fix. No new question.

## codex R2-B01 — resolved

Same change and tests as SOL-R2-B02. Existing multiplicity and disjoint-global
store tests still pass.

## SOL-R2-B03 — resolved

`call_discipline.rs::validate_matched_oracle_calls` conservatively includes
every containing-function row as a qjumpr successor, matching §4.2's range,
including rows inside a block. Pin:
`r1_rules::r2_qjumpr_can_enter_an_uncall_at_a_non_label_row`: label+1 bypasses
the call and rejects §5.5 (RED: accepted); computing call at entry dominating
all numeric destinations accepts and runs/reverses to (0,0,5). No target-value
analysis or new spec question.

## SOL-R2-B06 — resolved

Coincides with codex R2-B04. `validation.rs::region_declarations` recursively
gathers lexical declarations; `validate_structured`/`declare_region` register
and validate each once at entry. Sequence occurrences are no-ops; duplicate
lexical declarations still reject. `borrow.rs::Compiler::declare_regions`
hoists allocations in root and inlined child functions too. Pin:
`r1_discipline::r2_regions_are_lexical_function_entry_declarations` (loop 2,
loop 0, use before text, qif, duplicate, borrow-compiler positive; all t,p basis
values, clean forward and adjoint, exact restoration). RED: loop 2 rejected
"region name already defined". No new question: §4.6 states function entry.

## codex R2-B04 — resolved

Same change and tests as SOL-R2-B06.

## codex R2-B03 — resolved

`validation.rs::operation_inverse` exchanges the two qinplace oracle names,
preserving token/target and the entire captured effective-control suffix.
`validate_action_sequence` already captures that suffix in `Discipline::change`.
Pin: `r1_qinplace::r2_qinplace_explicit_adjoint_cleans_workspace`: identity and
NOT/NOT (B=1/3), distinct callee names to verify the exchange, both workspace
basis values via copying t, forward/adjoint; changed predicate rejects §6.2.
RED: the identity-pair explicit cleanup rejected at load. No new question.

## SOL-R2-B04 — resolved

`borrow.rs::Compiler` now carries immutable classical bindings. Oracle opaque
identities contain the callee, resolved token and complete resolved classical
argument tuple; inlined permutation calls substitute the caller's bindings
for child formals. qinplace token identities use the same environment. Pin:
`r1_borrow::r2_opaque_borrow_calls_bind_classical_args_and_tokens_through_inlining`
rejects x/y in both direct and inlined witnesses (§5.6), accepts x/x and checks
all x,f,d basis states forward/adjoint. RED: direct x/y parsed successfully.
No new specification question.

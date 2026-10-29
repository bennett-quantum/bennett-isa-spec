# Editor response: council round 2

Inputs: `out-sol.json` (8 blockers), `out-codex.json` (7 blockers), reviewed
snapshot dc46d07. Member prefixes disambiguate IDs. This response is appendable
for the pending muse second pass. No specification or Python seed edits.

## Per-blocker status

| Member ID(s) | Status |
|---|---|
| SOL-R2-B01 | resolved |
| SOL-R2-B02; codex R2-B01 | resolved |
| SOL-R2-B03 | resolved |
| SOL-R2-B04 | resolved |
| SOL-R2-B05; codex R2-B07 | resolved |
| SOL-R2-B06; codex R2-B04 | resolved |
| SOL-R2-B07; codex R2-B05 | resolved |
| SOL-R2-B08; codex R2-B06 | resolved |
| codex R2-B02 | resolved |
| codex R2-B03 | resolved |

All 15 blocker IDs are resolved. No decline or partial resolution in this
round. Pending muse feedback is a subsequent pass, not an omitted input.

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

## SOL-R2-B01 — resolved

`validation.rs::Discipline::exchange` snapshots old dependency unions and
other provenance, records them on every output, and updates exchange logs
jointly; external transitive readers prevent cleanup of an other value.
`validate_action_sequence` routes swap, qexch and qexchm through it, including
region cell versions. Pin:
`r1_discipline::r2_exchanges_preserve_transitive_entangling_read_dependencies`
(all three negatives §6.2, doubled middle-CX positives, all small basis inputs
plus superposition, forward/adjoint and exact restoration). RED: swap witness
accepted before the fix. The differential generator now includes all three
exchange operations in compute and middle sequences. It exposed another
unsound accept: a joint exchange could cancel only one output's log while the
partner had an intervening write. Both outputs must now cancel together;
`r2_exchange_inverse_must_cancel_both_outputs_together` pins that witness.
No new spec question: these are the dependency unions printed in §6.1.

## SOL-R2-B08 — resolved

Coincides with codex R2-B06. `exact.rs::Amplitude::approximate` evaluates the
phase as a product of bounded limb phases, with complex squaring for exponent
bits beyond binary64's finite angle range; it never converts the combined
exponent to an f64 angle. The exact Laurent shadow still decides support,
cleanliness and equality, with no tolerance. Subnormal limb phases are scaled
before rounding. `QuantumState::exact_support` and the CLI use the corrected
display conversion without further changes. Pin:
`cli_xam::r2_finite_rotations_render_when_the_combined_angle_exceeds_binary64`
(four literal rz and four argument rz, both basis states, finite unit-modulus
amplitude 0.588863244801576 ± 0.8082326886001081i; reverse exactly [1,0]).
RED: run-action exited 1 with non-finite JSON diagnostic. No new question.

## codex R2-B06 — resolved

Same change and tests as SOL-R2-B08; finite f64 1e308 is not rejected.

## Nonblocking suggestions and additional controls

- codex S1 (qif cross-arm lifetimes): SPEC-QUESTIONS **28** records the
  §5.3 shorthand / §4.6 expansion conflict and the full-expansion reading.
  `action.rs::action_sequence` reverses else before then under adjoint, so
  unconditional lifetime operations reverse in the right order.
  `r1_discipline::r2_qif_cross_arm_unconditional_lifetimes_reverse_the_expansion`
  was red with "unknown live bundle d", now checks both p,t values and full
  restoration. No lifetime restriction was added.
- codex S1 (staging diagnostics): SPEC-QUESTIONS 21 now names 9–11 for
  target/schedule/interval work; `claims.rs::verify_claim_guarded` names
  deliverable **11** for interval claims. Numerical/channel claims remain
  Unverified. `r1_certificates::r2_interval_claim_diagnostics_name_deliverable_eleven`
  was red naming 10 and now passes for both claim kinds.
- `cli_xam::r2_qinplace_identity_workspace_cleanup_runs_and_reverses` pins
  the exact codex @g/@g workspace witness through forward, adjoint and reverse
  CLI forms for both t values, plus canonical sum_db(0), stack 16, B=10.
- `r1_fuzz_discipline::generated_borrow_frames_keep_differing_classical_bindings_distinct`
  generates 80 borrowed frames with differing x/y argument lists; an independent
  parity formula determines acceptance, and every accepted frame executes and
  reverses for all x/y and sampled borrowed/fault entries.

### Follow-up pinning from the extended sweep

The first extended ignored run found an additional unsound accept in the
exchange class (SOL-R2-B01): identical exchange text was reused after a partner
operand changed, so a later control snapshot could alias an earlier value.
`Discipline::exchanges` now binds each joint computation to its entry versions;
only an inverse whose **every** output still has that same joint computation
on top can reuse that identity. Pin:
`r1_discipline::r2_exchange_computations_pin_their_entry_versions` (red accepted,
now §6.2 rejection). The earlier joint-cancellation pin remains green.

For SOL-R2-B05 / codex R2-B07, a stack floor also prevents a body from consuming
an expansion-owned selector and pushing it back later. Pin:
`r1_discipline::r2_qif_body_cannot_temporarily_consume_its_implicit_selector`
(red accepted, now §3.5 rejection). Balanced user-owned predicate pairs remain
legal. No new specification question.

The next extended sweep exposed missing predicate **polarity** in memory
operation identities: complementary qif arms had equal address/control version
snapshots. Memory keys, generic change contexts, joint exchange contexts, and
outstanding diagonal phase keys now also retain predicate signs. Red-green pins:
`r1_discipline::r2_memory_exchange_and_target_writes_do_not_cancel_independently`
(the generated witness), `r2_memory_operations_pin_predicate_polarity`,
`r2_complementary_diagonal_phases_do_not_cancel`, and
`r1_qinplace::r2_qinplace_complementary_arms_do_not_cancel`. These extend the
SOL-R2-B01 / codex R2-B03 effective-control fixes, without a new reading.

### Frozen-snapshot RED audit

In addition to the development-time red runs above, the final regressions were
compiled against an archived dc46d07 crate in the ignored project-local `.tmp`.
The audit observed each exchange witness separately (swap, qexch, qexchm), both
literal and argument large-angle cases, both members' multi-input witnesses,
qjumpr, qinplace, borrow bindings, and the region loop/forward-reference/zero-loop
witnesses fail against dc46d07. Token, f64, qreg[1], load.i08, and store.i016
were also audited separately and all failed their rejection assertions. The
unchanged integer/ptr comparison positive passed on dc46d07. This archive is
verification-only, not a checkout or deliverable; no source outside the owned
crate was changed.

A subsequent full sweep was SIGKILLed by the OS. A deterministic small
regression reproduced geometric identity-text growth: 24 alternating
exchange/write events expanded to **715,850,424 bytes** of version keys.
`Discipline::value_ids` now interns collision-free snapshot references;
`update_log` and `record_change` register each complete key, while `snapshot`
references that key without recursively serializing its history. Every complete
identity and inverse comparison is retained; no threshold or rejection was
added. `validation::tests::exchange_entry_histories_have_compact_references`
was red on the expanded representation and now passes immediately. The full
normal and ignored suites are rerun after this change.

### Final round-2 verification

All commands ran headlessly with `DISPLAY`, `WAYLAND_DISPLAY`,
`DBUS_SESSION_BUS_ADDRESS`, and `BROWSER` unset,
`TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/.tmp`,
and the absolute `--manifest-path
/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml`.

| Command | Result |
|---|---|
| `cargo test --offline` | **202 passed**, 0 failed, 5 ignored (201 integration + 1 unit; 0 doctests) |
| `cargo test --offline -- --ignored` | **5 passed**, 0 failed; `--nocapture` retained the sweep counts |
| `cargo clippy --offline --all-targets -- -D warnings` | clean |
| `cargo fmt -- --check` | clean |
| `python3 /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/isa_check.py` | unchanged seed passes |

| Ignored sweep | Cases | Reported time |
|---|---|---|
| u8 division/remainder | 262,144 (a,b) pairs × 4 operations | 2.39 s |
| 17-wire borrow-frame execution | 2^17 basis entries | 11.83 s |
| checked borrow-frame certificate | 2^18 claim elements, both programs | 82.92 s |
| extended §6 differential generator | 200,000 programs: **64,717 accepted, all exactly clean**, 135,283 rejected | 259.39 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 12.90 s |

The accepted differential programs run on all eight interface basis inputs and
one superposed input, forward and adjoint. The sweep includes the new exchange
operations. No runtime width, iteration, precision, or program-size cap was added.
Specification and Python seed diffs against dc46d07 are empty. No push.

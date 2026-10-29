# Editor response: council round 3

Inputs: SOL out-sol.json and codex out-codex.json, both reviewing snapshot
2e61144. The first pass resolves five SOL blockers; the codex second pass below
covers R3-B01..B03 after rechecking the overlaps against 788a5e0. This response
remains appendable for muse. No spec or Python seed edits.

## SOL-R3-B01 — resolved

`src/call_discipline.rs::validate_matched_oracle_calls` records both payload
and fault deliveries for calls and uncalls as writes. Every outstanding input
reader is invalidated; only output XOR delivery retains the commuting exception.
The inverse consumes its own pre-row identity before invalidating other readers.
Pin: `r1_rules::r3_nested_oracle_deliveries_invalidate_outstanding_inputs`:
exact inc8 witness, another uncall delivery, fault delivery, parser and CLI
hash/check rejection (§5.5, exit 1, empty stdout); no-middle positive gives
(payload,code,steps)=(0,0,4), interpreter/machine agreement and prefix reversal.
Existing two-call/two-uncall multiplicity positive remains accepted.
RED: exact witness was accepted; GREEN: all 16 r1_rules tests pass.
No SPEC-QUESTIONS entry: §§4.2 and 5.5 explicitly specify the writes.

## SOL-R3-B02 — resolved

`validation.rs::validate_action_sequence` snapshots all mutable bundles and
mutable-region cells jointly, with immutable interface/cell/control reads;
`Discipline::joint_change` generalizes exchange tracking to forward/adjoint
call pairs. Every output inherits the OLD transitive union and other provenance.
Cancellation requires the complete output group, entry versions and unchanged
read/control versions; outside readers block other-value cleanup. Mutable cell
epochs and non-permutation window closures still advance on every call.
Pins: `r1_discipline::r3_action_calls_transfer_the_old_joint_dependencies`
(exact call-wrapped swap witness and mutable-region variant reject §6.2;
doubled-CX positives are exact identities on all basis inputs and superposition,
forward/adjoint). `r1_fuzz_discipline::generated_child_call_wrapping_preserves_bell_cleanup_obligations`
generates 12 direct/wrapped, odd/even CX variants; independent Bell/parity
reasoning requires rejection or clean execution over the full small interface.
RED: exact witness accepted. GREEN: 17 discipline + 3 active generator tests.
No SPEC-QUESTIONS entry: this is the explicit §6.1 union and §6.2 identity rule.

## SOL-R3-B03 — resolved

`borrow.rs::Application` retains callee/classical identity, output bit and full
symbolic input tuple. `Evaluation::{symbol,substitute,compose,apply}` substitutes
recursively inside applications at every Repeat composition and at the final
entry substitution; equal applications are re-interned. A substituted borrowed
input explicitly requires the existing verified §12 claim route. No frozen
opaque symbol remains across a changing-input substitution.
Pin: `r1_borrow::r3_opaque_loop_composition_substitutes_inside_applications`:
exact loop and unrolled witness reject §5.6 in parser/hash/check; invariant-input
positives preserve all 64 t,f,d basis entries and a superposition, forward/adjoint.
`r1_fuzz_discipline::generated_changing_input_opaque_loops_match_independent_parity`
generates 32 loops/nested loops, n=0..7, with/without input X; independent parity
requires precisely the valid borrow identities and checks all accepted programs
on all 64 entries plus a superposition, forward/adjoint.
RED: exact changing-input loop accepted. GREEN: 5 active borrow tests and the
new generator pass. SPEC-QUESTIONS 20 and module prose now distinguish exact
Boolean decisions from sound opaque-function symbolic proofs; unresolved
function-specific identities retain the verified certificate path. This corrects
the existing reading's description, with no new ambiguity or scope restriction.

## SOL-R3-B04 — resolved

`validation.rs::classical_instruction` resolves a normalized call's ctl operand
through `rd` and requires `qreg[1]` before region-call validation. It checks
oracle target/fault aliases and action inherited targets (including through
child calls) plus mutable-region aliases. `call_discipline.rs::validate_inherited_targets`
now also accepts qreg bindings, applying the same bit-for-bit target analysis
as structured calls. Forward type inference remains iterative.
Pins: `parser::r3_normalized_cc_controls_are_typed_and_disjoint_at_load`
(exact i8 witness, token, unknown, qreg[2], target alias; parser and hash/check
exit 1, empty stdout, §3.3); `registers::r3_typed_cc_controlled_calls_execute_and_restore_history`
(distinct qreg[1] control/target, all four basis pairs, exact t XOR c,
two-step completion, full reversal; also six-step definition-after-use-in-text
positive using select). RED: exact i8 witness accepted. GREEN: 10 parser tests
and the typed execution/reversal pin pass. No SPEC-QUESTIONS entry: §§3.3,
4.7 and 4.8 fix the operand type and disjointness requirement.

## SOL-R3-B05 — resolved

`borrow.rs::Compiler::compile` includes the entire effective control context
(normalized ctl plus inherited predicates), mut/in interface bundles and every
passed region cell in a non-permutation action call's `Node::Other` touch set.
Pin: `r1_borrow::r3_nonpermutation_calls_touch_all_effective_controls` rejects
the exact ctl-H witness, its inherited-qpred form and direct controlled-H,
all through parser and CLI hash/check (§5.6, exit 1, empty stdout). A perm_action
X child invoked twice under the borrowed control remains accepted and exactly
preserves every t,d basis input plus a superposed input, forward/adjoint.
RED: exact ctl witness accepted. GREEN: all 6 active r1_borrow tests pass.
No SPEC-QUESTIONS entry: §§4.7, 5.6 and 14.2 explicitly include controls.

All five first-pass SOL blocker IDs are resolved; no declines or partials.
The codex second pass is appended below; muse remains a subsequent pass. Deliverables 8–11 remain
staged as documented in PROGRESS.md, outside this round's received blockers.

## Final verification of this pass

All commands used absolute manifest paths, project-local TMPDIR and the four
display/browser variables unset. Normal tests: **210 passed, 0 failed, 5 ignored**.
Ignored tests (`--ignored --nocapture`): **5 passed, 0 failed**. Strict all-target
Clippy, formatting check and unchanged Python seed check pass. No dependencies
added. The five ignored sweeps and timings are recorded in PROGRESS.md; the
200,000-program sweep reported **64,717 accepted, all exactly clean**, 135,283
rejected. Dedicated call-wrapping and changing-input loop families run in the
normal suite in addition to the unchanged random sweep.

Conformance command: `cargo run --offline --manifest-path
/home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/Cargo.toml --
--bin /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/target/debug/bennett-isa-ref`.
Result: **445 pass, 0 fail, 45 unavailable, 490 total**, exit **2** for explicit
staging (same as baseline). No conformance/include/spec/docs edits or pushes.

## Codex R3-B01 — resolved-by c2d97f8 (SOL-R3-B01)

Re-ran the exact six-row identity-oracle witness against HEAD 788a5e0:
parser and CLI hash/check reject with §5.5, exit 1 and empty stdout. No new
semantic fix needed. `r1_rules::codex_r3_b01_exact_implicit_delivery_witnesses`
pins that object and a four-bit input overwritten by another call's fault
delivery. The five-row no-middle control, outer B=5 and token B=1, yields
(payload,code,steps)=(0,0,5), agrees with the step machine, and reverses to the
complete entry snapshot. Existing two-call/two-uncall multiplicity positive
also passes. All **17 r1_rules tests pass**. No new development RED claimed:
the earlier SOL fix already handles these witnesses; its RED is recorded above.
Re-read §§4.2 and 5.5 and SPEC-QUESTIONS: implicit deliveries explicitly write
both destinations, and unchanged ins are mandatory. No ambiguity or decline.

## Codex R3-B02 — resolved-by f83fc71 (SOL-R3-B02)

Re-ran the exact call-wrapped swap witness against HEAD: rejects §6.2.
Extended `r1_discipline::r3_action_calls_transfer_the_old_joint_dependencies`
with the CX-child witness; both reject in parser and CLI hash/check, exit 1,
empty stdout. The existing mutable-region analogue also rejects. Doubled-CX
positives preserve every t,m basis input (and region address) and a superposed
entry, forward and adjoint, with exact state equality and no workspace.
No semantic change needed: f83fc71 already jointly transfers old mut/ins/cell
and effective-control dependencies and cancels complete inverse output groups.

`r1_fuzz_discipline::generated_child_call_wrapping_preserves_bell_cleanup_obligations`
now generates **36** cases (swap, CX, mutable-region exchange; direct/called;
0..5 middle CXs). Independent parity demands **18 rejects / 18 accepts**;
all accepts execute over the eight-entry interface and a superposed input and
reverse exactly. Random seeds unchanged. All **17 discipline + 4 active
generator tests pass** (1 ignored sweep deferred to final verification).
No new semantic RED: these witnesses already pass after the SOL commit.
Re-read §§5.1, 5.7, 6.1–6.3 and SPEC-QUESTIONS: full joint dependency union
and exact inverse obligations are explicit; no new ambiguity or decline.

## Codex R3-B03 — resolved

RED witnessed for all three new regressions: exact adjoint child, nonzero
image and zero-trip declaration each rejected with §5.6's unclean-release
error before the fix. `borrow.rs::Compiler::compile_function` now emits the
complete local-region entry/body/exit for top-level analysis and every inlined
child. Clean `Node::Alloc` and explicit unconditional image-X `Node::Flip`s
replace image-carrying allocations; the exit is the entry's exact adjoint.
The call adjoint reverses the complete lifetime, preserving image preparation
and restoration. No cleanliness or borrow-factorisation diagnostic is weakened.

Pins: `r1_borrow::codex_r3_b03_adjoint_child_local_region_exact_witness`,
`codex_r3_b03_adjoint_child_nonzero_image`, and
`codex_r3_b03_adjoint_child_zero_trip_region_declaration`. Each exercises
forward/adjoint children, with/without an inherited borrowed predicate,
all four t,b basis values, superposition, and b entangled with an untouched
external reference. Parent execution and its adjoint each equal entry exactly;
no workspace remains. The zero-trip case reads the image before its lexical
declaration. The nonzero image is the requested one-byte ff for region[1,4].
CLI run-action for t=0/1 returns the unchanged basis state and reverse reports
equals_entry=true for every variant; hashes check. All **9 active borrow tests
pass** (1 ignored), and the full normal suite is **214 passed, 0 failed,
5 ignored**. Re-read §§4.6, 5.3 and 5.6; no new ambiguity. SPEC-QUESTIONS 28
retains its proposed wording and now references these local-lifetime pins.

## Codex second-pass final verification

All three codex blockers resolved, no declines or partials. Commits: 377e807
(B01 exact pins, resolved-by c2d97f8), faf6dff (B02 pins/generated coverage,
resolved-by f83fc71), fbe8320 (B03 lifetime fix and pins).

After the fixes, rebuilt offline and ran the documented standalone conformance
harness with the absolute reference `--bin`: **445 pass, 0 fail, 45 unavailable
of 490**, exit **2** for explicit staged work, unchanged from baseline.
`CARGO_TARGET_DIR` redirected build artefacts into the interpreter's ignored
`.tmp`; conformance sources remained read-only. Then the normal Rust suite
passed **214 tests, 0 failed, 5 ignored**; the ignored suite passed **5 tests,
0 failed**. Strict all-target Clippy, formatting check and unchanged Python seed
check pass. The 200,000-program sweep again reports **64,717 accepted, all
clean / 135,283 rejected**, with the original seeds unchanged. PROGRESS.md
records each exhaustive sweep's cases and timing. All validation was headless
with the requested absolute project-local TMPDIR and absolute manifest paths.
No spec changes, dependency additions or pushes.

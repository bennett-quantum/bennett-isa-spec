# Editor response: council round 4

Inputs: codex out-codex.json and sol out-sol.json, snapshot 8e11fe7. This
response is appendable for the pending muse pass. Owned paths are exclusively
reference-interpreter; ISA 0.8, the Python seed and random seeds are unchanged.

## R4-B01 — resolved

RED: `r1_borrow::r4_loop_local_returns_cannot_be_restored_by_later_borrowings`
accepted the exact loop-local borrow/X/return witness before the fix.
`src/borrow.rs::Node` now retains Borrow/Return lifetime boundaries;
Compiler emits them and reverses them under adjoints. Evaluation checks
restoration of every returned wire and independence of every other live
output at Return, then removes the returned system. Repeat proves these
boundaries on the generic body BEFORE composing its surviving-wire map by
squaring. Borrowing starts a fresh independent symbolic entry. Lifetimes
spanning a loop remain in its map and are checked only at their actual return.
Zero-trip loops apply no lifetime; no unrolling threshold or new cap is added.
The exact evaluator includes surrounding allocation/cleanup so a clean
allocation preceding Borrow is known clean if freed before Return.

Pins: the exact witness, straight-line unrolling (reused name), single lifetime,
nested even loops, and a restored-but-dependent CX output all reject at parse
and CLI hash/check with §5.6, exit 1 and empty stdout. The positive pin
`r4_borrow_lifetimes_inside_and_across_loops_preserve_full_state` covers spanning
and iteration-local lifetimes, nesting, and surrounding clean allocation.
Every positive runs forward/adjoint over all t,b basis pairs, superposition,
and a borrowed wire entangled with an untouched reference, with exact equality.
`r1_fuzz_discipline::generated_loop_local_borrow_returns_match_alpha_renamed_unrollings`
adds 32 deterministic cases (trip counts 1..4, one/two X gates, loop/unrolling,
with/without outer even loop). Independent per-return parity gives precisely
16 rejects / 16 accepts; every accept passes the same full-state checks.
GREEN: 11 active borrow + 5 active generator tests; whole normal suite
217 passed, 0 failed, 5 ignored. No SPEC-QUESTIONS entry: §§4.3, 4.6, 5.1
and 5.6 explicitly require return AFTER its frame and factorisation on B.

## R4-B02 — resolved

RED: exact select-created interface alias in the library and CLI produced an
internal §3.3 error instead of fault JSON. Replaying all five new library
regressions against pre-fix classical.rs also witnessed the control-target
internal error, region-interface internal error, and an oracle alias silently
completing. The combined CLI pin initially failed at the exact action witness.

`src/classical.rs::ClassicalMachine::region_record` validates physical wire
uniqueness in every interface bundle and passed region BEFORE handle-id
deduplication. Effective controls are checked against oracle target/fault
wires, transitive action interface targets and mutable-region cells, before
execution or lifecycle changes. `call_discipline.rs::interface_targets` is
shared with the static checker. Only the specified runtime alias preconditions
are marked `trap`; simulator/internal failures still propagate as diagnostics.
The existing step path pushes the ordinary terminal fault-4 history frame.
Control/read-only-input overlap remains valid and bookkeeping still deduplicates
its handle. No gate runs and no handle is consumed when validation fails.

Pins in `tests/registers.rs`: `r4_dynamic_interface_alias_is_an_atomic_reversible_cc_and_r_fault`,
`r4_dynamic_control_target_alias_is_an_atomic_reversible_cc_and_r_fault`,
`r4_dynamic_oracle_input_output_alias_is_an_atomic_reversible_cc_and_r_fault`,
`r4_dynamic_control_can_overlap_a_readonly_input`, and
`r4_dynamic_alias_checks_cover_regions_block_parameters_and_child_targets`.
Exact select witnesses run in both CC and R: c=1 records fault 4 at two steps,
no results, unchanged quantum state AND handles; c=0 completes at three steps,
target 1, read-only source 1. Both restore the complete entry snapshot under
reversal. Extra pins cover passed-region aliases, block parameters, nested
child targets and directly aliased R entry bindings.

`cli_xam::r4_dynamic_region_aliases_report_failed_json_and_reverse_the_prefix`
pins all four families for run-cc/reverse (CC and R), plus run-program (CC).
The invalid branch exits 0 with FAILED/fault=4/steps=2/results=[], one named
FAULT_UB_TRAP record; reverse exits 0 with equals_entry=true after the faulted
prefix. Disjoint branches remain COMPLETED at three steps with exact reversal.
GREEN: 11 register tests + 29 CLI tests, and the full normal suite: **223
passed, 0 failed, 5 ignored**. No SPEC-QUESTIONS entry: §§3.3, 4.7 and 7.4
explicitly prescribe physical disjointness, the trap and prefix reversal.

Both received blockers are resolved; no declines or partials. This response
remains appendable for sol/muse.

## Final verification of this received pass

Blocker commits: **40ef8ce** (R4-B01), **09fe805** (R4-B02). Rebuilt offline,
then ran the exact standalone conformance harness from conformance/PROGRESS.md
against the absolute reference binary: **445 pass, 0 fail, 45 unavailable /
490**, exit **2** for the previously disclosed deliverables 8–11. Conformance
and headers were read-only; build artifacts went into the interpreter's `.tmp`.

Final normal suite: **223 passed, 0 failed, 5 ignored**. All ignored tests:
**5 passed, 0 failed** (u8 division/remainder, 2^17 borrow basis entries,
2^18 certificate elements, 200,000 generated discipline programs, all prefixes
for 256 recursive inputs). The discipline sweep reports **64,717 accepted,
all exactly clean / 135,283 rejected**; the new 32-case lifetime family runs
separately in the normal suite. PROGRESS.md records runner timings.

Strict all-target Clippy, formatting check, unchanged Python seed check and
diff check pass. Every verification command used absolute manifest paths,
DISPLAY/WAYLAND_DISPLAY/DBUS_SESSION_BUS_ADDRESS/BROWSER unset, and
TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/.tmp.
No dependencies, spec edits, seed changes, declines or pushes. README retains
the failed-result promise and documents lifetime and dynamic-alias semantics.

## SOL-R4-B01 — resolved

RED: `cli_xam::sol_r4_b01_tiny_born_branch_is_selectable` selected 1 instead
of 0 at theta=1e-162, draw=0. Independently,
`quantum::sol_r4_b01_tiny_projection_preserves_entangled_residual_and_second_born_weights`
failed on nonfinite conditional amplitudes.

`quantum.rs::measurement_branches` now derives exact Laurent-polynomial
weights from the exact amplitude shadows. `registers.rs::measure` compares
cumulative weights against draw times the total weight, preserving ascending
outcome order and strict CDF boundaries. `exact/measurement.rs` decides exact
zero first, then refines outward-rounded dyadic enclosures until the sign is
proved. Phase enclosures use a Taylor tail bound on |x| <= 1/2 and repeated
squaring; sqrt(2) uses bounding binary digits. There is no precision cutoff,
probability floor or guessed clamp. Projection display amplitudes use refined
shadows and a shared binary exponent before normalization; subsequent Born
weights use norm ratios of the unnormalised shadows. Binary64 probability
fields remain diagnostic approximations and may display zero for a positive
weight. README documents this distinction.

Pins: the exact memoryless ptr_width=32 CLI witness at 1e-100, 1e-161,
1e-162, 1e-200 and the smallest positive binary64 angle: draw=0 selects 0,
draw=0.5 selects 1; angle=0 selects 1. Each completes in three rows, without
fault; every measured reversal is refused. The residual entangled-pair pin
checks finite normalized amplitudes and a second measurement's independent
1/2,1/2 probabilities and correlations. A third pin,
`quantum::sol_r4_b01_projection_recovers_amplitudes_below_binary64_range`,
uses two tiny rotations (amplitude ~2.5e-401), projects a residual |+>, then
cleans it with H and releases it exactly.

GREEN: full normal suite **226 passed, 0 failed, 5 ignored**. No new spec
question: §§3.3, 4.4 and 14.2 specify this behavior directly. No seed changes.

## SOL-R4-B02 — resolved

RED: `r1_certificates::sol_r4_b02_translation_claim_must_prove_its_complete_named_frame`
returned Proof for the exact two-gate truncated frame. The separately run
nested-line positive failed with `unknown instruction ordinal` before the fix.

`claims.rs::check_translation_witness` now resolves every V/W point AND both
frame endpoints through the shared canonical §9.2 resolver in `metadata.rs`.
`borrow.rs::check_translation_frame` uses an opt-in trace of the existing
borrow compiler, retaining physical wires, live borrow identities, inherited
controls, loop structure and complete instruction expansion. It requires the
actual contiguous named frame to equal V;W;reverse(V);W, with the listed V/W
points naming its leading instructions in order. It checks X/B side conditions
on the compiled flips, requires B to name live borrowed wires and cover every
borrowed touch, and rejects omitted adjacent touches in the permutation run.
Lifetime changes/non-permutation operations bound that run. No source line,
intervening operation, swap, phase or control can silently disappear. Tracing
is disabled in ordinary borrow validation; no exhaustive limit is added.

Pins exercise both `verify_claim` and `verify_certificate`, leaving
`verification.borrow_frames` empty: truncated [1,2] rejects, full [1,4] proves;
nonexistent/reversed endpoints, nonexistent or misplaced V/W, fake B and changed
second W reject. `sol_r4_b02_translation_claim_uses_nested_line_ordinals`
proves gates 2..5 inside a qloop and rejects its truncated frame.
`sol_r4_b02_translation_frame_rejects_omissions_and_checks_reverse_order`
proves a two-gate V in exact reverse order, rejects a skipped intervening gate,
and rejects a four-gate slice omitting adjacent borrowed touches. The negative
objects themselves remain valid full circuits, so the claim checker is tested
independently of parse-time factorisation. README documents the complete check.

GREEN: all 3 new structural regressions pass; whole normal suite: **229 passed,
0 failed, 5 ignored**; strict Clippy clean.
No SPEC-QUESTIONS entry: §§5.6, 9.2 and 12.1 specify the checks directly.

## SOL-R4-B03 — resolved

RED: `r1_certificates::sol_r4_b03_expected_program_uses_signature_positions_for_renamed_bundles`
failed on element 1 of the exact four-element identity-vs-renamed-identity
witness (subject closure e153e677…, expected program 44e334e5…). The region
analogue independently failed on element 1 before the fix.

`claims.rs::Component` now identifies interface bundles by (ABI group, slot)
and region cells by (ABI group, slot, cell index). `domain` resolves these
positions once against the subject; `execute_element` binds those same
positions to either function, independently of local names. Borrowed pool
components retain their domain order. Deliberately omitted partial-domain
inputs still initialize to zero, and output tuples retain signature order.
Both exhaustive and sampled comparisons use this common representation.

Pins: the exact four tuples now return Proof through both verify_claim and
verify_certificate, as do reversed domain enumeration order and two extra
borrowed pool components in deliberate z,b order. Omitting p still yields
Evidence(2). A single-X reference still fails with a §12.1 full-tuple mismatch.
`sol_r4_b03_expected_program_maps_renamed_region_cells_positionally` proves
all 16 interface/memory tuples for m→n and checks a partial cell-1 domain
(Evidence(8)); a reference changing only memory rejects.
`sol_r4_b03_positions_distinguish_same_width_slots_and_crossed_names` uses
two mutable bundles and reuses t/u/p in different groups in the reference:
all eight tuples agree positionally; writing the wrong mutable slot rejects.
README documents positional input binding and retained partial-domain rules.

GREEN: full normal suite **232 passed, 0 failed, 5 ignored**. No new ambiguity:
§§5.1, 9.4 and 12.1, consistently with SPEC-QUESTIONS 21, determine the ABI
and tuple correspondence. All three SOL blockers are resolved; no declines,
partials, dependencies, Python/random seed changes or spec edits.


## Final verification of the SOL second pass

Separate blocker commits: **3ba7154** (SOL-R4-B01), **21da2cc**
(SOL-R4-B02), **d7a14fd** (SOL-R4-B03). All three resolved, no declines or
partials. The response remains appendable for muse's pending review.

Rebuilt the binary offline, then ran the exact standalone conformance command
from conformance/PROGRESS.md, keeping conformance/include read-only and build
artifacts in the interpreter's ignored `.tmp/conformance-target`:
**445 passed, 0 failed, 45 unavailable / 490**, exit **2** for existing staging.

Final normal suite: **232 passed, 0 failed, 5 ignored**, 5.55 s summed runner
time. Ignored suite: **5 passed, 0 failed**, 336.09 s summed runner time,
including 2^18 checked certificate elements and the 200,000-program discipline
sweep (**64,717 accepted, all exactly clean / 135,283 rejected**). Strict
all-target Clippy, formatting, unchanged Python seed and diff checks pass.
PROGRESS.md records individual sweep counts/times and retained log names.

All validation used absolute manifest/binary paths, the specified local
TMPDIR, and DISPLAY/WAYLAND_DISPLAY/DBUS_SESSION_BUS_ADDRESS/BROWSER unset.
No dependencies, spec/conformance/header edits, seed changes or pushes.

## MUSE-R4-B01 — resolved (by 40ef8ce; exact muse pins in this pass)

Input: `out-muse.json` and the complete round-4 report; implementation starts
from 8fa5d98 with the read-only conformance update at 92c26af. This blocker
shares the lifetime/factorisation cause fixed in pass 1, rather than requiring
a second, different loop model. No blocker is declined or partial.

RED: copied the committed 8e11fe7 crate into the interpreter's ignored `.tmp`
and added the new regression/support files. Running
`r1_borrow::muse_r4_b01_each_iteration_has_an_independent_borrowed_entry`
there failed on the EXACT `qloop 2 { d = qborrow any 1; cx d t; qreturn d }`
object: Object::parse accepted it. The same new test passes on the current
implementation. Log: `.tmp/r4-muse-borrow-red.log`. This replay did not replace
or edit live source, spec, conformance or the Python seed.

The existing `Compiler::compile` emits Borrow/Return nodes, including inside
inlined permutation children. `Evaluation::apply(Return)` proves both identity
on the returned wires AND absence of their entry variables in every other live
output, before removing the returned system. Repeat checks that generic body
before squaring its surviving-wire map. Thus each iteration-local beta is
universally quantified independently, while a lifetime spanning the loop stays
carried and can be restored by the whole loop. This pass clarifies this invariant
on Node::Repeat; no new cap, unrolling strategy or semantic special case is added.

Named pins in `tests/r1_borrow.rs`:

- `muse_r4_b01_each_iteration_has_an_independent_borrowed_entry`: both CX
  directions, single lifetime, trips 2 and 4, nested 2x2 and independently named
  unrolling. Every negative requires Object::parse Err plus CLI hash/check exit
  1, empty stdout and §5.6. For d→t, t'=t XOR beta at each return; for t→d,
  d'=beta XOR t. Even iteration counts cannot remove either per-return failure.
- `muse_r4_b01_carried_and_local_borrow_cx_identities_have_no_trip_cap`:
  borrowing outside the two-iteration loop, and doubled-CX local lifetimes at
  trips 2 and **1,099,511,627,776**, for both CX directions. Every positive checks
  hashes and exact forward/adjoint state equality over all t,b basis entries,
  superposition, and b entangled with an untouched reference.

New independent generated families in `tests/r1_fuzz_discipline.rs`:

- `generated_fresh_borrow_cx_loops_match_per_lifetime_maps`: 64 cases, **32
  accept / 32 reject**; both CX directions, trips 1..4, one/two gates per
  lifetime, nested loops, alpha-renamed unrollings. The oracle is each lifetime's
  map, never whole-loop parity or simulator agreement. Every accept gets the
  full basis/superposition/entangled-reference identity check.
- `generated_nested_calls_with_regions_and_local_borrows_match_boolean_maps`:
  128 cases, **96 accept / 32 reject**, direct and doubly nested calls, renamed
  interfaces, passed mutable-region cells, iteration-local borrowed pools,
  loop bounds 1..4 and nested loops. From V=(b ^= p), W=(t ^= b; r[p] ^= b),
  the independent map for V;W;V;W is t ^= p, r[1] ^= p, with r[0],p,b unchanged.
  Omitting the second W with odd multiplicity must reject irrespective of loop
  parity. All accepts compare the complete state against that map on 32 basis
  inputs, a full superposition and an entangled borrowed reference, then reverse
  exactly. No RNG, Python seed or existing sweep expectation changed.

No SPEC-QUESTIONS entry is needed: §4.6 explicitly says "body n times", §5.1
requires each qreturn after its frame, and §5.6 requires identity on EVERY
borrowed bundle and its external reference. Readings 20 and 28 are unchanged.

## Non-blocking round-4 suggestions — dispositions

All three JSON `suggestions` arrays and criterion-level `change` proposals
were read. The table includes proposals already completed by passes 1–2;
non-blocking coverage declines below are not blocker declines.

| Source / suggestion | Disposition and reason |
|---|---|
| codex suggestions / C7: lifetime loops versus unrollings and dynamic aliases | Applied in pass 1; this pass adds the fresh-CX 64-case family and exact muse pins. Existing select/block-parameter CC/R alias tests still pass with FAILED/fault=4 and reversible prefixes. |
| muse suggestions 1 / C8: refuse --samples 0 | Applied in main.rs, README and `cli_xam::muse_r4_step_check_requires_a_nonempty_sample`; zero exits 1 with §17.13 and empty stdout; one sample still runs. Witnessed RED: the old CLI returned cases=0/status=ok. |
| muse suggestions 2 / C8: cite rules on malformed CLI numbers/HEX | Applied: shared budget parsing cites §5.4, including host-counter overflow; memory/HEX diagnostics cite §16.1. `cli_xam::muse_r4_malformed_options_cite_the_isa_without_partial_output` pins all four exact reported options. Archived old CLI reproduces each bare error independently in `.tmp/r4-muse-options-red.log`; the new test was also witnessed RED live. |
| muse suggestions 3: handle-map and SSA error expects | Applied: all seven register transition expects use a fallible §3.3 status helper; the SSA retry invariant now returns a §3.3 error if its recorded diagnostic is unexpectedly absent. Existing ownership/alias/reversal tests verify normal behavior. |
| muse suggestions 3: signature Json::Array expects | Declined: these arrays are constructed as Json::Array immediately before reading them, and header validation checks singleton arity; making the infallible public signature API return Result for this unreachable invariant is not a pure correctness gain for validated objects. |
| muse suggestions 4 / C7: broaden generated structured borrowing | Applied with the 64-case local-borrow family and the 128-case nested mutable-region/call/pool family; existing seeds and the 200,000-case random distribution are unchanged. |
| muse suggestions 4 / C7: also add qinplace, ctl/adj oracle calls, while selectors and indexed multi-bit random families | Declined in this pass: these are separate coverage projects with different semantic oracles; changing the old random distribution would obscure the retained sweep baseline. Existing instruction support and dedicated tests remain in scope and unchanged. |
| sol suggestions: nested calls with mutable regions and borrowed pools | Applied: the 128-case Boolean-map family checks the full interface/memory/borrow/reference state and its exact adjoint, independently of interpreter-to-interpreter agreement. |
| muse suggestions 5: adopt readings into spec 0.9 | Declined here: spec/ is explicitly read-only for this task; all proposed wordings remain recorded in SPEC-QUESTIONS.md. |
| muse suggestions 6: complete staged deliverables 8–11, first F probes | Retained as whole-ISA work, not falsely marked complete; the current pass verifies the updated harness and keeps every unavailable outcome explicit. F-link-conflict/F-frame-live-ranges remain the proposed first F promotions when those surfaces land. |
| codex C2 / muse C2: independent two-member SCC envelope | Applied in `identity::council_r4_mutual_scc_has_independent_literal_envelopes`: literal LF bytes, f/g name order, @$0/@$1 references, no intra-SCC dependency lines, concatenation and ASCII member suffixes; FIPS-tested SHA-256 only builds the expectation. |
| sol C2: independent CC envelope with service and model | Adapted in `identity::council_r4_cc_envelope_includes_service_but_not_unused_model`: literal service/layout/LF bytes and exclusion of a declared unused model. A single function cannot directly use both: §4.8 makes effect R-only and hostcall CC-only. Existing R model envelope/hash pins remain. |
| codex C3 / sol C3: literal recursive step/stack boundaries | Applied in `oracle::council_r4_sum_db_literal_step_and_stack_cutoffs`: n=2 at B=38/37 and stack=47 gives (3,0,38)/(0,1,37)/(0,3,29); n=0 at B=9/10 gives (0,1,9)/(0,0,10). |
| muse C4: small recursive every-prefix sweep in normal tests | Applied for n=0..3, 132 prefixes through the first idle-HALT step, in `step_machine::council_r4_small_recursive_inputs_gate_every_prefix_in_normal_suite`; the full ignored 256-input sweep is retained. |
| codex C4 / sol C4: inspect private call-record fields | Declined: no incorrect record was identified; inspecting private slots/stack requires a new inspection API, while complete pre/post snapshot inversion, every-prefix checks and literal outcomes already gate this path. |
| muse C3: add a fault-to-fixture index to all fixture comments | Declined: no incorrect fixture derivation was identified; a second index duplicates the existing fixture/test inventory rather than correcting it. |
| muse C5: one new registers.rs test name for each trap cause | Declined: reorganizing every trap witness into registers.rs is a separate test-organization expansion; this pass retains the current named fault/reversal pins and makes no trap-code semantic change. |
| sol C1/C5/C6/C7/C8 and codex C1/C5/C6/C8 | Applied by the five prior blocker fixes and their independent regressions; all retained tests pass in this pass. |


## Final verification of the MUSE third pass

MUSE-R4-B01 is resolved-by **40ef8ce**, now pinned by its exact independent
muse witnesses, huge-bound positives and two new generated families. No new
borrow semantic fix was necessary or claimed. All six round-4 blocker IDs
(codex 2, SOL 3, MUSE 1) are resolved; none is declined or partial. The
non-blocking dispositions above deliberately distinguish coverage suggestions
from correctness requirements. SPEC-QUESTIONS.md and the accepted ISA 0.8
are unchanged.

After rebuilding the binary offline, the exact standalone harness command
from conformance/PROGRESS.md against the updated **92c26af** conformance crate
reported **445 passed, 0 failed, 46 unavailable / 491**, exit **2**. The extra
unavailable target case is the documented harness addition; no unavailable
outcome is silently skipped or counted as a pass. Conformance/include stayed
read-only; CARGO_TARGET_DIR kept build artifacts in the interpreter's `.tmp`.

Normal suite: **242 passed, 0 failed, 5 ignored**, 12.05 s summed runner time.
Ignored suite: **5 passed, 0 failed**, 596.07 s summed runner time, including
2^17 borrow entries, 2^18 certificate elements, every recursive prefix for all
256 inputs, and the 200,000-program discipline sweep (**64,717 accepted, all
exactly clean / 135,283 rejected**). The two new deterministic families run
separately in the normal suite; existing seeds/distribution are unchanged.
Strict all-target Clippy, formatting, unchanged Python seed and diff checks
pass. PROGRESS.md records per-sweep times and log paths.

All validation used the required absolute manifests/binary, with DISPLAY,
WAYLAND_DISPLAY, DBUS_SESSION_BUS_ADDRESS and BROWSER unset and
TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/.tmp.
Only reference-interpreter paths changed; no dependencies, spec/docs/header/
conformance/worklog/HANDOFF/CLAUDE edits, seed changes or pushes. The full-ISA
bead remains in progress for staged deliverables 8–11.

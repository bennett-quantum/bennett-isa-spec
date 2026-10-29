# Editor response: council round 4

Input so far: codex out-codex.json, snapshot 8e11fe7. This response is
appendable for the pending sol/muse second pass. Owned paths are exclusively
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

GREEN: all 3 new structural regressions pass; whole normal suite is green.
No SPEC-QUESTIONS entry: §§5.6, 9.2 and 12.1 specify the checks directly.

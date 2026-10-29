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
remains appendable for sol/muse. Final exhaustive/conformance gates follow.

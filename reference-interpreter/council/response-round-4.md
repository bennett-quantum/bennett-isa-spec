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

## R4-B02 — implementation in progress

Dynamic physical alias validation and reversible CC/R fault-frame pins follow
in the next blocker commit. Final exhaustive/conformance validation follows
both received blockers. No declined capabilities or scope changes.

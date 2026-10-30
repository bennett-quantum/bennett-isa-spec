# Editor response: council round 5

Input so far: SOL out-sol.json against 5d6d03e. This response remains
appendable for the pending codex/muse second pass. ISA 0.8, the Python seed,
random seeds and conformance sources are unchanged; owned paths are exclusively
reference-interpreter/.

## SOL-R5-B01 — resolved

RED: `r1_certificates::sol_r5_b01_certificates_cannot_waive_completed_lifetimes`
accepted the exact certified X witness. Both literal interface envelopes and
object digests are independently SHA-256 checked before parsing; construction
does not parse an invalid uncertified subject. X and CX subjects, with and
without correctly bound certificates, now reject at Object::parse and CLI
hash/check (exit 1, empty stdout, §5.6).

`borrow.rs` distinguishes a definite Boolean/non-permutation failure from an
unresolved opaque-function identity. Only the latter can invoke the library
route. `claims.rs::check_borrow_independence` additionally audits every actual
qreturn during full-domain replay, including inlined/nested action calls.
`quantum.rs` retains opt-in basis observations: the returned factor must equal
its entry, and every other live wire must be independent of that entry at that
boundary. Comparisons hold all other input bits fixed. Prior completed frames
have already restored their independent entries, so subsequent physical pool
reuse cannot conceal an invalid earlier frame. Audited replay visits every
iteration rather than skipping a period with unobserved internal boundaries.
Normal execution retains cycle skipping and allocates no audit records.

The existing round-4 lifetime negatives, carried-even-loop and doubled-X/CX
positives, entangled-reference tests and round-4 certificate/structural pins
remain green: targeted borrow/certificate run **24 passed, 0 failed, 2 ignored**.
No new ambiguity or blocker decline. §4.3 requires qreturn to be the matched
inverse after its frame; §§5.1/5.6 require the property per completed lifetime.
Final whole-suite/conformance verification follows after all received blockers.

## SOL-R5-B02 — resolved

RED: `r1_borrow::sol_r5_b02_clean_surrounding_loop_controls_are_reachable_invariants`
rejected the exact clean-d witness with §5.6. `Evaluation` now records
restoration equations and Boolean-derivative independence obligations at each
Return (and cleanliness equations at Free). Repeat composes both its carried
map and these obligations by doubling: obligations in a later block are
substituted through the preceding block's map. Only then are actual entry
values substituted. All obligations must vanish, independently of the final
map; repeated constraints are deduplicated, never parity-cancelled. No unrolling
bound or wire/trip cap is introduced.

The exact positive accepts at trips 1,2,3,2^40, with nesting; hash/check pass,
and forward/adjoint preserve all interface/borrowed basis entries,
superposition and an external entangled reference. Alpha-renamed unrollings
agree. Setting d=1 before the loop and restoring it afterward rejects at
every bound, including even counts. An additional d-toggle witness checks the
second reachable entry: first-entry validity alone cannot admit it.

GREEN: borrow/certificate/generator targets **32 passed, 0 failed, 3 ignored**,
including all round-4 lifetime and opaque-composition pins. No spec question:
§§4.3/5.2/14.2 fix allocation at zero and §4.6 requires the body n times.

## SOL-R5-B03 — resolved

RED: exact CLI and library witnesses independently failed with `§3.3: scalar
wire required`. `action.rs::infer_widths` now scans live producing/consuming
lifetimes in source order and records widths by instruction line identity,
which is retained by body cloning. Both qalloc/qborrow and inverse qfree/qreturn
use their own row's resolved width; `produce` cannot overwrite another row's
metadata. qcopy/quncopy and qload/qunload participate in lifetime resolution.

`quantum::sol_r5_b03_consumed_workspace_names_keep_their_own_widths` pins the
literal interface/object hashes, all basis inputs and superposition, then
mixed qalloc/qcopy/qload/qborrow lifetimes at widths 1/2 with nested loops,
alpha-renamed counterparts, all eight t/pool basis entries, superposition and
an entangled reference. Each direction is independently identity and leaves no
workspace. Producing a still-live spelling remains rejected.
`cli_xam::sol_r5_b03_reused_workspace_widths_run_and_reverse` checks hash-bound
check, forward/adjoint at t=0/1, exact state output and equals_entry reversal,
plus a complex superposition. GREEN: quantum/CLI targets **42 passed, 0 failed**.
No ambiguity: §§4.3/5.1 fix each producer's width and cleanup, and §5.3 explicitly
renumbers versions under adjoints.

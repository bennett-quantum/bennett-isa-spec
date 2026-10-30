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
qreturn and each non-permutation frame boundary during full-domain replay,
including inlined/nested action calls.
`quantum.rs` retains opt-in exact-state observations: the returned factor must equal
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

## SOL-R5-B04 — resolved

RED: `quantum::sol_r5_b04_sparse_observer_reports_the_conditional_state`
reported (1/sqrt(2),0) for the exact two-wire witness instead of (1,0).
`QuantumState` now marks projected conditional states. `exact_support` retains
exact support and emits normalized amplitudes through the extended-exponent,
refined evaluator already used by projection's dense cache. Gates/allocation/
clean release retain the marker, and `set_state_vector` explicitly resets it
without imposing normalization. Basis-only projection also normalizes its
conditional phase. Exact shadows remain unnormalised for support, cleanliness
and subsequent Born ratios; no probability floor or reversal rule changed.

The exact two outcomes report (1,0) and agree with dense output; H on the
survivor reports two amplitudes 1/sqrt(2). Existing tiny entangled-residual and
below-binary64-range tests now also check finite sparse amplitudes, unit norm,
dense agreement before/after gates, and unchanged subsequent Born outcomes.
Unnormalised user vectors retain their supplied scale, including replacing a
previously projected state. GREEN: quantum/register/CLI targets **54 passed,
0 failed**. No ambiguity: §14.2 specifies conditional projective measurement
with Born probabilities.

## Additional admission and metamorphic coverage

`sol_r5_b01_library_replay_checks_boundaries_even_when_region_map_is_identity`
independently exercises verify_certificate: public Object fields reconstruct
the exact invalid subjects without parse admission. verify_claim proves the
final region identity, but verify_certificate rejects each internal X/CX
return. Certified doubled-X/doubled-CX and carried-even-loop positives prove
and preserve the entangled reference. Shared fixture construction hashes the
literal envelope/object without parsing the uncertified subject.

`borrow::tests::unresolved_earlier_frame_cannot_hide_a_definite_later_nonpermutation`
was separately RED: an unresolved earlier frame hid a later illegal borrowed
touch. decide_frames now scans all runs, retains an unresolved result, and lets
any definite failure take precedence before certificate admission.

Two separate deterministic families (no RNG/Python seed change):
`generated_r5_certificate_and_clean_context_lifetimes_match_independent_maps`
checks **128 cases, 64 accept / 64 reject**, with and without correctly bound
certificates: X/CX in either direction, trips 1..4, nesting, doubled gates,
known-zero vs known-one outer controls and alpha-renamed unrollings. Every
accepted subject preserves all basis/superposed/entangled-reference states;
every accepted certificate verifies. `generated_r5_alpha_renaming_consumed_workspace_preserves_widths`
adds **18 positive cases**, reused/renamed consumed widths 1..3 in loops 1..3,
with the same exact full-state checks in both directions.

Further audit control: `sol_r5_b01_certificate_return_checks_keep_quantum_surrounding_state`
was RED on the first audit implementation: basis-reading all surviving wires
incorrectly rejected an identity with t in superposition at return. Audits now
compare the exact surviving quantum state (including phase), requiring only
the borrowed entry to be restored, and do not narrow quantum action contexts.
`sol_r5_b01_library_replay_checks_frames_before_nonpermutation_boundaries`
isolates verify_certificate on X/CX maps that cancel across two H gates:
verify_claim sees the region identity, but frame auditing rejects the earlier
invalid frame before the non-permutation boundary. Valid frames surrounding
such gates still certify and preserve superposition/entangled references.

## Final verification of this received SOL pass

All four SOL blockers resolved; no declines or partials. Separate blocker
commits: **d0f7dbb** (B01), **e205f93** (B02), **860ebbd** (B03), **f34ff55**
(B04), with additional coverage in **6188112** and full-frame/quantum-context
audit controls in **738b233**. Response stays appendable for codex/muse.

After the final source changes, rebuilt offline and ran the exact standalone
command from conformance/PROGRESS.md: **445 pass, 0 fail, 46 unavailable / 491**,
exit **2** for unchanged explicit staging. Then ran the normal suite: **253
passed, 0 failed, 5 ignored** (251 integration + 2 unit; 4.56 s summed runner
time), and all ignored sweeps: **5 passed, 0 failed** (268.07 s summed runner
time). The 200,000-program sweep retains **64,717 accepted, all exactly clean /
135,283 rejected**. PROGRESS.md records each sweep's case count and timing.

Strict all-target Clippy, formatting, unchanged Python seed and diff checks
pass. An additional independent CLI replay verifies **72 B02 clean-context
runs**: bounds 1/2/3/2^40, flat/nested, both t basis entries and a complex
superposition, forward/adjoint plus equals_entry reversal. Every round-4
regression remains green. No source changes followed these gates.

All validation used absolute manifest/binary paths, the requested project-local
TMPDIR and all four display/browser variables unset. Conformance/include are
read-only; harness artifacts stay under the interpreter's ignored `.tmp`.
No accepted spec, seed, dependency or random-distribution changes; no pushes.
Staged/unverified whole-ISA deliverable disclosures remain unchanged.

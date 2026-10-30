# Editor response: council round 5

Inputs: SOL out-sol.json, codex out-codex.json and muse out-muse.json against
5d6d03e. The SOL first pass and codex/muse second pass are recorded below. ISA 0.8, the Python seed,
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

## Codex R5-B01 — resolved-by d0f7dbb (SOL-R5-B01), 738b233

Re-ran the exact three astra/codex witnesses against HEAD before changing
production code. New pin
`r1_certificates::codex_r5_b01_exact_three_certified_lifetime_witnesses`
independently checks all three published interface/object digests, including
CX t b, without parsing the invalid subject to compute its identity. All
three reject at Object::parse and CLI hash/check with §5.6, exit 1, empty
stdout, both with and without their correctly bound exhaustive certificates.
Certified straight-line and nested equivalents also reject. GREEN on HEAD:
1 test passed. No additional production fix required.

The SOL pins `sol_r5_b01_library_replay_checks_boundaries_even_when_region_map_is_identity`
and `sol_r5_b01_library_replay_checks_frames_before_nonpermutation_boundaries`
retain the independent public-API checks; certified carried/doubled-gate
positives retain full basis, superposition and external entangled-reference
checks. §§4.3/5.1 require each matched return after its frame; §5.6 requires
P_F = P′ ⊗ I_B. No decline, partial, new ambiguity or scope change.

## Codex R5-B03 — resolved-by e205f93 (SOL-R5-B02)

Re-ran both exact one-iteration qfree variants and the unrolled form against
HEAD before production edits. New pin
`r1_borrow::codex_r5_b03_one_iteration_release_matches_unrolled_clean_entry`
checks all three published interface hashes, parse/hash/check, run-action and
reverse at t=0/1. All accept and act as identity; reverse reports equals_entry.
Full basis entries, superposition and an external entangled borrowed reference
are preserved in each direction. Adding X c before release remains rejected.
GREEN on HEAD: 1 test passed. No additional production fix was necessary.

SOL's reachable-entry obligation substitution covers this free obligation as
well as returns; `sol_r5_b02_clean_surrounding_loop_controls_are_reachable_invariants`
also pins later iterations and 2^40 without a trip cap. §4.3's “n clean wires”,
§14.2's “qalloc tensors |0⟩” and §4.6's “body n times” decide the reading.
No certificate is needed. No decline, partial or new ambiguity.

## Codex R5-B02 — resolved (existing parse fix plus public-API hardening)

First re-ran the exact published CZ signature, interface hash and object hash:
parse/hash/check already reject with §5.6 after d0f7dbb/738b233, with the
exhaustive borrow_frames claim, with borrow_frames=[], and without metadata.
Existing exact-state replay also rejects the isolated public-API CZ witness
and CZ;CZ on arbitrary t. New pin:
`r1_certificates::codex_r5_b02_exact_cz_certificate_cannot_admit_a_nonpermutation_frame`.
Its phase-sensitive |++⟩ execution checks the three +1/2 coefficients and the
−1/2 coefficient at 11, independently of integer basis tuple comparisons.

The additional public-API control CZ b c; CZ b c with known-clean c was RED:
verify_certificate returned Proof because no reachable replay state sees a
phase. This still violates the gate-boundary structural premise. Added
`borrow.rs::check_borrow_frame_structure` and the shared recursive
`check_permutation_touches`, applied during parse validation and before
`claims.rs::check_borrow_independence` replay. They compile/in-line the actual
subject and inspect borrowed touches, including nested loops and calls;
non-permutation touches are errors independently of tuple/phase cancellation.
Zero-trip bodies remain unexecuted. This guard never calls certificate
admission recursively and does not require an entire action to be a permutation.

§12.1's structural checker obligation is “check the rule's side conditions
syntactically against the object and the subclaims' results”. Its exhaustive
obligation is “re-execute every domain element … compare full tuples”, not
permission to waive another claim's premise. §5.6 requires “maximal contiguous
permutation-only subcircuits”; §14.3 T3 begins “If the realised unitary V … is a
permutation matrix”. CZ = diag(1,1,1,−1) (§4.4) fails that premise even if
particular reachable inputs hide its phase. Tuple Proof alone is not a borrow
proof; verification.borrow_frames now explicitly checks the premise in both
admission surfaces.

GREEN: targeted borrow/certificate tests 31 passed, 0 failed, 2 ignored; full
normal suite 256 passed, 0 failed, 5 ignored. Positive actions with H only on
unrelated t and a valid permutation frame certify and preserve all basis,
superposition and entangled-reference states. Legitimate unresolved black-box
claim routes remain covered. README documents per-frame/per-return admission.
No decline, partial or genuine new ambiguity; reading 20 is unchanged.

## Nonblocking codex/muse suggestions — dispositions

- Muse draw-order contract: applied. README and SPEC-QUESTIONS 29 record
  lexicographic printed bits (wire 0 first), strict cumulative comparison and
  little-endian numeric values. `muse_r5_measurement_draws_order_bits_lexicographically`
  pins |++> draws 0.1/0.3/0.6/0.9 → values 0/2/1/3. No execution change.
- Muse loop-nested maximality: applied. README and SPEC-QUESTIONS 30 record
  maximality within the containing static sequence; the pre-loop identity
  pair plus full in-loop translation frame at ordinals 4–7 returns Proof in
  `muse_r5_translation_frame_maximality_is_within_its_static_sequence`.
  No rule was weakened and every actual frame/return remains checked.
- Muse exact-helper robustness: applied with RED before source changes.
  `zero_amplitudes_normalize_without_refining` hit precision-doubling overflow;
  `zero_probability_weight_does_not_refine` did not terminate (3 s timeout).
  Exact-zero amplitudes now emit (0,0) without setting the shared exponent;
  all-zero input emits zeros. A zero probability weight returns 0 directly.
  Mixed zero/nonzero normalization is pinned too. The negative measurement
  threshold is computed once outside the outcome loop. Both unit pins GREEN.
- Codex sparse-diagnostic contract: behavior already fixed by f34ff55
  (SOL-R5-B04). Expanded the public exact_support method comment and added
  `codex_r5_bell_projection_diagnostics_are_conditional_amplitudes`: initial
  Bell coefficients 1/sqrt(2), branch probabilities 1/2, projected sparse and
  dense coefficient 1. Raw shadows remain internal, so callers cannot mistake
  their old scale for the conditional diagnostic's amplitude.
- Codex deterministic interaction generator: applied in addition to SOL's
  128-case certificate-present family (unchanged). New
  `generated_r5_one_iteration_cleanup_and_nonpermutation_certificates` adds
  28 cases: 8 accepts / 20 rejects, with/without correctly bound certificates,
  wrapped/unwrapped clean/dirty qfree and inner/outer borrowing, plus forbidden
  CZ/S/H borrowed touches. Accepts preserve basis/superposition/entangled
  reference states in both directions. Existing RNG and Python seeds unchanged.
- Both seats' staging suggestions: retained explicitly, not claimed delivered
  by this correctness pass. Deliverables 8–11, the proposed F-link-conflict /
  F-frame-live-ranges promotions and the interval checker remain in the same
  recorded order. Unavailable/Unverified results remain explicit; final
  conformance must have 0 fail, not relabel unavailable cases as passes.
- Muse's proposed ISA 0.9 adoption is for the spec editor. No accepted spec
  edit; the proposed wording remains in SPEC-QUESTIONS for that work.

Targeted suggestion gates: 50 integration tests passed / 0 failed / 2 ignored,
plus the 2 new helper unit tests passed. Complete final gates follow below.

## Final codex/muse second-pass verification (2026-10-30)

All three codex blockers resolved, no declines or partials. Separate commits:
**f108397** (B01 exact pins, resolved-by d0f7dbb/738b233), **6635e4f** (B03
exact pins, resolved-by e205f93), **88a62ff** (B02 public certificate structural
premise); **5be6fcf** contains the nonblocking correctness/clarity suggestions.
The exact witnesses were run against HEAD before any additional production
fix. Muse accepted with zero blockers; its pure correctness/clarity suggestions
are applied and every other suggestion has an explicit disposition above.

After the last source changes, rebuilt offline and ran the standalone command
from conformance/PROGRESS.md, with absolute conformance manifest and reference
binary paths. CARGO_TARGET_DIR kept build artifacts under this crate's ignored
`.tmp/conformance-target`; conformance sources remained read-only. Result:
**445 pass / 0 fail / 46 unavailable of 491**, exit **2** for unchanged staged
deliverables 8–11. No unavailable case is counted as successful conformance.

Then ran all requested Rust gates with absolute `--manifest-path
/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml`:

| Gate | Result |
|---|---|
| `cargo test --offline` | **262 passed / 0 failed / 5 ignored**, 258 integration + 4 unit, 0 doctests; 3.71 s summed runner time |
| `cargo test --offline -- --ignored --nocapture` | **5 passed / 0 failed**, 353.16 s summed runner time |
| `cargo clippy --offline --all-targets -- -D warnings` | clean |
| `cargo fmt -- --check` | clean |
| unchanged Python `isa_check.py` | passes; seed diff against 71c1d6e empty |
| `git diff --check` | clean |

Ignored sweeps: u8 division 262,144 executions (1.75 s); 17-wire borrow 2^17
entries (10.24 s); checked 17-wire certificate 2^18 elements, both programs
(60.54 s); 200,000-program §6 sweep **64,717 accepted, all exactly clean /
135,283 rejected** (271.54 s); recursive prefix reversal 256 inputs × every
prefix (9.09 s). Original RNG distribution and Python seed are unchanged.

All validation was headless with DISPLAY, WAYLAND_DISPLAY,
DBUS_SESSION_BUS_ADDRESS and BROWSER unset, and
TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/.tmp.
Logs: `.tmp/r5-codex-{conformance,tests,ignored,clippy,python}.log`.
Accepted spec SHA-256 remains
`90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e`.
No Rust changes followed these final gates. No dependencies, accepted spec,
docs, conformance, include, worklog, HANDOFF or CLAUDE edits, and no push.
Bead b-isa-spec-zq2 remains open for the explicitly staged whole-ISA work.

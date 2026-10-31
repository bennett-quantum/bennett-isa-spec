# Bennett ISA reference interpreter progress

Resume from this file. Scope is the entire accepted ISA 0.8 (BRIEF.md); the
current CLI brief is `docs/briefs/2026-10-29-isa-ref-cli-xam.md`. Write only
under `reference-interpreter/`; commit own paths
(`git add -A -- reference-interpreter && git commit -- reference-interpreter`),
never push. Run headless with `TMPDIR=$PWD/reference-interpreter/.tmp`.

## Round-12 fixes (2026-10-31)

Sole implementation sub-agent; owned interpreter and external round-12 response.
Baseline: **353 passed / 0 failed / 5 ignored**, summed over all binaries.
All cargo/Python commands headless with repository .tmp; only crate tests.

| Blocker | RED tests | Status | Commit |
|---|---|---|---|
| SOL-R12-B01 ≡ R12-B01 | identity::r12_cc_r_colliding_dotted_leaves / r12_cc_r_layout_paths_preserve_colliding_leaves / r12_dotted_leaf_collision_family; cli_xam::r12_cli_checks_lossless_layout_envelopes | GREEN; full suite 358/0/5 | 76714aa |
| R12-B02 | identity::r12_interface_address_is_not_a_function_dependency; r1_linking::r12_interface_region_may_share_function_name; cli_xam::r12_cli_interface_region_may_share_function_name | GREEN; full suite 365/0/5 | 054bb7e |

B01 RED: 0 passed / 4 failed across identity and CLI; omitted duplicate
layout lines and wrong main pin. Independent support/r12_pins.py reproduces
all supplied oracle/main/object pins with literal LF envelopes, no rehash.
GREEN: six exact CC/R objects and 30 depth-2–3 collision-family objects;
equal, ascending and descending values, both-leaf sensitivity and foreign-owner
invariance. Noncolliding dotted global/lproc image selection and action/data
closure exclusions retained. Added lproc control initially moved a region
into the heap; corrected its test-only address to a disjoint range before
GREEN. First full run stopped on that setup error; final full B01 is 358/0/5.
All round 4–11 tests retained. Logs: .tmp/r12-b01-{red,green,full}.log.
Design: each leaf carries original JSON key components, rendered path, value;
select by structural keys, sort by rendered path then original components,
emit every leaf once. SPEC-QUESTIONS 31 records sol's exact tie wording for
0.9. Exact oracle-owner filtering and lproc closure selection are unchanged.
No spec/conformance edits or execution, no push, no new scope restriction.

B02 RED: 0 passed / 3 failed, §9.2 namespace ban (2 library, 1 CLI).
Removing the ban alone gives a second RED (0/1): false @$0 in the envelope.
Both exact self-named and db controls reproduce the supplied literal envelope,
interface/closure and object pins independently in r12_address_pins.py.
GREEN: 12 targeted R12 tests; final normal **365 passed / 0 failed / 5 ignored**.
Runtime (payload,code,steps) at B=2 is (16,0,2), at B=1 (0,1,1); CLI reversal
restores entry and cleans frames for both budgets and both local names.
Fourteen family cases admit local overlap with each of the seven symbol kinds,
defined and imported (unresolved imports do not contaminate @o's closure).
Call variants retain their actual dependency and change root identity when
callee changes; global and qregion image operands remain data references.
A non-declaring oracle rejects §4.2; actual-global resolution returns base 24.
The family generator initially omitted CC/R's foreign-owner leaf in the extra
symbol's envelope; corrected the independent expectation, not implementation.
No supplied member pin needed correction. Every previous test is retained.
Design: graph and replacer use the same contextual token classification;
only a lift.addr operand naming the current oracle's interface stays literal.
Object admission drops symbol/import bans, retaining only the existing actual
layout.globals ambiguity (recorded narrowly in SPEC-QUESTIONS 32, as codex
requests). No new rejection or rule-10 blocker; no Beads issues created/closed.
Logs: .tmp/r12-b02-{red,cli-red,graph-red,green,family,full}.log.

### Final round-12 verification

Implementation commits: **76714aa**, **054bb7e**. Normal suite totals before
and after, summed over every binary: **353/0/5 → 358/0/5 → 365/0/5**.
Final suite: 8 unit + 357 integration, 0 doctests; 36.27 s runner sum.
B01 full: 37.76 s. Final R12 target: 12/0. Fmt --check passes; strict
all-target Clippy passes, 0 warnings (2.52 s), 0 tests executed by either.
Independent corpus regeneration is byte-identical: 36 layout + 20 address
records, every supplied pin reproduced without correction, no interpreter
or support::rehash used to define expected answers.

All **5 ignored sweeps passed / 0 failed**, 168.57 s summed runner time.

| Sweep | Full domain | Observed result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 operand pairs × 4 = 262,144 executions | 1/0; 0.89 s |
| 17-wire borrow | 131,072 entries | 1/0; 4.73 s |
| checked borrow certificate | 262,144 elements, subject and expected program | 1/0; 26.83 s |
| §6 differential | 200,000 programs: 70,931 accepted (all clean), 129,069 rejected | 1/0; 131.76 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 1/0; 4.36 s |

No Rust source or test changes after the final normal suite or sweep start.
All round 4–12 tests retained, 12 tests added, original RNG unchanged.
Owned diff --check is clean. No new rule-10 blocker; the two requested 0.9
clarifications are SPEC-QUESTIONS 31 (sol's exact equal-path wording) and
32 (codex's narrow existing global/interface ambiguity wording).
Historical arithmetic labels in the round-1/2/3 progress tables and local
round-2 response now correctly say 65,536 operand pairs × 4 operations =
262,144 executions; no old domain/result changed.
No Beads issues created or closed; no export required. No spec/conformance
edits or execution, dependencies, push or Dolt sync. Existing whole-ISA staged
deliverables are unchanged. External response is
../bennett/docs/council/isa-ref-2026-10-29/round-12/response.md.
Logs: .tmp/r12-{b01-red,b01-green,b01-full,b02-red,b02-cli-red,
b02-graph-red,b02-green,b02-family,b02-full,clippy,sweeps,pins,address-pins}.log.

## Round-11 fixes (2026-10-31)

Tracking b-isa-spec-x02; sole implementer; owned interpreter and external
round-11 response only. Baseline 338 passed / 0 failed / 5 ignored.
Independent support/r11_pins.py reproduces both members' exact H/object pins,
including malformed objects, without interpreter calls or pin corrections.

| Blocker | RED tests | Status | Commit |
|---|---|---|---|
| SOL-R11-B01 ≡ R11-B01 | parser::r11_action_regs_do_not_produce_wires / r11_action_regs_cannot_invent_angles; CLI wire/angle counterparts | GREEN, 6 targeted pass; full suite 344/0/5 | d3e3c21 |
| SOL-R11-B02 ≡ R11-B02 | identity::r11_oracle_layout_ownership_uses_exact_names / r11_dotted_oracle_layout_ownership; CLI counterpart | GREEN; full suite 348/0/5 | fc6ab68 |
| SOL-R11-B03 ≡ R11-B03 | r1_fixtures::r11_qinplace_comment_matches_budget_contract; quantum_control::qif_and_nested_control_preserve_interference | GREEN; full suite 353/0/5 | 352e8f5 |

B01 RED: parser 0 passed / 3 failed, CLI 1 passed / 1 failed;
separate exact angle CLI 0 passed / 1 failed. Initial test-source quoting
error was corrected before semantic RED and is not counted as a witness.
GREEN: 6/0; regs family 8 accepted / 52 rejected across six kinds and nine
types, plus empty regs and loop-bounds controls. CLI negatives exercise 32
commands (hash/check/run-action, both action kinds, scalar/indexed, t=0/1).
Both supplied positive controls check independently pinned H/object bytes
and run forward/adjoint on both t basis entries.

Design: register-file legality is decided by function kind at load: oracle
and lproc locations are reg[N], CC/R use SSA, actions use Q producers.
Empty regs may carry bounds. Named rotation angles resolve specifically to
f64 classical arguments fixed at entry, never to a general type-map entry.
No runtime default, count cap or scope reduction.

Logs: .tmp/r11-{baseline,b01-red,b01-angle-cli-red,b01-green,b01-full,pins}.log.
Only crate tests with absolute offline manifest; no conformance execution.
B02 RED: 3 identity failures (both literal envelopes plus family), CLI 3 passed /
1 failed (§9.2 mismatch). GREEN: 7/0 including 3 retained B01 CLI tests.
Exact structured key selection precedes flattening; all non-interface leaves
remain. Six dotted-name positives plus three own-base sensitivity controls
include colliding rendered paths. Both member pin sets match without correction.
Initial test build used the wrong OracleInput field; fixed before semantic RED.
Logs: .tmp/r11-b02-{red,green,full}.log.
B03 RED: 8 passed / 2 failed (3/1 quantum_control, 5/1 fixture target).
The strict basis assertion rejected the old rounded-angle circuit and the
qinplace sidecar equality failed; retained rounded probe already passed.
First GREEN: 74/0 (63 CLI, 4 quantum_control, 7 fixtures); added exact minus-I
both-basis/direction coverage and corrected the same misleading inline TAU
comment in quantum.rs afterward. Final full suite: 353 passed / 0 failed / 5 ignored (15.36 s runner sum).

All 16 fixture sidecars and generated comments were audited against literal
rows and ISA rules. Details: tests/support/r11-comment-audit.md. Only the two
reported sidecars needed semantic corrections; control_flow's data-symbol
wording was clarified. The r9 generator branches on sol-qinplace, preserving
valid direct-oracle B=0 cancellation. All four r9 .bisa byte hashes remain.
The original @phase retains its literal bytes/hash. @minus_i uses S S X S S X;
controlled_phase now calls it. Independent literal envelope derivation repins
only this dependency plus the new symbol. Both codex standalone rounded/exact
probe H/object triples reproduce exactly; no member pin correction.
Design: exact structured owner selection precedes dotted rendering, and
fixture evidence distinguishes exact support from diagnostic rounding and
checks token preconditions. README now records all three round-11 rules.
No runtime quantum or qinplace semantics changed for B03.
Normal/fmt/strict all-target Clippy gates pass; 0 warnings (Clippy 1.12 s).
Owned diff --check is clean. New control_flow object hash:
e7e0acadaebf638a1adb34adb61efc2c79b43f0ec90b786582d2fdb6db7634d8.
Final sweeps: 5 passed / 0 failed, 94.46 s summed runner time.
| Sweep | Domain | Observed result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 pairs × 4 = 262,144 executions | 1/0; 0.51 s |
| 17-wire borrow | 131,072 entries | 1/0; 2.74 s |
| checked certificate | 262,144 domain elements, subject and expected | 1/0; 14.76 s |
| §6 differential | 200,000 programs; 70,931 accepted, all clean / 129,069 rejected | 1/0; 73.97 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 1/0; 2.48 s |

Implementation commits d3e3c21 / fc6ab68 / 352e8f5. All round-11 blockers
resolved; 15 new tests, normal 338→344→348→353, 0 failed, 5 ignored.
Final suite 8 unit + 345 integration; 0 doctests. Baseline/full runner sums:
15.02 / 15.28 / 15.13 / 15.36 s. Fmt clean; strict all-target Clippy clean,
0 warnings (1.12 s). No semantic source changed after the normal gate or
sweep start. All round 4–10 positives/negatives retained.
Independent regeneration is idempotent: 16 R11 corpus records, every member
pin correct, four R9 .bisa bytes unchanged, control_flow envelope pins stable.
Owned diff --check clean. Tracking b-isa-spec-x02 closed at final handoff.
External response: ../bennett/docs/council/isa-ref-2026-10-29/round-11/response.md.
All work is complete for this round; staged whole-ISA deliverables remain.
Logs: .tmp/r11-final-{fmt,clippy,sweeps}.log and all per-step logs above.
No conformance test or foreign source was edited/executed; no push.
Beads auto-export and the pre-existing untracked repository .tmp remain
outside owned commits. Accepted spec hash remains
90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e.
ISA 0.8 gap/amendment: none. Existing staging and SPEC-QUESTIONS unchanged.

## Round-10 fixes (2026-10-30)

Tracking b-isa-spec-eka. Sole implementer; owned interpreter and external
round-10 response only. Baseline normal suite: 327/0/5 (13.40 s; log .tmp/r10-baseline.log).
Independent tests/support/r10_pins.py reproduces every supplied malformed,
positive-control and effective-control H/object pin exactly, with no correction.
Exact LF bytes are retained in tests/support/r10_cases.json, including invalid
objects (kept outside the valid .bisa fixture audit).

| Blocker | RED tests | Status | Commit |
|---|---|---|---|
| SOL-R10-B01/B02 ≡ R10-B01 | r10_function_header_prefix_matches_kind; r10_present_bounds_requires_an_integer; r10_exact_fhead_and_nonempty_optional_productions; CLI counterparts | resolved; full gate GREEN | 4ba57f9 |
| R10-B02 | r10_equivalent_control_sets_preserve_opaque_inverse; CLI counterpart | resolved; full gate GREEN | 4ba57f9 |

Exact RED: parser 0/3, CLI 1/2 (positive control passed), library 0/1.
Both controls and duplicate witnesses rejected §6.2; all 12 header positions,
the three inc8 mutations and 152 CLI invocations were exercised, including
later-position forms already rejecting §16.2. No existing correct rejection
is claimed as a new RED. Exact GREEN: 7 passed / 0 failed (3 CLI, 3 parser,
1 library). Baseline: 327 passed / 0 failed / 5 ignored.

Design: strict fhead alternatives share recognition with function boundaries;
present bounds and LHS are nonempty; legal empty operands/omitted bounds stay.
Lexical predicate stacks preserve push/pop ordering. Operation keys instead
use sorted, deduplicated physical scalar literals retaining polarity and full
current version snapshots. Both static passes share EffectiveControls.
Borrow nodes canonicalize after physical resolution and after instantiation;
all-operand histories, ABI operand order, token/callee identities, frame
boundaries and every-return obligations remain mandatory. No count cap.

No spec/conformance/Python-seed changes, pushes, semantic caps or scope reductions.
Implementation and deterministic families are GREEN. Full suites: 334/0/5
after exact witnesses, then 338/0/5 after families. Targeted extended families
and pins: 4/0; extended CLI positive controls: 3/0. Families: 102/6 effective
controls, 9/1 cleanup contexts, 7/16 grammar (accepted/rejected). All small
control positives execute independent identities on 32 full t,p,q,b,R basis
entries, a control superposition and a Bell-paired bR entry, both directions;
oracle B=0/1, qinplace B=1. No semantic source changes followed the full gate.
Nonsemantic test additions subsequently pin one-trip and tuple-LHS CLI controls;
targeted 3/0. Final verification rerun and ignored sweeps follow.
Fmt --check and strict all-target Clippy passed (0 warnings).
One initial implementation build caught an accidental lexical-stack edit;
it was corrected before GREEN. Extended pin-target compilation initially
lacked mod support; corrected before the 4/0 run. Neither is semantic RED.
The initial 2-family run was 2/0, 109 accepted / 22 rejected; final 3-family
run was 3/0, 118 accepted / 23 rejected. RNG/seeds unchanged.

### Final round-10 verification

Implementation **4ba57f9**. Tracking b-isa-spec-eka closed.
All commands headless/offline, absolute manifest, repository .tmp; tests use
CARGO_PROFILE_TEST_OPT_LEVEL=1 with unchanged assertions/domains/overflow checks.
No semantic source changed after the first 338/0/5 gate or during sweeps.
Only the one-trip/tuple positive-control pin additions followed that first gate;
the final rerun below includes them. SPEC-QUESTIONS unchanged; no new ISA gap,
amendment, count cap, frame waiver, certificate exception or scope reduction.
Existing deliverables 8–11 and performance follow-up b-isa-spec-436 remain staged.

| Gate | Observed result |
|---|---|
| normal baseline | 327 passed / 0 failed / 5 ignored; 8 unit + 319 integration; 13.40 s |
| exact RED (initial parser / library) | 0/3 / 0/1 |
| initial grammar CLI RED / control CLI RED | 1/1 / 0/1 (passed grammar control retained) |
| aggregate exact RED | 1 passed / 6 failed; 3 parser + 2 CLI + 1 library failures |
| exact GREEN | 7 passed / 0 failed |
| first full suite | 334 passed / 0 failed / 5 ignored; 16.85 s |
| initial deterministic families | 2/0; 109 accepted / 22 rejected |
| extended families + pin target | 4/0; final families 118 accepted / 23 rejected |
| full suite with final families | 338 passed / 0 failed / 5 ignored; 15.67 s |
| extended one-trip/tuple CLI controls | 3 passed / 0 failed |
| final normal rerun | 338 passed / 0 failed / 5 ignored; 8 unit + 330 integration, 0 doctests; 17.70 s |
| fmt --check | pass, 0 tests executed |
| strict all-target Clippy | pass, 0 warnings, 0 tests; final rerun 1.25 s |
| five ignored sweeps | 5 passed / 0 failed; 100.42 s runner sum |
| independent Python pins | all supplied H/object pins reproduce, no corrections; 13 case records including new controls |
| owned diff --check | clean |

| Sweep | Domain | Observed result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 pairs × 4 operations = 262,144 executions | 1/0; 0.52 s |
| 17-wire borrow | 131,072 entries | 1/0; 2.91 s |
| checked borrow certificate | 262,144 domain elements, subject and expected program | 1/0; 15.23 s |
| §6 differential | 200,000 programs; 70,931 accepted (all exactly clean) / 129,069 rejected | 1/0; 78.84 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 1/0; 2.92 s |

The same RNG/seeds/distribution previously produced 70,930/129,070; the semantic
set fix admits one more independently checked clean sample. All prior named
round 4–10 positives and negatives pass. Conformance was not run.
Response: ../bennett/docs/council/isa-ref-2026-10-29/round-10/response.md.
Logs: repository .tmp/r10-{baseline,red-all,green-exact,first-full,families,
families-extended,positive-extended,final-tests,final-rerun-tests,
final-sweeps,final-rerun-clippy,final-fmt,independent-pins}.log.
Accepted spec SHA-256 unchanged:
90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e.
Only explicit owned paths committed; no push.

## Round-9 fixes (2026-10-30)

Tracking b-isa-spec-82k. Sole implementer; owned crate and external round-9 response only.
Independent Python hashlib derivation reproduced all four member H/object pins
without correction; exact LF bytes live in tests/fixtures/r9-*.bisa and derivation
in tests/support/r9_pins.py. Exact library/CLI tests added before source changes.

| Blocker | RED tests | Status | Commit |
|---|---|---|---|
| SOL-R9-B01 ≡ R9-B01 | r9_opaque_adjoint_restores_borrow_control, r9_matched_opaque_oracle_restores_outer_fact, r9_qinplace_adjoint_restores_borrow_control; CLI counterparts | exact GREEN | 334c27d |
| R9-B02 | r9_mixed_loop_threads_reallocated_bindings; CLI counterpart | exact GREEN | 334c27d |

Exact GREEN: 8/0; first completed full suite 323 passed / 0 failed / 5 ignored.
The initial baseline run raced the newly added fixture files: its fixture audit
failed 2 tests, so it is not a baseline pass. A clean git-archive baseline was
rerun under the owned ignored .tmp, with the accepted spec read via symlink:
315 passed / 0 failed / 5 ignored. Its first build lacked the spec include;
adding the read-only symlink resolved that setup error.
Initial RED setup also had an Argument-vs-controls API compile error, corrected
before the semantic REDs. No compile error is counted as a reproduced blocker.

Design: opaque calls carry an explicit adjoint key (oracle self-adjoint;
qinplace swaps g/ginv) through lowering, instantiation and adjoints. They join
all-operand LIFO stacks above quantum history; assigning qinplace starts history
itself because its opaque symbolic assignment loses information. Unmatched
readers prevent outer pops; lifetime/import barriers invalidate connected stacks.
Loop lowering consumes released names and resolves each next body against its
current live entry map. Equal-width, equal-role, nonoverlapping successor
lifetimes are alpha-renamed to reusable entry slots. Changed environments get
separate transfers; recurring environments share the exact structured cycle.
Zero visits publish no binding or fact effects; lexical qregions stay hoisted.
All physical frame boundaries, complete borrowed-touch guards and per-return
obligations remain mandatory. No trip cap, expansion sample or scope narrowing.

Extended R9 families: 2 passed, **33 accepted / 8 rejected** (opaque 13/5,
loop bindings 20/3; one huge loop is static-only). Intermediate family 2/0:
28/7 (opaque 11/4, loops 17/3). The initial family 0/2 was fixture blank lines;
a changed-control fixture then hit §3.3 for an aliased target and was corrected
to a distinct read-only operand. No implementation change was needed for these.
The exploratory 2^40 fresh-allocation execution was stopped without a result:
execution cycle keys still distinguish fresh wire ids (existing performance
follow-up b-isa-spec-436 updated). Small family cases execute both directions;
large analysis has no trip cap or sampled proof.

An additional causal audit reproduced **0/1 RED** for three qinplace inverse
pairs on clean workspace with no outer H. Plain permutation-loop ANF squaring
cannot simplify assigning opaque g^-1(g(x)); `Node::needs_history` now routes
these loops through the exact stack transfer/cycle. This extends the same rule
rather than adding a function-specific oracle identity. Targeted GREEN: 8/0 (6 library regressions + 2 deterministic families).
Zero-budget qinplace rejects §4.5 before its own mutation, including independent
adjoints; the literal member action rejects at execution as required. Earlier
exact GREEN was 8/0, extended budget GREEN 9/0. RNG/seeds remain unchanged.

### Final round-9 verification

Core commit **334c27d**, extended history/families commit **b07ab5b**.
All gates use absolute crate manifest, offline, headless and repository .tmp;
test optimization level 1 preserves domains/assertions/overflow checks.

| Gate | Observed result |
|---|---|
| clean archived baseline | **315 passed / 0 failed / 5 ignored**, 8 unit + 307 integration; 15.96 s |
| first full suite | **323 passed / 0 failed / 5 ignored**; 37.65 s |
| final full suite | **327 passed / 0 failed / 5 ignored**, 8 unit + 319 integration, 0 doctests; 14.65 s |
| fmt --check | pass, 0 tests executed |
| strict all-target Clippy | pass, 0 warnings, 0 tests executed; 1.16 s |
| all five ignored sweeps | **5 passed / 0 failed**, 93.83 s runner sum |
| independent pins | all four root H/object pairs and both oracle H values match, no correction |
| archived pre-fix CLI replay | **44 commands**, each exit 1, empty stdout, exact §5.6 diagnostic |

| Sweep | Domain | Result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 pairs × 4 = 262,144 executions | 1/0; 0.51 s |
| 17-wire borrow | 131,072 entries | 1/0; 2.72 s |
| checked certificate | 262,144 domain elements, subject and expected program | 1/0; 14.85 s |
| differential | 200,000 programs: **70,930 accepted, all clean / 129,070 rejected** | 1/0; 73.38 s |
| recursive prefixes | 256 inputs × every prefix through completion/first fault | 1/0; 2.37 s |

Final pin-only strengthening explicitly compares interface AND closure values
with stored member pins in library/CLI tests. Semantic sources are unchanged
since the passing sweeps. Pin-targeted rerun: **10/0**; final full-suite rerun: **327/0/5**, 14.61 s runner sum.
Fmt passes; strict all-target Clippy passes with 0 warnings (0.29 s).
Tracking b-isa-spec-82k closed; evidence commit follows.
No conformance test was run. All existing round 4–8 negatives remain.
No new ISA gap or amendment; SPEC-QUESTIONS unchanged. Accepted spec SHA-256:
90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e.
Beads metadata auto-export is outside owned commits; no push. External response:
../bennett/docs/council/isa-ref-2026-10-29/round-9/response.md.
Logs: repository .tmp/r9-final-*.log, r9-clean-baseline.log,
r9-red-{library,cli,assign-repeat}.log, r9-independent-pins.log.

## Round-8 fixes (2026-10-30)

Tracking b-isa-spec-41i; sole interpreter implementer, no push.

| Blocker | RED test | Status | Commit |
|---|---|---|---|
| SOL-R8-B01 ≡ R8-B02 | r8_compound_adjoint_restores_borrow_control, r8_diagonal_nested_adjoint_restores_borrow_control; r8_cli_compound_x/s | exact nested X/S witnesses green | 366a711 |
| SOL-R8-B02 ≡ R8-B01 | r8_zero_trip_lifetime_environment_does_not_escape, r8_zero_trip_surviving_fact, r8_zero_trip_cannot_hide_borrowed_quantum_touches; CLI counterparts | exact witnesses green | b11b43f |
| R8-B03 | r8_action_call_preserves_readonly_control; r8_cli_call | exact disabled controlled-H witness green | 5ad0555 |
| SOL-R8-B03 | r8_unsigned_decimal_multiplication; r8_unsigned_multiplicity_composition | exact unsigned boundary and composition green | 366a711 |

RED: nested library **0 passed / 2 failed**, CLI **0 passed / 2 failed**,
§5.6 released wire not clean. Numeric unit **0 passed / 1 failed** at
2^127 (negative); linking **0 passed / 1 failed** (zero instead of 2^128).
Targeted GREEN `cargo test --offline ... r8_ -- --nocapture`: **6 passed /
0 failed**, 1 unit + 5 integration. Logs `.tmp/r8-{red-*,green-stack-numeric}.log`.

Design: each basis-changing gate, and each reversible gate above a saved
quantum history, is recorded on every operand's operation stack.
An exact adjoint pops only when the same entry tops every operand stack;
nested/disjoint computation therefore restores in LIFO order, while an
outstanding entangling reader prevents an outer restoration. Saved ANFs retain
correlations. Opaque/lifetime barriers invalidate the connected history.
Quantum frame boundaries, full touched sets and every return equation remain.
`BigInt::from_u128` constructs all unsigned limbs, with no signed narrowing.

Zero-trip RED: **0 passed / 3 failed** in each library and CLI target.
The H subject was incorrectly admitted; wide c reported §3.3 scalar width;
same-width c reported §5.6 released wire not clean. Both H and S pins remain
literal tests (the first failure stops their combined RED test). GREEN:
**10 passed / 0 failed** across R8 borrow/CLI tests. Existing huge permutation
loop `sol_r5_b02` separately **1 passed / 0 failed**. Source lowering now omits
zero visits before ANY binding, borrow-set, node or trace effect, following the
executed-lifetime rule in LifetimeWidths; lexical qregions remain hoisted.
The environment and fact-node output share that single structured traversal.
An early full-suite run overlapped test authoring, observed the new zero-trip
REDs and was stopped during a needless history expansion; it is not final gate
evidence. Basis-only permutations now retain exact ANFs without recording
history unless touching an existing quantum history; the huge test passes.

Call RED: library and CLI each **0 passed / 1 failed**, §5.6 released
wire not clean. GREEN combined R8 borrow/CLI targets: **12 passed / 0 failed**.
Local non-permutation calls now have memoised canonical structured transfers,
instantiated on actual wires and inherited controls. Cache keys include exact
classical bindings; summaries are reused per callee specialization. The §5.1
SCC validator excludes action recursion before this pass. Unresolved imports
make only mut wires/mut cells unknown. Known-false call controls disable fact
transfer; read-only operands keep their facts. The independent call boundary
guard retains FULL actual touched sets, including disabled borrowed controls;
callee-internal borrowed touches are checked recursively. Every physical call
and quantum gate still delimits frames.

Deterministic R8 families: **3 passed / 0 failed**, **134 accepted / 25
rejected**: nested stacks **86/6**, zero trips **33/10**, callee transfers
**15/9**. Includes depth 3, X/CX/S/SWAP/Rx/Ry, interleaved wires, loop wrapping,
renamed/dead qcopy/qload/qborrow bindings, hoisted dead qregions, certified
H/S borrowed-touch negatives, live-name/scalar-width negatives, known-zero/one
controls, readonly input controls, unknown-control negatives and callee chains.
Initial family run **0 passed / 3 failed**: wrong qload address width and
missing explicit adjoint-call cleanup were fixture errors; the nested
SWAP/Rx identity exposed a real §6.2 over-rejection, tracked as b-isa-spec-2kd.
`Discipline::change` now distinguishes readers of the current operation's
captured version ID from older readers of an ancestor. Outstanding entangling
readers still block cancellation. SPEC-QUESTIONS 25's version-sensitive
reading remains unchanged; no new ISA ambiguity or scope restriction.

First completed normal gate after variants: **314 passed / 0 failed /
5 ignored** (8 unit + 306 integration, 0 doctests). Fmt passed; strict Clippy
found one map-entry style violation, corrected without a behavior change.
Final gates and all five ignored sweeps follow after that correction.

Final causal audit: nested two-output SWAP pairs reproduced §6.2 rejection
(**0 passed / 1 failed**) and now use the same reader rule as single-wire
changes. `reads_current` compares the complete captured version-stack prefix,
not only the top gate ID: repeated H gates can share an operation identity
while naming different nested events. Added repeated-H/SWAP positives.
An imported unknown-control reader exposed the distinction between preserving
a read-only wire's current fact and preserving its ancestor inverse history.
The new static import test first rejected at §6.2 (**0/1**); after correcting
version-prefix matching it exposed the unsound borrow admission (**0/1**).
Opaque imported calls now invalidate connected history across ALL touched
operands, while assigning fresh unknowns ONLY to modified outputs. Disabled
imports remain fact no-ops. Local calls retain their analysed transfers.
The test explicitly marks its unlinked positive as static-only evidence.
S-only pre-fix replay from isolated 366a711 beneath `.tmp`: **0/1**, library
accepted and CLI check exit 0/status ok with the exact published H/object pins.
No shared source checkout was rewound.

Final variant GREEN: **4 passed / 0 failed**, **139 accepted / 26 rejected**:
nested **90/6**, zero-trip **33/10**, local-call **15/9**, imported-call
static **1/1**. Intermediate joint-only family run was **3 passed / 0 failed**,
**136/25** (88/6 nested). Final source fmt and strict all-target Clippy pass,
0 warnings, 0 tests executed (Clippy 2.84 s). All semantic changes are frozen
for the final full suite and five sweeps.

### Final round-8 verification

All four S0 findings resolved. Implementation commits: **366a711** (nested
stacks and unsigned arithmetic), **b11b43f** (executed zero-trip bindings),
**5ad0555** (memoised calls), **b658ac9** (families and version-aware readers),
**0b70157** (complete version prefixes, joint readers and opaque histories).
Only owned crate paths committed; no push. Council response is the separately
owned `bennett/docs/council/isa-ref-2026-10-29/round-8/response.md`.

Final gates use absolute manifest, headless/offline, TMPDIR at the repository's
`.tmp`, CARGO_PROFILE_TEST_OPT_LEVEL=1 (same assertions/domains/overflow checks).

| Gate | Observed result |
|---|---|
| fmt --check | pass, 0 tests executed; comment-only rerun also passes |
| strict all-target Clippy | pass, 0 warnings, 0 tests executed; 2.84 s; comment-only rerun 1.29 s |
| normal suite | **315 passed / 0 failed / 5 ignored**, 8 unit + 307 integration, 0 doctests; 21.07 s summed runner time |
| all ignored sweeps | **5 passed / 0 failed**, 109.73 s summed runner time |
| R8 deterministic families | **4 passed / 0 failed**, **139 accepted / 26 rejected** (90/6 nested, 33/10 zero, 15/9 local call, 1/1 static import) |
| read-only standalone conformance | **445 pass / 0 fail / 65 unavailable / 510**, exit 2 for explicit staging |
| unchanged Python seed | 2 example objects and 2 printed envelopes verified; execution as stated |
| owned diff --check | clean |

| Sweep | Domain | Result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 pairs × 4 operations = 262,144 executions | 1 passed / 0 failed; 0.70 s |
| 17-wire borrow | 131,072 entries | 1 passed / 0 failed; 4.31 s |
| checked borrow certificate | 262,144 claim elements, subject and expected program | 1 passed / 0 failed; 22.04 s |
| §6 differential | 200,000 programs: **70,930 accepted (all exactly clean) / 129,070 rejected** | 1 passed / 0 failed; 80.30 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 1 passed / 0 failed; 2.38 s |

Acceptance improved from the historical 64,717/135,283 with unchanged RNG
code/distribution/seeds; every newly accepted sampled program is independently
executed for exact cleanup. Conformance's concurrently maintained checkout
adds explicitly unavailable contract cases; none is counted as passed.
Logs: repository `.tmp/r8-complete-{fmt,clippy,tests,sweeps,conformance,seed}.log`
and `.tmp/r8-comment-{fmt,clippy}.log`. Only a comment explaining the equivalent
controlled-image conjugation in cached proof templates changed after the final
semantic tests; fmt and strict Clippy were rerun. Execution's unconditional
image preparation is unchanged. No semantic source changes followed the gates.

No new ISA 0.8 gap or proposed spec change is needed. Existing SPEC-QUESTIONS
20/25/28/30 and staged deliverables 8–11 remain; no semantic cap or frame waiver.
Accepted 0.8 SHA-256 remains
`90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e`.
Beads' automatic export is outside owned commit paths and left for the maintainer.

## Round-7 fixes (2026-10-30)

Tracking bead b-isa-spec-lto. Sole interpreter implementer; no push.

| Blocker | RED test | Status | Commit |
|---|---|---|---|
| SOL-R7-B01 | r7_restored_zero_before_borrow / r7_cli_restored | exact witness green | 1caa062 |
| SOL-R7-B02 ≡ R7-B01 | r7_reject_loop_bad, r7_nonpermutation_loop_positive, r7_reject_rotate_4/8; CLI counterparts | exact witnesses green | 1caa062 |
| R7-B02 | r7_diagonal_basis, r7_readonly_control; CLI counterparts | exact witnesses green | 1caa062 |
| SOL-R7-B03 | r7_dyadic_scaled_conversion_boundary_rounding_and_subnormals; r7_cli_bounded_5000_h_t_reports_finite_amplitudes | exact witness and rounding/cancellation checks green | 51d45dc |

### Final round-7 verification

All four blockers are resolved; no declines or partials. Commits **1caa062**
(gate facts/structured loop proofs), **51d45dc** (scaled dyadics/algebraic
cancellation), **6152d25** (families, literal pins, epoch cycle equivalence),
**516052b** (zero-trip fact transfer). Only owned paths committed; no push.
Tracking b-isa-spec-lto is complete; performance follow-up b-isa-spec-436
remains open, without a semantic count limit or new ISA gap.

Final sequential gates after all semantic source changes, headless with
TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp,
absolute manifest, offline, CARGO_PROFILE_TEST_OPT_LEVEL=1 (same domains,
assertions and overflow checks):

| Gate | Observed result |
|---|---|
| fmt --check | pass, 0 tests executed |
| strict all-target Clippy | pass, 0 warnings, 0 tests executed |
| normal suite | **297 passed / 0 failed / 5 ignored**, 7 unit + 290 integration, 0 doctests; 40.69 s summed runner time |
| all ignored sweeps | **5 passed / 0 failed**, 143.52 s summed runner time |
| read-only standalone conformance | **445 pass / 0 fail / 62 unavailable / 507**, exit 2 for explicit staging |
| owned git diff --check | clean |

| Sweep | Domain | Observed result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 pairs × 4 ops = 262,144 executions | 1 passed / 0 failed; 1.10 s |
| 17-wire borrow | 131,072 entries | 1 passed / 0 failed; 5.93 s |
| checked borrow certificate | 262,144 elements, subject and expected program | 1 passed / 0 failed; 30.31 s |
| §6 differential | 200,000 programs: **64,717 accepted (all exactly clean) / 135,283 rejected** | 1 passed / 0 failed; 102.22 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 1 passed / 0 failed; 3.96 s |

Final logs: repository `.tmp/r7-verified-{fmt,clippy,tests,sweeps,conformance}.log`.
The current read-only conformance harness added unavailable contract cases
concurrently: its earlier pass in this session was 445/0/60/505; neither run
counts unavailable cases as passes. Accepted 0.8 SHA-256 remains
90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e.
No semantic source changes followed these final gates. Full commands, RED
witnesses, retained negatives, pinned hashes and Design are in the council
round-7 response. Beads auto-export is outside the owned commit paths and is
left for the maintainer.

RED before source edits: library 0 passed / 7 failed; CLI 0 passed / 7
failed. Both bad-loop admissions and all four erroneous rejections reproduced.
GREEN targeted: 14 passed / 0 failed. Normal suite: 288 passed / 0 failed /
5 ignored. All existing round 4–7 negatives remain in the suite.

Numeric RED: unit 0 passed / 1 failed at k=1024 (NaN vs 1.0);
pinned CLI 0 passed / 1 failed, §16.1 non-finite binary64 has no JSON number;
Root2 cancellation 0 passed / 1 failed. GREEN: 2 unit tests and 1 CLI test
passed (the CLI covers forward, independent adjoint, norm and reversal).
Dyadic conversion rounds the combined numerator/exponent once, nearest ties to
even, including subnormal ties. Root2 severe cancellation uses refining dyadic
enclosures before conversion. No denominator/gate-count ceiling or NaN clamp.
An initial CLI RED attempt raced a subsequent rebuild and passed (103.67 s);
that attempt is not RED evidence. The pinned old-conversion replay reproduced
the member's diagnostic (35.89 s), followed by GREEN (70.78 s). An intermediate
conversion failed the new subnormal assertion; exact power-of-two construction
fixed it before commit. Diagnostic rejection of truly non-finite JSON remains.

Deterministic R7 families: **93 accepted / 63 rejected** in 3 passing tests:
60/24 diagonal/control/inverse cases, 32/38 evolving-loop cases, and 1/1
large noncycling fact-invariant cases. Small loops include renamed unrollings,
nested 2×2, trips 1/3/4/5/8; huge H/X evolving controls cover 2^40 in both
valid and invalid shapes. Every accepted executed case uses the independent
I_t tensor I_bR map; the invariant-only case is explicitly static evidence.
Original RNG distribution/seeds are unchanged. Primitive 5000 H/T quantum
map and exact reversal: 1 passed (3.54 s). Rejected literal byte hash pins:
7 passed / 0 failed after strengthening the support helper.

The family exposed an existing absolute-window-clock cycle-key problem.
`validation.rs::Discipline::cycle_key` compares rank-normalized current/saved
epochs, preserving all open/closed window comparisons and leaving actual
counters unchanged. The huge H/X family was interrupted before this correction;
afterwards it passes. A dedicated window-equivalence unit passes too.
Tracking b-isa-spec-gui. Broader existing noncancelling §6 operation histories
(huge TH, swap rotations and an outstanding-copy variant) remain enumerative:
exploratory runs were interrupted without a completed result, not counted as
passes/rejections; follow-up **b-isa-spec-436**. No ISA count was capped or
newly rejected. The final huge noncycling *borrow-fact* case uses H X X H,
whose independent matrix is I but whose intervening touches force the new
unknown-top invariant proof. Rotation family coverage is through eight visits;
large evolving positive and negative controls use H/X.

Normal suite after families: **295 passed / 0 failed / 5 ignored** (6 unit,
289 integration); new window-equivalence unit separately **1 passed**.
Strict all-target Clippy and fmt --check passed before that test-only addition.
Read-only standalone conformance: **445 pass / 0 fail / 60 unavailable / 505**,
exit 2; concurrent conformance additions are outside this implementation.
The completed final gate and sweeps are recorded above.

Final audit RED: `r7_zero_trip_is_not_an_intervening_inverse_operand_touch`
failed with §5.6 released wire not clean: a zero-trip body was invalidating an
H inverse despite having no dynamic touches. `Facts::sequence` now skips a
zero Repeat before any fact/frame transfer. GREEN: 8 R7 borrow tests passed.
This additional zero-trip guard and the corrected Repeat comment are committed
separately. The completed normal gate before this audit was **296 passed /
0 failed / 5 ignored**, 7 unit + 289 integration, 206.19 s summed runner time.
The initial debug sweeps completed division (16.64 s), 17-wire execution
(97.84 s), certificate (492.05 s), each 1 passed / 0 failed; the fourth
200,000-program run was stopped without a result so all five can be rerun
in sequence after the final source change. The completed final gate uses
`CARGO_PROFILE_TEST_OPT_LEVEL=1` (same tests, domains, assertions and overflow
checks) to avoid repeating these long unoptimized runs. Final counts are recorded above.

The borrow engine now walks structured nodes with per-wire ANF basis facts.
Unknown is a fresh independent symbol, never zero. Permutations transform facts;
diagonal gates and read-only controls retain them; non-diagonal targets become
unknown. An exact inverse with untouched operands restores the saved prior fact.
Any intervening operand touch invalidates that saved inverse, including a
multi-wire operation, so an entangled result cannot resurrect a clean fact.
Every quantum gate remains a frame boundary with its full borrowed-touch set.
Every return's restoration/independence equations remain mandatory. Pure
permutation loops compose both maps and obligations by repeated squaring. Mixed
loops retain the actual count, execute fact transfers per iteration, and skip
only identical complete analysis states. For large noncycling transfers, a
join replaces disagreeing facts by independent unknowns until the body proves
an inductive invariant, including frame and return obligations. No sampled
expansion or semantic trip cap remains. Execution cycle skipping is separate.
No reading in SPEC-QUESTIONS is contradicted; no new ISA gap is needed.

## Round-6 fixes (2026-10-30)

| Blocker ids | RED test | Status | Commit |
|---|---|---|---|
| SOL-R6-B01 / codex R6-B02 | r6_sol_boundary_exact; r6_codex_boundary_exact; CLI equivalents | resolved; variants/families green | dceaa22, 32ef2e6 |
| SOL-R6-B02 / codex R6-B01 | r6_zero_trip_exact; r6_changing_width_exact; CLI equivalents | resolved; variants/families green | dceaa22, 32ef2e6 |

RED: library 0/2 + 0/2 and CLI 0/4, all four reported diagnostics
reproduced (qreturn restoration, released wire not clean, missing consumed
lifetime width, wire index out of range). GREEN: exact targets 8 passed;
full normal suite 270 passed / 0 failed / 5 ignored, including retained R4/R5
certificate and structural negatives. Pinned interface/closure/object hashes
are unchanged. Independent forward/adjoint maps include basis, superposition,
borrowed-reference entanglement, exact reversal and no escaping workspace.

`borrow.rs::factorises` replays the established prefix as entry context,
forgetting only outputs touched by non-permutation gates; previous return
obligations remain separate. `action.rs::infer_widths` produces a structured
plan indexed by sequence position and loop entry width map, sharing recurring
width environments as cycles. Zero trips have no lifetime effects; inverse
iteration j uses forward iteration j's matched release. Reverse quantum period
skipping leaves transient prefix iterations to execute explicitly. No trip cap.
Additional pins: `r6_boundary_variants_retain_clean_controls_and_reject_dirty_frames`
(5 positive variants, 8 certified/uncertified negatives) and
`r6_structured_lifetime_variants_match_independent_identity` (14 positives,
including nested zero trips, dead copy/load/borrow changes, renamed unrolling,
and qalloc/qborrow width changes at 1/2/3/128 trips).
`r6_map` now checks all 16 t/three-wire-pool basis entries plus superposition
and a borrowed wire entangled with an external reference, each direction
independently followed by exact reversal and no remaining workspace.

C7: separate deterministic families
`generated_r6_disjoint_h_boundaries_match_identity_and_renamed_unrollings`
(**48 accepted cases**) and
`generated_r6_dynamic_width_lifetimes_match_identity_and_renamed_unrollings`
(**96 accepted cases**). They cover trips 0/1/2/3, nesting, disjoint H placement,
qalloc/qborrow, widths 1/2/3 and alpha-renamed unrolling. Identity is derived
from c=0, H²=I and within-lifetime X²=I, not from another interpreter run.
Original RNG code/distribution/seeds are unchanged. Both family tests pass.
Fresh normal suite: **274 passed / 0 failed / 5 ignored** (270 integration,
4 unit, 0 doctests; 29.53 s summed runner time under concurrent sweeps).
All-target strict Clippy and fmt --check pass (0 tests executed), as does the
unchanged Python seed (2 example objects, 2 printed envelopes).
Fresh affected ignored sweeps so far: 17-wire execution **1 passed / 0 failed**,
131,072 entries, 138.73 s; checked certificate **1 passed / 0 failed**,
262,144 domain elements, subject and expected program, 357.91 s. The 200,000
program discipline sweep passed **1 test / 0 failed**, **64,717 accepted (all
exactly clean) / 135,283 rejected**, 624.80 s. Three affected ignored tests
passed in total; the unrelated division and recursive-prefix sweeps were not
rerun. Commands (with the absolute manifest above, headless and offline):
`cargo test --test r1_borrow -- --ignored codex_b14_seventeen_wire_swap_identity_frame_exhaustive --nocapture`,
`cargo test --test r1_certificates -- --ignored b14_seventeen_wire_frame_with_checked_claim --nocapture`,
`cargo test --test r1_fuzz_discipline -- --ignored accepted_random_actions_run_exactly_clean_sweep --nocapture`.
Full exact commands and RED/targeted counts are in the council round-6 response.
Logs: `reference-interpreter/.tmp/r6-*.log`; actual TMPDIR for all gates:
`/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp`.
Standalone conformance: **445 pass / 0 fail /
48 unavailable / 493**, exit 2. Its current read-only checkout adds two explicit
unavailable contract-gap cases since the review's 491-case snapshot; no
unavailable case is counted as passed. No new ISA reading was needed.

Implementation commits: dceaa22 (shared core fixes/exact pins), 32ef2e6
(variants/C7/C8), 0d12ff1 (fresh borrow/certificate evidence and obsolete comment
removal). Tracking bead b-isa-spec-7dd is complete. No semantic source change
followed the passing normal and ignored suites; fmt/strict Clippy were rerun
after the comment removal. Only owned interpreter paths are committed; the
Beads CLI's automatic .beads/issues.jsonl export is left for the maintainer.
No push; accepted spec hash remains
90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e.

## Council round 5, codex/muse second pass (2026-10-30)

Codex R5-B01 resolved-by d0f7dbb and 738b233. Exact three certified witnesses
(including the third CX direction's published hashes) reject on HEAD before
production edits; straight-line/nested certificate forms reject too. New pin
`codex_r5_b01_exact_three_certified_lifetime_witnesses`: 1 passed. The
appendable response records the SOL proof/audit pins and positive controls.

Codex R5-B03 resolved-by e205f93: both exact one-iteration qfree placements
and the unrolled form already pass, with all three published interface hashes,
CLI check/run/reverse, full basis, superposition and entangled-reference pins.
X-before-free negatives still reject. New pin
`codex_r5_b03_one_iteration_release_matches_unrolled_clean_entry`: 1 passed.

Codex R5-B02: exact certified CZ already rejects at parse/hash/check on HEAD.
An additional public-API CZ pair against a clean auxiliary was RED (Proof),
then fixed: certificate borrow proofs now independently require permutation-only
borrowed touches, not just final tuples or reachable phase comparisons.
The full exact witness and phase-sensitive |++> pin, borrow_frames=[] negative,
and unrelated-H positive certify the intended boundary. Targeted tests:
31 pass / 0 fail / 2 ignored; normal suite: 256 pass / 0 fail / 5 ignored.
No decline, partial or new ambiguity. README records mandatory frame/return checks.

Muse's pure correctness/clarity suggestions applied: exact-zero measurement
helpers now terminate (RED overflow/timeout first), the negative draw threshold
is hoisted, draw order and containing-body frame maximality are documented and
pinned (SPEC-QUESTIONS 29/30). Codex's Bell projection diagnostic contract is
pinned and exact_support's comment clarified (normalized after projection,
raw shadows internal). Additional deterministic 28-case family: 8 accept /
20 reject, certificate-present and one-iteration cleanup/nonpermutation
coverage. Original random seeds/distribution unchanged. Targeted suggestion
checks: 50 integration tests and 2 new helper unit tests passed, 2 ignored.
Deliverables 8–11 and all existing unavailable/Unverified disclosures stay staged.

### Final codex/muse second-pass verification

All three codex blockers resolved; none declined or partial. Blocker commits:
**f108397** (B01, resolved-by d0f7dbb/738b233), **6635e4f** (B03, resolved-by
e205f93), **88a62ff** (B02 mandatory structural premise in public borrow
certificate verification). Pure correctness/clarity suggestions: **5be6fcf**.
See council/response-round-5.md for exact witnesses, RED evidence, pin names,
§12 claim-semantics quotes and each suggestion's disposition.

After the last Rust edits, rebuilt offline and ran the exact standalone command
from conformance/PROGRESS.md:
`cargo run --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/Cargo.toml -- --bin /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/target/debug/bennett-isa-ref`.
Result: **445 pass / 0 fail / 46 unavailable of 491**, exit **2** for existing
staging. CARGO_TARGET_DIR redirected build artifacts to this crate's ignored
`.tmp/conformance-target`. Conformance sources are read-only and unavailable
cases are never counted as passes; deliverables 8–11 remain staged below.

Then ran all required gates with absolute
`--manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml`:

| Gate | Result |
|---|---|
| `cargo test --offline` | **262 passed / 0 failed / 5 ignored** (258 integration + 4 unit, 0 doctests; 3.71 s summed runner time) |
| `cargo test --offline -- --ignored --nocapture` | **5 passed / 0 failed**, 353.16 s summed runner time |
| `cargo clippy --offline --all-targets -- -D warnings` | clean |
| `cargo fmt -- --check` | clean |
| unchanged Python `isa_check.py` | passes; diff against 71c1d6e empty |
| `git diff --check` | clean |

| Ignored sweep | Cases | Runner time |
|---|---|---|
| u8 division/remainder | 65,536 pairs × 4 operations = 262,144 executions | 1.75 s |
| 17-wire borrow execution | 2^17 entries | 10.24 s |
| checked borrow-frame certificate | 2^18 claim elements, both programs | 60.54 s |
| extended §6 differential | 200,000 programs: **64,717 accepted, all exactly clean / 135,283 rejected** | 271.54 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 9.09 s |

All validation ran with DISPLAY, WAYLAND_DISPLAY, DBUS_SESSION_BUS_ADDRESS and
BROWSER unset, and
TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/.tmp.
Logs: `.tmp/r5-codex-{conformance,tests,ignored,clippy,python}.log`.
Accepted spec SHA-256:
`90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e` (unchanged).
No Rust changes followed these gates. No spec/docs/conformance/include/worklog/
HANDOFF/CLAUDE or Python-seed edits, no dependencies or RNG changes, no push.
Bead b-isa-spec-zq2 remains open for the staged whole-ISA deliverables.

## Council round 5, SOL first pass (2026-10-30)

SOL-R5-B01 resolved after RED on the exact correctly bound certificate witness.
Definite Boolean/non-permutation failures cannot fall back to certificates;
full-domain library proof replay checks every completed return, including
restoration and independence before later pool reuse. Exact X/CX witnesses
reject at parse/hash/check with or without metadata. Targeted borrow/certificate
suite: 24 pass, 0 fail, 2 ignored; round-4 pins remain green. Reading 20 now
explicitly records per-completed-lifetime qualification. See appendable
council/response-round-5.md. SOL-R5-B02 also resolved after RED on the exact clean-d loop: Repeat
composes per-return restoration/independence obligations through every reachable
entry using doubling, preserving clean invariants without aggregate cancellation.
Trips 1/2/3/2^40, nesting, renamed unrolling, d=1 negatives and a second-entry
toggle pin pass. Borrow/certificate/generator targets: 32 pass, 0 fail, 3 ignored.
SOL-R5-B03 resolved after independent CLI/library RED witnesses. Workspace
widths were verified on the tested straight-line and self-contained loop
producing/consuming lifetimes, including inverse releases. Round-6 evidence
above extends this to zero-trip and loop-carried width changes. Exact hashes, basis/superposition, mixed qcopy/qload/qborrow,
alpha-renamed counterparts, nested loops and entangled references pass in both
directions; quantum/CLI targets: 42 pass, 0 fail. SOL-R5-B04 resolved after
RED on the exact two-wire sparse observer. A projection marker normalizes
sparse conditional diagnostics with extended exponents while retaining raw
exact shadows and caller-supplied input scale. Tiny entangled residuals and
subsequent Born results pass; quantum/register/CLI targets: 54 pass, 0 fail.
All four received SOL blockers are resolved; final complete gates are recorded below.
Additional deterministic coverage: 128 certified/uncertified lifetime and clean
context cases (64 accept / 64 reject), plus 18 reused/renamed-width positives.
Direct certificate verification rejects invalid internal returns and frames
even when the whole-region claim proves identity. An additional RED control
found that basis-only audit observations narrowed valid superposed contexts;
audits now compare exact surviving quantum states and observe non-permutation
frame boundaries as well as returns. Original random distribution is unchanged.

Correction to round-4 evidence: the original X/CX rejection pins lacked
certificate-adorned subjects, and generic-body proof discarded clean entry
invariants. Round 5 covers these separately; prior counts below are historical.

### Final round-5 SOL verification

Blocker commits: **d0f7dbb** (B01), **e205f93** (B02), **860ebbd** (B03),
**f34ff55** (B04). Additional admission/metamorphic coverage: **6188112**;
quantum-context and full-frame audit correction/controls: **738b233**.
Every received blocker is resolved, none declined or partial. The response
remains appendable for pending codex/muse seats reviewing 5d6d03e.

After the final source change, rebuilt offline and ran the exact standalone
harness command from conformance/PROGRESS.md:
`cargo run --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/Cargo.toml -- --bin /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/target/debug/bennett-isa-ref`.
Result: **445 pass, 0 fail, 46 unavailable / 491**, exit **2** for the existing
staged deliverables 8–11. No unavailable case is counted as a pass. Build
artifacts were redirected with CARGO_TARGET_DIR to this crate's ignored
`.tmp/conformance-target`; conformance/include sources remained read-only.

Then ran the complete required gates, each Rust command with absolute
`--manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml`:

| Command | Result |
|---|---|
| `cargo test --offline` | **253 passed, 0 failed, 5 ignored** (251 integration + 2 unit; 0 doctests), 4.56 s summed runner time |
| `cargo test --offline -- --ignored --nocapture` | **5 passed, 0 failed**, 268.07 s summed runner time |
| `cargo clippy --offline --all-targets -- -D warnings` | clean |
| `cargo fmt -- --check` | clean |
| unchanged Python `isa_check.py` | passes; diff against 5d6d03e empty |
| `git diff --check` | clean |

| Ignored sweep | Cases | Runner time |
|---|---|---|
| u8 division/remainder | 65,536 operand pairs × 4 operations = 262,144 executions | 2.20 s |
| 17-wire borrow execution | 2^17 basis entries | 12.29 s |
| checked borrow-frame certificate | 2^18 claim elements, both programs | 64.41 s |
| extended §6 differential | 200,000 programs: **64,717 accepted, all exactly clean**, 135,283 rejected | 180.51 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 8.66 s |

Independent CLI replay additionally passes **72 clean-context checks** for B02:
trips 1/2/3/2^40, flat/nested, t=0/1 and complex superposition, forward/adjoint
and equals_entry reversal. These use the hash-bound regression subjects.
All round-4 tests remain green. Original RNG distribution/Python seed unchanged.

All commands ran headlessly with DISPLAY, WAYLAND_DISPLAY,
DBUS_SESSION_BUS_ADDRESS and BROWSER unset, and
TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/.tmp.
Logs: `.tmp/r5-sol-{conformance,tests,ignored,clippy,python,clean-cli}.log`.
Accepted 0.8 SHA-256 remains
`90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e`.
No source changes followed these gates. No dependencies, spec/docs/conformance/
include/worklog/HANDOFF/CLAUDE edits or pushes. Bead b-isa-spec-zq2 stays open
for staged whole-ISA work; its round-5 notes are updated through bd.

## Council round 4, muse third pass (2026-10-30)

MUSE-R4-B01 **resolved by 40ef8ce**, independently pinned in this pass. The
new exact CX witness was RED on archived round-4 source 8e11fe7, GREEN on the
current source: each Return proves restoration and independence before Repeat
composes only its carried-wire map. Added parser/hash/check negatives for
both CX directions, trips 2/4, nesting and distinct-name unrolling, plus carried
and local doubled-CX positives at 2 and 2^40 iterations. Every positive preserves
all t,b basis inputs, superposition and an entangled external reference exactly
in both directions. No loop cap or unrolling was added; the Repeat comment now
states the local-lifetime invariant, including inlined children.

Two separate deterministic families add **64 fresh-CX lifetime cases (32 accept /
32 reject)** and **128 nested-call/mutable-region/borrow-pool cases (96 accept /
32 reject)**. Expectations come from per-return freshness and an independent
small Boolean map over the complete interface and memory. All accepted nested
cases check 32 basis inputs, full superposition and an entangled borrowed
reference against that map, then reverse exactly. Existing RNG/Python seeds
and random-sweep distribution are unchanged.

Non-blocking improvements: refuse vacuous --samples 0 (§17.13), cite §5.4 and
§16.1 in all four reported malformed-option errors, remove seven handle-map
expects and the SSA retry expect, add independent literal SCC/CC-service
envelopes, five literal sum_db step/stack boundary tuples, and 132 normal-suite
recursive prefixes for n=0..3. The CLI witnesses were RED before the fixes.
Full suggestion-by-suggestion dispositions (including coverage/API/spec-edit
declines and already-applied earlier suggestions) are in council/response-round-4.md.
No blocker declines/partials and no new spec ambiguity; SPEC-QUESTIONS unchanged.

Read-only conformance at **92c26af**: **445 pass, 0 fail, 46 unavailable / 491**,
exit **2** for the existing staged deliverables. The extra unavailable target
case is disclosed by the updated harness; it is not an interpreter regression.
The exact command from conformance/PROGRESS.md used the absolute reference
binary and redirected CARGO_TARGET_DIR into this crate's ignored `.tmp`.
Conformance/include/spec/docs/worklog/HANDOFF/CLAUDE remained unedited.

### Final muse-pass verification

| Command | Result |
|---|---|
| `cargo test --offline` | **242 passed, 0 failed, 5 ignored** (241 integration + 1 unit; 0 doctests; 12.05 s summed runner time) |
| `cargo test --offline -- --ignored --nocapture` | **5 passed, 0 failed**; 596.07 s summed runner time |
| `cargo clippy --offline --all-targets -- -D warnings` | clean |
| `cargo fmt -- --check` | clean |
| unchanged Python seed `isa_check.py` | passes; seed diff against 8fa5d98 empty |
| standalone conformance | **445 pass, 0 fail, 46 unavailable / 491**, exit 2 for explicit staging |
| `git diff --check` | clean |

| Ignored sweep | Cases | Runner time |
|---|---|---|
| u8 division/remainder | 65,536 operand pairs × 4 operations = 262,144 executions | 4.27 s |
| 17-wire borrow execution | 2^17 basis entries | 21.55 s |
| checked borrow-frame certificate | 2^18 claim elements, both programs | 119.32 s |
| extended §6 differential | 200,000 programs: **64,717 accepted, all exactly clean**, 135,283 rejected | 432.24 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 18.69 s |

All Rust commands used the absolute `--manifest-path
/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml`.
All validation ran headlessly with DISPLAY, WAYLAND_DISPLAY,
DBUS_SESSION_BUS_ADDRESS and BROWSER unset, and
TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/.tmp.
Logs: `.tmp/r4-muse-{tests,ignored,clippy,conformance,python}.log`; archived RED
logs: `.tmp/r4-muse-{borrow,options}-red.log`. Independent Python hashlib
reproduced the literal SCC digest 05e79eab6ee962e7d1c2c9c4f4db5d27f07ef0e0231b6d394ab802d24bea7724
and both ASCII-suffixed member hashes. No Rust changes followed these gates.
No dependencies, spec edits, changed seeds or pushes. Bead b-isa-spec-zq2 stays
open for the whole-ISA staged deliverables, including the proposed first
F-link-conflict/F-frame-live-ranges promotions when their surfaces are ready.

## Council round 4, sol second pass (2026-10-29)

SOL-R4-B01 resolved: exact-shadow Born weights and refined outward-rounded
CDF comparisons preserve nonzero branches under probability/amplitude
underflow. Conditional display amplitudes normalize with a shared extended
exponent; later measurements use exact norm ratios. Exact CLI and independent
entangled-residual regressions were witnessed RED, then GREEN, including the
smallest positive binary64 angle and an amplitude below binary64 range.
Normal suite: **226 passed, 0 failed, 5 ignored**; initial strict Clippy clean.
Details and named pins: council/response-round-4.md. No new ambiguity or cap.
The response remains appendable for the pending muse seat.

SOL-R4-B02 resolved after RED on the exact truncated frame and the nested-line
positive. Canonical point resolution and an opt-in borrow compilation trace
check the complete named V;W;reverse(V);W circuit, actual borrowed identities,
inherited controls, syntactic side conditions and omitted adjacent touches.
Three new tests exercise verify_claim and verify_certificate independently of
borrow_frames admission, including full/nested positives and endpoint, changed-W,
fake-B and omitted-gate negatives. Full suite: **229 passed, 0 failed, 5 ignored**;
strict Clippy clean. No spec question.

SOL-R4-B03 resolved: claim domains retain ABI group/slot and region cell index,
so expected programs receive the corresponding inputs under any local renaming.
Borrowed entries preserve domain order; genuinely omitted inputs stay zero.
The exact four-element witness and renamed-region analogue were both RED.
Three new regressions now prove these identities, reversed enumeration,
crossed local names and multiple same-width slots, while single-X, wrong-slot
and memory-only mismatches reject. Partial domains remain evidence.
Normal suite: **232 passed, 0 failed, 5 ignored**. No new ambiguity; reading 21
is unchanged. All SOL blockers resolved; muse response remains appendable.
Blocker commits: **3ba7154** (SOL-B01), **21da2cc** (SOL-B02),
**d7a14fd** (SOL-B03).

### Final round-4 sol-pass verification

Rebuilt offline, then ran the exact standalone conformance command documented
in conformance/PROGRESS.md against the absolute reference binary. Harness
result: **445 pass, 0 fail, 45 unavailable / 490**, exit **2** for the existing
staged deliverables 8–11. No conformance/include source or header was edited;
CARGO_TARGET_DIR redirected harness build artifacts into the interpreter's
ignored `.tmp/conformance-target`.

Then ran the required gates, with absolute `--manifest-path
/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml`:

| Command | Result |
|---|---|
| `cargo test --offline` | **232 passed**, 0 failed, 5 ignored (231 integration + 1 unit; 0 doctests; 5.55 s summed runner time) |
| `cargo test --offline -- --ignored --nocapture` | **5 passed**, 0 failed; 336.09 s summed runner time |
| `cargo clippy --offline --all-targets -- -D warnings` | clean |
| `cargo fmt -- --check` | clean |
| unchanged Python seed `isa_check.py` | passes; diff against 2f167cc empty |
| `git diff --check` | clean |

| Ignored sweep | Cases | Runner time |
|---|---|---|
| u8 division/remainder | 65,536 operand pairs × 4 operations = 262,144 executions | 2.80 s |
| 17-wire borrow execution | 2^17 basis entries | 15.28 s |
| checked borrow-frame certificate | 2^18 claim elements, both programs | 76.68 s |
| extended §6 differential | 200,000 programs: **64,717 accepted, all exactly clean**, 135,283 rejected | 220.24 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 21.09 s |

Every validation command ran headlessly with DISPLAY, WAYLAND_DISPLAY,
DBUS_SESSION_BUS_ADDRESS and BROWSER unset, and
TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/.tmp.
Logs: `.tmp/r4-sol-{tests,ignored,clippy,conformance,python}.log`.
No source changes followed these gates. No dependencies, changed seeds,
spec edits, declines, partial SOL resolutions or pushes. All committed edits
are under reference-interpreter; bd notes track this pass on b-isa-spec-zq2.
The full-ISA bead remains open for staged work; muse's response may follow.

## Council round 4, codex first pass (2026-10-29)

R4-B01 resolved after RED on the exact loop-local borrow witness. Borrow/Return
nodes preserve lifetime boundaries through loops and adjoints; every return
proves identity and output independence before repetition can compose it.
Spanning lifetimes retain whole-loop restoration. Exact, unrolled, nested and
output-dependency negatives reject at parser/hash/check; positive controls
preserve all t,b bases, superposition and an entangled reference forward/adjoint.
The new deterministic 32-case family checks trip counts 1..4 against per-return
parity: 16 accepted / 16 rejected. Existing RNG/Python seeds unchanged.
Normal suite after B01: **217 passed, 0 failed, 5 ignored**. 11 active borrow
and 5 active generator tests pass. Response: council/response-round-4.md;
appendable for sol/muse.

R4-B02 resolved: physical region slot disjointness is checked before handle
deduplication; effective controls are bound to transitive actual targets.
Only specified alias preconditions become terminal fault-4 frames; internal
errors remain diagnostics. Exact CC/R select witnesses, oracle aliases,
regions, block parameters, nested targets and directly aliased R bindings
are pinned. Both quantum state and handles are unchanged on fault and complete
prefix reversal restores entry. CLI run-cc/run-program give FAILED/fault=4/
steps=2/results=[]; reverse equals_entry=true. Disjoint c=0 remains three-step
completion and control/read-only-input overlap remains legal. Both main
witnesses were RED; replaying the five library pins against pre-fix source
found oracle aliases had silently completed as well. GREEN: 11 register +
29 CLI tests; full normal suite **223 passed, 0 failed, 5 ignored**.
Blocker commits: **40ef8ce** (B01), **09fe805** (B02).

### Final round-4 first-pass verification

All cargo commands used the absolute manifest path
`/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml`.
All verification ran with DISPLAY, WAYLAND_DISPLAY, DBUS_SESSION_BUS_ADDRESS,
and BROWSER unset and
TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/.tmp.
Logs are retained in ignored `.tmp/r4-{tests,ignored,clippy,conformance}.log`.

After rebuilding the binary offline, ran the exact standalone harness command
from conformance/PROGRESS.md, with absolute conformance manifest and `--bin`
paths. `CARGO_TARGET_DIR` kept build artifacts under the interpreter's ignored
`.tmp/conformance-target`; conformance/include sources remained read-only.
Result: **445 pass, 0 fail, 45 unavailable / 490**, exit **2** for existing
staged capabilities, unchanged from the reviewed baseline.

Then `cargo test --offline` passed **223 tests, 0 failed, 5 ignored**
(222 integration + 1 unit; 0 doctests; 2.53 s summed runner time).
`cargo clippy --offline --all-targets -- -D warnings` and
`cargo fmt -- --check` pass. Unchanged Python seed check passes and its diff
against 8e11fe7 is empty. README documents lifetime boundaries and dynamic
alias fault reporting. No dependencies added; no pushes.

`cargo test --offline -- --ignored --nocapture`: **5 passed, 0 failed**.
All original random seeds are unchanged; the loop-lifetime family is separate.

| Ignored sweep | Cases | Runner time |
|---|---|---|
| u8 division/remainder | 65,536 operand pairs × 4 operations = 262,144 oracle executions | 2.22 s |
| 17-wire borrow execution | 2^17 basis entries | 15.45 s |
| checked borrow-frame certificate | 2^18 claim elements, both programs | 64.53 s |
| extended §6 differential | 200,000 programs: **64,717 accepted, all clean**, 135,283 rejected | 200.70 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 10.29 s |

Both received codex blockers are resolved with individual commits. The editor
response stays appendable for the pending sol/muse second pass. Only
reference-interpreter paths were committed, with bd comments for tracking.

No new ambiguity, decline, spec edit or narrowed capability. Bead b-isa-spec-zq2.

## Council round 3, codex second pass (2026-10-29)

R3-B01 is resolved-by c2d97f8 (SOL-R3-B01): the exact six-row codex witness
and fault-to-input variant reject at load and CLI hash/check (§5.5). New pin
`codex_r3_b01_exact_implicit_delivery_witnesses` also verifies the exact
five-row positive (0,0,5) and full prefix reversal. All 17 r1_rules tests pass.
No source change needed for this blocker; no spec/Python seed edits.

R3-B02 is resolved-by f83fc71 (SOL-R3-B02): exact swap/CX-child negatives
reject §6.2 in parser and hash/check; mutable-region analogue also rejects.
Doubled-CX controls restore all basis and superposed entries forward/adjoint.
The call-wrapping generator now covers swap, CX and mutable-region exchange:
36 cases, 18 accepted / 18 rejected by independent parity, full execution of
accepts. 17 discipline + 4 active generator tests pass; random seeds unchanged.

R3-B03 resolved after RED on the exact witness, ff image and zero-trip
lexical declaration: borrow compilation now wraps every function/inlined child
in complete entry/body/exit region lifetimes. Allocations are clean, image Xs
explicit and unconditional, so adjoints preserve preparation/restoration.
Three regressions cover both call directions and inherited predicates, all
t,b bases, superposition and an entangled external reference, plus CLI t=0/1
run/reverse and checked hashes. 9 active borrow tests pass, 1 ignored; full
normal suite: 214 passed, 0 failed, 5 ignored. SPEC-QUESTIONS 28 gains pin
references, without new ambiguity. All three second-pass blockers resolved;
no declines, no scope narrowed. Blocker commits: 377e807 (B01 pin), faf6dff
(B02 pins/generator), fbe8320 (B03 fix/pins).

### Second-pass final verification

After the fixes, rebuilt the reference binary offline and ran the standalone
conformance command documented in conformance/PROGRESS.md:
`cargo run --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/Cargo.toml -- --bin /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/target/debug/bennett-isa-ref`.
Conformance source is read-only; `CARGO_TARGET_DIR` was redirected to
`reference-interpreter/.tmp/conformance-target`. Result: **445 pass, 0 fail,
45 unavailable / 490**, exit **2** for the existing staged capabilities.

Then ran the required Rust commands with absolute `--manifest-path
/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml`.
Every command ran headlessly with DISPLAY, WAYLAND_DISPLAY,
DBUS_SESSION_BUS_ADDRESS and BROWSER unset, and
TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/.tmp.
Logs: ignored `.tmp/r3-codex-{tests,ignored,conformance,clippy}.log`.

| Command | Result |
|---|---|
| `cargo test --offline` | **214 passed**, 0 failed, 5 ignored (213 integration + 1 unit; 0 doctests; 2.90 s summed runner time) |
| `cargo test --offline -- --ignored --nocapture` | **5 passed**, 0 failed |
| `cargo clippy --offline --all-targets -- -D warnings` | clean |
| `cargo fmt -- --check` | clean |
| unchanged `python3 isa_check.py` | passes; seed diff against 788a5e0 empty |
| conformance harness | **445 pass, 0 fail, 45 unavailable**, same staging as baseline |

| Ignored sweep | Cases | Runner time |
|---|---|---|
| u8 division/remainder | 65,536 operand pairs × 4 operations = 262,144 oracle executions | 2.30 s |
| 17-wire borrow execution | 2^17 basis entries | 12.42 s |
| checked borrow-frame certificate | 2^18 claim elements, both programs | 84.11 s |
| extended §6 differential | 200,000 programs: **64,717 accepted, all clean**, 135,283 rejected | 280.77 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 10.78 s |

No spec, docs, conformance, include, worklog, or Python seed edits; no new
dependencies; no push. Bead b-isa-spec-zq2 remains open for the full-ISA work.

## Council round 3 (2026-10-29)

Received SOL-R3-B01..B05 on snapshot 2e61144; response-round-3.md is appendable
for the pending seats. B01 resolved with a witnessed RED then GREEN (16
r1_rules tests): nested target/fault/uncall deliveries invalidate outstanding
input readers while commuting output deliveries remain legal. B02 resolved:
joint call dependency/provenance and inverse groups, including mutable cells;
17 discipline and 3 generator tests pass, including odd/even call wrapping.
B03 resolved: opaque applications retain recursively substituted inputs during
loop squaring. Exact loop/unrolled negatives reject; invariant inputs and 32
changing-input/nested generated loops pass their independent parity checks.
SPEC-QUESTIONS 20 now precisely describes the opaque symbolic proof route.
B04 resolved: normalized CC controls are typed qreg[1] through rd and checked
for effective-target aliases; five negative variants and basis/forward-type-
inference execution/reversal positives pass. B05 resolved: non-permutation
call touch sets include effective controls and passed cells. Direct/ctl/qpred
borrowed-H negatives reject; paired borrowed-controlled X calls preserve all
basis inputs and a superposition. All five received blockers resolved, no
declines or partials. Bead b-isa-spec-zq2.

### First-pass round-3 verification

Every command used the absolute manifest path and ran headlessly with
DISPLAY, WAYLAND_DISPLAY, DBUS_SESSION_BUS_ADDRESS and BROWSER unset;
TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/.tmp.
Logs are retained in ignored `.tmp/r3-{tests,ignored,conformance}.log`.

| Command | Result |
|---|---|
| `cargo test --offline` | **210 passed**, 0 failed, 5 ignored (209 integration + 1 unit; 0 doctests; summed runner time 2.25 s) |
| `cargo test --offline -- --ignored --nocapture` | **5 passed**, 0 failed |
| `cargo clippy --offline --all-targets -- -D warnings` | clean |
| `cargo fmt -- --check` | clean |
| unchanged `python3 isa_check.py` | passes; seed diff from 2e61144 is empty |
| conformance standalone harness with absolute `--bin` | **445 pass, 0 fail, 45 unavailable / 490**; exit **2** for staged capabilities, identical to baseline |

| Ignored sweep | Cases | Runner time |
|---|---|---|
| u8 division/remainder | 65,536 operand pairs × 4 operations = 262,144 executions | 2.96 s |
| 17-wire borrow execution | 2^17 entries | 16.06 s |
| checked borrow-frame certificate | 2^18 claim elements, both programs | 72.61 s |
| extended §6 differential | 200,000 programs: **64,717 accepted, all clean**, 135,283 rejected | 209.69 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 11.19 s |

The dedicated generated families additionally exercise 12 direct/call-wrapped
Bell-cleanup variants and 32 opaque-loop/nested-loop variants, against independent
parity formulas. Accepted programs run on their full small basis domains plus a
superposed interface and restore exactly under adjoint. Each blocker has a
witnessed development-time RED and a named GREEN regression in the appendable
`council/response-round-3.md`. No capabilities or new ambiguities were declined.

## Council round 2: sol/codex implementation (2026-10-29)

The 15 blockers from the dc46d07 round-2 snapshot are implemented in ten
coincident groups; `council/response-round-2.md` has one entry per member ID
and is appendable for muse's pending second pass. No declines or partial
round-2 blocker resolutions. Deliverables 8–11 remain staged below; this
round does not claim they are implemented. Bead: b-isa-spec-zq2.

Changes: typed matched-invocation tuples and all-row qjumpr CFG; canonical
lifted memory suffixes and integer/ptr-only comparisons; expansion-owned
predicate protection; function-entry qregions in both static checkers;
qinplace explicit inverse recognition; complete classical environments in
borrow proofs; joint exchange dependency/provenance and entry-version
identities; full effective-control polarity in memory/phase/change identities;
bounded-factor phase rendering for finite rotations with unbounded combined
angles; collision-free interned references to full version keys (preventing
geometric identity-text growth found by the extended sweep). SPEC-QUESTIONS 28 resolves cross-arm unconditional lifetimes by taking
the adjoint of the full qif expansion (else before then); entry 21 and interval
claim diagnostics now consistently name deliverable 11.

Every cause had a development-time red witness. The final tests were also
compiled against archived dc46d07 in ignored `.tmp`, observing the independent
exchange, argument/literal angle, member-specific call, suffix, operand-kind,
and declaration variants fail there. Extended differential generation includes
swap/qexch/qexchm plus 80 generated borrow frames with differing classical
bindings. It found additional unsound exchange/polarity accepts, now pinned by
named red-green regressions in the response. No Python seed or spec changes.

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
| u8 division/remainder | 65,536 operand pairs × 4 operations = 262,144 executions | 2.39 s |
| 17-wire borrow-frame execution | 2^17 basis entries | 11.83 s |
| checked borrow-frame certificate | 2^18 claim elements, both programs | 82.92 s |
| extended §6 differential generator | 200,000 programs: **64,717 accepted, all exactly clean**, 135,283 rejected | 259.39 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 12.90 s |

The accepted differential programs run on all eight interface basis inputs and
one superposed input, forward and adjoint. The sweep includes the new exchange
operations. No runtime width, iteration, precision, or program-size cap was added.
Specification and Python seed diffs against dc46d07 are empty. No push.

## CLI commands (xam)

Bead `b-isa-spec-xam` (brief `docs/briefs/2026-10-29-isa-ref-cli-xam.md`):
four commands for the dialect 0.2 pre-flight checker, exposing the existing
library machinery. Tests are `tests/cli_xam.rs` against the existing fixtures
(`classical`, `actions`, `registers`, `inc8`), every expected value
hand-derived in the test comments.

| Command | Status | Test | Commit |
|---|---|---|---|
| `run-cc` | done | `cli_xam::run_cc_sum_reports_every_step_with_register_deltas` | bc63483 |
| `reverse` | done | `cli_xam::reverse_oracle_prefix_reversal_restores_the_basis_tuple` | d987500 |
| `run-action` | done | `cli_xam::run_action_permutation_basis_and_adjoint` | 9d4c0dc |
| `run-program` | done | `cli_xam::run_program_region_calls_complete_and_close` | 623a9bb |

Resumption review preserved and completed the uncommitted README, budget
reporting and arbitrary-width CLI input work. README Commands now has exact
structural JSON schemas for all four commands (including each reverse form)
and an invocation per command. All 26 `cli_xam` tests were copied into an
archived pre-CLI `ddf356c` source tree and failed there; they pass on the
current implementation. The expectations derive from fixture rows and ISA
rules, including an independent decimal expansion of 2^299 + 1.

Three diagnostics regressions were red in the resumed tree: ambiguous
arguments and a misplaced `--steps` lacked ISA citations; a nonexistent
`--target` reported only an I/O error instead of naming staged deliverable 9.
They now pass with exit 1, a rule citation and empty stdout. Extra tests pin
ordered token budgets across CC/reverse/program commands and exact i128
measurement reporting. No spec or Python seed changes. Deliverables 8–11
remain staged as detailed below; no capability was declined.

Final xam verification (headless, project-local `.tmp`, all four display /
browser variables unset):

| Command | Result |
|---|---|
| `cargo test --offline` | 178 passed, 0 failed, 5 ignored (26 CLI tests) |
| `cargo test --offline -- --ignored` | 5 passed, 0 failed |
| `cargo clippy --offline --all-targets -- -D warnings` | clean, no tests executed |
| `cargo fmt -- --check` | clean, no tests executed |

Each command used the absolute `--manifest-path` to this crate. The normal
suite and the ignored sweeps were run after the final Rust changes.

## Session 2026-10-29 (Opus, council fix loop): milestone 1 reached

All 30 round-1 blocking issues are resolved, declined with a spec quote, or
(B19) partial with staging; the editor's response is
`council/response-round-1.md`, the readings `SPEC-QUESTIONS.md` (entries 4, 6,
11, 12 revised, 14–27 new). Each witness is a test in `tests/r1_*.rs`, run red
against the handover checkpoint `0164b5a` before its fix (results in the
response file).

### Verification (this commit)

| Command | Result |
|---|---|
| `cargo test --manifest-path reference-interpreter/Cargo.toml --offline` | 152 passed, 0 failed, 5 ignored |
| `cargo test ... --offline -- --ignored` | 5 passed in 4 min 57 s wall time: see the sweep table |
| `cargo clippy ... --offline --all-targets -- -D warnings` | clean |
| `cargo fmt --manifest-path reference-interpreter/Cargo.toml -- --check` | clean |
| `python3 reference-interpreter/isa_check.py` (unchanged) | passes |

Ignored sweeps (debug build, this machine):

| Sweep | Cases | Time |
|---|---|---|
| `tests/oracle.rs` `exhaustive_unsigned_and_signed_division_u8` | 65,536 operand pairs × 4 operations = 262,144 executions | 1.64 s |
| `tests/step_machine.rs` recursive prefixes (all 8-bit n, every prefix) | 256 inputs × every B to completion | 8.87 s |
| `tests/r1_borrow.rs` 17-wire swap frame, exhaustive execution | 2^17 × 2 | 9.02 s |
| `tests/r1_certificates.rs` 17-wire frame with its checked claim | 2^18 claim elements, both programs | 50.16 s |
| `tests/r1_fuzz_discipline.rs` §6 differential sweep | 200,000 programs, 82,797 accepted and all exactly clean, 117,203 rejected | 227.05 s |

### What changed in milestone 1 (summary; details per blocker in the response)

- One engine pair on bit-vectors of any width (`oracle.rs`, `machine.rs`), u128
  adapters only; machine counters/cells/slots at the §3.7 derived widths.
- Canonical INT/FLOAT everywhere; const range; iN suffix schema; one call spelling.
- `schema.rs`: §4.8 schemas before interprocedural passes; §5.1 region graph.
- §5.5 dataflow pairing of oracle uncalls; §7.4 whole-run reversal ends at entry.
- §6: windows open at computation; transitive reads; phase kickback; predicate
  versions; a differential fuzzer against the exact simulator.
- `exact.rs` + sparse `quantum.rs`: exact cleanliness, no wire limit.
- `borrow.rs`: exact ANF decision of §5.6 frames; no 16-wire or loop caps.
- `claims.rs`: exhaustive/sampled/structural claim checking, the library
  borrow route, `verify_certificate`.
- qinplace §4.5 conditions checked exactly under the actual token.
- Linking composes certificates; closure hashes over pinned imports.
- Tiers (R → R only; CC → CC/R), typed CC traps, §7.3 effect table at load.
- CLI: `step-check --exhaustive|--suite|--samples`, `check` reports `unlinked`.

### Deliverable status

| Deliverable | Status | Evidence |
|---|---|---|
| 1. Parser and well-formedness | Implemented for every §16.2 production and the suite-F rules named in round 1; §6 checker cross-checked by the differential fuzzer | `tests/parser.rs`, `structured.rs`, `discipline.rs`, `r1_literals.rs`, `r1_rules.rs`, `r1_discipline.rs`, `r1_fuzz_discipline.rs` |
| 2. Identity and linking | Envelopes, SCCs, the four §16.3 hashes, pins (closures over pinned imports), dedup, certificate composition | `identity.rs`, `linking.rs`, `r1_linking.rs`, `r1_fixtures.rs` |
| 3. §14.1 interpreter | Every §4.2 row on bit-vectors of any width; every fault code | `oracle.rs`, `r1_widths.rs`, division sweep |
| 4. Step machine | Literal forward/inverse steps, derived widths, T2/T6 sweeps | `step_machine.rs`, `wide_oracle.rs`, `r1_widths.rs`, recursive sweep |
| 5. CC/R tiers | History, tapes, devices, hostcall, uncall, tiers, traps | `classical.rs`, `registers.rs`, `r1_rules.rs` |
| 6. Q tier and §6 | Exact simulator, actions, borrow frames, qinplace, §6 checker | `quantum*.rs`, `borrow.rs`, `r1_exact.rs`, `r1_borrow.rs`, `r1_qinplace.rs` |
| 7. CLI and docs | check/hash/run-oracle/step-check plus run-cc/reverse/run-action/run-program, README schemas and examples | `cli.rs`, `cli_xam.rs` |
| 8–11 | Staged (below) | — |

### Staging of deliverables 8–11 (codex B19)

The whole-ISA scope includes §8, §10–§12 and suites D, E, G, H. They are staged
in this order after milestone 1; nothing is exempt, and every claim or path
that needs them already reports so explicitly (`ClaimVerdict::Unverified`).

8. **Launch ABI (§10), suite G.** The one-shot `run-program` CLI now exposes
   positional launch, HOST_REQUEST/reply, results/faults/measurements, optional
   reversal and close. The full library ABI remains staged: a Rust API
   mirroring every C entry point
   (`bennett_load` … `bennett_coverage_get`) over `ClassicalMachine`: the
   CREATED → … → CLOSED state machine with E_INVALID_STATE checks (waiting on
   SUSPENDED/REVERSED/CLOSED, unload with an unclosed run), positional binding
   with E_ARG, copied arguments and entropy, HOST_REQUEST/reply, results and
   fault records, reverse eligibility (no measure, untouched registers,
   E_NOT_REVERSIBLE), the invocation tree and post-order reversal, tape
   iteration (64-bit fields), `bennett_budget` validation. Tests: every
   transition, the inc8 launch with budget (3,0,0,0) → 9 and its
   reverse/close lifecycle. First, because it needs only the CC/R machine.
9. **Target descriptions (§8), suite E.** JCS target parsing and `target_hash`;
   the closed expression AST over exact rationals with the per-context variable
   binding table and load-time errors (division by zero, table index,
   unbound variable, `log2inv_eps` outside (0, 1]); resources, classes, gates,
   schemes, capabilities (`mbu` false); matching (first `gates` entry by op,
   classes, total control count and epsilon, else schemes; template expansion
   with `repeat`); aggregation from scheduled event lists (count, time,
   occupancy, peak); timing; imports.
10. **Schedule consumer (§11), suite H (consumer side).** The `ref` expansion
    library (§17 item 7) with generated-operation ordinals; invocation chains;
    path matching with the specificity order and the invalid-schedule rules;
    defaults; timing constraints; binding validation against object and target;
    verbatim execution or E_SCHEDULE_INFEASIBLE.
11. **Rigorous certificate checker (§12.1), suites C and D.** Interval
    arithmetic at `precision_bits` with outward rounding, §17 item 8 Taylor
    enclosures, realised versus reference unitaries per invocation context
    (needs 9 and 10), the Frobenius operator-norm bound, the diamond-norm dual
    witness check with interval Cholesky (strictly positive pivots), fault-rate
    verification and the inclusive-error recursion; `expected: "oracle"` claims.

### Resume here: outstanding work, in priority order

1. Deliverables 8–11 above, in that order.
2. Broaden randomized §6 call/mutable-region/borrow generation beyond round 3
   dedicated child-call Bell/parity and changing-input opaque-loop families;
   extend the borrow-frame
   symbolic decision to black-box calls whose inputs depend on borrowed wires
   (currently the claim route).
3. Claims on regions with classical arguments (the claim domain carries none;
   SPEC-QUESTIONS 21).
4. Independent second-implementation checks of the CC/R fixture hashes (the
   Python seed cannot hash CC closures; the stored hashes are the crate's own).
5. Rerun every gate after each change: tests, ignored sweeps, clippy, fmt, seed.

---

# History (codex job, superseded where the milestone-1 section above differs)

## Session 2026-10-29

### Current checkpoint: wide reference execution and literal reversible machine

Wide arithmetic is now connected through the reference engine, literal step
machine, Q oracle construction and both CLI execution commands. Public APIs:
BitVector; OracleVectorInput/Outcome; Object::run_oracle_vectors;
OracleVectorMachine and its complete BasisVectorConfiguration snapshot.
The previous u128 APIs remain compatible for small results; the u128 machine
explicitly directs wide closures to OracleVectorMachine instead of panicking.

Red-green: --test wide_oracle first failed on the missing vector machine.
Implemented src/wide_machine.rs with the same printed dispatch/slot/exchange,
call/ret and inverse lists using concrete bit-vectors; no inverse snapshots.
The all-row comparison regression exposed callee parameters padded to128
instead of their declared width; fixed reads to enforce the static width.
Five tests now pass:257-bit arithmetic/faults, high-bit Q XOR delivery,
all prefix/stack boundaries of wide calls and faults, comparisons over all
existing arithmetic/heap/memory/nested/permutation fixtures, and legacy API
representation diagnostics. A narrow machine test explicitly failed with a
shift overflow before its checked adapter diagnostic was added.

wide.bisa has hand expectations and valid hashes; all12 canonical fixtures
are identity-checked. CLI --test cli wide_oracle first failed on u128 input
parsing; now the quotient2^100-5 prints exactly as a decimal string. A second
CLI test first failed converting the high fault code to u128; step-check now
uses the vector machine and checks a one-input-bit oracle with257-bit target
and fault wires. Four CLI tests pass. The --max-bits cap restricts the domain
only, not target or fault representation.

Another red-green borrow test: (CX b,t; CX t,b)^3 is identity, but the old
qreturn version-log check rejected it before its full-domain frame proof.
qreturn now relies on the required §5.6 frame verifier; qfree still requires
workspace cleanup. All six borrow tests pass. Hand check: the two-gate map
(b,t)->(t,b XOR t) has order3 over GF(2), so every b/t tuple is restored.

Most recent full cargo test before the final CLI/borrow edits passed87 normal
tests; strict Clippy passed. Current expected normal count89; rerun all tests,
ignored sweeps, fmt, Clippy and seed at the next milestone. No user decision is
needed. Immediate next work: audit descriptor/address/counter bounds, close
remaining suite-F memory/uncall and borrow/cell-version checks, implement
certificate execution/composition and target/schedule contracts. All original
seven deliverables remain partial until their full obligations are verified.

### Latest checkpoint: canonical literals, Q verification, arbitrary-width values

`cargo test --manifest-path reference-interpreter/Cargo.toml --offline --quiet`
PASSED 83 normal tests, two ignored sweeps. Both ignored sweeps were rerun
successfully before the latest literal/bit-vector changes: u8 division262144
cases (0.87s), recursive all n/prefixes (4.79s). The unchanged Python seed passed.
Latest Clippy found a boolean assert in a new test; fixed it, rerun pending.

New control_flow.bisa fixture and adjacent hand expectations cover coherent
while (all selector/target values and superposition), qif interference,
nested controlled Rz(2*pi), all128 basis tuples for a two-cell quantum-memory
exchange, local image bytes de/ad, and a translation-lemma borrow action.
Three --test quantum_control tests pass. An action now reclaims its private
wire identifiers after all workspace/local factors are cleaned; the original
while round-trip test exposed the allocator-counter residue. All11 canonical
fixtures are now included in identity verification.

Two new parser regressions first accepted redundant floats / rejected quoted
service literals. All operands now follow canonical INT/FLOAT/STRING/index
spellings, with strict instruction escapes. Quoted host services decode for
binding, identity and execution. qreg/region dimensions reject leading zeros.
A third parser regression first accepted inexact JSON integer encodings; large
integers now require canonical decimal strings. A loop regression first
rejected the legal string-encoded2^60 bound; signature generation now preserves
it exactly and static cycle checking finishes immediately.

A constant regression first rejected2^128+5. Constants now reduce signed
arbitrary-length decimal literals modulo their declared width. Both engines
check all256 x values for addends5 and251 and reverse cleanly.

Active width work: src/bits.rs has arbitrary-width arithmetic, comparisons,
shifts, signed/unsigned division/remainder, extensions and exact decimal/byte
conversion. --test bitvectors passes two tests, including exhaustive widths1..6
and independent257-bit hand vectors: (2^200+3)/(2^100+5) gives quotient2^100-5,
remainder28. src/wide_oracle.rs exposes OracleVectorInput/Outcome and
Object::run_oracle_vectors; every lifted row uses concrete BitVector values.
--test wide_oracle passes two tests, including comparisons with the existing
narrow interpreter across recursion/memory/arithmetic/nesting/action fixtures.
Reg descriptors now accept wide bundles; lifted permutation calls have a
vector-valued adapter. IMPORTANT: arbitrary widths are not complete yet:
wire the bit-vector values into the reversible machine and quantum oracle
adapter, update legacy/CLI interfaces to avoid their u128 assumptions, broaden
fault-literal/descriptor/counter checks and persist wide fixtures. Do not mark
D3/D4 done. This is the immediate next work, followed by certificate execution
and composition and the remaining §6 audit. No user decision is required.

### Current checkpoint: deeper suite F and metadata validation

`cargo test --manifest-path reference-interpreter/Cargo.toml --offline` PASSED
71 normal tests, two ignored sweeps pending rerun. `cargo clippy --manifest-path
reference-interpreter/Cargo.toml --offline --all-targets -- -D warnings` PASSED.
`cargo fmt --manifest-path reference-interpreter/Cargo.toml` applied formatting.

Red-green: discipline now rejects H d; CX d,t; H d; free d (scratch remains
entangled), and accepts it after cancelling the two CX readers. Cleanup pins
predicate values and qif arm polarity. Local regions must be explicitly
restored; cell version advances prevent stale qunload even after inverse
stores. Calls transfer mut-region versions and permit local region actuals;
two oracle calls cancel because the oracle is its own adjoint. Coherent while
rejects writes to the selector's dependency set, including changes undone
inside the body. Hand checks: H-CX-H produces entangled scratch; X under both
qif arms is one unconditional X; two XOR stores restore a local image but
still advance its cell versions. Twelve discipline tests pass.

Red-green: five new metadata tests first accepted malformed documents. Added
src/metadata.rs and src/certificate.rs. Schedules now require expansion binding,
real closure binding, typed budgets/selectors, valid invocation paths and loop
iteration depths, no ambiguous equal-specificity matches. Coverage resolves
program-point tuples. Certificate parsing checks versions/bindings, all claim
kind schemas, domain widths/ranges, samplers, subclaim/index references,
subjects, frame c-only expressions/live-range disjointness, errors and
composition shape. Hand checks: 256 cannot be the high endpoint of wire[8];
all-zero xoshiro state is invalid; instruction ordinal3 is outside inc8.

These are schema and static checks, NOT a completed certificate proof engine.
Re-executing exhaustive/sampled claims, interval numerical/channel checks,
error composition semantics and certificate linking are still unfinished.
Frame JSON field choices and tuple encodings must be recorded in
SPEC-QUESTIONS.md. Next: persist verification fixtures, extend certificate
checking/composition and continue the remaining suite-F audit.

### Current continuation: suite F, borrow frames, full-domain reversal

The current baseline passes 61 normal tests (two ignored sweeps). Commands:
`cargo test --manifest-path reference-interpreter/Cargo.toml --offline` passed
2026-10-29 after the loop fix. Latest strict Clippy found one collapsible if;
fixed it, final rerun still required. Beads b-isa-spec-zq2 remains in_progress.

New red-green milestones since the quantum-handle entry below:

- `--test discipline`: inherited diagonal targets, effective-control release,
  angle/interface types, matched nested oracle calls, lproc fault width, and
  H/H and rotation/adjoint cleanup. Each regression failed before its fix.
- `--test step_machine full_small_domain`: missing explicit commit API failed;
  now enumerates the full 14-bit tiny_memory interface domain for B=0..9,
  including all memory bytes and target/fault entries (163,840 commits).
  Hand check: x=0 reaches DIV0 at row4, x=1 returns db's low bit at row8;
  smaller B has fault1, B=9 includes an idle step. Every prefix reverses.
- `--test borrow`: translation lemma checks its four required negatives,
  exhaustive small-frame factorization, a borrowed Bell half and its external
  reference, designated idle CC qregs, and lifted action borrowed workspace.
  Missing CC idle binding / lifted pool first failed, now all five tests pass.
  Hand check: V=CX x,b; W=CX b,t repeated as V,W,V,W gives t XOR x, independent
  of b, and restores b. Certificate/library borrow claims remain unfinished.
- `--test quantum wide_structured`: wire[256] was rejected as an iN width;
  now basis execution copies bit255 correctly and adjoint restores it. Regs
  and scalar oracle containers remain limited to128; larger descriptors need
  a complete audit and implementation, not a claim of full-width support.
- `--test discipline loop_cleanup`: first failed by applying X once; now
  exact repeated transfer plus cycle skipping accepts X^(2^40), nested clean
  allocation reuse and zero-trip loops, rejecting odd dirty loops. Bounds
  are collected lexically, including bodies with zero iterations.

Next active work: complete §6 dependency/provenance checks (entangling reads,
controls and region-cell versions), then deep metadata and certificate schemas.
All seven deliverables remain partial pending the full obligations. No user
choice is needed. Historical entries below describe earlier checkpoints.

### Continuation: imports and linking (current)

Tracking: Beads issue b-isa-spec-zq2, already in_progress. Read beads skill and
ran bd prime. Claim was rejected because the existing job issue is already in
progress; updated its notes with the status guard instead of changing ownership.
The new AGENTS instruction authorizes Beads tracking metadata; implementation
edits remain confined to reference-interpreter/. No git or remote sync ran.

Red: `cargo test --manifest-path reference-interpreter/Cargo.toml --offline
--test linking` failed because Object::link did not exist. Implemented imported
declaration schemas, import fingerprint validation, explicit unresolved-call
execution errors, resolution by signature, pins, equal-definition dedup,
data/layout/device conflicts and regenerated canonical linked symbol hashes.
Green: four linking tests pass; full cargo test passes 41 normal tests with no
warnings (two ignored sweeps unchanged). Hand count: call + const + add + ret +
retval = 5; input 255 produces zero through 8-bit wrap, now tested after linking.

Files: src/linking.rs; Function::from_signature and Object::declaration;
tests/linking.rs plus tests/support/mod.rs. Remaining linking limitation:
different schedule/certificate metadata is still treated as conflicting;
certificate composition and stronger metadata validation are unfinished.
Next active work remains deliverable 1: inherited-control and nested-uncall
validation, then Q handle integration and remaining formal verification.

### Continuation: CC quantum handles and region history

Red: `cargo test --manifest-path reference-interpreter/Cargo.toml --offline
--test registers` failed on missing QuantumRegisters and measurement APIs.
Implemented live/viewed/consumed handles, slices and concat, clean preparation
and free, quantum action/oracle history frames, Born branch selection, and
measurement reversal/uncall exclusions. Handle reversal stores affected handle
records and released-wire positions; region reversal applies the actual inverse
operation once, not a saved quantum state. Trial execution only makes a failing
region invocation atomic; trial states are not retained as inverse history.

Red view-lifetime test initially accepted use of a viewed parent. Added CFG
validation of view-only-one-region-call and parent restrictions. Green: all 5
register tests pass. Full cargo test passes 46 normal tests; strict Clippy with
--all-targets -- -D warnings passes. New registers.bisa fixture has expected
results in registers.comments and validated identities.

Hand checks: for inc8(7), existing target 31 becomes 31 XOR 8 = 23, existing
fault 3 stays 3; at B=2 target stays 31 and fault becomes 3 XOR 1 = 2. Both calls
reverse to their complete entry tuples. Bell draws below/above 1/2 select 00/11.
Numerical Q state comparisons use 1e-12; reversible basis tests remain exact.
Beads issue b-isa-spec-zq2 remains in_progress; no deliverable is claimed done.

Read CLAUDE.md, HANDOFF.md, spec/README.md, the complete 1,675-line specification,
the complete Python seed, the workspace handoff, and the budget design note.
Rust 1.99.1 / Cargo 1.99.1 are available. No dependencies are permitted.
The existing crate contains only a version function and a skeleton test.

| Deliverable | Status | Verification |
| --- | --- | --- |
| 1. Canonical parser and well-formedness | Partial: lexical/structural parser, types, lifted row schemas/scope, signatures, layout validation | 4 integration tests green |
| 2. SHA-256, envelopes, SCC identity | Core implemented, including mutual SCCs and CC/R closures; import resolution/pins/conflicts implemented; certificate composition unfinished | 7 identity + 5 linking tests green |
| 3. Sequential reference interpreter, every lifted row | Every row has an implementation, including lift.perm_call; broader adversarial verification and unbounded bundle widths remain | 5 oracle tests plus Q lifted-call test green; full division sweep passed |
| 4. Literal reversible step machine and exhaustive comparisons | Slot/exchange machine, C1–C5/R1–R5 and inverses, all rows implemented; full obligations not yet met | 4 normal tests and ignored recursive sweep passed |
| 5. CC/R interpreter, tapes, history, uncall | Classical rows, dynamic SSA, memory, four device models, tapes, host request/reply, segment/whole reversal; qreg/Q integration implemented; broader interprocedural validation pending | 4 classical + 5 register tests green |
| 6. Q simulator and static cleanup discipline | Basis/vector simulation and instruction paths implemented; small/structural borrow checks implemented; certificate proofs and complete §6 remain | 6 Q + 5 borrow tests green |
| 7. CLI and documentation | Required commands plus hash implemented/documented; check depends on incomplete validator | 2 CLI tests green |

### Milestone: canonical core and identity

Red: `cargo test --manifest-path reference-interpreter/Cargo.toml --offline`
failed with unresolved `json`/`Object` APIs before implementation.
Red: the same command with `--test identity` failed with missing SHA-256 and
identity APIs before implementation. Green: the complete command now passes all
8 integration tests in approximately 0.2 seconds, no compiler warnings.

Hand checks: inc8's envelope consists of 3 prefix lines and the 6 canonical
function lines, each LF-terminated; its resulting hash matches the printed
8ce5e64f… value. sum's recursive self-reference is @$0, so the test independently
derives SHA256(SHA256(envelope)-hex + ':0'). R_sum = 3 + (3+1+9) = 16 cells.

Files: src/json.rs, object.rs, identity.rs, sha256.rs; tests/parser.rs,
identity.rs; tests/fixtures/{inc8,sum_db}.bisa with adjacent .comments files.

### Milestone: broader validation and sequential oracle engine

Red: `cargo test --manifest-path reference-interpreter/Cargo.toml --offline
--test structured` ran 4 tests and failed all 4 against accepted malformed
actions/metadata. Implemented action/classical operand validation, predicates,
loop bounds, input-role checks, initial cleanup-window tracking, region-call
schemas, acyclic region graphs and metadata field validation. All 4 now pass.
This is still partial suite-F coverage: borrow proofs, full provenance/window
analysis, handle lifetimes, linking and deep certificate/schedule schemas need
more work. The initial no-op validation hooks have been replaced.

Red: `cargo test --manifest-path reference-interpreter/Cargo.toml --offline
--test oracle` failed before the execution APIs existed. Implemented independent
dynamic activation environments, monotonic stack accounting, allocator and
little-endian memory, all binops/divisions/comparisons/casts, simultaneous
parmov, label/range control, faults and nested oracle call/uncall. These are in
src/oracle.rs. lift.perm_call explicitly errors pending the Q engine.

Green: full cargo test passed 16 tests before nested-call additions;
`--test oracle nested_oracles` also passes. Python seed's unchanged
`python3 reference-interpreter/isa_check.py` passes. New canonical arithmetic,
heap and nested fixtures have hand-derived adjacent comments and hashes
independently generated using the unchanged seed's envelope implementation.

Hand checks: recursive sum_db(3), db=1..16, yields 6 in 52 rows (4 oracle rows +
3*14 recursive rows + 6 base rows); stack demand 64 cells. Signed -7/2 truncates
to -3, encoded 253; remainder -1, encoded 255. Heap 0x1234 stores bytes 34 12.
Fixed a test input typo: jump's expected 9 requires y=9, not y=0.

Next: literal reversible step machine and its one-step call/ret inverse tests;
run the ignored 262,144-case u8 division sweep; broader suite F and identity
tests; CC/R and Q engines; CLI and docs. None of the deliverables is yet marked
complete. Known engine gap: closure order currently sorts individual member
hashes, so mutual-lproc SCC member ordering must be corrected before D3/D4 done.

### Milestone: reversible basis machine

Red: `cargo test --manifest-path reference-interpreter/Cargo.toml --offline
--test step_machine` failed because OracleMachine did not exist. Implemented
src/machine.rs with assignment, capture and delivery slots, flags, saved
successors, PC XOR/exchanges, monotonic stack records and literal ordered
call/ret inverses. Inverse execution does not restore saved state snapshots.
Snapshots exist only as an external assertion surface for tests.

Green: the same command passes 4 tests (one ignored), covering all inc8 inputs
and B=0..5, arithmetic/branch/parmov/fault prefixes, heap/table restoration,
and immediate forward/inverse checks on every row of recursive sum_db(2).
The complete mutable tuple, including all frames, is compared on every check.
All 11 literal fault codes are exercised; code 10 here is an explicit fault
row, not a claim that R entropy is implemented. Nested oracles are recursively
executed as reversible machines, independently of the sequential engine.

`cargo test --manifest-path reference-interpreter/Cargo.toml --offline --test
oracle -- --ignored` PASSED: 262,144 unsigned/signed division and remainder
cases, 2.07 seconds. Expected values use host i16 arithmetic rather than the
engine's sign extension implementation.

Hand check: inc8(8), B=4, has pc sequence 0,1,2,HALT,HALT; registers become
one=1,r=9 and result_val=9. The idle frame retains old_pc=HALT. Inverse PC
exchanges restore 2 before undoing retval, then restore r=0 and one=0; all
four old_pc fields and slots clear. The commit result 9 remains external.

Next: run ignored recursive-prefix sweep; correct mutual-SCC closure ordering;
add linking and matched-nested-call validation; implement Q/action engine and
lift.perm_call, CC/R engine, CLI/docs. Parser/schema/provenance checks still
need adversarial expansion before any deliverable can be called complete.

### Milestone: CC/R history and device models

Ignored recursive sweep PASSED: `cargo test --manifest-path
reference-interpreter/Cargo.toml --offline --test step_machine -- --ignored`,
all 8-bit n and every prefix through completion/first memory fault, 12.58 sec.

Red: `cargo test --manifest-path reference-interpreter/Cargo.toml --offline
--test classical` failed with missing execution APIs. Green: 3 tests now pass
for dynamic SSA loops, pure copies surviving uncall, nested cancelled segments,
memory/allocator reversal, all four device models including absent-block
presence restoration, terminal fault reversal and entropy atomicity.

src/classical.rs records per-instruction deltas and activation controls,
not complete memory snapshots. New classical.bisa fixture has hand-derived
comments. Its symbol hashes remain placeholders (the seed cannot hash CC/R
closures correctly); regenerate/check independently when the identity/link
milestone is finished. No test currently claims its placeholder hashes pass.

Hand check: nested f calls g (clock 0->1), uncalls g (clock 1->0), then returns;
uncall f skips g's reversed frames and marker, keeping clock 0. effects emits
five tape entries (entropy, console, clock, block write, block read), returns
the clock's initial zero, and reverses to an absent sector map and empty tape.

Next: Q basis/state-vector kernels and action execution, then CC hostcall and
qreg/measurement integration; complete suite F, SCC/linking, CLI/docs. All
seven statuses remain partial/pending until exhaustive obligations are met.

### Milestone: Q execution and CLI

Red: `cargo test --manifest-path reference-interpreter/Cargo.toml --offline
--test quantum` failed before QuantumState/execute_action existed. Implemented
src/quantum.rs and action.rs: complex amplitudes, allocation, controlled gates,
copy/XOR/exchange, quantum memory and local images, predicate stacks, qif,
bounded loops, coherent while flags, region calls/adjoints, qinplace expansion,
borrow-pool binding and measurement branches. Some paths lack required tests,
particularly borrow validity, qinplace certificates, nested controlled actions
and coherent while loops. Do not interpret implemented paths as verified.

Green: 5 Q tests pass for Bell/Born outcomes, every primitive gate/inverse,
controlled global phase, >64-wire permutation fast path, clean release,
action adjoints, copy/memory cleanup, loops, and lifted permutation calls.
The lifted-call test first failed on the explicit missing-engine error; both
sequential and reversible engines now execute it, with the adjoint on reversal.

Red: `cargo test --manifest-path reference-interpreter/Cargo.toml --offline
--test cli` failed before a binary existed. Green: check, hash, run-oracle and
step-check now have passing process tests and README documentation. Full
input/memory enumeration is capped (default 16 bits, configurable to 24) and
never silently sampled; fixed regions are reported. Target/fault entry values
are handled via their XOR-delivery law, not enumerated in this command.

Hand check: Bell from |00> has amplitudes 1/sqrt(2) at indices 0 and 3 and
Born probabilities 1/2 each. Controlled Rz(2*pi) contributes -1 on the enabled
branch; H-control interference therefore yields control=1, explicitly tested.

### Milestone: host requests, mutual SCCs, and fixture identities

Host request test first failed with missing request/reply APIs. It now passes:
request payload a5 00 00, reply 11 22 33 44 55, three-byte write 11 22 33,
rlen=5, and complete prefix reversal. Null request pointers fault before any
request. The transcript is retained. Reply-length overflow reading is question 8.

Mutual SCC test first returned label 2 instead of hand-derived 4. Fixed canonical
lproc order to keep SCC members in name/index order, with scc_hash group keys.
The test independently derives both member hashes from concatenated envelopes.
Both interpreters share Program's canonical layout; the hand-derived check is
therefore essential and independent of their agreement.

Unknown effects initially could execute fault 8 but could not hash. Regression
failed with missing model entry; fixed per §17 (question 6). Independently
assembled g's complete device/layout envelope in Python, checked SHA-256
e8a3dc5429049b16559aa6a4262779e3dcd37a758967def4e08c371a5b765fff against Rust.
Regenerated classical fixture hashes including new host procedures. A new test
checks hashes of all eight fixture objects; no placeholder hashes remain.

The empty-HEX regression first failed the global trailing-space rule. Resolved
the grammar contradiction as question 9, accepting NAME SP LF only inside data
for zero-byte images. Leading-zero type widths are now rejected.

## Resume here: outstanding work, in priority order

No user decision is required. Do not mark the job complete; no deliverable has
its full verification obligation met yet. The entire ISA remains the scope.

1. Complete §6 provenance/window analysis. Loops now apply exact repeated
   transfer, but the version-log checker still lacks complete transitive
   dependencies, entangling-read restrictions, predicate-version pins and
   precise region-cell tracking. Matched oracle_uncall rejects all intervening
   stores/calls conservatively; implement precise transitive memory effects.
2. Deep schedule/certificate/coverage schemas and bindings, certificate claim
   verification/composition, frame live ranges and borrowed library claims.
   Imports resolve by signature and pin, including data; linking is tested.
3. Generalize lifted reg widths and oracle bitvectors beyond128. Structured
   wire bundles and Q basis execution support256 in tests; descriptors remain
   u16. Audit JSON/JCS number corner cases and all integer spellings.
4. Complete Q verification: qinplace certificates/full domain, qif, coherent
   while dependencies, nested controlled calls, local-image and cell cleanup.
   CC qreg handles/measurement/region inverse history are now implemented;
   audit cross-call aliasing and run eligibility. Idle pools now reach lifted
   actions and designated CC registers.
5. Expand complete-D machine verification across call/ret special cases and
   every first-fault boundary. tiny_memory now covers all interface memory,
   target/fault entries and explicit commit; prior suites cover every opcode.
   Audit concrete counter widths/generalized frame sizing.
6. Audit remaining §8–§13 interpreter-facing schemas/contracts. Current shallow
   checks are not full target evaluation, schedules or certificate verification.
7. Persist new discipline/borrow objects as canonical fixtures with hand
   expectations; cover every path/fault. Rerun ignored sweeps, strict Clippy,
   fmt, unchanged Python seed and documented CLI after substantive changes.

Current source files: lib.rs, json.rs, object.rs, validation.rs, sha256.rs,
identity.rs, oracle.rs (independent dynamic activations), machine.rs (literal
slots and inverse steps), classical.rs (delta history), quantum.rs, action.rs,
main.rs. Tests are integration tests in tests/*.rs. No dependencies were added.
No git commands were run; spec/ and isa_check.py were not edited.

### Latest regression and verification

Added a failing narrow-label test: an independent jump oracle has ten rows and
legally uses a 4-bit label location, despite many unrelated functions in the
same object. It previously failed width validation. Fixed validation to count
each reachable oracle/lproc closure (the largest applicable closure for a
shared lproc), rather than all object functions. The test now passes and
independently returns 42 on the zero branch.

Before this last regression: complete `cargo test` passed 36 normal tests, no
warnings; both ignored tests passed again (u8 divisions 2.08 sec, recursive
prefixes 8.78 sec). `cargo clippy --manifest-path
reference-interpreter/Cargo.toml --offline --all-targets -- -D warnings` and
`cargo fmt --manifest-path reference-interpreter/Cargo.toml -- --check` passed.
The unchanged Python seed passed. Final rerun results follow below.

Final rerun after the label fix: `cargo test --manifest-path
reference-interpreter/Cargo.toml --offline` PASSED, 37 normal integration tests,
2 deliberately ignored sweeps, no warnings. `cargo test --manifest-path
reference-interpreter/Cargo.toml --offline -- --ignored` PASSED both sweeps:
262,144 u8 division cases in 1.45 sec; all recursive n/prefix cases in 6.76 sec.
Clippy with `--offline --all-targets -- -D warnings` PASSED; fmt `-- --check`
PASSED; unchanged Python seed PASSED. Direct documented CLI invocation of
sum_db(3), db=1..16, B=1000/heap=4080/stack=512/depth=64 printed exactly
`{"code":0,"payload":6,"steps":52}`, matching the hand derivation.

This session is a verified intermediate implementation, not JOB-COMPLETE and
not blocked on a user decision. Continue from the outstanding-work list above.

Open specification readings are recorded in SPEC-QUESTIONS.md. No decision from
the maintainer is currently required. In particular the printed 0.8 HALT idle
step is authoritative; the design note's proposed done-invariance is not 0.8.

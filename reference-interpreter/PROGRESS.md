# Bennett ISA reference interpreter progress

Resume from this file. Scope is the entire accepted ISA 0.8 (BRIEF.md); the
current CLI brief is `docs/briefs/2026-10-29-isa-ref-cli-xam.md`. Write only
under `reference-interpreter/`; commit own paths
(`git commit -- <explicit owned paths>`),
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

## Round-13 fixes (2026-10-31)

Sole implementation sub-agent; interpreter and external round-13 response only.
Baseline: **365 passed / 0 failed / 5 ignored**, summed over all binaries.

| Blocker | RED tests | Status | Commit |
|---|---|---|---|
| SOL-R13-B01 | metadata::r13_service_versions_require_strings; cli_xam::r13_cli_rejects_nonstrings_in_service_versions | GREEN | 3242123 |
| SOL-R13-B02 | r1_widths::r13_derived_counter_width_overflow_reports_host_limit; cli_xam::r13_cli_counter_width_limit_is_a_diagnostic | GREEN | 3242123 |
| R13-B01 | parser::r13_block_image_schema_is_checked_without_a_layout_consumer / r13_classical_missing_block_image_never_panics; cli_xam::r13_cli_rejects_invalid_block_image | GREEN | 3242123 |

Witness RED: **0 passed / 7 failed**. Service/block schema witnesses wrongly
admitted; derive panicked and reverse exited 101. All seven targeted tests now
pass. tests/support/r13_pins.py independently hand-assembles eight literal
objects/envelopes, asserts every supplied interface/object pin and emits
r13_cases.json; no interpreter/rehash dependency or supplied-pin correction.
Block-image absent/disk controls keep the action identity and execute X|0>=|1>.
The auxiliary zero-hash R object rejects at parse; a publicly modified Object
also returns a constructor error without unwinding. Complete sectors are
validated at load. Both u64-max exponents remain admissible/checkable and
run-oracle at B=0 still returns exactly (payload,code,steps)=(0,1,0).
Design: every service/model version is schema-typed before model-specific
checks; block_image is a data NAME resolved at load for every function kind.
Derived Hc/Ac/Sc widths use checked addition and cited host-limit errors,
without an ISA exponent cap. Existing BitVector storage arithmetic is checked.
Logs: .tmp/r13-{before,witness-red-all,witness-green,witness-full}.log.
### Generated-family green step

Full normal gate: **374 passed / 0 failed / 5 ignored** (8 unit + 366
integration; 0 doctests), after the witness step's **372/0/5**. Baseline
365/0/5 took 67.14 s runner sum; witness full 48.63 s; family full 20.56 s
(test optimization 1, retaining assertions, overflow checks and all domains).
Fmt --check and strict all-target Clippy pass, 0 warnings.

metadata::r13_generated_metadata_schema_type_family: **25 positive pinned
objects, 2,268 field visits, 12,849 negative mutations**. All seven metadata
sections, all signature forms/import pins, nested globals/interface mappings,
block_image, four devices, arbitrary service strings, every schedule payload,
coverage classification and certificate claim/optional-field production are
represented. All negatives fail Object::parse with §9.1, before caller-side
hashing/execution. A new metadata_types preflight checks types before internal
identity/binding consumers too. Cross-field and reference validators remain.
The initial family RED reported 1,282 failures: mostly correct rejections
without §9.1 or with an earlier consumer diagnostic. Six were generator
misclassifications (legal scratch number expressions and channel Y arrays);
corrected the independent union table rather than narrowing admission.

r1_widths::r13_generated_derived_width_extremes_family: **507 cases**, with
**159 constructed, 286 cited layout rejections, 62 cited resource diagnostics**;
no unwinding. All layout scalar paths (including nested bases/sizes), Hc/Ac/Sc,
P/Did/W_max/W_cell and call/ret/allocation slots are exercised across inc8,
heap, recursive sum_db, tiny_memory and a 128-bit-address layout. Twelve
values include limb boundaries and u64-max neighbours through 2^64; fifteen
depth limits also reach the u128 d+2 boundary. Legal B=0 cases retain (0,1,0).
Tests use independent width formulae and existing complete-tuple controls.

All three S1 suggestions applied: the typed-field matrix, the narrow
interface_region_names_cannot_alias_layout_globals name/comment (fixture and
assertion unchanged), and README's optional-field validator/test index.
README also links static cleanup, full-frame factorisation and runtime-return
validators. No accepted-spec question or new scope restriction was needed.
All round 4–13 positives/negatives and RNG distributions remain unchanged.
Logs: .tmp/r13-{family-red,family-green,family-full,families-final,clippy}.log.
The five ignored sweeps are running; final results will be appended here.
No beads mutation, spec/conformance edit/execution, dependency or push.

## Earlier rounds (2–11) and the codex-job history

Removed from this file on 2026-10-31 (process review: the sections restated `council/response-round-N.md`
and the git log). Read them with `git show b76c6a9:reference-interpreter/PROGRESS.md` or in the council
artefacts under `bennett/docs/council/isa-ref-2026-10-29/round-N/`.

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

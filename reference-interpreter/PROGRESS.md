# Bennett ISA reference interpreter progress

Resume from this file. Scope is the entire accepted ISA 0.8 (BRIEF.md); the
current CLI brief is `docs/briefs/2026-10-29-isa-ref-cli-xam.md`. Write only
under `reference-interpreter/`; commit own paths
(`git add -A -- reference-interpreter && git commit -- reference-interpreter`),
never push. Run headless with `TMPDIR=$PWD/reference-interpreter/.tmp`.

## Council round 3 (2026-10-29)

Received SOL-R3-B01..B05 on snapshot 2e61144; response-round-3.md is appendable
for the pending seats. B01 resolved with a witnessed RED then GREEN (16
r1_rules tests): nested target/fault/uncall deliveries invalidate outstanding
input readers while commuting output deliveries remain legal. Bead zq2.

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
| u8 division/remainder | 262,144 (a,b) pairs × 4 operations | 2.39 s |
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
| `tests/oracle.rs` `exhaustive_unsigned_and_signed_division_u8` | 262,144 (a, b) × 4 ops | 1.64 s |
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
2. Extend the §6 differential fuzzer to region calls (child actions with mut
   regions, oracle calls) and borrowed systems; extend the borrow-frame
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

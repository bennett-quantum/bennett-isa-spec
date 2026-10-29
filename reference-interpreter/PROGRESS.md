# Bennett ISA reference interpreter progress

Resume from this file. Scope remains the entire accepted ISA 0.8. No git commands;
write only under reference-interpreter/. The user limits take precedence over the
repository-wide worklog rule, so this file also serves as this job's worklog.

## Session 2026-10-29

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

# Bennett ISA reference interpreter progress

Resume here. The accepted ISA 0.8 remains supported; staged deliverables
are not complete. The explicit ISA 0.9 migration is in progress under
b-isa-spec-fuk. Slices b-isa-spec-fuk.1–.6 track admission, typed faults,
bound ref expansion, K79–K81 execution, CLI/replay and final documentation.
Additional fuk.7 retains the remaining authoritative delta (K1–K4 and
nonempty lproc forwarding among it); fuk.6 depends on that audit too.
The README admission milestone and new council acceptance remain pending.
No sibling writes, spec/catalog edits or push. Gates use the headless
prefix, offline Cargo and repo-local .tmp.

## 2026-11-01 — ISA 0.9 migration, slice 2 (fuk.2)

K5 fault C is a version-gated CC/R terminator, checked as a canonical uint64.
All nonzero bits are preserved; zero maps to 4. The existing terminal-history
path retains one record, no results and no caller continuation. 0.9 faults
retain the failing control point as well as all instances, memory, device/tape
history and open activations; 0.8 keeps its original control behavior. The
shared effective-cause helper also maps explicit zero for both Q engines and
the literal inverse, only on 0.9. run-program --reverse accepts FAILED 0.9
runs through existing reversal eligibility; the 0.8 CLI diagnostic is retained.

Eight tests in tests/typed_faults.rs cover 0..256 and every uint64 bit boundary
and complement/adjacent value, both CC/R, all widths 1..128 for all four divide
operations, implicit memory/heap/effect/entropy/illegal-uncall/use-before-definition
and CC hostcall causes, repeated terminal observation, nested CC/R callers,
unchanged pre-fault instances/control/memory/tape, and complete entry reversal.
The source-counter test instruments every R instruction across a call and
exercises every cut 0..9, including zero and both returns, plus direct zero
entry before invalid memory and resumable host-fuel diagnostics. The executor
does not invent a source budget: §4.9 assigns that counter to producer code.
CLI checks run-cc, reverse, run-program and FAILED --reverse for exact codes
through UINT64_MAX and one/no-results output. Q tests independently compare
the sequential and literal engines at every cut, idle HALT and full inverse,
including the unchanged 0.8 zero-cause behavior.

Red: .tmp/migration-s2-red.log (seven failures); atomic-point red:
.tmp/migration-s2-atomic-red.log (two failures). Harness fixes retained the
R-only effect rule, used a CC wrapper calling R for prefix effects, and sized
H=12 for a 1024-byte heap. Targeted eight tests pass. Full gates pending.
New instruction families and their corresponding preconditions (dynamic
qregs, budget.mint, parametrised qinplace, forwarding) remain in fuk.7;
no complete 0.9 executor/producer or council-acceptance claim is made.

## 2026-11-01 — ISA 0.9 migration, slice 1 (fuk.1)

Explicit IsaVersion selects coherent 0.8/0.9 headers, ABI signatures,
closure envelopes, linked rendering, certificate versions, device model
versions and the implicit ref binding. Mixed/unknown versions reject;
linking never silently upgrades a member. Function::signature remains the
legacy 0.8 API; signature_for takes an explicit version. 0.9 lproc signatures
require the new empty forwarding arrays for the existing unforwarded form;
forwarded lproc execution remains part of the outstanding 0.9 delta.

Five tests in tests/versions.rs cover the published 0.9 inc8 interface hash,
ordinary execution, version-separated identities and linking, all seven header
and certificate version fields, local/import data/function ABIs, mandatory
lproc arrays, device models and the implicit schedule library version.
Red evidence: .tmp/migration-s1-{red,devices-red}.log. Version coherence is a
semantic pass after the unchanged grammar-family checks: moving it there
preserves the latter's intentionally context-free generated positive cases.
All required gates exit 0: reference 509 passed / 0 failed / 5 unchanged
ignored; conformance 113 passed / 0 failed; draft witnesses PASS; catalog
PASS (219 recipes, envelope 5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197);
neutral 16/16. Strict lib/bins Clippy and whitespace pass. Logs:
.tmp/migration-s1-{reference-final,conformance,witness,catalog,neutral}.log.
All 64 reference fixture files and the frozen 0.8/catalog bytes match
8e481ce. fuk.1 closed; no complete ISA 0.9 execution milestone claimed.

## 2026-11-01 — Pass XI: rung-3 prerequisites

b-isa-spec-221 is an already-resolved ISA 0.9 wording issue (classification c),
not evidence that ISA 0.8 implements typed R faults. README cites accepted K5,
§4.9 and §10 and names the required producer/executor/contract-pin migration.
Host fuel is not the dialect §17.6 counter. This pass leaves interpreter
admission at 0.8 and does not clear the consumer's BD-B-ISA7 blocker. Open b-isa-spec-fuk retains the actual migration prerequisite. No code
or normative spec change for this bead; existing regressions retained.

b-isa-spec-04d is classification (b), an implementation interface under
ISA 0.8 §7.3/§7.4 and dialect 0.2.10 @023f5a7 §19.2, not a new spec rule.
`--tape-out file` and `--replay-tape file` on run-cc, CC/R reverse and
run-program save/consume strict Tape[] journals. Every dev/op/args/bytes
entry is checked in the existing step transaction before publication; bad
entries/exhaustion roll back the step without advancing the cursor, and unused
entries fail at end of forward execution (or requested prefix). Entropy and
HOST_REQUEST replies retain their existing inputs. Saved journals include
subsequently uncalled effects; output JSON retains the active-tape schema.
Successful replay is byte-identical to ordinary execution, including reset.
No tape-output write occurs on replay validation failure. The snapshot's new
read-only effect_tape accessor permits transactional observation.

Red evidence: `.tmp/pass-xi-04d-red.log`, five missing-option failures (the
malformed-input diagnostic control already passed). Eight new integration
regressions in tests/tape_replay.rs cover all four models, every field/order,
short/extra/empty/prefix tapes, changed/depleted entropy, strict schema,
duplicate/missing options and I/O, preserved output on errors, nested uncall,
CC launch + real checked host replies + reset, and failed-prefix replay/reset.
One main.rs unit regression rejects the same bad effect twice with complete
snapshot/history/trace/cursor equality, then retries and reverses to entry.
Targeted tests: 8 integration + 3 binary (including 2 existing), all green.
Test-harness corrections: synthetic CC stub needed additional heap after its
host-call allocation; the rollback harness uses three single steps rather
than run(3), which correctly diagnoses fuel exhaustion on unfinished runs.
No production-semantic fallback or fixture weakening was introduced.
Required gates finished successfully (all exit 0), headless with project-local
TMPDIR and offline Cargo:
- 221 disposition: reference **491 passed / 0 failed / 5 ignored**;
  conformance **113 passed / 0 failed / 0 ignored**; both Python gates PASS.
- 04d implementation: reference **500 passed / 0 failed / 5 unchanged ignored**;
  conformance **113 passed / 0 failed / 0 ignored**; both Python gates PASS.
Catalog: 219 recipes and unchanged envelope
`5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197`.
Logs: `.tmp/pass-xi-{221,04d}-{reference,conformance,witness,catalog}.log`.
Owned rustfmt/whitespace and strict lib/bins Clippy pass; offline release build
passes. The exact rung-3 consumer executor-control fixture passes both debug
and release replay/reset and corrupted entry-1 rejection, with byte-identical
stdout; `.tmp/pass-xi-consumer-*` and `.tmp/pass-xi-release-info.json` retain
that evidence. Release binary lives in this repo's target/release; the pinned
sibling release remains unchanged and needs an explicit consumer override/update.
This is not a production musl verdict. Frozen artifacts, K1–K80 inventory and
exact-label diff byte equality pass independent audit (`.tmp/pass-xi-audit.log`).
221 disposition commit c809fc0; first-pass gate record 666cb6b; replay code/docs
commit fdf553e. 04d closed after gates; open fuk retains actual 0.9 migration.
Concurrent unrelated beads-only commit 1a7f011 is preserved. The orchestrator
retains session-worklog ownership. No push, remote sync or sibling write.

## 2026-11-01 — n8j: persistent dynamic-instance checkpoints

The growing dynamic-instance environment now uses a shared immutable AVL root
and shared linked value-stack tails (standard library only). A checkpoint of
this environment is O(1); definitions/removals copy O(log live keys) tree nodes
and no retained values. Stack equality and destruction are iterative, including
40,000-value loop histories. Snapshot/delta ordering remains by (activation,
name). The existing transaction boundary, byte-delta frames, history/segment
checkpoints and restoration by assignment are retained; no undo replay or
allocation is needed to restore an error prefix. Other control/device/quantum
metadata retain their existing costs; this is not an arbitrary-OOM guarantee.

Red-first clone-count regression: the old code copies 499,500 / 1,999,000
ClassicalValues for 1,000 / 2,000 straight-line definitions. The new code copies
0 / 0. The unchanged-boundary control denies a HostResource observer after the
2,001st definition twice, checks complete snapshot and frame-count equality,
then retries and reverses to entry. It passes before and after the fix. Two
additional tests compare persistent updates/checkpoints with a BTreeMap/Vec
model and exercise long-stack equality/destruction; all prior R23 stage tests
remain. No fixture was weakened. Red evidence: `.tmp/n8j-red.log`.

Release build: `cargo build --release --offline` (interpreter manifest).
The supplied rung-2 launch object, ht_demo_basic, budget
10,000,000/1,048,576/65,536/256, `timeout 900`:
- n=0: **2.00 s wall**, COMPLETED/CLOSED, no fault, i64 result **0**,
  **258,272 steps**, peak RSS 244,128 KiB.
- n=1000: **4.88 s wall**, COMPLETED/CLOSED, no fault, i64 result
  **54380828142828**, **436,715 steps**, peak RSS 417,296 KiB.
- n=0 fuel exhaustion at 20k/40k: **0.14/0.17 s wall**, versus the reported
  pre-fix **3.48/27.6 s** (the old full n=0 run exceeded 600 s).
These are individual /usr/bin/time runs, not stable benchmarking claims.
Raw results/times: `.tmp/n8j-{20k,40k,rung0,rung1000}.{json,err,time}`.

Reference gate: **477 passed, 0 failed, 5 unchanged ignored** (473 baseline
plus four new tests). Conformance **113 passed, 0 failed, 0 ignored**;
`draft_witnesses.py` and `ref_catalog.py --check` PASS (219 recipes, unchanged
envelope 5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197).
All four gates used the prescribed headless environment, project-local TMPDIR
and offline Cargo; accepted 0.8 and catalog bytes are unchanged. Strict Clippy on library/binaries and owned-source rustfmt
pass. Optional all-target strict Clippy remains blocked by the pre-existing
`needless_range_loop` at `tests/k77_adapter.rs:203`; that unrelated test is
unchanged. Logs: `.tmp/n8j-{reference,conformance,witness,catalog,clippy-lib}.log`.

## Round-23 fixes (2026-10-31)

Baseline **451 passed / 0 failed / 5 ignored**, **123.70 s** runner sum over
**36 result blocks**. Final validation counts/times: **461 passed / 0 failed / 5 ignored**,
**264.43 s** over **37 result blocks**: **26 unit + 435 integration**, 0 doctests.
All round 4–23 controls retained. Five ignored release sweeps: **5 passed / 0 failed**, **161.83 s** runner sum.
Fmt/check and strict offline all-target Clippy pass without warnings.

| Blocker | RED tests / observations | Status | Commit |
|---|---|---|---|
| SOL-R23-B01 ≡ CODEX-R23-B01 | Both pinned store witnesses fail full snapshot equality at control after COW denial; no frame; next opcode skipped store | GREEN | fa73f17 |

Steps now hold restorable pre-write shared owners, prepare a private candidate,
and publish state/history together. Resource-error restoration assigns the
checkpoint and discards candidate history without calling byte undo, sparse
preparation or any allocator. Existing frames keep deltas, avoiding a resident
memory image per committed frame. `uncall` restores edited history/segments;
measurement restores its draw queue/RNG. Reply uses the same boundary.
`step_once_observed` / `reply_observed` include fallible observer preparation;
CLI publishes prepared trace records and consumes reply entries only on success.
No payload duplication, ISA length cap or arbitrary-OOM theorem is introduced.

Both exact reviewer objects/envelopes, results and probes are retained byte-exact
(8 independent pins). Linux children run 8-byte and exact 1/1.5 MiB reply controls
at 128/256 MiB AS limits with MALLOC_ARENA_MAX=1 and cores disabled. After denied
store: fieldwise snapshot equality, four frames, repeated same-limit denial;
raising only the soft limit permits the same store and its fifth frame, correct
byte 0 / byte 2 and Int(64,2), completion and entry reversal. Tests also allow a
first successful store when the representation fits, with all semantics required.

**Transactionality family: 533 indexed stage cases / 70 operation scenarios /
31 stage tags**; each stage denies twice and then succeeds on retry. Counts
are preparation boundaries, including capacity checks, not allocator-call counts.

- 340 instruction cases: all 16 byte-multiple store/load widths, alloc/free,
  hostcall, all five device operations, qreg new/slice/concat/free, measurement,
  action transfer, nested call/ret/uncall and their observers.
- 27 reply/observer + 7 standalone trace-observer + 144 immutable certificate
  replay + 15 actual CLI trace/reply cases (trace/queue equality included).

This is separate from R22's retained 12-stage/53-case inventory. The rollback
extension explicitly records held owners, whether mutation began, error-prefix
preservation and the retry opcode (external round-23 response). It exposed a
further trace publication defect: the old CLI prepared deltas after a committed
step/reply. A boundary-removal RED control reproduces advanced ip/instances;
observer APIs and actual CLI stage tests cover the repair.

All three suggestions applied: the rollback inventory; 56 public oracle
ins/tgt/fault and multi-cell explicit-region coverage cases with borrowed-name
renaming invariance; and sparse COW capacity prepared once for genuinely new
addresses. Existing-address rewrite checks preserve old snapshot values,
resident count and sorted entries without reserving growth.

Python seed: 2 examples / 2 literal envelopes. R20/R21/R22 independent pins and
8 new R23 artifact pins pass. Logs: interpreter .tmp/r23-{baseline,red,codex-red,
trace-red,family,cli-family,green,final,sweeps,clippy,fmt}.log. Initial family
harness schema/control errors were corrected; only final stage runs are evidence.
Beads **b-isa-spec-98l** tracks the blocker and trace discovery. Worklog and beads
export are left for the orchestrator. No spec/conformance/worklog writes, push
or Dolt sync. Deliverables 8–11 remain staged/Unverified.

## Resume here: outstanding work, in priority order

No user decision is required. Do not mark the job complete; no deliverable has
its full verification obligation met yet. The entire ISA remains the scope.

1. Complete §6 provenance/window analysis. Loops now apply exact repeated
   transfer, but the version-log checker still lacks complete transitive
   dependencies, entangling-read restrictions, predicate-version pins and
   precise region-cell tracking. Matched oracle_uncall rejects all intervening
   stores/calls conservatively; implement precise transitive memory effects.
2. Complete semantic schedule/certificate/coverage verification/composition,
   frame live ranges and borrowed library claims (recursive admission schemas
   are centralized and tested in metadata_schema.rs).
   Imports resolve by signature and pin, including data; linking is tested.
3. Continue generalized-width and host-resource verification beyond the R23
   finite boundary family. Lifted widths are u64 descriptors; concrete vectors
   and Q basis execution already exceed128. Audit remaining JSON/JCS numeric
   corner cases and spellings without an ISA width cap.
4. Complete Q verification: qinplace certificates/full domain, qif, coherent
   while dependencies, nested controlled calls, local-image and cell cleanup.
   CC qreg handles/measurement/region inverse history are now implemented;
   audit cross-call aliasing and run eligibility. Idle pools now reach lifted
   actions and designated CC registers.
5. Expand complete-D machine verification across call/ret special cases and
   every first-fault boundary. tiny_memory now covers all interface memory,
   target/fault entries and explicit commit; prior suites cover every opcode.
   Audit concrete counter widths/generalized frame sizing.
6. Audit remaining §8–§13 interpreter-facing schemas/contracts. Admission schemas
   do not implement target evaluation, schedule execution or interval verification.
7. Persist new discipline/borrow objects as canonical fixtures with hand
   expectations; cover every path/fault. Rerun ignored sweeps, strict Clippy,
   fmt, unchanged Python seed and documented CLI after substantive changes.

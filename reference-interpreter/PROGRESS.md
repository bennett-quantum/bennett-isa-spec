# Bennett ISA reference interpreter progress

Resume here. The whole accepted ISA 0.8 remains in scope; staged deliverables
are not complete. Sole implementation sub-agent; owned interpreter and external
round-23 response only. Never push. Cargo gates use the headless prefix and
TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp.

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

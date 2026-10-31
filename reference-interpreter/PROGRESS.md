# Bennett ISA reference interpreter progress

Resume here. The whole accepted ISA 0.8 remains in scope; staged deliverables
are not complete. Sole implementation sub-agent; owned interpreter and external
round-22 response only. Never push. Every cargo invocation uses the headless
prefix and TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp.

## Round-22 fixes (2026-10-31)

Baseline **436 passed / 0 failed / 5 ignored**, **112.97 s** runner sum across
**34** result blocks. Final **451 passed / 0 failed / 5 ignored**, **137.93 s**
across **36** blocks: **20 unit + 431 integration**, 0 doctests. All round 4–22
controls retained. All five ignored release sweeps passed: **84.94 s**.
Fmt/check and strict offline all-target Clippy pass with no warnings.

| Blocker | RED tests / observations | Status | Commit |
|---|---|---|---|
| CODEX-R22-B01 | r22_retention::r22_b01_request_retention_and_observers: child SIGABRT at 64 MiB request clone after first reservation | GREEN | 73eb62049e7624541b58730c59c417e061f112e0 |
| SOL-R22-B01 | r22_coverage::sol_r22_b01_false_proof_and_controls: Proof instead of Evidence(4) | GREEN | 73eb62049e7624541b58730c59c417e061f112e0 |

Host requests now share immutable descriptors and payloads. `Retained<T>` moves
reserved vectors into Arc<Vec<T>>, never via Arc<[T]>'s payload conversion.
History capacity reserves before stepping; pending/history/outcome/snapshot and
transcript share request/reply/undo/result/console/tape payloads. Sparse memory
snapshots share sorted resident entries. Reply prepares retained bytes, undo,
sparse writes and result retention before publication. A denied request/console
instruction or reply preserves the complete prefix and reverses exactly.
Public observer storage and CLI hex, UTF-8, trace and JSON output are fallible.
No ISA length cap. Fixed-size Arc owners/bookkeeping and unrelated control/Q
metadata are not an arbitrary-OOM recovery claim.

The audit exposed and reproduced another SIGABRT: legacy JSON serialization
cloned a feasible 64 MiB string infallibly. Executed payload output now uses
single-buffer Json::try_canonical. Existing canonical metadata/identity bytes
remain unchanged. ErrorKind::HostResource and category-preserving context
wrappers implement the previously deferred diagnostic suggestion; Display text
and ISA fault codes remain distinct. Public constructor/observer/value API
migrations are documented in README.

**Retention/later copies: 12 tagged stages / 53 stage cases**, separately from
R21's retained **24 paths / 186 cases** and R20's **102 paths / 1,113 cases**;
these overlap and are not additive path counts. Lengths 0/1/8/1 MiB/64 MiB/2^63;
8 MiB reply storage then denied undo storage; successful 256 KiB nonzero reply.
Every new resource family runs in a fresh Linux child with RLIMIT_AS=128 MiB,
RLIMIT_CORE=0 and MALLOC_ARENA_MAX=1. Reached-stage checks distinguish first
reservation, shared retention, observer copies and denied later storage. The
exact 64 MiB CLI now suspends normally at HOST_REQUEST and exits 1 when no reply
was supplied, with empty stdout. This is valid because the shared request fits.
Full path/change/case inventory is in external round-22 response.md.

Exhaustive coverage uses resolved Bundle(group,slot), complete region-cell
coverage (all or a complete explicit full-range list), and borrowed ordinals.
Borrowed names never discharge interface obligations; cross-role spellings
remain legal. Program and table spoofs yield Evidence(4)/Evidence(2); the real
full-domain CNOT comparison still fails at element 4. A **32-case public** family
asserts exact-label/count invariance under borrowed renaming; **256 direct**
coverage cases include region collisions and complete/partial explicit lists.
See INV-6 1.3.4, 1.3.5 and 1.5.5; its other spec readings remain open.

Unchanged Python seed: 2 examples / 2 envelopes. Independent R20/R21 pins and
R22 pins (12 byte-exact reviewer artifacts) pass. Both new aborts were observed
before repair; logs: interpreter .tmp/r22-{baseline,red,sol-red,json-red,family,
final,sweeps,clippy,fmt}.log. An initial full-run COW probe rejected too early
because of an unrelated glibc thread arena; allocator-arena control fixed the
harness, without accepting early rejection as later-stage coverage.

Beads **b-isa-spec-949** and **b-isa-spec-7ld** record the defects. No spec,
conformance or worklog edits; no push or Dolt sync. Beads export/worklog are left
for the orchestrator. Deliverables 8–11 remain staged/Unverified.

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
3. Continue generalized-width and host-resource verification beyond the R22
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

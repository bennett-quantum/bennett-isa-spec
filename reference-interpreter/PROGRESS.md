# Bennett ISA reference interpreter progress

Resume here. The whole accepted ISA 0.8 remains in scope; staged deliverables
are not complete. Sole implementation sub-agent; owned interpreter and external
round-21 response only. Never push. Every cargo invocation uses the headless
prefix and TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp.

## Round-21 fixes (2026-10-31)

Baseline **425 passed / 0 failed / 5 ignored**, **105.24 s** runner sum across
**33** result blocks. Final **436 passed / 0 failed / 5 ignored**, **139.44 s**
across **34** blocks: **18 unit + 418 integration**, 0 doctests. All round 4–21
controls retained. Focused R21: **11 passed / 0 failed**, **0.49 s** runner sum.

| Blocker | RED tests | Status | Commit |
|---|---|---|---|
| CODEX-R21-B01 ≡ SOL-R21-B01 | r21_resources::r21_b01_claims_public_consumers: allocator SIGABRT child | GREEN | f931307 |
| CODEX-R21-B02 | r21_resources::r21_b02_hostcall_public_consumer / r21_b02_console_public_consumer: allocator SIGABRT; r21_b02_cli_pinned_transfers: exact CLI capacity panic / exit 101 | GREEN | f931307 |

`claims::Object::domain` now checked-adds and fallibly reserves all-cells
components before expanding 2^A. Replay still materializes complete region
states, so this uses the allowed checked-eager repair and returns a §12.2
host-limit Result when unavailable, including count=1 on huge regions. No ISA
A/W/count cap or range relabelling. Both seats' complete witnesses and small
controls are retained byte-for-byte with independent pins. Small samples remain
Evidence(1), and sol's eight-row exhaustive identity remains Proof.

Classical memory/hostcall/console transfers reserve before reading bytes;
console growth and the tape entry reserve before mutation. Failed transfers
publish no request/output/tape entry, leave the complete prefix unchanged and
reverse exactly. Sparse alloc remains legal through the 64-bit pointer range.
Memory-range faults and entropy fault 10 keep their ISA meanings. Hostcall
errors cite §10, console errors §7.3, direct memory queries §7.1.

The wider audit reproduced another allocator abort in `oracle::region_bytes`.
Nested transfers, replay region bytes/cells, memory loads, undo/reply/entropy,
qreg slice/concat, CLI hex output and incremental trace growth now reserve
fallibly or retain sparse/lazy traversal. Bead **b-isa-spec-3va** records the
class and is closed. No spec gap or theorem assumption was introduced.

**24 tagged consumer paths / 186 generated cases**, separate from the retained
R20 **102 fixture-qualified paths / 1,113 cases** (overlapping paths, not an
additive 126-path claim). R20 checked scalar arithmetic and selected admission /
direct consumers; it missed executable all-cells domains and classical transfer
lengths. README corrects that coverage description. Full path/guard/case rows
are in external round-21 response.md.

| Family / consumers | Paths | Cases |
|---|---:|---:|
| All-cells and explicit-list domains, both public claim entry points | 2 | 32 |
| Exhaustive powers/products; samples; direct oracle replay | 4 | 30 |
| Hostcall / console / memory query / undo / entropy / cumulative console | 6 | 42 |
| Nested region transfer / memory load / symbolic table capacity / sparse allocation | 4 | 27 |
| Sparse memory digest/snapshot consumers | 1 | 12 |
| Qreg allocate/slice and dense amplitudes | 3 | 17 |
| CLI region cells / sparse output / hex expansion / lazy sample count | 4 | 26 |
| **Total** | **24** | **186** |

Region exponents include 0/1/3/62/63/64/127/128; value exponents include
1/2/3/62/63/64/127/128. Transfer lengths include 0/1/8/2^63−1/2^63/2^64−1.
Every resource family runs under catch_unwind in a fresh Linux child with
RLIMIT_AS=256 MiB and RLIMIT_CORE=0. The family distinguishes admission,
cardinality, materialization, sparse/lazy traversal and range/fault results.
Huge CLI sample counts enter lazily and stop on a downstream frame-storage
error; no completed astronomical replay or general OOM proof is claimed.
Pinned controls, exact reversals and successful concat controls are additional.
Sol S1 applied; codex's nonblocking structured Error-category migration deferred
because the public Error(pub String) API and contextual wrappers need a
coordinated change. Display/CLI semantics remain unchanged.

All **5 ignored release sweeps passed / 0 failed**, **88.31 s** runner sum:

| Sweep | Full domain | Result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 pairs × 4 = 262,144 executions | 1/0; 0.39 s |
| 17-wire borrow | 131,072 entries | 1/0; 2.12 s |
| checked borrow certificate | 262,144 elements, subject and expected program | 1/0; 12.16 s |
| §6 differential | 200,000 programs: 70,931 accepted clean / 129,069 rejected | 1/0; 71.22 s |
| recursive prefix reversal | 256 inputs × every completion/first-fault prefix | 1/0; 2.42 s |

Fmt/fmt --check and strict offline all-target Clippy pass with zero warnings.
Unchanged Python seed: 2 examples / 2 envelopes. Independent R20 pins and R21
pins (9 complete objects / 2 envelopes) pass; all 9 reviewer-file byte comparisons
pass. Production and test inputs stayed fixed throughout final tests/sweeps.
Implementation commit **f931307a002877317fb6d35813e97023f0aa2917**. Logs: interpreter
.tmp/r21-{baseline,red,transfer-consumer-red,nested-region-red,family,final,sweeps,
final-clippy}.log. Early exploratory transfer logs with invalid generated pins
are superseded by transfer-consumer-red and are not used as RED evidence.

No user decision needed. Deliverables 8–11 remain staged/Unverified; the full ISA
and open other-value theorem obligations remain. Only owned paths committed;
Beads auto-export and worklog left for the orchestrator. No push or Dolt sync.

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
3. Continue generalized-width and host-resource verification beyond the R21
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

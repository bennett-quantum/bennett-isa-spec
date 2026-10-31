# Editor response: council round 23

**SOL-R23-B01 ≡ CODEX-R23-B01 is resolved.** Both seats independently found
the same round-22 regression: failed sparse COW followed by allocation-dependent
rollback could leave `ip` advanced without a frame. Both complete pinned objects,
literal envelopes, original probes and results are retained byte-exact.

Implementation: **fa73f17b0a1a091670a2c84fec8288bcffb180ff**.
Interpreter evidence/progress commit: **c0df4cce67fc7810b3210aa0e84df30720d9913c**.

## Blocker disposition

| Blocker | RED tests / observations | Fix location | Commit |
|---|---|---|---|
| SOL-R23-B01 ≡ CODEX-R23-B01 | `r23_transaction::r23_store_cow_retry`, both exact objects at 128 MiB: HostResource with four frames but control changed from ip=4 to ip=5; snapshot equality fails | `classical.rs::{transaction,step,step_prepared,step_once_observed,reply_observed}`; `oracle.rs::Memory::store`; `retained.rs::SparseBytes::{prepare,prepare_write,set}` | fa73f17b0a1a091670a2c84fec8288bcffb180ff |

Determining clauses: §3.4 sequential small steps, §4.1 store semantics, §7.4 one
history frame for every executed instruction, and §10 retained host exchanges.
A host-resource rejection is neither an executed instruction nor an ISA fault.

## Transactional publication

The execution boundary retains the pre-step state, sharing the existing sparse,
console, tape and request/reply/result owners without duplicating payloads. It
executes a private candidate and publishes only after execution and any observer
preparation succeed. On Err, it restores those owners by assignment and drops /
truncates candidate history. **No `undo_record`, sparse preparation, byte replay
or allocation runs on resource-error restoration.** The original ErrorKind and
rule-cited error propagate unchanged. The step still advances its candidate ip
before branch/call execution as required by the existing control semantics; that
candidate cannot escape on failure.

Committed frames continue to retain byte deltas, rather than one complete
resident map per store. `undo_record` remains the forward-history inverse for
successful instructions; it is no longer the recovery procedure for failed
execution. `uncall` checkpoints existing history flags and segment records so
failure partway through its inverse also restores the entire prefix. Calls
truncate newly created segments on rejection; returns restore edited segment
results. Measurement checkpoints its supplied-draw queue and RNG. No pending,
finished, fault, register, quantum, memory, device, tape or control delta escapes
an error. History capacity changes are allocator bookkeeping, not history frames.

Replies use the same boundary around reply storage, overwritten bytes, shadow
memory, result definition and retained transcript publication. Rejected reply
preparation leaves the same request pending. The observer variants
`step_once_observed` and `reply_observed` construct fallible trace values within
the transaction. CLI trace publication and reply-queue consumption occur only
after success. Callbacks return prepared values; external callback side effects
are the caller's responsibility.

The shared-storage properties from round 22 remain: reserved vectors move into
`Arc<Vec<T>>`, retained payload clones share owners, and there is no new transfer
limit, ISA width cap, or implied proof of recovery from arbitrary process OOM.
Infallible Rust control/Q metadata clones and fixed-size owner bookkeeping are
not an allocator-abort recovery theorem. The finite family below concerns every
explicit fallible preparation reached by its instruction/consumer cases.

## Red/green witnesses and rollback inventory

Both pre-fix executions were independently observed with the copied witness,
not inferred only from source. Logs: `.tmp/r23-red.log` (SOL) and
`.tmp/r23-codex-red.log` (CODEX, reversed seat order). For the second reproduction,
HEAD's original execution/storage/CLI sources were temporarily read into the
owned worktree, the RED test was run, and all implementation edits were restored
in `finally`; no foreign checkout or reviewer source was written.

The exact SOL object is SHA-256
`e94c5cbbbdfdfeb32d4101cf6a6033172afcea1c6f6b9cc1bd1df80ad541ece5`.
Its literal envelope hashes to
`abcde78cd19169240834bca4e8c30f1a036782615a97495b329c6f7b4dd3d97f`.
The CODEX object and envelope are independently pinned alongside it by
`tests/r23-independent-pins.py`. Rust compares the exact LF envelope bytes
before checking their interface hash. Eight copied objects/envelopes/probes /
results compare byte-for-byte with the read-only reviewer sources.

| Preparation / path | Mutation begun? / shared observer | Rejection and retry check |
|---|---|---|
| Initial history reserve | No / checkpoint held | Full prefix and frame count unchanged; same opcode on retry |
| Store undo retention and value buffer | Candidate ip advanced / pre-write owner held | Original ErrorKind; no published control/history delta |
| Sparse COW/growth | Candidate ip advanced; byte writes have not begun at denied COW / observer-held resident reply plus checkpoint | Complete fieldwise snapshot equality, four frames, unchanged byte/transcript; repeated same-limit rejection; higher-limit retry is store, never ret/load |
| Device write → final tape preparation | Candidate bytes/device counter/console may already have changed / checkpoint held | Rejection restores memory, console, clock/entropy/block state and tape; retry matches successful row |
| Reply → result/observer preparation | Shadow memory and candidate result may already exist / suspended observer held | Pending request, transcript, history and complete snapshot unchanged; successful retry retains full reply and correct delivery |
| Snapshot delta / digest / trace | Candidate instruction and frame may already exist privately / before-state held | Error restores state/history and leaves external trace/reply queue unchanged; successful retry emits the correct row |
| `uncall` inverse → later observer | Candidate inverse and frame flags may already have changed / checkpoint and history journal held | Exact prefix restoration; same uncall on retry; subsequent whole-run reversal restores entry |

This **rollback-preparation inventory is separate from R22's retained 12 tagged
stages / 53 stage cases**. It explicitly records whether writes began, the held
owner that forces COW, the prefix after Err and the retry opcode; it does not
relabel the 53 earlier transfer observations as rollback coverage.

Linux subprocess controls use RLIMIT_AS=128 or 256 MiB, RLIMIT_CORE=0 and
MALLOC_ARENA_MAX=1, with no filler allocation. The exact 1 MiB and 1.5 MiB reply
must succeed before the rejected store stage. On resource error, both witnesses
check full snapshot equality, four history frames, the same Error on immediate
retry and no ISA fault. Raising only the soft limit in that same process then
executes store.i8, adds its fifth frame, writes byte 2 / byte 0, and completes
with Int(64,2) for CODEX. Entry reversal is required with the old observer still
held. Eight-byte controls and 256 MiB controls also succeed. A representation
improvement is allowed to make the first store fit, but then the correct write,
frame and result are mandatory; the injected-stage family always exercises
rejection. Independent literal sorted memory digests check the small rewrites.

## Transactionality family

**533 indexed preparation cases / 70 operation scenarios / 31 stage tags**.
Each index is rejected twice at the same prefix, then disabled for a successful
retry. Thus the family checks **1,066 injected HostResource returns** and **533
successful retries**, plus the unfailed reference execution of each scenario.
Stages count attempted fallible preparation boundaries, including capacity
checks that may reuse storage, rather than physical allocator invocations.
The test-only hooks are thread-local; production hooks are no-ops. Error-path
stage-prefix equality also proves recovery did not attempt another preparation.

| Instruction / consumer kinds × preparation stages | Operation scenarios | Indexed cases |
|---|---:|---:|
| All store.iN, N=8,16,…,128 × history, undo bytes, bit-vector words, sparse COW/growth | 16 | 64 |
| All load.iN, N=8,16,…,128 × history, load bytes, bit-vector words, retained definition | 16 | 64 |
| alloc × history/allocation table/result; free × history | 2 | 4 |
| hostcall × history/initial request transfer | 1 | 2 |
| console.write × history/tape/transfer/console/final tape; clock.read × history/tape/undo/value/sparse/final tape; entropy.read × history/tape/undo/transfer/sparse/final tape; block.read/write × history/tape/undo/sparse/final tape as applicable | 5 | 30 |
| qreg.new/slice/concat/free, action transfer, measure and ret × history/wire identities/map/basis/slice/concat/result, then snapshot delta/digest/JSON observer stages as reached | 16 | 111 |
| Nested call/const/alloc/store/ret/uncall/ret × forward and inverse preparations plus observer stages | 8 | 65 |
| Standalone snapshot delta/digest/JSON trace after a candidate constant step | 1 | 7 |
| reply × reply storage/undo/sparse/result plus snapshot delta/digest/JSON observer | 1 | 27 |
| Immutable public certificate replay × bit-vector/domain enumeration/tuple/wire allocation stages throughout replay | 1 | 144 |
| Actual CLI store trace / load trace / suspended reply trace × trace reserve, concrete delta JSON builders and reply record extension (2/2/11) | 3 | 15 |
| **Total** | **70** | **533** |

Instruction/reply tests assert complete snapshots, history counts, tape and
transcripts on failure, equality with successful semantics on retry, literal
store bytes/load values, and exact entry reversal. The measurement cases also
check deterministic draw reuse and successful quantum state; their successful
measurement histories remain intentionally irreversible. Certificate replay
checks immutable object identity and exact public verdict/reports after each
denial. The actual CLI cases additionally assert byte-for-byte-equivalent JSON
trace values and unchanged reply-queue length before successful consumption.

### Further defect exposed

The old CLI called step/reply and only then allocated snapshot deltas and trace
records. A later trace failure could therefore leave a committed instruction
or consumed reply without its record. The observer-stage family exposed that
same publication boundary. `.tmp/r23-trace-red.log` is a boundary-removal
control reproducing the old ordering: denial at snapshot register deltas leaves
const.i64's instance/ip advanced. It is explicitly a control using the new
observer test seam, not a claim that the old CLI exported that seam. Observer
preparation now participates in transaction commit. Fifteen actual CLI stage
cases test the concrete drive/trace builders, including unchanged traces and
reply queues; all pass.

## Suggestions

All three list-level suggestions are applied:

1. Public role-collision coverage adds **56 cases**: oracle ins/tgt/fault
   subsets with each omission independent, borrowed labels renamed across all
   real names, and two-cell action regions with complete/partial explicit cell
   lists. Both public verifiers agree. Exact Evidence counts remain invariant
   under renaming; Proof is possible only for complete real coverage.
2. The explicit rollback-preparation inventory above complements the existing
   R22 retention inventory without conflating their counts.
3. Sparse preparation reserves for genuinely new nonzero addresses. COW
   reserves final capacity once instead of copying at exact capacity and then
   growing geometrically for a replacement. The existing-address control holds
   the old observer, replaces three bytes, preserves resident count and sorted
   entries, and requires capacity exactly equal to the three resident entries.
   Public small controls independently check sorted memory digests as well.

README documents the post-Err step/reply contract, observer publication and the
finite injector scope. PROGRESS has only this round and the retained resume list.
No unrelated validator/index or literal oracle-call-record rubric suggestions
are claimed as implemented in this blocker repair.

## Validation and retained evidence

Baseline: **451 passed / 0 failed / 5 ignored**, **123.70 s** runner sum over
**36 result blocks**. Final: **461 passed / 0 failed / 5 ignored**, **264.43 s**
over **37 result blocks**, **26 unit + 435 integration**, 0 doctests.
All round 4–23 tests remain, including all round-22 64 MiB host/console/reply
limits, shared observers, original 2^63 denials and both borrowed-name blockers.
A final focused store rerun also checks the added independent sorted digests.

All five ignored release sweeps: **5 passed / 0 failed**, **161.83 s** runner sum.

| Sweep | Full domain | Result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 pairs × 4 = 262,144 executions | 1/0; 0.91 s |
| 17-wire borrow | 131,072 entries | 1/0; 2.81 s |
| checked borrow certificate | 262,144 elements, subject and expected program | 1/0; 20.39 s |
| §6 differential | 200,000 programs | 1/0; 134.82 s |
| recursive prefix reversal | 256 inputs × every completion/first-fault prefix | 1/0; 2.90 s |

Offline strict all-target Clippy and fmt/check pass without warnings. The
unchanged Python seed verifies two example objects and two literal envelopes;
independent R20/R21/R22 pins and all eight new R23 artifact pins pass. No
production source changed during final validation; the last test-only addition
is the independent small sorted-digest assertion, verified by the focused rerun.

Logs: interpreter `.tmp/r23-{baseline,red,codex-red,trace-red,green,family,
cli-family,final,sweeps,clippy,fmt}.log`. Initial family harness grammar/layout /
R-only-effect errors were corrected before counting stage cases; none is cited
as a transactional RED witness. The separate R23 COW REDs and trace ordering
RED remain reproducible evidence. All cargo gates use the mandated headless
prefix and project TMPDIR. One preliminary fmt/check invocation had a misspelled
DBUS unset name; it was immediately rerun with the exact required prefix, and
only the corrected final fmt/check is counted.

Beads **b-isa-spec-98l** records the blocker and trace discovery. Owned paths
only: `reference-interpreter/` and this response. Beads export and session
worklog remain for the orchestrator; no spec/conformance/worklog edits, push,
background-hook interference or Dolt sync. Deliverables 8–11 remain staged /
Unverified, with the full ISA and other-value obligations still in scope.

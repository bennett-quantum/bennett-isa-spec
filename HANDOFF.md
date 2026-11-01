# Current state — 2026-11-01 14:14:55 UTC

Pass XII implements b-isa-spec-9hv, **in_progress pending council round 9**.
K1–K80 remain accepted through round 8; K81 is the sole new MUST and §0 row.
The normative text and complete row/carrier witness are in ISA 0.9 §11,
**Source cleanup-pair partition (K81)** / **Intrinsic-boundary witness (K81)**.

Reading B for qcopy / mix / adjoint-mix / quncopy: the call pair is
entry:1/2, carrier entry:2; the intrinsic copy pair is entry:0/3,
carrier entry:3. They select independently and an entry:2 maximal tie is
observable independently of entry:3. Intrinsic cleanup owns only its direct
computing operation; maximal terminal inverse chains ending in qfree/qreturn
coalesce first. Disjoint rows do not alter pairing, joint inverse occurrences
are indivisible with first-release precedence, and no cleanup inverse has
two owners. K80's release-ended multirow witness and K64 oracle-support
partition are preserved. For a nonzero call target, recompute replays exact
source-derived entry history and restores the pre-call value; this example
computes delta = mix(x) XOR x in fresh scratch. New replay call_index 0/1
belongs to entry:2, with separate callee ordinal spaces.

This is a real pair-formation gap; 9hv is not closed as not-a-defect.
Reference src was read only: validation.rs binds quncopy to the original
Creation snapshot, requires later target changes undone, and joint_change
captures/restores the old joint versions. action.rs runs the written call
inverse before intrinsic inverse-copy/release. No semantic mismatch or src
alignment was required. Production admission remains 0.8, with no general
scheduled 0.9 source-cleanup builder claimed.

Three new regressions in reference-interpreter/tests/cleanup_partition.rs:
source forward/adjoint exact state and allocator restoration (primitive/call,
disjoint rows, basis/coherent/entangled inputs); finite-adapter carrier
partition and actual selector-engine ties; all four independent cleanup
realisations through unchanged catalog recipes and QuantumState, including
intermediate nonzero d=x, fresh call occurrences and leaf spaces. The finite
adapter explicitly supplies source partition/coordinate metadata; it is not
production source-builder verification. Existing fixtures are unchanged.
The filed indexed two-slot list is schematic under §16.2's NAME-only list
grammar; the admitted bundled mix(d) denotes the same CX. Its literal syntax
rejection is also locked. Initial failing runs were harness list/declaration
errors, not red evidence of a reference semantic discrepancy.

All four required gates exit 0:
- Reference Cargo offline: **503 passed, 0 failed, 5 unchanged ignored**
  (500 baseline plus three new regressions).
- Conformance Cargo offline: **113 passed, 0 failed, 0 ignored**.
- python3 -I conformance/tests/draft_witnesses.py: PASS.
- python3 -I conformance/tools/ref_catalog.py --check: PASS, 219 recipes,
  envelope `5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197`.

Every gate unset DISPLAY, WAYLAND_DISPLAY, DBUS_SESSION_BUS_ADDRESS and
BROWSER and used TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp.
Logs: .tmp/pass-xii-{reference,conformance,witness,catalog}.log;
targeted: .tmp/pass-xii-regression.log; initial harness evidence:
.tmp/pass-xii-before-text.log; frozen-artifact audit: .tmp/pass-xii-audit.log.
Owned rustfmt and source/prose whitespace checks pass. The generated diff
is byte-exact with labels a/spec/Bennett-ISA-Spec.md and
b/spec/Bennett-ISA-Spec-0.9.md; ordinary single-space blank diff context
lines are intentionally retained. Frozen 0.8/catalog bytes equal 8d9a5c6.
- spec/Bennett-ISA-Spec.md SHA-256: `90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e`.
- spec/Bennett-ISA-Spec-0.9.md SHA-256: `3b565b25eb34b78b9f710a894f24c084509e7e56507a8f05f15976e6c68544bc`.
- spec/ref-0.9.json SHA-256: `52dbc3f8f0c91d587fd933a43080385fd5284abcfec0ce84788229fcb15c53d8`.
- spec/isa-0.8-to-0.9.diff SHA-256: `3c4aa826b341615c36b671e1e71295b0d56c117875c77d32842861f83e1cfac5`.

Pass XI now explicitly names open **b-isa-spec-fuk** for producer/executor
support and contract-pin migration for typed R faults. Its blocker and the
full consumer rung-3 scope are unchanged; no sibling pin migration is claimed.

Own commits: e0d007b (K81/spec/three regressions/Beads); 5906f51
(reference-green gate record). Final green records, Beads note/export and
this real UTC handoff are committed next by explicit path list; final report
gives HEAD. The orchestrator retains session-worklog and council ownership.
No push, remote/Dolt sync or sibling write. Preserve pre-existing .tmp/ and
all logs; only ?? .tmp/ is expected after the final records commit.

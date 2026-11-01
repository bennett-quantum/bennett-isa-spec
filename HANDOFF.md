# Current state — 2026-11-01 11:31:34 UTC

Both requested jobs are implemented, in order, with separate commits and
complete headless gates. No push, remote/Dolt sync or sibling repository write.
The orchestrator owns the session worklog, council fix round and downstream
statement-layer regeneration. Preserve the pre-existing untracked .tmp/.
Intervening Beads/pre-push-hook commits from the orchestrator were preserved;
this implementer committed only explicitly listed owned paths.

Job 1 / b-isa-spec-n8j is CLOSED. Implementation commit: 7193103.
Dynamic instances use a shared persistent AVL root and immutable linked value
stacks. Instance checkpoints are O(1), updates copy O(log live keys) nodes,
and retained values are not copied. Snapshot/delta ordering stays by key;
long-stack equality/destruction is iterative. Existing transaction restoration
assigns the old snapshot root and truncates candidate history without undo
replay or allocation. Other control/device/quantum metadata retain their
existing costs; this is not a guarantee of recovery from arbitrary allocator
aborts. Committed frames still retain deltas.

The red-first 1k/2k-definition clone counter was 499500/1999000 before,
0/0 after. A HostResource observer denied after the 2001st definition twice
preserves the complete snapshot and frame count, then retry/reversal succeeds;
this control passed before and after. Additional tests compare persistent
updates/checkpoints with an independent BTreeMap/Vec model and check a 40k
shared stack. All existing R23 instruction/reply/observer stage cases pass.

Release build used cargo build --release --offline with the interpreter
manifest. Supplied rung-2 ht_demo_basic launch, budget
10000000,1048576,65536,256, timeout 900:
- n=0: 2.00 s wall; COMPLETED/CLOSED, no fault; result i64 0; 258272 steps;
  peak RSS 244128 KiB. Old run exceeded 600 s.
- n=1000: 4.88 s wall; COMPLETED/CLOSED, no fault; result i64
  54380828142828; 436715 steps; peak RSS 417296 KiB.
- 20k/40k fuel exhaustion: 0.14/0.17 s wall versus the reported pre-fix
  3.48/27.6 s. Individual timings, not a stable benchmarking claim.
Raw evidence: .tmp/n8j-red.log and
.tmp/n8j-{20k,40k,rung0,rung1000}.{json,err,time}.
Full design/evidence is the top block of reference-interpreter/PROGRESS.md.

Job 2 / ISA 0.9 pass IX / b-isa-spec-7js stays IN_PROGRESS FOR COUNCIL.
Reading 1 is adopted: exactly A coherent address wires. The accepted
reference and published qstore/qexchm catalog already enforce that shape;
Reading 2 would broaden addressing and add an unsupported out-of-range no-op.
There was a real A=0 source-binding gap because wire[0] is forbidden. New
§4.6 accepts the address-only () actual at A=0, bound to catalog [], selecting
cell 0 subject to the enclosing controls. Ordinary zero-width wire values,
allocations and empty d/v remain forbidden. §16.2 cross-references the form.
One new MUST and K79 only; K1–K78 are byte-identical to 7193103. Exact text,
dispositions and evidence are the top block of spec/isa-0.9.PROGRESS.md.
The status header marks K79 pending the council fix round; prior core
acceptance and pending pass VIII/K78 are preserved.

Before-change memory-address witnesses: 2 pass / 4 fail; all four A=0
positives fail with unknown/live wire (). After alignment, eight regressions
pass: all four rows; A=1/2 exact-width positive/negative controls; empty-form
exclusions; borrow compiler and one-cell local-region restoration; catalog
A=0, W=1/2/3 symbolic copy/store/exchange bindings and inverses. CLI hash/check
and all target/cell/control basis values are included. Validation, action
execution and the borrow compiler recognize () only in the coherent address
position. No fixture was weakened or removed. The reference remains 0.8-gated;
this does not claim a full 0.9 executor.

Optional clarification: §3.3/§4.8 explicitly distinguish CC/R binop iN from
icmp iN or ptr, with equal widths/types; lifted forms remain §4.2. No new MUST
or K row for this. Existing r2_validation admission/execution/reversal tests
and council response-round-2 B07/B05 establish the accepted reading.

All required gates exit 0, with DISPLAY, WAYLAND_DISPLAY,
DBUS_SESSION_BUS_ADDRESS and BROWSER unset and TMPDIR set to this repo's .tmp;
Cargo uses --offline and the requested manifest paths:
- Job 1: reference 477 passed / 0 failed / 5 unchanged ignored;
  conformance 113 passed / 0 failed / 0 ignored; both Python checks PASS.
- Pass IX: reference 485 passed / 0 failed / 5 unchanged ignored;
  conformance 113 passed / 0 failed / 0 ignored; both Python checks PASS.
Logs: .tmp/n8j-{reference,conformance,witness,catalog}.log and
.tmp/pass-ix-{reference,conformance,witness,catalog}.log.
Owned-source rustfmt and whitespace checks pass; generated diff is verified
byte-for-byte with the exact labels (ordinary blank context lines retained).
Strict Clippy on library/binaries and the new memory-address test passes.
Optional all-target strict Clippy is blocked by the pre-existing
needless_range_loop at tests/k77_adapter.rs:203, which was left unchanged.

Frozen accepted 0.8 SHA-256: 90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e.
Published catalog bytes SHA-256: 52dbc3f8f0c91d587fd933a43080385fd5284abcfec0ce84788229fcb15c53d8.
Catalog envelope: 5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197
(219 recipes, unchanged).
Final 0.9 spec SHA-256: 38a5b5c4cd04eeeb9ee930d77b3fcfa3554da0f38ba62aab827ba4a0e05fc3f1.
Exact-label diff SHA-256: 4d5f3231e6fa7ce7e40983dc342c6396be6f737a17d5abe4058f3c518749ceca.

This final pass-IX commit includes the spec, exact-label diff, regressions,
reference alignment, PROGRESS, exported Beads and this handoff. After committing,
only the preserved untracked .tmp/ should remain in git status.

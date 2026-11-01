# Current state — 2026-11-01 10:43:51 UTC

Round-7 housekeeping and pass VIII are implemented in order with separate
commits and complete headless gates. No push, remote/Dolt sync or sibling
repository write. The orchestrator owns the session worklog, council fix
round and final statement-layer regeneration / INV-3 citation refresh in
bennett-conformance. This implementer did not run a council round.

Round 7 accepted passes VI/VII (codex seat, 0 blocking). Beads
b-isa-spec-73y and b-isa-spec-5xm are closed with that acceptance;
PROGRESS and the 0.9 status/history header record K76/K77 acceptance in
fix rounds 6–7 while preserving three-seat acceptance of the core.
Housekeeping commit: 989948e. An intervening bead-only commit, 7dd1ffd,
captured the exported closures while the housekeeping gates ran; it was
preserved. Unrelated consumer issues in the shared Beads export were
preserved. No source changes from another implementer were incorporated.

Round-7 S1: §15 H states the existing K77 argument/body/template capture,
forwarding, nesting, inverse and rejection vector. The finite inc8 adapter
witness in reference-interpreter/tests/k77_adapter.rs uses the actual
catalog expander for invoke/call, body splicing and template application.
Its out-of-band source binding captures B=2/B=3 under unrelated B=99
context and restores outer bindings after nested builds. Across all 256 x,
the independent literal frame machine checks exactly 2/3 frames,
zero-seed (t,f)=(x+1 mod 256,1), retained forward indices under inverse
traversal and full state restoration. Absent, malformed, noncanonical and
source-inconsistent records reject. This is a finite adapter witness,
not a general ISA-to-catalog source builder or complete 0.9 executor.
The reference uses a local conformance-crate dev dependency for this test;
no published catalog bytes changed. Statement-layer S1 stays with the
orchestrator; no bennett-conformance write was made.

Pass VIII / b-isa-spec-9eq: Reading B adopted. The existing text was
ambiguous: §4.4 and pre-transfer resolution support primitive identity,
while “exact §5.3 adjoint” can suggest source opcode identity. The
accepted reference already lowers X-family gates to Node::Flip and
qxor/qexch to per-bit flips/swaps before EntryAnalysis causal matching.
§5.6 now explicitly requires resolved primitive operation identity,
including all own/contextual control literals and polarity. It retains
all-operand top-event matching and excludes target-scheme decomposition
and Boolean-equivalent circuit replacement as entry facts. Source opcodes,
hashes and occurrence identities are unchanged. K78 is the only new row
and MUST; K1–K77 are byte-identical to 989948e. The status header marks
K78 pending the codex-seat council fix round; 9eq remains in_progress.
Exact normative text/dispositions: top block of spec/isa-0.9.PROGRESS.md.

reference-interpreter/tests/frame_identity.rs adds three regression tests,
green against unchanged reference src before the text edit. Six positive
circuits pass CLI hash/check and forward identity on all 64 interface/
borrowed basis configurations: cx/mcx both ways, ccx/mcx control order,
contextual-X/cx, qxor/per-bit cx and qexch/per-bit swap. Different controls,
intervening readers and mixed-spelling polynomial-only prefixes reject
§5.6. No fixture edit, test removal or reference implementation fix.
Before-text evidence: .tmp/pass-viii-before-text.log.

All required gates exit 0, with DISPLAY, WAYLAND_DISPLAY,
DBUS_SESSION_BUS_ADDRESS and BROWSER unset and TMPDIR set to this repo's
.tmp. Cargo used --offline and the specified manifest paths.
- Housekeeping reference: 470 passed, 0 failed, 5 unchanged opt-in ignores.
- Pass VIII reference: 473 passed, 0 failed, 5 unchanged opt-in ignores.
- Both conformance runs: 113 passed, 0 failed, 0 ignored.
- Both runs: draft_witnesses.py PASS; ref_catalog.py --check PASS,
  219 recipes and unchanged envelope. Rustfmt and source whitespace pass.
Logs: .tmp/round7-{reference,conformance,witness,catalog}.log and
.tmp/pass-viii-{reference,conformance,witness,catalog}.log.

Accepted 0.8 is unchanged; SHA-256:
90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e.
Published catalog bytes are unchanged; SHA-256:
52dbc3f8f0c91d587fd933a43080385fd5284abcfec0ce84788229fcb15c53d8.
Catalog envelope:
5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197.
Final 0.9 spec SHA-256:
3a665d6834b94e90646e65626a222f70d7788bf2ff885b449815e2fda9096126.
Final exact-label diff SHA-256:
3a71d9229859cfa158211ddc2ed364837706fbdadd7cfa2e11ef7654ed564b03.
The reference's 0.8 admission/version gate is unchanged.

The final pass-VIII commit includes the spec, exact-label diff, regression,
PROGRESS, exported Beads and this handoff. Preserve the pre-existing
untracked .tmp/; it also contains this session's logs and drafting files.
After the final commit, only that directory should remain in git status.

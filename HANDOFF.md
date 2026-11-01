# Current state — 2026-11-01 08:53:54 UTC

ISA 0.9 editorial pass V / b-isa-spec-73y is implemented. Maintainer's
2026-11-01 ruling selects Reading B with a normative entry-state analysis.
The orchestrator runs the pending one-reviewer council fix round; no muse
round. Bead 73y stays in_progress. Earlier three-seat acceptance is the
pre-pass-V publication's history; this amendment has not yet been reviewed.

§5.6 now fixes D_F by constants plus exact all-operand causal adjoint
cancellation, freeing every interface/borrowed component and every unknown
local component independently. T3 proves restricted factorisation on
span(D_F); the checker establishes reachable entry support, and linearity
extends equality to entangled states. K76 is the only new K row; K1–K75
are byte-for-byte unchanged. The reference has a separate EntryAnalysis,
retains exact ANF proofs inside frames and rejects polynomial-only prefix
zeros. Region-D certificate replay cannot waive a definite domain failure
or stand in for a widened frame-domain proof. No new serialized claim kind
or producer-controlled domain is introduced.

Accepted 0.8 remains untouched at spec/Bennett-ISA-Spec.md; SHA-256:
90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e.
New spec/errata-0.8.md incorporates the binding 0.9 interpretation.
The reference's object version gate still admits 0.8, with this erratum;
no 0.9 object admission is claimed. New consumers use the published 0.9.
The bound catalog remains spec/ref-0.9.json, 219 recipes, envelope
5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197.

Commits owned by this implementation:
- 9df1541: §5.6 / K76 / exact-label 0.8-to-0.9 diff.
- 6182d7c: short 0.8 erratum, four affected fixtures named.
- 1a19072: reference analysis, header, SQ20 and five added regression tests.
- bennett-conformance 2095a7d39b7e4502e77a0930f06be805e704cc1c:
  only proofs/INV-3-discipline-soundness.md, adding reachable-entry support,
  linearity including R, and composition under 1.5.5, with explicitly
  versioned 0.9 statement IDs checked against a scratch statement layer.
- This final record commit: pass V progress, detailed brief, this handoff
  and Beads export (exported explicitly before committing).
Unrelated Beads-only bbb758d arrived during the session and is retained.

Final required headless gates, all exit 0:
- Reference cargo test --offline: 466 passed, 0 failed, 5 unchanged ignored
  opt-in exhaustive tests (461 baseline + five added tests).
  Log: .tmp/pass-v-reference-verified.log.
- Conformance cargo test --offline: 113 passed, 0 failed, 0 ignored.
  Log: .tmp/pass-v-conformance.log.
- python3 -I conformance/tests/draft_witnesses.py: PASS, including byte-exact
  diff labels, accepted 0.8 fingerprint, examples and published artifacts.
- python3 -I conformance/tools/ref_catalog.py --check: PASS (219 recipes).
- Neutral runner: 16/16 cases, every expected hash/check/run PASS.
  Log: .tmp/pass-v-neutral.log.
- Before/after CLI admission audit: 48/48 fixture and neutral .bisa objects
  accepted; no fixture changes, removals or losses. All four r9 fixtures
  (sol-oracle, codex-oracle, loop, sol-qinplace) still pass check.
- Polynomial-only interleaved-prefix negative: rejected with §5.6;
  interface-correlation, public certificate bypass and widened opaque-domain
  regressions pass. Exact prefix cleanup keeps the negative clear of §6.2.
- Owned-path whitespace checks pass; K1–K75 continuity/bytes verified.
All gates unset DISPLAY, WAYLAND_DISPLAY, DBUS_SESSION_BUS_ADDRESS and
BROWSER, with TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp.

Final 0.9 SHA-256:
7b77c96e08a1f7e40d8a70155bef703bb4e7d2e9c850a95e3579a7ee7ae288ce.
Exact-label diff SHA-256:
4f7f506018bcd46769d2711eab2e3847391d58ed575247050608c65876d4335a.
The brief docs/briefs/2026-11-01-isa-09-pass-v-73y.md contains the Lamport
argument, dispositions, gotchas and a verbatim copy of the amended §5.6
paragraph and normative rules. spec/isa-0.9.PROGRESS.md has the pass V block.

No push, remote/Dolt sync, VM write or other sibling write. The orchestrator
owns the session worklog and council round. Preserve pre-existing untracked
.tmp/ here (also contains this pass's evidence logs), and sibling untracked
clauses/DLT-0.2.11-draft.json and suite/tests/__pycache__/.

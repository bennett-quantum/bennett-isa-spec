# Current state — 2026-11-01 09:45:07 UTC

ISA 0.9 passes VI and VII are implemented in order, with separate commits,
PROGRESS blocks, Beads notes/exports and complete headless gates. Both
b-isa-spec-73y and b-isa-spec-5xm stay in_progress for the orchestrator's
one-reviewer council round 7. No council round was run by this implementer.
The orchestrator also owns the session worklog and final 0.9 statement-layer
regeneration / repaired INV-3 citation resolution in bennett-conformance.
No sibling repository was written in this session.

Pass VI / round 6 (codex seat, astra): R6-CODEX-1 adopted exactly in
§5.6 rule 1, counting own and contextual controls; disabled operations
preserve values and record no causal event. The full CCX witness was
accepted by the unchanged reference before the text edit and is now a
suite F/frame_domain regression lock: c=0 at entry admits the single
CX(c,b) frame; replacing both CCXs by X gives c=1 with valid cleanup and
rejects §5.6. Polynomial-prefix, interface-correlation and certificate
negatives still pass. All three S1s adopted: exact finite-transfer domain
versus coarser invariants (§5.6 rule 3), frame versus region claim domain
and illustration (§12.2), statement-layer regeneration assigned to the
orchestrator. K1–K76 unchanged; no new row in this pass.

Pass VII / 5xm: Reading B, matching the source header and reference's
separate budget argument. §11.1 defines explicit source-binding channel
oracle_invocation: exactly token:value(token), canonical four-string tuple
(B, heap_limit, stack_limit, depth_limit). It is outside A and catalog
JSON. The linked source-body builder receives per-invocation context and
derives token-dependent construction parameters from the tuple and linked
source/layout. Captured A/body/templates preserve it, qinplace captures
both calls, nested calls resolve their own tk. Source-adjoint replacements
precede fresh instantiation (K62); inverse traversal preserves instantiated
forward identities and tuples (K63). §4.7/§16.2 signpost the channel;
K77 is the only new row. No synthetic source formal or args entry.
The two-token inc8 worked example uses the same closure/[x,t,f] at B=2
and B=3: two versus three frames, fault versus x+1, final (x+1,1) from
zero target/fault. The regression covers all 256 x with nonzero target/
fault seeds and complete inverse restoration, already green before the
text edit. Existing K63 qualifies the bead's shorthand: separate source
points each have call_index=0; two calls in one row expansion use 0/1.
The generic constructor expander is not a complete ISA source-body builder;
no new 0.9 executor/source-expander conformance is claimed.

Commits in this session:
- 44ffea0 — pass VI spec/diff and frame-domain regression.
- 74af3fe — pass VI dispositions, gates and Beads export.
- 1d86f00 — pass VII context contract, K77, diff and two-token regression.
- 4f7e18f — distinguish source-adjoint from inverse context traversal.
- Final record commit — pass VII PROGRESS/Beads export and this handoff.
Exact normative text, dispositions, rationale and evidence are in the top
two blocks of spec/isa-0.9.PROGRESS.md. Earlier pass V brief remains at
docs/briefs/2026-11-01-isa-09-pass-v-73y.md.

Required full gates, all exit 0:
- Pass VI reference: 467 passed, 0 failed, 5 unchanged opt-in ignores.
- Pass VI conformance: 113 passed, 0 failed, 0 ignored.
- Pass VII reference: 468 passed, 0 failed, 5 unchanged opt-in ignores.
- Pass VII conformance: 113 passed, 0 failed, 0 ignored.
- Both passes: python3 -I conformance/tests/draft_witnesses.py PASS;
  python3 -I conformance/tools/ref_catalog.py --check PASS (219 recipes).
  Final witness rerun after the source-adjoint wording clarification.
- Rustfmt checks pass for the two touched test files; owned source/progress
  whitespace checks pass. K1–K76 byte-identical to starting 6a7cea3;
  K77 only new row. Exact-label diff regenerated after each spec amendment.
All gate commands unset DISPLAY, WAYLAND_DISPLAY, DBUS_SESSION_BUS_ADDRESS
and BROWSER and set TMPDIR to this repo's .tmp. Cargo uses --offline with
the required reference-interpreter/Cargo.toml and conformance/Cargo.toml.
Logs: .tmp/pass-vi-{reference,conformance,witness,catalog}.log and
.tmp/pass-vii-{reference,conformance,witness,catalog}.log.
Before-text regression evidence: .tmp/pass-vi-before-text.log and
.tmp/pass-vii-before-text.log.

Accepted 0.8 spec/Bennett-ISA-Spec.md was never edited; SHA-256:
90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e.
Reference object admission remains 0.8 under the existing binding erratum;
no version-gate change. Published spec/ref-0.9.json is byte-identical to
starting 6a7cea3; byte SHA-256:
52dbc3f8f0c91d587fd933a43080385fd5284abcfec0ce84788229fcb15c53d8.
Its 219-recipe envelope remains:
5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197.
Final 0.9 spec SHA-256: 9cfba0edcf76ebcd1e0eca8fc14f8028e2e1b0bc098af8d4a8514e8e2f0cb833.
Final exact-label diff SHA-256: 290cdeb18ae65318037c90874c31e300563415059440c3e5436135a66a3de5ce.

No push, remote/Dolt sync, VM/conformance sibling writes, fixture changes
or test removals. Preserve the pre-existing untracked .tmp/ (also contains
this session's logs). After the final record commit, owned tracked paths
are clean; only that pre-existing untracked directory remains.

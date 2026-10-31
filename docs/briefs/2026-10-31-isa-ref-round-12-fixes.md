# Brief: reference interpreter, council round 12 → round 13 fixes (lossless layout leaves; interface names shared with symbols)

Repository `/home/tobias/Projects/bennett-workspace/bennett-isa-spec`, crate `reference-interpreter/` (reviewed snapshot
09fc712; 353 tests + 5 sweeps). You are the **sole implementer** of `reference-interpreter/` for this round. Council
round 12 (`/home/tobias/Projects/bennett-workspace/bennett/docs/council/isa-ref-2026-10-29/round-12/`, members sol and
codex; report.md, out-sol.json, out-codex.json) returned: **sol core accept** (overall revise, one contract blocker);
codex revise (one contract, one core); every round-11 blocker resolved by both members; no regressions; mean 4.56.
The round-7…11 briefs (`docs/briefs/2026-10-3*-isa-ref-round-*-fixes.md`) still bind; this brief states the round-12
work. Read both members' full text in report.md first (exact witnesses, independently derived pins, expected
envelopes line by line).

## The blockers

- **SOL-R12-B01 ≡ codex R12-B01 (contract, §9.2, §16.1).** `identity.rs::layout_leaves` flattens structured layout
  leaves into a `BTreeMap<String, Json>` keyed by the dotted rendering, which is not injective when NAME contains
  dots: `interface_regions["a"]["c.db"]` and `interface_regions["a.c"]["db"]` both render as
  `interface_regions.a.c.db` and the second overwrites the first in CC/R envelopes ("all layout leaves"). Round 11's
  exact-owner oracle selection is correct and must be preserved. Fix: a lossless leaf collection (original key
  components, rendered path, value); closure selection by structural keys; every selected leaf emitted once, including
  leaves whose rendered paths coincide; adapt lproc global lookup/image collection to the lossless representation
  without changing their closure selection; never ban dots. Tie order within a coincident rendered path: §9.2 (5b)
  specifies none; adopt sol's proposal (sort by rendered dotted path bytewise, break ties bytewise by the original
  key components in structural order) and record it as a SPEC-QUESTIONS entry with sol's wording, flagged for the
  0.9 draft (one sentence in §9.2; do not edit `spec/`). Red tests `tests/identity.rs::r12_cc_r_colliding_dotted_leaves`
  (codex's equal-valued object, whose expected envelope is tie-order independent: two identical
  `layout interface_regions.a.b.c {"base":16}` lines) and sol's unequal-valued object (deep-a-32 plus `@main`; both
  leaves in structural order) with literal envelope bytes, both interface and closure pins, `check_hashes`
  succeeding; CLI check/hash counterparts; the round-11 exact-owner oracle tests retained (a foreign owner's base
  must still not change an oracle's envelope).
- **codex R12-B02 (core, §4.2, §9.2; rule 10).** `object.rs::Object::validate_symbols` rejects an oracle interface
  region whose name also names a function symbol or import ("§9.2: interface region o of @o also names a symbol"),
  although §4.2 says `lift.addr @g` names an interface region only in the body of the declaring oracle and §9.2
  says such a reference "is not a symbol reference and is left unchanged". This is a scope reduction with no
  SPEC-QUESTIONS record. Fix: remove the ban; in `identity.rs::Object::identity` and `envelope` classify reference
  tokens by instruction/operand context (a `lift.addr` naming the current oracle's own interface region is not a
  dependency and is not replaced, even when the spelling matches a function; call/qregion/global operands remain
  references), so the reference graph does not form a false self-SCC or `@$0`. Red tests with codex's exact object
  (`oracle @o … (o:region[0,8] in)`, `p = lift.addr @o`; interface=closure 05827c10…2e91, object 44158d33…7f5d; the
  expected envelope listed line by line) in `tests/identity.rs` and `tests/cli_xam.rs`: hash/check succeed;
  `run-oracle … o --input "" --mem o=00 --budget 2,0,0,0` → `{"code":0,"payload":16,"steps":2}`; budget 1,0,0,0 →
  `{"code":1,"payload":0,"steps":1}`; the control differing only as codex states; negatives where the same spelling
  is used as a call operand (still a reference) and where a non-declaring oracle uses `lift.addr @o` (resolves to a
  global or rejects per §4.2).

Hand-verify every pin with the member's derivation before trusting it. Every round 4–12 positive and negative is
retained.

## Deliverables and process

As in the round-11 brief: red-green per witness; deterministic families (dotted-name leaf collisions at depth 2–3,
equal and unequal values; interface names colliding with every symbol kind); rerun the five ignored sweeps;
README/PROGRESS "Design" paragraph (lossless layout leaves and tie order; context-classified references); the
SPEC-QUESTIONS entry; response at
`/home/tobias/Projects/bennett-workspace/bennett/docs/council/isa-ref-2026-10-29/round-12/response.md` in the
round-11 format with a Design section. Owned paths: `reference-interpreter/` and the response file only (`spec/` and
`conformance/` belong to a concurrent agent; `conformance/tests/ref_call_flag.rs` is intentionally red there; run
only the crate's own tests with `--manifest-path …/reference-interpreter/Cargo.toml --offline`). Commit by explicit
path list after every green step with `BEADS_HOOK_TIMEOUT=20` and absolute paths; never push; headless (unset
DISPLAY, WAYLAND_DISPLAY, DBUS_SESSION_BUS_ADDRESS, BROWSER; TMPDIR under the repo's `.tmp`); no `cd` in compound
commands; rule 10 throughout. Keep `reference-interpreter/PROGRESS.md` current. Final message: commit hashes,
per-blocker table, design summary, every observed count (tests before → after, sweeps), the SPEC-QUESTIONS wording,
any ISA 0.8 gap with the exact change.

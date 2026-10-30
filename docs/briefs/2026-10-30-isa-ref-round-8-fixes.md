# Brief: reference interpreter, council round 8 → round 9 fixes (complete the fact engine)

Repository `/home/tobias/Projects/bennett-workspace/bennett-isa-spec`, crate `reference-interpreter/` (reviewed snapshot
00ceee2; 297 tests + 5 sweeps). You are the **sole implementer** of `reference-interpreter/` for this round. Council
round 8 (`/home/tobias/Projects/bennett-workspace/bennett/docs/council/isa-ref-2026-10-29/round-8/`, members sol and
codex) returned no consensus: four distinct S0 blockers (two seen by both), mean 4.25, no regressions; SOL-R7-B01 and
R7-B02 are marked *partially* resolved because the new engine's matched-inverse and call rules are incomplete. The
round-7 brief (`docs/briefs/2026-10-30-isa-ref-round-7-fixes.md`: read-first list, method, gate, owned paths, process)
still binds; this brief states the round-8 work.

## The blockers (full text in `round-8/out-sol.json`, `round-8/out-codex.json`)

- **SOL-R8-B01 ≡ codex R8-B02 (core).** Matched-inverse restoration recognises only a single untouched gate pair.
  `h c; x c; x c; h c` and `h c; s c; sdg c; h c` on clean `c` are exact identities (§5.3 adjoint table) but the outer
  saved fact is destroyed by the inner gates. Implement matched-inverse recognition as a **per-wire operation stack**:
  push each non-permutation or basis-changing gate on the wire; when the next gate is the exact adjoint of the top
  (same operands, adjoint per §5.3, no intervening touch of the other operands' facts that the pair depends on), pop
  and restore the fact state saved at the push; nested pairs restore in LIFO order. A gate that is not the adjoint of
  the top leaves the stack as is (facts stay unknown). Known-basis permutation gates (x, cx with known operands) keep
  transforming facts as now and do not disturb the stack of a wire whose fact is unknown.
- **SOL-R8-B02 ≡ codex R8-B01 (core).** A zero-trip `qloop 0 { … }` is skipped by `Facts::sequence` but its body still
  mutates the compiler environment (`Compiler.bundles`, `live_borrowed`), so a live borrowed wire can be relabelled as
  an allocation that never happened and the non-permutation touch guard is bypassed (unsound admission); in the
  other witness a known-zero `c` is lost. A zero-trip body must have **no** effect on facts, on the name/bundle
  environment, or on the borrowed set; make the fact pass and the environment pass one pass over one structured
  traversal (the round-6 LifetimeWidths plan already models executed lifetimes; reuse it).
- **codex R8-B03 (core).** `action_call` (and `ctl … action_call`) becomes `Other(all touched wires)`, so a read-only
  control with a known-zero value becomes unknown although the controlled call is the identity. Transfer facts through
  calls by the callee's analysed effect: a call under a control whose fact is a known value that disables it is a
  no-op for facts; otherwise apply the callee's per-wire fact transfer (computed once per callee, memoised, recursion
  bounded by the call graph's SCCs; unknown for anything the callee's own analysis cannot establish). Read-only
  controls never lose their fact.
- **SOL-R8-B03 (contract).** `Decimal::mul_int(n: u128)` builds `BigInt::from_i128(n as i128)`; u128::MAX becomes −1 and
  certificate composition silently treats a large unsigned multiplicity as negative. Convert without narrowing and
  add the member's exact `tests/r1_linking.rs` fixture (client/callee composition multiplicity u128::MAX, expected
  inclusive error map value 340282366920938463463374607431768211456).

Every member witness (exact objects, independently computed hashes, expected check/forward/adjoint/reverse results,
plus the required negatives: an unmatched inner gate must still reject; a control with unknown fact must still make
the call opaque) goes in as a red test first; retain every round 4–8 positive and negative.

## Deliverables and process

As in the round-7 brief: red-green per witness; deterministic variant families (nested matched pairs to depth 3,
interleaved pairs on two wires, zero-trip bodies that rebind names, controlled calls with known-zero / known-one /
unknown controls, callee chains) in `tests/r1_fuzz_discipline.rs`; rerun the five ignored sweeps; README/PROGRESS
"Design" paragraph updated with the stack rule, the single-traversal rule and the call-transfer rule; response at
`/home/tobias/Projects/bennett-workspace/bennett/docs/council/isa-ref-2026-10-29/round-8/response.md` in the round-7
format with a Design section. Owned paths: `reference-interpreter/` and the response file only (`spec/` and
`conformance/` belong to a concurrent agent). Commit by explicit path list after every green step with
`BEADS_HOOK_TIMEOUT=20` and absolute paths; never push; headless with TMPDIR under the repo's `.tmp`; no `cd` in
compound commands; rule 10 throughout. Final message: commit hashes, per-blocker table, design summary, every
observed count, any ISA 0.8 gap with the exact change.

# Brief: reference interpreter, council round 14 → 15 fixes (one declarative metadata schema)

Repository `/home/tobias/Projects/bennett-workspace/bennett-isa-spec`, crate `reference-interpreter/` (reviewed snapshot
e3ae1af; 375 tests + 5 sweeps). Sole implementer of `reference-interpreter/`. Round 14 (codex seat only, policy
2026-10-31; `bennett/docs/council/isa-ref-2026-10-29/round-14/report.md`, `out-codex.json`): all round-13 blockers
resolved; two new core blockers, again metadata typing, now inside `certificate` claims. Rounds 7…13 briefs bind.

## Blockers
- **R14-B01** `metadata_types.rs::claim`, `certificate.rs::validate_claim`: channel Y interval endpoints accept JSON
  numbers; §12.1 requires signed finite decimal **strings**. The r13 generated family's union permitted both, so it
  could not catch this.
- **R14-B02** same files: `expected.table` checks only the outer array; rows and integer components are untyped
  (`[false]`, `["wrong"]`, `[[false]]`, `[[{}]]` all check ok). The family visits no table row.

## Required: stop patching, make the schema one reviewable artefact
This is the third round on the same defect class (R13: services version, block_image, derived widths; R14: Y
endpoints, table rows). Fix the class:
1. **One declarative schema table** in one file (`src/metadata_schema.rs` or a data file it includes): every metadata
   section the crate accepts (`layout`, `devices`, `certificate` with claims/dual/Y/expected.table/…, `data`, any
   other) as a recursive type grammar (object with required/optional keys, array-of, string-decimal, u64, bool,
   enum, hash-hex, …), each leaf annotated with the §9.1/§12.1/§12.2 clause it encodes. **Both** admission
   validators consume this table (no second hand-written type check anywhere); hand-written checks remain only for
   semantic constraints the grammar cannot express (ranges, cross-field) and each cites its §.
2. **The generated family is derived from the same table** and walks every path recursively (including array
   elements and nested object keys at every depth), substituting every wrong JSON kind plus the integer/decimal
   edge values; positives for every accepted alternative. Report the path count and negative count. Because the
   family is derived from the table, its job is to prove the validators enforce the table; the **table itself is
   what the council reviews against the spec**, so make it readable: one entry per line, clause cited.
3. Red-green for both witnesses with the member's exact objects and pins (hand-verify); CLI negatives; the r13
   family replaced, not duplicated; all round 4–14 tests retained; five sweeps; README Design paragraph (single
   metadata schema; where semantic checks live); PROGRESS round-14 block in the current style; response in the
   round-13 format at `bennett/docs/council/isa-ref-2026-10-29/round-14/response.md`.

## Process
Owned paths `reference-interpreter/` and the response file only; `spec/`, `conformance/` belong to a concurrent
agent. Headless prefix; `cargo test --offline --manifest-path …/reference-interpreter/Cargo.toml`; fmt; clippy
`--all-targets -- -D warnings`. Explicit-path commits with `BEADS_HOOK_TIMEOUT=20`, absolute paths, no `cd`; never
push. Final message: blocker table; schema table location, entry count and clause coverage; family path/negative
counts; totals before/after; sweeps; commits in both repos.

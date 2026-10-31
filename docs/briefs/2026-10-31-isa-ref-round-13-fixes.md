# Brief: reference interpreter, council round 13 → 14 fixes (metadata type validation; derived-width overflow)

Repository `/home/tobias/Projects/bennett-workspace/bennett-isa-spec`, crate `reference-interpreter/` (reviewed snapshot
932f19b; 365 tests + 5 sweeps). Sole implementer of `reference-interpreter/` for this round. Round 13
(`bennett/docs/council/isa-ref-2026-10-29/round-13/report.md`, `out-sol.json`, `out-codex.json`): every round-12
blocker resolved by both seats; three new core blockers, all in the metadata/layout validation layer. The round
7…12 briefs still bind. Read both members' full text first (exact witnesses and pins).

## Blockers
- **SOL-R13-B01** `validation.rs::Object::validate_metadata`: `devices.services.*.version` accepts any JSON type
  (7, false, null, [], {}); only the models branch calls `version.string`. Reject at load with §9.1.
- **SOL-R13-B02** `machine.rs::DerivedWidths::from_program` (lines 81, 85): legal u64-extreme layout values
  (`S_cells` or `A_tab` = 2^64−1) panic with "attempt to add with overflow" on `reverse`; `run-oracle` at budget 0
  is correct. Checked arithmetic everywhere derived widths are computed; a loud §-cited rejection or a correct
  result, never a panic (rule 1).
- **R13-B01** `object.rs::Object::validate_layout` / `classical.rs::ClassicalMachine::construct`: optional
  `layout.block_image` admitted without type or data-reference validation; action-only objects bypass the check that
  identity happens to perform for oracle/CC/R envelopes. Validate at load for every function kind.

## Required, beyond the three witnesses (council policy 2026-10-31: a recurring family gets a generated test)
1. **Generated schema-type family**: for every metadata section the crate accepts (`layout`, `devices`, every
   optional field including `block_image`), for every field, substitute each wrong JSON type (number, string,
   bool, null, array, object, and for integers: negative, 2^64−1, 2^64, 1.5) and assert a §9.1-cited load
   rejection before hashing or execution; positive controls for each accepted form. One table-driven test, not
   one test per field.
2. **Derived-width extremes family**: every arithmetic path from layout scalars to widths/sizes under u64 extremes,
   asserting no panic (checked ops, `saturating`/`checked` as the spec allows, else rejection citing the §).
3. Red-green per witness with the members' exact objects and pins (hand-verify the pins); all round 4–13 positives
   and negatives retained; five sweeps rerun; README Design paragraph (schema-typed metadata; checked derived
   widths); PROGRESS round-13 block (append to the existing round-12 block style; the file was cut to the latest
   round + resume list, keep it that way); apply the three suggestions where cheap, else record why not.

## Process
Owned paths: `reference-interpreter/` and `bennett/docs/council/isa-ref-2026-10-29/round-13/response.md` (round-12
format). `spec/`, `conformance/` belong to a concurrent agent; `conformance/tests/ref_call_flag.rs` is red by design.
Headless env prefix; `cargo test --offline --manifest-path …/reference-interpreter/Cargo.toml`; fmt + clippy -D
warnings. Commit by explicit path list with `BEADS_HOOK_TIMEOUT=20`, absolute paths, no `cd`; never push. Final
message: per-blocker table (red tests, fix location, commit), the two family test names and their case counts,
totals before/after, sweeps, commit hashes in both repos.

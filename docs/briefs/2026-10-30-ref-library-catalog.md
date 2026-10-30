# Brief: the `ref` expansion library catalog for ISA 0.9 (bead b-isa-spec-8r4)

Repository `/home/tobias/Projects/bennett-workspace/bennett-isa-spec` (HEAD 0b913da or later). You are the sole
implementer of the reference expansion library catalog. The maintainer ratified the library contract on 2026-10-30
(`/home/tobias/Projects/bennett-workspace/bennett/docs/decisions/2026-10-30-maintainer-decision-memo.md`, item A1):
schema, four-line LF envelope hashed over the whole canonical JSON document, per-row leaf ordinals, scratch identity by
expansion path, `removable` empty everywhere, deterministic Boolean decomposition (left-to-right postorder, fresh
scratch per node, no CSE, bit loops from the LSB), decomposition set = the §17 item 7 defaults made explicit. The
contract text is draft 0.9 §11.1 (`spec/Bennett-ISA-Spec-0.9-draft.md`, "Proposed reference-library contract (K11)",
now decided) and its "Decomposition set" table. Your job is the **complete** catalog under that contract, with
independent full-domain correctness evidence, and its hash. Publication to `ref/` happens after 0.9 acceptance, not
in this job.

## Read first

`CLAUDE.md` (rules 1, 3, 5, 8, 10, 11); the memo item A1 and its ratification line; draft 0.9 §11, §11.1 in full, §17
item 7, §3.5–§3.7, §4 (every instruction and row), §5.2, §5.3, §5.7, §14.2; the accepted 0.8 `spec/Bennett-ISA-Spec.md`
§4 and §11 for the motivating sentences each recipe must cite; `conformance/README.md` and the existing case
`conformance/cases/E/E-ref-09-qxor-witness/` (the single-recipe witness with its envelope hash
`299510fd67cc626fdcbcc10adcd40730f9af294ba596d86fb25fd31be08a5c7f`); `spec/isa-0.9.PROGRESS.md` (K11 entries).

## Deliverables

1. `spec/ref-0.9-draft.json`: the canonical (RFC 8785, UTF-8, no trailing LF) library document with `isa`, `name`,
   `version`, `recipes` exactly as §11.1 defines, recipes in bytewise `kind` order then parameter-tuple order, each
   with `kind`, parameters or instance widths, `operations` (fully instantiated leaves with `ordinal`) or
   `parameters` + `body` (closed constructors: gate, alloc, release, classical, for, invoke, inverse), `scratch`,
   `removable: []`, `source` (the motivating ISA sentence). Coverage, with no implementation-private leaf: every §4
   instruction and oracle row (copy/XOR/exchange, native gates kept as leaves, ripple add/sub/compare, multiply,
   division and remainder, shifts, casts, select, Boolean ops, linear inrange and flags, address dispatch and
   multiplexers, loads/stores, alloc/free, call/ret activation operations of §3.7, the §3.6 step, the §5.2
   construction order, `qif`/loops/local structure). The draft says explicitly: no permission to publish a partial
   library omitting division, stack, coherent control, memory or any instruction. Work family by family in the
   §11.1 table's order and keep the progress file current so the job is resumable.
2. `spec/ref-0.9-draft.README.md`: schema, parameters, coverage table (every §4 row → recipe kind), witnesses, and the
   envelope hash; the README is not hashed.
3. Independent checker in `conformance/` (Rust, std only; never read or copy `reference-interpreter/` source, rule 8:
   the catalog's correctness must not rest on the implementation it will later be compared with): a recipe expander
   that instantiates parameters, unrolls `for`, resolves `invoke`, reverses `inverse` per §5.3, numbers ordinals, and
   a basis-state permutation simulator that executes the expanded leaves. For every recipe at small widths (at least
   widths 1–4, and the exact §16.3 example widths where a row has one) over the **full** input domain: the result
   equals the §4 row semantics computed independently in plain Rust integer arithmetic (including wrap, signed cases,
   division by zero per the spec's fault rule), every input is preserved where the row says so, every scratch bit
   returns to zero, and reversing the ordinals restores the entry tuple. Record counts (recipes, widths, tuples).
   Add these as conformance cases (`E-ref-09-<kind>-<width>` or one aggregated case per family, unavailable pending
   0.9, naming b-isa-spec-8r4) and as `cargo test` tests in the conformance crate.
4. The library hash: compute the envelope SHA-256 independently in Rust and in Python (`python3 -I`, stdlib only; JCS
   via a small canonicaliser you write and verify on the existing qxor witness) and require agreement. Write the hash
   and the recipe count into draft 0.9 §11.1 at the single marked place that says the catalog job supplies them
   (surgical edit; nothing else in the draft), regenerate `spec/isa-0.8-to-0.9.diff`.

## Rules that bind every recipe

- Determinism: the expansion of a recipe at a given parameter tuple is a function of the catalog bytes alone.
- Cleanliness: all library scratch is clean, allocated unconditionally, released in reverse allocation order after
  uncomputation; borrowed user operands stay borrowed; external effective controls apply to semantic gates, not
  allocations (§3.5, §4.3); compute/uncompute pairs balanced when the external control is false.
- No inference from zero-valued inputs; no CSE; no implementer-private leaves; every leaf is a §4.4 gate, an
  allocation/release, or a classical event (§8.2).
- Rule 10: a row that is hard to decompose (division, dispatch over memory) is still decomposed; cost is not a reason
  to omit. If the §11.1 contract text itself cannot express a row, stop, file a bead naming the exact sentence and
  the change it needs, record it in the progress file, and continue with the other families.

## Gate

Headless, offline, `TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp` (create it), `DISPLAY`,
`WAYLAND_DISPLAY`, `DBUS_SESSION_BUS_ADDRESS`, `BROWSER` unset. `cargo fmt --check`, `cargo clippy --offline --all-targets
-- -D warnings` and `cargo test --offline` on `conformance/Cargo.toml` green before every commit; the full 0.8 harness
(reference binary at `reference-interpreter/target/debug/bennett-isa-ref`, build it offline if absent) must keep
445 pass / 0 fail; your cases add to `unavailable` only.

## Owned paths and process

- Write only `spec/ref-0.9-draft.json`, `spec/ref-0.9-draft.README.md`, `spec/ref-0.9-draft.PROGRESS.md` (your
  progress file: family, recipes done, widths checked, tuple counts, commit), `conformance/` (new modules, tests,
  cases), the single marked hash/count edit in `spec/Bennett-ISA-Spec-0.9-draft.md` plus `spec/isa-0.8-to-0.9.diff`,
  and `bd` notes/creates. Never `spec/Bennett-ISA-Spec.md`, `spec/README.md`, `reference-interpreter/`, `include/`,
  `docs/`, `worklog/`, `HANDOFF.md`, `CLAUDE.md`, any sibling repo.
- Commit after every family by explicit path list:
  `BEADS_HOOK_TIMEOUT=20 git -C /home/tobias/Projects/bennett-workspace/bennett-isa-spec commit -- spec/ref-0.9-draft.json spec/ref-0.9-draft.README.md spec/ref-0.9-draft.PROGRESS.md conformance`
  (retry on `index.lock`); never push; no `cd` in compound commands; absolute paths.
- Final message: commit hashes, the coverage table (every §4 row → recipe), recipe count, widths and tuple counts
  verified per family, the library hash from both computations, the conformance counts, and every contract gap you
  filed as a bead.

# Brief: complete the §11.1 recipe language (beads b-isa-spec-cbk, b-isa-spec-8sc) and resume the `ref` catalog

Repository `/home/tobias/Projects/bennett-workspace/bennett-isa-spec` (HEAD 51d45dc or later). You are the sole
implementer of the 0.9 §11.1 recipe-language completion **and** the `ref` catalog that depends on it (one implementer,
because the spec text and the catalog must agree in the same pass). The previous catalog job stopped correctly at a
54-recipe checkpoint (`spec/ref-0.9-draft.json`, `.README.md`, `.PROGRESS.md`; checkpoint hash
3a99ce30…055d; 40 conformance tests; harness 445/0/60) and filed two contract gaps. Read first: `CLAUDE.md` (rules 1,
3, 5, 8, 10, 11); `bd show b-isa-spec-cbk b-isa-spec-8sc` (from the repo root, `BEADS_HOOK_TIMEOUT=20`); the previous
brief `docs/briefs/2026-10-30-ref-library-catalog.md` (its rules still bind); draft 0.9 §11.1 and the
"Decomposition set" table; `spec/ref-0.9-draft.README.md` (the row → disposition table: every "blocked (cbk)" and
"(8sc)" row is your work list); `spec/ref-0.9-draft.PROGRESS.md`; ISA 0.8 §4.4 ("Gate angles are finite `f64`
literals"), §4.6, §4.2, §3.6, §3.7, §5.2, §5.3, §14.2; draft 0.9 §12.3 CLAIM-BINDING (binary64 bit-pattern hex
encoding, ratified) for reuse.

## Part 1 — extend §11.1 (surgical, within the ratified contract)

The ratified decisions (memo item A1) fix the schema shape, envelope hash rule, ordinals, scratch identity, empty
`removable`, deterministic Boolean decomposition. They do not forbid adding the binding and parameter rules the
constructor language lacks. Add, as normative text with a witness each:

1. **Typed formals and actuals** (cbk). A recipe's `parameters` become typed: integer parameters (as now), **wire
   formals** (a name with a width expression; bundles are wire formals of width n), **body formals** (an ordered
   constructor list spliced lexically at the point of use, for `qif` arms, `qloop` bodies, and structured rows),
   **symbol formals** (a callee referenced by its §9.2 closure hash; its expansion is the callee's own expansion
   under a new invocation-path element, never inlined into the caller's ordinal space, matching the existing "calls
   have their own invocation-path element and callee ordinal space" sentence), and **environment** (the inherited
   effective-control set, applied to semantic gates only, per the existing §3.5/§4.3 sentence). `invoke` binds
   actuals to formals positionally with width checks; substitution is lexical and capture-free (fresh names are
   derived from the expansion path, so repeated bodies never collide). State the error cases (arity, width,
   unbound name, cyclic invoke) as rejections of the library document.
2. **Real-valued gate parameters** (8sc). Angle actuals are finite binary64 values encoded as the K13 bit-pattern hex
   (reuse that rule verbatim), with the `inverse` constructor negating each angle by the §5.3 adjoint table; angle
   formals are typed `angle`; no arithmetic on angles inside recipes (an angle is passed through, never computed),
   which keeps expansion bytes exact.
3. **Variadic wire lists** (8sc). A wire-list formal with a length parameter (`w[n]`), index-range actuals, and an
   `each` constructor (or an extension of `for` that yields wire-list elements) so `mcx(k)` and similar gates expand
   deterministically for any k.
4. **Witness semantics for non-permutation leaves** (8sc). State that recipe correctness for h/phase/rotation leaves
   is judged against the §14.2 matrices on the full computational basis of the recipe's wires (not by a basis
   permutation), and that the T3 witness for such a recipe is its unitary on that basis.

Every sentence must pass the rubric's S0 test (two conformant implementations would not differ). Record each addition
in §0 (new K rows), the rejected alternatives in `spec/isa-0.9.PROGRESS.md`, regenerate `spec/isa-0.8-to-0.9.diff`,
and update the qxor witness only if its bytes change (they should not).

## Part 2 — resume the catalog under the completed language

Work the remaining README rows in the §11.1 table order: §4.1 classical events as bound classical leaves; §4.2
assignment/row composition (`lift.*`, boundscheck, load/store/alloc/free, faultif, call/ret with C1–C5/R1–R5
bindings); §3.6 step and §3.7 activation; §4.6 `qif`/`qloop` via body formals; §5.2 construction; native parametric
gates (rx/ry/rz, phase, mcx(k)) via angle and wire-list formals. Extend the conformance expander for the new
constructors and add an **exact** complex-amplitude checker for non-permutation leaves (exact arithmetic over the
dyadic rationals with √2 and i for h/s/t/sdg/tdg and the Clifford set; for arbitrary binary64 angles compare against
the §14.2 matrices with an explicitly stated binary64 round-trip criterion, and say so in the README). Full-domain
checks at widths 1–4 for every recipe as before; structured recipes at bounds 0, 1, 2, 3 and one large bound through
the fixpoint of the expander (expansion is finite because bounds are static). Then compute the complete library hash
in Rust and Python `-I`, require agreement, and write hash and recipe count at the single marked place in §11.1.

If the completed language still cannot express a row, stop on that row, file a bead naming the sentence and the
change, record it, and finish every other row. Rule 10: no omission for cost.

## Gate, owned paths, process

As in the previous brief: headless, offline, `TMPDIR` under the repo's `.tmp`; fmt, clippy -D warnings, `cargo test
--offline` on `conformance/Cargo.toml`; harness stays 445 pass / 0 fail (unavailable may grow by your pending-0.9
cases only). Owned paths: `spec/Bennett-ISA-Spec-0.9-draft.md` (§0 rows and §11.1 only), `spec/isa-0.8-to-0.9.diff`,
`spec/isa-0.9.PROGRESS.md`, `spec/ref-0.9-draft.json`, `spec/ref-0.9-draft.README.md`, `spec/ref-0.9-draft.PROGRESS.md`,
`conformance/`, `bd`. Never `spec/Bennett-ISA-Spec.md`, `spec/README.md`, `reference-interpreter/` (another agent
commits there concurrently), `include/`, `docs/`, `worklog/`, `HANDOFF.md`, `CLAUDE.md`, siblings. Commit by explicit
path list after Part 1 and after every family (`BEADS_HOOK_TIMEOUT=20 git -C /home/tobias/Projects/bennett-workspace/bennett-isa-spec commit -- spec/Bennett-ISA-Spec-0.9-draft.md spec/isa-0.8-to-0.9.diff spec/isa-0.9.PROGRESS.md spec/ref-0.9-draft.json spec/ref-0.9-draft.README.md spec/ref-0.9-draft.PROGRESS.md conformance`;
retry on `index.lock`); never push; no `cd` in compound commands; absolute paths. Close cbk and 8sc when Part 1 lands
with their witnesses. Final message: commit hashes, the §11.1 additions in ten lines, the completed coverage table,
recipe count, verified configurations and tuple counts per family, the library hash from both computations,
conformance counts, and any row still blocked with its bead.

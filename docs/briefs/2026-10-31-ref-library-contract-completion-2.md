# Brief: §11.1 recipe language, contract completion II (beads 1a4, 3xe, 1oa, 82x, evi) and catalog resume

Repository `/home/tobias/Projects/bennett-workspace/bennett-isa-spec` (HEAD 17a77a7 or later). Sole implementer of
the 0.9 §11.1 recipe-language completion and the `ref` catalog, as in `docs/briefs/2026-10-30-ref-library-catalog.md`
and `2026-10-30-ref-library-contract-completion.md` (both still bind: same owned paths, gate, rule 10, independence
from `reference-interpreter/`). The previous pass reached a **195-recipe checkpoint** (envelope
84e89406…3230, Rust = Python), added K21–K24, closed afi and lph, filed evi, and stopped on §4.2 `lift.load`.

## Part 1 — close the remaining contract gaps in one pass
Read each bead (`env -C <repo> BEADS_HOOK_TIMEOUT=20 bd show <id>`): `b-isa-spec-1oa` (lifted byte memory needs a
typed ordered layout-region descriptor binding: base/extent/cells with deterministic byte mapping), `b-isa-spec-1a4`
(REF symbol calls forwarding arbitrary typed interface actual lists), `b-isa-spec-3xe` (allocation-class formals and
bound output allocation identities), `b-isa-spec-82x` (qinplace printed release order vs reverse-allocation scratch
rule), `b-isa-spec-evi` (ordered heterogeneous activation location descriptors). Design the binding rules together
so they share one descriptor vocabulary (one K row per concept, cross-referenced), rather than one ad-hoc rule per
bead; every sentence passes the rubric's S0 test; each row gets a witness. Record rejected alternatives in
`spec/isa-0.9.PROGRESS.md`; regenerate `spec/isa-0.8-to-0.9.diff`; accepted 0.8 bytes and the qxor witness unchanged.
Where a bead's resolution would change already-accepted 0.8 semantics rather than extend 0.9, stop on it and say so.

## Part 2 — resume the catalog
Continue in the §11.1 table order from `lift.load`: §4.2 memory rows, allocation, call/ret bindings, §3.7 activation,
§5.2 construction, qinplace. Full-domain widths 1–4 and bounds 0/1/2/3 plus one large bound as before; exact
checkers for non-permutation leaves as before. Compute the complete library hash in Rust and Python `-I` only if the
catalog is complete; otherwise commit a checkpoint and leave the §11.1 marker untouched.

## Gate, owned paths, process
Unchanged from the previous brief. Harness 445/0 (unavailable may grow only by pending-0.9 cases); `cargo fmt`,
`clippy -D warnings`, `cargo test --offline` on `conformance/Cargo.toml` green; `ref_call_flag.rs` stays green through
the contract. Commit by explicit path list after Part 1 and after every family; `bd export -o .beads/issues.jsonl`
committed by path; never push. Final message: K rows added; beads closed/filed/carried; per-family table; recipe count
and hash (complete or checkpoint, with the blocking row and sentence if any); harness and cargo totals; commits.

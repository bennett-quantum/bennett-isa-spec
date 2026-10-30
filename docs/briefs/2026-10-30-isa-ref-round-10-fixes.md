# Brief: reference interpreter, council round 10 → round 11 fixes (strict §16.2 grammar; canonical effective controls)

Repository `/home/tobias/Projects/bennett-workspace/bennett-isa-spec`, crate `reference-interpreter/` (reviewed snapshot
697c9a8; 327 tests + 5 sweeps). You are the **sole implementer** of `reference-interpreter/` for this round. Council
round 10 (`/home/tobias/Projects/bennett-workspace/bennett/docs/council/isa-ref-2026-10-29/round-10/`, members sol and
codex; report.md, out-sol.json, out-codex.json) returned: both revise; **every round-9 blocker resolved by both
members**; no regressions; mean 4.31; the fact engine itself drew no new blocker. The round-7/8/9 briefs
(`docs/briefs/2026-10-30-isa-ref-round-{7,8,9}-fixes.md`) still bind; this brief states the round-10 work. Read both
members' full text in report.md first (exact witnesses, independently computed interface and object hashes, expected
CLI results).

## The blockers

- **SOL-R10-B01 + SOL-R10-B02 ≡ codex R10-B01 (core, §16.2 grammar).** `Object::parse_functions` strips an optional
  `fn` and then accepts any kind, so `fn oracle/action/perm_action/lproc` and bare `cc`/`r` headers load and execute;
  an explicitly empty `bounds {}` suffix is treated as absent although the production requires at least one INT
  inside present braces; and `instruction` parses the LHS with the operand-list parser, so `() = lift.retval r` is
  accepted although `lhs := NAME | "(" NAME ("," SP NAME)* ")"` is non-empty. Fix: parse exactly the `fhead`
  alternatives (`"fn" SP ("cc"|"r") | "oracle" | "action" | "perm_action" | "lproc"`), and use the same recogniser
  in `is_head` so function-boundary detection is not position-dependent; reject an empty bounds list when the clause
  is present (omitting the clause stays legal and does not forbid an empty signature `bounds` array); a non-empty LHS
  parser (reject `() =` even for zero-result instructions; legal empty operand lists stay legal). Red tests
  `tests/parser.rs::r10_function_header_prefix_matches_kind`, `r10_present_bounds_requires_an_integer`,
  `r10_exact_fhead_and_nonempty_optional_productions` with every literal mutation the members give (first and later
  definition positions; every invalid `fn`-prefixed Q kind; bare `cc`/`r`), each rejecting with a §16.2 diagnostic
  before any row executes; CLI counterparts (check/hash/run-cc exit 1, empty stdout, §16.2 on stderr, regardless of
  inputs or budget) using the members' pinned mutated objects; positive controls with the members' repinned hashes
  (`fn cc @a …` one-step COMPLETED; `regs {  }` without bounds; `regs {  } bounds {1}` on a one-trip action; ordinary
  tuple LHS). Never normalise a malformed spelling into the hash input.
- **codex R10-B02 (core, §2, §3.5 vs opaque inverse identity).** Inverse recognition keys the ordered lexical
  predicate stack instead of the semantic effective-control set: `qpred p; qpred q; O; qunpred q; qunpred p; qpred q;
  qpred p; O; …` (and `[p]` vs `[p,p]`) is rejected "§6.2: released value has not been uncomputed" although §2 defines
  the effective control context as a set ("duplicates count once") and §3.5 makes the active predicate their
  conjunction. Fix: keep the lexical stack for push/pop validation, but introduce one canonical semantic
  effective-control representation for operation identities (resolved scalar physical wires, polarity retained, sorted
  and deduplicated literals, full current version/dependency snapshots) and use it in
  `validation.rs::Object::validate_action_sequence` oracle/action keys, `Discipline::change/joint_change` and every
  other context-bearing key, and in `borrow.rs` Compiler/Node construction and instantiation before
  `Facts::reversible` compares events. Do not sort ABI input/output operands; do not discard token, callee, version
  identity, negative polarities, or the all-operand LIFO requirement. Red test
  `tests/r1_borrow.rs::r10_equivalent_control_sets_preserve_opaque_inverse` with the member's exact pinned object
  (interface 386dce74…deef, object 7b1153ba…76ad) and the duplicate-predicate variant: parse/check/hash succeed,
  `run-action a --mut 0 --in 1,1 --arg token:1,0,0,0` returns `{"011":[1,0]}` with wires=3, same for token
  0,0,0,0; forward and adjoint across all t,p,q basis inputs and a control superposition; borrowed b Bell-paired with
  an untouched reference; reverse equals_entry=true; nested `qloop 1` and local-call variants; negatives where a
  predicate is absent, has opposite polarity, changes version, or the token differs (no spurious cancellation); the
  same effective-set rule for `qinplace` swapped g/ginv keys.

Hand-verify every pinned hash with the member's derivation before trusting it; if a pin is wrong, say so with your own
derivation and the exact bytes. Every round 4–10 positive and negative is retained.

## Deliverables and process

As in the round-9 brief: red-green per witness; deterministic variant families in `tests/r1_fuzz_discipline.rs`
(permuted and duplicated predicate stacks around matched opaque pairs; grammar mutations); rerun the five ignored
sweeps; README/PROGRESS "Design" paragraph updated with the strict-grammar rule and the canonical-effective-control
rule; response at `/home/tobias/Projects/bennett-workspace/bennett/docs/council/isa-ref-2026-10-29/round-10/response.md`
in the round-9 format with a Design section. Owned paths: `reference-interpreter/` and the response file only
(`spec/` and `conformance/` belong to a concurrent agent; `conformance/tests/ref_call_flag.rs` is intentionally red
in their tree; run only the crate's own tests with `--manifest-path …/reference-interpreter/Cargo.toml --offline`).
Commit by explicit path list after every green step with `BEADS_HOOK_TIMEOUT=20` and absolute paths; never push;
headless (unset DISPLAY, WAYLAND_DISPLAY, DBUS_SESSION_BUS_ADDRESS, BROWSER; TMPDIR under the repo's `.tmp`); no
`cd` in compound commands; rule 10 throughout. Keep `reference-interpreter/PROGRESS.md` current so the job can be
resumed. Final message: commit hashes, per-blocker table, design summary, every observed count (tests before → after,
sweeps), any ISA 0.8 gap with the exact change.

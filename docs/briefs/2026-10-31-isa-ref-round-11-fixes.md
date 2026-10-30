# Brief: reference interpreter, council round 11 → round 12 fixes (action regs; layout ownership by exact name; fixture claims)

Repository `/home/tobias/Projects/bennett-workspace/bennett-isa-spec`, crate `reference-interpreter/` (reviewed snapshot
45475a1; 338 tests + 5 sweeps). You are the **sole implementer** of `reference-interpreter/` for this round. Council
round 11 (`/home/tobias/Projects/bennett-workspace/bennett/docs/council/isa-ref-2026-10-29/round-11/`, members sol and
codex; report.md, out-sol.json, out-codex.json) returned: both revise; **every round-10 blocker resolved by both
members**; no regressions; mean 4.44; three S0 items, each found independently by both members. The round-7…10
briefs (`docs/briefs/2026-10-30-isa-ref-round-{7,8,9,10}-fixes.md`) still bind; this brief states the round-11 work.
Read both members' full text in report.md first (exact witnesses, independently derived pins, expected results).

## The blockers

- **SOL-R11-B01 ≡ codex R11-B01 (core, §3.1, §3.3, §16.2).** `validate_header` restricts oracle/lproc `regs` to
  `reg[N]` and forbids CC/R register files, but has no action/perm_action guard, so `regs { d:wire }` manufactures a
  statically live wire (sol: `cx d t` passes check, fails at run with "§3.3: unknown live bundle d") and
  `regs { angle:f64 }` an angle with no source (codex: `rx angle t` passes check, fails at run with §4.4). Fix:
  reject non-empty `regs` on action/perm_action under §3.3 at load, before hashing/execution (`regs {  }` and the
  bounds clause stay legal); in `validation.rs::validate_action_sequence` a non-literal rotation angle must name an
  f64 in the function's classical argument list (§3.1 "finite f64 literals or classical arguments fixed at region
  entry"), not merely a `Type::F64` in the type map. Red tests `tests/parser.rs::r11_action_regs_do_not_produce_wires`
  and `r11_action_regs_cannot_invent_angles` with both exact objects, perm_action and indexed `d[0]` variants, regs
  types across all six kinds; CLI counterparts (hash/check/run-action exit 1, empty stdout, §3.3 before any row
  executes); the members' positive controls with their repinned hashes (qalloc'd `d`; a real f64 argument).
- **SOL-R11-B02 ≡ codex R11-B02 (contract, §9.2, §16.1).** `identity.rs::Object::envelope` selects an oracle's
  layout leaves by `starts_with("interface_regions.<name>.")` on flattened dotted paths; NAME may contain dots, so
  oracle `a` absorbs the `interface_regions.a.b.*` leaves of the distinct oracle `a.b` and its interface/closure hash
  changes when `a.b`'s base moves. Fix: select `layout.interface_regions.get(name)` by exact JSON key on the
  structured object before flattening, keep every non-interface layout leaf, never ban dots. Red tests
  `tests/identity.rs::r11_oracle_layout_ownership_uses_exact_names` and `r11_dotted_oracle_layout_ownership` with
  both members' objects, literal expected envelope bytes and independent SHA-256 pins (sol: @a 1b593b90…9521 with
  the exact envelope lines listed; codex: @a 4671ca55…2aec, @a.b 26455a8e…d160 at base 32 and the given value at
  base 48; @a unchanged across that move); check must exit 0 with the correct pins.
- **SOL-R11-B03 ≡ codex R11-B03 (contract, fixture evidence; §3.1, §4.4, §4.5, §14.2).** Two hand-derived fixture
  claims are false while the runtime is right. (1) `tests/fixtures/r9-sol-qinplace.comments` (and the generator
  `tests/support/r9_pins.py`, which stamps the same sentence on every oracle-bearing R9 case) says B=0 fault XORs
  cancel; for qinplace B=0 correctly rejects under §4.5. Replace with sol's exact sentence ("Hand: for B >= 1, @o is
  the one-step identity permutation, both qinplace rows are I, H^2=I, and CX(0,b)=I_bR. B=0 rejects under §4.5
  because the oracle faults; it is not an identity run.") and keep the cancellation text only for direct oracle_call
  fixtures where it holds; add `tests/r1_fixtures.rs::r11_qinplace_comment_matches_budget_contract` asserting the
  comment and running the literal object at B=0 (§4.5, exit 1) and B=1 (exact identity). (2) `control_flow.comments`
  claims `Rz(2*pi) = -I` and a probability-one `p=1` outcome; the literal angle is the binary64 6.283185307179586,
  so P(p=0) = cos(theta/4)^2 ≈ 3.75e-33 > 0 and the actual state is the one codex pins. Fix per codex: add an exact
  minus-I action (`s t / s t / x t / s t / s t / x t`) and point controlled_phase at it; keep the rounded-angle
  @phase for literal-parser/rotation coverage, described by the exact formula; update dependent identity pins from
  literal envelopes; strengthen `tests/quantum_control.rs::qif_and_nested_control_preserve_interference` to require
  `read_basis(p)==1` and exact support {01: (1,0)}.

Hand-verify every pin with the member's derivation before trusting it. Audit every other `.comments` file and
generated comment for the same class of error (hand claim vs. actual runtime result) and fix any you find, listing
them in the response. Every round 4–11 positive and negative is retained.

## Deliverables and process

As in the round-10 brief: red-green per witness; deterministic families where they apply (regs across kinds; dotted
names in ownership); rerun the five ignored sweeps; README/PROGRESS "Design" paragraph (kind-specific regs typing,
structured layout ownership, fixture-evidence rule); response at
`/home/tobias/Projects/bennett-workspace/bennett/docs/council/isa-ref-2026-10-29/round-11/response.md` in the
round-10 format with a Design section. Owned paths: `reference-interpreter/` and the response file only (`spec/` and
`conformance/` belong to a concurrent agent; `conformance/tests/ref_call_flag.rs` is intentionally red there; run
only the crate's own tests with `--manifest-path …/reference-interpreter/Cargo.toml --offline`). Commit by explicit
path list after every green step with `BEADS_HOOK_TIMEOUT=20` and absolute paths; never push; headless (unset
DISPLAY, WAYLAND_DISPLAY, DBUS_SESSION_BUS_ADDRESS, BROWSER; TMPDIR under the repo's `.tmp`); no `cd` in compound
commands; rule 10 throughout. Keep `reference-interpreter/PROGRESS.md` current. Final message: commit hashes,
per-blocker table, design summary, every observed count (tests before → after, sweeps), any ISA 0.8 gap with the
exact change.

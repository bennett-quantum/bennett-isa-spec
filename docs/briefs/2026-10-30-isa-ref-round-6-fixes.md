# Brief: reference interpreter, council round 6 → round 7 fixes

Repository `/home/tobias/Projects/bennett-workspace/bennett-isa-spec`, crate `reference-interpreter/`
(HEAD b2e4fc1; the crate is at the state reviewed as snapshot 9e4984f, 262 tests + 5 ignored sweeps). You are the
**sole implementer** of `reference-interpreter/` for this round. Council round 6
(`/home/tobias/Projects/bennett-workspace/bennett/docs/council/isa-ref-2026-10-29/round-6/`, members sol and codex)
returned no consensus with two shared S0 core blockers; every previous blocker is resolved. Fix both in place; nothing
is argued away.

## Read first, in full

1. `CLAUDE.md` (rules 1, 3, 8, 10, 11). Rule 10 is absolute: no fix may reject, fault or restrict a well-formed object
   that ISA 0.8 admits (no trip-count limits, no frame-merging waivers, no banning of the witnesses).
2. `../bennett/docs/council/isa-ref-2026-10-29/round-6/report.md`, then the full `problem`/`fix` text in
   `round-6/out-sol.json` and `round-6/out-codex.json` (the JSON is the source; the report abbreviates).
3. `../bennett/docs/council/README.md` (editor loop), `../bennett/docs/council/rubric-impl.md` (S0 definitions, the
   five hand-reproduction methods the members apply), `round-5/response.md` (the response format you must follow).
4. `reference-interpreter/PROGRESS.md`, `reference-interpreter/README.md`, `reference-interpreter/SPEC-QUESTIONS.md`.
5. `spec/Bennett-ISA-Spec.md` (ISA 0.8; never edit it) §§3.3, 4.3, 4.6, 5.1–5.3, 5.6, 6.1–6.2, 14.2 as each blocker needs.

## The two blockers (each raised independently by both members)

- **Borrow analysis loses clean workspace across a non-permutation boundary** — SOL-R6-B01 ≡ codex R6-B02.
  `src/borrow.rs::Object::decide_frames` / `factorises` partition at `Other(H t)` and re-initialise an untouched,
  still-live `qalloc`'ed wire as a free variable, so `c = qalloc; h t; b = qborrow; [cx c b;] qreturn b; [h t;] qfree c`
  is rejected (§5.6 / "released wire not clean") although `c` is provably |0⟩ throughout. Carry established facts about
  untouched live workspace across the boundary, invalidating only what the boundary operation can affect; keep fresh
  borrowed entries independent and keep the mandatory permutation-only borrowed-touch check and every per-return,
  per-frame and certificate negative of rounds 4–5.
- **Lifetime widths are keyed by static source line and walk zero-trip bodies against the outer map** — SOL-R6-B02 ≡
  codex R6-B01. `src/action.rs::infer_widths`, `Environment::widths`, `Object::action_sequence`: `qloop 0 { qfree d }`
  removes `d` for the unexecuted body and the real `qfree d` then fails ("§3.3: missing consumed lifetime width");
  the changing-width loop (`qloop 2 { qfree d; d = qalloc any 2; x d[1]; x d[1] }` after `d = qalloc any 1`) recreates
  width 1 at both inverse visits and the inverse `x d[1]` fails ("wire index out of range"). Model the executed lifetime
  environment (zero-trip bodies have no effect on the outer live map; releases match their dynamic producer per loop
  entry/iteration, or an equivalent sound structured representation) without any trip-count restriction.

Both members give the exact witness objects, their independently computed interface/closure hashes and object hashes,
the expected forward/adjoint/reverse results, the required positive variants (H after qborrow before CX; lifetime in a
loop; doubled `x b; x b`; nested zero-trip loops; qborrow/qreturn analogue; alpha-renamed unrollings; `b` entangled
with an untouched reference in library tests) and the negatives that must keep rejecting. Use them verbatim.

## Method

- Red-green per witness (rule 3): the exact fixture as a failing test first (tests/r1_borrow.rs and tests/quantum.rs for
  library-level, tests/cli_xam.rs for the CLI: hash/check exit 0 with the pinned hashes, forward and adjoint at
  t=0/1 and superposition, `reverse` with `equals_entry=true` and no remaining workspace), then the fix, then the
  retained negatives and the full suite.
- Also add the two deterministic families both members asked for under C7 in `tests/r1_fuzz_discipline.rs` (unrelated
  H/H boundaries around clean-control borrowed lifetimes; zero/one/multiple trips with outer-to-inner width-changing
  lifetimes; independent identity maps and alpha-renamed unrolling as expectations; existing RNG distribution unchanged).
- Rerun the ignored sweeps whose domain the two fixes touch (`cargo test --offline -- --ignored <name>`) and record the
  observed counts; do not copy the old numbers.
- C8: update `README.md`'s round-2 static/display-semantics paragraph and `PROGRESS.md` with the exact boundary /
  zero-trip / loop-carried pins and observed results; qualify the old R5 width evidence as covering its tested
  lifetimes. The C2–C5 extra tests the members suggested are welcome after the blockers are green, not required.
- If a fix needs a reading of the spec that the text does not settle, add a SPEC-QUESTIONS entry with a proposed 0.9
  wording; never change behaviour on an unjustified reading (rubric S0).

Write `/home/tobias/Projects/bennett-workspace/bennett/docs/council/isa-ref-2026-10-29/round-6/response.md` in the
format of `round-5/response.md`: one entry per blocker id (SOL-R6-B01, SOL-R6-B02, codex R6-B01, R6-B02, grouped
pairwise), each with the RED test name and its failing diagnostic, what changed (file::function), the retained
negatives, and the exact commands and counts you observed. Written for a hostile reader; the members re-grade from a
fresh worktree snapshot with the diff attached.

## Gate (headless, offline, project-local TMPDIR)

```sh
mkdir -p /home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER \
  TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp \
  cargo fmt --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml -- --check
# same env for: cargo clippy --offline --all-targets -- -D warnings ; cargo test --offline
```

## Owned paths and process

- Write only `reference-interpreter/` (never `spec/`, `isa_check.py`, `conformance/`, `include/`, `SPEC_VERSIONS.toml`,
  `worklog/`, `HANDOFF.md`, `CLAUDE.md`) and the response file above. Do not read or copy code from `../bennett-vm` or
  `../bennett-circuits` (contract boundary, rule 5).
- Keep `reference-interpreter/PROGRESS.md` current (a "Round-6 fixes" table: blocker id, RED test, status, commit) so
  a restarted session resumes from it.
- Commit your owned paths after every green step, never a red tree:
  `BEADS_HOOK_TIMEOUT=20 git -C /home/tobias/Projects/bennett-workspace/bennett-isa-spec commit -- reference-interpreter`
  and `BEADS_HOOK_TIMEOUT=20 git -C /home/tobias/Projects/bennett-workspace/bennett commit -- docs/council/isa-ref-2026-10-29/round-6/response.md`
  (retry on `index.lock`). Do not push. No `cd` in compound commands; absolute paths everywhere.
- Surprises or spec gaps for the maintainer: `bd create` run from the repo root (`BEADS_HOOK_TIMEOUT=20`); never
  narrow to fit.
- Final message: commit hashes, the per-blocker status table, every observed test count (normal suite, each rerun
  sweep, clippy, fmt), and any ISA 0.8 gap you hit with the exact ISA change it needs.

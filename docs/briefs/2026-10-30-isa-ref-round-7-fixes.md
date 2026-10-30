# Brief: reference interpreter, council round 7 → round 8 fixes (root cause in the borrow analysis)

Repository `/home/tobias/Projects/bennett-workspace/bennett-isa-spec`, crate `reference-interpreter/` (reviewed snapshot
67d1b94; 274 tests + 5 ignored sweeps). You are the **sole implementer** of `reference-interpreter/` for this round.
Council round 7 (`/home/tobias/Projects/bennett-workspace/bennett/docs/council/isa-ref-2026-10-29/round-7/`, members
sol and codex) returned no consensus: four S0 blockers, every round-6 blocker confirmed resolved, mean 4.25.

## Read first, in full

1. `CLAUDE.md` (rules 1, 3, 8, 10, 11). Rule 10: no fix may reject, cap or restrict a well-formed object ISA 0.8
   admits (no trip-count limits, no truncated loop expansions that *admit*, no frame-merging waivers).
2. `round-7/report.md`, then the full `problem`/`fix` text in `round-7/out-sol.json` and `round-7/out-codex.json`
   (the JSON is the source).
3. `round-6/response.md` (format to follow), `../bennett/docs/council/rubric-impl.md`, `reference-interpreter/
   PROGRESS.md`, `README.md`, `SPEC-QUESTIONS.md`.
4. `spec/Bennett-ISA-Spec.md` (ISA 0.8, never edit) §§3.3, 3.5, 4.3, 4.4, 4.6, 5.1–5.3, 5.6, 6.1–6.2, 14.2.

## The blockers

- **SOL-R7-B02 ≡ codex R7-B01 (regression, core).** `src/borrow.rs` expands a loop containing a non-permutation gate
  into at most `n.min(3)` body copies. Combined with the round-6 prefix replay, the truncated circuit supplies entry
  facts for frames that are never checked (iteration 4 of a `qloop 4` with `x c[0]`/`swap` evolving `c`), and an
  invalid borrowed operation is admitted. In round 6 the same object rejected because each post-H frame started
  unconstrained. Both members give the exact object, hashes and the expected rejection.
- **SOL-R7-B01 (core).** A matched non-permutation pair on clean workspace itself (`c = qalloc; h c; h c; b = qborrow
  ...`) is never recognised as restoring c's known-zero entry before the following borrow frame; the valid object
  is rejected.
- **codex R7-B02 (core).** Boundary repair preserves facts only on *untouched* wires; a diagonal gate (`s c`, `sdg c`)
  or a read-only quantum control on a wire with a known basis value wrongly destroys that fact, rejecting a valid
  object.
- **SOL-R7-B03 (contract).** `Dyadic::to_f64` overflows its numerator before applying the cancelling denominator, so a
  valid bounded one-wire circuit (`qloop 5000 { h t; t t }`) cannot be reported.

## Direction: fix the cause, not the symptom

Rounds 4–7 have each added a special case to the borrow-frame analysis (frame partition at `Other`, prefix facts,
boundary carry) and round 7's regression came from one patch meeting an older shortcut. Do not add a fourth patch.
Replace the fact engine with one principled, sound rule set and derive every current behaviour from it:

1. **Facts per wire, transformed by every gate's known action on basis states**, not "kept if untouched, dropped if
   touched": X flips a known basis value; Z, S, Sdg, T, Tdg, phase and any diagonal gate preserve it; a CX/CCX/SWAP
   with known basis operands maps known values to known values; a read-only control with a known value leaves the
   control's fact intact; H and other non-diagonal non-permutation gates make the wire's value unknown **and** record
   the operation so that an exactly matched inverse on the same wire with no intervening touch restores the prior
   fact (§5.3 adjoint; this closes SOL-R7-B01); when a wire becomes unknown, facts on wires it was entangled with by
   intervening two-qubit gates are invalidated conservatively (never the other way round). Unknown is the only
   permitted default; a fact is never invented.
2. **Loops: no truncation that admits.** For a structured body whose iteration evolves facts, compute the per-iteration
   fact state exactly for every iteration the bound names when that is finite (iteration count is static per §4.6);
   where the count is too large to enumerate, iterate the fact transfer to a fixpoint on the fact lattice (unknown
   is the top) and check the frame under the fixpoint state. Admission requires a proof under either route;
   `n.min(3)` and every other sampled expansion goes away. Keep the existing cycle-skipping of *execution* (round-1
   `qloop 2^40` fix) separate from *analysis*.
3. The mandatory permutation-only borrowed-touch check, the per-return `P_F = P' ⊗ I_B` check, certificate binding,
   and every negative from rounds 4–7 stay exactly as they are; list them in the response as retained and re-run them.
4. `Dyadic::to_f64`: reduce or scale before conversion (cancel common powers, or convert via exponent arithmetic) so
   any representable value converts without intermediate overflow; red test at the member's exact witness and at
   the boundary of the old overflow.

If the principled engine contradicts a reading recorded in `SPEC-QUESTIONS.md`, say so in the progress file and add
the entry; never pick a convenient reading.

## Method

Red-green per witness (both members supply exact objects, independently computed hashes and expected outcomes):
library tests in `tests/r1_borrow.rs`/`tests/quantum.rs`, CLI tests in `tests/cli_xam.rs` (hash/check/run-action
forward and adjoint at t = 0/1 and superposition, `reverse` with `equals_entry=true`, no remaining workspace;
rejections with the quoted §5.6/§6.2 diagnostic, exit 1, empty stdout). Add the members' requested variant families
(diagonal and read-only-control boundaries; matched pairs with and without intervening touches; loops of 1, 3, 4, 5
and large bounds with evolving workspace, both the valid and the invalid shapes) to `tests/r1_fuzz_discipline.rs`
as deterministic families with independent identity maps as expectations; keep the RNG distribution unchanged.
Re-run the five ignored sweeps and record observed counts. Update `README.md` and `PROGRESS.md` (the borrow-analysis
paragraph must describe the new engine, not the old patches).

Write `/home/tobias/Projects/bennett-workspace/bennett/docs/council/isa-ref-2026-10-29/round-7/response.md` in the
round-6 format: one entry per blocker id (SOL-R7-B01, SOL-R7-B02 ≡ R7-B01, SOL-R7-B03, R7-B02), the RED test name and
diagnostic, what changed (file::function), retained negatives, exact commands and counts observed, plus a section
"Design" stating the fact rules and the loop rule in a dozen lines for a hostile reader.

## Gate (headless, offline, project-local TMPDIR)

`mkdir -p /home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp`; with `DISPLAY`, `WAYLAND_DISPLAY`,
`DBUS_SESSION_BUS_ADDRESS`, `BROWSER` unset and `TMPDIR` set there: `cargo fmt -- --check`, `cargo clippy --offline
--all-targets -- -D warnings`, `cargo test --offline` on `reference-interpreter/Cargo.toml`, then the ignored sweeps.

## Owned paths and process

- Write only `reference-interpreter/` and the response file. Never `spec/`, `conformance/` (another agent commits
  there concurrently), `include/`, `isa_check.py`, `worklog/`, `HANDOFF.md`, `CLAUDE.md`. Do not read or copy code from
  `../bennett-vm` or `../bennett-circuits`.
- Keep `reference-interpreter/PROGRESS.md` current (a "Round-7 fixes" table: blocker id, RED test, status, commit).
- Commit after every green step, never a red tree, by explicit path list:
  `BEADS_HOOK_TIMEOUT=20 git -C /home/tobias/Projects/bennett-workspace/bennett-isa-spec commit -- reference-interpreter`
  and `BEADS_HOOK_TIMEOUT=20 git -C /home/tobias/Projects/bennett-workspace/bennett commit -- docs/council/isa-ref-2026-10-29/round-7/response.md`
  (retry on `index.lock`). Do not push. No `cd` in compound commands; absolute paths.
- Surprises or spec gaps: `bd create` from the repo root (`BEADS_HOOK_TIMEOUT=20`); never narrow to fit.
- Final message: commit hashes, per-blocker table, the design summary, every observed count (suite, each sweep,
  clippy, fmt), and any ISA 0.8 gap with the exact change it needs.

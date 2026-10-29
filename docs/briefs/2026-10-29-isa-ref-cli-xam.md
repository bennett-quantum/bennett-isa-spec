# Brief: reference interpreter CLI commands run-cc, reverse, run-action, run-program (bead b-isa-spec-xam)

Repository `/home/tobias/Projects/bennett-workspace/bennett-isa-spec`, crate `reference-interpreter/`
(`bennett-isa-ref`, Rust 2021, std only). You are the **sole implementer** of this crate for this task; no
other agent edits it. Milestone 1 of the council fix loop landed at e8629de (152 tests + 5 sweeps green).

## Why

The dialect 0.2 pre-flight checker (`../bennett-dialect-spec/scripts/dialect_check_v02.py`, a separate
resumable job) cannot execute the spec's tdl / sem2 / effect example families until the interpreter exposes
four commands beyond `check` / `hash` / `run-oracle` / `step-check`. Dialect 0.2 outline part D
(`../bennett-dialect-spec/spec/outline-0.2.md`) names them:

- `run-cc <object> <symbol> --input ... [--budget ...] [--mem ...]`: a CC-tier or R-tier program on concrete
  inputs; output a JSON trace (every executed instruction with its pre/post register and memory deltas,
  budgets, final reported outcome).
- `reverse <object> <symbol> ...`: whole-run or segment reversal (ISA §14.1 prefix reversal); output the
  state after reversal and whether it equals the entry state, byte for byte, with the first difference if not.
- `run-action <object> <action> --basis <bits> | --state <amplitudes>`: a Q-tier action on a basis state or a
  small state vector; output the resulting amplitudes (exact where the crate is exact, §9.1 number format).
- `run-program <object> ...`: a launch-level host/device program (ISA §10 launch API) on concrete inputs;
  output the launch outcome, budgets and fault, in the same JSON style as `run-oracle`.

The library machinery for all four exists (machine.rs, action.rs, quantum.rs, oracle.rs, classical.rs); the
task is to expose it faithfully, not to reimplement it. Where a launch-level path depends on the staged
deliverables 8–11 (`PROGRESS.md`), implement what §10 defines and report anything staged as an explicit
error naming the staged deliverable: never a silent partial result (CLAUDE.md rule 1).

## Read first

`CLAUDE.md` (rules 1, 3, 8, 10, 11); `reference-interpreter/README.md`, `PROGRESS.md`, `SPEC-QUESTIONS.md`,
`council/response-round-1.md`; `spec/Bennett-ISA-Spec.md` §9.1, §10, §14.1 and the sections each command
touches; `bd show b-isa-spec-xam`; the outline part D above for the exact inputs the checker will pass.

## Task

Red-green per command: a failing CLI test with a hand-derived expected JSON (fixture under
`reference-interpreter/tests/` in the existing style, comments deriving the expected values from the spec),
then the implementation, then the whole suite. Exact JSON schemas documented in README.md "Commands".
Diagnostics to stderr, exit 1, citing the ISA rule. Then `cargo test --offline`, `cargo test --offline --
--ignored`, `cargo clippy --offline --all-targets -- -D warnings`, `cargo fmt -- --check`. Build/test
headless: `TMPDIR=$PWD/reference-interpreter/.tmp` created first; `DISPLAY`, `WAYLAND_DISPLAY`,
`DBUS_SESSION_BUS_ADDRESS`, `BROWSER` unset. Python seed `isa_check.py` unchanged.

## Owned paths and process

- Write only under `reference-interpreter/`. Never `spec/`, `docs/`, `worklog/`, `HANDOFF.md`, `CLAUDE.md`.
- Keep `reference-interpreter/PROGRESS.md` current (a "CLI commands (xam)" table: command, status, test, commit).
- `bd update b-isa-spec-xam --claim` at start; `bd note` for surprises; spec ambiguities go to
  `SPEC-QUESTIONS.md` with the reading chosen; `bd close b-isa-spec-xam` at the end.
- Commit owned paths after every command lands:
  `git -C /home/tobias/Projects/bennett-workspace/bennett-isa-spec commit -- reference-interpreter`
  (retry on `index.lock`). Do not push. No `cd` in compound commands; absolute paths only.
- Final message: commit hash, the four commands with one example invocation and output each, test counts,
  anything staged or declined.

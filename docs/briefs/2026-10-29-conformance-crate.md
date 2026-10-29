# Brief: bennett-isa-conformance crate (suites A-H as data + harness), C headers compiled

Repository `/home/tobias/Projects/bennett-workspace/bennett-isa-spec`. You own `conformance/` and `include/` only.
`reference-interpreter/` is being edited by another agent (council fix loop): read-only for you, use its binary
`reference-interpreter/target/debug/bennett-isa-ref` (rebuild offline if HEAD moves; README "Commands"). Never edit
`spec/`, `docs/`, `worklog/`, `HANDOFF.md`, `CLAUDE.md`.

## Read first
`CLAUDE.md` (rules 1, 3, 4, 8, 10, 11); `README.md` (layout: conformance crate `bennett-isa-conformance`, suites A-H);
`spec/Bennett-ISA-Spec.md` §16 (object format, examples, hashes) and every section that defines a suite (search
"suite" and "Suite"; §14.1 interpreter, §12 certificates, §10 launch, §11 scheduling, §8 targets); `conformance/src/
lib.rs` (skeleton); `reference-interpreter/PROGRESS.md` (which suites the interpreter already serves, which are
staged as deliverables 8-11); `reference-interpreter/tests/fixtures/`.

## Task
1. Make `conformance/` the spec's executable conformance suite as **data plus a harness**, implementation-agnostic:
   each case is a fixture file (object text, inputs, budgets, memory, expected outcome JSON, the spec sentence it
   tests) and the harness runs any binary that implements the reference CLI (`--bin PATH`, default the reference
   interpreter), comparing the spec-defined outcome fields. Cover every suite the spec defines; where the reference
   interpreter cannot yet run a suite (staged deliverables), the case still exists and fails loudly as
   "unavailable: deliverable N", never skipped. Hand-derive every expected value from the spec with a comment; never
   copy an expected value from the interpreter's output without the derivation.
2. Red-green: a case for every row/rule in §16.3 and each suite section; run the whole suite against the reference
   binary and record pass / fail / unavailable per suite in `conformance/PROGRESS.md`. Any failure where you believe
   the interpreter is wrong: `bd create` (type bug, label conformance) quoting the spec sentence; do not change the
   expected value to match.
3. `include/bennett_launch.h` and `include/bennett_sched.h` have never been compiled. Add a test in the crate that
   compiles both with `cc` (C11, `-Wall -Werror`) and, if either fails, fix only what is a plain header defect
   (missing include, typo) and `bd create` anything that is a spec question.

Headless (rule 11): `TMPDIR` under `conformance/.tmp` (create), display vars unset. `cargo test --offline`
(std only). `bd update --claim` a bead for this job (create it if none: "conformance crate suites A-H"), `bd export
-o .beads/issues.jsonl` before committing `.beads/`. Commit: `git -C /home/tobias/Projects/bennett-workspace/
bennett-isa-spec commit -- conformance include .beads/issues.jsonl` (retry on index.lock); do not push; absolute paths
only; never `cd` inside compound commands. Final message: commit hash, per-suite case counts and pass / fail /
unavailable against the reference binary, header compile result, beads created.

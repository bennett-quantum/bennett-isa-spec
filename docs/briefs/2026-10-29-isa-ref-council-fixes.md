# Brief: reference interpreter, council fix loop (round 1 → round 2)

Repository `bennett-isa-spec`, crate `reference-interpreter/` (`bennett-isa-ref`). You are now the **sole
implementer** of this crate; the codex job that built it so far is stopped (its last checkpoint is commit
0164b5a, `cargo test --offline` green: create `reference-interpreter/.tmp` first, the doctest runner needs
`TMPDIR` to exist). The maintainer's rule: this crate is the executable definition of ISA 0.8 (§14.1), so it
goes through the three-model council until every member accepts, and every blocker is fixed, not argued away.

## Read first, in full

1. `CLAUDE.md` (rules 1, 3, 8, 10, 11), `reference-interpreter/BRIEF.md` (the original job: scope is the
   whole ISA, deliverables 1–7, red-green, fixtures with hand-derived comments), `reference-interpreter/
   PROGRESS.md` (what is verified, the outstanding-work list), `reference-interpreter/SPEC-QUESTIONS.md`.
2. The council's round-1 output: `../bennett/docs/council/isa-ref-2026-10-29/round-1/report.md` and, for
   the full `problem`/`fix` text (the report truncates nothing, but the JSON is the source),
   `round-1/out-fable.json`, `round-1/out-codex.json`, `round-1/out-muse.json`. What the council grades:
   `../bennett/docs/council/rubric-impl.md` and `task-impl.md`.
3. `spec/Bennett-ISA-Spec.md` (never edit it), as needed per blocker.

## Task, in order

**Milestone 1 (report back when reached): every round-1 blocking issue resolved.** There are 30 entries
from three members with overlap (fable R1-B1 = codex B01 = muse B1 on the 128-bit cap; R1-B2 = B12 = muse
B2 on literal spellings; B14 = muse B6 on borrow frames; B16 = muse B7 on metadata; and others). For each:

- Red first: turn the council's witness into a failing test (fixture fragment or unit test; the `fix`
  fields give the input, budget and expected output). Watch it fail. Then fix. Then the whole suite.
- Where the council's reading of the spec is wrong, you may decline: quote the spec sentence that decides
  it and keep the existing behaviour, with a test that pins it. Declines must be rare and airtight; the
  members re-grade from scratch and re-raise anything unresolved.
- Where the spec is genuinely ambiguous, choose the reading most consistent with the rest of the text,
  add a `SPEC-QUESTIONS.md` entry (the question, the reading, the section, and a proposed 0.9 wording),
  and implement that reading. Never leave a path silent, never narrow.
- B19 (no target/launch/scheduling/certificate-checker modules for §8, §10–§12 and suites D/E/G/H): the
  brief's scope is the whole ISA, so these are deliverables 8–11, not exemptions. For milestone 1, record
  the staging explicitly in `PROGRESS.md` (what each will contain, which suite it serves, the order) and
  implement what the fixes for B07, B14, B16 and B17 need (certificate claims validation and composition,
  the library-backed borrow proof path). The full modules come after milestone 1.
- Fail fast and loud (rule 1): every reported outcome of the spec is a reported outcome of the crate; no
  `unwrap`/`expect`/`unreachable` on a path the spec defines; no thresholds or limits the spec does not
  state (B15's 1e-24, B14's 16 wires).

Write `reference-interpreter/council/response-round-1.md`: one entry per blocker id (all 30, grouped where
they coincide) with status `resolved` / `declined` (spec quote) / `partial` (what remains and why), what
changed (files, functions), the test that now pins it, and any SPEC-QUESTIONS entry it produced. This is
the editor's response the council reads next round; write it for a hostile reader.

Then: `cargo test --offline`, `cargo test --offline -- --ignored` (record times and case counts in
PROGRESS.md), `cargo clippy --offline --all-targets -- -D warnings`, `cargo fmt -- --check`, the Python
seed unchanged. Update `PROGRESS.md` and `README.md`. Commit. **Report back** (final message) with the
commit hash, the per-blocker status table, and anything declined or partial. The orchestrator then freezes
that commit for round 2; you will be asked to continue with deliverables 8–11 and the outstanding list
while the council works, so do not wait for it.

## Owned paths and process

- Write only under `reference-interpreter/` (and `bd note` / `bd create` in this repo for surprises; the
  job issue is b-isa-spec-zq2). Do not touch `spec/`, `docs/`, `worklog/`, `HANDOFF.md`, `CLAUDE.md`.
  Another job is writing `docs/notes/` concurrently; read-only for you.
- Commit your owned paths regularly, at least after every resolved blocker group:
  `git add -A -- reference-interpreter && git commit -m "..." -- reference-interpreter` (retry on
  index.lock). Never push.
- Rust standard library only, edition 2021, offline. No new dependencies.
- Headless (rule 11): run everything with `DISPLAY`, `WAYLAND_DISPLAY`, `DBUS_SESSION_BUS_ADDRESS`,
  `BROWSER` unset and `TMPDIR=$PWD/reference-interpreter/.tmp` (create it; it is not committed).
- Skepticism (rule 8): the council members are strong but not infallible, and so was the previous
  implementer. Derive expected values by hand before trusting any test's expectation, including the
  existing fixtures' comments.

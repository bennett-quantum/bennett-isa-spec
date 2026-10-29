# ISA 0.9 evidence job — running state

Job issue: `b-isa-spec-xrt` (already in progress). Output: `2026-10-29-isa-09-evidence.md` in this directory. Only these two files may be written. The runner commits; do not run git, push, edit the spec, interpreter, worklog, or other files. Commands must unset DISPLAY, WAYLAND_DISPLAY, DBUS_SESSION_BUS_ADDRESS, BROWSER and set TMPDIR to this repository. Beads notes are explicitly allowed; create issues only for defects found in ISA 0.8. Do not close implementation issues merely for completing this evidence review.

## Milestone 1 — source reading underway

Done: read beads skill, `CLAUDE.md`, `HANDOFF.md`, `spec/README.md`, all 1,675 lines of `spec/Bennett-ISA-Spec.md`, budget note, plan R1–R6 (and later risks), north star (all sections), and audit sections 0–1.B and 2–4. Read outline lines 1–800. Ran `bd prime` and `bd list --all --limit 0 --json`; the list has 39 issues, of which 37 are candidate/relevant issues (verify count when selecting). No file outside the allowed pair was deliberately changed.

Verified with: headless `cat`, `sed -n` source reads and `bd list`. Tool output can truncate at the orchestration layer: keep individual reads modest and set exec output budgets explicitly. Remaining source reads: outline 801–2062, SQLite REPORT 1–470, audit 236–300 (body rows/omissions), all matching `bd show` results, interpreter SPEC-QUESTIONS and read-only checker code as needed. Re-read any truncated HANDOFF portion only if relevant.

Findings requiring independent derivation:

- ISA §3.6 idle forward steps write HALT to old_pc; they are not literal identities on frames (budget note itself corrects its earlier broad claim). A controlled-commit variant needs a correct inverse when idle old_pc is zero; simply reusing the original inverse dispatch is wrong.
- Audit ISA-10 says inc8 is a direct envelope digest, while round-8 suggestion .11 asks to wrap trivial SCCs. Recompute the four published hashes independently before choosing wording.
- ISA-3: raw compute/swap/inverse-call expansion conflicts with §6.2 exact cleanup sequence. However a finite permutation gate decomposition may provide an existing construction; do not claim every construction forbidden solely from missing operands.
- Outline snapshot replay must preserve allocation order including dead/zero-sized allocations; `free` never reclaims and alloc/free history matters. Read G-12 carefully.
- R allocation is §25 BLK-4/new ISA-12, separate from dynamic widths/handle tables.

Next milestone: finish mandatory reads and every relevant `bd show`, then initialise evidence note with source/quote conventions, summary and detailed proposals. Keep capability versus direct-form/efficiency distinctions explicit; no narrowing. No tests or fixture files may be created outside this pair; use inline headless Python with bytecode disabled for mathematical/hash checks.

## Milestone 2 — mandatory reads complete; baseline verified

Read all remaining outline, SQLite REPORT and audit lines, plus `reference-interpreter/SPEC-QUESTIONS.md`. Ran `bd show ID --json` for all 38 matching issues (the prior estimated 37 was wrong; 40 total list entries, 38 relevant). Ran headless `python3 -B reference-interpreter/isa_check.py`: PASS, two example objects and both printed envelopes. Independent hashlib: inc8 direct digest matches 8ce5…be93; wrapped digest is 5f3e…8ee8, contradicting suggestion zsw.11. Recursive sum uses wrapped digest 74f1…f92 as published.

New findings: SQLite `region[25,8]` cannot fit H=18; positive null area and disjoint heap require H≥26, so the report's literal-step-machine cost table is not a conformant object. Its figures are profile-model estimates, not measured ISA budgets. E1: 1,214,248×128=155,423,744 bits. E2: 740×289,590=214,296,600 bits; with 10% margin 235,726,260, explaining the approximate 235M claim without inventing widths. Image padding is 61,440 bytes.

Filed two ISA-text defects (permitted exception to no-new-issues): SCC group ordering in §3.6 and zero-sized data spelling in §16. Record returned ids in the evidence note. Existing I8 and ISA-10 already cover row indices and acyclic hashing; no duplicate issues for those. Next: write the evidence sections and exact small proposals, verify math/canonical fragments using inline Python, append verdicts to all relevant issues, audit only the permitted files, and hand off to runner (no git).

## Milestone 3 — blockers, remaining ISA items and G-12–G-16 drafted

The evidence note now has the summary, conventions, four BLK sections, remaining ISA-2/5/6/7/8/9/10/11, and G-12 through G-16. New defect ids: `b-isa-spec-was` (SCC ordering), `b-isa-spec-oim` (empty data). Neither has been closed; proposals are evidence only.

Derivations to retain: runtime minting and parameterised qinplace are missing direct contracts, but finite enumeration/synthesis means absence of an opcode is not a universal impossibility proof. BLK-1's unbounded-instance-only claim also needs that qualification (recursion can retain fixed-width registers). G-15 has a simpler construction than the outline: k slice calls sharing the caller's fault register, plus a zero-payload fault-only call when k is even. No scratch-fault cleanup, hence no questionable exact-sequence exception. Each variant must have the identical fault/row-count prefix. G-12 replay must include dead and zero-sized allocations; concat cost has C−1 joins and log2 C levels.

Next: I1–I8 (I7 includes the 16 suggestion decisions), E1–E5, final ordered change list and new-defect exact wording; then independent inline verification and beads verdict notes. Fix a hand-count typo in ISA-6: printed measure_keep executes 4 instructions on m=0 and 6 on m=1 (not 5/7). Source §9.2 and layout incompatibility calculations already checked. All other proposed numeric claims still need the final independent audit.

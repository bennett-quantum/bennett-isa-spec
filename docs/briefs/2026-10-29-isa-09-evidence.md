# Job: evidence and derivations for ISA 0.9 (no spec edits)

Working directory: the git repository `bennett-isa-spec`. The ISA specification `spec/Bennett-ISA-Spec.md`
(draft 0.8) is council-accepted and **settled**: do not edit anything under `spec/`. Your job is the evidence
that ISA 0.9 will be built on, written as one note, `docs/notes/2026-10-29-isa-09-evidence.md`, with your
running state in `docs/notes/isa-09-evidence.PROGRESS.md`. Write only those two files.

Read first, in full: `CLAUDE.md` (rules 8 and 10 especially), `HANDOFF.md`, `spec/README.md`, the whole
specification, `docs/notes/2026-10-28-budgets-loops-variable-time.md`, the dialect outline
`../bennett-dialect-spec/spec/outline-0.2.md` (part B paragraph B-BLK and §25 give the blocker list; part C the
rung-by-rung check), the SQLite spike `../bennett-dialect-spec/docs/spikes/sqlite/REPORT.md` (items E1–E5), the
narrowing audit `../bennett-dialect-spec/docs/audit/2026-10-29-dialect-0.1-narrowing-audit.md`, the north star
`../bennett/docs/Bennett-NorthStar.md` (§3–§5, §8, §9), and the plan
`../bennett/docs/plan/2026-10-29-path-to-north-star.md` (risks R1–R6). The issue list is in beads: run
`bd list` and `bd show <id>` for every issue labelled `isa-0.9-candidate` or titled BLK-*, ISA-*, G-*, I1–I8.

## The question, item by item

For each of: BLK-1 (ISA-4), BLK-2 (ISA-1), BLK-3 (ISA-3), BLK-4 (new ISA-12), the remaining ISA-1..ISA-11
of dialect draft 0.1, G-12..G-16, I1..I8, and SQLite E1..E5, answer with derivations, not opinion:

1. **What the north star promises** (quote the clause) and which programs / rungs need it.
2. **Whether ISA 0.8 already realises it.** Quote the exact 0.8 text. If a construction exists on 0.8, write
   it out as a concrete object in the §16 canonical form (or a fragment of one) and argue its correctness;
   where the construction has a cost, derive the cost (steps, registers, budget) as a function of the inputs.
   If 0.8 truly lacks it, give the minimal counter-argument: the sentence of 0.8 that forbids every candidate
   realisation.
3. **For every genuine gap, the exact 0.9 change**: the new or altered normative sentences, the sections
   touched, the new instruction's frame/history/reversal rules, its fault codes, its well-formedness rules,
   its identity (§9.2) impact, the conformance-suite additions (which suite, what test, the expected result
   derived by hand), and whether it changes any published §16.3 hash. Keep each proposal to the smallest
   change that keeps the capability; never propose dropping or restricting a capability (CLAUDE.md rule 10).
4. **Semantic consequences you derive**, e.g.: the reversal of `qreg.new` inside an R closure and its
   cleanliness obligation; the interaction of a device-minted budget token with prefix reversal (T2) and
   with the step-machine proof obligations; `qreg[*]` binding checks versus static schedules; `qinplace`
   with parameters against the §6.1 transfer rule. Where a claimed consequence depends on a numerical rule,
   derive it; the council has caught every invented numerical rule within a round.
5. **A verdict**: `already-realised` (with the construction), `wording-only`, `efficiency` or `gap` (with
   the proposal), and which rung of the demo ladder, if any, depends on it.

Also classify the 16 council round-8 suggestions filed as beads `b-isa-spec-zsw.1` … `.16`: accept /
reject / defer, one line of reasoning each, and for accepted ones the exact edit.

## Output

`docs/notes/2026-10-29-isa-09-evidence.md`: one section per item in the order above, each ending with its
verdict line; a summary table at the top (item, verdict, rung dependency, hash impact); a final section
"Proposed ISA 0.9 change list" that orders the gaps by rung dependency and groups them into the fewest
council rounds. Mark every claim you could not verify as such. Add a `bd note <id>` to each issue with its
verdict line (allowed; create no new issues except for defects you find in the 0.8 text, which go in as
`bd create` with label `isa-0.9-candidate`).

## Constraints

- No edits under `spec/`, `reference-interpreter/` or anywhere except your two files. Another job is
  working in `reference-interpreter/` concurrently; you may read its code and `SPEC-QUESTIONS.md`.
- Commit your two files regularly (`git add <paths> && git commit -m "..." -- <paths>`; on index.lock wait
  and retry). Never push.
- Headless: no LLVM tools are needed for this job. If you run anything, run it with `DISPLAY`,
  `WAYLAND_DISPLAY`, `DBUS_SESSION_BUS_ADDRESS` and `BROWSER` unset and `TMPDIR` inside the repo.
- Keep `docs/notes/isa-09-evidence.PROGRESS.md` current: items done, items in progress, surprises. A fresh
  session may be started from it, so it must be enough to resume.

# bennett-isa-spec

## What this repo is

The virtual ISA contract: spec text, reference interpreter, formal model, conformance suites A-H as a crate, and C headers for the launch and scheduling ABIs. Depends on nothing; every other repo targets it.

## Where the spec is

Normative text: `spec/Bennett-ISA-Spec.md`, draft 0.8, council-accepted 2026-10-28 (see `spec/README.md` for the change rules). Versions in `SPEC_VERSIONS.toml`. `reference-interpreter/isa_check.py` is the Python seed of the reference interpreter and the spec's pre-flight check.

## Non-negotiables

0. **Worklog, kept by whoever commits the session.** Each working session that changes the repo leaves one `## Session log - YYYY-MM-DD - ...` block at the top of the highest-numbered chunk under `worklog/` (`ls worklog/ | sort -r | head -1`; past ~280 lines start the next `NNN_` chunk): what changed, gotchas, surprises and root causes. When an orchestrator runs subagents, the orchestrator writes it; subagents keep their job's progress file, put surprises in bd (`bd note` / `bd create`), and never need to write outside their owned paths. A session that changes nothing writes nothing.
1. **Fail fast, fail loud.** Assertions and explicit errors, never silent returns or quiet fallbacks.
2. **One implementer per change.** Code review is batched over accumulated diffs, not per change.
3. **Red-green TDD.** Write the test, watch it fail, make it pass, then refactor.
4. **Exhaustive verification** against the spec's conformance suite for the spec versions declared in `SPEC_VERSIONS.toml` (where present). "Runs without errors" is not a pass.
5. **Coupled to other repos ONLY through published specs.** Never share code across a contract boundary; if you need to, the boundary is wrong.
6. **NO GitHub CI, NO remote automation.** Quality gates are local: `scripts/pre-push` runs the tests (install with `scripts/install-hooks.sh`).
7. **Task tracking: beads (`bd`)**, initialised 2026-10-29 (`bd ready`, `bd show`, `bd create`; `bd export -o .beads/issues.jsonl` before committing `.beads/`). No TODO.md. No Dolt remote sync.
8. **Skepticism.** Verify and reproduce; distrust subagent output, earlier agents' claims and your own assumptions.
9. **Git policy (maintainer, 2026-10-29).** Any agent commits its own paths regularly (`git commit -- <paths>`; retry on `index.lock`); the orchestrator pushes after verified milestones and at least every couple of hours. This opts the repo into the Beads "team-maintainer" profile below for commits only, not for pushes or Dolt sync.
10. **No narrowing of north-star scope (maintainer, 2026-10-29).** Nothing may reject, refuse, fault, declare inert or omit a capability that `bennett/docs/Bennett-NorthStar.md` promises (thesis, §3–§5, every rung of the §8 demo ladder; Grover over unmodified SQLite is the core). A deferral may postpone mechanics only. If a contract truly lacks something, raise it as a blocker naming the exact change; never shrink scope to fit. See `bennett/docs/plan/2026-10-29-path-to-north-star.md`.
11. **Headless, no /tmp artefacts.** Never run LLVM `-view-*` / `view-cfg` / `view-dom` / `view-callgraph` passes or viewCFG-style helpers outside an interactive human session (use the `dot-*` variants). Batch and agent commands run with `DISPLAY`, `WAYLAND_DISPLAY`, `DBUS_SESSION_BUS_ADDRESS` and `BROWSER` unset and `TMPDIR` set to a project-local or job directory (2026-10-28: ~80,000 .dot files reached the maintainer's editor via xdg-open).


<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:1105d646 -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

**Architecture in one line:** issues live in a local Dolt DB; sync uses `refs/dolt/data` on your git remote; `.beads/issues.jsonl` is a passive export. See https://github.com/gastownhall/beads/blob/main/docs/core-concepts/sync-concepts.md for details and anti-patterns.

## Agent Context Profiles

The managed Beads block is task-tracking guidance, not permission to override repository, user, or orchestrator instructions.

- **Conservative (default)**: Use `bd` for task tracking. Do not run git commits, git pushes, or Dolt remote sync unless explicitly asked. At handoff, report changed files, validation, and suggested next commands.
- **Minimal**: Keep tool instruction files as pointers to `bd prime`; use the same conservative git policy unless active instructions say otherwise.
- **Team-maintainer**: Only when the repository explicitly opts in, agents may close beads, run quality gates, commit, and push as part of session close. A current "do not commit" or "do not push" instruction still wins.

## Session Completion

This protocol applies when ending a Beads implementation workflow. It is subordinate to explicit user, repository, and orchestrator instructions.

1. **File issues for remaining work** - Create beads for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **Handle git/sync by active profile**:
   ```bash
   # Conservative/minimal/default: report status and proposed commands; wait for approval.
   git status

   # Team-maintainer opt-in only, unless current instructions forbid it:
   git pull --rebase
   git push
   git status
   ```
5. **Hand off** - Summarize changes, validation, issue status, and any blocked sync/commit/push step

**Critical rules:**
- Explicit user or orchestrator instructions override this Beads block.
- Do not commit or push without clear authority from the active profile or the current user request.
- If a required sync or push is blocked, stop and report the exact command and error.
<!-- END BEADS INTEGRATION -->

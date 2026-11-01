# Current state — 2026-11-01 16:00:46 UTC

Reference ISA 0.9 migration, implementation passes 1–2 complete.
Start HEAD: 8e481ce (pushed). Own code commits: 4d1a1c2 and 3788db9.
This final gate/status/handoff record is committed next; the final report
supplies HEAD. No push, sibling writes, spec/catalog edits or remote sync.
Orchestrator retains session-worklog and council acceptance ownership.

Created b-isa-spec-fuk.1–.7; .1 and .2 CLOSED. Parent fuk stays OPEN.
.3–.6 are the requested remaining slices. .7 explicitly retains the rest of
the authoritative 0.8-to-0.9 delta; final documentation .6 depends on it too.
Resume at .3 (bound ref catalog expansion), after reading reference progress.
Do not claim complete ISA 0.9 execution or close the producer/rung-3 blocker.
README uses interim migration wording; its council line still names the
original accepted 0.8 snapshot. Final admits-both status waits for all slices
and the orchestrator runs the new milestone council round.

Slice 1 (4d1a1c2): explicit IsaVersion for coherent headers, ABI signatures,
closure envelopes, linked rendering, certificate versions, device model
versions and implicit ref binding. Reject mixed/unknown versions; never
silently upgrade linked inputs. Published 0.9 inc8 hash and execution pass.
Mandatory 0.9 lproc arrays are checked for the existing unforwarded form;
nonempty forwarding is still .7 work. Five version regressions added.
Full gates: reference 509/0/5, conformance 113/0/0, neutral 16/16, both Python
gates and strict lib/bins Clippy green. Logs: .tmp/migration-s1-*.

Slice 2 (3788db9): CC/R fault C, canonical uint64 validation, exact nonzero
codes and zero -> 4. 0.9 faults retain the failing program point and intact
instances/memory/tape/open activations; one terminal record, no result or
caller continuation. Eligible FAILED 0.9 run-program --reverse works.
0.8 launch restriction and Q zero-cause behavior remain unchanged.
Both Q engines and literal inverses use the versioned effective cause.
Eight new tests cover code bit boundaries, every integer division width
1..128, implicit causes, nested CC/R callers, CLI exact records through
UINT64_MAX, intact prefixes and full entry reversal. Every source-counter
cut 0..9 across a nested R call covers zero and both returns, plus direct
zero entry before invalid memory. Host fuel remains a resumable diagnostic;
§4.9 puts the R counter in producer instrumentation, not the Q machine.
Red logs: .tmp/migration-s2-red.log and migration-s2-atomic-red.log.
The R-only effect rule was preserved in tests by a CC wrapper calling R.

Final required gates all exit 0, headless/offline with repo-local TMPDIR:
- Reference: 517 passed, 0 failed, 5 unchanged ignored (504 + 5 + 8).
- Conformance: 113 passed, 0 failed, 0 ignored.
- python3 -I conformance/tests/draft_witnesses.py: PASS.
- python3 -I conformance/tools/ref_catalog.py --check: PASS; 219 recipes;
  envelope 5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197.
- Neutral suite: 16/16.
- Strict lib/bins Clippy, rustfmt and whitespace: PASS.
Logs: .tmp/migration-s2-{reference,conformance,witness,catalog,neutral,clippy}.log.
All 64 pre-existing reference fixture files, frozen ISA 0.8 and catalog bytes
match 8e481ce. No fixture weakening or catalog regeneration. No spec ambiguity
beads filed; .7 is implementation work, not a normative-text question.

Outstanding .7 examples: qreg[*]/ledger/handle and runtime forms, budget.mint
validity/undo, parametrised qinplace, lproc forwarding, and an exhaustive delta
audit. Confirmed E17.13: Memory::lifted_free and literal direct(lift.free)
still toggle matching pinned entries after checking for an unpinned match.
0.9 needs to exclude pinned entries, especially a zero-sized same-base global,
with version gating to retain 0.8. The exact finding is in .7 notes.
.3 also requires K77 oracle_invocation and byte-exact K78–K81 catalog vectors;
.4 needs actual 0.9 end-to-end ownership/addressing, .5 the full CLI/profile-v1
replay matrix. Do not confuse existing finite adapters with those deliverables.

Preserve pre-existing .tmp/ plus gate logs and scratch red tests. Expected
final git status: only ?? .tmp/. No pushes or sibling pin updates authorized.

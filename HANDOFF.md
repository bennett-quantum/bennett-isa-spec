# Current state — 2026-11-01 17:05:13 UTC

Reference ISA 0.9 migration: implementation pass 3, PARTIAL slice .3.
Start HEAD: 530b301. This code/records commit is made after all required gates;
its final HEAD is supplied in the final report. No push or remote sync.
The orchestrator retains session-worklog, council acceptance and sibling pins.

b-isa-spec-fuk.1/.2 remain CLOSED. .3 remains IN_PROGRESS; .4–.7 remain OPEN.
No new slice is closed. Resume .3 before starting .4. README still states
migration in progress and retains its prior council snapshot honestly.
Do not claim complete 0.9 execution or clear the producer/rung-3 blocker.

Landed in this pass:
- Production RefLibrary uses the conformance crate as a normal std-only
  dependency and checks the pinned canonical full catalog/envelope in Rust.
  A small conformance Expander::with_registry API reuses validated recipes
  with fresh registry/counters; no constructor semantics or catalog changes.
- Object::bound_ref_library resolves explicit/default 0.9 binding. K57's
  omitted expansion exposes the full-library hash without inserting metadata
  or changing schedule identity. 0.8 retains its previous default API.
- Immutable OracleInvocation captures a K77 record and positional actuals,
  checks the tuple against independently resolved source budget/layout and
  exact declared arity, preflights actuals, and invokes a supplied builder
  per capture. Source formals come from the linked oracle, with no extra token.
- Seven new tests: pins/defaults, actual catalog expansion, exact controlled
  leaf bytes and basis execution, B=2/B=3 inc8, malformed/source-inconsistent
  records, nested rejection/retry and actual validation before builder effects.
  K79–K81 existing byte locks now run against the checked complete catalog,
  rather than private modified recipe subsets; all original bytes are retained.

Why .3 is NOT closed:
- The unmodified published controlled-action-09/controlled.bisa still rejects
  certificate bindings.invocation (K13); K16 typed frames also remain legacy.
  Exact current-binary diagnostic: .tmp/migration-s3-fixture-red.log.
  New source/schedule tests explicitly exclude that certificate and do not
  claim complete fixture acceptance. Implement versioned admission/validation.
- A complete linked-source constructor builder, full invocation identity
  composition and K77 capture across arguments/body/template adapters, nested
  source calls and inverse traversal are still required. Finite callback inc8
  tests and the immutable capture boundary are not that general builder.
  Detailed findings/evidence are on existing .3/.7; no ambiguity bead was filed.

Full required gates all EXIT 0 (headless, offline Cargo, repo-local TMPDIR):
- Reference: 524 passed, 0 failed, 5 unchanged ignored (517 + seven new).
- Conformance: 113 passed, 0 failed, 0 ignored.
- python3 -I conformance/tests/draft_witnesses.py: PASS.
- python3 -I conformance/tools/ref_catalog.py --check: PASS, 219 recipes;
  envelope 5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197.
- Neutral suite: 16/16.
- Strict lib/bins Clippy, owned rustfmt and whitespace: PASS.
Targeted tests: 24/0. Logs: .tmp/migration-s3-{reference,conformance,witness,
catalog,neutral,clippy,targeted,library,preservation}.log.
Initial missing-API reds: migration-s3-red.log and migration-s3-context-red.log.

All 64 pre-existing reference fixtures, frozen ISA 0.8 and catalog bytes match
both 530b301 and pre-migration 8e481ce. All published controlled/K78–K81 fixture
bytes match 530b301. No spec, fixture, sibling, worklog or remote writes.
Only the Resume block of reference-interpreter/PROGRESS.md was updated; no
per-slice log block or record-only commit. Preserve existing .tmp/ and logs.

Remaining order: finish .3; .4 actual 0.9 K79–K81 ownership/addressing;
.5 run-program/run-cc/reverse and saved-tape profile-v1 0.9 matrix; .7 remaining
authoritative delta; .6 final admits-both documentation only after all close.
.7 still includes dynamic qregs/ledger/handles, budget.mint, parametrised
qinplace, nonempty lproc forwarding and the pinned lifted-free E17.13 fix in
both engines with explicit version dispatch. Newly noted K13/K16 admission
also blocks .3. Keep all 0.8 outputs/hashes and neutral 16/16 unchanged.

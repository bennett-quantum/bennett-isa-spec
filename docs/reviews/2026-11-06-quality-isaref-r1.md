# ISA reference interpreter: quality batch 1, review 1

**Verdict: BLOCK — two P0, one P1.** No implementation files changed.

Reviewed head: `f0c97a864bf7afe23c23371b4f94bcf2ed7c0661` (`job/quality-isaref`).
Batch range: `main..job/quality-isaref`, seven commits, `7518447..f0c97a8`.
At review start `main` was `e0424a99197e95057529eae25c176fa34491757a`;
its source equals the brief's `be109bd` (only the Beads export differs).
Also reviewed the inherited reference-interpreter changes in
`9825c42ad5c95cce9cd1419dcbcae0bfffab6e16..e0424a99197e95057529eae25c176fa34491757a`
(`origin/main..main`) affecting claims, replay, certificates, metadata and token aliases.
Findings below concern these inherited semantics as well as the batch's intended K4/K92 coverage.

## Blocking findings

**P0-1 — K92 certificate obligations are omitted. Bead `b-isa-spec-jple`.**
`reference-interpreter/src/claims.rs:874` returns claim reports without checking
the qinplace pair inventory, bound invocation applicability, or an unresolved
applicability outcome. Starting from `isa09_qinplace.rs::inverse_certificate()`,
remove the `addya` record's claims and `verification.inverse_pairs`: parsing
succeeds and `verify_certificate()` returns `Ok([])` despite its qinplace site.
Alternatively retain the correct pair and bind root invocation
`{"args":[["2","0","0","0"],"1"],"budget":["0","0","0","0"],"token_aliases":[0]}`.
The entry minimum is 3, but verification reports only `[Proof]`; execution
of that same root rejects `no covering inverse-pair entry`.
ISA 0.9 §12.3 K92 requires missing owner-local entries and uncovered bound
sites to reject, and unresolved applicability to be reported explicitly.
Execution-only checks do not discharge the checker obligation.
Ran `timeout 60 .tmp/rev-quality-isaref-r1/tmp/review-probe`: exit 0,
asserting both observations and the execution rejection.

**P0-2 — A library proof transfers to a different, noninverse pair. Bead `b-isa-spec-b6ds`.**
`reference-interpreter/src/claims.rs:807`–812 returns the referenced claim's
verdict without checking K4's same-pair side condition. Copy the fixture's
correct exhaustive `g/ginv` claim and entry into a linked `g` region record
(with its oracle signature and frame). Replace `addya` claim 0 with a
concrete `inverse_pair_library` claim referencing that record's claim 0,
but change the owner entry to `g/g`. Both public verification APIs report
`Proof`. For the published `g(x;y,a) = x+y+a mod 4`, at `x=y=0,a=1`,
`g(g(x))=2`, not 0. The exhaustive route correctly rejects this same pair.
ISA 0.9 §4.5 permits library reuse only for the same resolved g/ginv closures.
Ran `timeout 60 .tmp/rev-quality-isaref-r1/tmp/review-library-probe`: exit 0,
asserting the wrong `Proof` and observing exhaustive-route rejection.

**P1-3 — Valid linked-library pair coverage cannot execute. Bead `b-isa-spec-0dq0`.**
`reference-interpreter/src/claims/qinplace09.rs:131`–134 admits only
`exhaustive` / `expected:"inverse_pair"`. Keep the correct `g/ginv` entry
and linked exhaustive donor from P0-2; its owner-local
`inverse_pair_library` claim verifies as `Proof`. At the covered `tk=3,a=1`,
`execute_action_with_token_aliases("addya", …, &[0])` nevertheless rejects
`unestablished linked inverse-pair proof`. The original exhaustive fixture
executes successfully. This excludes the library route explicitly promised
by ISA 0.9 §4.5 and suite C, rather than merely leaving applicability unknown.
Ran `timeout 60 .tmp/rev-quality-isaref-r1/tmp/review-probe`: exit 0,
asserting that rejection.

The two probes were compiled from stdin with `timeout 120 rustc`, linked
against the freshly built review rlib, and included the existing
`tests/isa09_qinplace.rs` helpers. Their mutations affect certificate JSON
only; the independent arithmetic counterexample is given above.

## Verification and scope

All commands were headless, with private `.tmp/rev-quality-isaref-r1/{tmp,target}`
directories; Cargo runs had an 8 GiB address-space cap and core dumps disabled.

```sh
timeout 600 cargo test --offline -j3 --manifest-path reference-interpreter/Cargo.toml \
  --test isa09_certificate --test isa09_claim_replay --test isa09_token_aliases \
  --test isa09_qinplace --test r21_resources --test k91_vectors -- --test-threads=4
timeout 300 cargo test --offline -j3 --manifest-path reference-interpreter/Cargo.toml \
  --test isa09_forwarding -- --test-threads=4
git diff --check main...job/quality-isaref
```

Results: exit 0 for each; respectively 53 passed, then 11 passed / one
`slow-source-forwarding` ignore. No full crate suite, gate or sibling gate ran.
The wide slow witness was inspected, not rerun; its script supplies the memory
and wall-time caps and the gate invokes it explicitly.

The scheduled table/program/oracle regressions now return `Unverified` without
coverage; dangling/missing schedules reject. Reading `git show main:<path>`
confirms those scheduled-table regressions and the existing source-qinplace
expansion test catch the old paths. The r21 correction matches §3.3's qreg
storage rule. The exhaustive inverse-pair route already existed on main;
the added certificate assertions exercise it, not a newly failing baseline.
K91's concrete partitions, uniform admission, cross-partition rejection and
unchanged 0.8 grammar pass the focused tests. Correcting the old scheduled
`Proof` expectation preserves its full reference-table checks.

P2+ nits, **one bead `b-isa-spec-nivd`**: redundant/dead inverse-pair dispatch;
tracked pycache/log/progress litter; deferred long functions and production
test hooks; missing gate ratchet, stale `HANDOFF.md`, coverage and runtime target.
Programme items 9–10 were checked: the batch-1 implementer brief expressly
assigns them to batch 2. The reported default runtimes exceed 180 seconds.

Anti-slop: checked each COMMON rule. No new inconsistent state flags, silent
failure default, repeated compound predicate, semantic magic number, process
identifier, oracle-derived test expectation, test weakening or owned-path
violation found in the batch fix. The library dispatch and missing K92
validation above are the confirmed semantic failures; structural debt is in
the combined nit bead.

Coverage: read closely the complete batch source/test diff, `claims/replay09.rs`,
`pairs09.rs`, `qinplace09.rs`, `images09.rs`, claim execution/verdict routes,
certificate admission, token partition resolution, and K91 schedule binding
paths against §§4.5, 12.1–12.3; skimmed other source-path/tie-selection code,
fixture generators, CLI plumbing and broad inherited changes outside those paths.
Full conformance, target realisation, numerical/channel and unrelated interpreter
subsystems were not exhaustively reviewed.

Provenance: implementer **gpt-6.1-sol**; reviewer **gpt-6.1-sol, fresh session, D9**.
This BLOCK review does not claim council clearance.

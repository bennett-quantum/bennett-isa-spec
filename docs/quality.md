# Reference interpreter quality floor

Batch 1 (2026-11-06) fixes the round-2 certificate findings and the two
previously failing crate tests. It leaves the broader coverage, integration
binary consolidation, and source decomposition work for batch 2.

## Semantic checks

- A K13 subject with a schedule denotes the realised operation for table,
  program, and oracle relations alike. Until that operation is executed,
  verification returns `Unverified`, with no reference-only proof or coverage.
  Absent or mismatching subject schedules are admission errors.
- K4 inverse-pair claims use their owner-local entry, entry budget and argument
  domain. Both public verification APIs prove the published parameterised pair;
  noninverse pairs and absent entries reject. The entry-bound route was already
  present in the inherited WIP; batch 1 adds certificate-level regression coverage.
- Example 7 is loaded directly from the published spec. Its qinplace source
  adapters now use a collision-free lexical alias when constructing an internal
  ISA operand list; private constructor identities remain private. The source
  test checks both orientations, instruction allocation/release order, arithmetic
  results, preserved inputs, and complete reversal.
- The qreg handle/class-vector capacity check cites §3.3 (the register/type
  rules); dense quantum-state allocation checks retain §4.3.
- K91 schedule/certificate readers are checked for canonical partitions,
  required token-argument bindings, tuple-inconsistent equality, cross-partition
  certificate reuse, uniform claims, and rejection by the unchanged 0.8 grammar.
  The inherited K91 support passed these checks; no schema widening was needed.

The regression tests are in `isa09_certificate`, `isa09_qinplace`,
`r21_resources`, `isa09_token_aliases`, and the published `k91_vectors`.

## Bounded forwarding

The full-width materialisation-capacity witness is ignored in default tests
with the named reason `slow-source-forwarding`. The default run retains both
the complete published Example 10 literal domain (4096 inputs, two engines,
full reversal) and a compact source semantic witness. A separate 1024-leaf
capacity fixture exercises the host-resource diagnostic in the default run.

Run the wide capacity witness explicitly with:

```sh
sh reference-interpreter/tools/test-slow-source-forwarding.sh
```

The script enforces `ulimit -v 8388608`, disables core dumps, bounds wall time
to 600 seconds, uses one test thread, and reports elapsed time and peak RSS
with `/usr/bin/time`. The gate invokes this script separately from default
tests. Both default crate checks in the gate also enforce the 8 GiB address-space
cap. No unbounded full-width source materialisation is run.

Test profiles use optimisation level 1 and line-table debug information;
debug assertions and overflow checks retain their enabled test defaults.

## Measurements

The wide slow witness passed with **1,030,472 KiB peak RSS**: 34.38 s test
execution, 86.91 s wall time including a 52.49 s rebuild. The first bounded
forwarding run passed 11 tests with one named slow ignore in 40.02 s
(179.69 s including the initial profile build; 930,700 KiB peak RSS).

The commands below run from the repository root. They use a private target
directory, a project-local temporary directory, at most three compiler jobs
and four test threads. GNU time reports wall time including compilation and
the largest resident-set high-water mark among the command's children,
not a sum of simultaneous processes.

```sh
unset DISPLAY WAYLAND_DISPLAY DBUS_SESSION_BUS_ADDRESS BROWSER
export TMPDIR="$PWD/.tmp/quality-isaref/tmp"
export CARGO_TARGET_DIR="$PWD/.tmp/quality-isaref/target"
mkdir -p "$TMPDIR"
ulimit -v 8388608
ulimit -c 0
/usr/bin/time -v timeout 1800 cargo test --offline -j3 --manifest-path reference-interpreter/Cargo.toml --no-fail-fast -- --test-threads=4
python3 -I reference-interpreter/tools/publish-cli.py "$CARGO_TARGET_DIR/debug/bennett-isa-ref"
/usr/bin/time -v timeout 1800 cargo test --offline -j3 --manifest-path conformance/Cargo.toml --no-fail-fast -- --test-threads=4
```

Verified at 2026-11-06 07:33:36 UTC. Both complete default
commands above exited 0 under the 8 GiB address-space cap.

| Crate | Passed | Ignored | Wall (min:sec) | Peak RSS (KiB) | Exit |
|---|---:|---:|---:|---:|---:|
| reference-interpreter | 653 | 6 | 19:08.93 | 1,591,284 | 0 |
| conformance | 129 | 0 | 25:39.00 | 723,100 | 0 |

The reference final run reused compiled artifacts; the conformance measurement
includes its initial test build. These runs overlapped on a shared Linux host,
so the numbers describe the observed commands rather than isolated benchmarks.
The reference's six ignores comprise five existing exhaustive sweeps and the
new named slow forwarding witness. Conformance has no ignored tests.

The original qinplace and register-capacity failures were reproduced before
their fixes. The first whole reference run also exposed an older controlled-table
test that expected the false realised `Proof`; its final test now checks
`Unverified` for the unchanged scheduled fixture and proves the full table
through an explicit reference-only invocation. The final complete rerun passed.

Both crates also pass `cargo fmt --check` with their manifest paths. Interpreter
library/binary linting passed:
`cargo clippy --offline -j3 --manifest-path reference-interpreter/Cargo.toml --lib --bins -- -D warnings`.

The 180-second programme target is **not met** by this batch: conformance's
1,048,320-tuple call-interface test alone took 337.93 s. Runtime reduction is
tracked in `b-isa-spec-e923` for batch 2; the remaining `b-isa-spec-x6to`
nits are also outside the batch-1 correctness fixes. Coverage is not remeasured
in this batch. The complete repository gate is reserved for the orchestrator.


## Certificate review remediation

The follow-up keeps the standalone inverse-pair theorem distinct from executable
applicability. `ClaimReport.qinplace_applicability` records either `Established`
or `Unestablished(reason)` for a supplied qinplace owner. Static inventory includes
nested sites and zero-trip loop bodies. Concrete root and standalone claim
invocations check the transitive reference construction with each callee's actual
tokens and arguments; coherent predicates retain their obligations. Missing
owner-local pairs and uncovered bound sites reject through both public checking
routes. Zero-trip loops have no dynamic applicability visits.

Linked inverse-pair claims must resolve to an exhaustive pair theorem through an
acyclic library chain. Every link preserves the exact resolved closure pair,
minimum tuple, argument-token tuples, token partition and proved argument domain.
The runtime accepts this library route in both orientations. The published
Example 7 arithmetic and complete reversal are checked over every x, y and a at
primary budgets 3 and 9, using an exhaustive donor with no domain on the library
claim itself. A missing library body retains an unverified theorem and explicit
unestablished applicability; execution rejects it before effects. Scheduled
realisation trees and unsupported bound root kinds also remain explicitly
unestablished rather than inheriting a reference-only acceptance.

Five review regressions were run against an archived `b572e17` source tree before
the implementation: missing owner inventory, uncovered bound minimum, omitted
unestablished outcome, proof transfer to g/g, and valid library execution each
failed for its intended semantic reason. The final suite also covers nested
sites, zero-trip inventory, disabled controls, transitive owners, donor budget and
argument-domain preservation, and missing library bodies. No published spec or
fixture bytes changed.


The remediation focused check passed **58 tests with one existing ignore**:

```sh
timeout 600 cargo test --offline -j3 --manifest-path reference-interpreter/Cargo.toml --test isa09_qinplace --test isa09_claim_replay --test isa09_certificate --test r1_certificates --test r1_qinplace -- --test-threads=4
```

Both crates passed `cargo fmt --manifest-path <crate>/Cargo.toml -- --check`
and strict library/binary linting with
`timeout 600 cargo clippy --offline -j3 --manifest-path <crate>/Cargo.toml --lib --bins -- -D warnings`.
The reference default command documented above was run once after the source
changes, with the same environment and caps: **664 passed, six existing ignores,
exit 0, 21:58.05 wall, 1,592,024 KiB peak RSS**. The freshly built CLI was then
published to the stable path inside this worktree for the conformance run.
The conformance default command also passed once after publication, with no
source changes between either complete run. Runtime and wider quality work remain
outside this remediation.


Final remediation validation recorded at 2026-11-07 06:23:46 UTC.
Both exact default commands are listed above; each used `ulimit -v 8388608`,
`timeout 1800`, three compiler jobs and four test threads.

| Crate | Passed | Ignored | Wall (min:sec) | Peak RSS (KiB) | Exit |
|---|---:|---:|---:|---:|---:|
| reference-interpreter | 664 | 6 | 21:58.05 | 1,592,024 | 0 |
| conformance | 129 | 0 | 16:52.64 | 723,256 | 0 |

The reference run includes rebuilding affected binaries; conformance reused its
compiled artifacts. They ran sequentially on a shared host. The 180-second
programme target remains open for batch 2; these measurements do not claim it
is met. No full repository or sibling gate was run for this remediation.

## Bound-root obligation reporting

Certificate checking now returns `CertificateReport`: its `claims` field retains
standalone claim verdicts, while `qinplace_applicability` records obligations by
owner or bound root closure independently of the claim collection. Callers that
previously consumed the returned claim vector select `.claims`. A retained
standalone `Proof` does not establish executable acceptance when the report
contains an unestablished applicability obligation.

Unsupported bound root kinds retain their obligation even when no supplied
record directly owns qinplace, all claims and inverse-pair entries are removed,
or the region map is empty. The same rule applies to unchecked scheduled trees,
parameter-uniform invocations and host-resource-limited traversal. Unresolved
contexts take precedence over checked contexts of the same closure. Concrete
reference traversal still rejects missing owner-local entries and uncovered
sites; unbound direct owners retain their separate unestablished outcome.

The attachment audit covered inventory owners, concrete roots, standalone claim
invocations, uniform invocations, scheduled traversal and work-limit exits, and
the final claim-report construction. Obligations no longer require a discovered
owner, claim, record or inverse-pair entry to reach the certificate report.
Three empty-collection variants and an unchecked uniform transitive claim were
reproduced as semantic failures before the fix. Additional regressions cover an
unchecked scheduled root with no reached owners or claims and a concrete check
that cannot discharge a separate uniform obligation. No published spec or
fixture bytes changed.


Bound-root focused validation passed **64 tests with one existing ignore**, using
the focused command above. Both crates passed the same formatting and strict
library/binary Clippy commands, and all reference test binaries compiled with
`timeout 600 cargo test --offline -j3 --manifest-path reference-interpreter/Cargo.toml --no-run`.

The full default reference command above ran once after the source was frozen:
**670 passed, six existing ignores, exit 0, 7:00.81 wall, 1,591,540 KiB peak RSS**.
This run reused the compiled test artifacts and retained the 8 GiB address-space
cap. Reference results recorded at 2026-11-07 06:52:55 UTC. The completed CLI was atomically
published inside this worktree before starting the conformance command.


Complete bound-root validation recorded at 2026-11-07 07:04:00 UTC. Both default commands
listed above ran once, sequentially, with `ulimit -v 8388608`, `timeout 1800`,
three compiler jobs and four test threads. Source stayed unchanged between runs.

| Crate | Passed | Ignored | Wall (min:sec) | Peak RSS (KiB) | Exit |
|---|---:|---:|---:|---:|---:|
| reference-interpreter | 670 | 6 | 7:00.81 | 1,591,540 | 0 |
| conformance | 129 | 0 | 11:27.10 | 727,476 | 0 |

Both runs reused compiled test artifacts on the shared host. The reference's
six ignores are unchanged; conformance has none. The 180-second target, coverage
measurement and remaining structural quality work are still deferred to batch 2.
No full repository or sibling gate ran.

## Council blocker regressions

The multi-entry inverse-pair regressions share claim 0 across pairs whose
classical argument names occur in different positional orders. Both the identity
probe and the four-row addition/subtraction probe check both concrete bindings,
entry orders, pair orientations and direct/linked-library routes. The selected
owner entry and terminal donor entry retain their own argument-domain mapping.
Covered bindings establish applicability and execute/adjoint exactly; uncovered
bindings reject through both public checker APIs and leave the complete
`QuantumState` unchanged. CC/R wrappers mint their token and fault with code 4
before quantum, memory, clock or tape effects; whole-run reversal restores entry.

Pinned imports with and without any local qinplace retain an independent bound
root `Unestablished` obligation naming the missing linked body. Both fixtures
parse and pass identity checks without panicking at verification; linking the
matching body with its owner-local inverse-pair entry establishes applicability.
Scheduled table/program relations reject conflicting subject closures before the
staged `Unverified` result; valid scheduled subjects still have no coverage.

Before implementation, production sources matched `8056ecf` (the later branch
commits changed review documents only). Command:

```sh
timeout 600 cargo test --offline -j3 --manifest-path reference-interpreter/Cargo.toml --test isa09_qinplace --test isa09_certificate --test isa09_budget_mint --no-fail-fast -- --test-threads=4
```

It exited 101 with six intended regression failures: wrong per-entry domain,
post-effect CC/R cleanliness error, unresolved-import panic, missing unresolved
root obligation, conflicting scheduled subject acceptance and revoked token use.
The two council test files subsequently passed 37 tests with:

```sh
timeout 600 cargo test --offline -j3 --manifest-path reference-interpreter/Cargo.toml --test isa09_qinplace --test isa09_certificate -- --test-threads=4
```

Logs are private job artifacts (`c1-baseline-red.log`, `c1-council-focused.log`).
The existing source lexical-alias regression passed in both runs. All runs are
headless with the job's private TMPDIR/CARGO_TARGET_DIR, an 8 GiB address-space
cap, disabled core dumps and four test threads. Full crate validation is recorded below; these focused results do not claim a
full gate.

Device mint handles now consult their owning run's published active mint frames
at the shared invocation-token resolver. Reversed frames and observer rollback
revoke escaped handles; closing the owning machine expires them. Both
`run_oracle_with_token` and `OracleMachine::new_with_token` reject these handles
before execution, while live tokens still run the one-row identity and nested
CC/R region calls. The mint suite passes all 13 tests:

```sh
timeout 600 cargo test --offline -j3 --manifest-path reference-interpreter/Cargo.toml --test isa09_budget_mint -- --test-threads=4
```

The added closed-run and extended observer-rollback checks also failed before the
token fix (three intended mint failures in `c1-token-red.log`) and passed after
it (`c1-token-green.log`).

The final three test files were also run against an archive of exact `8056ecf`:

```sh
timeout 600 cargo test --offline -j3 --manifest-path .tmp/quality-isaref/tmp/c1-baseline-8056ecf/reference-interpreter/Cargo.toml --test isa09_qinplace --test isa09_certificate --test isa09_budget_mint --no-fail-fast -- --test-threads=4
```

This exited 101 with all eight intended failures (four qinplace, one subject
binding, three mint validity), recorded in `c1-8056ecf-red.log`. The archive
retained baseline production sources and used the final regression fixtures.

At code head `7fc37fd`, the combined focused check passed **119 tests, two existing
ignores**, exit 0, **58.23 s wall**, **960108 KiB maximum RSS**:

```sh
timeout 900 cargo test --offline -j3 --manifest-path reference-interpreter/Cargo.toml --test isa09_qinplace --test isa09_claim_replay --test isa09_certificate --test isa09_token_aliases --test isa09_forwarding --test isa09_budget_mint --test r1_certificates --test r21_resources --test r22_coverage -- --test-threads=4
```

Both crates also passed `timeout 120 cargo fmt --manifest-path <crate>/Cargo.toml
-- --check` and `timeout 600 cargo clippy --offline -j3 --manifest-path
<crate>/Cargo.toml --lib --bins -- -D warnings`.

Full crate validation completed at **2026-11-07 08:10:22 UTC**, on the unchanged
production code from `7fc37fd` (run head `d3d906d` adds evidence only). Both full
default commands ran sequentially under the same headless 8 GiB cap:

```sh
timeout 2400 cargo test --offline -j3 --manifest-path reference-interpreter/Cargo.toml -- --test-threads=4
BENNETT_CONFORMANCE_BIN="$CARGO_TARGET_DIR/debug/bennett-isa-ref" timeout 2400 cargo test --offline -j3 --manifest-path conformance/Cargo.toml -- --test-threads=4
```

- Reference: **677 passed, six existing ignores**, exit 0, **5:28.36 wall**,
  **1591476 KiB maximum RSS**.
- Conformance: **129 passed, no ignores**, exit 0, **9:08.00 wall**,
  **722896 KiB maximum RSS**. The selected private interpreter binary contains
  the mint-validity fix; its SHA-256 is retained with the run evidence.

The named lexical-alias check also passed on its own (one test, exit 0):

```sh
timeout 180 cargo test --offline -j3 --manifest-path reference-interpreter/Cargo.toml --test isa09_qinplace source_qinplace_adjoints_preserve_instruction_allocation_and_release_order -- --test-threads=4
```

Completed logs, statuses and measurements are private artifacts named
`c1-full-reference.*`, `c1-full-conformance.*`, `c1-conformance-binary.sha256` and
`c1-alias.log`. No full repository gate, gate tool or sibling gate ran. The
180-second runtime target and the broader quality programme remain open; this
validation covers the scoped semantic fixes.

## Missing-body rejection ordering

K92's owner inventory is checked before any unavailable-body outcome. Local
closure identities come from `identity.closure_hashes`; imported identities come
from their §9.3 pins. An exact pair in either orientation is required even without
`bindings.invocation`. Missing entries and references to another claim rule
reject; unavailable pair bodies defer proof replay and retain `Unestablished`
applicability. The imported witness is preserved verbatim in
`reference-interpreter/tests/fixtures/imported-qinplace-missing-inventory.bisa`.
Entry and argument-token budgets and alias partition structure also validate
before unavailable bodies or library donors can defer theorem replay. Formal
token-argument names and positional tuple comparisons still require the body.

The class sweep examined every added `continue`, `return` and `break` in
`d46df99` and `7fc37fd`, including relocated exits and changed guard conditions:

| Location / exit | Mandatory rejection ordering and regression |
| --- | --- |
| `certificate.rs::validate_claim`, subject-conflict error | Already rejects before replay; `scheduled_and_reference_relations_reject_conflicting_subject_closures` retains the scheduled/reference table/program matrix. |
| `pairs09.rs::inverse_pair_theorem09`, missing donor `Ok(None)` | Another ordering defect: a parameter-uniform claim could hide malformed local owner `arg_tokens` or `token_aliases`. Local pair context now validates first. `unavailable_library_still_rejects_malformed_owner_pair_context` and `unavailable_library_still_rejects_malformed_owner_alias_context` check both public checker APIs. Pinned pair bodies may remain unavailable; undeclared pair closures reject. |
| Same function, first covering donor `break` and no-covering-donor error | The break selects an alternative theorem binding; the error is a terminal rejection. Existing `library_proof_cannot_transfer_to_a_different_pair`, `library_reuse_preserves_the_proved_minimum_and_argument_domain`, and shared-claim entry-order/orientation matrices retain pair/context rejection and valid donor controls. |
| `qinplace09.rs::check_qinplace_bindings09`, no-work guard and missing-pair-body `continue` | No-work requires neither imports nor local sites. The defective missing-body skip now follows exact pair presence, valid claim index and theorem-rule checks. New `imported_pair_requires_owner_inventory_before_missing_body_outcome`, `imported_pair_requires_static_owner_inventory_without_invocation`, `imported_pair_rejects_invalid_referenced_claim_before_missing_body_outcome`, and `local_pair_requires_owner_inventory_with_and_without_invocation` cover the rejections. `imported_pair_inventory_resolves_pins_in_either_orientation_without_proof_replay` retains valid inventory without inventing a proof. |
| Same missing-pair-body and missing-donor exits, independent entry context | `imported_pair_rejects_invalid_entry_budget_before_missing_body_outcome` checks both the entry and an argument-token budget; `imported_pair_rejects_invalid_alias_structure_before_missing_body_outcome` checks alias arity and label gaps. Both checker APIs reject before returning unavailable-body/donor outcomes. |
| `qinplace09.rs::qinplace_coverage09`, no-work guard / missing callee body `continue` | Every supplied owner's static inventory is checked before invocation traversal. `unresolved_action_callee_cannot_hide_another_supplied_owners_missing_inventory` adds the ordering control. Both existing `pinned_import_*_local_qinplace_keeps_unresolved_root_obligation` tests retain absent-body outcomes without panic and linked positive controls. |
| Same function, oracle missing-import `continue` | `unresolved_oracle_body_cannot_hide_supplied_owner_inventory` requires the visible owner's missing-pair rejection first; removing that record leaves the root `Unestablished`, naming the unavailable import. |
| Same function, reached qinplace missing-pair-body `continue` | The new imported-pair rejection and theorem-rule tests bind a concrete root invocation; static obligations reject before this branch. Existing unresolved-root tests and the valid imported-pair inventory control retain unavailable-body outcomes. Execution still rejects unestablished applicability before effects. |
| `tokens.rs::BudgetToken::mint_is_live`, expired owner `Ok(false)`; `resolve_invocation_tokens`, revoked-token error | The false result feeds an immediate K3 rejection, before token partition handling or effects. Existing `device_tokens_expire_when_their_owning_run_is_dropped`, `reversed_mint_tokens_reject_both_public_oracle_entry_paths`, and observer-rollback tests exercise both public entry paths and live-token controls. |

`claims.rs` and `classical.rs` add no early exits in those commits. Existing
zero-budget, zero-trip, predicate and argument-domain tests remain in the focused
suite; their pre-existing traversal skips were not removed or weakened.

The final regression file was copied into a private archive of exact `5b71208`
with production sources unchanged. This command exited **101**, with **35 passed
and seven intended failures** (imported bound/static inventory, invalid referenced
claim, imported invalid budgets/alias structure, malformed local token arguments
and aliases hidden by a missing donor):

```sh
timeout 600 cargo test --offline -j3 --manifest-path .tmp/quality-isaref/tmp/c2-baseline-5b71208/reference-interpreter/Cargo.toml --test isa09_qinplace -- --test-threads=4
```

The baseline result is retained in private `c2-baseline-red.log`, `.status` and
`.time` artifacts. Fixture-construction errors encountered before that final run
were corrected and are not counted as production regressions.

The corrected implementation passes **77 focused tests**, exit 0, **21.13 s
wall**, **203964 KiB maximum RSS** (`c2-focused.log`, `.status`, `.time`):

```sh
timeout 900 cargo test --offline -j3 --manifest-path reference-interpreter/Cargo.toml --test isa09_qinplace --test isa09_claim_replay --test isa09_certificate --test isa09_budget_mint --test r1_fixtures -- --test-threads=4
```

Both crates pass `timeout 120 cargo fmt --manifest-path <crate>/Cargo.toml --
--check`, and `git diff --check` passes. These runs use the same headless private
TMPDIR/target, 8 GiB address-space cap and four test threads as above.
Both crates also pass `timeout 600 cargo clippy --offline -j3 --manifest-path
<crate>/Cargo.toml --lib --bins -- -D warnings`.

The first full reference run exposed the new fixture's required audit integration:
its `.comments` companion and literal interface pin were missing. Both were added
without weakening the audit; all seven `r1_fixtures` tests pass in the focused
command above. The failed run is retained as `c2-fixture-audit-red.*`; it is not
counted as a completed full validation.

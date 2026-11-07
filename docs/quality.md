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

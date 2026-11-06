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

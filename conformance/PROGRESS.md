# Conformance crate verification — 2026-10-29

Resumed implementation bead **b-isa-spec-t8i** after commit `81cd4ba`.
The brief is `docs/briefs/2026-10-29-conformance-crate.md`. Owned edits are
confined to `conformance/`, `include/`, and the Beads export. No interpreter
source, spec, worklog, or other agent's edits were changed; nothing was pushed.

## Results against the reference binary

Binary: `/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/target/debug/bennett-isa-ref`.
Latest reference-interpreter commit: `2e61144` (report-only; offline build below
confirmed Cargo's executable was current). Binary SHA-256:
`d9e9c8cd86b2ccae1d56624e2950c4ea2f33e4f1952a2b0bf74ec058d795f6c5`.

| Suite | Cases | Pass | Fail | Unavailable |
|---|---:|---:|---:|---:|
| A — classical execution/history | 21 | 21 | 0 | 0 |
| B — oracle totality/reversal | 35 | 35 | 0 | 0 |
| C — permutations/agreement/examples | 339 | 339 | 0 | 0 |
| D — unitary/channel comparison | 12 | 7 | 0 | 5 |
| E — targets | 15 | 0 | 0 | 15 |
| F — producer validation/identity | 35 | 33 | 0 | 2 |
| G — launch | 23 | 10 | 0 | 13 |
| H — scheduling | 10 | 0 | 0 | 10 |
| **Total** | **490** | **445** | **0** | **45** |

The standalone harness exited **2**, not success: every staged case prints
`UNAVAILABLE <id>: unavailable: deliverable N`. These outcomes are counted,
never skipped. Deliverables: **8** full stateful C launch/library surfaces,
**9** target evaluation, **10** scheduling consumer, **11** rigorous certificate
checker. Suite F's two unavailable cases require linking and frame-descriptor
checking surfaces absent from the CLI; this does not claim that the
interpreter library lacks all linking support. Target and schedule probe
commands confirmed the loud staged diagnostics. Every staged case also has
structured `scenario` and `expected_outcome` data plus its spec derivation;
semantic outcomes are not claimed executed while their CLI surface is absent.

`cargo test`: **18 passed, 0 failed, 0 ignored** (including the whole-reference
integration test). That integration test asserts available cases and reports
unavailable outcomes; the standalone command's exit 2 remains the full-suite
status. Header compilation: **3/3 passed**, each header alone (included twice)
and both together, using `cc -std=c11 -Wall -Werror`. Strict Clippy passed.

## Exact commands

All validation was headless, with the four variables removed and the absolute
project-local TMPDIR below. Commands were run in this order for the final
verification; shell redirection only saved output under `conformance/.tmp/`.

```sh
mkdir -p /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/.tmp
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/.tmp cargo build --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/.tmp cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/Cargo.toml > /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/.tmp/cargo-test.log 2>&1
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/.tmp cargo run --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/Cargo.toml -- --bin /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/target/debug/bennett-isa-ref > /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/.tmp/full-suite.log 2>&1
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/.tmp cargo clippy --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/Cargo.toml --all-targets -- -D warnings
```

The build, tests, and Clippy exited 0; the harness exited 2 for unavailable work.
`cargo fmt --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/Cargo.toml`
ran before the final tests. `git diff --check` passed.

## Acceptance audit and red-green evidence

- All eight suites from §15 have fixture cases. Baseline was 182 cases:
  148 pass, 0 fail, 34 unavailable. Added 308 cases, with no expected value
  changed to match a binary's answer.
- A: classical loop, new recursive `rsum(3)=6` and its full reversal (28
  instructions), effects, hostcall, fault records, nested/impure/measurement
  uncall, all qreg families and Born outcomes.
- B: every literal fault code 0–10, B=0, exact/under-budget halts, recursive
  capture and return prefix reversal, depth/stack limits, inherent memory
  checks, allocator pre-entries and pinned free, nested oracle cleanup.
- C: the §16.3 objects/hashes, inc8 wrap and budget boundaries, sum_db n=0,
  3, 16, 17, stack 271 versus 272; arithmetic and lifted rows, quantum
  memory, actions and adjoints. Added independent complete truth tables:
  256 tuples for inc2 (all input/target/fault entries), 16 for flip, 4 for
  qinplace, 8 for the borrow frame with both borrowed entry values exposed.
  The frame uses the exact four gates of the actual qborrow fixture, so its
  basis permutation establishes independence by T3 and linearity; separate
  cases exercise the qborrow lifetime path. Engine-agreement/prefix checks
  remain separate from these independently derived expected tuples.
- D: added two H-controlled(-I)-H interference witnesses, which require
  retention of the callee's global phase. The first fixture draft used an
  indexed name in an operand list, rejected correctly under §16.2; separate
  named interface operands fixed the fixture without changing its math.
  Numerical/channel obligations, including a nonpositive-pivot rejection,
  remain explicit deliverable-11 cases.
- E/H: target AST errors, matching, expansion versus scheme correctness,
  aggregation with explicit live intervals, overlap/timing, imports,
  capacity, specificity, reproduction, unsupported and verbatim rejection
  rules exist as staged scenario/expected data and probes.
- F: connected the abandoned version/name-resolution/permutation/window
  fixtures; added adjoint, canonical index-space and independently computed
  fixture hash checks. Existing row classes, schemas, disjointness, bounds,
  four negative borrow-frame rules, grammar, envelope and linking cases stay.
- G: covered one-shot launch results/faults, positional arguments, host reply,
  completion/reversal/close and reverse refusals; added stateful scenario data
  for all four argument kinds, suspend/resume, accessors/free ownership,
  register exclusivity, result encodings, with existing tape/estimate/coverage,
  wait, unload and tree-reversal scenarios. The remaining C API is deliverable 8.
- Red: `acceptance_rule_witnesses_exist` failed at missing A-recursion and
  `small_t3_domains_are_exhaustive` failed at 0/256 oracle tuples. Green after
  the fixture additions. Staged-data integrity failed at missing scenario
  data, then passed after adding structured inputs and expected outcomes.
- Red: empty `--cases` incorrectly exited 0, and unavailable diagnostics did
  not contain the requested `unavailable: deliverable N`; both CLI tests now
  pass. Red: the JSON parser accepted `01`; it now rejects noncanonical
  number grammar, nonfinite numeric output, unescaped controls and malformed
  surrogate pairs. Existing comparator and fake-binary rejection tests pass.

## Corrections to inherited staged expectations

These came from re-reading the spec, before any executable staged verdict:
§11 specificity is **lexicographic**, so (1,1,0,0,0) beats (1,0,1,0,0);
a component-wise incomparability assertion was wrong. Interval arithmetic
need not derive a zero bound for a Bell circuit; the numerical zero-bound
fixture now uses exact rational X matrices. Upper error bounds alone do not
prove a tighter claim must reject. A full-D oracle proof includes arbitrary
target/fault entries, not just inputs. Aggregation requires explicit live
intervals. Resolving a single unpinned import is not a duplicate-definition
conflict. These corrections retain each spec citation and hand derivation.

## Headers and Beads

Kept the reviewed one-line comment corrections only:
`bennett_launch.h`: section 10, launch 0.8;
`bennett_sched.h`: section 11, sched 0.8.
No declarations, types, layouts, or ABI behavior changed; both headers still
contain only guards and comments, so compilation does not claim ABI delivery.

No reference-interpreter disagreement with a hand-derived expected value
remained; **no new bug beads were created**. Existing implementation bead
`b-isa-spec-t8i` records the audit and completion. Pre-existing specification
questions (including fault 0 and empty data text) remain referenced by their
original cases. Beads were exported before committing the owned paths.

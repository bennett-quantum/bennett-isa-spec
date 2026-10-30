# VM hsd/clw contract-gap witnesses — 2026-10-30

Added two 0.8 cases, each explicitly **unavailable pending 0.9**:
`G-estimate-resolved-class-decimal` names b-isa-spec-c15 / bennett-vm-hsd;
`D-sampled-relation-invocation` names b-isa-spec-9uy / bennett-vm-clw.
Their derivations cite the accepted spec's omissions; expected outcomes
separate the gap from proposed 0.9 witnesses. Neither has an invented
CLI command or claims a 0.9 implementation pass.

Extended the existing independent draft witness test. RED: the test failed
at the absent LAUNCH-ESTIMATE witness before draft additions. GREEN:
Fraction/Decimal independently reconstruct scheduled per-class peaks and
occupancy, check zero rows and escaped class keys, enclose 1/3 at 34 digits,
and check exact decimals, decade carry and a >34-digit integer. Full tuple
XOR checks distinguish the sampled relation from identity and resolve the
argument/budget witnesses, including unchanged dirty/cell components.
Singleton replay counts one distinct sample out of 2048 full-domain tuples
and remains evidence. A separate typed parameter hash checks i128, f64 −0
and token tuples. No VM/interpreter code is imported or copied.

Headless offline Rust gate: **25 passed, 0 failed/ignored**. Full harness:
**445 pass, 0 fail, 48 unavailable, 493 total**, exit **2**. D now has
13 cases (7 pass, 6 unavailable), G has 24 (10 pass, 14 unavailable);
all other suite counts match the historical table below. Two new
UNAVAILABLEs are counted, not skipped or promoted. Observed reference
binary SHA-256 (not rebuilt; council work is off limits):
`2e67862edb87abc932ce6d10d9609c3b154c64fd1103dd364924958519b0546a`.

Commands (absolute paths, graphical environment unset, project-local TMPDIR):

```sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/.tmp cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/Cargo.toml
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/.tmp /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/target/debug/bennett-isa-conformance --bin /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/target/debug/bennett-isa-ref > /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/.tmp/hsd-clw-suite.log 2>&1
git -C /home/tobias/Projects/bennett-workspace/bennett-isa-spec diff --check
sha256sum /home/tobias/Projects/bennett-workspace/bennett-isa-spec/spec/Bennett-ISA-Spec.md
```

Accepted 0.8 SHA-256 unchanged:
`90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e`.
Draft K12/K13 and the maintainer alternatives are in
`spec/isa-0.9.PROGRESS.md`; both bug mirrors remain open. The following
record is historical.

# VM milestone 4 follow-up verification — 2026-10-30

Resumed the inherited uncommitted work after the infrastructure outage.
Owned paths only: `conformance/`, the 0.9 draft/progress, `include/README.md`
and the Beads export. No reference-interpreter or VM source, accepted 0.8,
header declarations or worklog was edited. No push or Dolt remote sync.

## Current reference results

Binary: `/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/target/debug/bennett-isa-ref`.
Offline build confirmed current; latest reference source commit `8fa5d98`.
Binary SHA-256:
`679de84f8c016ac2626a2eca2c262129d204ae9f7723cec00a92cead0f4beff5`.

| Suite | Cases | Pass | Fail | Unavailable |
|---|---:|---:|---:|---:|
| A | 21 | 21 | 0 | 0 |
| B | 35 | 35 | 0 | 0 |
| C | 339 | 339 | 0 | 0 |
| D | 12 | 7 | 0 | 5 |
| E | 16 | 0 | 0 | 16 |
| F | 35 | 33 | 0 | 2 |
| G | 23 | 10 | 0 | 13 |
| H | 10 | 0 | 0 | 10 |
| **Total** | **491** | **445** | **0** | **46** |

Standalone exit **2**: unavailable cases are counted and printed, never
skipped or counted as passes. The extra case is the separate valid exact
entry witness. `cargo test --offline`: **25 passed, 0 failed, 0 ignored**;
strict all-target Clippy and formatting checks pass. The current compiler
flagged a pre-existing `manual_strip` in `tests/headers.rs`; using
`strip_prefix` fixed it, and all five header tests passed again. This changes
no declaration or inventory check.

## Re-derived fixture corrections and staging policy

0.8 §8.1 says: **"A target is a JSON document in RFC 8785 canonical form;
target_hash = SHA-256 of those bytes"**. Removed the trailing LF from all five
E `target.json` files (hash, exact-entry negative, matching, mbu and probe).
The old hash fixture bytes hashed to
`52cd3dcd6ca4634f311f4ba0b15806c52763e70504f1220ef5c2d64d10676ecb`;
canonical bytes hash to
`71e1973c642abe3ea4ae5b4c7211fd4e2b2b7817894ca958d138c992cc321596`.
The existing expected hash was already the latter; no expected hash was
changed to match a binary. Contrary to the VM report, it did not include LF.
The E probe uses a separate `noncanonical.json`, exactly canonical bytes
plus LF, and its primary expected outcome is the §8.1 rejection.

0.8 §8.1 also says: **"an entry with epsilon 0 MUST NOT reference eps or
log2inv_eps"**, and **"an entry with epsilon 0 is exact and matches every
eps"**. `E-epsilon-zero-entry` now actually references `eps` in `cost.time`
and must reject. New `E-epsilon-zero-exact` retains constant cost 1, literal
epsilon 0, and must load/match at shares 0, 0.001 and 1. The accepted rules
and precise semantic `expected_outcome` data are unchanged.

Only the two explicit staged E/H probes have `unavailable.expect`: the
precise §8.1 canonical-byte error / §11 `E_SCHEDULE_INFEASIBLE` is the primary
expectation, and a named staged-deliverable refusal is the alternative while
mechanics are unavailable. Both require exit 1 and empty stdout. An empty
alternative, wrong deliverable, unrelated error, success or wrong exit fails.
On promotion (remove `unavailable`), the alternative disappears and the
precise spec-defined outcome is required. Either accepted probe refusal still
reports UNAVAILABLE and retains exit 2. This is not permission to substitute
an arbitrary outcome for a rule the spec already defines.

The §11 library-dependent D/H cases remain unavailable, with the reason
naming **b-isa-spec-8r4** / VM **bennett-vm-47t**. No full library hash or
ordinals are claimed. The `sched_inc8.bisa` bindings remain placeholders for
a refusal probe, not a valid executable schedule. Target-only aggregation
and scheme-expansion witnesses do not acquire a spurious library dependency.

## Red-green evidence and draft contract work

Preserved new tests and copied committed baseline sources/fixtures into
`conformance/.tmp/resume-red/`, without changing the live checkout or touching
the interpreter crate. Running the fixture, fake-binary and draft test targets
with `--no-fail-fast` reproduced failures for trailing LF, the missing forbidden
eps reference, precise E error versus obsolete staging prose, and absent draft
witnesses. Log: `conformance/.tmp/resume-red.log`. Against the inherited parser,
`staged_alternative_must_be_an_explicit_refusal` also failed at `{}` before
the alternative validation was implemented; all corresponding tests are green.

The 0.9 additions are §8.1 **K9 EPS-DECADE**, §7.3 **K10 BLOCK-IMAGE**, and
§11.1 **K11 ISA-REF-LIB**, each logged in §0.2 and `spec/isa-0.9.PROGRESS.md`.
Independent Python checks verify the printed rational boundary indices, sector
witness, all 16 qxor tuples and their reversals, ordinals and the witness-only
four-line envelope SHA-256. These are draft witness checks, not 0.9 executor
conformance. **maintainer decision required: decomposition set and hash rule**;
no `ref/` artifact or retroactive 0.8 contract was published.

Mirrors created by the previous attempt were recovered from the Beads DB
instead of duplicated: bugs labeled `isa-0.9`, **b-isa-spec-8r4** (47t),
**b-isa-spec-f31** (cxg), **b-isa-spec-0d0** (47i), and **b-isa-spec-1pl**
(clh). Only 1pl is closed; the other three remain open 0.9 items. Exported
with `bd export -o /home/tobias/Projects/bennett-workspace/bennett-isa-spec/.beads/issues.jsonl`.

Accepted 0.8 SHA-256 remains
`90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e`;
the draft SHA-256 is
`e8299a4c41231f85097a70691c86db382a9ab2efb46a1ffb81c2efa322d80855`.

## Exact final verification commands

```sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/.tmp cargo build --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/.tmp cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/Cargo.toml > /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/.tmp/resume-tests.log 2>&1
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/.tmp cargo run --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/Cargo.toml -- --bin /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/target/debug/bennett-isa-ref > /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/.tmp/resume-suite.log 2>&1
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/.tmp cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/Cargo.toml --test headers
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/.tmp cargo clippy --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/Cargo.toml --all-targets -- -D warnings
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/.tmp cargo fmt --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/Cargo.toml -- --check
git -C /home/tobias/Projects/bennett-workspace/bennett-isa-spec diff --check
```

Build/tests/Clippy/fmt/diff checks exit 0; standalone harness exits 2.
The following 2026-10-29 verification record is historical.

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

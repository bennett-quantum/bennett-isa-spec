# Editor response: council round 8

Inputs: full `out-sol.json` and `out-codex.json`, reviewed snapshot 00ceee2,
and both implementation briefs. All four distinct S0 findings are resolved;
final verification is recorded below. Owned paths: `reference-interpreter/`
and this response only. No push, dependency, RNG-distribution, accepted-spec,
conformance or Python-seed change. Tracking: b-isa-spec-41i and the adjacent
version-sensitive cleanup finding b-isa-spec-2kd. Staged deliverables 8–11
retain their disclosures.

## Design

1. A live wire has an ANF basis expression; allocation alone supplies zero.
2. Diagonal gates and read-only controls preserve basis facts; permutations transform ANFs.
3. Non-diagonal targets become fresh unknowns; no unknown result invents a fact.
4. Each information-losing gate pushes its saved expressions on every operand stack.
5. Reversible gates above those histories push likewise; independent wire stacks interleave.
6. An exact adjoint pops only when the same saved event tops ALL operand stacks.
7. Nested pairs restore LIFO; outstanding readers block an outer pop; opaque/lifetime barriers invalidate connected history.
8. One structured source lowering publishes both physical bindings/borrow sets and the fact-node program; zero trips publish none of them. Static dead-body checks and lexical qregion hoisting remain.
9. Local calls reuse canonical structured per-wire transfers, memoised by callee and exact classical specialization; SCC validation excludes action recursion (§5.1).
10. Effective controls predicate callee gates; a known-false call transfers no facts; imported calls leave modified outputs unknown and preserve read-only operands.
11. Every physical quantum gate/call boundary and FULL borrowed-touch guard remains, including disabled calls; every return proves restoration and output independence.
12. Permutation loops square maps AND return obligations; mixed loops visit exact entries, skip identical complete analysis states or prove an unknown-top inductive invariant. No sampled admission or semantic trip cap.

`borrow.rs::Facts::{reversible,forget_inverses,sequence}` implement the causal
per-wire stacks. Ordinary basis permutations need no history unless they touch
an existing quantum history: their ANFs already retain exact basis information.
Saved expressions include every operand, preserving correlations. The stack
checks all controls and targets, so an unmatched entangling operation cannot
resurrect a saved clean control. Physical frame boundaries are retained even
when a pair cancels in the fact domain.

`Compiler::compile` is the sole traversal of source lifetime/binding effects;
it emits the exact structured node program that Facts traverses. Zero-trip
bodies are omitted there before any `produce`, live-borrow mutation, call,
node or trace effect, following the existing `action.rs::LifetimeWidths`
executed-lifetime rule. There is no successor environment inferred from a dead
body. Static validation still checks that body in its cloned state; function
entry still hoists every lexical qregion, including declarations in dead bodies.

`Compiler::call_transfer` compiles a local callee into canonical wire coordinates
once per classical specialization, including nested calls, then remaps the
structured effect onto actual operands. The cache is shared by child compilers.
`Node::Call` separates its complete physical touched set from mutable outputs
and the analysed transfer. `Facts::sequence` handles its predicate and transfer;
`check_permutation_touches` independently checks the full call touch set and
recurses into callee-local lifetimes. Imported calls leave facts unknown only on modified outputs; opaque readers
invalidate inverse history across all touched operands, including read-only
controls, without erasing those controls' current basis facts. The existing Tarjan SCC call-graph validation runs first;
recursion through lifted oracle procedures remains opaque to this action pass.

## SOL-R8-B01 ≡ codex R8-B02 — resolved

RED exact library tests `r8_compound_adjoint_restores_borrow_control` and
`r8_diagonal_nested_adjoint_restores_borrow_control`: **0 passed / 2 failed**.
CLI counterparts `r8_cli_compound_x/s`: **0 passed / 2 failed**, exit 1,
empty stdout, `§5.6: a wire released inside a borrow frame is not clean`.

The first H record now survives nested X/X or S/Sdg computation. Each inner
pair pops before the final H restores c=0. CX(0,b) is identity for arbitrary
b and its external reference; c is clean at release. Commit **366a711**.

The deterministic family covers nested matched pairs through depth 3, H, X,
CX, SWAP, S/Sdg and literal Rx/Ry inverse pairs, loop wrappers and interleaved
histories on two workspace wires. The library checks all 16 t/pool basis
entries plus superposition and Bell-paired borrowed/reference entries, with
forward and adjoint initialized independently, followed by exact reversal.
Unmatched inner S, entangling CX and SWAP before the borrow still reject,
including with a certificate and with valid complete cleanup after the return.

The family exposed an adjacent §6.2 over-rejection: an older SWAP reader was
blocking an inner Rx/Rx† pair that it never read. `validation.rs::Discipline::change`
now tests the captured current-operation version-stack prefix. An added nested SWAP/SWAP
case reproduced the same defect in `joint_change`; the same version-sensitive
rule now covers joint events. Both REDs reported `§6.2: released value has not
been uncomputed` (initial family **0/3**, later joint audit **0/1**).
Older ancestor readers do not block a nested pair; an outstanding reader whose
snapshot contains the complete current event prefix still blocks its inverse. This implements
the existing SPEC-QUESTIONS 25 reading without changing its text or weakening
outstanding-entanglement negatives. Tracking b-isa-spec-2kd; initial correction
and families **b658ac9**; final joint/prefix and opaque-reader correction **0b70157**.

A final static import family distinguishes fact preservation from history
preservation. Its unknown-control negative initially stopped at §6.2 (**0/1**),
showing why matching only a reused top gate ID was insufficient for repeated H
events. Complete captured stack prefixes allow the matched whole-region
cleanup; the test then exposed an unsound borrow admission (**0/1**) through an
opaque imported reader. The import now invalidates connected inverse history
on all touched operands while changing facts only on mutable outputs. A
known-false imported call still preserves c=0; this positive is explicitly
static-only because no callee definition is linked. Added repeated-H/SWAP
identities protect the prefix distinction.

## SOL-R8-B02 ≡ codex R8-B01 — resolved

RED library **0 passed / 3 failed**, CLI **0 passed / 3 failed**:

| Subject / test | Observed RED |
|---|---|
| `zero-wide` / `r8_zero_trip_lifetime_environment_does_not_escape`, `r8_cli_zero_wide` | §3.3: c must be one wire |
| `zero-same` / `r8_zero_trip_surviving_fact`, `r8_cli_zero_same` | §5.6: released wire not clean |
| `zero-h` / `r8_zero_trip_cannot_hide_borrowed_quantum_touches`, CLI counterpart | parse/hash/check incorrectly accepted |
| `zero-s` / same literal test loop | independently replayed against pre-fix 366a711: parse accepted, CLI check exit 0/status ok; **0 passed / 1 failed** |

The separate S replay uses an isolated git-archive source snapshot beneath the
repository's `.tmp`, never a shared checkout mutation. It prints both outcomes
before failing its rejection assertion; its stdout pins the published S hashes.
This supplies independent RED evidence because the initial combined H/S test
stopped at H. Final GREEN runs cover both H and S and require §5.6, exit 1,
empty stdout before execution; certificate-present variants retain rejection.

Zero trips now execute no compiler lifetime or binding operation, so the
successor c is still the original clean one-wire allocation and b is still
the original live borrowed wire. Commit **b11b43f**. GREEN for the two
positives checks pinned hash/check; forward and independently started adjoint
at t=0/1 and 0.6|0>+0.8i|1>; one surviving interface wire; reverse
`equals_entry=true`; and exact identity with arbitrary/entangled bR.

Variants include nested zero loops, alpha-renaming, dead qcopy/qload/qborrow
rebindings and a qregion declaration in a dead body subsequently used live.
Static live-name redefinition still rejects; trip count 1 with the two-wire
successor still rejects scalar CX. The full non-permutation borrowed-touch
guard applies to H/S even when a certificate is attached.

## codex R8-B03 — resolved

RED `r8_action_call_preserves_readonly_control` and `r8_cli_call`: each
**0 passed / 1 failed**, §5.6 released wire not clean, CLI exit 1/empty stdout.
The literal two-function object has c=0, so controlled H and CX(c,b) both
transfer identity. It now preserves all library/CLI identity inputs, standalone
adjoints and exact reversals described above. Commit **5ad0555**.

The deterministic family adds known-zero/known-one effective controls (expected
I/H on t), the equivalent read-only in-parameter control, callee chains of
length 1–3, analysed compound identities on workspace, and unknown-control
negatives. The compound-identity workspace case explicitly calls the adjoint
after the borrow, retaining §6.2's required exact call cleanup; its c=0 fact
at the earlier borrow must come from the callee analysis. Unknown interface
controls of H on workspace cannot establish workspace=0 and therefore reject
the intervening borrow even though the call is subsequently undone.
Borrowed effective controls and direct borrowed operands of non-permutation
calls continue to reject, including disabled calls in the retained R3 family.

## SOL-R8-B03 — resolved

RED `exact::r8_tests::r8_unsigned_decimal_multiplication`: **0/1**, 2^127
became negative. RED `r1_linking::r8_unsigned_multiplicity_composition`:
**0/1**, composed `epsilon_total.diamond` was JSON number 0 instead of the
required exact string `340282366920938463463374607431768211456`.

`exact.rs::BigInt::from_u128` builds the complete unsigned limb magnitude;
`Decimal::mul_int` uses it without an intermediate signed cast. Signed
construction also reuses the unsigned magnitude constructor. Commit **366a711**.
Tests cover 0, i128::MAX, 2^127, u128::MAX and fractional 0.1, with exact
expected decimal strings. The exact client/callee fixture uses local diamond=1,
callee inclusive=1, client inclusive=2^128 and multiplicity=u128::MAX;
`Object::link` now returns the exact positive string and `check_hashes` passes.
No multiplicity cap, precision fallback or interval-verifier dependency.

## Published literal pins

The helper asserts exact stored-H object bytes with SHA-256 even for rejected
subjects. Accepted subjects also assert interface, closure and object hashes
and `check_hashes`. Hash expectations are the independently published member
values, never implementation outputs copied into fixtures.

| Subject | interface = closure H | object_hash |
|---|---|---|
| H X X H | 0fd6d6eadcb219457ce3d9aa091a2176d38ce9f81551235799dae7cabe6e0e2f | 5eff1283a8f71dcce612e1802e510fdcae30c609395686311962bd630f01461c |
| H S Sdg H | a8a4f90c95a1995df9a28afe4369f3455239ae54b6c22b5c8133df7a60253401 | 535f274d3c7328e13cfd99e8785751ec3874e4d3090da3544e781855ca35439f |
| zero / wide | c9cbc01c190e4454e37b553e1d7e8e9eeea856631a6ab397029577447b053b87 | 708531c08cb63447ea2bbaa966c69562300ab1bc89e07efb9c3278799c54922d |
| zero / same | d607c1df25e3a4d8f4e730501cf13a04a62cfa6adbeae5c36133a4f140f406bd | 568f2a4ee6c4cb927cafe4bd54414def5e91257186832704e6be7abfe99fd1bc |
| zero / H borrowed | 90cbbcb904bff3ae413e3aaa3ec398173654dc1690354777bcb3837881098080 | 60865de88a18adc53e2ecb23ab54d86f44836e4895cb7fb54e3ac78c0c879280 |
| zero / S borrowed | 1c0599e2f06674ccbc34f0017d5d158af1588d7d54982aa1adfebddc4262fa58 | fd5167fefabbbc72d9cd486e6cf036119f2ba5088558cccb155f898eaa7285ec |
| controlled call / @a | 63ae90bc9af75ebe3e958956282567f52129b0097651a445c08f7c5f2845ac79 | c4a3c1adad250d856a334556766f0cc398a71b6c65df3ce5f81adf86d07f968e |

The controlled-call @u interface/closure pin is
`c0c51a4bf1e9713e237601daaf342134f91eae5fd7143f8638595157d33db5b6`.

## Retained negatives and contracts

All round 4–8 tests remain. Retained checks include independent loop-local
returns (later borrowing cannot repair an earlier return), globally restored
four/eight-trip rotation negatives, evolving clean controls after iteration 3,
unknown dirty controls, unmatched H, outstanding transitive/diagonal readers,
cell-version/window closure, illegal borrowed quantum/phase/call controls,
opaque borrowed-input claim requirements, certificate subject binding, checked
claim replay, mixed lifetimes and nonzero/zero-trip qregion hoisting.
Neither translation-lemma structure nor a final whole-region certificate can
waive a definite per-return or non-permutation touch failure. RNG generator,
seeds and distribution are unchanged. No semantic trip, wire or multiplicity
limit was added; no accepted ISA capability was intentionally narrowed.

## Verification commands and observed counts

All gates use the repository root, absolute manifest, offline, headless and
project-local temporary storage. `CARGO_PROFILE_TEST_OPT_LEVEL=1` changes
optimization only; assertions, overflow checks and test domains are unchanged.

```sh
unset DISPLAY WAYLAND_DISPLAY DBUS_SESSION_BUS_ADDRESS BROWSER
export TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp
export CARGO_PROFILE_TEST_OPT_LEVEL=1
cargo fmt --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml -- --check
cargo clippy --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --all-targets -- -D warnings
cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml
cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml -- --ignored --nocapture
```

Development commands used the same prefix/manifest with these filters:
`r8_unsigned`, `--test r1_linking r8_unsigned`,
`--test r1_borrow r8_`, `--test cli_xam r8_`,
`--test r1_borrow r8_zero`, `--test cli_xam r8_cli_zero`,
`--test r1_borrow r8_action_call`, `--test cli_xam r8_cli_call`,
`--test r1_borrow sol_r5_b02`,
`--test r1_fuzz_discipline generated_r8`, and later `generated_r8_nested`,
all with `-- --nocapture`. RED S replay uses the git archive of 366a711 at
`.tmp/r8-red-s/reference-interpreter/Cargo.toml`, `--test r8_red_s -- --nocapture`.

Observed development GREEN counts: **6/0** nested/numeric; **10/0** R8
borrow/CLI after zero-trip fix; **1/0** existing huge permutation regression;
**12/0** R8 borrow/CLI after call fix; family intermediate **3/0**, **134
accepted / 25 rejected** (86/6 nested, 33/10 zero, 15/9 call). Initial family
**0/3** includes two fixture errors (address width; missing explicit call
adjoint) corrected to comply with §§3.3/6.2, plus the genuine nested-reader
rejection. The extended joint-reader RED was **0/1**. The separate S witness
pre-fix replay was **0/1**, with both load surfaces observed before assertion.

First completed full suite: **314 passed / 0 failed / 5 ignored**, 8 unit +
306 integration, 0 doctests; 36.79 s summed runner time. Initial strict Clippy
reported one map-entry style violation; after its correction strict Clippy
passed with zero warnings (9.07 s). Fmt checks passed. One earlier full run
raced test authoring, observed new zero-trip REDs, and was stopped during
needless permutation-history expansion; the basis-ANF fast path was corrected
and its huge regression passed. A later gate attempt was stopped to include
the final joint-reader RED. Interrupted runs are not reported as passes.

Final source gates after **0b70157**:

| Gate | Observed result |
|---|---|
| fmt --check | pass; 0 tests executed |
| strict all-target Clippy | pass; 0 warnings, 0 tests executed; 2.84 s |
| normal suite | **315 passed / 0 failed / 5 ignored**, 8 unit + 307 integration, 0 doctests; 21.07 s summed runner time |
| deterministic R8 families | **4 passed / 0 failed**, **139 accepted / 26 rejected** |
| standalone read-only conformance | **445 pass / 0 fail / 65 unavailable / 510**, exit 2 for explicit staging |

Final family details: **90/6** nested, **33/10** zero trips, **15/9** local
calls, **1/1** static imports. The earlier joint-only family run was **3/0**,
**136/25** (88/6 nested). Every accepted executable R8 family subject is
checked against independently derived I or H maps, including arbitrary and
entangled borrowed inputs; the unlinked import positive is static-only.

Read-only conformance command (build artifacts confined to the owned crate):

```sh
CARGO_TARGET_DIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/.tmp/conformance-target cargo run --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/Cargo.toml -- --bin /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/target/debug/bennett-isa-ref
```

| Suite | pass | fail | unavailable | total |
|---|---|---|---|---|
| A | 21 | 0 | 1 | 22 |
| B | 35 | 0 | 0 | 35 |
| C | 339 | 0 | 1 | 340 |
| D | 7 | 0 | 6 | 13 |
| E | 0 | 0 | 29 | 29 |
| F | 33 | 0 | 4 | 37 |
| G | 10 | 0 | 14 | 24 |
| H | 0 | 0 | 10 | 10 |

The concurrently maintained harness adds unavailable contract cases compared
with round 7's 507-case run. No unavailable item is counted as a pass; the
existing deliverables 8–11 staging is retained. Conformance sources were never
modified by this implementation. Final logs are repository
`.tmp/r8-complete-{fmt,clippy,tests,sweeps,conformance}.log`; all RED logs and
isolated S replay are retained there too.


| Ignored sweep | Domain | Observed result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 pairs × 4 operations = 262,144 executions | **1 passed / 0 failed**, 0.70 s |
| 17-wire borrow | 131,072 entries | **1 passed / 0 failed**, 4.31 s |
| checked borrow certificate | 262,144 domain elements, subject and expected program | **1 passed / 0 failed**, 22.04 s |
| §6 differential | 200,000 programs: **70,930 accepted (all exactly clean) / 129,070 rejected** | **1 passed / 0 failed**, 80.30 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | **1 passed / 0 failed**, 2.38 s |

All five ignored sweeps: **5 passed / 0 failed**, 109.73 s summed runner time.
The higher accepted count than the historical 64,717/135,283 follows the
version-sensitive cleanup correction with unchanged random code/distribution
and seeds; every accepted sampled program passed independent exact cleanup.

The unchanged Python seed was also run headlessly:
`python3 /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/isa_check.py`.
It verified **2 example objects and 2 printed envelopes**, executing as stated.
Only a source comment was clarified after the final semantic tests: cached
call proof templates predicate paired image preparation/unpreparation together,
using C(P†) C(U) C(P) = C(P† U P); actual execution still prepares images
unconditionally. Fmt and strict all-target Clippy were rerun and passed
(0 warnings, 0 tests; Clippy 1.29 s), logged as
`.tmp/r8-comment-{fmt,clippy}.log`. No semantic Rust changes followed the final
normal or ignored suites. Owned diff checks pass.

## ISA 0.8 gaps and disposition

No new ISA gap, spec amendment, decline or partial finding. The stack and
version-prefix rules implement §§5.3/6.2 and existing SPEC-QUESTIONS 25; zero
trips follow §4.6 and executed lifetimes; call transfer follows §§3.5/4.7/6.1;
unsigned composition follows §§9.1/9.3/12.3. No scope, trip, wire, multiplicity
or frame restriction was added. Existing staged deliverables 8–11 and the
pre-existing performance follow-up b-isa-spec-436 retain their disclosures.
Accepted ISA 0.8 SHA-256 is unchanged:
`90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e`.

Implementation commits: **366a711**, **b11b43f**, **5ad0555**, **b658ac9**,
**0b70157**. Final evidence/comment commit: **e15a187**. Only explicit owned paths
were committed with BEADS_HOOK_TIMEOUT=20; no push. Beads' automatic export is
left to the maintainer outside owned commit paths.

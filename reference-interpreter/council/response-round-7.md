# Editor response: council round 7

Inputs: full `out-sol.json` and `out-codex.json`, reviewed snapshot 67d1b94.
All four S0 findings are resolved; no declines or partials. Final verification is recorded below.
Owned paths: `reference-interpreter/` and this response only. No push, dependency,
RNG-distribution, accepted-spec, conformance or Python-seed change.
Tracking: b-isa-spec-lto; cycle-key correction b-isa-spec-gui; performance
follow-up b-isa-spec-436. Staged deliverables 8–11 retain their disclosures.

## Design

1. A live wire has an ANF basis expression; an unconstrained entry is a variable.
2. Only allocation supplies zero. Losing information supplies a fresh unknown.
3. X-family gates transform target expressions; controls remain read-only.
4. Controlled swaps transform both outputs with the complete condition.
5. Diagonal gates preserve basis expressions, including known zero/one.
6. A false predicate disables the gate's fact transfer, not its §5.6 touch guard.
7. Non-diagonal targets become unknown; an untouched exact inverse restores saved expressions.
8. Any intervening operand touch invalidates the inverse record, including entangling gates.
9. Every quantum gate still ends a frame; the full touched set guards borrowed controls/targets.
10. Every return retains restoration and output-independence equations; certificates cannot waive them.
11. Zero trips transfer nothing; permutation loops square maps and obligations with the actual bound.
12. Mixed loops traverse exact entries, skip identical full analysis states, or prove a joined unknown-top invariant.

`src/borrow.rs::Compiler::compile` emits structured Repeat nodes for all loops
and Quantum nodes carrying gate action separately from the full touched set.
`Facts::sequence`, `repeat`, `invariant`, `flush` replace prefix replay.
No `n.min(3)`, frame merging, sampled admission or semantic trip limit remains.
Inverse records preserve old correlations, and every intervening multi-wire
touch invalidates them; unknown results cannot resurrect a clean value.
`Evaluation::apply` retains exact permutation/opaque composition and every
per-return obligation. The cropped translation-lemma route still requires all
lifetime equations. Opaque unresolved proofs retain their certificate route.
Execution's quantum cycle skipping remains separate from analysis.

## SOL-R7-B01 — resolved

RED: `r1_borrow::r7_restored_zero_before_borrow` and
`cli_xam::r7_cli_restored` rejected the exact H(c); H(c) subject with
`§5.6: a wire released inside a borrow frame is not clean`.

`borrow.rs::Facts::sequence` now restores the saved zero expression at the
second untouched H. The following CX(0,b) is I for arbitrary b and an external
entangled reference. GREEN pins interface = closure =
`12efae749cb84eec0918c9aa49adb348957eba9e81e0eaa8f2514481a0d9dba0`,
object = `b2534762bf5eb795efd7ba74195198f0b67e2012b9f3243700e58ee92ee9fc53`.
Implementation commit: **1caa062**.

CLI hash/check succeed; forward and independently initialized adjoint preserve
t=0/1 and 0.6|0>+0.8i|1>; reverse reports equals_entry=true with one interface
wire. Library checks 16 t/pool basis entries, a superposition and a borrowed
wire entangled with an untouched reference, independently in both directions,
then exact reversal. The deterministic family adds H/H, Rx/Rx†, Ry/Ry†,
empty lifetimes, disjoint intervening touches and changed-c negative controls.

## SOL-R7-B02 ≡ codex R7-B01 — resolved

RED library and CLI tests:

| Subject | Test suffix | Observed RED |
|---|---|---|
| SOL four-trip bad | r7_reject_loop_bad / r7_cli_reject_loop_bad | parse/hash incorrectly succeeded |
| SOL four-trip good | r7_nonpermutation_loop_positive / r7_cli_loop_good | released wire not clean, §5.6 |
| codex four-trip rotation | r7_reject_rotate_4 / r7_cli_reject_rotate_4 | parse/hash incorrectly succeeded |
| codex eight-trip rotation | r7_reject_rotate_8 / r7_cli_reject_rotate_8 | parse/hash incorrectly succeeded |

The exact compiled trip count now governs every entry and return. SOL's c at
the borrow is initial_c XOR (n mod 2). Codex's four returns see c[3]=0,0,0,1;
the fourth return therefore rejects before hash/check emit anything. Eight
trips reject despite the later global restoration. The positive four-trip
subject preserves the complete I_t tensor I_bR map in both directions.
Implementation commit: **1caa062**.

Published pins, including rejected objects' exact stored-H byte hashes:

| Subject | interface = closure | object_hash |
|---|---|---|
| SOL bad | 232f157c4fe6c4abdb3ebc384d1b19b01751b68df4949b86d25996e87c300515 | 3e8f134a472e3e31545e6bd3ec22058da463927ea24af3929bf3357a36ca3926 |
| SOL good | fe26a45f4d75fde7d13f0e915a13d15a6ab9bb6b280dcc302996df1391f899d9 | 7f1312cd66fc88f5819be157a1a5964c27363f8f1bc728a9b21cdb342685b198 |
| codex 4 | 5dcd11b58f335ac7f301caf4e2773ffbfb172a01330fdb6c7c990c15036b340f | d58420852a90bad7d351d52072827bff5c6a8b592ae25072cffdf3b793e88de3 |
| codex 8 | 4036a594dfa32ab65353367719970549a2cbe1237e471ff6f43b169e09844112 | bc9ac8d642fe4c4569d81021da302108919d500fda82cde3577a2916b681068c |

`generated_r7_evolving_loops_match_per_return_and_renamed_unrollings`:
**32 accepted / 38 rejected**, with counts 1/3/4/5/8 and 2^40, initial c=0/1,
nested 2×2, renamed straight-line expansion, rotation through eight returns,
and certified/uncertified subjects. Expectations are literal parity and each
return's one-hot control, never whole-region reversal as a substitute.
`generated_r7_large_noncycling_loop_proves_unknown_top_invariant`:
**1 accepted / 1 rejected**, static proofs at 2^40. H X X H is independently
I, but intervening touches invalidate the saved H in this fact domain, forcing
the invariant path; c=0 enables every lifetime proof, c=1 fails it.

The large family exposed an adjacent static scalability defect:
`validation.rs::Discipline::cycle_key` now compares equality/order classes of
current/saved window epochs instead of absolute ever-increasing clocks.
The actual counters and all cleanup comparisons stay intact. A dedicated unit
test distinguishes an open saved window from a closed one after epoch shifts.
No epoch is discarded from a live cleanup obligation. Family/window/pin commit:
**6152d25**. A final RED audit pin,
`r7_zero_trip_is_not_an_intervening_inverse_operand_touch`, found that a zero
Repeat still invalidated an H inverse via its lexical touched set. The correct
§4.6 transfer now skips zero Repeat before any dynamic touch/frame/lifetime
operation. RED: 0 passed / 1 failed, released wire not clean (§5.6); GREEN:
8 R7 borrow tests passed. Commit **516052b** also corrects the obsolete Repeat
comment. This adds no exception for executed gates.

## SOL-R7-B03 — resolved

RED: `exact::r7_tests::r7_dyadic_scaled_conversion_boundary_rounding_and_subnormals`
failed at k=1024 with NaN instead of 1.0; the pinned CLI replay
`r7_cli_bounded_5000_h_t_reports_finite_amplitudes` failed exit 1, empty stdout,
`§16.1: non-finite binary64 has no JSON number`.
`r7_root2_cancellation_does_not_overflow_intermediate_terms` also failed its
finite-positive assertion for (1−sqrt(2))^810.

`exact.rs::Dyadic::to_f64` rounds a bounded mantissa using guard/sticky bits and
the net exponent (nearest, ties to even). Subnormals use their exact binary64
power-of-two encoding, avoiding powi's intermediate underflow. `Root2::to_f64`
uses `exact/measurement.rs::root2_approximate` when separate terms would
otherwise overflow or severely cancel, refining dyadic enclosures until both
endpoints round alike. No NaN clamp, gate/denominator ceiling, or weakened JSON
non-finite check. Implementation commit: **51d45dc**.

Pins: interface = closure =
`2b29d4983fd4149572528e684e31ed7802cd7727f628c50058f7d62329ca22e9`,
object = `8af0b609daca8eb239a88ad3f47fddec0e6e43e9a456bca951887ccd89af0080`.
Forward coordinates match the member's independent 100-digit matrix product
within 1e-12, with both support keys, unit norm, mut=[null] and one wire.
Standalone adjoint coordinates come from the conjugate first row of that
matrix; reverse equals_entry=true. `quantum::r7_bounded_h_t_exact_support_is_finite_and_reverses`
independently drives all 5,000 primitive pairs, checks the supplied coordinates,
and reverses exactly. Unit cases cover k=1023/1024/1056/2000/5000, signed
values, nearest-even ties, smallest subnormals, underflow and f64::MAX.

## codex R7-B02 — resolved

RED: `r7_diagonal_basis` / `r7_cli_diagonal` failed with
`§5.6: qreturn does not restore borrowed wire v2_0`;
`r7_readonly_control` / `r7_cli_control` failed with
`§5.6: a wire released inside a borrow frame is not clean`.

`borrow.rs::Facts::sequence` preserves diagonal operands and read-only
controls rather than erasing every touched wire. Both exact actions are I,
with the same full library/CLI basis, complex-superposition, entangled-reference,
standalone-adjoint, one-wire and reversal checks as SOL-R7-B01.
Full touched sets still enforce permutation-only borrowed touches, including
disabled quantum controls and diagonal inverse pairs. Commit: **1caa062**.

Diagonal interface = closure =
`0d7a3eccbc3cdd7440ec3c1ad66a12e8650203486b2736f6fe5ea2ddb87a56f6`,
object = `5b407065c615539c15c7f9381b8fccfebc8e109cae3e8966c05fbd4acd687205`.
Known-zero controlled-H interface = closure =
`06b70e4ca0799aac622c280b3f1de5b825888d21c19da5ec3f015c88b4fc1fe6`.
`generated_r7_diagonal_control_and_inverse_boundaries`: **60 accepted /
24 rejected**, including S/Sdg, T/Tdg, Rz±, CZ, known-zero/one predicates,
matched non-diagonal pairs, empty frames and certificate-present negatives.

## Retained negatives and contracts

All existing tests remain, including R4 independent loop-local returns,
borrowed-output dependence, distinct-name unrollings, spanning lifetimes;
MUSE-R4 fresh-CX returns and paired-CX positives through 2^40;
R5 definite X/CX errors with validly bound exhaustive certificates, internal
return audits, certified CZ phase/control touches, empty certificate indices,
static translation-frame maximality and full-domain opaque composition;
R6 dirty c, single X b, single-H dirty control, dirty free, zero-trip/changing
width lifetimes and public certificate replay. Round-7 X c/single CX,
changed non-diagonal controls and every non-permutation B touch still reject
with §5.6/§6.2, exit 1 and empty stdout. Hash/schema/certificate binding, oracle
step/stack tuples, complete inverse machine snapshots, CC/R history, exact
support and Born-selection tests remain in the complete gates.

README and PROGRESS describe gate transfer and the actual loop proof, replacing
the old untouched-wire/prefix-patch description. SPEC-QUESTIONS 20/25/28/30
remain consistent with this implementation. No new ISA 0.8 ambiguity or exact
specification change is needed.

## Verification commands and observed counts

Final sequential gates use the following environment and absolute manifest
(test optimization only; same domains/assertions and overflow checks):

```sh
unset DISPLAY WAYLAND_DISPLAY DBUS_SESSION_BUS_ADDRESS BROWSER
export TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp
export CARGO_PROFILE_TEST_OPT_LEVEL=1
```

```sh
cargo fmt --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml -- --check
cargo clippy --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --all-targets -- -D warnings
cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml
cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml -- --ignored --nocapture
```

Development commands use the same manifest/environment: `--test r1_borrow r7_`,
`--test cli_xam r7_`, `--lib r7_dyadic`, `--lib r7_root2`,
`--test cli_xam r7_cli_bounded`, `--test quantum r7_`,
`--test r1_fuzz_discipline generated_r7`, and `--lib r7_epoch`;
`-- --nocapture` records diagnostics and family counts.

Observed development: RED 0/7 library and 0/7 CLI; GREEN 14 exact pins.
First full normal suite 288 passed / 0 failed / 5 ignored. Numeric RED:
0/1 dyadic, 0/1 pinned CLI, 0/1 Root2; GREEN 2 unit + 1 CLI.
Primitive quantum target: 1 passed / 0 failed. Final deterministic families:
3 tests, **93 accepted / 63 rejected**; RNG code and seeds unchanged.
An initial CLI RED attempt raced a subsequent executable rebuild and passed
(103.67 s); it is not counted as RED evidence. The isolated old-conversion
replay failed as required (35.89 s), followed by GREEN (70.78 s). An intermediate
scaled conversion failed the new smallest-subnormal assertion and was fixed
before commit. Repeated hash pins: 7 passed / 0 failed. Epoch-equivalence unit: 1 passed.
The complete debug gates also passed at 295 tests (232.68 s) and 296 tests
(206.19 s), each 0 failed / 5 ignored. The final zero-trip RED and 8-test GREEN
are above. No interrupted run is included in a completed-suite count.

Exploratory huge noncancelling static histories (TH on an interface, swap
rotation, and an outstanding-copy variant) were interrupted without a result;
none counts as a pass or rejection. Their existing enumerative §6 provenance
path is tracked by **b-isa-spec-436**; it is not the new borrow invariant path.
The final family keeps huge valid/invalid H/X controls, rotating cases through
eight visits and a huge noncycling borrow-fact proof. No semantic bounds were
capped and no object was newly rejected to hide this performance limitation.

The earlier unoptimized ignored command completed three tests: division
**1 passed**, 16.64 s; 17-wire execution **1 passed**, 97.84 s; checked
certificate **1 passed**, 492.05 s. Its discipline sweep was stopped without a
result after the last zero-trip audit so the complete final sequence could be
rerun after all source changes. Those three completed counts remain partial
historical evidence, not a five-sweep pass.

Final sequential gates after the last source change:

| Gate | Observed result |
|---|---|
| fmt --check | pass, 0 tests executed |
| strict all-target Clippy | pass, 0 warnings, 0 tests executed |
| normal suite | **297 passed / 0 failed / 5 ignored** (7 unit + 290 integration, 0 doctests), 40.69 s summed runner time |
| complete ignored suite | **5 passed / 0 failed**, 143.52 s summed runner time |
| owned diff checks | clean |

| Ignored sweep | Domain | Observed result | Runner time |
|---|---|---|---|
| u8 division/remainder | 65,536 pairs × 4 operations = 262,144 executions | 1 passed / 0 failed | 1.10 s |
| 17-wire execution | 131,072 entries | 1 passed / 0 failed | 5.93 s |
| checked certificate | 262,144 elements, subject and expected program | 1 passed / 0 failed | 30.31 s |
| §6 differential | 200,000 programs: **64,717 accepted (all exactly clean) / 135,283 rejected** | 1 passed / 0 failed | 102.22 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 1 passed / 0 failed | 3.96 s |

Fresh read-only standalone conformance command:

```sh
CARGO_TARGET_DIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/.tmp/r7-conformance-target cargo run --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/Cargo.toml -- --bin /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/target/debug/bennett-isa-ref
```

Result after the last fix: **445 pass / 0 fail / 62 unavailable / 507**, exit 2.
Earlier in this session the concurrent harness was **445/0/60/505**, also exit 2.
Unavailable/staged cases are never passes. Conformance sources and its added
contract cases belong to the concurrent agent and were not edited here.

Final logs: repository `.tmp/r7-verified-{fmt,clippy,tests,sweeps,conformance}.log`;
development logs use the same `.tmp/r7-` prefix. Accepted 0.8 SHA-256 remains
`90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e`.
No semantic Rust changes followed the final sequential gates. Tracking
b-isa-spec-lto and b-isa-spec-gui are complete; b-isa-spec-436 remains the
explicit performance follow-up. Beads automatically updated its export;
that file is outside the owned commit paths and is left for the maintainer.
No push; no new ISA gap or specification edit required.

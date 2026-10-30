# Editor response: council round 6

Inputs: full `out-sol.json` and `out-codex.json` against reviewed snapshot
9e4984f (implementation checkout b2e4fc1). Both shared S0 blockers are resolved;
no declines or partials. Owned implementation paths are `reference-interpreter/`;
this response is the only edited council path. Accepted ISA 0.8, Python seed,
RNG distribution, dependencies and staged deliverables 8–11 are unchanged.
No push. Tracking bead: b-isa-spec-7dd.

## SOL-R6-B01 / codex R6-B02 — resolved

RED before production edits:

| Library test | Observed failure |
|---|---|
| `r1_borrow::r6_sol_boundary_exact` | §5.6: qreturn does not restore borrowed wire v2_0 |
| `r1_borrow::r6_codex_boundary_exact` | §5.6: a wire released inside a borrow frame is not clean |

`cli_xam::r6_cli_sol_boundary_exact` and `r6_cli_codex_boundary_exact`
independently failed at CLI hash, exit 1, empty stdout, with the same respective
diagnostics. Both objects are literal member witnesses, with their stored H.
They now pin all three identity surfaces:

| Witness | interface = closure | object_hash |
|---|---|---|
| SOL (CX, H/H) | ba813954a842962eecd96263d6dbe2f549f7f8d4c99241c660f0a3c7626cf10a | db747f53ae732bd2cef0eeeaed48b8bb2a523dc74f53b38d2876601a3760445f |
| codex (empty borrow, one H) | 6c7101e365eccc827653aa5dc596d68fefae112b9e4b6df00b5078f5639bf7d3 | e5517c39c82f32f9c006828c6b9535d899dad9c0a7b59339ba9fabea3672fc3f |

`src/borrow.rs::Object::decide_frames` still partitions at each actual
non-permutation boundary and retains the cropped translation-lemma route.
`factorises` now receives the established preceding context as well as the
current permutation run. Context replay keeps the polynomials of untouched
live wires; each touched output at an Other boundary receives a fresh free
variable. Old variables remain available for surviving correlations. Thus
H(t) cannot erase c=0, while H(c) does invalidate c's known value. Prefix
obligations are not reused as current-frame obligations: earlier frames are
checked independently by decide_frames, and the current run must still prove
all its own return restoration/independence equations. Fresh qborrow entries
remain independent. The permutation-only borrowed-touch guard, definite-error
precedence and certificate admission/replay paths are unchanged.

GREEN: hash/check exit 0, H/H acts as exact identity, and the codex object
acts as H_t tensor I_bR. CLI checks forward and standalone adjoint at t=0/1
and 0.6|0>+0.8i|1>, the literal H amplitudes/signs, one remaining interface
wire, and reverse equals_entry=true. Library checks every t/three-wire-pool
basis entry (16), superposition and borrowed/reference entanglement (18
states), independently in each direction and then under exact reversal.

`r6_boundary_variants_retain_clean_controls_and_reject_dirty_frames` adds five
positives: H after qborrow before CX; doubled X on b; a loop-wrapped lifetime;
H following cleanup; and unrelated quantum workspace with matched H cleanup.
Eight certified/uncertified negatives retain rejection: X c before borrowing
and after return for explicit cleanup; single X b; dirty c release; and H c
before a single CX. Each rejected object reports §5.6 or §6.2. All earlier
R4/R5 per-return, per-frame, opaque, structural, phase-touch and certificate
negative tests remain in the passing complete suite. No frames are merged,
no certificate waives a return, and no gate/trip restriction was added.

No spec gap: §§4.3/5.2/14.2 give the clean allocation entry, §5.6 fixes
factorisation per frame, and §§6.1–6.2 preserve unrelated dependencies.

## SOL-R6-B02 / codex R6-B01 — resolved

RED before production edits:

| Library test | Observed failure |
|---|---|
| `quantum::r6_zero_trip_exact` | §3.3: missing consumed lifetime width |
| `quantum::r6_changing_width_exact` | §3.3: wire index out of range (inverse execution) |

The two CLI counterparts (`r6_cli_zero_trip_exact`,
`r6_cli_changing_width_exact`) independently reproduced those failures after
hash/check succeeded. The first is the R5 live.remove regression; the second
exposes the static-line identity limitation. Exact unchanged pins:

| Witness | interface = closure | object_hash |
|---|---|---|
| zero trip | 9a453b6720abda445235294c98c6288e616dc5bc1907c78af1755104b3a2e48a | 1e9a003077d2afd78e6f14384902f031c6aa140bebaf14702889de13d4cd814a |
| changing width, two trips | de76a9295cf8233d97909660d48c9f6ed82abe593735257858457d35a0c0557e | 128fa2a938dfbdcccdd255aafd128d125737a4a00fb63e9aef791a1f405bfb1a |

`src/action.rs::infer_widths` now builds `LifetimeWidths`, a structured
execution plan indexed by sequence position and dynamic loop entry. It walks
qif's complete expansion in forward order and models unconditional loop
lifetimes on every entry map. A zero-trip body has no lifetime effect;
lexical qregion hoisting and static validation still execute normally.
Repeated live-width maps share a periodic plan, including any transient
prefix, without a trip-count cutoff. No static source line keys metadata.
`Environment::widths` is removed. `Object::action_sequence` passes the matching
body plan to each visit, reversing loop iteration indices in adjoints. Inverse
qfree/qreturn use the width of that dynamic release's producer. Inverse quantum
period skipping stops before transient width-plan prefix iterations, so an
identity steady iteration cannot skip the first width-1 lifetime.

GREEN: the exact fixtures independently preserve every input forward and
adjoint and reverse with equals_entry=true, leaving no workspace. CLI checks
t=0/1 and complex superposition, one interface wire, pinned hashes/check and
exact states. `r6_structured_lifetime_variants_match_independent_identity`
adds 14 positives: nested zero-trip cleanup, a dead inner loop, dead
qcopy/quncopy, qload/qunload and qborrow/qreturn changes, alpha-renamed
changing-width unrolling, and both qalloc/qfree and qborrow/qreturn loop-carried
width changes at trips 1/2/3/128. Existing huge-bound action tests still pass;
there is no ISA trip-count restriction. The R5 straight-line reused-width,
mixed-lifetime and live-name redefinition pins remain green.

README and PROGRESS qualify the historical R5 width evidence as its tested
straight-line/self-contained lifetimes; the R6 evidence explicitly covers
zero trips and loop-carried widths. No spec question: §§3.3/4.3/4.6/5.3
already require executed lifetime matching and adjoint version renumbering.

## C7 deterministic families / C8 reporting

`generated_r6_disjoint_h_boundaries_match_identity_and_renamed_unrollings`:
48 accepted cases, H placement across disjoint lifetimes, trips 0/1/2/3,
nesting and renamed unrolling. `generated_r6_dynamic_width_lifetimes_match_identity_and_renamed_unrollings`:
96 accepted cases, trips 0/1/2/3, widths 1/2/3, qalloc/qborrow, nesting and
renamed unrolling. Expectations are hand-derived identity maps from c=0,
H²=I and within-lifetime X²=I. Every case checks 18 complete quantum inputs,
forward and adjoint independently, each followed by reversal. The existing
random generator and seeds are unchanged. README's round-2 paragraph now
states the boundary and executed-lifetime behavior explicitly.

## Verification commands and observed counts

All commands below run headless, offline where applicable, with absolute paths.
Equivalent shell prefix used for every gate:

```sh
unset DISPLAY WAYLAND_DISPLAY DBUS_SESSION_BUS_ADDRESS BROWSER
export TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp
```

Development commands (each used the absolute manifest below):

```sh
cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --test r1_borrow r6_ -- --nocapture
cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --test quantum r6_ -- --nocapture
cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --test r1_borrow --test quantum --test cli_xam r6_ -- --nocapture
cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --test r1_borrow r6_boundary_variants -- --nocapture
cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --test quantum r6_structured -- --nocapture
cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --test r1_fuzz_discipline generated_r6 -- --nocapture
```

Observed development counts: RED library 0 passed / 2 failed in each of the
two targets; CLI 0 passed / 4 failed. GREEN exact targets 8 passed (4 CLI,
2 quantum, 2 borrow). First full suite 270 passed / 0 failed / 5 ignored.
Additional variant targets each 1 passed. The first family-authoring run had
1 passed / 1 failed because zero-trip alpha-unrolling produced an illegal
empty nested body; replacing that empty sequence with X t; X t made both
families pass (2 passed, 144 cases). This was a fixture construction error,
not a production behavior change. Initial compile-only test-authoring errors
were fixed before the semantic RED runs. An exploratory huge carried-allocation
run was interrupted without a completed result; it is not counted as evidence.

Final gates and fresh ignored sweep results follow below.

Final normal command:

```sh
cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml
cargo clippy --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml -- --check
python3 /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/isa_check.py
```

| Gate | Observed result |
|---|---|
| Normal suite | **274 passed / 0 failed / 5 ignored**, 270 integration + 4 unit, 0 doctests; 29.53 s summed runner time |
| Strict all-target Clippy | clean, 0 tests executed; rerun after removing an obsolete source comment, also clean |
| fmt --check | clean, 0 tests executed; final rerun also clean |
| Unchanged Python seed | passes: 2 example objects and 2 printed envelopes |
| Owned diff checks | clean |

Affected ignored sweeps (separate commands; neither unrelated arithmetic nor
recursive-prefix sweep was rerun in this pass):

```sh
cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --test r1_borrow -- --ignored codex_b14_seventeen_wire_swap_identity_frame_exhaustive --nocapture
cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --test r1_certificates -- --ignored b14_seventeen_wire_frame_with_checked_claim --nocapture
cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --test r1_fuzz_discipline -- --ignored accepted_random_actions_run_exactly_clean_sweep --nocapture
```

| Sweep | Observed result | Domain | Runner time |
|---|---|---|---|
| 17-wire borrow | **1 passed / 0 failed** | 131,072 entries | 138.73 s |
| Checked borrow-frame certificate | **1 passed / 0 failed** | 262,144 claim elements, subject and expected program | 357.91 s |
| §6 differential | **1 passed / 0 failed** | 200,000 programs: **64,717 accepted, all exactly clean / 135,283 rejected** | 624.80 s |

All three affected ignored tests passed, 0 failed. These are fresh observed
counts, not the historical R5 results.

Fresh standalone conformance command, with harness build output confined to
the interpreter's ignored directory:

```sh
CARGO_TARGET_DIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/.tmp/conformance-target cargo run --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/Cargo.toml -- --bin /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/target/debug/bennett-isa-ref
```

Result: **445 pass / 0 fail / 48 unavailable / 493**, exit **2** for explicit
staging/contract gaps. The current read-only conformance checkout has two
additional unavailable cases since the review's 491-case snapshot:
G-estimate-resolved-class-decimal and D-sampled-relation-invocation. This pass
does not promote, skip or count them as passes. Deliverables 8–11 retain their
existing order and Unverified disclosures. No new ISA 0.8 ambiguity was needed
for either fix, so no SPEC-QUESTIONS or proposed ISA change was added.

Implementation commits: **dceaa22** (both shared blockers and eight exact pins),
**32ef2e6** (variants, C7 families and C8 README/progress), **0d12ff1**
(borrow/certificate evidence and comment cleanup), **05675d1** (final sweep
counts and completed progress record). Tracking bead b-isa-spec-7dd is closed.
Only an obsolete
source comment was removed after the full semantic suite; final fmt/Clippy
were rerun. No semantic Rust changes followed the recorded normal/sweep runs.
Logs are retained under `reference-interpreter/.tmp/r6-*.log`. Accepted spec
SHA-256 remains `90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e`.
The Beads CLI automatically exported its tracking update to .beads/issues.jsonl;
that export is outside the owned commit paths and is left to the maintainer.

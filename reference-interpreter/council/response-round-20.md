# Editor response: council round 20

**SOL-R20-B01 is resolved; S1 is applied.** No blocker is declined or partial.
The literal range and unrestricted positive lifted widths remain unchanged.
Only interpreter paths and this response are committed; no push. No accepted
0.8 spec, draft spec or conformance file was edited by this implementer.

Implementation commit: af52c9cd42965c1f7fd3fa0bcd4139673074e10c.

Final evidence commit: **2a32dfbeb29d0c204bba5a6c81f3e84df1f086b0**.

## SOL-R20-B01 — resolved

Determining clauses: §§3.3, 4.2, 16.1–16.2; SPEC-QUESTIONS 12/15. A positive
literal fits iff its magnitude bit length is at most N. A negative magnitude
fits iff its bit length is less than N, or equals N and is a power of two.
Thus 1, 0 and -1 fit both supplied positive widths. No theorem assumption or
spec amendment is needed.

| Blocker | RED tests / observations | Fix | Commit |
|---|---|---|---|
| SOL-R20-B01 | `r1_literals::r20_pinned_extreme_const_widths_parse_and_hash_without_panic`; `cli_xam::r20_cli_pinned_extreme_const_widths_check_and_hash`. Original N=max−1: exit 1, false §3.3 range error. Original N=max: exit 101, unchecked-add panic. | `object.rs::{const_literal,literal_magnitude,unsigned_literal}` | af52c9cd42965c1f7fd3fa0bcd4139673074e10c |

The original function was reinstalled temporarily, tested, and restored in a
`finally` block to obtain isolated RED evidence for both exact CLI witnesses:
`.tmp/r20-exact-red.log`. The library test failed 0/1; CLI stdout was empty in
both cases. An earlier focused library/CLI RED also rejected max−1.

GREEN: both full reviewer files are copied byte-for-byte into
`tests/support/r20-const-width-*.bisa`. Library admission runs under
`catch_unwind`, then `check_hashes`, and asserts both interface and closure.
CLI `check`/`hash` exit 0 with those identities; check status is `ok`.
`r20_extreme_const_execution_reports_host_resource` requires both sequential
execution and reversible-machine construction to return a cited host limit,
never panic or describe literal 1 as out of range.

| N | Interface = closure | Whole-file SHA-256 |
|---|---|---|
| 18446744073709551614 | 9146b47a91b5cade7648cdb4386bab0fc399048bd754674cb4745e79eeccb318 | 254bf985f88a26f94c13ff4e503cb1fff95ea5d4452f17422e194e73bb6eb367 |
| 18446744073709551615 | 97eb756124446d26628e2abbe5fb74223cbad5eb7902aacd31b7a309069f48f1 | 94416b11560b138ee17b68f5aa930d73c5c9521e343a533842a7d0d6aa0a8344 |

`tests/support/r20_pins.py` independently reconstructs each 269-byte envelope
using Python json/hashlib, without the interpreter or rehash helper. Both
reviewer files compare equal with `cmp`. Existing i8 256 / -129 negatives and
255 / -128 positives remain; the generated family adds 0, 1, -1, -2, 255,
256, -128, -129 against every extreme width, with independent expectations.

## Design and additional defects found by the sweep

1. Magnitude parsing reserves four bits per decimal digit plus
   limb padding, and scans only those bits. Neither N+1 nor N-bit storage is
   used at admission. Parser/resource errors propagate; only a mathematical
   range failure becomes the §3.3 out-of-range diagnostic.
2. Fault-code admission and certificate bundle/borrowed/cell endpoints had the
   same destination-sized allocation pattern. They now share the spelling-sized
   unsigned check. Canonical decimal endpoint ordering uses length then lexical
   order. The certificate family reproduced the old legal small-endpoint host
   failure with a misleading "outside bundle width" wrapper; that wrapper is
   gone. Actual claim execution still allocates concrete vectors fallibly.
3. `Function::signature` emitted large wire/reg and region A/W dimensions as
   JSON numbers, causing false §9.1 signature mismatches for canonical strings.
   It now emits numbers only through 2^53−1, decimal strings above it. The text
   family reproduced this mismatch before repair. Published identities remain
   unchanged; both new reviewer pins are preserved.
4. `Discipline::wires` and `borrow::Compiler::{fresh,declare_regions,
   borrow_compiler}` expanded scalar counts through infallible collect/push or
   unchecked cells*W. A large whole-wire operand reproduced a caught capacity
   overflow panic after the signature defect was repaired. Wire names, images
   and cell vectors now reserve fallibly; region products are checked.
5. The audit found unchecked storage growth in exhaustive claim enumeration
   and sample collection. Exhaustive interval cardinality, product, usize
   conversion and vector reservation are checked before enumeration. Sample
   counts reserve fallibly before drawing. These are host-resource failures,
   never an arbitrary ISA count cap. The unit family exercises the cardinality
   and reservation consumers directly, avoiding an unsafe pre-fix OOM run.
6. Quantum wire identities now reserve before collecting; CLI interface cells
   reserve before iteration and check cells*cell_bytes. Resource diagnostics in
   these paths, derived machine frames/indices, borrow-capacity sums and depth
   counters name the host limit and cite the relevant §. Qreg descriptor-sum
   overflow is distinguished from an invalid slice's mathematical range.

No new specification gap was exposed. Bead **b-isa-spec-61i** records the
recurring class; full-ISA staged work remains separate.

## Extreme-width path inventory

**102 fixture-qualified paths, 1,113 generated cases.** Every scalar path is
driven by 0, 1, 2^63−1, 2^63, 2^64−2, 2^64−1. Text uses canonical unsigned
INT; metadata uses numbers through 2^53−1 and decimal strings above it. Twenty
additional descriptor spelling controls include 00, +1, -0, 1.0 and 2^64.
The retained R13 values and its three u128 depth boundaries remain included.
Counts below exclude the two pinned objects and their execution/CLI controls.

| Path(s) | Where validated / consumed | Paths | Cases |
|---|---|---:|---:|
| Every layout scalar, qualified by five fixtures (listed below) | `Object::validate_layout`, `DerivedWidths::from_program`, `BitVector::zero`, machine construction | 41 | 574 |
| depth+2 / Dc | `DerivedWidths::from_program`, machine construction | 1 | 17 |
| constant width × eight signed/unsigned literals | `const_literal` | 1 | 48 |
| action wire, qalloc, qborrow, wire index, qloop/bounds list, qregion A, qregion W | `Type::parse`, `validate_action_sequence`, `Discipline::wires`, borrow compiler | 7 | 42 |
| iN, reg, wire, qreg, region A/W, plus 20 spelling controls | `Type::parse`, `host_count` | 6 | 56 |
| concrete bit storage | `BitVector::zero` | 1 | 6 |
| concrete wire storage | `QuantumState::allocate` | 1 | 6 |
| B (machine and sequential), heap, stack, depth | `validate_budget`, `DerivedWidths`, `OracleVectorMachine::new`, `run_oracle` | 5 | 30 |
| fault width with literal code 1 | signature construction, `unsigned_literal` | 1 | 6 |
| allocation length | `validate_row` pointer-range check | 1 | 6 |
| zext and trunc destination widths | `lifted_suffix`, `validate_row` | 2 | 12 |
| boundscheck length | `validate_row` pointer-range check | 1 | 6 |
| local region cells*W | `declare_region`, borrow compiler checked sizing/reservation | 1 | 6 |
| local region image size | `declare_region` exact power/product comparison | 1 | 6 |
| qreg.new, slice offset, slice length, concat | `validate_classical`, handle-view discipline | 4 | 24 |
| borrowed-capacity sum | public `Object::borrow_capacity` consumer, isolated from wire expansion | 1 | 6 |
| certificate borrowed width, bundle/borrowed lo/hi, sample count, table cell, frame width, bounds, gate count | TABLE and `validate_certificate_details` / `domain` | 10 | 60 |
| certificate region A, W, index, lo, hi | `certificate::domain` (direct semantic consumer) | 5 | 30 |
| exhaustive cardinality/product and sampled-element capacity | `claims::{enumerate,sample_elements}` (direct consumers) | 2 | 12 |
| schedule B/heap/stack/depth, inherited controls, step, generated ordinal, point ordinal | TABLE, `validate_schedule_details`, symbolic selector domains | 8 | 48 |
| CLI interface A and W | `build_action_state`, bit-vector/region allocation | 2 | 12 |
| **Total** | | **102** | **1,113** |

The 41 layout instances are:

- inc8/ptr32 and inc8/ptr128: A_tab, H, S_cells, heap_base, heap_limit_max,
  null_size, ptr_width (7 each).
- heap: those seven plus globals.g.base and globals.g.size (9).
- sum_db: those seven plus globals.zero.base, globals.zero.size,
  interface_regions.sum_db.db.base (10).
- tiny_memory: those seven plus interface_regions.tiny.db.base (8).

Numeric table capacities use checked/exact powers; only actual allocations
create entries. Array/table lengths, instruction/program counters, frame-cell
counts, ABI slot counts and ordinal worklists otherwise derive from resident
parsed collections, not an untrusted virtual length. Scalar lengths do not
control an unchecked eager allocation on the audited paths. Pointer lengths,
layout base+size, region shifts/products, qreg slices/concats and the guarded
selector lo−1/hi+1 operations were inspected. The selectors keep large domains
symbolic; loops retain transfer/cycle rules and acquire no trip cap.

### S1: independently assigned categories

`r13_generated_derived_width_extremes_family` retains all 507 original inputs
and adds 84: 41 layout paths × two new 2^63 boundaries plus two depth cases.
Expected categories are computed *before* production by independent §§7.1–7.2
range/non-overlap/capacity formulas and §§3.7/5.4 width arithmetic:
legal, layout rejection, deterministic host representation, or allocation-
dependent host storage. Allocation-dependent cases allow a successful concrete
construction or a cited host allocation failure; they are not advertised as
deterministic representation failures. The observed distribution is
**167 constructed / 342 layout rejections / 82 resource diagnostics**.

All generated families use `catch_unwind`; admission/range/resource results are
asserted against preassigned expectations, with § citation and host-limit
separation. The families are finite boundary coverage, not an exhaustive
u64-domain or resource-availability proof. Direct capacity/domain tests are
identified above; they do not claim completion of staged semantic claims.

## Validation and retained evidence

Clean baseline: **412 passed / 0 failed / 5 ignored**, **118.60 s** runner
sum over **33 result blocks**. Final: **425 passed / 0 failed / 5 ignored**,
**176.88 s**, **16 unit + 409 integration**, 0 doctests, the same 33 blocks.
Focused R20: **13 passed / 0 failed**, 0.15 s runner sum. The 591-case width
family reports 167 constructed / 342 layout rejections / 82 resource errors.

All five ignored sweeps: **5 passed / 0 failed**, **106.12 s** runner sum,
rerun in release mode on final code:

| Sweep | Full domain | Result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 pairs × 4 = 262,144 executions | 1/0; 1.09 s |
| 17-wire borrow | 131,072 entries | 1/0; 5.81 s |
| checked borrow certificate | 262,144 elements, subject and expected program | 1/0; 23.01 s |
| §6 differential | 200,000 programs; 70,931 accepted clean / 129,069 rejected | 1/0; 74.22 s |
| recursive prefix reversal | 256 inputs × all completion/first-fault prefixes | 1/0; 1.99 s |

Fmt/fmt --check and strict all-target offline Clippy pass with **0 warnings**.
The unchanged Python seed passes **2 example objects / 2 printed envelopes**.
Independent pins and both exact file comparisons pass. Logs under interpreter
`.tmp/`: `r20-{baseline-clean,exact-red,class-red,allocation-red,cert-red,
focused,widths,final,final-clippy,sweeps}.log`.

All round 4–20 tests are retained. In particular the R13 service/block_image
controls, R14 Y/table schema family, R15 frame/epsilon controls, R16–17 defaults,
R17 union-subtraction schedule oracle, R18 per-invocation deferred/bound budget
family, borrow/discipline positives and negatives and exact fixture hashes
remain in the full suite. Deliverables 8–11 retain their staged/Unverified
status; no literal other-value theorem is newly claimed proved.

The clean baseline was rerun in isolation against the unchanged HEAD
interpreter, restoring all working files in a finally block. Use
`.tmp/r20-baseline-clean.log`; the initial `.tmp/r20-before.log` picked up one
rebuilt R20 test binary during concurrent focused compilation and is explicitly
superseded. The final gates run on restored repaired code, with no production
or test-input changes during those gates.

Commands (each cargo command has the mandatory prefix):

```sh
ENV_PREFIX='env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp'
MANIFEST=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml
$ENV_PREFIX cargo test --offline --manifest-path "$MANIFEST"
$ENV_PREFIX cargo test --offline --manifest-path "$MANIFEST" r20_ -- --nocapture
$ENV_PREFIX cargo test --offline --manifest-path "$MANIFEST" --test r1_widths -- --nocapture
$ENV_PREFIX cargo test --offline --release --manifest-path "$MANIFEST" -- --ignored --nocapture
$ENV_PREFIX cargo fmt --manifest-path "$MANIFEST"
$ENV_PREFIX cargo fmt --manifest-path "$MANIFEST" -- --check
$ENV_PREFIX cargo clippy --offline --manifest-path "$MANIFEST" --all-targets -- -D warnings
python3 /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/tests/support/r20_pins.py
```

The Python seed is unchanged. Source/test changes belong only to
`reference-interpreter/`; the Beads auto-export is left for the orchestrator,
not included in the explicit-path commits. No foreign changes were staged or
committed by this implementer. Worklog ownership remains with the orchestrator.

# Editor's response to council round 1 (isa-ref-2026-10-29)

Round 1 graded snapshot `3dea965`. The codex job continued to `0164b5a`
before the handover, so some witnesses were already fixed when this round's
work began; the tables below say so, with the test that now pins each one.
Work since `0164b5a` is in the `reference-interpreter:` commits from `c3d8bfd`
onward. Every witness is a test in `tests/r1_*.rs`. The file `RED` column
reports that test run against `0164b5a` (it compiled against that tree unless
it says "API absent"): FAIL means it was red before the fix.

Verification at the commit that adds this file:
`cargo test --offline` (152 passed, 0 failed, 5 ignored);
`cargo test --offline -- --ignored` (all 5 sweeps pass, times and counts in
`PROGRESS.md`); `cargo clippy --offline --all-targets -- -D warnings`;
`cargo fmt -- --check`; `python3 reference-interpreter/isa_check.py`
unchanged and passing.

Statuses: **resolved** (the witness is a passing test and the cause is fixed);
**declined** (the spec sentence that decides it, with a test pinning the
behaviour); **partial** (what remains and why).

## Summary

| Blocker(s) | Topic | Status |
|---|---|---|
| fable R1-B1 = codex B01 = muse B1 | widths beyond 128 bits | resolved |
| fable R1-B2 = codex B12 = muse B2 | canonical INT/FLOAT, const range, suffix schema | resolved |
| fable R1-B3 = codex B03 | predicate versions; window opens at computation | resolved |
| codex B04 | qif polarity; loops compose | resolved |
| codex B05 | quantum-memory cleanup and restoration | resolved |
| muse B5 | loop bodies per iteration | resolved |
| codex B02 | region call graph includes lifted calls | resolved |
| codex B06 | local qregion passed to a child | resolved (at 0164b5a; pinned) |
| codex B07 | qinplace certificates / conditions | resolved |
| codex B08 | matched uncall identities; disjoint stores | resolved |
| codex B09 | whole-run reversal after an initial cancelled segment | resolved |
| codex B10 | ptr arguments to actions | resolved |
| codex B11 | classical definitions in any block order | resolved |
| codex B13 | malformed callee panics | resolved |
| codex B14 = muse B6 | borrow frames above 16 wires; library route | resolved |
| codex B15 | exact cleanliness, no 1e-24 threshold | resolved |
| codex B16 = muse B7 | typed metadata; checked claims | resolved (variant enum declined in part) |
| codex B17 | certificate composition at link time | resolved |
| codex B18 | fixture audit audits every fixture | resolved |
| codex B19 | target, launch, schedule, interval-checker modules | partial (staged as deliverables 8–11) |
| codex B20 | call-tier policy | resolved |
| muse B3 | effect operation table at load | resolved |
| muse B4 | `in` wire as qstore value | declined |

30 entries: fable 3, codex 20, muse 7.

## Widths: fable R1-B1, codex B01, muse B1 — resolved

The council's 129- and 256-bit witnesses already ran at `0164b5a` (the codex
job had added vector engines beside the u128 ones). Four narrowings remained
and were red at `0164b5a`: bundle widths were `u16` (wire[70000] rejected),
lifted suffixes were capped at 128 (`lift.const.i300`), layout exponents
H/A_tab/S_cells were capped at 127, and the machine's counters, cells and
slots were u128 rather than §3.7 derived widths. Worse, keeping two engine
pairs had let them diverge: the vector engine computed `ashr` by N or more as
a sign fill while §4.1 says "shift ≥ N → 0", and `tests/bitvectors.rs`
asserted the wrong value.

Changes:
- `src/object.rs` `Type`: `Wire(u64)`, `Reg(u64)`, `Region(u64, u64)`; only
  `iN` keeps 1..=128. `host_count` parses every width exactly; a width beyond
  64 bits is reported as a host limit, not an ISA rule.
- One engine pair. The u128 sequential engine and u128 machine are deleted;
  `src/oracle.rs` `run_oracle_vectors` is the §14.1 interpreter and
  `src/machine.rs` `OracleVectorMachine` the §3.6 machine. `run_oracle`,
  `OracleInput`, `OracleOutcome` and `OracleMachine` are checked adapters that
  fail loudly when a value does not fit u128.
- `src/machine.rs` `DerivedWidths`: Hc = H + 2, Ac = A_tab + 1, Sc = S_cells + 1,
  Dc = max(1, ⌈log₂(depth_limit + 2)⌉), P, Did, W_max, W_cell and per-slot
  widths; counters, stack cells and slots are bit-vectors of those widths,
  exchanged in their low bits with the high bits asserted clean (§3.7 "the
  cell's remaining bits stay clean").
- Layout ranges are exact up to 2^128 (`validate_ranges`); `inrange` is
  computed without wrap (`Memory::valid`).
- `src/bits.rs` `binary`: `ashr` by ≥ N is 0; `division_fault`, `compare`.
- `src/quantum.rs` and `src/action.rs`: the oracle-from-action adapter always
  uses the bit-vector machine (the former 128-bit `read_bits` is gone).

Tests (`tests/r1_widths.rs`): `codex_b01_wire129_identity_at_b0_and_b1`,
`codex_b01_reg129_move_with_bit_128_set_reverses`,
`fable_r1_b1_wide_add_is_2x_mod_2_256_in_both_engines` (x ∈ {0, 1, 2^200,
2^256−1} × B ∈ 0..=3, both engines plus full prefix reversal),
`fable_r1_b1_unused_wide_input_through_the_small_integer_api`,
`muse_b1_mov_of_2_255`, `wide_lifted_suffixes_constants_and_casts`,
`bundle_widths_above_u16_and_large_region_dimensions_are_well_formed`,
`ashr_by_n_or_more_is_zero_in_every_engine`,
`derived_counter_widths_follow_section_3_7` (H = 127 → Hc = 129; H = 128 → 130).
RED at 0164b5a: 4 of 9 FAIL (suffixes, u16 widths, ashr divergence, derived
widths: API absent).

SPEC-QUESTIONS: 4 (revised: the round-0 entry claimed derived widths the
code did not have), 14 (ptr_width ≤ 128 from §4.8's `iN (rlen, width
ptr_width)`), 15 (lifted suffixes name reg widths).

## Literals: fable R1-B2, codex B12, muse B2 — resolved

Already rejected at `0164b5a`: `fault 09`, `qloop 03`, `qalloc any 01`, the
non-canonical FLOATs. Red at `0164b5a`: `ptr_width=032` / `=+32`, `bounds
{03}`, CC `const.i8 256` (silently reduced), `load.ptr`, metadata `"16"` as a
string, and `call @f (x)` beside `call @f x`.

Changes: `host_count` is the one canonical-INT reader for the header, widths,
bounds, loop counts, allocation widths, region dimensions, qreg literals and
wire indices; fault codes use `BitVector::parse_unsigned_exact` (citing
§16.1); `const_literal` requires −2^(N−1) ≤ c < 2^N for `const.iN` and
`lift.const.iN`; `validation.rs` requires an `iN` suffix on
`zext/sext/trunc/ptrtoint/load/store`; `Json::uint_text` makes an integer a
JSON number iff it is at most 2^53 − 1; `call` takes bare operands only.

Tests (`tests/r1_literals.rs`): `header_ptr_width_is_a_canonical_int`,
`fault_codes_are_canonical_ints`, `type_widths_and_bounds_are_canonical_ints`,
`action_literals_are_canonical_ints`, `gate_angles_are_shortest_round_trip_floats`,
`classical_const_range_and_qreg_literals`, `opcode_suffixes_follow_the_schema`,
`metadata_integers_above_2_53_minus_1_must_be_strings`,
`classical_call_has_one_spelling`; `tests/oracle.rs`
`const_literals_outside_the_destination_range_are_rejected_not_reduced`
(replaces the round-0 test that asserted the reduction). RED: 7 of 9 FAIL.

SPEC-QUESTIONS: 12 (revised: rejection replaces the round-0 modulo reading),
17, 18.

## §6 discipline: fable R1-B3, codex B03, B04, B05, muse B5 — resolved

At `0164b5a` the codex job had already fixed the round-1 witnesses except one:
codex B03(b), `h t; h t; d = qcopy t; quncopy t d`, was rejected because the
closed-window set was permanent. A window now opens at its computation:
`Discipline::closed` counts window-closing events per wire and each `Creation`
snapshots the counts of its dependencies (`window_closed`).

The council asked for a sound checker, so I added a differential test that
does not depend on anyone's reading: `tests/r1_fuzz_discipline.rs` generates
random actions (compute, middle and possibly mutated uncompute sequences, qif,
qloop, nested predicates, local regions, qload/qstore) and requires every
statically accepted one to run exactly clean on all basis inputs and on a
superposed input, forward and adjoint, using the exact simulator (B15). It
found four unsound accepts, all also accepted at `0164b5a`:

1. `h d0; ccx d0 p d1; cx d1 t; ccx d0 p d1; h d0` released d0 entangled with
   t: recorded reads were not transitive. `note_reads` now records the
   transitive dependency set.
2. The same through a predicate: `change` rebuilt each operation key by
   splitting its version at `":context"`, which also occurs inside nested
   snapshots, so the key was wrong and the recorded reads silently fell back to
   direct reads. The key is now taken before formatting.
3. A diagonal phase on a perm value that depends on an `other` value kicks back
   onto it (`h d0; cx d0 d1; cz d1 t; cx d0 d1; h d0`): outstanding diagonal
   phases are now tracked (`Discipline::phases`) and block the adjoint of the
   `other` values they depend on.
4. Gate operation identities used predicate names, not versions, for `other`
   targets (the fable R1-B3 class): versions now include every control and
   effective-control wire's version.

The ignored 200,000-program sweep then accepts 82,797 programs and every one
runs clean (`PROGRESS.md`).

Tests (`tests/r1_discipline.rs`): `fable_r1_b3_predicate_flip_closes_the_window`
(both witnesses and both positive controls),
`codex_b03_predicate_versions_and_windows_start_at_the_computation`,
`codex_b04_qif_arms_are_complementary_and_loops_compose`,
`codex_b05_quantum_memory_cleanup_and_restoration`,
`codex_b05_child_call_advances_loaded_cells`,
`muse_b5_loop_bodies_are_validated_per_iteration`,
`fuzz_witnesses_of_entangling_reads_are_rejected`; each accepted positive is
also executed. RED: codex B03(b) and the fuzz witnesses FAIL; the others pass
(fixed by the codex job after `3dea965`).

SPEC-QUESTIONS 25: the literal §6.2 sentence "other values are released only by
the adjoint of the sequence that produced them" admits witness 1; the crate
implements the stronger reading T5 needs, with a proposed 0.9 wording.

## codex B02 — region call graph — resolved

`src/schema.rs` `validate_call_graph` builds the graph from structured region
calls, `lift.oracle_call`/`lift.oracle_uncall`/`lift.perm_call`, `qinplace`
pairs and `lift.call`, finds strongly connected components, and rejects any
component containing an edge other than `lift.call` (§5.1 "recursion exists
only through lift.call"). Test: `tests/r1_rules.rs`
`b02_region_recursion_through_lifted_calls_is_rejected` (the self-calling
oracle and the action → oracle → lift.perm_call cycle rejected; `sum_db` still
parses). RED: FAIL.

## codex B06 — local qregion as a call operand — resolved (at 0164b5a)

Fixed by the codex job after `3dea965`. Pinned by `tests/r1_rules.rs`
`b06_local_qregion_passed_to_a_child_action`: parent `qregion r 3 1 @img`
passes `r` to a child that loads cell p, XORs it into t and unloads; for all
p ∈ 0..8 and t ∈ {0, 1}, t ends as t ⊕ bit p of 0xa5, p is unchanged, and the
adjoint restores the state exactly.

## codex B07 — qinplace — resolved

At load (`validation.rs`, `qinplace` arm) @g and @ginv must be oracles mapping
one wire[n] input to a wire[n] target with no mem and no args and one fault
width (the §4.5 expansion names none). At execution (`action.rs`
`verify_inverse_pair`), before any state change, both oracles run on every
n-bit input under the actual token: FAULT_NONE everywhere and
g∘ginv = ginv∘g = id, else a §4.5 error. Tests (`tests/r1_qinplace.rs`):
`b07_zero_zero_pair_is_rejected_before_mutation` (B = 1 and B = 2, both t,
state unchanged), `b07_not_not_pair_executes_controlled_and_adjoint` (fails at
B = 2, inverts at B = 3, controlled on c, adjoint), and
`b07_interfaces_are_checked_at_load`. RED: 3 FAIL. qinplace had no test at
all before. SPEC-QUESTIONS 26.

## codex B08 — matched oracle uncall — resolved

`src/call_discipline.rs` `validate_matched_oracle_calls` is now a forward
dataflow over the row CFG: each `lift.oracle_uncall` must have an outstanding
`lift.oracle_call` with identical operands on every incoming path (multisets
meet by pointwise minimum), consuming one. An outstanding call is invalidated
by any write to its ins, tgt or fr other than oracle XOR delivery (which
commutes) and by a possible write to a supplied cell: a store whose address is
a known `lift.addr` constant outside the region does not invalidate; an
unknown address, a perm call with a mut region, or a `lift.call` whose closure
stores does. Tests (`tests/r1_rules.rs`):
`b08_two_calls_then_two_matched_uncalls` (input 8, tk = (3,0,0,0), B = 5:
(0, 0, 5); a third uncall is rejected), `b08_disjoint_global_store_between_a_matched_pair`
(store to global g at 32, region db at 16: accepted and run to (0, 0, 6);
store through `lift.addr @db`: rejected). RED: both FAIL.

## codex B09 — whole-run reversal — resolved

`src/classical.rs` `reverse`: history traversal undoes frame effects and skips
reversed frames and uncall markers (§7.4); afterwards the run's administrative
control (activation stack, next instance id, fault/finished/pending) is the
control recorded before the first frame, so a run whose first frame opens a
later-cancelled segment ends at its entry. Test: `b09_whole_run_reversal_after_an_initial_cancelled_segment`
(`main: call @f; uncall @f () (); ret`, `f: ret`; run(4) and reversal twice).
RED: FAIL.

## codex B10 — ptr arguments — resolved

`src/oracle.rs` `check_argument` is the one §3.3 classical-argument check for
oracles, actions and CC region calls (ptr included). Test:
`b10_pointer_arguments_reach_permutation_actions` (B = 2: (1, 0, 2), machine
agrees and reverses; p = 2^32 rejected). RED: FAIL.

## codex B11 — block order — resolved

`src/validation.rs` `validate_classical` types definitions to a fixpoint
(`classical_instruction`), independent of block text order. Reading a name
with no instance on the executed path is FAULT_UB_TRAP. Test:
`b11_classical_definitions_need_not_precede_uses_in_text` (run(4) = 7,
reversal restores entry). RED: FAIL. SPEC-QUESTIONS 24.

## codex B13 — malformed callees — resolved

`src/schema.rs` `validate_local_schemas` checks the §4.8 operand schema of every
structured and classical instruction before any pass reads callee bodies.
Test: `b13_malformed_callee_is_a_diagnostic_never_a_panic` (`Object::parse`
returns a §4.8 error; the CLI exits 1 citing §4.8). RED: FAIL (panic at
`call_discipline.rs:81`).

## codex B14, muse B6 — borrow frames — resolved

`src/borrow.rs` decides P_F = P′ ⊗ I_B exactly: every wire's final value is an
algebraic-normal-form polynomial over the frame's inputs (canonical, so the
decision covers the full domain); loops are composed by repeated squaring;
permutation actions are inlined; quantum memory is multiplexed exactly;
black-box oracle and qinplace calls are canonical symbols when their inputs do
not depend on borrowed wires. The 16-wire cap and the 10^6 loop cap are gone.
The translation lemma remains the structural route. A frame the decision
cannot settle is admitted only through verified certificate `borrow_frames`
claims (`claims.rs` `borrow_frames_certified`), the "library certificate"
route. `Object::verify_certificate` re-executes claims and rejects altered and
incomplete ones.

Tests (`tests/r1_borrow.rs`): `codex_b14_seventeen_wire_swap_identity_frame`
(512 samples, plus the ignored exhaustive 2^17 run),
`muse_b6_eighteen_wire_paired_cx_frame_and_its_dependent_variant`,
`borrow_frames_inside_bounded_loops_are_composed_not_unrolled` (X^(2^40)
accepted and executed; an odd count rejected; the order-3 map cubed accepted,
squared rejected). `tests/r1_certificates.rs`
`b14_checked_borrow_claim_and_altered_or_incomplete_claims` (a proof against an
identity program and against a full table; one wrong table row rejected; a
domain short of D is evidence and is rejected as a borrow_frames proof; a claim
without the borrowed bundle is rejected) and the ignored 17-wire claim. While
running the 2^40-iteration loop I found the executor ran it literally; it now
skips whole periods once its exact state returns to the loop-entry value.
RED: 3 FAIL; the claim tests had no API at `0164b5a`. SPEC-QUESTIONS 20, 21.

## codex B15 — exact cleanliness — resolved

`src/exact.rs`: every amplitude reachable by §4.4 gates lies in
Q(√2, i)[z, 1/z] with dyadic coefficients and z = e^{i·2^−1075}, because every
finite binary64 angle is an integer multiple of 2^−1074; z is transcendental
(Lindemann–Weierstrass), so a Laurent polynomial is zero iff its coefficients
are. `src/quantum.rs` carries this exact shadow beside the binary64 amplitudes
(which now serve only Born probabilities) and decides release, basis reads and
state equality exactly. The superposition form is sparse over arbitrary-width
basis keys, so the fixed 24-wire limit is gone too. Tests (`tests/r1_exact.rs`):
`b15_tiny_rotation_is_not_clean_and_release_changes_nothing` (ry(1e-13) and
rx(1e-300)), `b15_exact_adjoints_release_cleanly`,
`b15_cc_free_of_an_almost_clean_register_traps` (fault 4, handle still live; the
adjoint pair frees cleanly and reverses), `superpositions_have_no_wire_limit`
(a 200-wire GHZ state). RED: 3 FAIL.

## codex B16, muse B7 — metadata and claims — resolved (variant enum declined in part)

At `0164b5a` the codex job had already added typed recursive schemas: the
council's schedule witnesses (boolean, null and negative budgets, phase
"garbage", malformed paths, negative start time) and certificate witnesses
(`versions:false`, false bindings, false composition fields, `{"bogus":1}`
claims, unknown fields) were rejected there; `tests/r1_certificates.rs`
`b16_schedule_values_are_typed` and
`b16_muse_b7_certificate_values_and_claims_are_typed` pin them. Added now:
variants contain no `:` (keys are `<variant>:<controls>`), domain bounds are
exact at any width (`Json::uint_text`), and claims are checked, not just
parsed (`src/claims.rs`, B14). Declined in part: codex objects to
`variant: "banana"`. The specification never lists variant names (§6.4, §11,
§12 only use "variant" as a key component), so rejecting a name would invent
an enum; SPEC-QUESTIONS 11 proposes 0.9 wording.

## codex B17 — certificate composition — resolved

`src/identity.rs`: closure hashes over pinned imports use the pin (§9.3 makes
the linked closure equal it), so a client's closure hash, and its certificate
region, exist before linking. `src/linking.rs` `compose_certificates` unites
region records and multiplicities (equal where they meet), rebinds every hash
including `<hash>:<variant>:<c>` keys and changed schedule hashes (a rebinding
collision is an error, never an overwrite), requires equal versions, dialect
mapping and target bindings, and recomputes `epsilon_total` exactly
(`exact::Decimal`). Tests (`tests/r1_linking.rs`):
`b17_certified_client_and_callee_link_and_compose` (both regions retained,
bindings re-validated by the re-parse, epsilon_total = 3 × 0.25 × 2 + 0.1 =
1.6, client(8) with tk = 3 → (9, 0, 2)), `b17_incompatible_certificates_are_link_errors`,
`interface_region_names_cannot_alias_symbols` (fable's §9.2 suggestion). RED:
3 FAIL. SPEC-QUESTIONS 22.

## codex B18 — fixture audit — resolved

`tests/r1_fixtures.rs` enumerates `tests/fixtures/*.bisa`
(`b18_every_bisa_file_is_audited_and_has_a_comments_file`), shows a corrupted
tiny_memory hash is detected (`b18_a_corrupted_stored_hash_is_detected_by_the_same_audit`),
and checks the inc8 and sum_db fixtures byte-for-byte against the §16.3 text
(fable's suggestion). The hand-listed loop in `tests/identity.rs` is removed.
At `0164b5a` the hand list happened to be complete; at `3dea965` it omitted
tiny_memory.

## codex B19 — target, launch, schedule and interval-checker modules — partial

Not built in this milestone. `PROGRESS.md` stages them as deliverables 8–11
(launch ABI §10 / suite G; target descriptions §8 / suite E; schedule consumer
§11 / suite H; rigorous interval certificate checker §12.1 / suites C and D),
with their contents and order. This milestone built what B07, B14, B16 and B17
needed: claim checking (exhaustive, sampled, translation-lemma and library
structural claims), the library borrow route, composition, and the §17.13
step-check suite mode. The rest is reported, never silently skipped: every
claim needing the missing machinery (`numerical`, `channel`,
`expected:"oracle"`) is reported `Unverified` with its reason.

## codex B20 — call tiers — resolved

`validation.rs` `classical_instruction`: an R function calls only R functions
and Q regions; a CC function calls CC or R functions; `uncall` follows the same
graph, and a CC segment containing `measure` traps when uncalled (§7.4). The
`registers.bisa` wrapper `measuring` is now CC (fixture rehashed, comments
updated). Test: `b20_call_tiers_follow_the_section_3_1_graph`; the existing
`tests/registers.rs` measurement test covers the trap. RED: FAIL. CC traps are
now marked by the trap conditions themselves (`classical.rs` `trap`), no longer
inferred from any error whose text starts with "§3.3:" (fable C5).
SPEC-QUESTIONS 19.

## muse B3 — effect table at load — resolved

`validation.rs` `validate_effect` checks every §7.3 device operation's name,
operand count and ptr/integer types, and requires the model to be declared;
an unmodelled device stays legal and faults 8 at run time (§17 item 3). Test:
`muse_b3_effect_operations_are_validated_at_load` (`clock.read p p`,
`clock.read`, `clock.write p`, `console.write p` rejected). RED: FAIL.

## muse B4 — `in` wire as a qstore value — declined

§4.6 defines `qstore name a v` as "for each i: under `a == i`, `qxor v
cell_i`", and §5.1 admits `in` wires "as `qcopy`/`qxor` sources": the stored
value is a qxor source and is never modified. `qexchm` exchanges its value
into a cell, so an `in` value there is rejected. Test:
`muse_b4_qstore_value_is_a_qxor_source_and_may_be_in`. SPEC-QUESTIONS 23
proposes adding "qstore values" to the §5.1 list.

## Suggestions also taken

Memory::free now follows the §4.2 row (SPEC-QUESTIONS 16); one `call`
spelling (18); CC trap codes recorded (19); SPEC-QUESTIONS 1, 7 and 9 carry
proposed wording; `check` reports `"unlinked"` with the unresolved imports;
`step-check` has no fixed bit ceiling and a §17.13 `--suite` mode
(SPEC-QUESTIONS 27); hand-written gate matrices with disabled controls and the
xoshiro256** reference vector (`tests/r1_gates.rs`); parameter roles must agree
with their header group (muse; `parameter_roles_agree_with_their_header_group`);
interface regions may not alias symbols; every fixture's symbol hashes are
pinned as literals (muse; `fixture_identities_are_pinned_as_literals`);
invariant `expect`/`unreachable!` on
validated paths became section-citing errors; the crate is rustfmt-formatted
and clippy-clean.

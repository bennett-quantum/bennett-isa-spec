# Bennett Virtual ISA — Specification, draft 0.9

**Date:** 2026-10-29. **Status:** draft 0.9, **not yet graded**. It
revises draft 0.8, which was **accepted by the council in round 8**
(2026-10-28; reports and the editor's responses under
`docs/council/isa-2026-10-28/round-{1..8}/` in the `bennett` meta
repository) and which remains the contract every other repository
targets until this draft is accepted. §0 lists every change from 0.8.
Implementation is expected to expose frictions; revisions go back
through the council. Founding text of
`bennett-isa-spec` (`Bennett-NorthStar.md` §4.4, §6.1); the north star
governs where this draft is silent. The reference interpreter implements
0.8; nothing of this draft is implemented. MUST, MUST
NOT, SHOULD, MAY are normative; *(non-normative)* explains; *(deferred)*
names items left open; every deferred decision that an implementer must
take carries a **default** in §17, which implementers follow until the
item is revised.

**Versioning.** ISA 0.9, object 0.9, launch 0.9, sched 0.9, cert 0.9,
coverage 0.9, target 0.9. The **full version string** (`0.9`) is part of
every semantic envelope (§9.2): drafts are not compatible with each
other, so every hash of every 0.8 object differs under 0.9 (§0, §16.3).

---

## 0. Changes from 0.8

Every change from the accepted draft 0.8 has one entry below; nothing
changes without an entry. **Kind**: *E* = editorial (wording, or a
reading pinned that a conformant 0.8 implementation could already take,
with local semantics unchanged); *S* = semantic (some outcome or
well-formedness verdict differs from some reading of 0.8, or a
capability is added). **Hash**: every entry rides on the version bump V,
which changes envelope part (1) and every signature's `abi`, hence the
hash of **every** symbol (§9.2): the four 0.8 example hashes all change
and §16.3 republishes their objects (the 0.8 vectors remain valid 0.8
vectors); "+bytes" marks an entry that also changes the canonical bytes
of functions that use the affected form; "new" marks new symbols.
Sources: *EN* = an item of the evidence note
`docs/notes/2026-10-29-isa-09-evidence.md`; *bd* = a bead of this
repository; *SQ n* = entry n of `reference-interpreter/SPEC-QUESTIONS.md`
(its reading is adopted unless the entry says otherwise); *RC* = the
reference-interpreter council round 1
(`docs/council/isa-ref-2026-10-29/round-1/report.md` in `bennett`),
member and finding; *r8* = a council round-8 suggestion (bead
`b-isa-spec-zsw.N`); *DL* = the dialect 0.2 outline's §25 list.

**No narrowing.** No entry rejects, faults, declares inert or removes a
capability that 0.8 permits or the north star promises. The entries
that add or sharpen a well-formedness check (E11's static view rule,
E16.9, E17.2, E17.3, E17.5's unmatched uncalls, E17.8, E17.9, E17.10,
E17.12, E17.14, the block terminators of K5) concern spellings that
always have a legal equivalent or uses that 0.8 did not define soundly
(E17.14: a location value as a classical argument would be superposed);
the run-time checks (E11, K1's width checks, K3's token validity, K4's
inverse-pair check) concern operations that 0.8 did not define or
already required (0.8's `qinplace` already demanded its certificates;
K4 admits library certificates as well as exhaustive claims, as the
north star §3.1 requires).

### 0.1 Group 1: repairs and pinned readings

| # | § | Change | Closes | Kind | Hash |
|---|---|---|---|---|---|
| V | title, §1.3, §6.4, §8–§13, §9.1, §9.2, §10, §16.3 | Full version string `0.9` in every sub-version, envelope part (1), every signature `abi` and every device-model version | brief | S | all |
| E1 | §3.6, §4.2 | The index space indexes **rows** (one per instruction line); a label denotes its block's first row; `qjumpr`'s range is the containing function's row indices | EN I8; bd ti2; SQ 1 | E | – |
| E2 | §3.6 | Lprocs are placed by SCC: a one-member SCC keyed by its member's closure hash, a multi-member SCC by its `scc_hash`, members consecutively in SCC index order | bd was; SQ 7; EN "SCC ordering"; RC fable s6, muse s1 | E | – |
| E3 | §9.2 | Acyclic symbols hash their own envelope; the SCC wrapper applies to cyclic SCCs, including a self-referencing singleton (pins the published vectors; r8 zsw.11 rejected) | EN ISA-10; zsw.11 | E | – |
| E4 | §16.1 | A data line with empty `HEX` is `NAME SP LF`, the sole trailing-space exception | bd oim; SQ 9 | E | – |
| E5 | §7.4 | Logical history (checkpoint/replay storage permitted); active versus audit history; an `uncall` marker is always skipped and nothing is undone twice | EN I4, I7; bd ql5; zsw.3, zsw.10 | E | – |
| E6 | §4.1 | The `uncall` summary states every §7.4 legality condition and the cancellation-aware traversal | zsw.1, zsw.9 | E | – |
| E7 | §12.1 | Invocation contexts come from the realised invocation tree, including default-covered invocations; context keys and their order are concrete | zsw.2, zsw.7 | E | – |
| E8 | §12.1 | Margin rule for numerically obtained SDP witnesses; non-normative Tr_out convention note | zsw.6, zsw.12, zsw.13 | E | – |
| E9 | §14.3 | T4 states its hypotheses (reverse eligibility, no `measure`, no unverified release) and restores *active* state | zsw.3, zsw.8 | E | – |
| E10 | §16.3, §15 | Example 3 (`borrow_demo`, an action with loop bounds and a translation-lemma borrow frame); suite D gains two SDP witness vectors | zsw.15; zsw.4 (part) | E | new |
| E11 | §3.3 | `qreg` values pass unchanged through `call`, `ret` and `br`; state belongs to the register; aliases; the exact static rule for views and repeated operands (suite F), every other overlap or use of a viewed register a run-time trap | DL "qreg across CC calls"; EN BLK-1; review of this draft | S | – |
| E12 | §3.3 | `qreg.slice` `lo` is a literal or a run-time `i64`; view state on reversal of the region call | EN ISA-5; DL "qreg.slice" | S | +bytes |
| E13 | §5.4 | Code 10 is not "R only": a lifted oracle virtualising entropy may `fault 10` | DL ISA §5.4 item; EN ISA-7 | E | – |
| E14 | §3.6, §4.2, §5.4 | **Effective cause**: a `fault 0` or triggered `lift.faultif p 0` records `FAULT_UB_TRAP` (4); no fault is ever recorded as `FAULT_NONE` | bd 7qu; EN "explicit fault code zero" | S | – |
| E15 | §3.1 | Call-tier policy as a table (CC calls CC and R; R calls R; `uncall` of CC or R segments) | RC codex B20 | E | – |
| E16.1 | §16.1 | The text form has no comment syntax | SQ 2 | E | – |
| E16.2 | §3.6 | An idle step writes `old_pc = HALT`; its inverse clears it (as printed) | SQ 3 | E | – |
| E16.3 | §3.7 | Counter widths: the reference realisation uses the minimum widths; wider counters give the same F_B | SQ 4; RC muse C4 | E | – |
| E16.4 | §14.1 | On budget exhaustion the reference interpreter has executed exactly B rows | SQ 5 | E | – |
| E16.5 | §9.1 | `devices.models` lists every §7.3 model an `effect` uses; an unmodelled device contributes no device line | SQ 6 | E | – |
| E16.6 | §10 | A host reply whose length does not fit `ptr_width` bits is refused with `E_ARG` before any write | SQ 8 | E | – |
| E16.7 | §3.3 | Fault literals in an lproc are checked against every enclosing oracle's F, again at link | SQ 10; RC muse s5 | E | – |
| E16.8 | §9.1, §9.4, §12.3, §13 | JSON encodings: type strings, program-point arrays, frame descriptor, certificate `versions`, variant identifiers, coverage field presence | SQ 11; DL "ISA-3 editorial" | E | – |
| E16.9 | §3.3 | A `const.iN`/`lift.const.iN` literal lies in [−2^(N−1), 2^N) (departs from SQ 12's modulo reading; NS §0 "never a silent wrong answer"; every value keeps a spelling) | SQ 12; RC fable R1-B2 | S | – |
| E16.10 | §16.1 | Canonical `FLOAT` spelling (RFC 8785 number rendering, `.0` appended, `-0.0`) | SQ 13; RC fable R1-B2, codex B12 | E | – |
| E17.1 | §3.3, §14.1 | `wire[N]`, `reg[N]`, cells and `qreg[n]` have no upper width bound; values are bit-vectors of any width | RC fable R1-B1, codex B01, muse B1 | E | – |
| E17.2 | §16.1 | Every integer token is a canonical `INT` (`032`, `+3`, `09` are ill-formed) | RC fable R1-B2, codex B12, muse B2 | E | – |
| E17.3 | §5.1 | The region call graph, including lifted calls and both `qinplace` callees, is acyclic | RC codex B02 | E | – |
| E17.4 | §4.7 | A caller's local `qregion` may be a region operand of a structured call | RC codex B06 | E | – |
| E17.5 | §5.5 | Matched `lift.oracle_uncall` pairs match LIFO; an intervening store is not ill-formed | RC codex B08 | S | – |
| E17.6 | §4.7 | Classical argument types `iN ptr f64 token` and how each is read in a lifted body | RC codex B10 | E | – |
| E17.7 | §3.4 | Uses need not follow definitions in block text order; a use with no current instance traps | RC codex B11 | E | – |
| E17.8 | §4.1, §9.1 | `load`/`store` take `.iN` suffixes only (no `load.ptr`); JSON numbers ≥ 2^53 are errors | RC codex B12 | E | – |
| E17.9 | §4.1 | `call` writes its arguments as bare operands (`r = call @f a b`); the parenthesised spelling is ill-formed | RC fable s2 | E | – |
| E17.10 | §9.1 | An interface region's name differs from every symbol and import name (a rename always exists) | RC fable s9, C2 | S | – |
| E17.11 | §5.1 | `in` wires may also be `qstore` value operands, memory-address operands and `qif` selectors | RC fable s10, muse B4 | S | – |
| E17.12 | §5.1 | Param role suffixes: optional on wires and equal to the group's role; required on regions | RC muse s3 | E | – |
| E17.13 | §4.2, §7.1 | `lift.free` / `free` match only unpinned entries (0.8 also cleared the live bit of a pinned size-0 global at the freed base) | RC fable s1, C3 | S | – |
| E17.14 | §4.2 | The `args` of lifted calls are classical arguments, never locations; Q14 (§0.2) supplies them to lprocs through explicit parameters | review of this draft; Q14 | S | – |
| E17.15 | §3.3, §4.2 | A lifted row's `.iN`/`.iM` suffix names a location width and has no upper bound (except `lift.load`/`lift.store`), so wide locations can be initialised and cast | review of this draft (after E17.1) | S | – |
| E17.16 | §3.6, §4.2 | `qjumpr` to an in-range index that is not a block's first row enters that row (the kept 0.8 sentence read "any other index faults" under block indexing); re-indexing preserves every reachable row index | review of this draft (after E1) | E | – |
| E17.17 | §2, §3.3, §10, §17 item 14 | **Verifying** versus non-verifying executors defined; cleanliness tests (`qreg.free`, host prepare/free) stated for both | review of this draft | E | – |
| D1 | §17 item 15, §8.2–§8.3, §11 | Routing/movement cost deferred to 0.10; optional topology and placement decided by K14 | zsw.16; b-isa-spec-wr7 | S | – |
| D2 | – | Deferred r8 suggestions: zsw.4 (the controlled-action target/schedule/certificate fixture waits for `ref`, §17 item 7), zsw.5 (citation unverified), zsw.14 (`ref` artefact, §17 item 7) | zsw.4, .5, .14 | – | – |

### 0.2 Group 2: capability contracts

| # | § | Change | Closes | Kind | Hash |
|---|---|---|---|---|---|
| K1 | §3.2, §3.3, §4.1, §4.8, §7.4, §9.1, §10, §16.2 | **Run-time-width registers and handle lookup**: type `qreg[*]`; `qreg.new` with an `i64` count; `qreg.width`, `qreg.id`, `qreg.from_id`; a register ledger (ids executor-chosen, never reassigned); width checks at every `qreg[*]` → fixed-width transfer; `measure.iN` (also on `qreg[n]` with N = n); run-time slice `len`; `QREG` binds `qreg[*]`; ownership acquired by `qreg.from_id` lasts until CLOSED; host calls on a register another open run owns are `E_INVALID_STATE` | EN BLK-1; DL BLK-1; bd dw7 | S | +bytes |
| K2 | §3.3, §4.1, §4.8, §7.4 | **Quantum allocation in R**: `qreg.*` execute in CC and R; ledger frames and their reversal (`qreg.new` ↔ release, clean by construction; `qreg.free` ↔ re-creation with the same id); `measure` stays CC-only | EN BLK-4; DL BLK-4 | S | +bytes |
| K3 | §3.2, §4.1, §4.8, §5.1, §5.4, §7.4, §10 | **Device-minted budgets**: `budget.mint @o s h st d` in CC and R with `bennett_budget`'s checks; token validity (run-scoped, revoked by reversal); a token may be supplied to any region call of the program; an invocation executes the object's schedule only when its bindings (tuple and `params_hash` included) match, else the §11 defaults; a token is encoded in `params_hash` as its tuple | EN BLK-2; DL BLK-2; bd 4qy; bd 8wn (binding, found while drafting) | S | +bytes |
| K4 | §4.5, §5.3, §5.4, §6.1, §12.1–§12.3 | **Parametrised in-place**: `qinplace @g @ginv tk x (ins..) (mem..) (args..)`; signature compatibility; inverse-pair obligation carried by an `inverse_pairs` entry with a minimum tuple, exact `arg_tokens` tuples and an `args` domain, by an exhaustive claim (`expected: "inverse_pair"`) or a library claim (`inverse_pair_library`), covering both orientations, checked before any effect of the CC/R region call (executors check inverse-pair claims, §1.2); monotonicity in the tuple; §6.1 transfer with `ins`, `mem` cells and `other` preserved; adjoint swaps the pair and keeps the lists | EN BLK-3; DL BLK-3; bd gv7; RC codex B07, muse s4 | S | +bytes |
| K5 | §3.4, §4.1, §4.9, §5.4, §7.4, §10, §14.1, §15, §17 item 16 | **Classical fault contract (ISA-7, ratified A5)**: CC trap = R fault record = one terminal record; a cause → code table; `fault C` in CC and R (literal < 2^64, effective cause); an `effect` memory operand out of range is 7; CC/R blocks end in `br`, `ret`, `trap` or `fault`; a `FAILED` run has exactly one record (0.8: "at least one"). Round-1 follow-up (2026-10-29): exact nonzero C through the launch API, no subsequent source effect, intact reversible prefix, and explicit classical charge versus Q step-machine semantics | EN ISA-7; DL ISA-7; bd 221; RC fable C5, s4; dialect draft 0.2.1 §25.1 BLK-5, council D02-21 re-derivation, b-dialect-spec-bz6 | S | +bytes |
| K6 | §7.4, §10, §14.3 | **Reversible failed runs**: R faults are terminal; `FAILED → REVERSED` under the shared eligibility (no `measure`, no unverified free, ownership); reversal undoes the record, then the rest including open activations (an `effect`'s undo also removes its tape entry); T4 covers `FAILED` runs, the ledger entries of the registers a run created or touched and token validity, and states the exactness hypothesis for quantum state (exact when every undo is the exact inverse of its realisation, else within the declared bounds) | EN ISA-8; DL ISA-8 (terminal reading, DL §17.6); bd 0b9 | S | – |
| K7 | §3.3, §17 item 14 | 0.8's §17 item 14 default (a `FAILED` record for a condition hardware cannot test) contradicted §7.4's unverified frees: non-verifying executors release unverified and exclude `uncall` and reversal; a verifying executor records `FAULT_UB_TRAP` (host calls: `E_QREG_NOT_CLEAN`) | found while drafting; bd exc | S | – |
| K8 | §16.3, §15 | Examples 4–9 (one per new family, hand-derived results) and the suite rows for every new instruction and fault | brief | E | new |
| K9 | §8.1 | **EPS-DECADE (ratified A5)**: exact zero-based decimal accuracy bucket, domain and exact-entry prohibition; boundary witness | bennett-vm-cxg; b-isa-spec-f31 | S | – |
| K10 | §7.3 | **BLOCK-IMAGE (ratified A5)**: contiguous complete 512-byte sectors, byte order, partial-image rejection at device initialisation; witness | bennett-vm-47i; b-isa-spec-0d0 | S | – |
| K11 | §11.1, §17 item 7 | **ISA-REF-LIB (ratified A1)**. Decided serialization, envelope hash, ordinals, scratch/removable rules and decomposition set; witness only, publication gated on accepted 0.9 | bennett-vm-47t; b-isa-spec-8r4 | S | new |
| K12 | §10, §10.1, §12.3 | **LAUNCH-ESTIMATE (ratified A2)**. Resolved parameters, class-preserving resource keys, upward decimal encoding and exact certificate binding; witness | bennett-vm-hsd; b-isa-spec-c15 (open) | S | – |
| K13 | §9.2, §12.1–§12.3 | **CLAIM-REPLAY (ratified A3)**. Mandatory sampled expected relation, standalone invocation binding, canonical parameter and full-table tuple encoding; witness; sampling stays evidence | bennett-vm-clw; b-isa-spec-9uy (open) | S | – |
| K14 | §8.2–§8.3, §11, §17 item 15 | **Topology (ratified A6)**: optional ordered class vertices and undirected edges; native adjacency, scheme/placement binding; absent all-to-all; routing deferral with explicit-gate construction | b-isa-spec-wr7; b-targets-dd3; circuits b4k | S | – |
| K15 | §5.3 | **Multi-block actions (ratified B1)**: text-order unitary fragments, descriptive labels, cross-block lifetimes | b-isa-spec-2c9; circuits 7bu | E | – |
| K16 | §9.1, §9.4, §12.3 | **Frame JSON (ratified B5)**: identified slots and typed scratch/history/borrow/ancilla records, c-only width expressions, disjoint shared lifetimes | b-isa-spec-2c9; circuits 4fa; SQ 11 | S | – |
| K17 | §4.1 | **Draw ordering (ratified A5)**: FIFO draws; wire-first lexicographic outcomes, strict cumulative threshold; reference CLI witness | b-isa-spec-bas; SQ 29 | S | – |
| K18 | §0, §3.7, §4.2, §4.9, §5.4, §7.3, §8.1 | **Confirmations (ratified A4/A5)**: Q14 explicit forwarding; EPS-DECADE, BLOCK-IMAGE, ISA-7 and Q1–Q13 confirmed as applied | b-isa-spec-7u6; b-isa-spec-f31; b-isa-spec-0d0; b-isa-spec-221; b-isa-spec-2c9 | E | – |
| K19 | §11.1 | **REF-BINDING**. Typed integer/wire/body/symbol/environment bindings, lexical substitution, invocation-local ordinals and validation witnesses | b-isa-spec-cbk | S | new |
| K20 | §11.1 | **REF-QUANTUM**. Binary64 angle bits, variadic wires and full-basis matrix witnesses | b-isa-spec-8sc | S | new |
| K21 | §11.1 | **REF-IMAGE**. Bound image bytes emit unconditional LSB-ordered X leaves with an exact adjoint | b-isa-spec-afi | S | new |
| K22 | §11.1 | **REF-TEMPLATES**. Indexed typed body templates bind explicit frame actuals while retaining lexical captures | b-isa-spec-lph | S | new |
| K23 | §11.1 | **REF-LITERAL-BITS**. Exact nonnegative integer bit extraction fixes literal X emission | b-isa-spec-8r4 | S | new |
| K24 | §11.1 | **REF-ARGUMENTS**. Closure-bound positional argument vectors and exact ISA classical/token values | b-isa-spec-1a4 | S | new |
| K25 | §11.1 | **REF-DESCRIPTORS**. Ordered typed records for byte layouts and heterogeneous activation locations | b-isa-spec-1oa; b-isa-spec-evi | S | new |
| K26 | §11.1 | **REF-REGIONS**. Shape, role, origin and cell bindings preserved in closure arguments | b-isa-spec-1a4 | S | new |
| K27 | §11.1 | **REF-LIFETIMES**. Exact class actuals and shared allocation declarations across invocations | b-isa-spec-3xe | S | new |
| K28 | §11.1 | **REF-INSTRUCTION-RELEASE**. Recipe scratch follows reverse allocation order; instruction-defined release sequences remain exact, with the qinplace ordinal witness. [deps: ISA-4.5-P5, ISA-11.1-P21] | b-isa-spec-82x; maintainer ruling 2026-10-31 | S | new |
| K29 | §5.3 | **QIF-ADJOINT**. Complete-expansion inverse for cross-arm lifetimes; §5.1 matches lifetimes region-wide, not arm-wise; witness: `qif p { d = qalloc any 1 } { qfree d }` recreates d before freeing it in the inverse. | INV-1 gap 2; b-isa-spec-bdv; SQ 28 | S | +bytes |
| K30 | §4.6 | **LOOP-BINDINGS**. Visit-local linear binding carry-over; witness: a body frees c then reallocates c, and the next visit uses the new c. | INV-1 gap 3; r9_mixed_loop_threads_reallocated_bindings | E | – |
| K31 | §5.3 | **MATCHED-PAIRS**. Callees, classical/token values, operand versions and effective control sets identify adjoints; witness: opaque oracle pairs under stacks [p,q] and [q,p,p] match. | INV-1 gap 4; r10_equivalent_control_sets_preserve_opaque_inverse | E | – |
| K32 | §3.7 | **LOCATION-DECLARATIONS**. Locations only in oracles/lprocs; witness: an action `regs { ghost:wire }` cannot manufacture a live wire; empty regs with bounds remains legal. | INV-2 gap 1; r11_action_regs_do_not_produce_wires | E | – |
| K33 | §3.3 | **WIRE-DISTINCTNESS**. Distinct denoted wires, not spellings; witness: w and w[0] overlap, and two w[0] operands overlap. | INV-2 gap 2; tier_arity_aliasing_and_input_roles | E | – |
| K34 | §3.3 | **REGISTER-FRESHNESS**. Fresh allocation disjoint from live/viewed registers; launch ids pairwise disjoint; witness: duplicate launch id returns E_ARG before execution, while from_id aliases remain references to one register. | INV-2 gap 3; register_handles_enforce_views_consumption_and_cleanliness | E | – |
| K35 | §3.3 | **CALL-ALIASES**. Atomic CC/R fault for overlapping qreg call slots, including control/target overlap; witness: the same id in two slots faults without consuming a view or mutating wires. | INV-2 gap 4; r4_dynamic_interface_alias_is_an_atomic_reversible_cc_and_r_fault | E | – |
| K36 | §6.1 | **REGION-PROVENANCE**. Image-relative cell version zero, perm, empty dependencies; witness: a nonzero image is the initial version, not outstanding garbage. | INV-3 gap 1; local_regions_require_explicit_restoration_and_load_pins_cell_versions | E | – |
| K37 | §6.2 | **UNDONE-CHANGES**. Matched permutation pairs with intervening basis-value preservation; witness: X on a dependency followed by X reopens the original wire binding, but any cell write invalidates its old version. | INV-3 gap 2; cleanup_windows_and_linear_lifetimes | E | – |
| K38 | §6.2 | **CLEANUP-VERSIONS**. Computation-time S and produced g versions current; witness: qcopy s then quncopy s d is legal without requiring the old zero d version. | INV-3 gap 3; cleanup_pins_predicate_versions_and_distinguishes_qif_polarity | E | – |
| K39 | §6.2 | **CLEANUP-CONTROLS**. Computing and cleanup effective control sets/versions agree; witness: compute under p, cleanup under q cannot discharge the obligation. | INV-3 gap 4; cleanup_pins_predicate_versions_and_distinguishes_qif_polarity | E | – |
| K40 | §6.2 | **OTHER-READERS**. SQ 25 outstanding-reader condition; witness: H d0; CCX d0 p d1; CX d1 t; CCX d0 p d1; H d0 cannot release d0. | INV-3 gap 5; b-isa-spec-0jo; SQ 25 | S | – |
| K41 | §6.1 | **OUTSTANDING-DEPENDENCE**. Exact residual-operator factorisation defines dependency jointly on live/interface values and phases, retaining transitive effects through cleaned scratch; witness: t still depends on d0 after d1 is cleaned, and S on d0 leaves a phase until SDG cancels it. | INV-3 gap 6; SQ 25; cleanup_adjoint_cannot_skip_an_entangling_reader | S | – |
| K42 | §5.6 | **TRANSLATION-CONTROLS**. Effective context included in every gate control set; inverse V has the same set/versions; witness: inherited p lies in X, never in a W target or B. | INV-3 gap 7; translation_lemma_checks_all_four_required_negative_patterns | E | – |
| K43 | §5.6 | **STATIC-FRAMES**. Maximality within containing qloop/qif static sequence, per lifetime; witness: outer touches and inner frame at ordinals 4–7 remain separate. | INV-3 gap 8; SQ 30; muse_r5_translation_frame_maximality_is_within_its_static_sequence | E | – |
| K44 | §4.3 | **STATIC-FREE**. qalloc version or explicit legal cleanup output at release; witness: fresh qalloc/free and H/H/free pass, H/free fails. | INV-3 gap 9; other_scratch_requires_the_exact_adjoint_but_accepts_h_h | E | – |
| K45 | §9.2 | **STRUCTURAL-OWNERSHIP**. Exact JSON owner/global keys determine closure selection; witness: oracle a excludes a.b even if its region suffix coincides. | INV-4 gap 1; r11_oracle_layout_ownership_uses_exact_names | E | – |
| K46 | §9.2 | **LAYOUT-ORDER**. Emit every structured leaf once, sorted bytewise by its JCS path; witness: both a/c.db and a.c/db survive, independently of their base values. | INV-4 gap 2; b-isa-spec-4it; SQ 31 | S | +bytes |
| K47 | §9.2, §16.3 | **INJECTIVE-PATHS**. JCS key-component arrays preserve leaf identity; witness: ["interface_regions","a","c.db"] differs from ["interface_regions","a.c","db"]. Prefer arrays: dialect §16.3 emits dotted block names and §12 Naming emits dotted temporaries; although its reference layout uses ref, no general ban on dotted oracle/region NAMEs is established, and a dot ban would narrow legal ISA names. Also disposes of INV-4 gap 5: structural selection is encoded regardless of any CC/R harm analysis. | INV-4 gaps 3, 5; b-isa-spec-ccv; SQ 31 | S | +bytes |
| K48 | §9.2 | **ENVELOPE-BOUNDARIES**. Grammar distinguishes function/data payload and suffix classes without new length fields; witness: a lhs named layout gives `layout = …`, never a layout-array line, and a data symbol named data has one payload line selected by its signature kind. | INV-4 gap 4; §16.2 grammar lemma | E | – |
| K49 | §11 | **INVOCATION-CHAINS**. Only region calls open schedule invocations; lproc rows retain their enclosing oracle chain; witness: two lift.call sites reaching the same lproc row give the same chain. | INV-5 gap 1 | S | – |
| K50 | §11 | **NESTED-BUDGETS**. Each nested oracle uses its token budget, independently of its parent; witness: root B=2 and nested B=3 include nested step 3; nested B=2 excludes it. | INV-5 gap 2 | E | – |
| K51 | §11 | **NONSTEP-SELECTORS**. Absent oracle coordinates match only wildcard step/phase; witness: an action-root X matches */* but neither step 1 nor phase rev, including adjoint X. | INV-5 gap 4 | E | – |
| K52 | §11 | **SELECTOR-LEVEL**. The last oracle-call element in the entry path selects the invocation whose step/phase it reads; witness: path [] step 1 reads the root even inside nested step 3; a path ending at the nested call reads nested step 3. | INV-5 gap 5 | S | – |
| K53 | §11 | **MAXIMAL-TIES**. Equal maximal lexicographic tuples at distinct array positions tie; shadowed overlaps do not; witness: duplicate C rejects; tied A/B covered by higher C (or a union of higher entries) admit in every order. | INV-5 gap 6 | S | – |
| K54 | §11 | **KIND-DOMAINS**. Gate, allocation and start decisions match only their operation classes; witness: gate and alloc entries with identical selectors cannot tie with each other. | INV-5 gap 7 | E | – |
| K55 | §11 | **TIE-CHECK-STAGES**. Admission checks known invocation domains; unknown nested domains check the first ambiguous concrete operation before its realisation; witness: unknown nested B=3 rejects at nested step 3; B=2 never generates it; a derivable B=3 rejects at admission. | INV-5 gap 9–10; b-isa-spec-6hs | S | – |
| K56 | §11 | **ENTRY-DEFAULTS**. Omitted payload fields take their defaults without inheritance from less specific entries; witness: a more-specific realisation-only gate entry uses the default share despite a broad explicit share. | INV-5 gap 11 | E | – |
| K57 | §17 item 7 | **DEFAULT-REF-BINDING**. The omitted expansion binds the object ISA version and its published library envelope hash; witness: a 0.9 object binds ref@0.9 at the §11.1 full-library hash, not the qxor witness hash. | INV-5 gap 12–13 | E | – |
| K58 | §17 item 9 | **SHARE-POPULATION**. An eps-independent exact-native test fixes N over the root expansion including nested invocations; witness: an exact native gate is excluded; three gates lacking an exact native match give N=3 even with explicit shares. | INV-5 gap 14 | S | – |
| K59 | §17 item 9 | **RATIONAL-DEFAULT-SHARE**. Exact reduced-rational resolved shares preserve §8.1 evaluation without rounding or modifying schedule bytes; witness: epsilon 0.1 with N=3 resolves to (1,30); N=0 assigns zero without division. | INV-5 gap 15 | S | – |
| K60 | §11 | **SCHEDULE-DECIMALS**. Schedule decimal values are exact rationals, independent of spelling; witness: 0, "0", "0.0" and "0.000" all denote exact zero. | INV-5 gap 16 | E | – |
| K61 | §11 | **UNCOVERED-NESTED-DEFAULTS**. Uncovered operations take defaults directly; no synthetic entry or multi-level selector is required; witness: root step 1 keeps scheme X while an uncovered root-step-2/nested-step-1 gate takes its default. | INV-5 gap 17 | E | – |

**Decided Q14 (applied 2026-10-29, confirmed 2026-10-30; b-isa-spec-7u6).** Lprocs take explicit
token and classical parameters (§3.7, §4.2, §4.7, §16.2), forwarded by
`lift.call`; mapped-region parameters preserve the complete lifted-call
interface. Fixed binding instances (§3.6–§3.7) keep those arguments
classical under coherent calls, without an extra budget charge (§5.4).
The interface and its hash include the new groups (§9.1–§9.2); §15 and
§16.3 example 10 supply conformance obligations and a witness. Source:
the maintainer's acceptance recorded as "the Q14 edit (lproc
token/classical params)" in the "interpreter milestone 1" session,
`worklog/001_2026-10-28_bootstrap.md` line 26; the orchestrator's
2026-10-29 decision selects explicit parameters. The maintainer's
2026-10-30 ratification confirms this reading: lproc header groups are
`(params)(result)(tokens)(classical)(mem)` and `lift.call` forwards tokens,
classical arguments and mapped regions.

**Decided confirmations (2026-10-30).** The maintainer ratified EPS-DECADE
(K9), complete-sector BLOCK-IMAGE (K10), the reference CLI draw ordering
(K17), ISA-7 `fault C` and its fault-code table (K5), and Q1–Q13 as
already applied. These decisions are recorded in the maintainer memo,
Ratification section, 2026-10-30. Q12 keeps the §16.3 draft example
hashes as placeholders until the 0.9 precheck; independently checked target/frame witness hashes
are published in §8.3 and §9.4.

### 0.3 Group 3: optional efficiency semantics (deferred)

| # | § | Change | Closes | Kind | Hash |
|---|---|---|---|---|---|
| P1 | §17 item 17, §3.6 | Identity-idle step (old index encoded as k ⊕ HALT, idle frames zero), deferred with the printed step as default and its normative hook | EN I5; bd roy | E | – |
| P2 | §17 item 18, §5.7, §9.4 | Checkpointed and pebbled oracle realisations, deferred with full history as default and the expansion-library hook | EN I1; bd 9c3 | E | – |

**Considered and deferred** (efficiency in the evidence note; no 0.9
text; the capability is kept by the stated construction, so no rung and
no north-star promise depends on the deferred primitive):

| Item | Construction that keeps the capability |
|---|---|
| ISA-2 (R-tier budget) | an R step counter threaded through the code; exhaustion is now `fault 1` (K5) |
| ISA-5 (two simultaneous slices) | one action on the parent register (e.g. `cx q[0] q[7]`); run-time indices by dispatch or by run-time slice `lo` (E12) |
| ISA-6 (non-consuming measurement) | CNOT copy of the wires into a fresh register, then `measure` of the copy; the original keeps its id |
| ISA-9 (borrowing driven from CC) | a clean allocation, or a Q region with its own borrow frame |
| ISA-11 (host memory at native addresses) | promotion of the pointer-sharing callers to CC (`alloc`, `hostcall` for genuine host effects) |
| G-12 (run-time snapshot at native addresses) | replay every allocation in bump order (dead and zero-size ones included) and copy the bytes |
| G-13 (interface base chosen at call time) | static arenas and explicit logical-to-physical address translation rows |
| G-14 (step machine inside actions) | bounded, padded structured expansion with immutable selectors |
| G-15 (payloads wider than 128 bits) | realised directly: `tgt`, `reg[N]` and `result_val` have no width bound (E17.1), and lifted suffixes name location widths (E17.15) |
| G-16 (fibers, interrupts) | explicit continuation records in memory and polling at defined points |
| I2 (bounded-loop recovery) | §5.7 structure recovery with immutable selectors or retained history |
| I3 (budget provenance) | host tuple or bounded wrapper; dialect documentation |
| I6 (step-granular oracle interface) | whole-oracle budget ladders of region calls |
| SQLite E1–E5 | logical stack and frames as specified; compare tree plus static `lift.call`; a copy lproc; classical-data specialisation under §5.7 |
| measuring registers wider than 128 bits (bd 04w) | copy 128-bit slices to fresh registers, measure the copies, reset the source wire by wire with run-time slices |

## 1. Scope and conformance

### 1.1 What the ISA is

A program model for classical, reversible and coherent computation with
one semantics, executed by the VM (`bennett-vm`) and the circuit
specialiser (`bennett-circuits`), produced by the plugin
(`bennett-plugin`) through the dialect's reference semantics, scheduled
through §11, described for hardware by target descriptions (§8). Nothing
here refers to LLVM.

### 1.2 Conformance classes

| Class | Who | Obligation |
|---|---|---|
| Executor | VM, specialiser, hardware stacks | §3–§7, §14 exactly, and the checking of inverse-pair claims (§4.5, §12.1; a certificate applies when its `closure_hash` binding equals the region's); suites A–E, H |
| Producer | plugin, library compilers | well-formed objects (§9, §16) and regions (§5, §6); suite F |
| Scheduler | `bennett-sched` and competitors | §11; only schedules valid under §6; suite H |
| Checker | `bennett-conformance` | validates certificates (§12) without trusting producers |
| Target author | vendors | target descriptions (§8); suite E |

### 1.3 Deferred *(asserted non-semantic)*

Binary encodings (§16 text is the interchange form); mechanisation of
§14; device models beyond §7.3; the gate-name registry process; the
memory layout of the opaque C objects of §10–§11 (every constructor,
accessor and destructor is listed). **Minimality claims** (north star
§5) are not supported by cert 0.9 and MUST NOT appear in a certificate.
Decisions deferred to implementation, each with its default, are listed
in §17.

## 2. Terminology

- **Tier.** CC, R, Q. **Wire**, **bundle** `wire[N]` (little-endian).
- **Location** `reg[N]`: a named mutable cell of an oracle or lproc
  register file. **Pseudo-location**: an oracle interface input or
  classical argument, or an lproc classical parameter (§3.7), readable
  only in its declaring function, never written.
- **Region**: a Q function with a declared interface: **lifted oracle**
  (§5.2) or **quantum action** (§5.3); **permutation action** (§5.3).
- **lproc**: a lifted procedure with its own params and locations; it
  resolves names only to its own params and locations (§3.7).
- **Budget**: classical (B, heap_limit, stack_limit, depth_limit).
- **Frame**: per machine step: `old_pc`, slots (assignment, capture and
  delivery slots, §3.6), `flag`, `succ`.
- **Basis configuration**: classical values on every component of D.
- **Gate classes.** Permutation: `x cx ccx mcx swap`. Diagonal: `s sdg t
  tdg cz rz`. Non-diagonal: `h rx ry`.
- **Verifying executor**: an executor that can decide exactly whether a
  set of wires is clean on every basis configuration (a state-vector or
  branch-enumeration simulator); every other executor (hardware) is
  **non-verifying**. Rules that depend on the difference say which
  applies (§3.3 `qreg.free`, §10, §17 item 14).
- **Effective control context** at a point: the set of wires on the
  active predicate stack plus the inherited control context (the control
  wires of every enclosing `ctl` and predicate stack of every enclosing
  invocation, transitively); duplicates count once. Its size is the
  **effective control count**.

## 3. Machine model

### 3.1 Tiers and the control invariant

```
  CC ──calls──▶ R functions (call/uncall)       CC ──calls──▶ Q regions
  R  ──calls──▶ R functions, Q regions          Q  ──calls──▶ Q regions (§5.5); oracle bodies call lprocs
```

CC runs on the device's classical-control processor; the host reaches
the device only through §10. **No instruction in any tier makes a
control or scheduling decision that depends on the value of a wire**
except through predication (§3.5). Gate angles are finite `f64` literals
or classical arguments fixed at region entry.

**Call-tier policy** (suite F). The diagram, completed with same-tier
calls; a call, uncall or region call outside this table is ill-formed:

| Caller | `call` (§4.1) | `uncall` of a completed segment (§7.4) | region calls (§4.7) | lifted calls (§4.2) |
|---|---|---|---|---|
| CC function | CC and R functions | of a CC or R callee | oracles, actions | – |
| R function | R functions | of an R callee | oracles, actions | – |
| oracle body, lproc | – | – | – | `lift.call` of lprocs; `lift.oracle_call`/`lift.oracle_uncall` of oracles; `lift.perm_call` of permutation actions |
| action | – | – | oracles, actions (§5.3, §5.5) | – |

An R function never calls a CC function: CC code may measure and make
host requests (§4.1), which an R history cannot reverse.

### 3.2 State

| Component | Tier | Contents |
|---|---|---|
| classical environment | CC, R | dynamic SSA instances (§7.4), block parameters, activation stack, **quantum-register handles** (`qreg[n]`, `qreg[*]`, §3.3), tokens (§5.4) |
| register and token ledgers | CC, R | every register's id, class, width and state (§3.3); every device-minted token's tuple and validity (§5.4) |
| classical memory | CC, R | byte-addressed, little-endian, sparse; allocation table (§7.1) |
| effect tape, entropy tape | R | §7.3 |
| run history | CC, R | one frame per executed instruction (§7.4) |
| wire store | Q (CC and R via `qreg`) | live wires with roles `input target fault borrowed mutable workspace location cell` |
| quantum memory | Q | regions of `wire[W]` cells: local (§4.6) or interface (§5.1) |
| register file | Q | locations of the active oracle and its lproc closure (§3.7) |
| predicate stack | Q | wires; the active predicate is their conjunction |
| oracle step state | Q | `pc, pc' : wire[P]`; `faulted : wire`; `code : wire[F]`; `result_val : wire[T]`; counters (§3.7); B frames |
| budget | Q | classical: B, heap_limit, stack_limit, depth_limit |

### 3.3 Types, ownership, operand schema

```
  iN (1 ≤ N ≤ 128)   ptr (width ptr_width)   f64   token   qreg[n]   qreg[*]   region[A,W]
  wire   wire[N]      linear values in structured code and interfaces
  reg[N]              a location (oracle bodies, lprocs); not linear
```

**Widths.** The bound 1 ≤ N ≤ 128 applies to the type `iN` only (a lifted
row's `.iN` suffix names a location width, §4.2). `wire[N]`,
`reg[N]` and `qreg[n]` have N, n ≥ 1 and `region[A,W]` has A ≥ 0, W ≥ 1,
with no upper bound (a `qreg[*]` register's width is a run-time value n
≥ 0, §3.3 below); a value of these types is a bit-vector of its
width in every semantics (§14.1), including interface bundles,
pseudo-locations, locations, cells, stack cells and frame slots.

**Linear SSA with implicit versioning** (structured code and CC region
calls): a `wire`/`wire[N]` value is produced once and consumed once;
mode `in` consumes and re-produces a successor of the same name;
`consume` consumes; `produce` creates; a name is not used after
`consume`. **Regions are threaded the same way.** **Indexed operand**
`w[i]` (literal i) borrows wire i for one instruction.

**Disjointness.** The wire operands of one instruction are pairwise
distinct; **every wire of the effective control context is distinct from
every target and from every consumed or produced operand** of an
instruction executed under it (so `qpred p; x p` is ill-formed); a
predicate wire is not modified between its push and its matching pop; a
`qif`/`qloop … while` selector is not modified inside its arms or body.
A region's interface bundles and regions are pairwise disjoint.
Distinctness is judged on the wires denoted: a bundle and an indexed
operand of it, or two indexed operands denoting the same wire, overlap
and are not distinct. [deps: ISA-3.3-P3, ISA-4.4]

**Per-opcode operand schema** (§4.8): every opcode has a tier set, a
fixed arity, operand kinds and a result type; results are written first
(`w = op a b`), modes are never written, and **result types that are not
determined by the operands are written as a suffix** (`r = const.i32 5`,
`r = zext.i64 a`, `r = trunc.i8 a`, `r = ptrtoint.i64 p`, `p = inttoptr
a`). Width rules: `binop`/`icmp` operands equal width; `select` arms equal
width; `zext`/`sext` strictly widen, `trunc` strictly narrows; `load.iN`
and `lift.load.iN` require N ∈ {8, 16, …, 128}; `measure` of `qreg[n]`
with n ≤ 128 yields `in`, and `measure.iN` (1 ≤ N ≤ 128) of a `qreg[*]`
yields `iN`; signed division truncates toward zero. The
literal c of `const.iN c` and `lift.const.iN c` satisfies −2^(N−1) ≤ c <
2^N and denotes c mod 2^N (both the signed and the unsigned spelling of
every value are accepted); any other literal is ill-formed (suite F).

**Lifted width rules** (suite F; a mismatch is ill-formed, never a
truncation or an extension). Every address operand (`a` of `lift.load`/
`lift.store`, `p` of `lift.boundscheck`/`lift.free`) and every address
result (`w` of `lift.addr`/`lift.alloc`) is `reg[ptr_width]`; the length
`n` of `lift.alloc`/`lift.boundscheck` is a location of any width or a
literal in [0, 2^ptr_width); `inrange` and every prospective sum are
computed at width max(widths of the operands involved) + 1 with no wrap.
The destination of `lift.label` and the operand of `qjumpr` have any
width N with 2^N > n (the index-space size). `lift.call` location
arguments have exactly the callee's parameter widths and its destination the callee's
result width; its token, classical and region lists match the callee's
corresponding groups in count, order and type (§3.7); `lift.ret v` has
the containing lproc's result width;
`lift.oracle_call`/`lift.oracle_uncall`/`lift.perm_call` operands have
exactly the callee signature widths, and `mem` operands name caller
regions whose `[A,W]` equal the callee's and whose role is `in` for an
`in` slot (an `in` region is never passed to a `mut` slot); `fault C` and
`lift.faultif p C` require the literal C < 2^F, where F is the fault
width of the containing oracle or, in an lproc, of every oracle whose
lproc closure contains it (checked at load and again at link; an lproc
in no oracle's closure is checked when linked into one); the width rules above
for `binop`, `icmp`, `select`, `zext`, `sext`, `trunc` apply verbatim to
their `lift.*` counterparts; the destinations of one `lift.parmov` are
pairwise distinct.

**Quantum registers in CC and R.** A **register** is n device-resident
wires of one class with a nonzero 64-bit **id**, recorded in the
program's **register ledger** with its class, width and state:
**live**, **viewed** (one slice of it is live) or **consumed**.
`qreg[n]` is the type of a register whose width n is fixed statically;
`qreg[*]` the type of a register whose width is known only at run time
(an unsigned 64-bit n ≥ 0). A `qreg[n]` value may be used wherever a
`qreg[*]` is expected; a `qreg[*]` value bound to a `qreg[n]` parameter
or block parameter (`call`, `br`, `ret`, launch) or to a slot of width n
(region call, §4.7) is checked at that transfer: its width MUST equal
the required one. The `qreg.*` instructions execute in CC and R;
`measure` only in CC. An instruction whose precondition fails is a CC
trap / R fault (§4.9) with code `FAULT_UB_TRAP` unless the table says
otherwise, and has no effect. An id's value is chosen by the executor
(the suites compare ids only for equality); an id is never reassigned to
another register while the program is loaded. **Handle transitions:**

| Instruction | Precondition | Effect |
|---|---|---|
| `q = qreg.new cls n` | `n` a literal n ≥ 1 (q: `qreg[n]`) or an `i64` value read unsigned, 0 included (q: `qreg[*]`); the device can provide n wires of class `cls`, else code `FAULT_BUDGET_HEAP` | q live, clean, with a fresh id and wires disjoint from every live or viewed register. [deps: ISA-3.3-P7] |
| `qreg.free q` | q live, not a view, clean on every basis configuration (a verifying executor checks it; on failure q stays live and unchanged; a non-verifying one: §17 item 14) | q consumed |
| `q2 = qreg.slice q lo len` | q live, not a view; `lo` and `len` each a literal (`len` ≥ 1) or an `i64` value, both unsigned; lo + len ≤ n, computed without wrap (a trap even when literals make the failure evident) | q2 a view of wires [lo, lo+len) of q, of type `qreg[len]` for a literal len and `qreg[*]` otherwise; q viewed; q2 MUST be the operand of exactly one region call and of nothing else; after that call q2 is consumed and q is live again; a use of q while it is viewed traps (the static rule below covers the cases suite F rejects). Reversing that region call (§7.4) restores q viewed and q2 live; reversing the slice frame then restores q live |
| `q = qreg.concat q1 q2` | q1, q2 live, not views, distinct registers | q1, q2 consumed; q live, with a fresh id, over the wires of q1 then q2; `qreg[n1+n2]` if both operands are static, else `qreg[*]` |
| `w = qreg.width q` | q live, not a view | w: `i64`, the width; no state change |
| `id = qreg.id q` | q live, not a view | id: `i64`, the bits of q's id; no state change |
| `q = qreg.from_id id` | `id` is the id of a register of the program that is live, not a view, and not owned by another run that is not CLOSED (§10) | q: `qreg[*]`, an alias of that register; no ledger state change; the run now owns the register (§10), which reversal does not undo (ownership ends when the run is CLOSED) |
| region call (§4.7) with q as operand | q live (a view included); its width equals the slot's (§3.3 binding); the wire sets filling distinct wire/region slots are pairwise disjoint, and an explicit control is disjoint from targets and consumed/produced operands (read-only input overlap remains allowed), else CC trap / R fault FAULT_UB_TRAP before any effect, including any view consumption. [deps: ISA-3.3-P4, ISA-4.7, ISA-4.9] | q unchanged. [deps: ISA-4.7] |
| `r = measure q` / `r = measure.iN q` | q live, not a view; the form `measure.iN` (1 ≤ N ≤ 128) for a `qreg[*]`, whose width MUST be N (checked before measuring); for a `qreg[n]` either `measure` or `measure.iN` with N = n (checked statically) | q consumed, its wires released; r: `in` / `iN` |
| reversal of a `qreg.*` frame (§7.4, §10) | – | the ledger state before that frame (below). A `measure` frame is **never reversed**: a segment containing one cannot be uncalled (§7.4) and a run containing one is not reversible (§10) |

**Ledger frames and their reversal** (§7.4). A `qreg.new` frame records
the id, class and width; its reversal releases that register without a
test: every later frame touching it has been undone first (LIFO) and the
run owns it exclusively (§10), so under T4's exactness hypothesis it is
clean, and otherwise its residual, within the declared bounds of the
undone realisations (§12.3), is discarded with it.
A `qreg.free` frame records the id, class, width and whether its
cleanliness was verified; its reversal re-creates the register clean and
live with the same id, class and width (every alias sees it live
again). A `qreg.slice` frame records the parent id, lo and len, a
`qreg.concat` frame the two operand ids and the new id; their reversal
restores the exact prior states and wire order without moving, copying
or measuring a wire. `qreg.width`, `qreg.id` and `qreg.from_id` change
no ledger state and their frames are pure (§7.4); if an `uncall` later
removes the register an alias refers to, every use of the alias traps.

**Binding**: a register of width n (`qreg[n]`, or `qreg[*]` of run-time
width n) fills a `wire[n]` slot bit for bit (bit 0 ↔ wire 0); it fills a
`region[A,W]` slot iff n = 2^A·W with cell i = bits [iW, (i+1)W); for a
`qreg[*]` the width is checked at the call, before any effect.
Host-supplied register ids are `qreg` values of the parameter's type.

The register ids supplied as arguments of one `bennett_run` MUST
denote pairwise disjoint registers, else `bennett_run` returns `E_ARG`
before executing the entry function or acquiring any register ownership.
[deps: ISA-3.3-P7, ISA-10]

**Registers, handles and aliases.** A `qreg` value is a reference to a
register of the device; the state live / viewed / consumed belongs to
the register, not to a name. `qreg` values pass unchanged as `call`
arguments, `ret` values and `br` block arguments (each transfer binds
another name to the same register and changes no state; a `qreg[*]`
reaching a `qreg[n]` parameter is width-checked, above). Two names bound
to one register (by such transfers or by `qreg.from_id`) are
**aliases**: a transition
made through either is observed through both. The wire operands of one
region call, with its effective control context, MUST denote pairwise
disjoint wires (§3.3 disjointness); aliases violate it. **Static and
run-time verdicts.** Suite F rejects exactly: two `qreg` operands of one
region call or of one `qreg.concat` that are the same name; and, on some path from a
`qreg.slice`, a use of the view's name other than as the operand of one
region call (a `br`, `call` or `ret` transfer included), its reaching a
`ret` or the end of the function unconsumed, or a use of the slice's
parent operand name before that call. Every other overlap, and every
use of a viewed register through another name, is a CC trap / R fault
with code `FAULT_UB_TRAP` before any effect.

### 3.4 Execution in CC and R

Sequential small-step execution with an instruction pointer, block
parameters instead of phi nodes, dynamic control flow and allocation;
every executed instruction pushes a run-history frame (§7.4).

**Names and definitions.** Every name used in a CC or R function is a
parameter, a block parameter or the result of at least one defining
instruction of that function, and all its definitions give it one type
(suite F). The text order of blocks is irrelevant: a definition need not
precede its uses textually. Executing a use of a name that has no
current instance (none was created on the executed path, or an `uncall`
removed it, §7.4) is a CC trap / R fault `FAULT_UB_TRAP`. Every CC or R
block ends in `br`, `ret`, `trap` or `fault` (suite F); `trap` and `fault`
end the run (§4.9).

### 3.5 Predication

Every Q instruction with unitary semantics executes controlled on the
active predicate. `qpred w` pushes, `qunpred w` pops the same wire; the
stack is empty at region entry and exit. Allocation, borrowing, release,
region creation and image gates are **unconditional**; only gates
(including those inside copy/XOR/exchange/memory instructions) are
controlled.

### 3.6 The predicated step machine

**Index space.** `I[0..n)` = the **rows** (one index per instruction
line) of the oracle's blocks, blocks in text order and rows in text
order within each block; then the rows of every lproc in its **lproc
closure** (transitively reachable through `lift.call`, linked ones
included), placed as follows: partition the closure into its SCCs
(§9.2); a one-member SCC's **order key** is its member's closure hash, a
multi-member SCC's order key is its closure `scc_hash`; SCCs are placed
in increasing bytewise order of their keys (64-hex), the members of a
multi-member SCC consecutively in SCC index order; each lproc's binding
instances (§3.7) consecutively in binding-key order, each instance's rows
in block text order as for the oracle. The oracle and each lproc
instance therefore occupy a contiguous range of indices, and a **label
`^b` denotes the index of the first row of block b**. `HALT = n`; 2^P > n. Branch and call
targets are labels; `w = lift.label ^b` is the only way a location
obtains an index value; executors MAY re-index dispatch extensionally
**provided every label value observable through `lift.label` keeps its
canonical numeric value and a `qjumpr` to any in-range index reaches the
row that canonical index names**. **Terminators**: `qjump qbranch qjumpr lift.
ret lift.retval fault`; every block ends in one; no label names HALT;
**only `lift.retval` reaches HALT with success**.

**Row classes.** Each §4.2 row is an **assignment** (A: written
locations/cells receive values computed out of place into frame slots
`disp_j`, then exchanged in, so `disp_j` retains the displaced old
value), a **direct** row (D: a controlled permutation in place), or
**mixed** (A+D) with an explicit ordered list of **elementary
sub-operations** printed in §3.7. **Slot kinds.** An **assignment slot**
receives a value computed out of place and is then exchanged with its
destination (it retains the displaced old value). A **capture slot**
receives a value computed out of place and is never exchanged (it
retains that value as history: the arguments of `lift.call`). The
**delivery slot** of `lift.ret` is a capture slot that is subsequently
exchanged, under a condition, with the destination location. Rows with
a **flag** compute it into `frame.flag` from the old state before any
effect; rows with a data-dependent successor compute it into `frame.
succ` before any effect; conditional-fault rows perform `faulted ^=
flag`, `code ^= flag·C′` after their other effects, where C′ is the
row's **effective cause** (§5.4: C′ = C, except C′ = `FAULT_UB_TRAP`
when a `lift.faultif` literal C is 0).

**Frame layout** (fixed per oracle): `(old_pc : wire[P], disp_1..disp_
{k_max}, flag : wire, succ : wire[P])`, `k_max` and widths `W_j` from the
widest write sets used (§4.2 lists each row's slot use).

**Forward machine step** t (1 ≤ t ≤ B), reads referring to the state at
the start of the step:

1. **Dispatch.** `at_k = (pc == k)` for k = 0..n into clean scratch.
2. **Execute.** For each k < n in index order: push `at_k`; run `I[k]`'s
   row: (a) compute `flag` into `frame_t.flag` if the row has one; (b)
   compute `succ` into `frame_t.succ` if data-dependent; (c) for A rows,
   compute **every** written value into its slot, then exchange **every**
   slot with its destination (all computes precede all exchanges); for D
   rows apply the row's permutation; for A+D rows follow the row's listed
   order; (d) apply the fault writes; pop `at_k`.
3. **Successor.** For each k < n, under `at_k`: XOR the row's successor
   into `pc'` (a label index under `flag`/`¬flag` as stated, or
   `frame_t.succ`). Under `at_HALT`: XOR `HALT` into `pc'`.
4. **Commit pc.** `qexch pc pc'`; `qexch pc' frame_t.old_pc` (now `pc` is
   the next index, `pc'` is clean, `old_pc` holds the old index).
5. **Clean dispatch.** Uncompute every `at_k` from `frame_t.old_pc`.
6. Decrement the classical step counter.

On an **idle** step (pc = HALT at its start) no row executes, step 3
XORs `HALT` into `pc'`, and step 4 leaves pc = HALT and `frame_t.old_pc` =
HALT; the inverse step clears it. (A variant that leaves idle frames at
zero is deferred, §17 item 17.)

**Inverse machine step** (prefix reversal runs it for t = B..1), in
this order:

1. Recompute every `at_k` from `frame_t.old_pc`.
2. `qexch pc' frame_t.old_pc`; `qexch pc pc'` (restores the old `pc`;
   `pc'` again holds the successor value; `old_pc` is clean).
3. For each k < n under `at_k`: XOR the row's successor into `pc'` again,
   using `frame_t.flag` / `frame_t.succ`; under `at_HALT`: XOR `HALT` into
   `pc'` (`pc'` is now clean).
4. For each k < n in **reverse index order** under `at_k`: undo the fault
   writes (`faulted ^= flag`, `code ^= flag·C′`); then undo the row's
   effects. **D row**: its sub-operations in reverse order, each by its
   own inverse (`lift.perm_call` by the callee's adjoint; XORs,
   controlled flips and exchanges by themselves). **A row**: exchange
   every assignment slot back with its destination **in the reverse of
   the forward exchange order** (the forward order is the row's listed
   slot order; every destination address, forward and inverse, is
   evaluated from the state current at that exchange, so an indexed
   destination such as `alloctab[atop]` is addressed with the counter
   value that the earlier inverse exchanges have already restored), then
   **uncompute every slot from the restored old state** (the same
   out-of-place computation, applied again as XOR, clears it).
   **A+D row**: **the exact reverse of its complete §3.7 sub-operation
   list**, each elementary sub-operation inverted at its reversed
   position: a direct sub-operation by its own inverse; an assignment
   sub-operation by exchanging its slot back and then uncomputing the
   slot; a capture sub-operation by uncomputing the slot (its source has
   been restored by the sub-operations already undone); the delivery
   sub-operation by the same conditional exchanges. **Every address and
   condition is evaluated from the state current at that position of the
   inverse** (so the `lift.ret` stack addresses and delivery condition
   use the callee `fp` restored one position earlier, and the `lift.call`
   record base uses the restored `sp`); §3.7 prints both inverse lists.
   Finally uncompute `frame_t.succ` and `frame_t.flag` from the restored
   old state.
5. Uncompute every `at_k` from `pc` (which now equals the old index).

After the inverse step every frame field is clean. *(non-normative)* The
forward step is a permutation Π_t; the inverse step is Π_t⁻¹ written
out; T2 relies on nothing else. B = 0 is legal: the oracle commits
`FAULT_BUDGET_STEPS` on every input.

### 3.7 Register files, procedures, counters, stack

**Name resolution.** A read operand of a lifted row in an **oracle body**
is a location of the oracle or a pseudo-location (an interface input or
a classical argument). A read operand in an **lproc** is a param or
location of that lproc; nothing else. Token and region parameters are
usable only in their operand kinds below, not as bit-vector locations.
A written operand is a location of the containing function. Globals and
mapped interface regions are reached only by address: `w = lift.addr @g`. Lprocs receive everything
else as parameters.

An oracle declares `regs { … }`; an lproc declares five header groups:

```
  lproc @f (params..) (result) (tokens..) (classical..) (mem..)
```

The first group consists of `reg[N]` parameters, the second of exactly
one `reg[T]` result location; the third consists of `token` parameters,
the fourth of `iN`, `ptr` or `f64` parameters, the fifth of `region[A,W]`
parameters with role `in`. Token and classical parameters are immutable;
their optional role suffixes are `token` and `classical` respectively
and MUST agree with their group (§16.2). They are not locations, stack
cells or frame slots. Classical parameters can be read as
pseudo-locations (§4.7); tokens cannot be read as bits or constructed
from them. Region parameters name the passed mapped regions, not copies
of their cells. Names in all groups and `regs` are distinct.

**Argument binding.** The complete call form is
`w = lift.call @f (a_1..a_m) (tokens..) (classical..) (mem..)`.
The first list supplies the location parameters using the capture and
bind steps below. The token and classical lists supply parameters from
the calling oracle's `args`, or from the calling lproc's token and
classical parameters, with exactly matching types. A location (including
a `reg[N]` parameter), an input wire or a literal cannot supply either
list. The region list supplies mapped interface regions of the calling
oracle, or region parameters of the calling lproc, with matching `[A,W]`
and `in` role. Thus the enclosing oracle passes all three lists
explicitly; a recursive or other lproc call forwards them explicitly.
The original form `w = lift.call @f (a_1..a_m)` is canonical exactly
when all three additional groups are empty; otherwise all four lists
are required, including empty ones.

**Fixed binding instances.** For one oracle invocation, resolve those
three lists transitively to the oracle's entry arguments and mapped
regions. An instance is a pair (lproc, binding key); its key is the
concatenation of the zero-based oracle `args` positions supplying its
token parameters, then its classical parameters, then the zero-based
oracle `mem` positions supplying its region parameters. Compare keys
lexicographically as integer sequences. Starting from calls in the
oracle body, close this set under calls in every reached instance;
only finitely many keys exist, even with recursion, since no call
constructs new token, classical or region bindings. Distinct keys stay
distinct even when entry values are equal. An lproc with no such
parameters has the single empty key, preserving its previous indexing.

The index space (§3.6), locations and labels are instantiated per key.
A call targets the callee instance selected by its resolved key; labels
and `qjumpr` ranges are local to the containing instance. All instance
bindings are classical and fixed before oracle entry; reaching a call
under a wire predicate only predicates its existing row operations.
No classical binding changes as a result of quantum control. Different
coherent paths may call the same lproc with different bindings: they
reach distinct instances, each using its own fixed arguments. This is
the reference semantics; executors MAY share their storage or inline
instances only extensionally (§5.7).

Each instance declares its own `regs`. All locations are clean at entry.
The oracle's `result_val : wire[T]` (T = the `tgt` width) is step state,
initially 0.

**Counters and sizes.** Size exponents: `mem` 2^H byte cells, `alloctab`
2^A_tab entries, stack 2^S_cells cells. Counter widths: `heap_used :
wire[Hc]`, Hc ≥ H + 2; `atop : wire[Ac]`, Ac ≥ A_tab + 1; `sp, fp :
wire[Sc]`, Sc ≥ S_cells + 1; `depth : wire[Dc]`, Dc ≥ ⌈log₂(depth_limit +
2)⌉. Every prospective sum (`heap_used + max(n,1)`, `sp + R_f`, `depth +
1`) is computed at width max(operand widths) + 1 before comparison, with
no wrap. `bennett_budget` rejects limits that do not fit (§5.4). The
layout fixes no counter width: the reference realisation uses the
minimum widths Hc = H + 2, Ac = A_tab + 1, Sc = S_cells + 1 and Dc =
⌈log₂(depth_limit + 2)⌉ (Dc from the invocation's budget). Every reachable
counter value fits them (heap_used ≤ heap_limit ≤ heap_limit_max < 2^H,
atop ≤ 2^A_tab, fp ≤ sp ≤ stack_limit ≤ 2^S_cells, depth ≤ depth_limit ≤
2^Dc − 2, each counter advancing only when its flag is clear), so an
executor MAY use wider counters with the same F_B (§5.7); `W_cell` below
uses the reference Sc.

**Initial allocator state** (oracle entry): `heap_used = 0`, bump =
`heap_base`, `atop` = number of pre-entries, `sp = fp = 0`, `depth = 0`,
`alloctab[0..atop)` = one pinned live entry per global in `layout.
globals` order then per mapped interface region in `layout.interface_
regions` order, storage of `mem` all zero except global images.

**Locations and the result name.** Locations of a function are numbered
in declaration order: an oracle's `regs`; for an lproc instance its
`reg[N]` parameters, then its **result name** (`(r:reg[8])` declares a
location `r` of the lproc, written like any location and read by `lift.ret`), then its
`regs`. Every location of the oracle and its instantiated closure has
a global id (Did bits: index-space order of instances, then declaration
order). `L_f` = all locations of the selected instance of `@f`, m′ =
|L_f| (`reg[N]` params + 1 + regs for an lproc); token, classical and
region parameters contribute no locations or stack cells; **R_f = 3 + m′**
cells; cell width W_cell = max(P, Sc, Did, W_max) with W_max the widest location width in the closure. A location
of width N exchanged with a cell exchanges the cell's low N bits; the
cell's remaining bits stay clean.

**`w = lift.call @f (a_1..a_m) (tokens..) (classical..) (mem..)`.**
Resolve the fixed callee instance above; m counts only location
arguments. flag := (depth + 1 > depth_limit) ∨
(sp + R_f > stack_limit), C = `FAULT_BUDGET_DEPTH`, computed first into
`frame.flag`; the elementary sub-operations C1–C5, in this order, run
under ¬flag:

1. **C1 capture**: `disp_i ^= a_i` for i = 1..m (capture slots; every
   `a_i` is read where it is, before any location moves);
2. **C2 push locations**: exchange `L_f` (each padded to one cell) with
   the clean cells `stack[sp+3 .. sp+3+m′)`;
3. **C3 record**: `stack[sp] ^= k+1` (ret index), `stack[sp+1] ^= fp`,
   `stack[sp+2] ^= id(w)` (direct XORs into clean cells);
4. **C4 bind**: `param_i(@f) ^= disp_i` for i = 1..m (the params are among
   L_f and clean after C2);
5. **C5 counters** (assignment): `disp_{m+1} ^= sp`, `disp_{m+2} ^= sp +
   R_f`, `disp_{m+3} ^= depth + 1` (all computed from the old counters),
   then `qexch disp_{m+1} fp`, `qexch disp_{m+2} sp`, `qexch disp_{m+3}
   depth` (now `fp` = old `sp`; the slots hold the old `fp`, `sp`, `depth`).

Successor: the entry label of the selected instance of `@f`. **Inverse** (§3.6 step 4, after the
fault-write undo): C5⁻¹ `qexch disp_{m+3} depth; qexch disp_{m+2} sp;
qexch disp_{m+1} fp` (the old counters are back), then `disp_{m+3} ^=
depth + 1; disp_{m+2} ^= sp + R_f; disp_{m+1} ^= sp` (slots clean); C4⁻¹
`param_i ^= disp_i`; C3⁻¹ `stack[sp+2] ^= id(w); stack[sp+1] ^= fp;
stack[sp] ^= k+1` at the restored `sp`, `fp`; C2⁻¹ exchange `L_f` back
with `stack[sp+3 .. sp+3+m′)`; C1⁻¹ `disp_i ^= a_i` (the `a_i` are
restored by C2⁻¹ even when they are locations of `@f` itself, as in a
recursive call).

**`lift.ret v`** (in an lproc `@f`; flag := (depth = 0), C = `FAULT_UB_
TRAP`; R1–R5 run under ¬flag):

1. **R1 succ**: `frame.succ ^= stack[fp]` (ret index);
2. **R2 capture result**: `disp_1 ^= v` (delivery slot);
3. **R3 pop locations**: exchange `L_f` with `stack[fp+3 .. fp+3+m′)` (the
   locations regain the caller's view; the cells hold the callee's final
   values as history);
4. **R4 deliver**: for every location ℓ of the closure, under `(stack[fp+2]
   == id(ℓ))`: `qexch disp_1 ℓ` (exactly one condition holds; the slot
   retains ℓ's old value);
5. **R5 counters** (assignment): `disp_2 ^= stack[fp+1]`, `disp_3 ^= depth
   − 1`, then `qexch disp_2 fp`, `qexch disp_3 depth`.

Successor: `frame.succ`. **Inverse**: R5⁻¹ `qexch disp_3 depth; qexch
disp_2 fp` (the callee `fp` is back), then `disp_3 ^= depth − 1; disp_2 ^=
stack[fp+1]` (slots clean); R4⁻¹ the same conditional exchanges, their
condition and addresses at the restored `fp` (ℓ regains its old value,
`disp_1` again holds v); R3⁻¹ exchange `L_f` back with `stack[fp+3 ..
fp+3+m′)` (the callee's values are back); R2⁻¹ `disp_1 ^= v`; R1⁻¹ `frame.
succ ^= stack[fp]` (this is the generic `succ` uncompute that ends §3.6
step 4).

The stack is monotonic (`sp` only grows; records are history). Callee
rows are charged to the caller's budget; `lift.call` and `lift.ret` are
one step each. Forwarding or resolving the fixed token, classical and
region bindings adds no rows, capture slots or stack cells. A nested
`lift.oracle_call`/`lift.oracle_uncall` is one outer row and executes the
callee with its own token's tuple; its internal rows do not decrement
the outer B. `lift.perm_call` is likewise one outer row. Executors MAY
inline extensionally. Suite B runs one
forward `lift.call` step and one forward `lift.ret` step of a recursive
call, each followed by its inverse, and asserts that the complete
pre-step tuple is restored and every frame field is clean.

Only oracles and lprocs declare locations, and every `regs` entry in
them has type `reg[N]`; the `regs` list of an action, permutation
action, CC or R function MUST be empty (an action's empty list may
still carry `bounds`). [deps: ISA-3.3-P1, ISA-3.7-P1, ISA-16.2]

## 4. Instruction set

### 4.1 Classical control (CC, R)

| Instruction | Form | Semantics |
|---|---|---|
| `const` | `r = const.iN c` | |
| `add sub mul and or xor shl lshr ashr` | `r = op a b` | wrap; shift ≥ N → 0 |
| `udiv urem sdiv srem` | `r = op a b` | divisor 0 or signed MIN/−1: CC trap; R fault record (`FAULT_UB_DIV0` / `FAULT_UB_SDIV_OVERFLOW`, §4.9) |
| `icmp` | `r = icmp.{eq,ne,ult,ule,slt,sle} a b` | |
| `select` | `r = select p a b` | |
| `zext sext trunc` | `r = op.iM a` | |
| `ptrtoint` / `inttoptr` | `r = ptrtoint.iM p` / `p = inttoptr a` | zero-extend/truncate to M / to ptr_width |
| `load` / `store` | `r = load.iN addr` / `store.iN addr v` | N/8 bytes little-endian, unaligned; outside every live allocation: CC trap / R fault record. The suffix is always `.iN` with N ∈ {8, 16, …, 128}: there is no pointer-typed load or store (a pointer is stored as `ptrtoint.iM` with M = ptr_width and reloaded with `inttoptr`) |
| `alloc` / `free` | `p = alloc n` / `free p` | §7.1 |
| `br` | `br ^b(args)` / `br p ^b1(args) ^b2(args)` | parallel block-parameter assignment |
| `call` / `ret` | `results = call @f args` / `ret vals` | arguments and values are bare operands (`r = call @f a b`, `(r1, r2) = call @f a`, `call @f`, `ret v`, `ret`); results bind in order; a parenthesised argument list is ill-formed for `call`; callee tiers per §3.1 |
| `uncall` | `uncall @f (args) (results)` | **segment uncall**, legal exactly under §7.4: the most recent completed call segment of the history is a call of `@f` whose recorded boundary argument and result values equal the current operand values, every frame after it is pure, and it contains, at any nesting depth, no `measure` frame and no `qreg.free` frame whose cleanliness was not verified; otherwise CC trap / R fault `FAULT_UB_TRAP` with no state change beyond the fault record. The segment is reversed by the cancellation-aware history traversal of §7.4 (LIFO, nested segments included; frames already reversed and `uncall` markers skipped) and its frames are marked reversed; the `uncall` itself pushes a frame naming the segment |
| `trap` | `trap` | CC trap; R fault record (`FAULT_UB_TRAP`, §4.9) |
| `fault` | `fault C` | CC and R; literal 0 ≤ C < 2^64; one terminal record with code C for C ≠ 0, else 4 (§5.4); run ends `FAILED`, no successor executes, prefix history retained for reversal (§7.4, §10, §14.1). One classical instruction (§8.2); in an oracle body or lproc, the one-budget-step §4.2 row instead |
| `qreg.new qreg.free qreg.slice qreg.concat qreg.width qreg.id qreg.from_id` | §3.3 | CC and R (§3.3) |
| `measure` | `r = measure q` / `r = measure.iN q` | CC only; Born rule (§14.2); consumes q; `.iN` for a `qreg[*]` (§3.3) |
| `budget.mint` | `tk = budget.mint @o s h st d` | CC and R; a token for the tuple (s, h, st, d) validated for oracle `@o` (§5.4) |
| `effect` | `effect dev.op args` | R only (§7.3); results are written to memory operands |
| `hostcall` | `rlen = hostcall service ptr len` | CC only (§10) |

**Measurement draws.** Supplied draws are consumed FIFO, one per
executed measurement in dynamic execution order (including measurements
of deterministic states); a failing precondition consumes no draw.
Each draw is in [0,1). Enumerate outcomes lexicographically by their
printed bits string, wire 0 first, and select the first outcome whose
cumulative Born weight is **strictly greater** than the draw times the
total weight. Thus an exact CDF tie selects the next nonzero-weight
outcome. Returned integers remain little-endian (wire 0 is bit 0).
This fixes the reference CLI convention; unsupplied draws follow the
Born distribution (§14.2).

**DRAW-ORDER witness**

```json
{"draws":["0.25","0.75"],"integer_order_values":[1,3],"outcomes_bits":["01","11"],"outcomes_values":[2,3],"reversed_draw_values":[3,2]}
```

Prepare two fresh two-wire registers with H on both wires and measure
them in order. Each has weights 1/4 on `00`, `01`, `10`, `11` in that
order. The first draw selects `01` (integer 2) and the second `11`
(integer 3). Reversed draw consumption would give integers 3,2;
integer-ordered outcomes would give 1,3. Both draws test exact CDF ties.

### 4.2 Lifted rows (oracle bodies and lprocs)

Columns: **Class**, **Reads**, **Writes / slots** (A: displaced values in
slot order; D: the permutation), **Flag / fault** (C = code), **Succ**.
Every row is one budget step; `k` is its own index.

| Row | Form | Class | Reads | Writes / slots | Flag / fault | Succ |
|---|---|---|---|---|---|---|
| `lift.const` | `w = lift.const.iN c` | A | – | w | – | k+1 |
| `lift.binop` | `w = lift.{add,…,ashr} a b` | A | a, b | w | – | k+1 |
| `lift.div` | `w = lift.{udiv,urem,sdiv,srem} a b` | A | a, b | w (under ¬flag) | flag = (b=0) ∨ (signed ∧ a=MIN ∧ b=−1); C = `FAULT_UB_DIV0` / `FAULT_UB_SDIV_OVERFLOW` | ¬flag: k+1; flag: HALT |
| `lift.icmp` | `w = lift.icmp.{pred} a b` | A | a, b | w:reg[1] | – | k+1 |
| `lift.select` | `w = lift.select p a b` | A | p, a, b | w | – | k+1 |
| `lift.cast` | `w = lift.{zext,sext,trunc}.iM a` | A | a | w | – | k+1 |
| `lift.mov` | `w = lift.mov a` | A | a | w | – | k+1 |
| `lift.parmov` | `(w_1..w_m) = lift.parmov (a_1..a_m)` | A | a_i | w_1..w_m (m slots) | – | k+1 |
| `lift.label` | `w = lift.label ^b` | A | – | w | – | k+1 |
| `lift.addr` | `w = lift.addr @g` | A | – | w = layout base of global or mapped region g | – | k+1 |
| `lift.boundscheck` | `lift.boundscheck p n` | D | p, alloctab | – | flag = ¬inrange(p, n); C = `FAULT_UB_MEMFAULT` | ¬flag: k+1; flag: HALT |
| `lift.load` | `w = lift.load.iN a` | A | the N/8 cells at a.. (§7.2) | w (under ¬flag) | flag = ¬inrange(a, N/8); C = `FAULT_UB_MEMFAULT` | ¬flag: k+1; flag: HALT |
| `lift.store` | `lift.store.iN a v` | A | v | the N/8 cells at a.. (under ¬flag; one slot of N bits) | flag = ¬inrange(a, N/8); C = `FAULT_UB_MEMFAULT` | ¬flag: k+1; flag: HALT |
| `lift.alloc` | `w = lift.alloc n` | A | n, heap_used, atop | under ¬flag, slots in this order: w ← heap_base + heap_used; alloctab[atop] ← (heap_base + heap_used, n, 1, 0); heap_used ← heap_used + max(n,1); atop ← atop + 1 (4 slots, all computed from the old state, then exchanged in this order, `alloctab[atop]` at the old `atop`). **Inverse exchanges**: atop, heap_used, alloctab[atop] (at the restored `atop`), w; then clear the four slots | flag = (heap_used + max(n,1) > heap_limit) ∨ (atop ≥ 2^A_tab); C = `FAULT_BUDGET_HEAP` | ¬flag: k+1; flag: HALT |
| `lift.free` | `lift.free p` | D | p, alloctab | match_i = (i < atop) ∧ (base_i = p) ∧ ¬pinned_i into scratch (at most one i matches: unpinned entries are heap entries, whose bases strictly increase); under ¬flag: live_i ^= match_i for all i; scratch uncomputed from the unchanged atop, bases and pinned bits | flag = ¬∃ i: match_i ∧ live_i; C = `FAULT_UB_MEMFAULT` | ¬flag: k+1; flag: HALT |
| `lift.faultif` | `lift.faultif p code` | D | p | – | flag = p; C = code, written as its effective cause C′ (§5.4) | ¬flag: k+1; flag: HALT |
| `lift.call` | `w = lift.call @f (a_1..a_m) (tokens..) (classical..) (mem..)` (short form: §3.7) | A+D | §3.7 | §3.7 (slots: m capture slots `disp_1..disp_m`, then assignment slots for fp, sp, depth) | flag per §3.7; C = `FAULT_BUDGET_DEPTH` | ¬flag: entry of @f; flag: HALT |
| `lift.ret` | `lift.ret v` | A+D | §3.7 | §3.7 (slots: the delivery slot `disp_1`, then assignment slots for fp, depth) | flag = (depth = 0); C = `FAULT_UB_TRAP` | ¬flag: `succ`; flag: HALT |
| `lift.retval` | `lift.retval v` | D | v | result_val ^= v | – | HALT |
| `lift.oracle_call` | `lift.oracle_call @o tk (ins..) tgt fr (mem..) (args..)` | D | ins, tgt, fr, mem | tgt ^= payload; fr ^= code (callee contract) | – | k+1 |
| `lift.oracle_uncall` | same form | D | as above | the matched inverse (§5.5): the identical permutation | – | k+1 |
| `lift.perm_call` | `lift.perm_call @a (mut..) (ins..) (mem..) (args..)` | D | ins, mut, mem | mut ← P(mut, ins, mem); inverse = `adj` of @a | – | k+1 |
| `qjump` | `qjump ^b` | D | – | – | – | ^b |
| `qbranch` | `qbranch p ^b1 ^b2` | D | p | – | – | p: ^b1; ¬p: ^b2 |
| `qjumpr` | `qjumpr a` | D | a | succ ← a | flag = (a < first_f) ∨ (a ≥ first_f + n_f), where `[first_f, first_f + n_f)` are the row indices of the containing function instance (contiguous, §3.6); C = `FAULT_UB_TRAP` | ¬flag: `succ`; flag: HALT |
| `fault` | `fault C` | D | – | code ^= C′; faulted ^= 1 (C′ the effective cause of C, §5.4) | – | HALT |

**`inrange(p, n)`** = ∃ i < atop: live_i ∧ base_i ≤ p ∧ p + n ≤ base_i +
size_i, every comparison computed at width max(ptr_width, width(n)) + 1
(no wrap; §3.3); `lift.load`/`lift.store` carry this check
**inherently** on their own address and length, so the reference
function is total for every well-formed body; `lift.boundscheck` is an
explicit assertion row the dialect emits before each access (after it
the access's own flag is never set, so dialect-produced bodies' step
counts are unchanged). Rules: `lift.retval` is ill-formed in an lproc;
`lift.ret` is ill-formed outside lprocs; `tk` is a token-typed classical
argument of the calling oracle or a token parameter of the calling
lproc. `lift.oracle_call`/`uncall`/`perm_call` carry the
complete callee interface; `mem` operands name the calling oracle's
mapped interface regions or the calling lproc's region parameters.
`lift.addr @g` names a global anywhere; it names an
interface region **only in the body of the oracle that declares it**
(lprocs are shared between oracles and receive region bases as
parameters, as in §16.3 example 2). Label values obtained by `lift.
label` denote first rows of blocks of the containing function instance;
`qjumpr` to an index outside that instance's row range faults, so
control never leaves a function except through `lift.call`/`lift.ret`; an
in-range index that is not a block's first row enters that row. The
`args` operands of `lift.oracle_call`, `lift.oracle_uncall` and
`lift.perm_call` are classical arguments of the calling oracle or token
and classical parameters of the calling lproc, in callee signature
order and with exactly matching types, never locations. They are fixed
at entry (§3.1, §3.7). Every lifted call available in an oracle body is
also available in an lproc by passing the required token, classical
arguments and mapped regions explicitly through `lift.call`. In lifted rows
a width suffix `.iN`/`.iM` names a location width and has no upper bound
(§3.3), except that `lift.load.iN`/`lift.store.iN` keep N ∈ {8, 16, …,
128}.

**Reference body and binding.** Every oracle carries its **reference
body**: the stream the dialect's versioned mapping produced from the
materialised source; hashed into `interface_hash` (§9.2). F_B and the
budget unit are defined solely by the reference interpreter (§14.1) on
it; the object holds only the reference body; every executor
optimisation is extensionally equal on D (T6).

### 4.3 Allocation

| Instruction | Form | Rules |
|---|---|---|
| `qalloc` | `w = qalloc cls n` | n clean wires of class `cls` or `any`; unconditional |
| `qborrow` | `w = qborrow cls n` | n wires in unknown state; usable only inside a borrow frame (§5.6) |
| `qfree` | `qfree w` | release a `qalloc`ed bundle clean on every basis configuration |
| `qreturn` | `qreturn w` | matched inverse of `qborrow` after its frame |

In addition to §3.3's operand rules, `qfree d` is well-formed iff
every wire's current version is its `qalloc` version (including a
version restored by a matched inverse) or the clean output of an
explicit §6.4 cleanup of the value occupying that wire.
[deps: ISA-3.3-P3, ISA-4.3-P1, ISA-5.1, ISA-6.4]

### 4.4 Gates

```
  x = [[0,1],[1,0]]   h = (1/√2)[[1,1],[1,-1]]   s = diag(1,i)   sdg = diag(1,-i)
  t = diag(1,e^{iπ/4})  tdg = diag(1,e^{-iπ/4})   cz = diag(1,1,1,-1)
  rx(θ) = exp(-iθX/2)   ry(θ) = exp(-iθY/2)   rz(θ) = exp(-iθZ/2) = diag(e^{-iθ/2}, e^{iθ/2})
  cx, ccx, mcx(k): X on the target iff all controls are |1⟩ (exact, no phase)    swap: exact exchange
```

Forms: `x a`, `cx c t`, `ccx c1 c2 t`, `mcx (c_1..c_k) t`, `swap a b`,
`h a`, `s a`, `sdg a`, `t a`, `tdg a`, `cz a b`, `rx θ a`, `ry θ a`, `rz θ
a`; all controlled on the active predicate. **Realisation**: permutation
status is a property of the realised unitary at the gate boundary;
conformant iff equal to the specified unitary exactly including global
phase (ε = 0) or within declared ε in operator norm. **Oracles and
permutation actions require ε = 0** for every gate; actions MAY use ε > 0
(§12). A relative-phase Toffoli is never a conformant `ccx`.

### 4.5 Copy, XOR, exchange, in-place

| Instruction | Form | Semantics |
|---|---|---|
| `qcopy` | `d = qcopy s` | d allocated clean; `cx s[i] d[i]` under the predicate |
| `quncopy` | `quncopy s d` | `cx s[i] d[i]` under the predicate; `qfree d` |
| `qxor` | `qxor s t` | `cx s[i] t[i]` |
| `qexch` | `qexch a b` | `swap a[i] b[i]` |
| `qinplace` | `qinplace @g @ginv tk x (ins..) (mem..) (args..)`; written `qinplace @g @ginv tk x` when all three lists are empty (and only then) | **Well-formed** iff `@g` and `@ginv` are oracles whose `in` slots are x's width n followed by the widths of `ins` in order, whose `tgt` width is n, whose fault widths are equal (F), whose `mem` slots match the `mem` regions (role `in`, §3.3's matching rule) and whose `args` types are those of `args`; x, the `ins` wires and the effective control context are pairwise disjoint. **Sem**: `t = qalloc any n; f = qalloc any F; oracle_call @g tk (x, ins..) t f (mem..) (args..); qexch x t; oracle_call @ginv tk (x, ins..) t f (mem..) (args..); qfree t; qfree f`, so x ← g(x; ins, mem, args); these are its generated operations, in this order (§11). **Inverse-pair obligation**: under the tuple of `tk`, the call's `args` values and the tuples of the tokens among them, on every basis value of x, of the `ins` wires and of the `mem` cells, both oracles are fault-free and g(·; ins, mem, args) and ginv(·; ins, mem, args) are mutually inverse permutations of the n-bit encoding. It is carried by an `inverse_pairs` entry (§12.3) of the certificate record of the region whose body contains the `qinplace`; an entry `{g: G, ginv: H}` covers both `qinplace @G @H` and `qinplace @H @G` (its claim checks both compositions). An entry **covers** a `qinplace` when its `budget` is componentwise ≤ the tuple of `tk` (success at a tuple implies the same result at every larger one, §5.4), every token-typed argument's tuple equals the entry's `arg_tokens` tuple for that argument (equality: a nested oracle's outcome may change with its budget), and every other `args` value lies in its claim's `args` domain. Its claim is an `exhaustive` claim with `expected: "inverse_pair"`, which the executor verifies as a checker does (once per closure hash, before first use), or a `structural` `inverse_pair_library` claim, which covers when the named claim of the linked library's certificate is such a claim for the same closure hashes of g and ginv (no re-execution; a library claim without an `args` domain covers every value; its proof or evidence label is the library claim's); a `sampled` claim never covers. Before any effect of a CC/R region call, every `qinplace` the callee executes (transitively; its token and `args` are resolved from the call's classical arguments, which reach lifted calls only as classical arguments, §4.2) MUST have a covering entry, else CC trap / R fault `FAULT_UB_TRAP`. The private t and f are not values of the region for §6: the certified pair cleans them, the only cleanup that is not a §6.4 realisation. Under control the allocations are unconditional and every gate of both calls and of the exchange is controlled |

### 4.6 Quantum memory, predication, structure

| Instruction | Form | Rules |
|---|---|---|
| `qregion` | `qregion name A W [@image]` | function-local region of 2^A cells of `wire[W]`, created at function entry (unconditional allocation, X gates from the data image, default zero), back at its image before exit, image gates reversed, cells released clean; scope is the function. `@image` is a `data` symbol reference (size 2^A·W/8 bytes, cell i = bits [iW, (i+1)W) of the little-endian image) and is part of the function's identity (§9.2) |
| `qload` | `d = qload name a` | d clean; for each cell i: under `a == i`, `qxor cell_i d` |
| `qunload` | `qunload name a d` | inverse |
| `qstore` | `qstore name a v` | for each i: under `a == i`, `qxor v cell_i` |
| `qexchm` | `qexchm name a v` | for each i: under `a == i`, `qexch v cell_i` |
| `qpred`/`qunpred` | | §3.5 |
| `qif` | `qif w { then } { else }` | `qpred w; then; qunpred w; x w; qpred w; else; qunpred w; x w` |
| `qloop` | `qloop n { body }` | body n times |
| `qloop … while` | `qloop n while cond { body }` | `cond : wire[n]` permutation-provenance wires computed before the loop from wires the body does not modify; flags `c_1..c_n` clean; `cx cond[0] c_1`; iteration i under `qpred c_i`; after iteration i < n: `ccx c_i cond[i] c_{i+1}`; flags uncomputed in reverse and freed |

A `qloop` body is threaded like a block: each visit resolves linear
names against the bindings live at its entry, and its exit bindings
supply the next visit's entry bindings; the entry and exit live names
and their types MUST agree, with consumed body-local names absent from
both. [deps: ISA-3.3-P3, ISA-4.6-P8, ISA-4.6-P9]

### 4.7 Region calls (structured code, CC and R)

`oracle_call @o tk (ins..) tgt fr (mem..) (args..)`; `action_call @a
(mut..) (ins..) (mem..) (args..)`; `adj action_call …`; `adj oracle_call
…` ≡ `oracle_call …`; `ctl w <call>` ≡ the call under `qpred w`. In CC
and R, wire and region slots are filled by `qreg` operands per §3.3
(§3.1: R functions call Q regions exactly as CC does; the call is one
frame of the R history, §7.4). In an action, a region operand names an
interface region of the caller or a local `qregion` of the caller
(§4.6); a local region fills an `in` or a `mut` slot (an `in` slot
returns it unchanged; the caller restores it to its image before exit
either way, §5.1). The `mem` operands of lifted rows follow §4.2.
**Classical arguments** have the signature's types `iN`, `ptr` (a value
of width ptr_width), `f64` and `token`. In an oracle body or lproc an
`iN` or `ptr` argument is a pseudo-location of width N or ptr_width, an
`f64` argument a pseudo-location of width 64 holding its binary64 bits,
and a `token` argument is usable only as the `tk` operand of
`lift.oracle_call`/`lift.oracle_uncall`, in a region call's `args` list,
or in `lift.call`'s token list. Lproc classical parameters have these
same bit readings and may also be forwarded in `lift.call`'s classical
list; token parameters retain their token identity and validity (§5.4).

### 4.8 Opcode table (tier, operands, result)

| Opcode family | Tiers | Operands | Result |
|---|---|---|---|
| `const.iN` | CC R Q(lift) | literal | iN / reg[N] |
| `add..ashr`, `udiv..srem`, `icmp.*`, `select` | CC R Q(lift) | values of equal width (select: i1, a, b) | same width (icmp: i1) |
| `zext.iM sext.iM trunc.iM` | CC R Q(lift) | value | iM |
| `ptrtoint.iM`, `inttoptr` | CC R | ptr / integer | iM / ptr |
| `load.iN store.iN` | CC R | ptr [, value] | iN / – |
| `alloc free` | CC R | iN / ptr | ptr / – |
| `br call ret uncall trap fault` | CC R (`fault` also Q(lift)) | per §3.1, §4.1, §4.9 | – |
| `qreg.new qreg.free qreg.slice qreg.concat qreg.width qreg.id qreg.from_id` | CC R | per §3.3 | qreg / – / qreg / qreg / i64 / i64 / qreg[*] |
| `measure`, `measure.iN` | CC | qreg[n] / qreg[*] | in / iN |
| `budget.mint` | CC R | oracle reference, four i64 values | token |
| `effect` | R | device op, ptr/len operands | – |
| `hostcall` | CC | service literal, ptr, len | iN (rlen, width ptr_width) |
| `lift.*`, `qjump qbranch qjumpr fault` | Q (oracle bodies, lprocs) | locations, pseudo-locations, labels, literals; call token, classical and region arguments per §3.7, §4.2 | locations |
| gates, `qalloc qborrow qfree qreturn qcopy quncopy qxor qexch qinplace qregion qload qunload qstore qexchm qpred qunpred qif qloop` | Q (actions) | wires, bundles, regions, literals | wires / – |
| `oracle_call action_call adj ctl` | Q, CC, R | per §4.7 | – |

### 4.9 Classical faults (CC, R)

A **CC trap** and an **R fault record** are one event. The failing
instruction is validated before any mutation and has no effect except
appending one **terminal fault record** (its cause code and program
point, §7.4); the run ends `FAILED` (§10) with no further instruction
executed, so the CC caller of a faulting R call executes no successor
instruction and receives no results, and a `FAILED` run has exactly
one fault record. Implicit trap causes and explicit faults have the
following code mapping:

| Cause | Code |
|---|---|
| `udiv urem sdiv srem` with divisor 0 | 5 `FAULT_UB_DIV0` |
| `sdiv srem` of the signed minimum by −1 | 6 `FAULT_UB_SDIV_OVERFLOW` |
| `load`/`store` outside every live allocation; `free` with no live unpinned entry at its address (§7.1); a `hostcall` range not in one live allocation (§10) | 7 `FAULT_UB_MEMFAULT` |
| `alloc` with the heap or the allocation table exhausted (§7.1); a `qreg.new` the device cannot satisfy (§3.3) | 2 `FAULT_BUDGET_HEAP` |
| an `effect` memory operand range not in one live allocation (§7.3) | 7 `FAULT_UB_MEMFAULT` |
| `effect` on a device outside §7.3 (§17 item 3) | 8 `FAULT_EFFECT_UNMODELLED` |
| `entropy.read` past the end of the tape (§7.3) | 10 `FAULT_ENTROPY_EXHAUSTED` |
| `trap`; an illegal `uncall` (§7.4); a use without a current instance (§3.4); a failed `qreg` precondition, width check or overlap check (§3.3), including an unclean `qreg.free` a verifying executor detects; an invalid token or a failed `budget.mint` validation (§5.4); a `qinplace` without a covering inverse-pair entry (§4.5) | 4 `FAULT_UB_TRAP` |
| `fault C` | C′, the effective cause of C (§5.4) |
| exhaustion of the executor's own classical resources | §17 item 16 |

`fault C` takes a literal 0 ≤ C < 2^64 (a launch fault code is a
`uint64_t`, §10); codes above 10 are program-defined. Its frame is the
fault record. `bennett_result_fault` returns the record's code
unchanged, including every nonzero program-defined code up to 2^64 − 1;
it MUST NOT replace such a code with `FAULT_UB_TRAP` or `FAULT_CALLEE`.
`trap` reports only code 4; it cannot stand in for an arbitrary `fault C`.
No host service is needed to report C (`hostcall` remains CC-only).

**Charge and history.** A CC/R `fault C` executes as one classical
instruction, taking `classical_step_time` (§8.2); it does not consume a Q
oracle's B. An R budget supplied by source instrumentation is enforced
by that code, not by the Q step machine. The terminal record
is appended to the existing logical history without erasing, cancelling
or changing any earlier frame, displaced value, effect-tape entry or
open activation. No subsequent source effect occurs, in this activation
or any caller. Forward failure does not reverse the prefix; an eligible
`bennett_reverse` removes the fault record and reverses that intact
prefix under §7.4, §10 and T4.

A Q region never produces a fault record: an oracle's fault is an
outcome in its fault register (§5.2). Its `fault C` is the existing
§4.2 D row: one executed reference row charges one step of B, writes
`code ^= C′; faulted ^= 1`, and selects HALT. Remaining forward machine
steps are the idle steps of §3.6; commit and prefix reversal follow
§5.2, undoing the fault writes and all preceding rows while retaining
the committed fault-register XOR (§14.1, T2). The classical terminal
record and the Q step-machine outcome are distinct reporting paths.

## 5. Regions

### 5.1 Interfaces and common well-formedness

Signatures list roles: `oracle @o (in a: wire[8]) (tgt t: wire[8]) (fault
f: wire[4]) (mem db: region[10,8] in) (args n: i64)`. **Interface
regions** are caller-owned, passed by name, role `in` (returned to entry
contents on every basis configuration) or `mut` (actions only). In the
text form (§16.2) a wire param's role suffix is optional and, when
present, MUST equal its group's role (`in` in an oracle's or action's
`in` group, `mut` in an action's `mut` group); a region param always
carries its role (`in` or `mut`); other region-interface params carry
no suffix. Lproc parameter roles are specified in §3.7 and §16.2. A region
MUST NOT contain `measure effect br call uncall trap hostcall qreg.* budget.mint` or
classical memory instructions, nor lifted rows outside oracle bodies.
Predicate stack empty at entry and exit. Every `qalloc` matched by a
`qfree` of a clean bundle, every `qborrow` by a `qreturn` after its
frame, every local `qregion` restored, by **explicit cleanup
instructions**; the schedule chooses only realisations. **`in` wires in
actions** appear only as controls (including the address operands of
`qload qunload qstore qexchm`, the wires pushed by `qpred`, the selector
of `qif`, whose two `x` gates restore it, and the `cond` wires of `qloop …
while`), as targets of diagonal gates, as `qcopy`/`qxor` sources, as the
value operand of `qstore` (it is only read, as the control of the XOR
into the selected cell), or as `in` arguments of calls. The **region
call graph** — an edge from a function to the callee of each of its
`oracle_call`, `action_call`, `lift.oracle_call`, `lift.oracle_uncall`
and `lift.perm_call` instructions and to both callees of each
`qinplace`, an oracle counting the rows of its lproc closure as its own —
is acyclic (suite F); recursion exists only through `lift.call` among
lprocs.

### 5.2 Lifted oracle

**Body**: lifted and control rows over its locations and lproc closure;
**exact-permutation only** (ε = 0 everywhere, transitively).

**Contract** on every x ∈ D: `ins` unchanged; `tgt` final = initial ⊕
payload(F_B(x)); `fr` final = initial ⊕ code(F_B(x)); `in` regions
unchanged; borrowed bundles returned (§5.6); workspace, locations, local
regions, step state clean / at image. **F_B(x)** = `(f(x), 0)` if the
reference interpreter reaches `lift.retval` within B steps without a
fault (f(x) = the XOR-accumulated `result_val`), else `(0, code)` with
`code` the first fault in sequential order, or `FAULT_BUDGET_STEPS` if
not halted after B steps. The oracle is an **involution** on `(ins, tgt,
fr)` and its own adjoint.

**Construction**: (1) allocate step state (`result_val` included),
locations, local regions, `alloctab` pre-entries, B frames; (2) run B
forward steps; (3) **commit**: `at_HALT = (pc == HALT)`; under `at_HALT ∧
¬faulted`: `qxor result_val tgt`; under `faulted`: `qxor code fr`; under
`¬at_HALT`: XOR the constant `FAULT_BUDGET_STEPS` into `fr`; uncompute
`at_HALT`; (4) **prefix reversal**: run the inverse step for t = B..1,
then release (1). The commit writes persist.

**Domain D**: every value of each `ins`, of `tgt`, of `fr`, of every cell
of every `in` region, of every borrowed bundle; `qalloc`ed wires at 0;
local regions at image.

### 5.3 Quantum action

**Semantics.** The body denotes a unitary U on mut ⊗ ins ⊗ mem_mut ⊗
(borrowed), block-diagonal in the computational basis of `ins` and of
`in`-region cells, identity on borrowed and external reference systems
(§5.6), workspace returned clean. No fault outcome. Coherent internal
control yields the padded unitary under declared bounds, part of the
action's identity. **Permutation action**: realised unitary a
permutation matrix (ε = 0, permutation gates only, transitively).

**Body**: gates; `qcopy quncopy qxor qexch qinplace`; `qload qunload
qstore qexchm`; `qpred qunpred qif qloop`; `oracle_call action_call`;
`qalloc qborrow qfree qreturn`; `qregion`.
An action body with several `block`s executes its blocks in text order,
each a unitary fragment; labels are descriptive and carry no control
semantics, and lifetimes may span blocks.

**Bounds.** The `bounds { … }` list of an action's `regs` line and the
`bounds` array of its signature hold, in text order, the trip count `n`
of every `qloop` and `qloop … while` in the body (nested loops
included, outer before inner); the list MUST equal the body (suite F)
and is empty for a body without loops. It is part of the action's
identity because the padded unitary of coherent internal control
depends on it.

**Adjoint table** (reverse order, instruction replaced, versions
renumbered):

```
  rx/ry/rz(θ) ↦ (−θ)   s ↔ sdg   t ↔ tdg   x cx ccx mcx swap h cz: self-adjoint
  qalloc ↔ qfree   qborrow ↔ qreturn   qregion creation ↔ release   qcopy ↔ quncopy   qload ↔ qunload
  qxor qexch qstore qexchm: self-adjoint   qinplace @g @ginv tk x (ins) (mem) (args) ↦ qinplace @ginv @g tk x (ins) (mem) (args)
  oracle_call ↦ oracle_call
  action_call @b ↦ adj action_call @b   qpred/qunpred pairs reversed as a whole
  qif w {A}{B} ↦ adjoint of the complete §4.6 expansion, including unconditional lifetimes, in reverse order. [deps: ISA-3.5, ISA-4.6-P7]
  qloop n {A} ↦ qloop n {adj A}
  qloop n while cond {A} ↦ flags computed forward as §4.6; adj A for i = n..1 under c_i; flags uncomputed
```

**Control.** `ctl w @a` is controlled-U: workspace, regions and flags
allocated unconditionally; every gate of the body and its cleanup runs
under `w` in addition to the active predicate.

The shorthand `qif w {adj A}{adj B}` denotes that adjoint when each
arm's allocations and releases are matched within that arm; no
arm-local lifetime restriction is imposed on the source `qif`.
[deps: ISA-3.5, ISA-4.6-P7, ISA-5.1]

Two operations form a **matched pair** iff the second is the §5.3
adjoint of the first, with the same callee(s) (the pair exchanged for
`qinplace`), identical classical argument values and token identities
and tuples, the same operand wires with the first's read-only input
versions and produced target versions current at the second, and equal
effective control sets including the versions of their wires; order and
repetition in predicate stacks are irrelevant. [deps: ISA-2-P10, ISA-3.3-P3, ISA-4.5-P5, ISA-4.7, ISA-5.3-P4, ISA-5.4]
A matched inverse restores preceding wire-version bindings for
subsequent matching only when the §6.2 cleanup conditions hold; matching
alone does not discharge an outstanding-reader or phase obligation, and
a cell write always advances its cell version even when its value is
restored. [deps: ISA-3.3-P3, ISA-6.1-P1, ISA-6.2]

### 5.4 Budgets and fault codes

Lproc token parameters forward existing tokens; they neither mint a
token nor alter its validity or tuple. Binding and forwarding have no
separate budget charge (§3.7); a lifted region call inside an lproc has
the same charge and token semantics as in an oracle body. In
particular, the enclosing oracle's token is not implicitly substituted
for a nested call's explicit token.

Budget = (B, heap_limit, stack_limit, depth_limit), classical. `bennett_
budget` validates: `heap_limit ≤ layout.heap_limit_max`, `stack_limit ≤
2^S_cells`, `depth_limit < 2^Dc − 1`; for a memoryless object (§7.2)
`heap_limit = stack_limit = depth_limit = 0`. B is enforced by running
exactly B steps; heap, stack and depth are wire counters.
*(non-normative, used by §4.5)* If the reference interpreter succeeds
on x under a tuple β₀, it succeeds with the same result under every
tuple β ≥ β₀ componentwise: its only limit-dependent decisions are the
flags of `lift.alloc` and `lift.call`, false on that trajectory and
still false with larger limits, and the step bound, not reached. A
nested `lift.oracle_call` row uses the tuple of its own token, not β,
which is why §4.5 fixes those tuples exactly.

**Tokens.** A token denotes a validated tuple (B, heap_limit,
stack_limit, depth_limit) and the oracle it was validated for; a region
call uses the tuple of the token it is given, and a token MAY be
supplied to any region call of the program (the checks above depend
only on the object's layout). Tokens come from `bennett_budget` (§10;
valid until the program is unloaded) or from **`tk = budget.mint @o s h
st d`** (CC and R): `@o` names an oracle of the program (a symbol
reference, §9.2) whose object's layout the checks use, and is recorded
with the token for diagnostics only; s, h, st, d are `i64` values read
as unsigned 64-bit;
the instruction applies exactly `bennett_budget`'s checks to (s, h, st,
d) before any mutation; failure is a fault record `FAULT_UB_TRAP` (§4.9)
and no token; success produces a fresh token for that tuple, valid
until its frame is reversed or its run is CLOSED, and never
constructible from an integer. A region call given a token that is not
valid is a CC trap / R fault `FAULT_UB_TRAP` before any effect (a launch
argument: `E_ARG`). A minted tuple binds its invocation exactly as a
host-minted one does (§11, §12): an invocation whose closure hash,
variant, inherited controls, tuple and `params_hash` equal the
`bindings` of the object's schedule executes that schedule; any other
invocation executes under the §11 defaults; a consumer MUST NOT apply a
schedule bound to another tuple, nor substitute another tuple. In
`params_hash` a token argument is encoded as its tuple
`[B,heap,stack,depth]`. A budget is classical and fixed before region entry (§3.1), so
`budget.mint` is not a region instruction (§5.1).

```
  0 FAULT_NONE            4 FAULT_UB_TRAP            8 FAULT_EFFECT_UNMODELLED
  1 FAULT_BUDGET_STEPS    5 FAULT_UB_DIV0            9 FAULT_CALLEE
  2 FAULT_BUDGET_HEAP     6 FAULT_UB_SDIV_OVERFLOW  10 FAULT_ENTROPY_EXHAUSTED
  3 FAULT_BUDGET_DEPTH    7 FAULT_UB_MEMFAULT
```

F ≥ 4. Code 10 is recorded by the R `entropy` device (§7.3) and is also
the code a lifted oracle that virtualises entropy uses (`fault 10`) when
its entropy input is exhausted.

**Effective cause.** The literal C of `fault C` (in every tier, §4.2,
§4.9) and of `lift.faultif p C` records the cause C′ = C if C ≠ 0 and C′ =
`FAULT_UB_TRAP` (4) if C = 0;
every other row's code is a fixed nonzero constant. So no fault is ever
recorded as `FAULT_NONE`: a faulted outcome F_B(x) = (0, code) has code ≠
0, and the fault register distinguishes it from every success (F ≥ 4
makes 4 representable). Forward and inverse fault writes use C′.

### 5.5 Nesting and matched cleanup

A `lift.oracle_call` supplies a caller location `fr'`, clean when
computing a fresh outcome. No automatic propagation: the dialect emits `p
= lift.icmp.ne fr' zero; lift.faultif p FAULT_CALLEE` (with `zero` a
`lift.const` location) where the source requires it. To clear `fr'`, the
producer emits `lift.oracle_uncall @o` with the same token, callee, `ins`
and `args` and `mem` values (no intervening row writes those locations, suite F,
nor those cells, below) and the successor versions of `tgt` and `fr'`; the pair shares a
dynamic invocation identity. Alternatively `fr'` is retained until the
prefix reversal. Matching is **LIFO**: on every path, a
`lift.oracle_uncall` matches the most recent preceding unmatched
`lift.oracle_call` with the same callee, token, `ins`, `args` and `mem`
operands (two calls followed by two uncalls form two nested pairs); an
uncall that matches no call on some path, or different calls on
different paths, is ill-formed. In an lproc the match includes the
fixed bindings of its instance (§3.7); parameter forwarding does not
weaken any matching or cleanup condition.
Suite F rejects a pair when a row on a path between them writes one of
its `ins` locations; an intervening `lift.store` is not ill-formed (its
address is data): `lift.oracle_uncall` is the same permutation as the
call (§4.2), so a pair whose cells changed in between still executes
and is undone by the prefix reversal, and clearing `fr'` is then the
producer's obligation.

### 5.6 Borrowed systems and borrow frames

**Requirement.** A region's completed operation equals its declared
operation ⊗ identity on every borrowed bundle and on any external system
entangled with it.

**Rule (suite F).** Borrowed wires (set B) are touched (as control or
target) only inside **borrow frames**: maximal contiguous permutation-
only subcircuits from the first to the last instruction touching B. The
frame's realised permutation P_F on its wires MUST factor as `P_F = P' ⊗
I_B`, established by a T3 claim over the frame's full domain
(exhaustive, or a library certificate, §12 `borrow_frames`) or
**structurally by the translation lemma**: the frame is `V; W; V⁻¹; W`
where

- (i) every gate of V is an X-family flip (`x cx ccx mcx`) whose **target
  is in B** and whose **controls are all in a set X with X ∩ B = ∅**; V
  touches no other wire; hence V maps b ↦ b ⊕ v(X);
- (ii) every gate of W is an X-family flip whose **target is outside X,
  outside B, and outside the set of all controls of W** (as sets, over
  the whole of W), and W writes no wire of X;
- (iii) every gate of W has **at most one control in B**;
- (iv) V⁻¹ is the exact reverse sequence of V; no `swap` appears in the
  frame.

Then W's effect is affine in b, the net map is t ⊕= Σ_g c_g(Y)·v_{j(g)}(X),
independent of b, and b is restored (the dirty-ancilla MCX pattern).
Suite F contains the negative tests `cx b y; cx y z; cx b y` (V;W;V⁻¹),
`V = cx b2 b1, W = cx b1 t` (b2 ∈ B, violates X ∩ B = ∅), `V = cx b x`
(target outside B), and `W = swap y z; cx b y` (swap excluded). Oracles
are one frame verified by their own T3 claim over D.

For (i)–(iii), every gate's controls include its effective control
context, and V⁻¹ runs under the same effective control set and wire
versions as V. [deps: ISA-2-P10, ISA-3.5, ISA-5.3-P5]
Maximality of a frame inside a `qloop` or `qif` body is judged within
that body's static instruction sequence, separately for each borrowed
lifetime, not across unrolled visits or across distinct arm sequences.
[deps: ISA-4.3-P2, ISA-4.3-P4, ISA-4.6, ISA-5.6-P2]

### 5.7 Executor transformations

Inlining, structure recovery, dispatch re-indexing and simplification
MUST produce on every x ∈ D the same final interface tuple as §5.2's
construction on the reference body (T6). Checkpointed and pebbled
realisations: §17 item 18.

## 6. Uncomputation validity (static discipline)

### 6.1 Provenance and transfer

Every workspace bundle version, location version and cell version has
provenance `perm` (computed by permutation gates from its **dependency
set** S of versions) or `other`. Cell versions `(region, i, v)` advance on
every write. Π is the effective control context at the instruction.

| Instruction | Effect |
|---|---|
| `qalloc` | perm, S = ∅ |
| `d = qcopy s` | d: perm, S = {s} ∪ Π |
| `d = qload r a` | d: perm, S = {a} ∪ cells(r) ∪ Π |
| `qxor s t` | t: perm with S_t ∪ S_s ∪ Π if both perm, else other |
| `qexch a b` | each output: perm with S_a ∪ S_b ∪ Π if both perm, else other |
| `qstore r a v` | each cell (r,i): version advances; perm with S_cell ∪ S_v ∪ {a} ∪ Π if both perm, else other |
| `qexchm r a v` | v and every cell: perm with S_v ∪ ⋃S_cells ∪ {a} ∪ Π if all perm, else other; cell versions advance |
| permutation gate, controls C, target t | t: perm with S_t ∪ C ∪ ⋃S_c ∪ Π iff t was perm, else other |
| diagonal gate on u | unchanged |
| non-diagonal gate on u | u: other; closes the window of every value whose S contains u |
| `oracle_call @o (ins) tgt fr (mem)` | tgt, fr: perm with S = ins ∪ {tgt, fr} ∪ cells(every region passed) ∪ Π; `in` regions unchanged |
| `lift.perm_call` / permutation `action_call` | each mut: perm with S = mut ∪ ins ∪ cells(regions passed) ∪ Π; `mut`-region cells advance with the same S |
| non-permutation `action_call` | every mut and every `mut`-region cell: other |
| `qinplace @g @ginv tk x (ins) (mem) (args)` | x: perm with S_x ∪ ins ∪ cells(every region in `mem`) ∪ Π if x was perm, else other; the `ins` wires and `mem` cells are unchanged |
| lifted rows | each written location/cell: perm with the union of its read operands' sets ∪ Π; written cells' versions advance |
| `qregion` creation | every cell: version 0, perm, S = ∅, with the declared image as its initial value. [deps: ISA-4.6-P1] |

**Outstanding dependence** (for `other` cleanup) extends the dependency
test beyond workspace provenance sets to the joint state of all other
live values, including interface `mut` wires, region cells, borrowed
wires and outstanding diagonal phases. [deps: ISA-3.3-P3, ISA-5.1, ISA-5.3-P1, ISA-6.1-P1]
For a proposed inverse of the computing sequence C of g, let G be g's
wires and M the composed intervening operation after C and before that
inverse, retaining every intervening gate and its effective controls,
with internal clean and borrowed lifetimes interpreted as in §14.2.
[deps: ISA-2-P10, ISA-3.5, ISA-4, ISA-14.2]
The remaining values or phases **depend on g** iff there is no single
operator V on the remaining wires such that M = I_G ⊗ V for every state
of G and every input in the remaining wires' declared domain, including
arbitrary external entanglement and the exact global phase.
[deps: ISA-5.2-P4, ISA-5.3-P1, ISA-5.6-P1, ISA-14.2]
This is a joint, transitive test on the composed operation: cleaning or
releasing an intermediate value does not erase its already-transferred
dependence, and a diagonal use remains an outstanding phase unless its
inverse cancels it in M. [deps: ISA-4.4, ISA-6.3, ISA-14.2]
Dependency-set propagation through reads supplies such transitive paths,
but absence of a direct provenance edge to g alone does not establish
the required factorisation. [deps: ISA-6.1-P1, ISA-14.2]

### 6.2 Window and obligations

A perm value g with set S is **in its window** from its computation
until the first of: a non-diagonal gate on a wire in S or in g; a version
advance of a cell in S; a non-permutation action call touching a wire in
S; a permutation change to a wire in S not undone before g's cleanup.
A permutation change to a wire of S is **undone** when it and its
undoing form a matched pair (§5.3), and every intervening operation
leaves the computational-basis values of S and g unchanged; a cell
version advance still closes the window even if its value is restored.
[deps: ISA-5.3, ISA-6.1-P1, ISA-14.2]
Each cleanup is bound to its exact computing sequence and requires the
external input versions that sequence read (the versions in S for a
perm value), and the version of g it produced, to be current; g's
pre-computation target versions are not required to be current.
[deps: ISA-3.3-P3, ISA-6.1-P1]
A cleanup runs under the same effective control set, at the same wire
versions, as its computing sequence. [deps: ISA-2-P10, ISA-3.5, ISA-5.3-P5]
Every perm value MUST be
cleaned within its window and before its region exits; otherwise the
region is **ill-formed** (closed-window measurement-based cleanup is not
admitted). `other` values are released only by the adjoint of their
computing sequence, and only while no other live value, interface wire
or outstanding diagonal phase depends on them, as defined in §6.1.
[deps: ISA-5.3, ISA-6.1, ISA-6.4]

### 6.3 T5

Cleaning g within its window returns g's wires to |0⟩ and factorises
them from the remaining state; diagonal uses of g or of its dependencies
have applied the corresponding diagonal phases to the remainder, which
is part of the program's declared semantics.

### 6.4 Effective control and cleanup realisations

Every cleanup in this version is **unitary**: realisation `reverse`
(the adjoint of g's exact computing sequence, §6.2) or `recompute`
(legal for `perm` values only: recompute g into a fresh clean bundle by
the same sequence, `qxor` it into g's wires, uncompute the fresh bundle
by the adjoint). **Measurement-based cleanup is not admitted in sched/
cert 0.9**: a schedule naming any other realisation is invalid, the
target capability `mbu` is reserved and MUST be `false`, and no cleanup
produces a `channel` claim. *(non-normative)* The fixup after an X-basis
measurement of g with outcome m is the phase (−1)^{m·g(x)}, a function
of g's computing sequence and not of the target; its normative form,
its controlled variant and its certificate kind are deferred (§17 item
6; default: unitary cleanup). Schedules and certificate cleanup entries
are keyed by `(region hash, variant, inherited control count)`; a region
reachable under a non-empty control context MUST have a cleanup
schedule for that key.

## 7. Memory and effects

### 7.1 Classical memory (CC, R)

Byte-addressed, little-endian, sparse. `layout` partitions the address
space: a **null region** `[0, null_size)` never allocated (null_size ≥
1); globals at fixed bases with images; the heap `[heap_base, heap_base +
heap_limit_max)`, all disjoint, `heap_base + heap_limit_max ≤ 2^ptr_
width`, and the number of globals ≤ 2^A_tab, all validated at load. `p =
alloc n` **fails before any mutation** when bump + max(n, 1) > heap_base
+ heap_limit_max or the table holds 2^A_tab entries (CC trap / R fault
`FAULT_BUDGET_HEAP`); otherwise it returns the bump pointer, records
`(base, size = n, live = 1, pinned = 0)` and advances by `max(n, 1)`;
globals have pre-entered pinned live entries; an access is valid iff its
whole byte range lies in one live entry; `free p` marks the unique live
unpinned entry with base p dead (unpinned entries are heap entries,
whose bases strictly increase; pinned entries cannot be freed and a
pinned entry with base p is left unchanged; no live unpinned entry with
base p: CC trap / R fault `FAULT_UB_MEMFAULT`); storage is never
reclaimed.
Initial state: bump = `heap_base`, table = the pre-entries in layout
order, memory zero except images.

### 7.2 Lifted memory of an oracle

`mem` has 2^H byte cells (H ≤ ptr_width); `alloctab` 2^A_tab entries
`(base, size, live, pinned)`; the stack 2^S_cells cells. The `layout`
fixes `null_size`, each global's base and image, `heap_base`, `heap_
limit_max`, and for each **interface region** of the oracle (W = 8
required) a base address; all ranges disjoint and within `[0, 2^H)`, and
for every oracle the number of pre-entries (globals plus its interface
regions) ≤ 2^A_tab, checked at load. `lift.load`/`lift.store` dispatch
by address range to `mem` or to the mapped interface region (role `in`
preserved by the construction). The low H bits address cells; `inrange`
guarantees the range lies in a live entry, hence within `[0, 2^H)`.

**Memoryless objects.** An object without a `layout` section MUST
contain no lprocs, no `lift.addr lift.boundscheck lift.load lift.store
lift.alloc lift.free lift.call lift.ret` rows, no `mem` interface slots
and no CC/R `alloc free load store`; H = A_tab = S_cells = 0, no `mem`,
`alloctab`, stack or counters are allocated, and `bennett_budget`
requires `heap_limit = stack_limit = depth_limit = 0` (§5.4). Example 1
of §16.3 is memoryless.

### 7.3 Device models and tapes (R)

Every device operation takes classical-memory operands and writes its
results to memory; each `effect` is **atomic**: validated (operands in
range, tape available) before any change, and a failing validation is a
fault record with no partial delta.

| Device | State (initial) | Operations | Failure | Frame records / reversal |
|---|---|---|---|---|
| `console` | byte sequence (empty) | `write(ptr, len)` | – | `len`; pop |
| `clock` | `c: u64` (0) | `read(ptr)`: writes c (8 bytes LE) at ptr, then c += 1 | – | overwritten 8 bytes; c −= 1 |
| `entropy` | position `p` (0) into the entropy tape (§10) | `read(ptr, len)`: copies tape[p..p+len) to ptr; p += len | p + len > length: fault `FAULT_ENTROPY_EXHAUSTED` | overwritten bytes; p −= len |
| `block` | sector map from the `block_image` (else empty), 512-byte sectors; absent sectors read as zero | `read(sector, ptr)`; `write(sector, ptr)` | – | read: overwritten 512 bytes; write: the old sector contents **and a presence bit** (absent sectors are removed again on reversal) |

**BLOCK-IMAGE** (K10; source: bennett-vm-47i). `layout.block_image`
names a data symbol (§9.1, §16.1). Decode each successive pair of its
lowercase hex digits to one byte, in textual order. Its decoded length
L MUST be a multiple of 512, including L = 0. At block-device
initialisation, before executing any instruction or changing device
state, reject an image of any other length; never pad or truncate it.
For 0 ≤ s < L/512, sector number s is present and its byte at offset b
(0 ≤ b < 512) is image[512s + b]. There is no sector header, length
prefix, sector-number field or reversal of bytes within a sector. An
all-zero sector in the image is present. Every other sector is absent
and reads as 512 zero bytes; no image and an empty image give an empty
map. This specifies the initial map only; the §7.3 write/reversal
presence-bit rule remains in force. Motivation (0.8 §7.3): "sector map
from the `block_image` (else empty), 512-byte sectors; absent sectors
read as zero"; §9.1: "`data` lines are `NAME SP HEX`". The contiguous
encoding and partial-sector policy are additions, not a claimed 0.8
loader requirement.

**BLOCK-IMAGE witness**

```json
{"length":1024,"present_sectors":[0,1],"sector_first_bytes":[[0,0],[1,255]],"absent_sector":2,"partial_lengths_rejected":[1,511,513,1023]}
```

The image is 512 bytes 00 followed by 512 bytes ff (2048 hex digits).
Sector 0 is present although zero; sector 1 is ff throughout; sector 2
is absent and reads zero. Writing sector 2 then reversing removes it;
writing sector 0 then reversing restores a present zero sector.

Each `effect` appends `(dev, op, args)` to the effect tape.

### 7.4 Run history and the dynamic SSA model (CC, R)

The classical environment holds **dynamic instances**: every executed
defining instruction creates a new instance retained in the run history
until reversal; a name denotes its most recent instance. Frames:

| Instruction | Frame |
|---|---|
| any defining instruction | its defined instance |
| `br` | predecessor block, prior block-parameter instances |
| `store` | address, width, overwritten bytes |
| `alloc` | old bump pointer, the table entry written, the zero-filled range |
| `free` | the entry's old live bit |
| `call` | invocation boundary (callee, argument values) |
| `ret` | result values, the callee's activation and the frame link |
| `uncall` | the reversed segment's id (skipped by whole-run reversal, below) |
| `effect` | tape index plus the device-specific record of §7.3; reversal also removes its effect-tape entry |
| `hostcall` | payload bytes and reply bytes |
| `qreg.*`, `measure` | the ledger record of §3.3 (ids, class, widths, prior states, whether a free's cleanliness was verified) and the measured bits |
| `budget.mint` | the token and its tuple; reversal revokes the token |
| `oracle_call` / `action_call` | (region hash, variant, inherited control count, classical args with every token's tuple, register ids): reversed by re-application / adjoint, exactly once, by this frame |
| fault record (a failing instruction, `trap`, `fault`) | cause code (§4.9) and program point; terminal. Reversal (§10) removes the record; the failing instruction had no other effect, so the state is then the state before it |

**Logical history.** Frames and dynamic instances specify the
*logical* history. An executor MAY store it by checkpoints, deltas or
deterministic replay, provided it reconstructs the same logical frames,
instances, reversal marks and traversal order; it never re-issues an
external event (effects are replayed from the effect tape and host
replies from their frames, never requested again), never re-executes a
region call to recover its record, and never copies quantum state. T4 is
stated over the logical history. **Active and audit history.** The
*active* history is the sequence of frames not marked reversed;
reversed frames and `uncall` markers remain as immutable *audit
records*. Undoing a defining frame removes its dynamic instance from the
active environment although its audit record is retained.

**Pure frames.** A frame is **pure** iff it belongs to `br` or to a
defining instruction among `const`, the binops, `udiv..srem`, `icmp`,
`select`, `zext sext trunc`, `ptrtoint inttoptr`, `load`, `qreg.width
qreg.id qreg.from_id`: it changes only the set of instances. Every other
frame (`store alloc free call ret uncall trap fault effect hostcall
qreg.new qreg.free qreg.slice qreg.concat measure budget.mint`, region
calls, fault records) is impure.

**History traversal** (the one reversal procedure, used by segment
`uncall` and by whole-run reversal). A range of frames is **reversed**
by visiting its frames in LIFO order; a frame already marked reversed
and an `uncall` frame are **skipped**; every other frame is undone by
its row of the table above and then marked reversed. Reversed frames
stay in the history, marked, and are never undone again. An `uncall`
marker is always skipped: when a later traversal (an enclosing
segment's `uncall`, or whole-run reversal) reaches it, the frames of the
segment it names are already marked and are skipped too, so neither the
marker nor its segment is undone, re-executed or redone a second time.

**Segment uncall** (`uncall @f (args) (results)`). Let S be the most
recent completed call segment of the history (a `call` frame, its
matching `ret` frame and every frame between them). The `uncall` is
legal iff S is a call of `@f`, S's recorded boundary argument and result
values equal the operand values, **every frame after S is pure**, and S
contains, at any nesting depth, **no `measure` frame** and no `qreg.
free` frame whose cleanliness was not verified (hardware, §17 item 14);
otherwise it is a CC trap / R fault `FAULT_UB_TRAP` and changes nothing
beyond the fault record (§4.9). Execution reverses S by the history traversal (nested segments are
undone; nested segments already cancelled by an inner `uncall`, and that
inner `uncall` frame, are skipped), and pushes an `uncall` frame naming
S. The instances defined inside S no longer exist; using such a name
before it is redefined is a CC trap / R fault `FAULT_UB_TRAP`.
**Whole-run reversal** (§10) is the history traversal over the whole
history: S's effects on memory, the allocation table, devices, the
register ledger and tokens were undone by the `uncall`, and the frames between S and the
`uncall` are pure, so at the `uncall` frame every component except the
retained instances already equals its value before S. T4 is stated over
these frames. *(non-normative)* S is never re-executed and never redone;
the compute–copy–`uncall` idiom derives what it keeps through pure
instructions before the `uncall`. Suite A contains: `f` calls `g`,
uncalls `g`, returns; `g` performs one `clock.read`; `call f; uncall f`
leaves the clock at its entry value; and `uncall` of a segment that
measured is a trap.

## 8. Target descriptions (target 0.9)

### 8.1 Format and expressions

A target is a JSON document in RFC 8785 canonical form; `target_hash` =
SHA-256 of those bytes; numbers are exact rationals from their decimal
text. Cost expressions are a closed JSON AST:

```
  expr := number | {"var": v} | {"op": o, "args": [expr, …]} | {"table": name, "index": "k" | "eps_decade"}
  v    := "k" | "c" | "eps" | "a" | "w" | "i" | "theta"
  o    := "+" (2) | "-" (2) | "*" (2) | "/" (2) | "max" (2) | "min" (2) | "ceil" (1) | "log2inv_eps" (0)
```

`log2inv_eps` = the exact integer min{ j ≥ 0 : eps · 2^j ≥ 1 } for 0 <
eps ≤ 1; eps outside (0, 1] is a load-time error; an entry with
`epsilon` 0 MUST NOT reference `eps` or `log2inv_eps`. Division by zero
and a table index outside the array are load-time errors. Costs MUST be
nonnegative and finite.

**EPS-DECADE** (K9; source: bennett-vm-cxg). A table expression with
`index: "eps_decade"` evaluates its zero-based array index as the exact
integer min{ j ≥ 0 : eps · 10^j ≥ 1 }. It requires a context binding
`eps` and 0 < eps ≤ 1; otherwise it is a load-time error. Thus it uses
the requested `epsilon_share`, not the entry's achieved `epsilon`.
Evaluation uses exact rational comparisons, never a floating-point
logarithm. The existing out-of-array rule applies to this integer. An
entry with `epsilon` 0 MUST NOT use `eps_decade`, including through a
table lookup. This extends the exact-entry prohibition above; it does
not assign an index to eps = 0. Motivation (0.8 §8.1): the grammar names
`"index": "k" | "eps_decade"`, and "Division by zero and a table index
outside the array are load-time errors"; 0.8 defines no decimal index.

**EPS-DECADE witness**

```json
[{"eps":"1","index":0},{"eps":"0.1","index":1},{"eps":"0.1001","index":1},{"eps":"0.0999","index":2},{"eps":"0.001","index":3},{"eps":"0.000999","index":4}]
```

For eps = 999/10000, eps·10 = 999/1000 < 1 and eps·100 = 999/100 ≥ 1,
so the index is 2. A two-entry table rejects this index, a three-entry
table selects element 2. eps = 0, eps > 1 and an unbound eps reject;
no clamping, padding or fallback entry is admitted.

**Variable binding** by evaluation context; referencing a variable that
the context does not bind is a load-time error:

| Context | Bound variables |
|---|---|
| a `gates` entry's `epsilon` and `cost` | `k` = the gate's own control count; `c` = its total control count (`k` + effective control count, §8.3); `eps` = the operation's `epsilon_share` from the schedule (§11) |
| a scheme's `ancillae.count`, template wire subscripts and `repeat` bounds | `k`, `c` as above; `i` inside a `repeat`; `theta` for a parametric scheme |
| a template gate's `params` | `theta`, `i` |
| `qram` costs | `a` = the region's address width A; `w` = its cell width W; `c` |
| a frame descriptor (§9.4) | `c` only |
| `classes.*`, `capabilities.*`, `resources.*.rate` | none (numbers only) |

**Requested versus achieved accuracy.** `eps` is the `epsilon_share`
the schedule assigns to the operation (§11; the default share is the
region's `epsilon` divided by its number of ε > 0 operations). A `gates`
entry whose `epsilon` expression evaluates to a value > `eps` does not
match the operation and matching continues with the next entry; the
error charged to a matched operation is the entry's evaluated `epsilon`;
an entry with `epsilon` 0 is exact and matches every `eps`.

### 8.2 Resources, aggregation, timing

```
  "resources": { "<name>": { "kind": "count"|"time"|"occupancy"|"peak", "unit": "<string>", "rate"?: { "<cls>": n } } }
```

Mandatory: `qubit_second` (occupancy, per class), `time`, `peak_qubits`
(peak, per class). Aggregation from the **scheduled event list** (every
**leaf event** — a native gate after scheme expansion, a classical step,
an effect — with its start time and duration, every wire with its live
interval): `count` sums the per-event costs, each leaf exactly once (a
scheme contributes only its expanded template, never a cost of its own);
`time` is the makespan; `occupancy` integrates, per class, the number of
live wires times the class rate over time (a class absent from `rate`
has rate 1); `peak` is the maximum over time of the number of
concurrently live wires per class (concurrent demands sum). Classical
instructions and effects take `classical_step_time`. Sequential in
program order unless the schedule assigns start times; a gate occupies
its wires for its `time`; gates on disjoint wires MAY overlap unless
`max_parallel` is declared. Connectivity is all-to-all when `topology`
is absent, and is constrained by §8.3 when present (§17 item 15).
Interface wires are charged to the caller for the call's duration;
borrowed wires to the borrower at `dirty_rate`, replacing the owner's
charge.

### 8.3 Classes, gates, schemes, capabilities

```
  "ptr_widths_supported": [32, 64],
  "classes": { "<cls>": { "qubit_second": n, "dirty_rate": n, "capacity": n | "unbounded" } },
  "topology"?: { "vertices": [ { "class": "<cls>" }, … ], "edges": [ [u, v], … ] },
  "gates":   [ { "op": "<gate>", "classes": ["<cls>"|"*", …], "controls": {"min": n, "max": n | "unbounded"},
                 "params": ["theta"]?, "epsilon": expr, "cost": { "<resource>": expr, "time": expr } }, … ],
  "schemes": [ { "name": "<name>", "realises": { "op": "<gate>", "controls": {"min": n, "max": n | "unbounded"}, "params": ["theta"]? },
                 "ancillae": { "count": expr, "state": "clean" | "dirty", "class": "<cls>" },
                 "template": [ { "gate": "<op>", "wires": [ "c[<expr>]" | "t" | "t2" | "a[<expr>]", … ], "params": [expr]?,
                                 "repeat": { "var": "i", "from": expr, "to": expr }? }, … ] }, … ],
                 /* from inclusive, to exclusive; a scheme has no cost or epsilon of its own (§8.2) */
  "qram": null | { "load": { "cost": {…} }, "store": { "cost": {…} }, "exch": { "cost": {…} } },
  "capabilities": { "measurement": {"mid_circuit": bool, "feedforward_latency": n}, "mbu": false,     /* reserved, §6.4 */
                    "max_parallel": n | "unbounded", "max_predicate_depth": n | "unbounded", "classical_step_time": n },
  "objective": { "weighted": { "<resource>": n } } | { "lexicographic": [ "<resource>", … ] },
  "tables": { "<name>": [ n, … ] },
  "imports": { "<model>": { "version": "...", "map": { "<quantity>": ["<resource>", factor] } } }
```

**Matching.** An executed gate with opcode o, operand classes, **total
control count c** (own controls + effective control count) and
parameters matches the first `gates` entry in array order whose `op`,
class pattern and inclusive `controls` range cover it and whose
`epsilon` ≤ `eps` (§8.1); else the first `schemes` entry whose
`realises` covers it; else load-time error. Template wires: `c[e]` the
e-th control (0-based, e an expr over k, c and i) in the operation's
control order (own controls in operand order, then the effective control
context in predicate-stack order, outermost first), `t` the target (`t`
and `t2` the two wires of `swap`/`cz` in operand order), `a[e]` the e-th
ancilla; `params` expressions may use `theta`; `repeat` binds `i` over
[from, to). Every template gate MUST itself match a `gates` entry
(schemes do not nest) and is a leaf event; the scheme's error is the sum
of its template gates' charged errors. Scheme ancillae add to the frame
descriptor. A scheme's correctness is the target author's obligation
(suite E: T3 for permutation schemes, suite D for ε > 0).

**Topology.** `topology` is optional. Its `vertices` array orders the
physical vertices; a vertex's index is its address and its `class` MUST
name a declared addressable class. Each edge is an undirected pair of
integer indices `[u,v]`, with `0 ≤ u < v < vertices.length`; edges MUST
be sorted lexicographically, without duplicates or self-loops. Unknown
fields reject. Each live wire occupies one vertex of its resolved
class, distinct from every other simultaneously live wire. The number
of vertices of a class limits occupancy in addition to its capacity.
A native multi-wire gate is realisable only if every pair of its
operand vertices is joined by an edge (including all effective control
wires); single-wire gates need no edge. Target schemes MUST meet this
rule for each native template leaf, including their ancillae. §11
schedules bind these placements to the target hash. Absent `topology`
means all-to-all with the existing class capacities. Routing and
movement costs are deferred to 0.10 under §17 item 15; placement itself
is validated in 0.9.

**TOPOLOGY witness**

```json
{"capabilities":{"classical_step_time":1,"max_parallel":"unbounded","max_predicate_depth":"unbounded","mbu":false,"measurement":{"feedforward_latency":0,"mid_circuit":false}},"classes":{"fast":{"capacity":4,"dirty_rate":1,"qubit_second":1}},"gates":[{"classes":["fast"],"controls":{"max":1,"min":1},"cost":{"time":1},"epsilon":0,"op":"cx"}],"imports":{},"objective":{"weighted":{"time":1}},"ptr_widths_supported":[32],"qram":null,"resources":{"peak_qubits":{"kind":"peak","unit":"qubit"},"qubit_second":{"kind":"occupancy","unit":"qubit_second"},"time":{"kind":"time","unit":"second"}},"schemes":[],"tables":{},"topology":{"edges":[[0,1],[0,2],[1,3],[2,3]],"vertices":[{"class":"fast"},{"class":"fast"},{"class":"fast"},{"class":"fast"}]}}
```

The vertices are the row-major 2×2 grid; its four edges connect
horizontal and vertical neighbors. A CX on vertices 0 and 1 is
realisable; on 0 and 3 it is infeasible under §17 item 15. The canonical
UTF-8 bytes above exclude the code-fence LF.
Target SHA-256: `960cee71e88da141cc6c2b13cab3f28fce1d50c006e67446aadb592e2bfd0882`.

### 8.4 Space budget

The capacity constraint is peak live qubits per class.

## 9. Objects, identity, linking, reversible ABI (object 0.9)

### 9.1 Sections and schemas

Order fixed: `symbols imports functions data devices layout schedule
certificate coverage`; each at most once; unknown sections or fields are
errors. Metadata sections are one JCS line each, an object:

```
  symbols:  { "<local index>": { "name", "kind": "cc"|"r"|"oracle"|"action"|"perm_action"|"lproc"|"data", "signature": <sig>, "interface_hash" } }
  imports:  { "<name>": { "signature": <sig>, "pinned_closure_hash"?: hex } }
  devices:  { "models": { "<dev>": { "version": "0.9" } }, "services": { "<reverse-dns id>": { "version": "…" } } }
  layout:   { "ptr_width", "H", "A_tab", "S_cells", "null_size", "heap_base", "heap_limit_max",
              "globals": { "<name>": { "base", "size", "image": "<data name>" } },
              "interface_regions": { "<oracle>": { "<region>": { "base" } } }, "block_image"?: "<data name>" }
  schedule: §11      certificate: §12      coverage: { "entries": [ §13 ] }
  <sig>:    kind "oracle": { "tier":"q", "kind", "in":[w…], "tgt":w, "fault":F, "mem":[{"A","W","role"}], "args":[types], "abi":"0.9" }
            kind "action"/"perm_action": { "tier":"q", "kind", "mut":[w…], "in":[w…], "mem":[…], "args":[types], "bounds":[…], "abi" }
            kind "lproc": { "tier":"q", "kind", "params":[w…], "result":w, "tokens":["token"…], "args":[types], "mem":[{"A","W","role":"in"}], "abi" }
            kind "cc"/"r": { "tier", "kind", "params":[types], "results":[types], "abi" }
            kind "data": { "tier":"-", "kind", "size", "abi" }
```

Integers that may exceed 2^53 are decimal strings, and a JSON number ≥
2^53 anywhere in metadata is an error. `data` lines are `NAME SP HEX`.
**Encodings.** `types` in `args`, `params` and `results` are the §16.2
type spellings as JSON strings (`"i64"`, `"ptr"`, `"f64"`, `"token"`,
`"qreg[8]"`); `w` in `in`, `tgt`, `mut`, lproc `params` and `result` is
the integer bundle or location width (§16.3). An lproc's `tokens`,
`args` and `mem` arrays are mandatory (empty arrays when absent from the
header); `args` excludes tokens and `mem` has only `in` roles (§3.7).
Array order is header order within each group. A **program-point id**
(§9.2) is the JSON array `[interface_hash, block name, ordinal]`.
Certificate `frame` fields use exactly the typed-record schema of §9.4;
unknown fields are errors there as in every metadata record.
`devices.models` lists every §7.3 model that an `effect` of the object
uses (an `effect` on an unlisted §7.3 model is a load error; a device
outside §7.3 is never listed and faults at run time, §17 item 3; a key
that is not a §7.3 model is an unknown field); `devices.services` lists
every `hostcall` service (§10). An interface region's name (its param
name and its `layout.interface_regions.<oracle>` key) MUST differ from
every symbol and import name of the object, since `@name` in `lift.addr`
denotes either (§9.2).

### 9.2 Identity

An lproc's token, classical and region parameter groups are part of
its JCS signature (§9.1), including their order and types. The header
and every forwarded operand list are canonical function bytes; changing
them changes the envelope and, through references, the caller's closure
hash. Fixed binding instances are derived per oracle (§3.7), not new
symbols or separately hashed bodies; instance rows share the source
row's program-point id, and invocation paths distinguish their uses
(§12.1). Entry argument values and token tuples bind invocations through
`params_hash`, not through the lproc's interface hash (this section).
Imports must match all five groups; validation and instance formation
are repeated after linking (§9.3).

**Canonical function bytes**: the text from the function's header line
through its last instruction line, lines joined by LF, no trailing LF.
**Semantic envelope** of a symbol (byte-exact; every part ends with LF):

```
  (1) "isa=" <full version string, e.g. 0.9>
  (2) "ptr_width=" INT
  (3) the JCS signature
  (4) the canonical function bytes (with reference tokens replaced, below); for a data symbol: its data line NAME SP HEX
  (5a) for every data entry in the closure set, sorted by name bytewise:        "data " NAME " " HEX
  (5b) for every layout leaf in the closure set, sorted by JCS path bytes: "layout " <JCS key-component array> " " <JCS value>. [deps: ISA-9.1, ISA-16.1]
  (5c) for an R/CC function, for every device model and service it uses, sorted: "device " <name> " " <JCS entry>
  (6) for every referenced symbol outside the symbol's SCC, in order of first reference: its 64-hex hash
```

**Closure set**, by symbol kind. **Oracle**: **all** `layout` leaves of
the object except `interface_regions` entries of other oracles (every
global's pre-entry affects its allocation faults), its own `interface_
regions.<oracle>.*` leaves, and every data image those leaves name.
**Lproc**: the leaves and images of every global `@g` it names in `lift.
addr`. **Action / permutation action**: no layout leaves; the `@image`
of each `qregion` is an ordinary symbol reference (replaced in part (4),
listed in part (6)). **CC / R function**: all `layout` leaves, every
data image they name, and in (5c) every device model and `hostcall`
service it uses. **Data symbol**: parts (5)–(6) are empty. A layout leaf
is a scalar or an object at the deepest level the schema lists (`globals.
<name>` and `interface_regions.<oracle>.<region>` are leaves; `ptr_
width`, `H`, … are leaves); a scalar's JCS value is its decimal text.
**Globals are data symbols**: every global `g` in `layout.globals` MUST
have `image` = `g` and the object MUST define a `data` symbol named `g`
of that size, so the token `@g` in `lift.addr` is both the global and a
symbol reference (replaced by the data symbol's hash; the global's
`base` and `size` enter through the layout leaf). An `@name` naming an
interface region (in `lift.addr`) is not a symbol reference and is left
unchanged. **Reference replacement**: in parts
(4)–(5) every `@name` symbol-reference token is replaced by `@` followed
by the referenced symbol's 64-hex hash (the symbol's own name in its
header line is a definition, not a reference, and is left unchanged); the **interface hash** uses
interface hashes of defined symbols and `SHA-256(name ‖ 0x00 ‖ JCS
signature)` (hex) for unresolved imports; the **closure hash** uses
closure hashes. **Acyclic and cyclic symbols.** The reference graph
has an edge from a symbol to every symbol it references (parts
(4)–(5)). An **acyclic** symbol — a one-member SCC that does not
reference itself — has hash = SHA-256 of its own envelope (example 1's
`inc8`, example 2's `zero` and `sum_db`); the SCC procedure below applies
to every **cyclic** SCC, including a one-member SCC whose function
references itself (example 2's `sum`). **SCCs**: members ordered by name bytewise; every intra-
SCC `@name` token (parts (4)–(5)) is replaced by `@$<decimal index>`;
intra-SCC references contribute no part-(6) line; `scc_hash` = SHA-256
over the concatenation of the members' envelopes in that order; member
hash = SHA-256 over the ASCII bytes of `<scc_hash hex>:<decimal index>`.
**object_hash** = SHA-256 of the canonical object text with the
`schedule`, `certificate` and `coverage` sections deleted together with
the blank line preceding each. `reference_program_hash` = the oracle's
interface hash. `params_hash` = SHA-256 of the JCS array of classical
parameter values (the typed encoding of §12.3, K13); `schedule_hash` =
SHA-256 of the schedule's JCS bytes.
**Program-point id** = `(symbol interface hash, block name, instruction
ordinal within the block counting nested lines in text order)`.
Executable certificates and caches bind to **closure hashes** (required,
§12).

A layout leaf's path is the JCS array of its original JSON key
components from the layout root to the deepest schema-listed leaf,
so a scalar has a one-component path and an interface-region leaf has
path `["interface_regions","<oracle>","<region>"]`.
[deps: ISA-9.1-P1, ISA-16.1]
Part (5b) emits exactly one line for each structured leaf in the closure
set, including distinct leaves whose dotted renderings coincide, ordered
by the unsigned UTF-8 bytes of that JCS path array, independently of the
leaf values; no dotted rendering is used as a key or an ordering key.
[deps: ISA-9.1-P1, ISA-16.1]
An oracle's own interface-region leaves and entries of other oracles
are selected by equality of the original JSON key under
`interface_regions` to the oracle's name, never by a prefix of a
rendered path; an lproc's global leaves are selected by equality of
the key under `globals` to the global name.
[deps: ISA-3.7-P1, ISA-9.1-P1]

**Envelope boundary lemma** (non-normative).
For a function symbol, §16.2 requires a header beginning `fn `,
`oracle `, `action `, `perm_action ` or `lproc `; following lines are
`regs`, block labels, instructions, or structured closing braces. [deps: ISA-16.2]
No §4 opcode is `data`, `layout` or `device`; a result with one of
those names has ` = ` after the name, which cannot begin any part-(5)
line (a data name, a JSON array, or a device name is required there). [deps: ISA-4.8, ISA-9.1, ISA-16.2]
No canonical function line is a bare 64-hex string, so the first
part-(5)/(6) line ends the function bytes unambiguously. [deps: ISA-4.8, ISA-16.2]
The fixed `data `, `layout `, `device ` prefixes distinguish (5a–c),
and a bare 64-hex line distinguishes (6); JSON strings escape LF. [deps: ISA-9.1, ISA-16.1]
For a data symbol, the signature kind selects exactly one data line
for (4), with (5)–(6) empty, even when the data name itself is `data`,
`layout` or `device`. [deps: ISA-9.1, ISA-16.2]
Together with the fixed first three lines and the injective JCS array
paths, these facts make the envelope parts recoverable without length
prefixes; this is an encoding argument, not a claim of SHA-256 injectivity. [deps: ISA-9.1, ISA-16.1, ISA-16.2]

### 9.3 Linking

Imports resolve by name + signature (mismatch: link error; pinned hash:
equality required); resolved closure hashes are recorded in the linked
certificate. Equal-closure-hash duplicates deduplicate; unequal under one
name, or conflicting `devices`/`layout`/`data` entries: link error.
Linking = resolution, validation, certificate composition (§12.3),
rebinding to closure hashes.

### 9.4 Reversible ABI

- CC/R calls: classical frame; callee frames follow the call boundary;
  `uncall` reverses segments (§7.4).
- Region calls: interface bundles and regions passed in declared order,
  caller-owned; callee owns its workspace, returns it clean, local regions
  at image, borrowed bundles returned.
- **Frame descriptor** (certificate): interface slots retain their identities;
  scratch is recorded by class and cleanliness, with widths as expressions
  in inherited control count `c`; retained history is B × frame width for
  the full-history oracle expansion, otherwise the bound library's realised
  history (§17 item 18). Borrowed wires and scheme ancilla frames are
  separate records. Callees `qborrow` only from the caller's designated
  idle set. Shared frames require disjoint live ranges (suite F).

**Frame JSON schema (B5).** The descriptor is the following JCS object;
all five arrays are required, including empty arrays. Every record has
exactly the shown fields; `live_ranges` is optional. Unknown fields at
any depth are errors under §9.1.

```
  { "slots": [ { "id": "<slot identity>", "role": "in"|"tgt"|"fault"|"mut"|"mem_in"|"mem_mut", "class": "<cls>", "width": expr }, … ],
    "scratch": [ { "id": "<frame identity>", "class": "<cls>", "state": "clean"|"dirty", "width": expr }, … ],
    "history": [ { "id": "<frame identity>", "class": "<cls>", "width": expr }, … ],
    "borrow": [ { "id": "<frame identity>", "class": "<cls>", "width": expr }, … ],
    "ancillae": [ { "id": "<frame identity>", "scheme": "<scheme name>", "class": "<cls>", "state": "clean"|"dirty", "width": expr }, … ],
    "live_ranges"?: [ { "frame": "<frame identity>", "start": ordinal, "end": ordinal }, … ] }
```

`slots` lists the source interface in operand order: oracle `in`, `tgt`,
`fault`, then regions; action `mut`, `in`, then regions. Slot identities
are `<group>:<zero-based index within group>` (`in:0`, `tgt:0`,
`fault:0`, `mut:0`, `mem:0`, …); a region uses role `mem_in` or
`mem_mut`. A region slot's width counts all its cells, in address then
bit order; bundle bits are least-significant first. Slot widths MUST
match the signature. Inherited controls are caller-owned and precede
these slots in §11 topology placement; they are not renamed callee
slots. Every class is a resolved target class. Non-slot frame identities
are nonempty producer-chosen strings, distinct from every other record
identity in the descriptor; a dynamic identity additionally includes
the invocation context and the expansion scratch identity (§11.1).
Array order is source interface order for slots, and first allocation
order in the realised expansion for each other array. `scratch` records
per-class, per-cleanliness workspace; `history` records retained history;
`borrow` records dirty requirements; `ancillae` records each scheme's
ancilla frame and its declared state. These categories do not double
count the same wires. No requirements may be omitted because they vanish
for one chosen basis input.

Every `expr` uses §8.1's expression language with **only `c` bound**;
other variables and table lookups (whose indices require `k` or `eps`)
are invalid. At every covered inherited control count the widths MUST
be nonnegative integers; interface widths are positive and signature
fixed. Instantiation substitutes the resolved budget, parameters and
expansion choices into constants before forming these expressions.
Widths describe the actual realisation, including history and scheme
ancillae, and are checked against it. Each `live_ranges` entry names a
non-slot frame identity and a half-open interval `[start,end)` of its
invocation's generated leaf order (including step/phase/path expansion,
§11); `start < end`. A shared frame identity retains its recorded class, state and width
across uses; every use of shared storage MUST be listed and intervals
of the same frame MUST be pairwise disjoint. An omitted list gives no shared-storage permission.

**FRAME witness**

```json
{"ancillae":[{"class":"fast","id":"a","scheme":"controlled-x","state":"clean","width":{"var":"c"}}],"borrow":[{"class":"fast","id":"b","width":1}],"history":[{"class":"fast","id":"h","width":3}],"live_ranges":[{"end":2,"frame":"s","start":0},{"end":4,"frame":"s","start":2}],"scratch":[{"class":"fast","id":"s","state":"clean","width":{"args":[2,{"var":"c"}],"op":"+"}}],"slots":[{"class":"fast","id":"mut:0","role":"mut","width":2},{"class":"fast","id":"in:0","role":"in","width":1}]}
```

At `c = 2`, the slot widths total 3, scratch width is 4, retained
history width is 3, borrow width is 1 and scheme ancilla width is 2.
The two half-open uses of `s` meet at 2 and do not overlap. This is a
schema/arithmetic witness; a real certificate must also bind the
expansion and target defining `controlled-x`. Its canonical UTF-8
bytes exclude the code-fence LF.
Frame SHA-256: `f314eeef24a5a65a9c72f91e062aa3c83556358eaca166e689571700a72083ff`.

## 10. Launch ABI (launch 0.9)

**Model.** A run executes a device CC entry; the host passes classical
arguments, register ids and the entropy tape, receives classical
results, fault codes and declared measurements; host services are
reached by `hostcall`. **Positional binding**: argument i of `bennett_
run` binds to parameter i of the entry's signature: `INT(width, bytes)`
to `iN` with width = N (else `E_ARG`), `F64` to `f64`, `QREG` to `qreg[n]`
with the register's n or to `qreg[*]` with any width, `TOKEN` to `token`
(a valid token, §5.4). Arguments and the entropy tape
are **copied** at `bennett_run`; the caller may free its buffers
afterwards. Host-minted tokens are scoped to the program and destroyed
at unload; device-minted tokens to their run (§5.4).
Accessor-returned pointers are valid until the owning object is freed.
`bennett_unload` with a run not CLOSED: `E_INVALID_STATE`.

**Invocation tree.** A run record is a tree (CC segments, R calls, Q
calls); an R call's subtree is owned by its R history; `bennett_reverse`
reverses post-order: CC segments by their run history, R calls by LIFO
reversal of their history (§7.4, reversed segments and `uncall` frames
skipped), top-level Q calls by re-application / adjoint. A fault record
is terminal (§4.9): the run is `FAILED`, and every activation open at
the fault keeps its frames, which reversal undoes like those of a call
that returned.

**States.** `CREATED → RUNNING → (SUSPENDED ⇄ RUNNING) → (HOST_REQUEST ⇄
RUNNING) → COMPLETED | FAILED`; `COMPLETED → REVERSED`; `FAILED →
REVERSED`; any of
`COMPLETED/FAILED/REVERSED → CLOSED` (`bennett_run_close` releases the
record; register ids created by the run remain valid unless measured or
freed). A run owns the registers and memory it touches exclusively while
not CLOSED; a register reached by `qreg.from_id` is touched from then
on; `bennett_qreg_prepare`, `bennett_qreg_free` and a `bennett_run`
argument naming a register owned by another run that is not CLOSED are
`E_INVALID_STATE`. **Reverse eligibility**: `COMPLETED` or `FAILED`; no `measure`
frame; no `qreg.free` frame whose cleanliness was not verified (§17
item 14); no register it touched has been prepared, freed, sliced or
used by another run since; else `E_NOT_REVERSIBLE`. Reversing a `FAILED`
run first undoes its terminal fault record, then traverses the rest of
its history exactly as for a completed run, including the frames of
activations that never returned; the run is then `REVERSED` with every
component T4 names at its entry value. `bennett_qreg_prepare` and `bennett_
qreg_free` require a clean register (`E_QREG_NOT_CLEAN` otherwise on a
verifying executor; a non-verifying one: §17 item 14); there
is no reset and no trace-out: a register in unknown state is released by
a `measure` inside a run, which consumes its handle (§3.3).

**Results.** `bennett_result_get` writes value i and sets `*kind` and
`*width`: `INT` as ⌈N/8⌉ little-endian bytes, `*width` = N; `F64` as the
8 IEEE 754 binary64 little-endian bytes, `*width` = 64; `QREG` and
`TOKEN` as the 8-byte little-endian id, `*width` = 64. A `cap` smaller
than that byte count is `E_ARG` and nothing is written. Fault records
are read by `bennett_result_nfaults` / `bennett_result_fault` (codes of
§4.9; a `COMPLETED` run has none, a `FAILED` run has exactly one, its
terminal record, and exposes no values).
For an R `fault C`, `bennett_wait` returns `OK` with `*result_out` set
and `*request_out` NULL as for any `FAILED` run;
`bennett_result_nvalues` is 0, `bennett_result_nfaults` is 1 and
`bennett_result_fault(result, 0)` is exactly C when C ≠ 0 (4 when C = 0,
§5.4), with all 64 bits preserved. The fault code is result data, not a
`bennett_status` error or a host-service reply.

**Waiting.** `bennett_wait` blocks until the run leaves `RUNNING` and
returns `OK` with exactly one outcome: `HOST_REQUEST` (`*request_out`
set, `*result_out` NULL; `bennett_reply` answers it and the run is
`RUNNING` again) or `COMPLETED` / `FAILED` (`*result_out` set, `*request_
out` NULL). Waiting on a `SUSPENDED`, `REVERSED` or `CLOSED` run is `E_
INVALID_STATE`.

**`hostcall`.** `rlen = hostcall service ptr len`: `[ptr, ptr + len)` MUST
lie in one live allocation (else CC trap and no request is made); those
bytes are copied into the request; the run enters `HOST_REQUEST`; the
reply payload is written to `[ptr, ptr + min(len, reply length))` and
`rlen` = the full reply length (so truncation is detectable; a reply
whose length does not fit `ptr_width` bits is refused by `bennett_reply`
with `E_ARG` before any write, and the run stays in `HOST_REQUEST`); the frame
records the request bytes, the overwritten bytes and the reply bytes.
Every `service` is declared in `devices.services`; an undeclared one is
a load error.

**Tape.** `bennett_tape_next` yields each `effect` in tape order; `args`
holds the operation's operands in the order of §7.3, each an unsigned
64-bit little-endian field (pointers zero-extended from `ptr_width`), so
`*alen` = 8 × arity.

```c
/* launch 0.9. All objects opaque. A function that returns an object transfers ownership to the caller,
   released with the matching *_free. Status 0 = BENNETT_OK. */
typedef enum { BENNETT_OK=0, BENNETT_E_VERSION, BENNETT_E_PTR_WIDTH, BENNETT_E_DEVICE_MISSING, BENNETT_E_GATE_UNSUPPORTED,
  BENNETT_E_TARGET, BENNETT_E_BUDGET, BENNETT_E_QREG_NOT_CLEAN, BENNETT_E_INVALID_STATE, BENNETT_E_NOT_REVERSIBLE,
  BENNETT_E_SCHEDULE_INFEASIBLE, BENNETT_E_ARG, BENNETT_E_NOMEM } bennett_status;
typedef struct bennett_program_t bennett_program_t;     typedef struct bennett_args_t bennett_args_t;
typedef struct bennett_result_t bennett_result_t;       typedef struct bennett_request_t bennett_request_t;
typedef struct bennett_estimate_t bennett_estimate_t;   typedef struct bennett_coverage_t bennett_coverage_t;
typedef struct bennett_tape_iter_t bennett_tape_iter_t;
typedef uint64_t bennett_qreg; typedef uint64_t bennett_token; typedef uint64_t bennett_run_id;
typedef enum { BENNETT_ARG_INT, BENNETT_ARG_F64, BENNETT_ARG_QREG, BENNETT_ARG_TOKEN } bennett_arg_kind;

bennett_status bennett_load(const uint8_t* object, size_t len, const uint8_t* target_jcs, size_t tlen, bennett_program_t** out);
bennett_status bennett_unload(bennett_program_t*);
bennett_status bennett_budget(bennett_program_t*, const char* region, uint64_t steps, uint64_t heap, uint64_t stack, uint64_t depth, bennett_token* out);
bennett_status bennett_qreg_new(bennett_program_t*, const char* cls, uint32_t n, bennett_qreg* out);
bennett_status bennett_qreg_prepare(bennett_program_t*, bennett_qreg, const uint8_t* bits, size_t nbits);
bennett_status bennett_qreg_free(bennett_program_t*, bennett_qreg);
bennett_status bennett_args_new(size_t n, bennett_args_t** out);
bennett_status bennett_args_set_int(bennett_args_t*, size_t i, uint32_t width, const uint8_t* le_bytes);
bennett_status bennett_args_set_f64(bennett_args_t*, size_t i, double);
bennett_status bennett_args_set_qreg(bennett_args_t*, size_t i, bennett_qreg);
bennett_status bennett_args_set_token(bennett_args_t*, size_t i, bennett_token);
void           bennett_args_free(bennett_args_t*);
bennett_status bennett_run(bennett_program_t*, const char* cc_entry, const bennett_args_t*, const uint8_t* entropy, size_t entropy_len, bennett_run_id* out);
bennett_status bennett_wait(bennett_program_t*, bennett_run_id, bennett_result_t** result_out, bennett_request_t** request_out);
bennett_status bennett_reply(bennett_program_t*, bennett_run_id, const uint8_t* payload, size_t len);
bennett_status bennett_suspend(bennett_program_t*, bennett_run_id);
bennett_status bennett_resume(bennett_program_t*, bennett_run_id);
bennett_status bennett_reverse(bennett_program_t*, bennett_run_id);
bennett_status bennett_run_close(bennett_program_t*, bennett_run_id);
size_t         bennett_result_nvalues(const bennett_result_t*);
bennett_status bennett_result_get(const bennett_result_t*, size_t i, bennett_arg_kind* kind, uint32_t* width, uint8_t* le_bytes_out, size_t cap);
size_t         bennett_result_nfaults(const bennett_result_t*);   uint64_t bennett_result_fault(const bennett_result_t*, size_t i);
size_t         bennett_result_nmeasurements(const bennett_result_t*);
bennett_status bennett_result_measurement(const bennett_result_t*, size_t i, bennett_qreg* reg, uint8_t* bits_out, size_t cap, size_t* nbits);
uint64_t       bennett_result_tape_length(const bennett_result_t*);   void bennett_result_free(bennett_result_t*);
const char*    bennett_request_service(const bennett_request_t*);
const uint8_t* bennett_request_payload(const bennett_request_t*, size_t* len);   void bennett_request_free(bennett_request_t*);
bennett_status bennett_tape_open(bennett_program_t*, bennett_run_id, bennett_tape_iter_t** out);
bennett_status bennett_tape_next(bennett_tape_iter_t*, const char** dev, const char** op, const uint8_t** args, size_t* alen);
void           bennett_tape_free(bennett_tape_iter_t*);
/* K12: resolved classical parameter values, not live register ids. */
bennett_status bennett_estimate_run(bennett_program_t*, const char* region, bennett_token,
                                    const char* variant, uint32_t inherited_controls,
                                    const uint8_t* params_jcs, size_t params_len, bennett_estimate_t** out);
size_t         bennett_estimate_nresources(const bennett_estimate_t*);
bennett_status bennett_estimate_get(const bennett_estimate_t*, size_t i, const char** name, const char** value_decimal);
const uint8_t* bennett_estimate_certificate(const bennett_estimate_t*, size_t* len);   void bennett_estimate_free(bennett_estimate_t*);
bennett_status bennett_coverage_get(bennett_program_t*, bennett_coverage_t** out);
const uint8_t* bennett_coverage_jcs(const bennett_coverage_t*, size_t* len);   void bennett_coverage_free(bennett_coverage_t*);
```

### 10.1 Launch-estimate contract (K12)

**LAUNCH-ESTIMATE** (K12; b-isa-spec-c15, from bennett-vm-hsd).
The 0.9 request carries the complete resolved parameter vector.

**Resolved request.** `bennett_estimate_run` MUST copy and
validate `params_jcs`, the JCS array of **all** classical arguments in
signature order, encoded as §12.3 specifies; `params_len` is its UTF-8
byte length, excluding any NUL terminator. The separately supplied
valid token of that program fixes `[B,heap,stack,depth]`; token-typed
arguments fix their own tuples, never the outer tuple by default.
Host- and live device-minted tokens are both admitted (§5.4); the
request copies the tuple while valid, so later revocation cannot
retarget an already returned estimate.
An empty array binds an empty signature only; no argument is implicitly
zero, inferred from a hash, or taken from an earlier run. A missing,
extra, mistyped or out-of-range argument is `E_ARG` before producing an
estimate. Every well-typed resolved argument vector is supported; the
interface bundles, arbitrary target/fault entries, dirty wires and
interface cells remain quantum inputs, not chosen basis values.

The caller supplies a nonempty `variant` and `inherited_controls` as
in §11; these are part of the request, not inferred from a different
invocation. The request denotes that region/variant/control count at
its declared accuracy and those resolved parameters and budget.
It uses the object's schedule exactly when every §5.4/§11 binding
matches, otherwise the §11 defaults. The estimator MUST resolve the
complete generated realisation and aggregate its scheduled leaf events
and wire live intervals by §8.2; running one basis input, counting only
executed oracle rows instead of all padded B steps, or silently changing
parameters, budget, target, expansion library or cleanup is not an
estimate of that request. A consumer lacking the selected expansion
library reports `E_SCHEDULE_INFEASIBLE` as §11 requires (b-isa-spec-8r4);
this postpones mechanics, not the estimate capability.

**Resource rows.** `estimate_nresources` counts scalar rows,
not target resource declarations. A `count` or `time` resource has one
row named by its escaped resource name. Each `occupancy` or `peak`
resource has one row **for every target class**, including unused classes with value
zero, named `escape(resource)[escape(class)]`; no sum across classes is
substituted. `escape` percent-encodes the UTF-8 bytes except ASCII
letters, digits, `-._~`, using uppercase hex (including `%`, `[` and
`]`). Scalar resource names use the same escape. Rows are enumerated
by increasing bytewise encoded name. Thus names are injective even when
target names contain delimiters; `name` and `value_decimal` remain
ordinary NUL-terminated strings with §10's ownership lifetime.

**Decimal values.** Aggregation is exact rational arithmetic;
rounding occurs only at the accessor boundary. `value_decimal` is the
least nonnegative decimal with at most **34 significant digits** that
is ≥ the exact resource value. For q > 0 let e be the unique integer
with 10^e ≤ q < 10^(e+1); return
`ceil(q / 10^(e−33)) × 10^(e−33)`. Zero is `0`. The canonical spelling
uses ASCII digits and an optional decimal point, no exponent, plus
sign, leading integer zeros (except `0`), trailing fractional zeros or
negative zero. Exact integers and terminating rationals that fit are
unchanged. This is an upper enclosure, not a fabricated exact value for
1/3; capacity, matching, objective comparison and certificate checking
MUST use the exact rationals, never the rounded display.

**Certificate binding.** The estimate's certificate MUST bind
object, interface, closure, target and effective schedule hashes, root
variant/controls, accuracy, expansion identity and the resolved
`bindings.invocation` (§12.3). Even a default-only realisation has an
explicit empty-entries schedule document with those bindings, hence a
schedule hash. Its root region has `estimate: { precision_digits: 34,
exact_resources: { "<encoded row name>": { numerator: "<integer>",
denominator: "<positive integer>" } } }`; fractions are reduced,
nonnegative and decimal-string encoded, zero is 0/1. `resources` uses
the identical encoded keys and canonical decimal strings returned by
`estimate_get`. A checker MUST rebuild the bound realisation, recompute
all exact rows and their upward decimal encodings, and reject any
binding, key, rational or display mismatch. A producer's resource map
alone is not verification.

**LAUNCH-ESTIMATE witness**

```json
{
  "variant":"forward","inherited_controls":0,
  "binding": {"args":["2"],"budget":["2","0","0","0"]},
  "precision_digits": 34,
  "classes": ["A","B]","idle"],
  "live": [
    {"class":"A","count":2,"start":"0","end":"1/3"},
    {"class":"B]","count":1,"start":"0","end":"1/3"}
  ],
  "rows": [
    ["peak_qubits[A]","2"],
    ["peak_qubits[B%5D]","1"],
    ["peak_qubits[idle]","0"],
    ["qubit_second[A]","0.6666666666666666666666666666666667"],
    ["qubit_second[B%5D]","0.3333333333333333333333333333333334"],
    ["qubit_second[idle]","0"],
    ["time","0.3333333333333333333333333333333334"]
  ],
  "decimal_checks": [
    {"rational":"0","decimal":"0"},
    {"rational":"2","decimal":"2"},
    {"rational":"1/8","decimal":"0.125"},
    {"rational":"1/3","decimal":"0.3333333333333333333333333333333334"},
    {"rational":"2/3","decimal":"0.6666666666666666666666666666666667"},
    {"rational":"1/30","decimal":"0.03333333333333333333333333333333334"},
    {"rational":"10/3","decimal":"3.333333333333333333333333333333334"}
  ]
}
```

This **aggregation witness** assumes the displayed scheduled intervals,
unit class rates and no other events; it does not assert an unbound
oracle gate count. A permitted cost expression `1 / 3` produces that
makespan. The exact class peaks are 2 and 1, not a replacement scalar 3;
occupancies are 2/3 and 1/3, and the idle class is present. Changing the
argument `2` to `1` denotes a different request, even at the same budget.
No 0.8 request can transmit that distinction through §10's signature.

## 11. Scheduling interface (sched 0.9)

```c
typedef struct bennett_sched_request_t bennett_sched_request_t;   typedef struct bennett_sched_response_t bennett_sched_response_t;
bennett_status bennett_sched_request_new(bennett_sched_request_t** out);
bennett_status bennett_sched_request_set_object(bennett_sched_request_t*, const uint8_t*, size_t);
bennett_status bennett_sched_request_set_target(bennett_sched_request_t*, const uint8_t* jcs, size_t);
bennett_status bennett_sched_request_set_region(bennett_sched_request_t*, const char* closure_hash_hex, const char* variant, uint32_t inherited_controls);
bennett_status bennett_sched_request_set_budget(bennett_sched_request_t*, uint64_t steps, uint64_t heap, uint64_t stack, uint64_t depth);
bennett_status bennett_sched_request_set_params(bennett_sched_request_t*, const uint8_t* params_jcs, size_t);
bennett_status bennett_sched_request_set_accuracy(bennett_sched_request_t*, const char* epsilon_decimal);
bennett_status bennett_sched_request_set_capacity(bennett_sched_request_t*, const char* cls, uint64_t peak_qubits);
void           bennett_sched_request_free(bennett_sched_request_t*);
bennett_status bennett_sched_schedule(const bennett_sched_request_t*, bennett_sched_response_t** out);
typedef enum { BENNETT_SCHED_OK, BENNETT_SCHED_INFEASIBLE, BENNETT_SCHED_UNSUPPORTED } bennett_sched_outcome;
bennett_sched_outcome bennett_sched_response_outcome(const bennett_sched_response_t*);
const char*    bennett_sched_response_reason(const bennett_sched_response_t*);
const uint8_t* bennett_sched_response_schedule(const bennett_sched_response_t*, size_t* len);
const uint8_t* bennett_sched_response_certificate(const bennett_sched_response_t*, size_t* len);
void           bennett_sched_response_free(bennett_sched_response_t*);
```

**Generated operations and the expansion library.** Every row and
instruction expands into **generated operations** (gates, allocations,
releases). Above gate level the order is fixed by §3.6, §4.2, §4.5–§4.6
and §5.2 (dispatch comparators k = 0..n in index order; a row's
sub-operations in listed order; successor XORs; commit; cleanup). Below
that level (the gate decomposition of comparators, adders, multiplexers,
`inrange` scans and their scratch) the expansion is defined by an
**expansion library**: a versioned document, identified in the
schedule's `bindings.expansion` as `{ "name", "version", "hash" }`, that
lists for every row kind and every instruction its generated operations
in order, with their **ordinals**, scratch identities and the subset it
marks removable, and that is extensionally equal to the reference (§5.7,
T6). A consumer that does not implement the bound library rejects the
schedule (`E_SCHEDULE_INFEASIBLE`); two consumers implementing it
execute and charge the same operation set. `bennett-isa-spec` publishes
the reference library `ref` (§17 item 7). §5.7 transformations under a
schedule MUST keep the ordinals of the operations they keep and MAY
remove only operations the library marks removable.

**Schedule document** (JCS):

```
  { "bindings": { "closure_hash", "target_hash", "region", "variant", "inherited_controls", "budget": [B,heap,stack,depth],
                  "params_hash", "epsilon", "expansion": { "name", "version", "hash" }, "scheduler": { "name", "version" } },
    "interface_vertices"?: [ [v, …], … ],
    "entries": [ { "path": [ { "symbol": hex, "point": "<block>:<ordinal>", "iteration": [n, …] | "*" }, … ],
                   "step": n | "*",  "phase": "fwd" | "rev" | "*",  "generated": n | "*",
                   "kind": "cleanup" | "gate" | "alloc" | "start",
                   "cleanup": { "realisation": "reverse" | "recompute" },
                   "gate":    { "realisation": "native" | "<scheme name>", "epsilon_share": decimal },
                   "alloc":   { "class": "<cls>", "vertices"?: [v, …] },           /* a concrete class; "any" is not allowed in a schedule */
                   "start":   { "time": decimal } } ] }
```

**Topology placement.** With a §8.3 topology, `interface_vertices`
lists inherited control wires first as individual width-one arrays, then
one vertex array per source interface slot in §9.4 slot order; array
elements follow bit order (and region cell address then bit order). A callee inherits its actual
operand placements. `alloc.vertices`, when present, assigns every bit
of that allocation in bit order. These arrays MUST have the declared
width, resolve to the correct classes and have disjoint live uses;
without a topology they MUST be absent. Omitted placements use the
lowest-index free vertex of the resolved class, assigning bits in order;
interface slots are assigned first in slot order, then allocations in
generated order. Vertices are retained for a wire's lifetime and freed
on release; borrowed wires retain their owner's vertices. Every native
leaf, including scheme leaves, MUST satisfy §8.3 adjacency, else
`E_SCHEDULE_INFEASIBLE` naming §17 item 15. No routing is inserted.

**Matching.** Every generated operation has an **invocation chain**:
the program points of the region calls and `lift.oracle_call`s that led
to it, outermost first, ending in its own point. [deps: ISA-4.7, ISA-4.2, ISA-3.6]
`lift.call` adds no element: an lproc row in an oracle's closure generates
its expansion once per step under its dispatch predicate, independently
of which `lift.call` reached that row or the recursion depth. [deps: ISA-3.6, ISA-3.7]
`path` matches
an operation iff it is a **prefix** of that chain, element by element:
`symbol` and `point` equal, and `iteration` either `*` or equal to the
vector of 0-based dynamic repetition counts of the `qloop`s enclosing
that point (outermost first; its length is the point's loop nesting
depth, `[]` outside every loop).
An oracle invocation's forward step coordinate is t ∈ 1..B and its
inverse step t has `phase: rev`; the root B is `bindings.budget[0]`
and a nested oracle's B is the steps component of its own call token,
not the parent's B. [deps: ISA-5.2-P3, ISA-5.4, ISA-10]
An entry's `step` and `phase` read the oracle invocation entered by the
last oracle-call element of its `path`, or the root invocation if there
is no such element. [deps: ISA-4.2, ISA-4.7, ISA-5.2-P3]
A call element enters that invocation only when it is followed along
the operation's chain into the callee: when the element is the
operation's own final point, it remains in its containing invocation. [deps: ISA-4.2, ISA-4.7]
`lift.call` never changes this level; a path ending at a call element
that is a proper prefix of the operation's chain already selects the
entered callee level. [deps: ISA-3.6, ISA-3.7, ISA-4.7]
At a level with no oracle step, including an action root and oracle
construction/commit/release outside the steps, only `step: "*"` and
`phase: "*"` match; an action's adjoint does not itself supply a
`rev` phase. [deps: ISA-5.2-P3, ISA-5.3, ISA-4.7]
`generated` is the ordinal within the row's expansion in the bound
library. [deps: ISA-11-P2.S3, ISA-11.1-P5]

**Specificity** is the lexicographic tuple (path length, number of
non-`*` iterations, step non-`*`, phase non-`*`, generated non-`*`);
an entry applies iff all its selectors match, and among matching entries
of the same kind the unique maximal one applies. [deps: ISA-11-P3, ISA-4.6]
Two distinct entries (distinct array positions, even if identical)
with equal maximal specificity for the same generated operation and
kind make a tie; this is the meaning of "incomparable", and document
order is never significant. [deps: ISA-11-P3]
A tie makes the schedule invalid for that operation under the validation
stages below; overlapping equal-specificity entries do not make a tie
where a more-specific matching entry applies, including coverage by a
union of such entries. [deps: ISA-11-P3, ISA-5.2-P3]
Operations outside an invocation's own budget or inside a zero-trip
loop are not generated and cannot witness a tie. [deps: ISA-5.2-P3, ISA-4.6]
Selectors are resolved independently per `kind`.
A `gate` entry matches only gate operations, an `alloc` entry only
allocations, and a `start` entry every generated operation. [deps: ISA-11-P2.S1, ISA-11-P3]
The applying entry supplies the decision: an omitted field with a
default takes that default, never a value from a less-specific matching
entry. [deps: ISA-11-P3, ISA-17]
Defaults
for uncovered operations: `reverse`; `native` if the gate matches a
`gates` entry, else the first matching scheme; class = the first class
declared in the target's `classes` object (JCS order) that has capacity
for it; sequential timing in generated order.
An operation uncovered for one kind, including in a nested invocation,
takes that kind's default directly, without inserting an entry or
altering any other operation's selected decision. [deps: ISA-8.3, ISA-17]
There is no requirement that these defaults be expressible by additional
entries in the single-`step`, single-`phase` selector language. [deps: ISA-11-P3, ISA-17]
**Timing**: a `start` time
MUST be ≥ the end of every operation the operation depends on (data,
wire lifetime, region cell version, effect order); an entry violating
this makes the schedule invalid. **Obligations**: satisfy §6; respect
capacity; report every resource; be reproducible from the request plus
scheduler version.

**Validation stages (K55).** A consumer validates every binding and
every path against the object, including the maximal-match condition
at the stages specified here, and executes verbatim or rejects with
`E_SCHEDULE_INFEASIBLE`. [deps: ISA-11-P3, ISA-9.2, ISA-5.2-P3]
For admission, a nested invocation budget is **object-derivable** exactly
when substituting literal integer constants and signature-positional
classical/token forwarding into the four operands of `budget.mint`
yields one tuple for that call site; an unresolved root parameter,
conflicting tuples or any other unresolved operand makes it
non-derivable. [deps: ISA-4.1, ISA-3.7, ISA-5.4, ISA-10]
Literal propagation initialises every nonliteral value to unknown,
takes exact bit-vector values from `const.iN`, copies values through
positional call/return and branch/block-parameter bindings to a fixed
point, and joins all syntactic incoming values as unknown unless they
are all the same known value; arithmetic, predicate reasoning and
additional implementation-specific inference do not make a budget
object-derivable for this staging rule. [deps: ISA-4.1, ISA-3.4, ISA-3.7, ISA-5.4]
The root region's token parameters remain unresolved at object
admission even if a CC/R caller in the object mints a literal-budget
token, since that caller does not bind every invocation of the
region. [deps: ISA-4.1, ISA-4.7, ISA-5.4, ISA-10]
This derivation does not invert `params_hash`, assume a token id's
tuple, or substitute an enclosing invocation's budget. [deps: ISA-9.2, ISA-12.3, ISA-5.4]
A consumer MUST reject root-level ties and nested ties whose invocation
domains are object-derivable at admission, and MUST reject a tie in a
non-derivable nested domain at the first concrete operation in generated
order having multiple maximal matching entries, before applying any realisation to
that operation, with `E_SCHEDULE_INFEASIBLE` and the diagnostic
"§11: multiple maximal matching schedule entries". [deps: ISA-11-P3, ISA-5.2-P3, ISA-5.4, ISA-10]
A nested domain is object-derivable only if every oracle budget on its
invocation chain is object-derivable (the root uses the bound budget);
admission quantifies over the generated operations of these domains,
and defers the remaining domains without rejecting merely because
some possible budget would contain a tie. [deps: ISA-5.2-P3, ISA-4.6, ISA-11-P3]
Admission success therefore leaves the prescribed concrete checks in
non-derivable domains outstanding, even when actual token values become
available earlier; learning them does not move rejection to a different
stage. [deps: ISA-5.4, ISA-10, ISA-11-P3]
Schedule decimal numbers and decimal strings denote the exact rational
of their decimal text; equal values match and resolve identically
regardless of spelling. [deps: ISA-8.1-P1, ISA-5.2-P1]
Token parameters in `params_hash` use §12.3's canonical four-component
tuple encoding, for host- and device-minted tokens alike. [deps: ISA-12.3, ISA-5.4]

**Selection witnesses** *(non-normative)*. [deps: ISA-11-P3, ISA-5.2-P3] For root B=2 and nested B=3,
nested step 3 exists; with nested B=2 it does not, independently of root
B. [deps: ISA-5.2-P3, ISA-5.4] A path of `[]` with `step: 1` reads root step 1 even for an operation
at nested step 3; a path through the nested call reads nested step 3. [deps: ISA-4.2, ISA-4.7, ISA-5.2-P3]
Two `lift.call` sites p1/p2 into the same lproc row q give one row-q
expansion per enclosing step, with neither p1 nor p2 in its chain. [deps: ISA-3.6, ISA-3.7]
An action-root X has no step/phase, including under adjoint. [deps: ISA-5.3, ISA-4.7]
Equal maximal entries A/B reject; A/B fully covered by higher C or a
union C1/C2 admit in every order; duplicate C entries reject. [deps: ISA-11-P3]
An alloc entry cannot shadow a gate entry. [deps: ISA-11-P2.S1, ISA-11-P3]
A more-specific realisation-only entry takes the default share even if
a broad entry writes a share. [deps: ISA-11-P3, ISA-17]
For root-step-1 scheme X and an uncovered root-step-2/nested-step-1
gate, only the latter takes its default; no synthetic nested entry
overrides the former. [deps: ISA-11-P3, ISA-17]
An unknown nested budget permits admission, but if the first ambiguous
operation is at nested step 3, B=3 rejects there before its realisation,
whereas B=2 has no such operation; an object-derivable B=3 rejects at
admission. [deps: ISA-11-P3, ISA-5.2-P3, ISA-5.4, ISA-10]
Numeral 0 and strings `"0"`, `"0.0"`, `"0.000"` all denote zero. [deps: ISA-8.1-P1, ISA-5.2-P1]

### 11.1 Reference-library contract (K11)

**ISA-REF-LIB** (K11; b-isa-spec-8r4, from bennett-vm-47t).
This is the decided 0.9 contract. The complete serialized constructor
catalog `spec/ref-0.9-draft.json` is produced by a separate job under
b-isa-spec-8r4; `ref/` is published after 0.9 acceptance. Until the
bound library is available, consumers report `E_SCHEDULE_INFEASIBLE`
as §11 requires; staged conformance cases remain unavailable naming
b-isa-spec-8r4. The witness below is not a full-library hash.

**Serialization and identity.** Publish `ref/README.md` and
`ref/ref-0.9.json` after acceptance. The JSON file is RFC 8785 canonical
UTF-8, with no trailing LF, containing exactly `isa`, `name`, `version`
and `recipes`. The first three are strings `0.9`, `ref`, `0.9`.
`recipes` is an array in bytewise `kind` order, then increasing parameter
tuple order. Each instantiated recipe has `kind`, its integer width or
other named integer parameters, `operations`, `scratch`, `removable`
and `source` (the motivating ISA sentence). A parameterised recipe
uses `parameters` (an ordered array of formals defined below) and `body` instead of the
instance's `operations`; its body uses the closed constructors below.
Recipes cover every §4 instruction/row, the §3.6 step, §3.7 activation
operations and §5.2 construction, with no implementation-private leaf.
Classical instructions/effects remain classical leaf events (§8.2),
not invented quantum gates. The README documents schema, parameters,
coverage and witnesses; it is not hashed and cannot alter operations.

The library hash is SHA-256 of this UTF-8 envelope (every line ends LF,
including the last); the fourth line is the **entire** canonical library
JSON, including all recipes, sources and versions, without an embedded
hash field:

```
isa=0.9
expansion=ref
version=0.9
<JCS library document>
```

`bindings.expansion` names `ref`, version `0.9`, and this 64-lowercase-hex
hash. This adopts §9.2's version-prefixed, LF-terminated **envelope
convention**, with explicit expansion-domain and library-version lines;
it is a new library rule, not §9.2's symbol signature/SCC envelope.
Motivation (0.8 §11): "a versioned document, identified in the schedule's
`bindings.expansion` as { name, version, hash }"; 0.8 §9.2: "Semantic
envelope ... every part ends with LF". A changed recipe, source, scratch
name or ordering changes the library hash; no schedule/certificate can
silently inherit that change (§12.1's realised-subject binding).

**Expansion and ordinals.** Recipe bodies are ordered arrays
of constructors: a gate (`gate`, ordered `wires`, optional `params`), an
allocation (`alloc`, `width`, `class`, `state`), release (`release`),
classical event (`classical`, `point`), counted loop (`for`, `from`,
`to`, `body`), named recipe expansion (`invoke`, ordered `args`), or
inverse (`inverse`, `body`). Constructor objects have exactly those
keys. Fully instantiated `operations` contain only emitted leaves, each
also carrying its computed `ordinal`; unexpanded `body` constructors
carry no ordinal. Scalars are integer literals, parameter names, or prefix arrays
`[op, operands…]` for `+ - * div mod bit`; div is floor division with a
positive divisor. `bit(x,i)` requires nonnegative integers x and i and
returns floor(x/2^i) mod 2, with no fixed machine-word truncation (K23).
For x=13, bit(x,0..4) is 1,0,1,1,0; literal construction emits X only
for the one bits, in increasing bit order. Wires are declaration names with integer-expression
subscripts. Loops enumerate [from,to) in increasing order; expansion is
depth first, array order, and recipe dependencies are acyclic. An
inverse reverses its fully expanded body and adjoints each leaf by
§5.3, interchanging allocation/release. The complete serialized catalog
must encode the algorithms below in these constructors before release;
this draft does not claim that prose alone is an interchangeable file.

**Typed binding (K19).** A formal is `{ "name": name, "type": type }`,
with exactly these keys, except `wire`, which additionally requires
`width`, and `templates`, which additionally requires `length` (integer
expressions using earlier integer formals); `arguments` requires `symbol`,
and `value` requires `isa_type`, as specified in K24; K25–K27 add
`records`, `region`, `class`, and `allocation` with the keys specified there. Types are `integer`,
`wire`, `angle`, `body`, `symbol`, `environment`, `event`, `image`,
`templates`, `arguments`, `value`, `records`, `region`, `class`, and
`allocation`. A bare name abbreviates an
`integer` formal, retaining the printed witness bytes. Names are unique
within a parameter list. Width expressions use only earlier integer
formals. A wire formal of width n is the ordered list w[n], with
little-endian indexing, including the empty list when n = 0.

`invoke` binds `args` positionally, evaluating actuals in the caller's
lexical environment before binding any callee formal. Integer actuals use
the scalar grammar above. A wire actual is a wire name, a subscripted
wire, an array of wire actuals concatenated left to right, or exactly
`{ "range": name, "from": scalar, "to": scalar }`, selecting [from,to)
from a named bundle; bounds must satisfy 0 ≤ from ≤ to ≤ its width.
An actual's resulting length must equal its formal's evaluated width.
In a gate's `wires` array a whole formal emits all its wires in order;
this encodes `mcx(k)` as k controls followed by one target, including k=0.
`{ "each": name, "in": actual, "body": [...] }` additionally visits
wire-list elements in order, binding name as a one-wire formal for each
visit. Its binding shadows lexically and is restored after the loop.

A body actual is an ordered constructor array, captured with its caller's
lexical environment, or the name of an existing body formal. Exactly
`{ "splice": name }` expands it at that point. Substitution is
capture-free: caller references never resolve to callee locals, loop and
`each` bindings have lexical scope, and every fresh allocation identity
is derived from its expansion occurrence path (including loop iteration,
body-splice occurrence and invocation path). Allocations must not shadow a
live declaration in the same scope; a release names its matching live
allocation. An inverse traverses constructors in reverse, toggles the
adjoint on nested bodies and uses the same restored bindings at each
position; allocation declarations supply release/adjoint metadata even
when allocation and release occur in different nested constructor lists.

A symbol actual is a 64-lowercase-hex §9.2 closure hash, or a bound symbol
formal. The linked closure registry supplies that symbol's own typed
interface and constructor body; the hash must match the linked closure,
not an arbitrary supplied body. Exactly `{ "call": name, "args": [...] }`
expands that symbol under a new invocation-path element identified by the
call constructor's expansion occurrence. It retains its own ordinal
space starting at zero. `invoke` is recipe composition within a row;
`call` is a semantic symbol invocation and never flattens callee ordinals
into the caller's space. Repeated calls have distinct paths.

The inherited effective-control set is always passed to callees and body
splices. An `environment` actual uses the wire-actual grammar and adds
its wires to that set (set union, repeated controls counted once).
Exactly `{ "under": actual, "body": [...] }` adds controls for its lexical
body and restores the outer set afterwards. Controls apply to semantic
gates only, never allocation/release or classical events (§3.5/§4.3).
A control may coincide with a gate's explicit control, but never with a
target (including a diagonal-gate target), as §3.3 requires. Gate operands
must otherwise be distinct.

An `event` actual is exactly `{ "instruction": text, "point": text }`,
or an event formal name. `instruction` is one complete canonical §9.1
classical instruction with its operand bindings; `point` identifies its
§8.2 classical execution point. `{ "classical": opcode, "point":
event_name }` emits that bound event (its opcode must match, allowing a
concrete `.iN` width suffix) without executing or substituting its
instruction text; classical semantics and retained history are §4.1/§7.4.
Literal classical leaves retain their existing `classical`/`point` form.
A classical event is not a unitary; an `inverse` containing one requires
retained §7.4 history and is not a quantum-library matrix witness.

Consumers MUST reject the library document on arity/type/width mismatch,
unbound name or closure hash, invalid wire range, duplicate formal,
nonfinite angle, unknown constructor/key/gate, invalid gate arity,
coincident target/control, unmatched allocation/release, or cyclic
`invoke`/symbol expansion. Static integer bounds must yield finite
expansion; validation is repeated for every actual parameter tuple before
its expansion can be used. There are no implicit bindings or defaults.
Acyclicity concerns declared
recipe/symbol dependencies, not repetition of a higher-order recipe name
inside finite captured actual syntax: qloop(3, qloop(2, B)) expands to six
copies of B. Consumers expand finite actual bodies to a constructor fixpoint.

**Image binding (K21).** An `image` actual is a bound image formal or
an even-length string of lowercase hexadecimal byte pairs in data-image
byte order. The empty string denotes the absent/default-zero image,
including on a wire list whose width is not byte-aligned. Exactly
`{ "image": name, "wires": actual }` emits no leaf for the empty image;
a nonempty image requires exactly eight wires per byte; bit 8j+i of the flattened wire list is bit i of byte j (i=0..7).
It emits one X leaf for each one bit in increasing bit order, with an
empty effective-control set regardless of inherited controls. This
constructor denotes only the unconditional data-image preparation or
restoration required by §3.5/§4.6; ordinary X gates remain semantic.
The bytes are serialized in the actual, not resolved from an unbound
external image name. Image leaves have ordinary row ordinals and the
same allocation wire identities as semantic leaves. Inverse traverses
these X leaves in reverse order, still unconditional. Reject malformed
hex, unbound image or a byte/wire width mismatch. Witness: `"8102"` on
16 wires emits X at indices 0,7,9; inverse emits 9,7,0, even under a
false inherited predicate. This fixes the image binding bytes without
altering §9.2's source data-symbol identity.

**Indexed typed body templates (K22).** A `templates` actual is an ordered
array of exactly `length` objects, each with exactly `parameters` and
`body`, or the name of a bound templates formal. Each object's parameters
use the same typed formal grammar as recipes; its body uses the same
closed constructors. The array is captured in the caller's lexical
environment. Exactly `{ "apply": name, "index": scalar, "args": [...] }`
selects its zero-based element and binds positional actuals evaluated at
the apply site, with the same type/width/arity validation as invoke.
Formal bindings shadow captured names only inside that application;
all free references still resolve in the captured environment. Effective
controls are the union of captured controls, apply-site controls and
explicit environment actuals. Fresh allocations include the application
occurrence in their identity. Applications compose within the containing
row's ordinal space; they do not introduce a semantic symbol call.
Reject negative/out-of-range indices, length mismatch, unknown/duplicate
formals or invalid constructor syntax even in unused templates. Inverse
reverses the applications and adjoints each selected body using its
restored positional bindings. This permits row templates with a typed
frame formal, applied to frame_t, and a step template applied to each
successive frame without rebinding a captured free name.
Witness: two templates `(f:wire[2])` containing CX outer f[0] and CX outer
f[1], applied in index order to a, emit CX outer a[0], CX outer a[1];
the inverse emits them in reverse order. Applying to a one-wire actual
rejects. Applying the same template to distinct frame slices cannot
change its captured outer binding.

**Closure-bound argument vectors (K24).** An `arguments` formal's
`symbol` names an earlier symbol formal. Its actual is an array of actuals
in that linked symbol's own formal order, or a bound arguments name for
the identical closure hash. The vector captures the actuals' lexical
environment; validate every argument against that symbol's interface at
binding, even when no call executes. `call` additionally accepts the
bound arguments name as its `args` value; it forwards the whole vector
without reinterpreting names in the forwarding recipe's scope. Reject a
vector bound to another closure, even if the two interfaces have equal
widths. Each call still has its own invocation path and callee ordinals.

A `value` formal's `isa_type` is `iN` (N a canonical positive decimal),
`f64`, `token`, or exactly `{ "ptr": N }` with N the linked object's
positive `ptr_width`. Values use exactly §12.3 invocation encodings:
canonical unsigned decimal strings within the iN/ptr range, binary64
bit-pattern hex (including signed zero and NaN payloads for classical
f64), or four canonical nonnegative decimal strings for a token tuple.
A value may also be the name of a bound value of exactly the same
`isa_type`; widening or converting it implicitly is forbidden. Token
values carry tuples, never process-local ids. This adds no arithmetic
on classical f64 or tokens inside a recipe.

Witness: a linked interface `(wire[1], wire[3], i128, f64, token)` accepts
`[a,b,"18446744073709551617","0x7ff8000000000001",["2","0","0","0"]]`.
A forwarding recipe `(S:symbol,A:arguments(S))` with two calls using A
preserves all bindings and emits two distinct invocation paths starting
at ordinal zero. Swapping a and b rejects; 2^128 for i128, a three-part
token, or an arguments vector for a different closure rejects. This
binding rule does not replace §3.3's source interface region shape/role
validation or the §9.2 linker obligation that the registry interface and
body match the supplied closure hash.

**Ordered descriptors (K25).** A `records` formal has exactly `name`,
`type`, `fields`, `order`. `fields` is an ordered typed formal list;
its types are `integer`, `wire`, `region`, `class`, `allocation`, or
`image` (K21, for source data-image descriptors).
A field's width/shape expressions may use enclosing earlier integer
formals and earlier integer fields, never later fields. `order` is an
array of distinct integer field names. An actual is an array of objects
whose keys are exactly the field names, or a bound records name with the
identical field schema and order. Each field actual resolves in the
actual's captured lexical environment; earlier integer fields additionally
bind width/shape expressions. Forwarding preserves resolved identities,
revalidates widths against the receiving enclosing integers, and never
resolves a wire name again. With nonempty `order`, the tuples of those
fields must be strictly increasing lexicographically; with empty `order`,
array order is normative. Empty arrays are valid.
`{"visit": name, "body": [...]}` visits records in array order, binding
all their fields lexically for that body, shadowing outer names across
all types and restoring them afterwards. An inverse visits the same
records in reverse order and adjoints each body. No record iteration
emits a leaf by itself. Reject extra/missing fields, an invalid schema,
order mismatch, duplicate/out-of-order keys, or any invalid field actual,
even for an unvisited list. Record iteration and K22 template application
share the same capture, effective-control and occurrence-identity rules.

The following descriptor schemas bind source metadata, rather than
caller-supplied circuit bodies. Source formation MUST check every record
against the linked source declarations/layout before expansion:

* A byte-layout list has integer fields `base`, `extent`, then
  `cells:wire[8*extent]`, ordered by `base`. Bases/extents are nonnegative
  mathematical integers, extents positive. Form the list by partitioning
  `[0,2^H)` at every mapped interface region endpoint. Each mapped interval
  binds its region's cells, flattened by increasing cell index and then
  LSB-first bit index. Each remaining maximal interval binds the same
  address slice of ordinary `mem`. Adjacent mapped intervals remain
  separate records. Require exact coverage, no overlap and no gap; mapped
  cells have W=8 as §7.2 requires. Byte `base+j` is exactly wires
  `cells[8*j .. 8*j+8)`. Ordinary `mem` under a mapped interval is never
  selected by an access. Interface shape and origin use K26.
* An activation location list has `id`, `width`, `slot` integers followed
  by `location:wire[width]`, ordered by `id`. The ids and widths are
  exactly §3.7's global ids and declared widths. For a selected instance,
  `slot` is its zero-based declaration index in L_f, including parameters
  and the result name; for a closure delivery list it is the declaration
  index within the location's own instance. A capture list has `position`,
  `width` integers followed by `source:wire[width]`,
  `parameter:wire[width]`, `capture:wire[width]`, ordered by `position`.
  Positions are 0..m-1 in source argument order; parameter and capture
  are the corresponding callee location and frame displacement slot.
  The stack low-width slice for location slot j starts at
  `(old_sp+3+j)*W_cell` in C2 and `(restored_fp+3+j)*W_cell` in R3;
  higher cell bits are untouched. Delivery scans the complete closure
  list in global id order. Reject records inconsistent with those source
  mappings. These lists add no locations for token/classical/region
  arguments; those remain K24/K26 fixed bindings.

* A construction state list has `width`, `initial` integers, `I:image`,
  `storage:allocation`, with empty `order` (array order). Each storage
  identity is distinct, clean and exactly `width` bits; its class is
  `any` for the implicit §5.2 state. In allocation order the list is:
  pc, pc-prime, faulted, code, result_val; then heap_used, atop, sp, fp,
  depth, ordinary mem, stack when layout state exists; locations in global
  id order; local regions in declaration order; alloctab when layout state
  exists; frames 0..B-1. Omit only the layout state excluded by §7.2's
  memoryless-object rule. Counter widths/values are the reference widths
  and initial values of §3.7; all other non-image initial values are zero
  except alloctab pre-entries. A table cell packs base[ptr_width],
  size[ptr_width], live, pinned, in that LSB-first order; pre-entries are
  in §3.7's layout order and unused entries are zero. Image-bearing mem
  and local regions have initial=0. Ordinary mem's I is exactly 2^H
  bytes, initially zero, with each global's image overlaid at its layout
  base; bytes hidden by mapped interface regions remain zero. A local
  region's I is its exact bound data image, or empty for absent/default
  zero. Every other record has I empty and initial
  the nonnegative integer encoding of its initial bit vector, strictly
  below 2^width. Mapped interface regions are borrowed inputs, never
  acquired by this state list. A separate frame list has `index` integer
  and `frame:allocation`, ordered by index; it contains exactly B entries
  indexed 0..B-1, referring to the identical frame handles in the state
  list, each with §3.6's fixed frame width and field order. Reject any
  mismatch of source state, image, width, class, order or identity. The
  construction acquires/initializes this list, invokes the catalog `step`
  once per frame, commits, then reverses frames and initialization before
  releasing the list in reverse order; source row/successor templates
  supply rows, never a replacement dispatcher or construction body.

Witness: byte records `(base=0,extent=1,cells=a[8])` and
`(base=1,extent=2,cells=b[16])` bind addresses 0,1,2 to a[0..8),
b[0..8),b[8..16), respectively. Visiting X on each first bit emits
X a[0], X b[0]; inverse emits X b[0], X a[0]. A two-bit location
id 0/slot 0 followed by a one-bit location id 1/slot 1 exchanges only
the low 2 and low 1 bits of stack cells old_sp+3 and old_sp+4;
return/inverse address them at restored_fp+3 and restored_fp+4.
Exchanging the two record positions rejects for id order.
A construction witness with rows `loc = lift.add x y; lift.retval loc`
commits fault 1 for B=0/1 and `(x+y) mod 2^W` into the arbitrary target
for B>=2, preserving inputs and cleaning every state/frame allocation.
A local image 81 emits unconditional X at cell bits 0,7 and restores them
before release, including under a false external predicate.

**Source region bindings (K26).** A `region` formal has exactly `name`,
`type`, `address`, `cell`, `role`; address/cell are integer expressions
using earlier integers, with A>=0, W>0, and role is `in` or `mut`.
Its actual is a bound region name or exactly
`{"address": A, "cell": W, "role": role, "origin": origin, "cells": actual}`.
A and W in the actual are nonnegative/positive integer literals, role
must equal the formal role, and the cell wire actual has exactly 2^A*W
wires, in cell-index/LSB-first order. Both shape components must equal
the formal components, even if a different shape has the same total
width. Origin is `interface:` followed by the canonical source operand
name for a caller-owned interface, or `local:` followed by the local
source declaration's NAME token. Each suffix is the exact canonical
§16.1 NAME token, without a sigil. Its binding together with the ordered
cell wire identities identifies the source occurrence (including repeated
invocations); forwarding preserves both. The suffix
is checked against the linked source binding; a local origin cannot be
substituted for an interface origin by changing its string. A region
formal can be used as its flattened cells in any wire actual, while
forwarding it as a region retains shape, role, origin and wire identities.
K24 argument vectors validate regions in the linked signature's slot
order just as wires/values; source role/overlap restrictions remain §3.3.
Lifted calls accept only their permitted mapped/parameter origins (§4.2);
action calls may accept local regions where §3.3 permits them. No rule
infers source shape or role from total bit width.
Witness: a region[1,3] in local:r over six wires forwards unchanged through
two K24 calls. Binding it to region[0,6] in or region[1,3] mut rejects,
despite equal total width. Invalid/missing origins also reject.

**Classes and allocation declarations (K27).** A `class` formal has
exactly `name`,`type`. A class actual is a bound class name or exactly
`{"class": text}`, where text is the exact source class spelling (`any`
included), never an empty string or a resolved substitute class.
`{"declare": name, "width": scalar, "class": actual, "state": state,
"body": [...]}` reserves a fresh allocation identity with nonnegative
width, that class, and state `clean` or `borrowed`. It emits no leaf;
it binds an initially inactive allocation declaration for its lexical
body, including captured actual bodies. It may not shadow any binding.
An `allocation` formal (exactly `name`,`type`) passes that declaration
by identity, including metadata and live/inactive status. Exactly
`{"acquire": name}` emits its unconditional allocation and changes it
from inactive to live; exactly `{"relinquish": name}` emits its matching
unconditional release and changes it from live to inactive. An acquire
requires inactive, a relinquish requires live, and the declaration must
be inactive when its lexical body exits. While live, its name is a wire
actual of its declared width, including for subsequent caller constructors
after `invoke` returns; while inactive any wire use rejects. Captures and
invocations share the same status, not a copy of the lifetime. Inverse
exchanges acquire/relinquish, using the same declaration metadata and
occurrence identity. Reacquisition denotes a new allocation occurrence,
with fresh identity even within one declaration. Clean release requires
zero; borrowed release requires restoration on the full domain (§5.6),
never zero. These are user/instruction allocation results; internal
library scratch still obeys the clean LIFO rule below. A declaration
or allocation formal listed in `scratch` when acquired marks that entire
allocation lifetime as library scratch; the mark follows its identity
through captures and invokes, requires clean state and cannot be removed
by releasing through a different formal name. Allocations are
unconditional even under a false inherited control.
Witness: declare out[2] of class ancilla; invoke an allocation recipe on
out; apply two X gates to out[1] in separate caller/callee bodies; invoke
its release recipe. All leaves refer to the one live allocation identity
and release it clean. Wire use before acquisition/after release, double
acquisition, unmatched release or leaving the declaration live rejects.
A borrowed declaration preserves the exact class and both initial values
of every borrowed bit; no conversion to class any is permitted.

**Angles and quantum witnesses (K20).** An `angle` actual is a bound angle
formal or a finite binary64 value. Reusing §12.3's bit-pattern encoding:
`f64` is `0x` followed by exactly 16 lowercase hex digits of its IEEE 754
binary64 bit pattern, most significant nibble first (so −0 and NaN payloads
are not lost). Angles additionally reject NaN and infinities (§3.1).
Only `rx`, `ry`, `rz` have `params`, an array containing exactly one angle;
other §4.4 gates have no angle parameters. No recipe arithmetic on angles
is permitted. `inverse` negates each rotation angle by toggling bit 63,
including signed zero, and exchanges s/sdg and t/tdg as §5.3 requires.

For h, phase gates s/sdg/t/tdg/cz, and rotations, correctness is equality
of the §14.2 unitary (using §4.4 matrices), column by column on the full
computational basis of all recipe interface wires, with scratch initially
clean and finally clean. Arbitrary targets, controls and borrowed inputs
are included. This full-basis unitary is the witness extending T3's
basis argument; non-permutation leaves MUST NOT be certified by a basis
permutation simulator. Exact algebraic amplitude checks apply to Clifford+T
leaves. For arbitrary binary64 angles a numerical checker must state its
round-trip criterion and error bounds; numerical agreement is evidence,
not a proof of ε=0 for arbitrary transcendental matrix entries.

**REF-BINDING / REF-QUANTUM witnesses.** With formals
`[{"name":"N","type":"integer"},{"name":"w","type":"wire","width":"N"},
{"name":"theta","type":"angle"},{"name":"B","type":"body"}]`, actuals
`[1,{"range":"a","from":0,"to":1},"0x8000000000000000",[]]` bind
w[0]=a[0], θ=−0 and an empty body. Replacing the range end by 0 rejects
for width mismatch. Splicing a body allocating/releasing `tmp` twice gives
two distinct scratch identities even if the callee also declares `tmp`.
An inverse rx at `0x3ff0000000000000` emits rx at
`0xbff0000000000000`. For `w=[a[0],a[1]]` and t=a[2],
`{"gate":"mcx","wires":["w","t"]}` emits exactly those three operands;
`each` over w visits a[0] then a[1]. Two calls to the same closure have
separate invocation paths, each starting ordinal zero. H's two columns
are (1,1)/√2 and (1,−1)/√2; S's second column is (0,i), distinguishing
these witnesses from a bit permutation and fixing global phase.

Number every emitted leaf (including allocation/release and classical
leaves) 0,1,… within the containing row/instruction, after instantiating
widths and static loop counts, before any §5.7 transformation or target
scheme expansion. Calls have their own invocation-path element and
callee ordinal space; `step` and `phase` select the invocation's oracle
step as in §11. The inverse of a row visits the forward ordinals in
reverse order, retaining each ordinal under `phase: rev`; it does not
renumber them. Scratch identity is `(invocation chain, step, recipe
expansion path, allocation ordinal, bit index)`; allocations in repeated
bodies have their expanded occurrence path. All library scratch is
clean, allocated unconditionally and released in reverse allocation
order after uncomputation; borrowed user operands remain borrowed.
`removable` is an explicit array of leaf ordinals and MUST be empty
for every recipe in `ref` version 0.9. No transformation
may infer extra permission from a zero-valued basis input. Motivation
(0.8 §11): "MUST keep the ordinals of the operations they keep and MAY
remove only operations the library marks removable"; §17 item 7:
"clean scratch released in reverse order".

The reverse-allocation release rule governs recipe scratch; an instruction whose own semantics fix a release sequence (qinplace per §4.5: `qfree t; qfree f`) keeps that sequence, and its recipe MUST reproduce it exactly. [deps: ISA-4.5-P5, ISA-11.1-P21]

**Instruction-release witness (K28).** For n=1, F=4 and identity oracles,
qinplace emits allocation t (row ordinal 0), allocation f (1), G's CX
(callee ordinal 0), exchange x/t (row 2), Ginv's CX (its own callee
ordinal 0), release t (row 3), release f (4). [deps: ISA-4.5-P5, ISA-5.4, ISA-11.1-P21] The certified pair leaves
t/f clean for both x values and both effective controls; allocations and
releases are unconditional. [deps: ISA-4.5-P5, ISA-3.5, ISA-4.3] Recipe-internal scratch within either call
still releases in reverse allocation order. [deps: ISA-11.1-P21] Inverse traversal retains
these ordinals, reverses their order and adjoints the leaves (§5.3). [deps: ISA-5.3, ISA-11.1-P21]

**Decomposition set.** The §17 item 7 defaults are made explicit below. Bit
loops below increase from the least-significant bit. To make Boolean
recipes deterministic, use no common-subexpression elimination: visit
every Boolean expression tree in left-to-right postorder, giving each
internal node a fresh clean scratch bit. NOT(a) uses X d; CX a d,
XOR(a,b) uses CX a d; CX b d, AND(a,b) uses CCX a b d. OR(a,b) is the
ordered tree XOR(XOR(a,b),AND(a,b)); repeated operands are separate tree
occurrences (coincident AND operands reduce to a single CX so the gate
has distinct operands). XOR the root into the destination, then reverse
all compute gates and release scratch in reverse allocation order.
Constants are zero or X of a clean bit. All formulas use old source
values captured before an assignment/exchange. External effective
controls apply to the resulting semantic gates, not allocations; scratch
computations may be unconditional provided their compute/uncompute pair
is balanced when the external control is false (§3.5, §4.3).

| Family | Ordered expansion | Motivating 0.8 sentence |
|---|---|---|
| Copy, XOR, exchange; native gates | `qxor`: CX s[i] t[i]; `qcopy`: allocate then the same; `quncopy`: the same then release; `qexch`: SWAP a[i] b[i]. Keep each §4.4 gate as a leaf; target schemes expand later. | §4.5: "`cx s[i] t[i]`" and "`swap a[i] b[i]`"; §8.3: "Every template gate MUST itself match a gates entry". |
| Ripple add/sub and comparisons | c[0]=0; p[i]=a[i] XOR b[i]; sum[i]=p[i] XOR c[i]; c[i+1]=(a[i] AND b[i]) XOR (p[i] AND c[i]); compute each bit in this order, XOR sums out, reverse scratch. Subtraction adds bitwise NOT b with c[0]=1. Unsigned a<b is NOT the final subtraction carry; equality is the AND of NOT(a[i] XOR b[i]) in increasing-bit left-associated order. `ule` is OR(ult,eq); signed comparison selects sign(a) if signs differ, else ult. | §17 item 7: "ripple-carry adders and comparators"; §4.1: "wrap"; §3.7: prospective sums use "no wrap" (retain the extra carry for those). |
| Other arithmetic, casts, select | AND/XOR/OR use the Boolean trees above; select emits ((NOT p) AND b[i]) XOR (p AND a[i]). Multiply folds shifted partial products in increasing multiplier-bit order through ripple add modulo 2^N. Shifts scan shift amounts j=0..N−1, equality-selecting the corresponding source bit; ≥N produces zero. Sign-extend replicates the high source bit, zero-extend supplies zero, truncate drops high bits. Division uses restoring unsigned long division, scanning dividend bits from N−1 to 0: append that bit to an N+1-bit remainder, compare with the zero-extended divisor, subtract iff ≥, and emit that predicate as quotient bit. Signed forms conditionally negate operands, divide magnitudes, negate quotient iff signs differ and remainder iff dividend negative; row fault predicates precede these outputs. | §4.2: "w (under ¬flag)" and the div fault predicates; §4.1 wrap/shift rules and signed MIN/−1 fault; §3.3 equal operand widths. |
| Linear inrange and flags | For table entries i=0..2^A_tab−1 compute `(i<atop) AND live_i AND (base_i≤p) AND (p+n≤base_i+size_i)` in that parenthesisation at §4.2's widened width; left-fold OR from false, then uncompute entry scratch. `free` and allocator/call fault trees use their explicit §4.2/§3.7 formula and slot order (including 0.9's E17.13 correction). | §4.2: "inrange(p,n) = ∃ i < atop ..." and "no wrap"; §17 item 7: "linear inrange scans". |
| Address dispatch and multiplexers | Scan every address i in increasing numeric order; compute equality to i, copy/exchange cell bits in increasing order under that bit, uncompute equality before the next i. Lifted memory chooses the §7.2 mapped region in address order and scans its cells in increasing address order; captures use old addresses before exchanged counters. | §4.6: "for each cell i: under a == i"; §17 item 7: "one-hot address dispatch"; §3.7 C1–C5/R1–R5. |
| Row, oracle and structure | Compose these recipes in the printed §3.6 step order, §4.2 slot order, §3.7 activation order, §5.2 allocate/forward/commit/reverse/release order. `qif`, loops, local-region image X gates and `qinplace` use the literal sequences of §4.5–§4.6, incorporating 0.9's explicit parameters. Cleanup `reverse` adjoints that sequence; `recompute` follows §6.4. | §11: "Above gate level the order is fixed by §3.6, §4.2, §4.5–§4.6 and §5.2"; §5.2: "run the inverse step for t = B..1, then release (1)". |

The permutation algorithms define exact permutation circuits, preserving
every input required by the row and cleaning scratch; circuit correctness must still be checked
against full-domain T3 witnesses and §5.7, not inferred from costs or
names. Complete draft catalog: **219 recipes**; envelope SHA-256
`fcb2f1e6d61aced0bf9689eab5f6d82c4ae42e9fea8d1b21ae9c7ffccda6f5ce`. [deps: ISA-11.1-P2, ISA-11.1-P3, ISA-11.1-P28, ISA-11.1-P29]
Publication follows acceptance. [deps: ISA-11.1-P1, ISA-11.1-P2] In particular this is
not permission to publish a partial library that omits division, stack,
coherent control, memory or any instruction.

**ISA-REF-LIB witness**

```json
{"isa":"0.9","name":"ref","recipes":[{"kind":"qxor","operations":[{"gate":"cx","ordinal":0,"wires":["s[0]","t[0]"]},{"gate":"cx","ordinal":1,"wires":["s[1]","t[1]"]}],"removable":[],"scratch":[],"source":"0.8 §4.5","width":2}],"version":"0.9"}
```

This is a single instantiated recipe witness, **not** a complete
library document for binding a schedule. Its envelope is the four-line
envelope above with this entire JSON as line four. It has two ordinals,
0 and 1, no scratch and no removable operations. For every two-bit s,t
(16 tuples), the result is `(s, t XOR s)`; reversing visits ordinal 1
then 0 and restores the entry tuple. Its hash identifies only these
witness bytes, not `ref@0.9` as a whole.

Witness envelope SHA-256: `299510fd67cc626fdcbcc10adcd40730f9af294ba596d86fb25fd31be08a5c7f`.

## 12. Certificate (cert 0.9; JCS)

### 12.1 Claims

| kind | payload | checker obligation |
|---|---|---|
| `exhaustive` | `domain` (§12.2); `expected`: `"oracle"` \| `{ "table": [...] }` \| `{ "program": closure_hash }` \| `"inverse_pair"` | re-execute every domain element on the reference interpreter / expected program; compare full tuples. For `"inverse_pair"` (the claim of an `inverse_pairs` entry, §12.3): for every element (x, ins, mem cells, args) run g and ginv at the entry's tuple, token-typed arguments at their `arg_tokens` tuples, and require both fault-free, ginv(g(x)) = x and g(ginv(x)) = x; the bundles and regions components MUST be the full domain of g's `in` and `mem` slots (the `args` component lists the covered classical values) |
| `sampled` | `domain`; `expected`: `"oracle"` \| `{ "table": [...] }` \| `{ "program": closure_hash }`; `subject` and invocation binding as below (K13); `sampler`: `{ "algorithm": "xoshiro256**", "seed": [4 decimal strings, not all zero], "count": n }` | independently replay both sides of the declared relation on every sampled element; compare full tuples; always **evidence**, even if every element is drawn |
| `structural` | `{ "rule": "translation_lemma" \| "inverse_pair_library" \| "adjoint_table", "subclaims": [indices], "witness": … }`; for `translation_lemma`: `{ "V": [points], "W": [points], "X": [wires], "B": [wires], "frame": [first point, last point] }`; for `inverse_pair_library`: `{ "library_closure_hash", "claim_index" }`; for `adjoint_table`: `{}` | check the rule's side conditions syntactically against the object and the subclaims' results |
| `numerical` | `{ "subject": { "closure_hash", "variant", "inherited_controls", "schedule_hash" }, "metric": "operator_norm", "precision_bits": n, "bound": decimal }` | **checker-computed, for every invocation context of the subject** (below): build the realised unitary U of that invocation (the region's realisation under the root schedule at that path, with its resolved classical arguments, budget, epsilon shares and cleanup choices, and `inherited_controls` control wires prepended) and the reference unitary V (the region's §5.3 semantics with the same arguments, controlled on the same wires) at the subject's signature widths in interval arithmetic at `precision_bits` (basis order = allocation order, controls first), form interval enclosures of `V†U − I` entrywise, derive the operator-norm bound as the Frobenius-norm upper bound in interval arithmetic, and verify it is ≤ `bound`; the claim holds iff it holds for every context |
| `channel` | `{ "subject": as above, "metric": "diamond", "precision_bits", "bound", "dual": [ { "Y": interval Hermitian matrix, "mu": decimal } … ] }` | for every invocation context of the subject, in enumeration order, verify its dual witness for the **diamond-norm SDP** below in interval arithmetic and `mu` ≤ `bound` (one witness per context; too few witnesses: reject) |

**CLAIM-REPLAY** (K13; b-isa-spec-9uy, from bennett-vm-clw).
Sampled observations remain evidence under the following replay contract.

**Comparison relation.** `exhaustive` and `sampled` claims
MUST name their `subject: { closure_hash, variant, inherited_controls,
schedule_hash? }` and an explicit `expected`; there is no implicit
self-comparison or default expected relation. Without a schedule the
left side is the subject's reference semantics at the bound invocation;
with one it is that invocation's realised operation. `expected: "oracle"`
compares the realised operation to §5.2's reference oracle map and
requires `schedule_hash`. `expected: {program: H}` independently runs
the reference semantics of linked closure H at the **same** arguments,
budget and interface tuple; its signature MUST be compatible.
`expected: {table: rows}` compares to the explicit full-tuple rows
below. The checker MUST compare every interface output, preserve inputs
and borrowed systems, and verify clean workspace and exact +1 basis
phase for a permutation assertion (T3); matching a target or fault
projection alone, or a producer's claimed verdict, is insufficient.
For a non-permutation realisation, use the existing numerical/channel
claims rather than labeling basis observations a permutation proof.
K4's `expected: "inverse_pair"` retains its separate two-composition
relation and entry/domain bindings; it is not a sampled proof option.

**Table tuples.** Each table row is `{ before: tuple,
after: tuple }`. A tuple is `{ bundles: ["<unsigned integer>", …],
borrowed: ["<unsigned integer>", …], regions: [["<cell>", …], …] }`.
`bundles` lists inherited control wires first (each width 1), then all
signature bundles in operand order, **including arbitrary initial
oracle target and fault bundles**. `borrowed` lists dirty bundles in
allocation order, with widths fixed by the checked borrow lifetimes;
`regions` lists interface regions in signature order, every cell in
increasing address order, each as an unsigned W-bit integer. All
integers use canonical decimal strings, within the declared widths;
no cells, inputs or unchanged components may be omitted. Workspace is
not an interface component and MUST finish zero. Rows are in domain
enumeration order, one per distinct declared domain element (including
for `sampled`); duplicate/missing/out-of-range rows reject. This favors
an explicit table; `{program: H}` provides the compact relation when a
full table is large. Sampling draws with replacement using the existing
sampler; coverage counts distinct drawn elements, not draws. A mismatch
rejects; success is always `evidence` with that distinct covered fraction
of D, including when the draws happen to cover all of D.

**CLAIM-REPLAY witness**

```json
{
  "invocation":{"args":["2"],"budget":["2","0","0","0"]},
  "row":{
    "before":{"bundles":["1","2","3"],"borrowed":["1"],"regions":[["2","1"]]},
    "after":{"bundles":["1","1","3"],"borrowed":["1"],"regions":[["2","1"]]}
  },
  "wrong_row":{
    "before":{"bundles":["1","2","3"],"borrowed":["1"],"regions":[["2","1"]]},
    "after":{"bundles":["1","2","3"],"borrowed":["1"],"regions":[["2","1"]]}
  },
  "changed_arg_after":{"bundles":["1","0","3"],"borrowed":["1"],"regions":[["2","1"]]},
  "short_budget_after":{"bundles":["1","2","2"],"borrowed":["1"],"regions":[["2","1"]]},
  "sampler":{"algorithm":"xoshiro256**","seed":["1","0","0","0"],"count":4},
  "sample_count":4,"distinct_samples":1,"covered_fraction":"1/2048","verdict":"evidence"
}
```

The subject is a two-row oracle body `r = lift.add x a;`
`lift.retval r`, widths x = target = fault = a = 2, with a preserved
one-bit dirty bundle and an unused two-cell, two-bit `in` interface
region; no inherited controls. The **body** does not access memory;
the interface cells remain part of D. The reference payload is
(1 + 2) mod 4 = 3: target 2 XOR 3 = 1 and fault 3 XOR 0 = 3.
The singleton declared domain is precisely `row.before`, within the
full 11-bit D. Four samples of singleton components draw no generator
words and cover one of 2048 full tuples. An identity implementation
would agree with itself but MUST fail against `row`; using `wrong_row`
also MUST fail. At argument 1 the target is 2 XOR 2 = 0; at B = 1 the
payload is zero and fault is 3 XOR 1 = 2. A certificate for argument 2,
B = 2 cannot be replayed as either of those invocations. The printed
rows illustrate the relation; bound closure/object hashes still have
to be supplied in an actual certificate, not invented for this witness.

**Invocation contexts.** A subject denotes the set of invocations of
the region `closure_hash`, with the given `variant` and `inherited_
controls`, in the root schedule `schedule_hash`: every invocation of
the region in the root realisation's complete invocation tree —
including invocations that no schedule entry names and defaults alone
cover (§11) — with the classical arguments, budget, `epsilon_share`s and
cleanup realisations resolved for it. An invocation's **context key** is
its concrete invocation chain (§11), outermost first, each element `{
"symbol", "point", "iteration" }` with every iteration index resolved
and, for an element inside an oracle, `"step"` (1..B) and `"phase"`
(`"fwd"` or `"rev"`) added; contexts are enumerated in increasing
bytewise order of the JCS encodings of their keys (wildcard selectors
play no part). Every claim bound and every
`error` entry (§12.3) is a **common upper bound over all contexts** of
its key; the multiplicity composition of §12.3 is sound for that reason.

**Diamond-norm SDP** (Watrous, *Theory of Computing* 5 (2009), §4; the
Hermiticity-preserving simplification). Let Φ = Φ_real − Φ_ref be the
difference of the subject's realised channel and the reference channel
(both on the subject's interface, controls included), d_in = d_out = d
the dimension. The **Choi matrix** is J(Φ) = Σ_{i,j} |i⟩⟨j| ⊗ Φ(|i⟩⟨j|)
(input factor first, output factor second, computational basis in
allocation order, **unnormalised**). The witness is a Hermitian Y of
dimension d² and a decimal mu. The checker accepts iff, with every
matrix entry an interval and every test rigorous (an inconclusive test
rejects): Y − J(Φ) ⪰ 0, Y + J(Φ) ⪰ 0, mu·I − Tr_out(Y) ⪰ 0 (partial trace
over the output factor), and mu ≤ `bound`; then ‖Φ‖◇ ≤ mu (the dual of
Watrous's SDP with Y₀ = Y₁ = Y; this is the diamond norm itself, not
half of it). *(non-normative)* With this input-first Choi convention
Tr_out traces the second factor, Tr[(ρ ⊗ I)Y] = Tr[ρ Tr_out(Y)]; a
primal formulation that labels the input factor ρᵀ ranges over the same
set of density matrices, so the bound is unchanged. Encoding: `Y` is a row-major array of entries `[re_lo, re_
hi, im_lo, im_hi]` (decimal strings). **Positivity test.** The checker
proves each of the three matrices **positive definite** by an interval
Cholesky (or Hermitian LDL†) factorisation in which the interval lower
bound of every pivot is strictly positive, and rejects otherwise; no
other test is admitted (nonnegative leading principal minors do not
imply positive semidefiniteness: diag(0, −1)). Producers make witnesses
strictly feasible by replacing Y with Y + δI and mu with mu′ = mu + δ·(d
+ 1) for a small rational δ > 0 (any mu′ > mu + δ·d suffices): then Y ±
J(Φ) ⪰ δI ≻ 0 and, since Tr_out(Y + δI) = Tr_out(Y) + δ·d·I, mu′·I −
Tr_out(Y + δI) ⪰ δI ≻ 0; ‖Φ‖◇ ≤ mu′, which MUST be ≤ `bound`. That
recipe assumes an exactly feasible witness. For a numerically obtained
witness the producer first certifies lower bounds −η₊, −η₋, −η_s on the
smallest eigenvalues of Y − J(Φ), Y + J(Φ) and mu·I − Tr_out(Y), chooses
δ > max(η₊, η₋, η_s) and publishes Y′ = Y + δI and mu′ = mu + (d + 1)·δ:
then Y′ ± J(Φ) ⪰ (δ − η±)I ≻ 0 and mu′·I − Tr_out(Y′) ⪰ (δ − η_s)I ≻ 0.
These margins make the matrices positive definite; they do not make the
interval test conclusive: every pivot enclosure the checker computes at
`precision_bits` MUST still have a strictly positive lower endpoint,
else the checker rejects (no fixed allowance such as 2^−`precision_bits`
bounds the interval error, which depends on d and conditioning). **Interval
arithmetic** at `precision_bits`
means every operation rounds outward to `precision_bits`-bit binary
rationals; transcendental entries (`rx ry rz`) are enclosed by a Taylor
series with the Lagrange remainder bound (§17 item 8 fixes the term
count). A `numerical` or `channel` subject's `schedule_hash` is required:
it names the realisation the claim is about. K13 also requires it for
`expected: "oracle"`; standalone reference relations use the explicit
invocation binding instead.

**Sampling.** For each domain component with range size R = hi − lo + 1,
assemble m = ⌈⌈log₂ R⌉/64⌉ consecutive generator words (least-significant
word first) into an integer, reduce modulo 2^⌈log₂ R⌉, reject and redraw
if ≥ R, add `lo`; components are consumed in enumeration order.

### 12.2 Domains

`"domain": { "bundles": [ { "name", "width", "lo", "hi" } ], "borrowed": [ same ], "regions": [ { "name", "cells": "all" | [ { "index", "lo", "hi" } ] } ], "args"?: [ same as bundles ] }`,
bounds inclusive, enumeration order as listed, last component fastest.
`args` (inverse-pair claims only) ranges over classical arguments other
than tokens (whose tuples the entry's `arg_tokens` fixes, §12.3), each
read as an unsigned integer of its width (`ptr`: ptr_width; `f64`: 64,
its bits).
A `proof` label requires the domain to equal D; otherwise the claim is
`evidence` with the covered fraction.

### 12.3 Schema and composition

```
  versions; bindings { object_hash, interface_hash, closure_hash (required), reference_program_hash, dialect_mapping_version, target_hash, schedule_hash?, invocation? (K13) }
  producer { name, version, materialisation { llvm_version, pipeline, normaliser_version } }
  regions { <closure_hash>: { kind, signature, bounds, estimate? (K12), frame (required typed-record descriptor, §9.4), resources { name: decimal }, gates_used { gate: { count, scheme?, epsilon? } },
            claims: [ <claim> … ], verification: { oracle: i, borrow_frames: [i], inverse_pairs: [ { g, ginv, claim: i, budget: [B,heap,stack,depth], arg_tokens?: { "<arg>": [B,heap,stack,depth] } } ], unitary: [i], channel: [i],
                                                   fault_rate: { domain, faults_by_code, undefined_excluded, layout_dependent_excluded } },
            cleanup: { "<variant>:<inherited_controls>": [ { path, realisation } ] },
            error:   { "<variant>:<inherited_controls>": { local: { "operator_norm"?: v, "diamond"?: v }, inclusive: { "operator_norm"?: v, "diamond"?: v } } },
            definedness_assumptions: [ids] } }
  composition { multiplicities: { "<hash>:<variant>:<inherited_controls>": n (from the realised schedule) }, epsilon_total: { "diamond": v }, method: "sum_over_multiplicities" }
```

**Invocation binding (K13).** An invocation is
`{ args: [value, …], budget: [B,heap,stack,depth] }`, with every value
resolved before replay and all four budget components explicit even
when zero. `args` is in signature order: `iN` and `ptr` values are
canonical **unsigned decimal strings** of exactly that bit-vector
value (ptr has `ptr_width` bits); `f64` is `0x` followed by exactly 16
lowercase hex digits of its IEEE 754 binary64 bit pattern, most
significant nibble first (so −0 and NaN payloads are not lost); a token
is its four-component tuple of canonical nonnegative decimal strings,
not a process-local token id. Budget components are also canonical
nonnegative decimal strings. Signature types disambiguate encodings;
widths/ranges and every tuple MUST pass §5.4 validation. `params_hash`
is SHA-256 of the UTF-8 JCS **args array**, no trailing LF (§9.2), not
of the whole invocation. Thus equal token tuples have equal parameter
hashes regardless of ids, while +0 and −0 `f64` parameters differ.

Each standalone table/program/structural claim MUST carry its own
`invocation`; it binds
**every** classical argument and budget. Missing values have no implicit
zero/default. Claims with no arguments still bind `args: []` and their
budget. K4 inverse-pair claims instead bind the entry's budget,
`arg_tokens` and quantified `args` domain as already specified; this
rule does not replace that quantification by one chosen argument.
A parameter-uniform structural claim MAY instead bind
`invocation: { args: "all", budget: "all" }`: its checker MUST discharge
the rule's side conditions for every well-typed argument vector and
valid budget, not by replaying a chosen instance. Thus generic library
and adjoint/translation proofs retain their universal scope.
A scheduled certificate MUST carry the root `bindings.invocation`;
its args hash and budget MUST equal the root schedule's bindings, and
all nested arguments and budgets MUST be resolved at each realised
invocation context (§12.1), including default-covered contexts.
Explicit nested bindings, if supplied, MUST equal those resolutions;
a root binding MUST NOT be silently reused for a different callee.
A checker MUST reject absent or conflicting bindings before replay,
and an executor may use a verified result only for the invocation(s)
it actually covers. Reference table/program/structural claims need no
schedule; they prove only their reference relation, not a target
realisation. Scheduled/numerical/channel claims retain the expansion
library gate and all existing context and composition obligations.

**CLAIM-BINDING witness**

```json
{"signature":["i128","f64","token"],"args":["18446744073709551617","0x8000000000000000",["2","0","0","0"]],"params_hash":"9f628a694fa7259bf899bfb677aabce1c084e268a738a40f7f643eb6d8c9c093"}
```

The first argument is > 2^64; the second is binary64 negative zero;
the third is a token tuple, not an id. Replacing negative zero with
`0x0000000000000000` changes `params_hash`; replacing the token by
another valid id of the same tuple does not.

`versions` is the object `{ "isa", "object", "launch", "sched", "cert",
"coverage", "target" }` of the seven header versions. A **variant** is a
nonempty string chosen by the producer (the ISA enumerates none); it is
compared bytewise.

`inclusive(R, key) = local(R, key) + Σ_callees n_c × inclusive(c, key_c)`
**per metric**, keyed like `cleanup` by `<variant>:<inherited_controls>`
(the realised error depends on both); every `local` value is a verified
common upper bound over all invocation contexts of its key in the bound
root schedule (§12.1), so multiplying by n_c is sound; an
`operator_norm` value converts
to `diamond` by ×2 once, at the point it is first combined with a
`diamond` value; `operator_norm` conclusions (T6, controlled unitary
variants) require `operator_norm` bounds throughout. A controlled
unitary variant requires a `numerical` claim; an ε > 0 realisation
compared as a channel requires a `channel` claim. Executable claims
(`numerical`, `channel`, and `exhaustive` or `sampled` with
`expected: "oracle"`, plus table/program comparisons of a realised
operation) bind to `schedule_hash` through their subject. Standalone
reference relations have only the invocation scope defined above (K13).

## 13. Coverage (coverage 0.9)

`{ "entries": [ { "point": <program-point id>, "source": "<dialect-spec stable id>", "classification": "lowered" | "trapped" | "shimmed", "reason", "fault_code"?, "shim_hash"?, "count" } ] }`.
`fault_code` is present iff `classification` is `trapped`, `shim_hash`
iff it is `shimmed`.

## 14. Formal model

### 14.1 Reference interpreter *(normative)*

Executes an oracle's reference body per basis configuration: locations
and cells are bit-vectors of their declared widths (no upper bound,
§3.3), pseudo-locations read-only, predication is
`if`, each §4.2 row is its classical function (including the inherent
`inrange` check of loads and stores, the activation records of §3.7, the
`alloctab` and stack arrays), `fault` and a triggered `lift.faultif`
record their effective cause (§5.4) and halt, `lift.
retval` halts with success and `f(x) = result_val`, the step counter
counts executed rows (on budget exhaustion exactly B rows have
executed), counters are unbounded integers. It **defines** F_B
and the budget unit and rejects any body that executes a non-permutation
gate.

For CC/R, `fault C` has the terminal-record semantics of §4.9 and the
logical history of §7.4: it preserves the complete executed prefix for
eligible reversal, including activations that have not returned. This
is T4's history traversal, not the Q machine's B inverse steps. In an
oracle body or lproc the same spelling remains the §4.2 fault row;
§3.6 and §5.2 supply its step-machine inverse and T2 prefix reversal.

### 14.2 Quantum state semantics

The state of a Q region is a vector in ⊗ over live wires in allocation
order (bundles little-endian). `qalloc` tensors |0⟩; `qfree` removes
clean factors; `qborrow` introduces a factor in unknown state possibly
entangled with a reference system R, and the region's operation must be
U ⊗ I on it (§5.6); gates act by §4.4 under the active predicate;
`measure` (CC) is the projective computational-basis measurement with
Born probabilities, consuming the wires.

### 14.3 Theorems

- **T1 (totality).** For every well-formed oracle, budget and x ∈ D the
  reference interpreter halts within B + 1 rows and F_B(x) is unique.
- **T2 (prefix reversal).** For every executor whose realisation is a
  composition of ε = 0 permutation gates implementing §5.2 steps 1–4
  with the inverse step of §3.6, on every x ∈ D every non-interface wire
  is clean after step 4 and borrowed bundles hold their initial values.
- **T3 (permutation theorem).** If the realised unitary V of a circuit on
  span(D) is a permutation matrix and the final basis tuple equals P(x)
  for every x ∈ D, then V|span(D) = P.
- **T4 (round trip).** For every finite CC/R trace with retained run
  history and tapes that is eligible for reversal under §10 — a
  `COMPLETED` or `FAILED` run containing no `measure` frame and no
  `qreg.free` frame whose cleanliness was not verified — the history
  traversal of §7.4 restores the active environment, memory, allocation
  table, device-model state, the ledger entries of every register the
  run created or touched, token validity and active history (§7.4) to
  their entry values; audit records may remain; observed external
  events are not retracted. If, in addition, every region call is undone
  by the exact inverse of its realised forward operation (ε = 0
  realisations, or a realisation that is applied as its own adjoint),
  the quantum state of every register the run touched is restored
  exactly; otherwise to within the sum of the realisations' declared
  bounds (§12.3).
- **T5.** As §6.3.
- **T6 (executor agreement).** Two executors conforming to §3–§7 on one
  target with the same schedule produce on every x ∈ D the same final
  tuple for every oracle (exact) and, for every action with `operator_
  norm` inclusive bounds e₁, e₂, unitaries within e₁ + e₂ in operator
  norm; for channels, diamond bounds d₁ + d₂.

### 14.4 Mechanisation *(deferred)*

## 15. Conformance suites

| Suite | Class | Tests |
|---|---|---|
| A | Executor | classical ISA; run history and LIFO reversal with loops, recursion, effects, segment `uncall` (impure suffix traps; nested `uncall` inside an uncalled segment; `uncall` over a `measure` traps; a segment that begins at the run's first instruction, uncalled, then whole-run reversal), `hostcall`, `qreg.*`, fault records (T4); CC `uncall` of a CC segment (§3.1); a use reached before any definition traps 4 whatever the block text order; `qreg[*]` (examples 4, 5): run-time widths 0, 2, 3, an id stored and reloaded, `qreg.from_id` of a stale, consumed or viewed id or of a register owned by another open run traps 4, a width-3 register bound to `wire[2]` or `qreg[2]` traps 4 with no effect, aliases consumed together, ids restored and never reused by reversal, run-time slice `lo` and `len`; `qreg.*` in R with whole-run reversal and segment `uncall` (a hardware-unverified free makes the segment non-uncallable); `budget.mint` valid and invalid and its reversal (example 6); the §4.9 causes — `udiv` by 0 → 5, i8 −128 `sdiv` −1 → 6, `load` at address 0 → 7, `alloc` beyond the heap → 2, `trap` → 4, `fault 1` → 1, `fault 0` → 4 — each with the pre-fault state unchanged and exactly one record; terminal R faults and FAILED-run reversal (example 8), also with `fault 2`, `fault 3` and `fault 18446744073709551615`: exact record code, no caller continuation or subsequent source effect, earlier memory/device/tape history intact until reversal |
| B | Executor | oracle totality and prefix reversal: every fault code; B = 0; boundary halts; `lift.call`/`lift.ret` recursion with argument capture; one forward `lift.call` and one forward `lift.ret` step of a recursive call each followed by its inverse (§3.7); stack exhaustion (`sp + R_f > stack_limit`); matched `lift.oracle_uncall`, including two calls followed by two uncalls; interface regions restored; pre-entries; `lift.free` on pinned entries, and of a heap entry whose base equals a size-0 global's (the global stays live); inherent load/store checks; row indices of multi-row blocks and their `lift.label` values; `qjumpr` to an in-range interior row enters it and to an out-of-range index faults 4; `fault 0` and a triggered `lift.faultif p 0` commit `FAULT_UB_TRAP`; `fault C` as the B-th executed row reports C′, whereas a cut just before that row reports 1; subsequent HALT steps and inverse steps preserve the §5.2 outcome and clean every frame; counters at the §3.7 minimum widths and interface widths above 128 bits |
| C | Executor, Producer | T3 over full D at exhaustive widths for oracles, permutation actions, `qinplace` pairs, borrow frames; T6 agreement among VM, specialiser, transformed forms and the reference interpreter; the §16.3 examples as normative test vectors, including the stack-capacity boundary of example 2 and example 3 over its 8 basis and 16 controlled configurations with an entangled reference on the borrowed wire; `qinplace` with `ins` and `args` (example 7) over all x, y, a and both control values, scratch clean, and its adjoint (covered by the same entry); example 10's lproc token/classical/region forwarding over every x, a and byte d, arbitrary target/fault contents, and an external reference, with successful and insufficient nested budgets; coherent calls to two binding instances of one lproc, including recursive forwarding, and matched `lift.oracle_uncall` through parameters; a library (`inverse_pair_library`) claim covering every `args` value; a token argument whose tuple differs from `arg_tokens` traps 4 |
| D | Executor | channel and unitary comparison for ε > 0 realisations; controlled-interference tests; checker-computed bounds; the diamond-norm SDP witness check of §12.1, including d = 2, J(Φ) = 0, Y = I₄/8, mu = bound = 3/8 (accepted: every pivot 1/8) and Y = 0, mu = 0 (rejected: pivot 0) |
| E | Target author, Executor | JCS parsing; AST evaluation; matching; scheme expansion and correctness; aggregation from event lists; timing model; imports |
| F | Producer | versioning, disjointness incl. control context; operand schema; row classes and frame sizing; index-space flattening; terminators; name resolution; permutation-only oracles; §6 transfer and windows; adjoint table; translation lemma side conditions with the four negative tests; `in`-wire use; acyclic action graphs; grammar; schemas; the §3.1 call-tier table (R → CC rejected); envelope and hashes, acyclic (inc8, zero, sum_db) and cyclic (sum, and a mutual SCC); SCC placement in the index space (mutual f, g whose member hashes order opposite to their names); empty data lines (`NAME SP LF` accepted, two spaces rejected); canonical `INT`/`FLOAT` at every literal site; const literal ranges; the region call graph with lifted calls and `qinplace`; `call` operand spelling; param role suffixes; interface-region names; local regions as call operands; the widened `in`-wire uses of §5.1; LIFO matched uncalls and an unmatched uncall rejected; the static view rule of §3.3 (a view passed through `br`, used twice, or left unconsumed is rejected); lifted-call `args` that are locations rejected; lproc header groups and `token`/`classical` suffixes (a suffix in the wrong group rejected), exact argument counts/types/region roles, missing nonempty lists and location-as-token/classical rejected, forwarding `iN ptr f64 token` without bit conversion, empty-list short form, fixed-binding closure and instance ordering (§3.7), lproc signatures including empty arrays and changed interface/closure hashes; `qreg[*]` typing, the `measure.iN` form and width checks; `budget.mint` rejected in regions; CC/R block terminators; `qinplace` signature compatibility, its short form exactly when the three lists are empty, and its §6.1 transfer (cleaning a value that depends on x after changing an `ins` wire is rejected); CC/R `fault` literal range; linking; frame-descriptor live ranges |
| G | Launch | every §10 entry point, state transition, binding rule, ownership rule; HOST_REQUEST/reply (a reply whose length does not fit `ptr_width` bits is `E_ARG`); reverse eligibility and tree order; `QREG` bound to `qreg[*]`; `FAILED → REVERSED` (example 8) and `E_NOT_REVERSIBLE` after a `measure` or an unverified free; exactly one fault record per `FAILED` run; an R `fault C` returns `OK` from `bennett_wait` with a result and no request, zero values and the exact nonzero 64-bit C (including 2^64 − 1) from `bennett_result_fault(result, 0)`; a device-minted token used after its run is CLOSED is `E_ARG` |
| H | Scheduler | §6 validity; capacity; reproducibility; specificity resolution; verbatim execution; INFEASIBLE/UNSUPPORTED; invocation contexts covering default-only invocations (an action calling one region twice, one call scheduled explicitly: a claim with one witness rejects, two passing witnesses accept); a device-minted tuple binds the schedule (a schedule bound to B = 3 is not applied to an invocation minted with B = 2, which runs under the defaults) |

## 16. Canonical textual form (normative)

### 16.1 Lexical

UTF-8; LF line endings; one instruction per line; tokens separated by
exactly one space; no trailing spaces; exactly one blank line before
each `section` line and none elsewhere. `NAME := [A-Za-z_][A-Za-z0-9_.]*`;
`INT := -?(0|[1-9][0-9]*)`; `FLOAT :=` shortest round-trip decimal with
`.` or exponent, finite; `STRING := "…"` with escapes `\"`, `\\`, `\n`;
`HEX := ([0-9a-f]{2})*`. Metadata lines are RFC 8785 canonical JSON
objects. Every integer token is an `INT` (`ptr_width`, type widths,
`bounds`, literal operands, data and region sizes), so `032`, `+3` and
`09` are ill-formed; every gate angle is a `FLOAT` whose spelling is the
ECMAScript Number-to-String rendering that RFC 8785 uses for numbers,
with `.0` appended when that rendering contains neither `.` nor `e`, and
`-0.0` for negative zero (`0.7`, `1.0`, `1e+21`, `-0.0`; never `0.70`,
`1.0e0` or `00.7`). Where §16.2 prints `SP` twice around an empty list
(`regs {  }`) that spelling is canonical. A data line whose `HEX` is
empty is `NAME SP LF`: its mandatory separator space is the sole
exception to the no-trailing-space rule. The text has no comment
syntax; annotations travel outside the object.

### 16.2 Grammar

```
  object    := header NL section*
  header    := "bennett-object" SP "isa=" VER SP "object=" VER SP "launch=" VER SP "sched=" VER SP "cert=" VER SP "coverage=" VER SP "target=" VER SP "ptr_width=" INT
  section   := NL "section" SP SECTNAME NL body
  SECTNAME  := "symbols" | "imports" | "functions" | "data" | "devices" | "layout" | "schedule" | "certificate" | "coverage"
  body      := jsonline | dataline* | function*
  jsonline  := JCS-canonical JSON object NL
  dataline  := NAME SP HEX NL
  function  := fhead NL regs? block+
  fhead     := ("fn" SP ("cc"|"r") | "oracle" | "action" | "perm_action" | "lproc") SP "@" NAME SP "(" plist ")" SP "(" plist ")" SP "(" plist ")" SP "(" plist ")" SP "(" plist ")"
               -- oracle: (in..) (tgt) (fault) (mem..) (args..) ; action: (mut..) (in..) (mem..) (args..) () ; lproc: (params..) (result) (tokens..) (classical..) (mem..) ; fn: (params..) (results..) () () ()
  plist     := (param ("," SP param)*)?
  param     := NAME ":" type (SP ("in"|"mut"|"token"|"classical"))?
  type      := "i" INT | "ptr" | "f64" | "token" | "wire" | "wire[" INT "]" | "reg[" INT "]" | "region[" INT "," INT "]" | "qreg[" (INT | "*") "]"
  regs      := "regs" SP "{" SP (NAME ":" type ("," SP NAME ":" type)*)? SP "}" (SP "bounds" SP "{" INT ("," SP INT)* "}")? NL
  block     := "^" NAME "(" plist "):" NL instr+
  instr     := (lhs SP "=" SP)? OPCODE (SP operand)* (SP "{" NL instr+ "}" (SP "{" NL instr+ "}")?)? NL
  lhs       := NAME | "(" NAME ("," SP NAME)* ")"
  operand   := NAME | NAME "[" INT "]" | "@" NAME | "^" NAME ("(" (NAME ("," SP NAME)*)? ")")? | INT | FLOAT | STRING | "(" (NAME ("," SP NAME)*)? ")"
  OPCODE    := a mnemonic of §4 with its dotted suffix (§4.8 fixes operand count, kinds and result placement)
  VER       := INT "." INT
```

Lproc groups have the types and roles of §3.7. A `token` suffix is
permitted only in the third group and a `classical` suffix only in the
fourth, each agreeing with that group's role; either suffix may be
omitted. The first and second groups carry no suffix, and the fifth
requires `in`. The `token` and `classical` suffixes are not permitted on
other kinds of function or on block parameters or `regs` entries. The additional
`lift.call` lists use the existing list operand grammar (§3.7).

### 16.3 Example objects (normative test vectors, suites A, C, F and G)

**Hashes.** Every hash in this section is written `<to be computed by
the 0.9 precheck: example k, name>`, one distinct placeholder per
example and symbol: the 0.9 precheck (`spec/README.md`) computes and
patches them before the draft is graded. Examples 1 and 2 are the 0.8
objects with the version string changed and, for example 2's lproc,
the three empty signature arrays required by Q14; their 0.8 hashes
remain the 0.8 vectors.

**Example 1** (`inc8`): one oracle, no lprocs, no memory. The function
has **six** canonical lines.

```
bennett-object isa=0.9 object=0.9 launch=0.9 sched=0.9 cert=0.9 coverage=0.9 target=0.9 ptr_width=32

section symbols
{"0":{"interface_hash":"<to be computed by the 0.9 precheck: example 1, inc8>","kind":"oracle","name":"inc8","signature":{"abi":"0.9","args":[],"fault":4,"in":[8],"kind":"oracle","mem":[],"tgt":8,"tier":"q"}}}

section functions
oracle @inc8 (x:wire[8] in) (t:wire[8]) (f:wire[4]) () ()
regs { one:reg[8], r:reg[8] }
^entry():
one = lift.const.i8 1
r = lift.add x one
lift.retval r
```

The object has no `layout` section, so the oracle's closure set is
empty and its envelope is exactly these bytes (each line LF-terminated):

```
isa=0.9
ptr_width=32
{"abi":"0.9","args":[],"fault":4,"in":[8],"kind":"oracle","mem":[],"tgt":8,"tier":"q"}
oracle @inc8 (x:wire[8] in) (t:wire[8]) (f:wire[4]) () ()
regs { one:reg[8], r:reg[8] }
^entry():
one = lift.const.i8 1
r = lift.add x one
lift.retval r
```

With B ≥ 3 every x yields `(x + 1 mod 256, 0)` (`result_val` = 0 ⊕ r);
with B = 2 every x yields `(0, FAULT_BUDGET_STEPS)`. The three rows have
indices 0, 1, 2, `HALT` = 3 and P = 2 (§3.6); `inc8` is acyclic, so its
hash is SHA-256 of these envelope bytes (§9.2).

**Example 2** (`sum_db`): an oracle reading an interface region through
lifted memory with a recursive lproc that receives the base as a
parameter, and a global accessed by address.

```
bennett-object isa=0.9 object=0.9 launch=0.9 sched=0.9 cert=0.9 coverage=0.9 target=0.9 ptr_width=32

section symbols
{"0":{"interface_hash":"<to be computed by the 0.9 precheck: example 2, sum_db>","kind":"oracle","name":"sum_db","signature":{"abi":"0.9","args":[],"fault":4,"in":[8],"kind":"oracle","mem":[{"A":4,"W":8,"role":"in"}],"tgt":8,"tier":"q"}},"1":{"interface_hash":"<to be computed by the 0.9 precheck: example 2, sum>","kind":"lproc","name":"sum","signature":{"abi":"0.9","args":[],"kind":"lproc","mem":[],"params":[8,8,32],"result":8,"tier":"q","tokens":[]}},"2":{"interface_hash":"<to be computed by the 0.9 precheck: example 2, zero>","kind":"data","name":"zero","signature":{"abi":"0.9","kind":"data","size":1,"tier":"-"}}}

section functions
oracle @sum_db (n:wire[8] in) (t:wire[8]) (f:wire[4]) (db:region[4,8] in) ()
regs { base:reg[32], acc:reg[8], r:reg[8] }
^entry():
base = lift.addr @db
acc = lift.const.i8 0
r = lift.call @sum (n, acc, base)
lift.retval r
lproc @sum (n:reg[8], acc:reg[8], base:reg[32]) (r:reg[8]) () () ()
regs { zp:reg[32], zv:reg[8], z:reg[1], one:reg[8], n2:reg[8], idx:reg[32], p:reg[32], v:reg[8], acc2:reg[8] }
^entry():
zp = lift.addr @zero
lift.boundscheck zp 1
zv = lift.load.i8 zp
z = lift.icmp.eq n zv
qbranch z ^done ^more
^more():
one = lift.const.i8 1
n2 = lift.sub n one
idx = lift.zext.i32 n2
p = lift.add base idx
lift.boundscheck p 1
v = lift.load.i8 p
acc2 = lift.add acc v
r = lift.call @sum (n2, acc2, base)
lift.ret r
^done():
lift.ret acc

section data
zero 00

section layout
{"A_tab":4,"H":13,"S_cells":9,"globals":{"zero":{"base":16,"image":"zero","size":1}},"heap_base":4112,"heap_limit_max":4080,"interface_regions":{"sum_db":{"db":{"base":4096}}},"null_size":16,"ptr_width":32}
```

Ranges: null [0,16), `zero` [16,17), `db` [4096,4112), heap [4112,8192)
within 2^13: disjoint; two pre-entries ≤ 2^4. `alloctab` starts with
pinned entries (16,1) and (4096,16), `atop = 2`. `@sum` resolves `n acc
base` to its params and everything else to its own locations; its
locations are `n acc base` (params), `r` (result name), then its nine
`regs`: m′ = 13, **R_sum = 16 cells**; `@sum_db` reads `n` as a pseudo-
location. Input n performs n + 1 nested calls (the last at depth n + 1)
and, the stack being monotonic, occupies 16(n + 1) stack cells. With
sufficient budget F_B(n) = (Σ_{i<n} db[i] mod 256, 0) for n ≤ 16 (n = 16
needs 272 cells, hence S_cells = 9 and `stack_limit ≥ 272`); for n > 16
the check at `p` faults `FAULT_UB_MEMFAULT` on the first out-of-range
byte; with small B, `FAULT_BUDGET_STEPS`; with `depth_limit ≤ n` or
`stack_limit < 16(n + 1)`, `FAULT_BUDGET_DEPTH`. Hashes by §9.2: `@zero`
is a data symbol and the `zero` global's image; `@sum` is a one-member
SCC (its self-reference is written `@$0`, its `@zero` reference is
replaced by `@` + the `zero` hash, which is its only part-(6) line; its
closure set is the `zero` global's leaf and image), so its envelope is
exactly the bytes below (the two `zero` placeholders stand for the
64-hex `zero` hash), its `scc_hash` is their SHA-256 and its hash
is SHA-256 of the ASCII `<scc_hash hex>:0`:

```
isa=0.9
ptr_width=32
{"abi":"0.9","args":[],"kind":"lproc","mem":[],"params":[8,8,32],"result":8,"tier":"q","tokens":[]}
lproc @sum (n:reg[8], acc:reg[8], base:reg[32]) (r:reg[8]) () () ()
regs { zp:reg[32], zv:reg[8], z:reg[1], one:reg[8], n2:reg[8], idx:reg[32], p:reg[32], v:reg[8], acc2:reg[8] }
^entry():
zp = lift.addr @<to be computed by the 0.9 precheck: example 2, zero>
lift.boundscheck zp 1
zv = lift.load.i8 zp
z = lift.icmp.eq n zv
qbranch z ^done ^more
^more():
one = lift.const.i8 1
n2 = lift.sub n one
idx = lift.zext.i32 n2
p = lift.add base idx
lift.boundscheck p 1
v = lift.load.i8 p
acc2 = lift.add acc v
r = lift.call @$0 (n2, acc2, base)
lift.ret r
^done():
lift.ret acc
data zero 00
layout ["globals","zero"] {"base":16,"image":"zero","size":1}
<to be computed by the 0.9 precheck: example 2, zero>
```

The displayed `sum` envelope renders its global leaf path as
`["globals","zero"]`, preserving the original leaf value.
[deps: ISA-9.1-P1, ISA-16.1]
`@sum_db`'s envelope includes every layout leaf, the `zero` image, `@` +
the `sum` hash in place of `@sum`, and the `sum` hash as its part-(6)
line (`@db` is an interface region, not a symbol):

```
inc8 = <to be computed by the 0.9 precheck: example 1, inc8>
zero = <to be computed by the 0.9 precheck: example 2, zero>
sum = <to be computed by the 0.9 precheck: example 2, sum>
sum_db = <to be computed by the 0.9 precheck: example 2, sum_db>
```

**Example 3** (`borrow_demo`): a permutation action with a loop bound
and a borrow frame established by the translation lemma (§5.6).

```
bennett-object isa=0.9 object=0.9 launch=0.9 sched=0.9 cert=0.9 coverage=0.9 target=0.9 ptr_width=32

section symbols
{"0":{"interface_hash":"<to be computed by the 0.9 precheck: example 3, borrow_demo>","kind":"perm_action","name":"borrow_demo","signature":{"abi":"0.9","args":[],"bounds":[1],"in":[1],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}}}

section functions
perm_action @borrow_demo (t:wire) (x:wire in) () () ()
regs {  } bounds {1}
^entry():
b = qborrow any 1
qloop 1 {
cx x b[0]
cx b[0] t
cx x b[0]
cx b[0] t
}
qreturn b
```

The bounds list is `[1]` (one `qloop`, trip count 1). The borrow frame is
the four `cx`: V = `cx x b[0]` (target in B = {b[0]}, control in X = {x},
X ∩ B = ∅), W = `cx b[0] t` (target t outside X, B and W's controls; one
control in B), V⁻¹ = V reversed, W again; no `swap`. On each of the 8
basis values of (t, x, b[0]) the action maps (t, x, b) ↦ (t ⊕ x, x, b):
V gives b ⊕ x, W gives t ⊕ b ⊕ x, V⁻¹ restores b, W gives t ⊕ x. Under an
inherited control w (16 configurations) it maps to (t ⊕ w·x, x, b); on a
borrowed wire entangled with a reference system it acts as that
permutation ⊗ I. The map is an involution, so the adjoint (the reverse
sequence `cx b[0] t; cx x b[0]; cx b[0] t; cx x b[0]`) realises the same
permutation; the adjoint's frame is not of the lemma's shape and is
covered by the `adjoint_table` rule (§12.1).

**Example 4** (`ones`, BLK-1): a register of run-time width, its id
stored to memory and reloaded, wires addressed by run-time slices.

```
bennett-object isa=0.9 object=0.9 launch=0.9 sched=0.9 cert=0.9 coverage=0.9 target=0.9 ptr_width=32

section symbols
{"0":{"interface_hash":"<to be computed by the 0.9 precheck: example 4, ones>","kind":"cc","name":"ones","signature":{"abi":"0.9","kind":"cc","params":["i64"],"results":["i3"],"tier":"cc"}},"1":{"interface_hash":"<to be computed by the 0.9 precheck: example 4, flip>","kind":"perm_action","name":"flip","signature":{"abi":"0.9","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}}}

section functions
fn cc @ones (n:i64) (m:i3) () () ()
^entry():
q = qreg.new any n
w = qreg.width q
id = qreg.id q
k = const.i32 8
p = alloc k
store.i64 p id
id2 = load.i64 p
q2 = qreg.from_id id2
zero = const.i64 0
one = const.i64 1
br ^test(zero)
^test(i:i64):
c = icmp.ult i w
br c ^body(i) ^done()
^body(j:i64):
s = qreg.slice q2 j 1
action_call @flip (s) () () ()
j2 = add j one
br ^test(j2)
^done():
m = measure.i3 q2
ret m
perm_action @flip (a:wire) () () () ()
^entry():
x a

section layout
{"A_tab":4,"H":12,"S_cells":0,"globals":{},"heap_base":16,"heap_limit_max":4080,"interface_regions":{},"null_size":16,"ptr_width":32}
```

Ranges: null [0, 16), heap [16, 4096), both within [0, 2^12); no globals. For n = 3:
`q` is a fresh register R (width 3, |000⟩); `p` = 16 (the bump pointer
starts at `heap_base`); `q2` is an alias of R; iterations i = 0, 1, 2
each slice [i, 1) (a `qreg[1]` view) and flip that wire; at i = 3 the
loop exits; `measure.i3` finds width 3 and outcome 0b111 = 7 with
probability 1. The run is `COMPLETED` with one `INT` result of width 3
(the byte `07`) and one measurement record (R, bits 1, 1, 1); it is not
reversible (a `measure` frame). For n = 2 the loop flips two wires and
`measure.i3` traps before measuring (width 2 ≠ 3): `FAILED`, one record
`FAULT_UB_TRAP`, no measurement, R live in |11⟩; that run *is*
reverse-eligible and reversal returns R to |00⟩ and releases it. For n =
0 the loop does not run and `measure.i3` traps likewise.

**Example 5** (`scratch`, BLK-4): quantum allocation inside an R
function, used and released, then reversed.

```
bennett-object isa=0.9 object=0.9 launch=0.9 sched=0.9 cert=0.9 coverage=0.9 target=0.9 ptr_width=32

section symbols
{"0":{"interface_hash":"<to be computed by the 0.9 precheck: example 5, main>","kind":"cc","name":"main","signature":{"abi":"0.9","kind":"cc","params":["i64"],"results":[],"tier":"cc"}},"1":{"interface_hash":"<to be computed by the 0.9 precheck: example 5, scratch>","kind":"r","name":"scratch","signature":{"abi":"0.9","kind":"r","params":["i64"],"results":[],"tier":"r"}},"2":{"interface_hash":"<to be computed by the 0.9 precheck: example 5, flip>","kind":"perm_action","name":"flip","signature":{"abi":"0.9","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}}}

section functions
fn cc @main (n:i64) () () () ()
^entry():
call @scratch n
ret
fn r @scratch (n:i64) () () () ()
^entry():
q = qreg.new any n
zero = const.i64 0
one = const.i64 1
br ^test(zero)
^test(i:i64):
c = icmp.ult i n
br c ^body(i) ^done()
^body(j:i64):
s = qreg.slice q j 1
action_call @flip (s) () () ()
t = qreg.slice q j 1
action_call @flip (t) () () ()
j2 = add j one
br ^test(j2)
^done():
qreg.free q
ret
perm_action @flip (a:wire) () () () ()
^entry():
x a
```

The object is memoryless. Forward, for every n the device can provide (else `qreg.new` records `FAULT_BUDGET_HEAP`): R creates register Q
(width n, |0…0⟩); iteration j flips wire j to |1⟩ and back to |0⟩; the
`qreg.free` finds Q clean (a verifying executor verifies it); the run is
`COMPLETED` with no values. Whole-run reversal re-creates Q clean with
its id, undoes iteration n − 1 (the second flip's adjoint makes wire n −
1 |1⟩, its slice frame restores Q live, the first flip's adjoint makes
it |0⟩), then every earlier iteration, and finally the `qreg.new` (Q is
clean: released); the run is `REVERSED` with the register ledger as at
entry, Q's id not reused. On hardware the free is unverified (§17 item
14) and `bennett_reverse` returns `E_NOT_REVERSIBLE`. Under 0.8 `@scratch`
is ill-formed (`qreg.*` was CC only).

**Example 6** (`adaptive`, BLK-2): a budget minted on the device from a
run-time value.

```
bennett-object isa=0.9 object=0.9 launch=0.9 sched=0.9 cert=0.9 coverage=0.9 target=0.9 ptr_width=32

section symbols
{"0":{"interface_hash":"<to be computed by the 0.9 precheck: example 6, adaptive>","kind":"cc","name":"adaptive","signature":{"abi":"0.9","kind":"cc","params":["i1","qreg[8]","qreg[8]","qreg[4]"],"results":[],"tier":"cc"}},"1":{"interface_hash":"<to be computed by the 0.9 precheck: example 6, overdraw>","kind":"cc","name":"overdraw","signature":{"abi":"0.9","kind":"cc","params":[],"results":[],"tier":"cc"}},"2":{"interface_hash":"<to be computed by the 0.9 precheck: example 6, inc8>","kind":"oracle","name":"inc8","signature":{"abi":"0.9","args":[],"fault":4,"in":[8],"kind":"oracle","mem":[],"tgt":8,"tier":"q"}}}

section functions
fn cc @adaptive (b:i1, x:qreg[8], t:qreg[8], f:qreg[4]) () () () ()
^entry():
two = const.i64 2
three = const.i64 3
zero = const.i64 0
s = select b three two
tk = budget.mint @inc8 s zero zero zero
oracle_call @inc8 tk (x) t f () ()
ret
fn cc @overdraw () () () () ()
^entry():
three = const.i64 3
one = const.i64 1
zero = const.i64 0
tk = budget.mint @inc8 three one zero zero
ret
oracle @inc8 (x:wire[8] in) (t:wire[8]) (f:wire[4]) () ()
regs { one:reg[8], r:reg[8] }
^entry():
one = lift.const.i8 1
r = lift.add x one
lift.retval r
```

`@inc8` is example 1's oracle. With b = 1 the token's tuple is (3, 0, 0,
0) and, on basis inputs, t ⊕= (x + 1) mod 256 with f unchanged; with b = 0
it is (2, 0, 0, 0): t unchanged, f ⊕= `FAULT_BUDGET_STEPS` (example 1's
results). Both runs are `COMPLETED` (each under the schedule bound to
its tuple if the object has one, else under the §11 defaults). Reversal
re-applies `@inc8` with the
recorded tuple (an involution), restoring t and f, then revokes the
token. `@overdraw` asks for heap_limit 1 in a memoryless object: the mint
fails `bennett_budget`'s check, the run is `FAILED` with one record
`FAULT_UB_TRAP`, and no token exists.

**Example 7** (`addya`, BLK-3): a parametrised in-place update x ← (x
+ y + a) mod 4 from a declared inverse pair.

```
bennett-object isa=0.9 object=0.9 launch=0.9 sched=0.9 cert=0.9 coverage=0.9 target=0.9 ptr_width=32

section symbols
{"0":{"interface_hash":"<to be computed by the 0.9 precheck: example 7, g>","kind":"oracle","name":"g","signature":{"abi":"0.9","args":["i2"],"fault":4,"in":[2,2],"kind":"oracle","mem":[],"tgt":2,"tier":"q"}},"1":{"interface_hash":"<to be computed by the 0.9 precheck: example 7, ginv>","kind":"oracle","name":"ginv","signature":{"abi":"0.9","args":["i2"],"fault":4,"in":[2,2],"kind":"oracle","mem":[],"tgt":2,"tier":"q"}},"2":{"interface_hash":"<to be computed by the 0.9 precheck: example 7, addya>","kind":"perm_action","name":"addya","signature":{"abi":"0.9","args":["token","i2"],"bounds":[],"in":[2],"kind":"perm_action","mem":[],"mut":[2],"tier":"q"}},"3":{"interface_hash":"<to be computed by the 0.9 precheck: example 7, run>","kind":"cc","name":"run","signature":{"abi":"0.9","kind":"cc","params":["qreg[2]","qreg[2]","token","i2"],"results":[],"tier":"cc"}}}

section functions
oracle @g (x:wire[2] in, y:wire[2] in) (t:wire[2]) (f:wire[4]) () (a:i2)
regs { s:reg[2], r:reg[2] }
^entry():
s = lift.add x y
r = lift.add s a
lift.retval r
oracle @ginv (x:wire[2] in, y:wire[2] in) (t:wire[2]) (f:wire[4]) () (a:i2)
regs { s:reg[2], r:reg[2] }
^entry():
s = lift.sub x y
r = lift.sub s a
lift.retval r
perm_action @addya (x:wire[2]) (y:wire[2] in) () (tk:token, a:i2) ()
^entry():
qinplace @g @ginv tk x (y) () (a)
fn cc @run (x:qreg[2], y:qreg[2], tk:token, a:i2) () () () ()
^entry():
action_call @addya (x) (y) () (tk, a)
ret
```

Each oracle has three rows, so F_B = ((x + y + a) mod 4, 0), resp. ((x − y
− a) mod 4, 0), for B ≥ 3 and (0, `FAULT_BUDGET_STEPS`) for B ≤ 2; for each
(y, a) the two maps are mutually inverse bijections of [0, 4). The object
is completed for execution by a certificate (generated with the 0.9
precheck, which first computes `object_hash`) whose `@addya` region
record carries `verification.inverse_pairs` = `[{"budget":[3,0,0,0],"claim":0,"g":"<closure hash of g>","ginv":"<closure hash of ginv>"}]`
and claim 0 = `{"domain":{"args":[{"hi":3,"lo":0,"name":"a","width":2}],"borrowed":[],"bundles":[{"hi":3,"lo":0,"name":"x","width":2},{"hi":3,"lo":0,"name":"y","width":2}],"regions":[]},"expected":"inverse_pair","kind":"exhaustive"}`
(64 elements). With it and a token whose tuple is ≥ (3, 0, 0, 0), `@run`
maps every basis (x, y) and a ∈ [0, 4) to x′ = (x + y + a) mod 4, y
unchanged, scratch clean: for x = 3, y = 2, a = 1, `@g` gives t = 2, the
exchange gives x = 2, t = 3, and `@ginv`(2; 2, 1) = 3 clears t. The adjoint
maps x to (x − y − a) mod 4; under a control 0 nothing changes. Without the
certificate, or with a token of tuple (2, 0, 0, 0), `@run` traps
`FAULT_UB_TRAP` before any effect.

**Example 8** (`fails`, ISA-7/ISA-8): an R fault after effects, and the
reversal of the failed run.

```
bennett-object isa=0.9 object=0.9 launch=0.9 sched=0.9 cert=0.9 coverage=0.9 target=0.9 ptr_width=32

section symbols
{"0":{"interface_hash":"<to be computed by the 0.9 precheck: example 8, main>","kind":"cc","name":"main","signature":{"abi":"0.9","kind":"cc","params":[],"results":[],"tier":"cc"}},"1":{"interface_hash":"<to be computed by the 0.9 precheck: example 8, work>","kind":"r","name":"work","signature":{"abi":"0.9","kind":"r","params":["ptr"],"results":[],"tier":"r"}}}

section functions
fn cc @main () () () () ()
^entry():
n = const.i32 8
p = alloc n
call @work p
ret
fn r @work (p:ptr) () () () ()
^entry():
v = const.i64 7
store.i64 p v
effect clock.read p
fault 21

section devices
{"models":{"clock":{"version":"0.9"}},"services":{}}

section layout
{"A_tab":4,"H":12,"S_cells":0,"globals":{},"heap_base":16,"heap_limit_max":4080,"interface_regions":{},"null_size":16,"ptr_width":32}
```

`p` = 16; the store writes `07 00 00 00 00 00 00 00` at [16, 24); the clock
read writes c = 0 there (`00` × 8) and sets c = 1; `fault 21` ends the run:
`FAILED`, `bennett_result_nfaults` = 1, `bennett_result_fault` = 21, no
values, `@main`'s `ret` not executed, [16, 24) = `00` × 8, c = 1, tape
length 1. `bennett_reverse` (0.8: `E_NOT_REVERSIBLE`) removes the fault
record, undoes the clock read ([16, 24) = `07` then `00` × 7, c = 0, tape empty),
the store ([16, 24) = `00` × 8), the open activation of `@work` and the
call, and the allocation: the run is `REVERSED` with memory, allocation
table, clock and tape at their entry values.

**Example 9** (`zero_cause`, effective cause): a literal fault code 0.

```
bennett-object isa=0.9 object=0.9 launch=0.9 sched=0.9 cert=0.9 coverage=0.9 target=0.9 ptr_width=32

section symbols
{"0":{"interface_hash":"<to be computed by the 0.9 precheck: example 9, zero_cause>","kind":"oracle","name":"zero_cause","signature":{"abi":"0.9","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}},"1":{"interface_hash":"<to be computed by the 0.9 precheck: example 9, zero_cc>","kind":"cc","name":"zero_cc","signature":{"abi":"0.9","kind":"cc","params":[],"results":[],"tier":"cc"}}}

section functions
oracle @zero_cause (x:wire in) (t:wire) (f:wire[4]) () ()
^entry():
lift.faultif x 0
lift.retval x
fn cc @zero_cc () () () () ()
^entry():
fault 0
```

Rows: `lift.faultif` (index 0), `lift.retval` (1); `HALT` = 2, P = 2. With
B = 0 every x yields (0, `FAULT_BUDGET_STEPS`). With B = 1, x = 0 yields (0,
`FAULT_BUDGET_STEPS`) (not halted) and x = 1 yields (0, `FAULT_UB_TRAP`).
With B ≥ 2, x = 0 yields (0, 0) (success, payload 0) and x = 1 yields (0,
`FAULT_UB_TRAP`); under 0.8, x = 1 yielded (0, 0) for every B ≥ 1,
indistinguishable from success. A run of `@zero_cc` is `FAILED` with one
record `FAULT_UB_TRAP`.

**Example 10** (`through`, lproc token/classical parameters): the
oracle forwards its entry token, an `i2` argument and a mapped byte
region to an lproc. The lproc makes both a parametrised oracle call and
a parametrised permutation-action call, passing its token as `tk` and
also as a token-typed `args` operand. All token uses preserve identity.

```
bennett-object isa=0.9 object=0.9 launch=0.9 sched=0.9 cert=0.9 coverage=0.9 target=0.9 ptr_width=32

section symbols
{"0":{"interface_hash":"<to be computed by the 0.9 precheck: example 10, through>","kind":"oracle","name":"through","signature":{"abi":"0.9","args":["token","i2"],"fault":4,"in":[2],"kind":"oracle","mem":[{"A":0,"W":8,"role":"in"}],"tgt":2,"tier":"q"}},"1":{"interface_hash":"<to be computed by the 0.9 precheck: example 10, worker>","kind":"lproc","name":"worker","signature":{"abi":"0.9","args":["i2"],"kind":"lproc","mem":[{"A":0,"W":8,"role":"in"}],"params":[2],"result":2,"tier":"q","tokens":["token"]}},"2":{"interface_hash":"<to be computed by the 0.9 precheck: example 10, plus_db>","kind":"oracle","name":"plus_db","signature":{"abi":"0.9","args":["i2"],"fault":4,"in":[2],"kind":"oracle","mem":[{"A":0,"W":8,"role":"in"}],"tgt":2,"tier":"q"}},"3":{"interface_hash":"<to be computed by the 0.9 precheck: example 10, flip_arg>","kind":"perm_action","name":"flip_arg","signature":{"abi":"0.9","args":["token","i2"],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[2],"tier":"q"}}}

section functions
oracle @through (x:wire[2] in) (t:wire[2]) (f:wire[4]) (db:region[0,8] in) (tk:token, a:i2)
regs { r:reg[2] }
^entry():
r = lift.call @worker (x) (tk) (a) (db)
lift.retval r
lproc @worker (x:reg[2]) (r:reg[2]) (tk:token token) (a:i2 classical) (db:region[0,8] in)
regs { fr:reg[4], zero:reg[4], bad:reg[1] }
^entry():
lift.oracle_call @plus_db tk (x) r fr (db) (a)
zero = lift.const.i4 0
bad = lift.icmp.ne fr zero
lift.faultif bad 9
lift.perm_call @flip_arg (r) () () (tk, a)
lift.ret r
oracle @plus_db (x:wire[2] in) (t:wire[2]) (f:wire[4]) (db:region[0,8] in) (a:i2)
regs { base:reg[32], byte:reg[8], low:reg[2], r:reg[2] }
^entry():
base = lift.addr @db
byte = lift.load.i8 base
low = lift.trunc.i2 byte
r = lift.add x a
r = lift.add r low
lift.retval r
perm_action @flip_arg (x:wire[2]) () () (tk:token, a:i2) ()
^entry():
x x[0]

section layout
{"A_tab":2,"H":6,"S_cells":4,"globals":{},"heap_base":32,"heap_limit_max":32,"interface_regions":{"plus_db":{"db":{"base":17}},"through":{"db":{"base":16}}},"null_size":16,"ptr_width":32}
```

The outer invocation's B is independent of `tk`'s tuple. The sole
`@worker` binding key is `[0,1,0]` (oracle argument positions 0 and 1,
region position 0). It has five locations (`x r fr zero bad`), so
R_worker = 8 cells; forwarding the three additional parameters adds no
cells. The two outer rows precede its six rows: indices 0, 1 and 2..7,
HALT = 8, P = 4. `@plus_db` has six rows and no lproc calls. The byte
region maps at 16 in the outer oracle and 17 in the nested oracle;
both mappings refer to the same passed cells, with no byte copy.

For every x, a ∈ [0,4) and byte d ∈ [0,256), an outer tuple
(B ≥ 8, 0, stack_limit ≥ 8, depth_limit ≥ 1) and `tk` tuple (6,0,0,0)
give F_B(x,d) = (((x + a + (d mod 4)) mod 4) XOR 1, 0).
For arbitrary initial target t and fault f, the final values are
(t XOR payload, f); x and d are unchanged and all workspace is clean.
This permutation extends by linearity to superpositions and an
entangled reference. With outer B = 7 the success path has not executed
`lift.retval`, so the outcome is (0, `FAULT_BUDGET_STEPS`); with outer
stack_limit = 7 or depth_limit = 0 the first row gives
(0, `FAULT_BUDGET_DEPTH`). With `tk` tuple (5,0,0,0), the nested oracle
returns fault 1; the lproc's `lift.faultif` propagates code 9 at outer
step 5, so B ≥ 5 gives (0, `FAULT_CALLEE`). The nested fault, stack
history, parameters and locations are cleared by prefix reversal; no
additional charge arises from forwarding, commit or reversal.

## 17. Decisions deferred to implementation, with defaults

Each item is one an implementer must decide to build; any decision is
conformant provided two implementations agree on the default below until
the item is revised. Items 1–5 are asserted non-semantic.

1. **Binary encoding.** Default: none; §16 text is the only interchange
   form.
2. **Gate-name registry.** Default: the names of §4.4 only; an unknown
   gate name is a load error.
3. **Further device models.** Default: the four of §7.3; an `effect` on
   any other device is `FAULT_EFFECT_UNMODELLED`.
4. **Mechanisation of §14.** Default: none; the reference interpreter is
   the oracle.
5. **Memory layout of the opaque C objects.** Default: opaque; only the
   listed functions touch them.
6. **Measurement-based cleanup** (§6.4): the normative fixup, its
   controlled variant and its certificate kind. Default: unitary cleanup
   (`reverse`); `capabilities.mbu` is `false`.
7. **Expansion library** (§11): the gate-level expansion below row
   level. Default: the library `ref` published by `bennett-isa-spec`
   (ripple-carry adders and comparators, linear `inrange` scans,
   one-hot address dispatch, clean scratch released in reverse order);
   a schedule that names no `expansion` binds `ref` at the ISA version
   in the object's header, with the full-library envelope hash published
   for that version in §11.1. [deps: ISA-9.1, ISA-11.1-P3, ISA-11.1-P4]
   For a 0.9 object this is exactly
   `{"name":"ref","version":"0.9","hash":"fcb2f1e6d61aced0bf9689eab5f6d82c4ae42e9fea8d1b21ae9c7ffccda6f5ce"}`,
   equally for omission and an explicit binding. [deps: ISA-11.1-P3, ISA-11.1-P4, ISA-11.1]
   §11.1 fixes its schema, ordinals, scratch identities, empty removable
   sets and deterministic decomposition algorithms. [deps: ISA-11.1]
   Serialization as `spec/ref-0.9-draft.json` is a separate job under
   b-isa-spec-8r4; publication in `ref/` follows 0.9 acceptance. The
   worked controlled-action target/schedule/certificate fixture bound to
   it (zsw.4) is pending;
   until then two consumers agree on every order fixed above gate level
   (§11).
8. **Enclosure procedure for transcendental entries** (§12.1). Default:
   Taylor series of sin and cos to the first term whose Lagrange
   remainder bound is below 2^−(`precision_bits` + 2), argument reduced
   modulo 2π with an outward-rounded enclosure of π.
9. **Default `epsilon_share`** (§8.1, §11) when a schedule assigns none.
   Default: the exact rational `bindings.epsilon / N`, where N is the
   number of gate operations in the bound-library expansion of one
   invocation of `bindings.region`, including every nested invocation,
   that have no exact native match as defined below. [deps: ISA-8.1, ISA-8.3, ISA-11-P2]
   For this count only, an exact native match is a `gates` entry whose
   opcode, classes, control range and parameters cover the operation
   and whose `epsilon` expression is independent of `eps` and evaluates
   to zero; compute this test before the requested-accuracy filter,
   without selecting an achieved-error entry using the default share. [deps: ISA-8.1, ISA-8.3]
   N counts the gate operations before target scheme expansion and
   includes operations with explicit shares as well as omitted shares;
   the root accuracy is `bindings.epsilon`, not a nested region's
   separately requested accuracy. [deps: ISA-11-P2, ISA-11-P3, ISA-8.2, ISA-8.3]
   Gates with an exact native match take a default share of zero; if
   N=0 all default shares are zero and no division is performed. [deps: ISA-8.1, ISA-8.3]
   The resolved share is represented exactly as a reduced pair (p,q)
   of nonnegative numerator and positive denominator, with zero
   represented by (0,1); it is passed as that rational value to every
   §8.1 comparison, expression and accuracy bucket, without rounding
   to a finite decimal. [deps: ISA-8.1]
   This is a resolved value, not an inserted `epsilon_share` field;
   existing decimal schedule fields and the hashed schedule bytes are
   unchanged by default resolution. [deps: ISA-11-P3, ISA-9.2]
   Explicit shares still supply `eps` by their exact value and the
   §8.1 requested-versus-achieved filter and achieved-error charge
   still apply to every resolved share. [deps: ISA-8.1, ISA-8.3]
   *(non-normative witness)* At `bindings.epsilon: "0.1"`, three gates
   lacking exact native matches give N=3 and default (1,30), even if
   one gate has an explicit share; a fourth, exact-native gate takes
   zero and is excluded from N. [deps: ISA-8.1, ISA-8.3, ISA-11-P2]
   With no such inexact gates N=0. [deps: ISA-8.1, ISA-8.3, ISA-11-P2]
   Omitting expansion on a 0.9 object selects the full-library hash in
   item 7, not the single-qxor witness hash. [deps: ISA-11.1-P3, ISA-11.1-P4]
10. **`any`-class resolution** (§11). Default: the first class in JCS
    order with capacity.
11. **Occupancy rate of a class absent from `rate`** (§8.2). Default: 1.
12. **Classical step timing of CC/R instructions and effects** (§8.2).
    Default: `classical_step_time` each, sequential.
13. **Suite sizes** (§15): the exhaustive widths of suite C and the
    sample counts of `sampled` claims. Default: exhaustive for ≤ 16
    interface bits; 2^16 samples above.
14. **Hardware handling of `qreg.free`** (§3.3). Cleanliness of an
    arbitrary state is not testable nondestructively. Default: on
    hardware the release proceeds without a test and its frame is
    marked unverified, so a segment containing it cannot be uncalled
    (§7.4) and a run containing it is not reverse-eligible (§10);
    releasing a register that was not clean is a program error the
    hardware does not detect *(non-normative: physically its wires are
    then discarded as by an unrecorded measurement; this is a program
    error, not a trace-out operation, §10)*. The same holds for
    `bennett_qreg_prepare` and `bennett_qreg_free` on a non-verifying
    executor. A verifying executor tests and records `FAULT_UB_TRAP` on
    failure (§4.9), or returns `E_QREG_NOT_CLEAN` from the host calls.
15. **Routing and movement cost under a topology** (§8.2–§8.3, §11),
    deferred to 0.10. Default: no routing cost is charged and no routing
    is inserted; a schedule that needs a non-adjacent pair is
    `E_SCHEDULE_INFEASIBLE` naming this item. Absent topology remains
    all-to-all. A producer can supply an explicitly routed gate sequence
    using §4.4 `swap` gates; those gates execute and are charged normally.
    The optional topology schema and adjacency validation are decided in
    0.9; only automatic routing and its movement-cost model are deferred.
16. **Exhaustion of an executor's own classical resources in CC/R**
    (activation stack, history storage; §4.9). CC and R carry no budget
    (§5.4 budgets are Q-tier), so where a finite executor runs out is
    its own limit. Default: a terminal fault record `FAULT_BUDGET_HEAP`
    at the instruction that cannot proceed, raised before any mutation
    by that instruction (§4.9; an executor that cannot guarantee this
    for an exhaustion inside a region call marks the run not
    reverse-eligible); the point is executor-defined and the suites do
    not test it.
17. **Identity-idle step** (§3.6). A variant of the step in which an
    idle step leaves its frame at zero: forward, a = (pc ≠ HALT) into
    clean scratch; dispatch k < n only; rows as printed; the successor
    XORs for k < n and both commit exchanges under a, with no HALT
    successor; then `old_pc ^= HALT` under a (an active frame holds old
    index ⊕ HALT, never 0, since the old index is < n = HALT);
    dispatch uncomputed from old_pc ⊕ HALT under a; a cleared from
    (old_pc ≠ 0). Inverse: a = (old_pc ≠ 0); dispatch from old_pc ⊕
    HALT under a; `old_pc ^= HALT` under a; the two inverse exchanges
    under a; successors and rows undone as printed; dispatch uncomputed
    from pc under a; a cleared from (pc ≠ HALT). Same frame width, same
    F_B and the same postcondition as T2; on pc = HALT with a fresh zero
    frame a forward step is the identity (candidate theorem T7).
    Default: the printed step, whose idle frames record HALT. Hook: an
    expansion library (§11) MAY name this variant as its realisation of
    §3.6, and a schedule binding that library executes and charges it;
    making it the reference changes §3.6 and adds T7.
18. **Checkpointed or pebbled oracle realisations** (§5.2, §5.7, §9.4,
    §11). Default: the full-history reference expansion (B frames, the
    library `ref`). Hook: an expansion library MAY realise §5.2 by
    checkpoints and replay or by pebbling; it MUST list every generated
    allocation, copy, forward evaluation, inverse evaluation and release,
    every repeated evaluation included, with distinct ordinals and
    scratch identities, and carry a full-domain argument that the final
    interface tuple of §5.2 is produced with private scratch clean and
    borrowed correlations restored; a schedule binds its name, version
    and hash; every replay is executed and charged (§8.2); the frame
    descriptor reports the realised history (§9.4); the reference body,
    F_B and the budget unit are unchanged, and T2 remains stated for the
    reference expansion. *(non-normative)* Binary pebbling P_{j+1}(a, b) =
    P_j(a, c); P_j(c, b); P_j(a, c) uses 3^⌈log₂ B⌉ base evaluations and at
    most ⌈log₂ B⌉ intermediate configurations; no polylogarithmic-time
    bound is claimed.

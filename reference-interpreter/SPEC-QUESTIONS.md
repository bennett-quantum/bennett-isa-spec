# Implementation readings of ISA 0.8

These entries do not narrow the supported ISA, and the accepted specification
is unchanged. Each entry gives the question, the section, the reading the crate
implements, and a proposed 0.9 wording. Entries marked *revised (round 1)*
changed in response to the council's round-1 review
(`bennett/docs/council/isa-ref-2026-10-29/round-1/`); the editor's response is
`council/response-round-1.md`.

1. **Indices: blocks or rows? (§3.6, §4.2, §16.3).** The index-space paragraph
   calls its elements blocks, but every instruction is a budget step with k+1
   fallthrough, calls record k+1, and the published inc8 takes three steps in
   one block. Reading: one index per instruction, concatenating blocks in text
   order; a label denotes its block's first instruction; a function's range
   contains all its instruction indices. *Proposed 0.9:* "`I[0..n)` = the rows
   of the oracle's blocks in text order, each block contiguous, …; a label
   denotes the index of its block's first row."
2. **Fixture comments versus canonical text (§16.1–§16.2).** The interchange
   grammar has no comment syntax. Reading: hand-derived comments live in
   adjacent `.comments` files; the `.bisa` files are canonical objects. No
   specification change proposed.
3. **Idle steps (§3.6, budget design note I5).** Reading: the printed HALT
   successor XOR and the two exchanges run, including `old_pc = HALT` in an idle
   frame; prefix reversal clears it. The design note's done-invariance is not
   0.8. No 0.8 change proposed.
4. **Counter widths (§3.7, §9.1)** *revised (round 1)*. The layout supplies size
   exponents but no Hc/Ac/Sc/Dc fields. Reading: the step machine allocates the
   minimum legal widths, Hc = H + 2, Ac = A_tab + 1, Sc = S_cells + 1,
   Dc = max(1, ⌈log₂(depth_limit + 2)⌉), cells of W_cell = max(P, Sc, Did, W_max)
   bits and slot j of the widest value any row writes into it
   (`DerivedWidths`, `src/machine.rs`); the §14.1 interpreter keeps unbounded
   counters. Every prospective sum is checked before any counter changes, so no
   counter ever exceeds its width (an excess is an internal error). Round 0's
   entry claimed derived widths that the u128 machine did not have.
   *Proposed 0.9:* "Executors MAY choose wider counters; the reference machine
   uses the minimum widths."
5. **Reported step count on budget exhaustion (§14.1, Python seed).** Reading:
   count executed rows, at most B. No change proposed.
6. **Unknown device identity (§9.2, §17 item 3).** Reading: an unmodelled effect
   is legal text with fault outcome 8; its operation token is in the canonical
   body; it contributes no device-envelope entry. A known device used without a
   `devices.models` declaration, a known device with an unknown operation, and a
   known operation with the wrong operand count or types are load errors
   (§7.3 table; *revised (round 1)*, muse B3). *Proposed 0.9:* "Every `effect`
   on a §7.3 device is checked against the table at load."
7. **Ordering multi-member lproc SCCs (§3.6, §9.2).** Member closure hashes are
   unrelated SHA-256 values and cannot order members by SCC index. Reading: a
   multi-member SCC sorts as a group at its `scc_hash`, members in SCC index
   order; a singleton sorts at its closure hash. *Proposed 0.9:* "lprocs are
   ordered by closure hash; the members of a multi-member SCC are placed
   together at the position of their `scc_hash` and follow in SCC index order."
8. **Host reply length wider than ptr_width (§10).** Reading: an unrepresentable
   full reply length is `E_ARG` before any memory write. *Proposed 0.9:* "A reply
   whose length does not fit ptr_width is `E_ARG`."
9. **Empty data images (§16.1 versus §16.2).** Reading: exactly `NAME SP LF` for
   a zero-sized data image, the sole trailing-space exception. *Proposed 0.9:*
   "`dataline := NAME SP HEX NL` (a zero-byte image is written `NAME SP NL`)."
10. **Fault width in shared lprocs (§3.3, §3.7).** Reading: an lproc's fault
    literals are checked against every enclosing oracle's F; an unattached lproc
    is rechecked when linked. *Proposed 0.9:* "`fault C` in an lproc requires
    C < 2^F for every oracle whose closure contains it."
11. **Metadata encodings the schema sketches omit (§9.2, §9.4, §12–13)**
    *revised (round 1)*. A program-point tuple is
    `[interface_hash, block_name, ordinal]`; schedule paths keep §11's
    `{symbol,point,iteration}`; certificate `versions` uses the seven object
    header names; a frame uses `interface` (role/width/class), class maps
    `scratch` (optionally `clean`/`dirty`), `borrow`, `scheme_ancillae`, and
    `retained_history` (an expression in c), with optional `live_ranges`
    (`{frame,start,end}`, disjoint per frame). Coverage `trapped` requires its
    `fault_code`, `shimmed` its `shim_hash`. A **variant** is any nonempty
    string without `:` (keys are `<variant>:<inherited_controls>` and
    `<hash>:<variant>:<inherited_controls>`); the specification enumerates no
    variant names, so `"banana"` is a legal variant (codex B16 declined in part).
    *Proposed 0.9:* list the variant names (for example `fwd`, `adj`) or state
    that variants are producer-chosen identifiers without `:`.
12. **Constant literals (§3.3, §4.2, §16.1)** *revised (round 1)*. INT has no
    magnitude bound, but a literal outside the destination's range would be
    silently reduced. Reading: `const.iN c` and `lift.const.iN c` require
    −2^(N−1) ≤ c < 2^N (a signed or unsigned N-bit spelling); anything else is
    rejected (`§3.3`). Round 0 reduced modulo 2^N; the council (fable R1-B2)
    showed that `lift.const.i8 257` silently became 1. Negative and positive
    spellings of one bit pattern (−1 and 255) remain two texts. *Proposed 0.9:*
    "c MUST satisfy −2^(N−1) ≤ c < 2^N" (or, for one canonical spelling,
    "0 ≤ c < 2^N").
13. **Canonical FLOAT rendering (§16.1).** Reading: the shortest round-trip
    decimal in the RFC 8785 number rendering, with `.0` appended to an integral
    result and `-0.0` for negative zero. *Proposed 0.9:* name RFC 8785 §3.2.2.3
    as the FLOAT rendering.
14. **Upper bound of ptr_width (§3.3, §4.8)** *new (round 1)*. The header INT is
    unbounded, but `hostcall`'s result is "iN (rlen, width ptr_width)" (§4.8) and
    iN exists only for 1 ≤ N ≤ 128 (§3.3). Reading: 1 ≤ ptr_width ≤ 128; the
    layout exponents H ≤ ptr_width, A_tab and S_cells are otherwise unbounded,
    and all address arithmetic is exact up to 2^128 (H = 128 included).
    *Proposed 0.9:* "`ptr_width` ∈ [1, 128]."
15. **Width suffixes of lifted rows (§3.3, §4.8)** *new (round 1)*. `lift.const.iN`
    and `lift.{zext,sext,trunc}.iM` write `reg[N]` locations, which §3.3 does not
    bound, while the suffix is spelled like an iN. Reading: a lifted suffix names
    the destination register width and is any positive N (`lift.const.i300`);
    CC/R suffixes are iN types (≤ 128); `lift.load.iN`/`lift.store.iN` keep the
    explicit N ∈ {8, …, 128}. *Proposed 0.9:* "In lifted rows the suffix `.iN`
    denotes the width N of the `reg[N]` destination; N is unbounded."
16. **`lift.free` with several matching entries (§4.2, §7.1)** *new (round 1)*.
    The §4.2 row flips `live_i ^= match_i` for every i < atop with base_i = p;
    §7.1 marks "the unique live entry with base p" dead. Several entries can
    share a base only through size-0 ranges (a size-0 global at the heap base).
    Reading: the lifted engines follow the row literally (both engines agree);
    CC/R `free` follows §7.1. *Proposed 0.9:* "match_i = (i < atop) ∧ (base_i = p)
    ∧ live_i ∧ ¬pinned_i", or forbid size-0 globals.
17. **Metadata integer spelling (§9.1)** *new (round 1)*. "Integers that may exceed
    2^53 are decimal strings." Reading: a value above 2^53 − 1 is a decimal
    string and a smaller one a JSON number, so each integer has one canonical
    spelling (as `Function::signature` already emits bounds). *Proposed 0.9:*
    "an integer is a JSON number iff it is at most 2^53 − 1, else a decimal
    string."
18. **Classical call spelling (§4.1, §16.2)** *new (round 1)*. Both `call @f a b`
    and `call @f (a, b)` parse under the operand grammar. Reading: §4.1 prints
    `results = call @f args` with bare operands and `uncall @f (args) (results)`
    with lists; each has exactly that one spelling. *Proposed 0.9:* make the two
    forms grammar productions.
19. **Classical call tiers and trap codes (§3.1, §4.1, §7.4)** *new (round 1)*.
    §3.1's diagram shows CC → R, R → R and region calls; it shows neither CC → CC
    nor R → CC. Reading: an R function calls only R functions and Q regions (R is
    the reversible tier; CC has measurement, handles and hostcall); a CC function
    calls CC or R functions; `uncall` follows the same graph, subject to §7.4 (a
    CC segment containing `measure` cannot be uncalled). A CC trap records the
    code the R fault record would carry: `trap`, illegal `uncall`, a failed
    `qreg` precondition (including a simulator's unclean `qreg.free`, reported
    as E_QREG_NOT_CLEAN) and a dead instance record 4; division 5/6; memory and
    hostcall ranges 7; `alloc` 2. *Proposed 0.9:* add the call matrix and "a CC
    trap records the fault code of the corresponding R fault record."
20. **How a borrow frame's factorisation is established (§5.6)** *new (round 1)*.
    §5.6 requires P_F = P′ ⊗ I_B "established by a T3 claim over the frame's full
    domain (exhaustive, or a library certificate) or structurally by the
    translation lemma". Reading: the reference checker establishes it itself,
    exactly, by deciding the frame's Boolean functions in algebraic normal form
    (a canonical form, so the decision is a proof over the full domain, with no
    wire limit; loops by repeated squaring with recursive substitution into
    opaque application inputs, corrected in round 3). With opaque functions this
    is a sound symbolic proof, not a complete function-specific decision; a
    frame it cannot prove (a black-box
    call whose inputs depend on borrowed wires) is admitted only through verified
    certificate `borrow_frames` claims (proofs over D whose borrowed components
    are restored and do not influence the other outputs). *Proposed 0.9:* "A
    checker MAY establish the factorisation by any exact decision procedure.
    Every completed qborrow/qreturn lifetime MUST satisfy it independently of
    future borrowing; a certificate of the final whole-region map alone does
    not discharge internal return obligations."
21. **Checking exhaustive and sampled claims (§12.1–§12.2)** *new (round 1)*. The
    claim schema fixes neither the output tuple nor the table encoding, nor the
    budget of an oracle claim. Reading: a domain element's output tuple is the
    region's final interface in signature order (oracle: ins, tgt, fault,
    in-region cells; action: mut bundles, in bundles, region cells), then the
    borrowed components; `expected.table` lists one tuple per element in
    enumeration order, each component a §9.1 integer; `expected.program` names a
    region of equal signature; an oracle claim takes its budget from its subject's
    schedule when that schedule is the object's own; `sampled` claims replay the
    §12.1 xoshiro256** draws and yield evidence; only an exhaustive claim over D is
    a proof. `expected: "oracle"`, `numerical` and `channel` compare realisations
    under a schedule and are reported unverified until the target/schedule and
    interval machinery lands (deliverables 9–11). *Proposed 0.9:* specify the
    tuple and table encodings and a `budget` field for oracle claims.
22. **Certificate composition at link time (§9.3, §12.3)** *new (round 1)*.
    Reading: linking unites region records (equal where they meet) and
    multiplicities (equal where they meet), rebinds every hash including composite
    `<hash>:<variant>:<c>` keys and changed schedule hashes, binds the composed
    certificate to the first certificate's root, requires equal versions, dialect
    mapping and target bindings, and recomputes `epsilon_total.diamond` exactly as
    Σ n × local (operator norms ×2) plus the root's local bounds. Closure hashes
    over pinned imports use the pin, so a client certificate can be keyed before
    linking (§9.3 requires the pin to equal the linked closure). *Proposed 0.9:*
    state the composed bindings and the epsilon_total formula.
23. **`in` wires as `qstore` values (§4.6, §5.1)** *new (round 1)*. §5.1 lists
    "qcopy/qxor sources"; §4.6 defines `qstore name a v` as "for each i: under
    a == i, qxor v cell_i". Reading: the stored value is a qxor source and may be an
    `in` wire; `qexchm`'s value is exchanged and may not (muse B4 declined).
    *Proposed 0.9:* add "qstore values" to the §5.1 list.
24. **Dynamic SSA without dominance (§3.4, §7.4)** *new (round 1)*. The grammar
    imposes no definition-before-use order and §7.4 resolves a name to its most
    recent instance. Reading: classical types are inferred independently of block
    order (codex B11); reading a name with no current instance on the executed
    path is the CC trap / R fault FAULT_UB_TRAP that §7.4 prescribes for uncalled
    instances. *Proposed 0.9:* "Reading a name that has no instance is
    FAULT_UB_TRAP."
25. **§6 cleanup of `other` values with outstanding readers (§6.1–§6.3)**
    *new (round 1)*. Read literally, "other values are released only by the
    adjoint of the sequence that produced them" admits
    `h d0; ccx d0 p d1; cx d1 t; ccx d0 p d1; h d0; qfree d0`, which releases d0
    entangled with t (found by the crate's differential fuzzer). Reading (needed
    for T5): the adjoint step of an `other` value is refused while any other
    value or outstanding diagonal phase depends on it, transitively through
    dependency sets; windows open at their computation; cell versions advance
    on every write, even one later undone; operation identities include the
    versions of every control and effective-control wire. *Proposed 0.9:* "An
    `other` value is released only by the adjoint of its computing sequence, and
    only while no other live value or outstanding diagonal phase depends on it."
26. **Executing `qinplace` (§4.5)** *new (round 1)*. The certificates §4.5 requires
    bind to a token whose budget is known only at run time. Reading: before any
    state change, the executor decides the §4.5 conditions exactly under the
    actual token (both oracles fault-free on every n-bit input, g∘ginv =
    ginv∘g = id) and refuses otherwise; at load it checks the interfaces
    (wire[n] → wire[n], no mem or args, one fault width). *Proposed 0.9:* "An
    executor checks the §4.5 conditions under the call's token."
27. **Suite sizes for the step-check tool (§17 item 13)** *new (round 1)*. Reading:
    `step-check --suite` applies the default (exhaustive at ≤ 16 bits, else 2^16
    xoshiro256** samples, reported as `"mode":"sampled"`); `--exhaustive` and
    `--samples N` are explicit, with no fixed size limit. No change proposed.

28. **qif adjoints with cross-arm lifetimes (§3.5, §4.6, §5.3)** *new (round 2)*.
    Question: §5.3 prints `qif w {A}{B} ↦ qif w {adj A}{adj B}`, but §3.5
    makes allocation/release unconditional, and §4.6 expands then before else.
    A legal `qif p { d = qalloc any 1 } { qfree d }` must recreate d in the
    inverse else before releasing it in the inverse then. Reading: reverse the
    complete §4.6 expansion, including unconditional lifetime operations, so the
    adjoint visits inverse else before inverse then. Disjoint controlled gates
    commute across arms; their lifetimes need not. No arm-local lifetime
    restriction is added. *Proposed 0.9:* "The qif adjoint shorthand assumes
    each arm's workspace lifetimes are self-contained; otherwise take the
    adjoint of the complete §4.6 expansion, reversing unconditional lifetime
    operations as well."

    Round-3 local-region regression: `r1_borrow::codex_r3_b03_adjoint_child_local_region_exact_witness`
    and its nonzero-image/zero-trip-declaration variants pin the same complete
    lifetime principle for inlined calls: unconditional entry allocation and
    image preparation, body, inverse image gates and release. Both call directions
    preserve the image, including under inherited predicates. This follows the
    explicit §4.6 rule; no additional ambiguity or restriction is introduced.

29. **Measurement draw order (§14.2)** *new (round 5, muse suggestion)*.
    Question: projective Born probabilities do not specify how a supplied draw
    maps to an outcome. Reading: outcomes follow lexicographic order of their
    printed bits string (wire 0 first), independently of the little-endian
    integer value. Select the first outcome whose cumulative exact weight is
    strictly greater than draw × total weight; an exact CDF tie selects the
    next outcome. The distribution is unchanged. *Proposed 0.9:* "Reference
    measurement draws enumerate outcomes lexicographically by the bits string
    (wire 0 first), selecting the first cumulative Born weight exceeding the
    draw times total weight." Pin:
    `registers::muse_r5_measurement_draws_order_bits_lexicographically`.

30. **Translation-frame maximality in structured bodies (§5.6)**
    *new (round 5, muse suggestion)*. Question: does “maximal contiguous
    permutation-only subcircuits” mean maximal in unrolled execution or within
    the static instruction sequence containing the endpoints? Reading: within
    the containing static qloop/qif body sequence, consistent with canonical
    nested-line points and independently checked frame instances. The region
    still satisfies factorisation at every frame/return; this does not waive
    any borrowed touch or lifetime obligation. *Proposed 0.9:* "Maximality of
    a frame inside a qloop/qif body is judged within that body's instruction
    sequence." Pin:
    `r1_certificates::muse_r5_translation_frame_maximality_is_within_its_static_sequence`
    has two pre-loop identity touches followed by a loop-body frame at ordinals
    4–7; the structural claim proves that complete inner frame.

31. **Coincident dotted layout paths (§9.2, §16.1)** *new (round 12)*.
    Question: legal dotted owner/region names can give distinct structural
    leaves the same rendered dotted path; §9.2 (5b) supplies no tie order.
    Reading: retain every leaf; sort by rendered path bytewise, then by the
    original JSON key components in structural order, component-wise bytewise.
    The order is independent of the values. Exact oracle ownership and lproc
    global selection use structural keys. *Proposed 0.9 draft, §9.2 (sol's
    wording):* "Emit one line for every structured layout leaf, including leaves
    with coincident dotted renderings. Sort by the rendered dotted path bytewise;
    break ties bytewise by the sequence of original JSON key components."
    Pins: identity::r12_cc_r_colliding_dotted_leaves and
    r12_cc_r_layout_paths_preserve_colliding_leaves; no accepted-spec edit.

32. **Global/interface address namespace (§4.2, §9.2)** *new (round 12)*.
    Question: both a layout global and an oracle's own interface region can
    supply a base for the same lift.addr spelling, but no precedence is given.
    Reading: retain the existing rejection only for names in layout.globals;
    function symbols, non-global data symbols and imports may share interface
    names. Classify lift.addr in its declaring oracle as an interface address;
    call operands and qregion images remain symbol references. This narrows the
    former rejection, never extends it. *Proposed 0.9 draft (codex's wording):*
    "An oracle interface region must not share a name with a global in
    layout.globals; it may share a name with a function symbol."
    The old global-collision negative remains in r1_linking; the unambiguous
    self-named positive is r12_interface_region_may_share_function_name.

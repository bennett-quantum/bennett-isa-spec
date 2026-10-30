# ref 0.9 constructor checkpoint — NOT a binding library

This is resumable work for b-isa-spec-8r4, not a complete catalog or a
publishable `ref@0.9`. No complete-library hash is asserted. Publication and
the draft's full-catalog hash/count edit remain blocked on b-isa-spec-cbk
(wire/body bindings and structured expansion) and b-isa-spec-8sc (native
rotation/variadic parameters and quantum witnesses). See the progress file.

The top-level schema and closed constructor keys follow draft §11.1.
Recipes are sorted by bytewise kind; parameters are ordered integer names.
`N >= 1` is a bit width. `i` is a loop variable. Wire subscripts contain an
integer literal, loop name, or JSON prefix expression. `inverse: true`
reverses the expanded body and adjoints each leaf. These concrete spellings
are checkpoint conventions, not amendments to the normative contract.
Scratch lists enumerate declaration names; expanded allocation occurrences
receive distinct bit identities. No node sharing or removability is inferred.
All gates inherit ambient effective controls; allocation/release do not.

`conformance/tools/ref_catalog.py` generates canonical UTF-8 (no final LF),
with a small integer-subset JCS canonicalizer, and validates the printed
qxor witness's four-line envelope hash before writing. Run with `python3 -I`.
`conformance/src/ref_catalog.rs` independently expands catalog constructors,
resolves integer-only invokes (no invented wire binding), unrolls loops,
adjoints inverse bodies, numbers leaves, and compiles a bit permutation
machine. Unsupported constructors, gate operands and dirty release fail
explicitly. Tests enumerate the full legal domain, both inherited control
values, and visit original ordinals backwards to restore the entire state.
The checker does not read or import interpreter source. The 0.8 harness
runs the existing binary as an independent regression gate only.

| §4 instruction/row | Recipe or blocking disposition |
|---|---|
| 4.1 const | Classical event; binding schema blocked (cbk) |
| 4.1 add sub mul and or xor shl lshr ashr | Classical events; binding schema blocked (cbk) |
| 4.1 udiv urem sdiv srem | Classical events; binding schema blocked (cbk) |
| 4.1 icmp eq ne ult ule slt sle | Classical events; binding schema blocked (cbk) |
| 4.1 select; zext sext trunc; ptrtoint inttoptr | Classical events; binding schema blocked (cbk) |
| 4.1 load store alloc free | Classical events; binding schema blocked (cbk) |
| 4.1 br call ret uncall trap fault | Classical events; binding schema blocked (cbk) |
| 4.1 qreg.new free slice concat width id from_id | Classical events; binding schema blocked (cbk) |
| 4.1 measure / measure.iN; budget.mint; effect; hostcall | Classical events; binding schema blocked (cbk) |
| 4.2 lift.const | Assignment composition blocked (cbk) |
| 4.2 lift.binop (all nine operations) | Value helpers below; assignment composition blocked (cbk) |
| 4.2 lift.div (all four operations) | Value helpers below; flag/assignment composition blocked (cbk) |
| 4.2 lift.icmp (all six predicates) | Value helpers below; assignment composition blocked (cbk) |
| 4.2 lift.select; lift.cast (all three casts) | Value helpers below; assignment composition blocked (cbk) |
| 4.2 lift.mov lift.parmov lift.label lift.addr | Assignment composition blocked (cbk) |
| 4.2 lift.boundscheck lift.load lift.store lift.alloc lift.free | Value/dispatch helpers below; row composition blocked (cbk) |
| 4.2 lift.faultif | Row composition blocked (cbk) |
| 4.2 lift.call lift.ret | C1–C5/R1–R5 binding blocked (cbk) |
| 4.2 lift.retval | Row composition blocked (cbk) |
| 4.2 lift.oracle_call lift.oracle_uncall lift.perm_call | Callee/body binding blocked (cbk) |
| 4.2 qjump qbranch qjumpr fault | Successor/row binding blocked (cbk) |
| 4.3 qalloc qborrow qfree qreturn | Generic class/borrow binding blocked (cbk) |
| 4.4 x cx ccx swap | gate.x gate.cx gate.ccx gate.swap |
| 4.4 mcx; h s sdg t tdg cz rx ry rz | Variadic/angle/quantum witness gap (8sc); not certified as basis permutations |
| 4.5 qcopy quncopy qxor qexch | qcopy quncopy qxor qexch |
| 4.5 qinplace | Oracle binding blocked (cbk); retain printed operation order for repair |
| 4.6 qregion | Image/body binding blocked (cbk) |
| 4.6 qload qunload qstore qexchm | Dispatch helpers below; region binding blocked (cbk) |
| 4.6 qpred qunpred qif qloop qloop-while | Body/control binding blocked (cbk) |
| 4.7 oracle_call action_call adj ctl | Body/symbol/control binding blocked (cbk) |
| 4.8 opcode summary | Same dispositions as the individual rows above |
| 4.9 classical faults (all causes) | Classical event/history binding blocked (cbk) |
| 3.6 forward/inverse step | Heterogeneous row/body binding blocked (cbk) |
| 3.7 activation C1–C5/R1–R5 | Location/callee/stack binding blocked (cbk) |
| 5.2 allocate/forward/commit/reverse/release | Oracle/row/body binding blocked (cbk) |

Completed first-family witnesses: widths 1–4 for copy/XOR/exchange, 1,480
legal tuples; fixed native permutation gates, 36 tuples (counts include
both ambient control values). `qcopy` has a clean allocated destination;
`quncopy`'s legal domain is the matching copied destination, not arbitrary
dirty inputs. `qxor`/exchange enumerate arbitrary source and target entries.

Ripple value helpers: `value.add`, `value.sub`, `value.icmp.eq/ne/ult/ule/slt/sle`.
They XOR the arithmetic result into arbitrary `t`, preserving separate `a,b`
bundles and cleaning all fresh Boolean-node scratch. These are value helpers,
not a claim to have encoded the lifted assignment/frame/successor rows.
Full domains at widths 1–4: 26,880 tuples including both controls.

Other arithmetic: `value.and/xor/or/mul/select/zext/sext/trunc/udiv/urem/sdiv/srem`
and `value.shl/lshr/ashr.w1/w2/w3/w4`. The shifts are explicit width
specializations; wider shifts remain unfinished. Division uses restoring
long division at N+1 remainder bits and conditional signed magnitudes,
XORs code 5/6 into arbitrary four-bit `f`, and leaves `t` unchanged on
fault. Full domains include signed MIN/-1 and every divisor-zero tuple.
Widths 1–4: 664,560 arithmetic and 21,760 cast/select tuples.
Rust std-only SHA-256 and Python hashlib independently agree on the
checkpoint envelope; both canonicalizers verify the printed qxor witness.
A checkpoint digest does not identify the complete library.

# ref 0.9 constructor catalog — incomplete, not publishable

Current checkpoint: **195 recipes**. The complete-library hash/count marker
in draft §11.1 remains untouched. No ref/ publication or schedule binding is
claimed. Historical checkpoints and family witness counts are retained in
[the progress file](ref-0.9-draft.PROGRESS.md). No interpreter source was read
or copied into the checker.

The document is RFC 8785 canonical UTF-8 without a trailing LF; recipes are
ordered by bytewise kind. Every recipe has kind, ordered parameters, body,
scratch, removable:[], and its motivating ISA source. Typed formals,
constructors, lexical capture, call ordinal spaces and fresh allocation
identities are defined by §11.1 K19–K24. An image actual binds exact data
bytes; empty means absent/default-zero even for non-byte-aligned regions.
Template application binds explicit frame/wire actuals while preserving free
captures. Arguments vectors are tied to one closure hash and capture their
actuals' environment; value formals preserve exact §12.3 integer, ptr, f64
and token encodings. They do not yet encode source region roles/shapes.

The generators under conformance/tools/ contain no interpreter imports.
Run ref_catalog.py with python3 -I (the required headless project TMPDIR
environment applies). Python's independent integer-subset JCS canonicalizer
checks the printed qxor witness and hashes the four-line envelope. Rust's
independent canonicalizer and SHA-256 must agree with it. Source/scratch
names, bound images and constructor order are hash-significant.

## Current coverage, in §11.1 family/row order

| ISA row | Recipe / exact remaining disposition |
|---|---|
| §4.5 qxor/qcopy/quncopy/qexch | qxor/qcopy/quncopy/qexch; positional bind.qxor/bind.qexch adapters. Generic allocation result lifetimes for copy/uncopy remain 3xe. |
| §4.4 x/cx/ccx/swap | gate.x/cx/ccx/swap; typed bind.gate.x/cx/ccx/swap |
| §4.4 h/s/sdg/t/tdg/cz | gate.h/s/sdg/t/tdg/cz; exact full-basis matrix witnesses |
| §4.4 rx/ry/rz/mcx(k) | gate.rx/ry/rz/mcx; exact angle bits, numerical matrix evidence for rotations, full-domain variadic mcx |
| Ripple add/sub/comparisons | value.add/sub/icmp.eq/ne/ult/ule/slt/sle and bind.value.* adapters |
| Other arithmetic/casts/select | value.and/or/xor/mul/select/zext/sext/trunc/udiv/urem/sdiv/srem; width-uniform bind.value.shl/lshr/ashr plus historical w1..w4 helpers |
| Linear inrange/free/alloc/call flags | scan.inrange/free.flag; value.alloc.flag; value.call.flag plus historical fixed-capacity helpers |
| Address dispatch | dispatch.copy/store/exchange, historical .a1 helpers; qstore/qexchm |
| §4.1 const; arithmetic/Boolean/shifts/div/rem; icmp; select/casts/pointer conversions | cc.<opcode>, bound classical instruction/execution-point leaf events |
| §4.1 load/store/alloc/free; br/call/ret/uncall/trap/fault | cc.<opcode>, bound classical events |
| §4.1 qreg.new/free/slice/concat/width/id/from_id; measure/measure.iN; budget.mint/effect/hostcall | cc.<opcode>, 48 classical bindings total |
| §4.2 lift.const/binop/icmp/select/cast/mov/parmov/label/addr | lift.const; lift.add/sub/mul/and/or/xor/shl/lshr/ashr; lift.icmp.<pred>; lift.select/zext/sext/trunc/mov/parmov/label/addr |
| §4.2 lift.div | lift.udiv/urem/sdiv/srem; old-state flag before success-only assignment and effective fault writes |
| §4.2 lift.boundscheck | lift.boundscheck with widened pointer/length endpoints |
| §4.2 lift.load/store | **Blocked 1oa:** typed ordered mapped-region layout records and byte/wire bindings absent |
| §4.2 lift.alloc/free | lift.alloc/free with exact slot/exchange order, pinned-free correction and no-wrap checks |
| §4.2 lift.faultif/fault/retval | lift.faultif, fault, lift.retval; cause 0 maps to 4, F≥4 |
| §4.2 lift.call/ret; §3.7 C1–C5/R1–R5 | **Blocked evi:** ordered heterogeneous location descriptors (id,width,wire,parameter,capture/slot/stack mapping) absent; general value.call.flag is a verified predicate only |
| §4.2 lift.oracle_call/oracle_uncall/perm_call | **Blocked 1a4:** source region shape/role descriptor binding; K24 repairs wire/value/token vector forwarding only |
| §4.2 qjump/qbranch/qjumpr | qjump/qbranch/qjumpr and successor.literal/flagged/dynamic |
| §3.6 forward/inverse step | step: indexed row/successor templates, explicit current frame, dispatch including HALT, commit pc, clean dispatch; inverse visits retained leaves backwards |
| §4.3 qalloc/qborrow/qfree/qreturn | **Blocked 3xe:** exact class, output/declaration wire binders and invocation lifetime results absent |
| §4.5 qinplace | **Blocked 82x:** printed t/f release order conflicts with ratified scratch LIFO; also needs 1a4 complete interfaces |
| §4.6 qregion | qregion: complete function-local scope, unconditional image preparation/restoration, explicit cells template argument, clean release |
| §4.6 qload/qunload | dispatch.copy verified; allocation-producing row lifetime binding **blocked 3xe** |
| §4.6 qstore/qexchm | qstore/qexchm, generic address scan |
| §4.6 qpred/qunpred/qif/qloop/qloop-while | qpred.scope (matched lexical pair), qif, qloop, qloop.while |
| §4.7 adj/ctl | adj/ctl body composition; generic symbol call source regions still **1a4** |
| §4.7 oracle_call/action_call | K24 closure-bound argument forwarding mechanism checked; source region shape/role binding **1a4** |
| §4.8 opcode summary; §4.9 faults | Same rows above; classical faults retained in bound §4.1 event/history semantics |
| §5.2 allocate/forward/commit/reverse/release | **Blocked 3xe:** heterogeneous state allocation/class/lifetime metadata; step-template frame binding itself is now K22 |

The exact blockers remain capabilities to implement. Passing a prebuilt
caller dispatcher/activation/construction as an opaque body does not put
its algorithm in the catalog and is not a completed row. Beads use the
b-isa-spec- prefix: cbk/8sc closed before this resumption, afi/lph repaired
by K21/K22, 1a4/3xe/1oa/82x carried, evi filed for residual activation
metadata. The catalog parent 8r4 stays open.

## Evidence and limits

Rust integer/permutation and reduced ordered BDD checkers independently
compare full-domain Boolean functions to plain arithmetic/range/dispatch
semantics, check every scratch boundary, preserve required inputs and
restore the complete state by reverse execution. BDD domain restrictions
(e.g. atop≤capacity, pc≤HALT) and exact tuple exponents are in progress;
these are exhaustive symbolic domains, not random samples. No table-only
helper is called a complete activation/oracle witness. Classical event
witnesses check exact binding bytes, not classical execution or a unitary.

Structured loops use bounds 0,1,2,3,17 at widths 1–4, including nested
finite actual bodies to expansion fixpoint. New step checks cover 1,218,560
tuples across widths 1–4 and row counts 0/1/2/3/17, including heterogeneous
assignment, fault and direct rows plus idle frames. Repeated frame slices
at bounds 0/1/2/3/17 add 3,400 tuples. New qregion scopes cover cell widths
1–4, cell counts 1/8, 272 image configurations (all 256 one-byte images,
default non-byte-aligned zero, and specified multibyte patterns), 1,088
full-domain tuples and 1,088 exact coherent matrix columns. The sparse
exact checker keeps every allocation coordinate and checks clean boundaries;
it agrees with the dense checker on entangled Clifford+T columns.

Exact amplitudes use checked dyadic rationals in Q(√2,i), including global
phase. Native H/S/SDG/T/TDG/CZ: 356 columns with spectators and controls;
MCX k=0/1/2/3/4/17: 524,412 basis tuples; typed X/CX/CCX/SWAP: 36 tuples.
Rotations use eight finite binary64 angle encodings (both zeros, subnormal,
±1, π encoding, positive/negative maxima), all three rotations and widths
1–4: 1,440 numerical columns. Required round trip: f64::from_bits(bits).
to_bits()==bits, inverse bits XOR 2^63; no decimal rendering or recipe
angle arithmetic. Matrix entries, norms and U†U have absolute residual
≤8·f64::EPSILON using independent Pauli/spectral formulas. This is numerical
evidence at those angles, not an ε=0 transcendental proof for every angle.
The generic call predicate adds 43,200 tuples across 192 configurations
(D,S independently1–4, R4/5/16, two depth/stack limits), plus 131,072 at
the exact §16.3 D5/S10/R16/limits17/272 counter configuration.

Current **checkpoint-only** envelope SHA-256 (Rust and Python -I):
`84e8940646f1cf2416790b71a9eed525589f0c62e2c397f2a7d08e0b33233230`.
The historical 54-recipe checkpoint was
`3a99ce3028e1ecc421b3991e3b5b7423ee33323badc5ca3ce1e61088a5e5055d`.
Neither is a complete-library binding. In particular the first remaining
blocked row, lift.load, still cannot express its ordered mapped layout
under §11.1's typed formal grammar/K22 template-only list rule (1oa).

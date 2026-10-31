# ref 0.9 constructor catalog — complete under draft 0.9

Status: **complete under draft 0.9 — 219 recipes; publication remains gated
on accepted 0.9**. Every row in the disposition table below is bound.
The complete-library hash/count is recorded once in draft §11.1. No ref/
publication or schedule execution is claimed. Historical checkpoints and
family counts remain in [the progress file](ref-0.9-draft.PROGRESS.md).
No interpreter source was read or copied into the checker.

The document is RFC 8785 canonical UTF-8 without a trailing LF; recipes are
ordered by bytewise kind. Every recipe has kind, ordered parameters, body,
scratch, removable:[], and its motivating ISA source. Typed formals,
constructors, lexical capture, call ordinal spaces and fresh allocation
identities are defined by §11.1 K19–K28. An image actual binds exact data
bytes; empty means absent/default-zero even for non-byte-aligned regions.
Template application binds explicit frame/wire actuals while preserving free
captures. Arguments vectors are tied to one closure hash and capture their
actuals' environment; value formals preserve exact §12.3 integer, ptr, f64
and token encodings. K25–K27 add ordered descriptors, source region roles/shapes/origins and shared allocation declarations.

The generators under conformance/tools/ contain no interpreter imports.
Run ref_catalog.py with python3 -I (the required headless project TMPDIR
environment applies). Python's independent integer-subset JCS canonicalizer
checks the printed qxor witness and hashes the four-line envelope. Rust's
independent canonicalizer and SHA-256 must agree with it. Source/scratch
names, bound images and constructor order are hash-significant.

## Current coverage, in §11.1 family/row order

| ISA row | Recipe / exact remaining disposition |
|---|---|
| §4.5 qxor/qcopy/quncopy/qexch | qxor/qcopy/quncopy/qexch; positional bind.qxor/bind.qexch adapters. qcopy.bound/quncopy.bound preserve K27 caller result lifetimes. |
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
| §4.2 lift.load/store | lift.load/store: K25 complete byte-partition dispatch, inherent range/fault check and single assignment slot |
| §4.2 lift.alloc/free | lift.alloc/free with exact slot/exchange order, pinned-free correction and no-wrap checks |
| §4.2 lift.faultif/fault/retval | lift.faultif, fault, lift.retval; cause 0 maps to 4, F≥4 |
| §4.2 lift.call/ret; §3.7 C1–C5/R1–R5 | lift.call/ret: complete C1–C5/R1–R5 over K25 location/capture descriptors, with stack.copy/store/exchange low-cell dispatch |
| §4.2 lift.oracle_call/oracle_uncall/perm_call | lift.oracle_call/uncall/perm_call: K24/K26 full typed argument vectors, semantic symbol paths and callee ordinals |
| §4.2 qjump/qbranch/qjumpr | qjump/qbranch/qjumpr and successor.literal/flagged/dynamic |
| §3.6 forward/inverse step | step: indexed row/successor templates, explicit current frame, dispatch including HALT, commit pc, clean dispatch; inverse visits retained leaves backwards |
| §4.3 qalloc/qborrow/qfree/qreturn | qalloc/qborrow/qfree/qreturn: K27 declaration acquire/relinquish with exact class/state metadata |
| §4.5 qinplace | qinplace: exact §4.5 allocation/calls/exchange/t-then-f release sequence under K28; K22 source-call adapters retain complete K24/K26 interfaces |
| §4.6 qregion | qregion: complete function-local scope, unconditional image preparation/restoration, explicit cells template argument, clean release |
| §4.6 qload/qunload | qload.in/mut and qunload.in/mut: K26 source region + K27 result declaration |
| §4.6 qstore/qexchm | qstore/qexchm, generic address scan |
| §4.6 qpred/qunpred/qif/qloop/qloop-while | qpred.scope (matched lexical pair), qif, qloop, qloop.while |
| §4.7 adj/ctl | adj/ctl body composition; complete symbol interfaces use K24/K26 |
| §4.7 oracle_call/action_call | oracle_call/action_call: complete K24/K26 interface forwarding |
| §4.8 opcode summary; §4.9 faults | Same rows above; classical faults retained in bound §4.1 event/history semantics |
| §5.2 allocate/forward/commit/reverse/release | oracle.construct: K25 state/frame descriptors, catalog step loop, commit, prefix reversal and reverse-order state release |

All current rows have serialized recipes. K28 resolves the final release-order
conflict without editing §4.5: instruction-private t/f are released in the
printed order; recipe scratch within either call still follows LIFO.
All catalog contract children (cbk/8sc/afi/lph/1oa/evi/1a4/3xe/4nx/82x) are
closed. Catalog parent 8r4 is closed after the final gate. As before,
the family witnesses certify their stated configurations, not every
possible source object or an ISA 0.9 executor. Linked source/layout and
closure-hash validation remains a prerequisite to typed descriptor input.

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

Complete library envelope SHA-256 (independent Rust and Python -I):
`fcb2f1e6d61aced0bf9689eab5f6d82c4ae42e9fea8d1b21ae9c7ffccda6f5ce`.
The previous 218-recipe checkpoint was
`64a05ef20201faad429ca2568ee27848fae0b833f95c006a7d594df0313da4ca`.
The historical 54-recipe checkpoint was
`3a99ce3028e1ecc421b3991e3b5b7423ee33323badc5ca3ce1e61088a5e5055d`.
Those historical checkpoints are not complete-library bindings; the 219-recipe binding above includes qinplace.

Previous-pass gate (historical): **88 Rust pass / 0 fail**, fmt/clippy green; harness
**445 pass / 0 fail / 68 unavailable**, versus that pass's entry 445/0/65. Previous-pass commits:
bbb1b62 (Part 1), 0656e14 (step/qregion family and resumed native evidence);
see progress for the exported-beads bookkeeping commit. No push.

## Completion II byte-memory family

K25 byte partitions select mapped cells and ordinary mem, preserving
ordinary bytes hidden under mappings as arbitrary spectators. Both rows
carry inherent widened inrange and effective fault 7. Load computes the
whole displacement slot before exchanging its destination (including
address/destination aliases); store copies the entire value before any
selected byte exchange. All scratch disappears before load's destination
exchange; inverse restores the old address before its scans.

ref_layout.rs proves 88 full symbolic configurations: pointer widths 1–4,
accesses 1/2/3/4/16 bytes, with 0/1/2/3 mapped byte regions as width permits;
plus P=5, 16-byte accesses with 0/17 mapped regions. All memory, mapped
cells, values, fault/code and table inputs are arbitrary, flag/slot clean;
one table entry, legal base+size<=2^P, atop=0/1, ambient control included.
Exact tuples per non-alias configuration: (M*(M+3)/2-1)*2^(P+9+8*B+16*M), M=2^P.
Total tuples: 167143296548916895889517112694306429377925266509132267598234820116853215623227220551812585180993364921141732939506160915932057223539145589421667637993405146421832216413988504826109858941640784576774144. Scratch and complete reversal checked.

Four additional P=N=8, H=2 byte configurations bind value/destination
to the address location itself (load and store, mapped/unmapped). Their
full domains add 60*2^89 tuples, including unused declared spectators.
Family gate: 95 Rust tests, 0 fail; fmt/clippy green; harness 445/0/69
(+1 pending-0.9 case). The alias-only final extension passes separately.

## Completion II allocation family

Ten recipes bind exact classes and declaration lifetimes for §4.3,
qcopy/uncopy and qload/unload with both source region roles. Widths 1–4;
A and W independently 1–4 for loads, classes any/ancilla/data.
108 full-domain BDD shapes / 7083657813172691625768 tuples; 180 exact borrowed
basis columns (all unknown borrowed inputs exposed, no zero assumption).
Four direct clean allocation/free shapes additionally check class metadata.

## Completion II call and activation family

Five source symbol-call recipes preserve complete interfaces, arbitrary
classical i128/token values, source region roles/shapes/origins, controls
and invocation-local ordinals. Widths1–4, valid in/mut roles: 1,048,320
full input/control tuples, preserving every spectator and complete inverse.

Three low-cell stack dispatch helpers and lift.call/ret encode all printed
C1–C5/R1–R5 operations. C1 reads recursive source aliases before C2; all
C5 counter values precede exchanges; inverse restores counters/fp before
header reads. R4 scans every global location, touching only the matching
location's low width; narrow nonmatching locations remain included.
W1–4; three locations per selected instance, one other closure location;
cell width4, capacity8, Sc4, Dc2, P3, Did2; limits depth2/stack8, R6.
12 call shapes / 11,070,410,916,511,088,640 tuples; 4 return shapes /
5,266,185,708,023,840,768 tuples. All record/history bits and locations
are arbitrary subject to active-record cleanliness, valid delivery ids
and legal counters/header addresses. False ambient controls retain the
whole arbitrary stack. BDD satisfying counts are exact, overflow-checked
and independently truth-table-tested on small Boolean domains.

Call-family gate: full Rust suite 104 pass/0 fail; harness 445/0/72;
fmt/clippy green. The subsequent ordinal audit caught lift.load/store
fault-write leaves preceding assignment effects through boundscheck
composition. A leaf-order regression was observed red; flag computation
is now separate, with fault writes after all exchanges (§3.6). The
88 full-domain memory shapes and ordinal test are rerun after correction,
and independent Rust/Python checkpoint hashes are recomputed. This
corrects generated order without changing the byte-memory permutation.

## Completion II construction family / final checkpoint

`oracle.construct` allocates and initializes the K25 state list, invokes
its catalog step algorithm for each source-bound frame, commits, reverses
frames and releases state in reverse allocation order. It takes source
row/successor templates; no caller supplies an opaque dispatcher.
Its 60 full-domain shapes use W1–4 and B0/1/2/3/17: assignment followed
by success, assignment followed by fault7, and a nonhalting update cycle.
**2,246,400 tuples** include arbitrary two-input/target/fault values and
both ambient controls, retained assignment history, complete reversal,
and unconditional local image initialization/restoration (81). Every
private state descriptor is marked scratch on acquire, with classification
retained through record scope and release aliases.

All corpus cases remain pending 0.9. Final gate and commit mapping are
recorded in the progress file; the final suite passes 108 Rust tests
and harness 445/0/73. Full-domain witnesses at one construction program
family do not claim exhaustive source-program enumeration. Large memory
examples, source/link validation and recursive programs are covered by
separate component witnesses, not inferred from these construction tuples.

Completion-II gate (historical): **108 Rust tests pass / 0 fail**, including
ref_call_flag.rs; fmt --check and clippy --offline --all-targets -D warnings
green. Harness **445 pass / 0 fail / 73 unavailable**, compared with
entry **445/0/68**; exactly five new pending-0.9 cases account for +5.
Rust/Python canonical bytes and checkpoint envelope agree. Accepted 0.8
SHA-256 remains 90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e;
qxor witness hash remains 299510fd67cc626fdcbcc10adcd40730f9af294ba596d86fb25fd31be08a5c7f.

Commit mapping: 90590a7 contract; fedc9c6 byte memory; f2427ec allocation;
7ff93a5 calls/activation and memory ordinal correction; e649e36 construction.
The final memory-domain count is independently asserted with the exact BDD
counter: 15 legal base/size pairs at P8/H2, adding 60*2^89 across four
alias shapes. The two memory tests pass after this bookkeeping correction.
Detailed per-family evidence and the final bd export are in progress.

## Completion III — qinplace and complete binding

One new recipe, `qinplace(N,F,x,G,H,E)`, owns the entire printed sequence.
G/H are one-element K22 templates with `(x:wire[N],t:wire[N],f:wire[F])`
formals. Each source-binding adapter contains exactly the corresponding
linked oracle `call`, capturing that source call's complete `ins`, `mem`,
`args` and token actuals. It does not supply allocation, exchange, cleanup,
or a decomposition algorithm. This permits arbitrary heterogeneous source
interfaces without referring to private wires before acquisition. Source
signature/overlap validation, closure identity and the covering inverse-pair
certificate remain §4.5 prerequisites; the expander is not a source linker
or certificate checker. Symbol fixtures use synthetic registry identities.
The adjoint swaps the two adapters, retaining all captures.

Private instruction allocations use K27 declarations, class any and clean
state, with t/f widths N/F. They are not recipe scratch under K28; all
ordinary recipe scratch retains identity-based clean LIFO enforcement.
The leaf witness checks exact allocation/release identities and ordinals,
two distinct callee invocation spaces, and unconditional lifetime boundaries.

| New family / evidence | Recipes | Widths / bounds | Exact full-domain witnesses |
|---|---:|---|---:|
| qinplace short form, both orientations | 1 shared recipe | W1–4, F4; B0/1/2/3/17 | 40 shapes / 5,592,320 tuples |
| qinplace full wire/region/i128/f64/token interface, both orientations | same recipe | W1–4, two W-bit region cells; same bounds | 40 shapes / 5,592,320 tuples |
| §16.3 example 7 addya and adjoint | same recipe | W2/F4; all a:i2; legal token (3,0,0,0) | 256 tuples |

For the 80 symbolic shapes, every x/y/region bit, the effective control and
two spectator bits are arbitrary: 2^(4W+3) tuples per shape, including unused
y/region wires as spectators in the short form. Total 11,184,640 tuples.
The direct oracle fixtures implement +B then context XOR and its inverse;
B is their static source loop count (0/1/2/3/17), with token (64,2,3,4),
not a claim that example 7
is fault-free below its three-row budget. The example-7 fixture separately
uses its legal budget and exact arithmetic. Every lifetime boundary is
clean, every unchanged input/spectator is preserved, and reversing original
leaves restores the entire state. These permutation witnesses extend by
T3/linearity to coherent inputs; no non-permutation leaf was added.

Both new tests were observed red on the absent qinplace recipe, then green.
The complete count/hash includes every source/scratch/body byte and agrees
between the independent Rust JCS/SHA-256 and Python -I stdlib computation.
Final fmt/clippy/cargo/harness evidence is recorded in the progress file.

Final verified gate: **111 Rust tests pass / 0 fail** (entry 108/0),
including ref_call_flag.rs; fmt --check and clippy --offline --all-targets
-D warnings green. Harness **445 pass / 0 fail / 74 unavailable (519)**
from entry **445/0/73 (518)**: exactly one new pending-0.9 qinplace case.
The corrected n=1/F=4 K28 witness and draft_contracts pass separately too.
All 31 disposition rows are bound; all 218 earlier recipe objects are
unchanged. Parent 8r4 and final child 82x are closed; beads exported by path.
Status: **complete under draft 0.9; publication gated on accepted 0.9**.

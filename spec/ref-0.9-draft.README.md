# ref 0.9 constructor catalog — incomplete, not publishable

Current checkpoint: **218 recipes**. The complete-library hash/count marker
in draft §11.1 remains untouched. No ref/ publication or schedule binding is
claimed. Historical checkpoints and family witness counts are retained in
[the progress file](ref-0.9-draft.PROGRESS.md). No interpreter source was read
or copied into the checker.

The document is RFC 8785 canonical UTF-8 without a trailing LF; recipes are
ordered by bytewise kind. Every recipe has kind, ordered parameters, body,
scratch, removable:[], and its motivating ISA source. Typed formals,
constructors, lexical capture, call ordinal spaces and fresh allocation
identities are defined by §11.1 K19–K27. An image actual binds exact data
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
| §4.5 qinplace | **Blocked 82x:** printed t/f release order conflicts with ratified scratch LIFO; K24/K26 interfaces are available |
| §4.6 qregion | qregion: complete function-local scope, unconditional image preparation/restoration, explicit cells template argument, clean release |
| §4.6 qload/qunload | qload.in/mut and qunload.in/mut: K26 source region + K27 result declaration |
| §4.6 qstore/qexchm | qstore/qexchm, generic address scan |
| §4.6 qpred/qunpred/qif/qloop/qloop-while | qpred.scope (matched lexical pair), qif, qloop, qloop.while |
| §4.7 adj/ctl | adj/ctl body composition; complete symbol interfaces use K24/K26 |
| §4.7 oracle_call/action_call | oracle_call/action_call: complete K24/K26 interface forwarding |
| §4.8 opcode summary; §4.9 faults | Same rows above; classical faults retained in bound §4.1 event/history semantics |
| §5.2 allocate/forward/commit/reverse/release | oracle.construct: K25 state/frame descriptors, catalog step loop, commit, prefix reversal and reverse-order state release |

The sole remaining contract blocker is qinplace (82x). Accepted §4.5
prints `qfree t; qfree f` after allocating t then f; ratified §11.1/A1
requires library scratch released in reverse allocation order. Resolving
that conflict needs a generated-order/ratification decision, not a new
binding type. No recipe or complete-library marker is supplied for it.
Closed: 1oa/evi (K25), 1a4 (K26), 3xe (K27), 4nx (existing K23).
The catalog parent 8r4 remains open. All other current rows have recipes;
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

Current **checkpoint-only** envelope SHA-256 (Rust and Python -I):
`64a05ef20201faad429ca2568ee27848fae0b833f95c006a7d594df0313da4ca`.
The historical 54-recipe checkpoint was
`3a99ce3028e1ecc421b3991e3b5b7423ee33323badc5ca3ce1e61088a5e5055d`.
Neither is a complete-library binding. The sole omitted row is qinplace, blocked on 82x.

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
Total tuples: 167143296548916895889517112694306429377925266509132267598234820116853215623227220551812585180993364921141732939506160915932057223539145589421667637993405146421832216413988502350229780370880234778525696. Scratch and complete reversal checked.

Four additional P=N=8, H=2 byte configurations bind value/destination
to the address location itself (load and store, mapped/unmapped). Their
full domains add 56*2^89 tuples, including unused declared spectators.
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

Final verified gate: **108 Rust tests pass / 0 fail**, including
ref_call_flag.rs; fmt --check and clippy --offline --all-targets -D warnings
green. Harness **445 pass / 0 fail / 73 unavailable**, compared with
entry **445/0/68**; exactly five new pending-0.9 cases account for +5.
Rust/Python canonical bytes and checkpoint envelope agree. Accepted 0.8
SHA-256 remains 90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e;
qxor witness hash remains 299510fd67cc626fdcbcc10adcd40730f9af294ba596d86fb25fd31be08a5c7f.

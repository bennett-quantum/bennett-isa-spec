# ref 0.9 catalog checkpoint (b-isa-spec-8r4)

Status: incomplete, not publishable; no normative catalog hash/count edit.
Contract blockers filed before implementation: b-isa-spec-cbk and
b-isa-spec-8sc. Their descriptions quote the exact §11.1 sentences and
required contract changes. Publication remains gated on accepted 0.9.
No reference-interpreter source was read or copied.

| §11.1 family, in required order | Recipes completed | Widths / tuples verified | Commit |
|---|---:|---|---|
| Copy/XOR/exchange/native | 8 (rotation/variadic/quantum subset blocked: 8sc) | 1–4 / 1,480; native / 36 | 5663477 |
| Ripple add/sub/comparisons | 8 value helpers; row binding still cbk | 1–4 / 26,880 | e332108 |
| Other arithmetic/casts/select | 24 helpers (12 width-specialized shifts); row binding cbk | 1–4 / 664,560 arithmetic + 21,760 cast/select | 65b24f5 |
| Linear inrange/flags | 11 helpers; table sizes 1/2 and call R_f=4 checkpoints | widths 1–4 (two entries: 1–2) / 1,318,144 inrange + 24,704 free + 149,760 alloc + 1,123,328 call = 2,615,936 | a0d7215 |
| Address dispatch/multiplexers | 3 two-cell helpers; region binding cbk | cell widths 1–4 / 56,160 | 08a2103 |
| Row/oracle/structure | Blocked on cbk: typed wires, heterogeneous bodies, callee binding, inherited controls | No semantic certification; integer invoke/inverse and fail-closed boundaries tested | Blocker checkpoint commit containing this entry |

Checker integer-only invoke support deliberately provides no wire renaming
or body substitution. Such behavior is not fixed by catalog bytes under the
current contract; adding it locally would create incompatible consumers.
Native H/phase/rotation witnesses need state-vector/matrix checks: H does
not map basis states to basis states, so integer permutation checking cannot
satisfy that portion of the brief. No rows are declared inert or out of scope.

Final checkpoint: **54 recipes**, **3,386,812 family witness tuples**.
Widths/configurations are precisely those in the table and README, not a
claim to exhaustive arbitrary-table/closure execution. Exact §16.3 lifted
row/frame/construction domains are not certified; those recipes cannot yet
be bound under the closed schema. Large/general configurations, aliases and
call records are not represented by the primitive helper witnesses.

Checkpoint envelope SHA-256, independently in Rust and Python -I:
`3a99ce3028e1ecc421b3991e3b5b7423ee33323badc5ca3ce1e61088a5e5055d`.
This is **not** a completed-library binding. Neither the draft's catalog
hash/count marker nor its diff was changed; byte-exact diff and unchanged
accepted 0.8 checks still run in the existing gate.

Resumption requires b-isa-spec-cbk / b-isa-spec-8sc contract repairs before
row/oracle/structure serialization, then replacing the specialized primitive
checkpoints with complete generic coverage and expanding the witnesses to
the required configurations. Do not promote any staged case or publish ref/.
The parent b-isa-spec-8r4 remains open. Unrelated concurrent edits in
reference-interpreter/ and docs/ were neither read for implementation nor
included in commits.

Final headless/offline gate: fmt --check and clippy --all-targets -D warnings
green; cargo test **40 pass / 0 fail**. Full 0.8 harness **445 pass / 0 fail /
60 unavailable (505 cases), exit 2**. Six new family cases add unavailable
only, naming b-isa-spec-8r4. No push.

## Resumed — Part 1 and §4.1

Part 1 commit: 5a8ec58; cbk/8sc closed with typed-language witnesses.
Gate: 45 Rust tests, fmt/clippy green, harness 445/0/60.
Classical family: 48 `cc.*` event bindings added (102 total recipes).
16,320 event-binding configurations: all two-operand bit-vector pairs at
widths 1–4 for each event kind. These check binding bytes, not classical
execution; history and effects remain ISA §4.1/§7.4 events.

### Lifted assignment and fault rows

Classical commit: 9847d91 (46 Rust tests; harness 445/0/61).
Added 51 recipes/adapters: width-uniform shifts; add/sub/mul/Boolean,
all comparisons, casts/select/mov/parmov, resolved const/label/addr,
faultif/fault, retval, and all four divisions with old-state flags,
conditional exchange and fault-code writes. 153 recipes total.
Full domains: 84,240 binop tuples; 10,200 comparison/literal tuples;
23,800 cast/select/mov/retval/parmov tuples (including parallel alias
exchange); 1,198,080 division tuples; 1,728 fault tuples. Widths 1–4;
all four-bit code entries and both faulted entries in division/fault rows.
Reverse execution restores the complete allocated state; every release
asserts cleanliness. No matrix claim is made for classical event recipes.

New contract gaps, with exact sentences and required changes in bd:
- b-isa-spec-afi: §4.6 arbitrary qregion images need unconditional image X
  identification; inherited environments only add semantic controls.
- b-isa-spec-1a4: arbitrary symbol interfaces need typed argument-list
  forwarding (including classical/token values and heterogeneous operands).
- b-isa-spec-3xe: named allocation classes and output/lifetime binders are
  absent; passing every class as any would change resource identity.
Do not treat fixed-width helper circuits as those missing generic bindings.

### Generic scans and memory wire operations

Row commit: 0bc8426. Generic `dispatch.copy/store/exchange`, `qstore`,
`qexchm`, `scan.inrange`, `scan.free.flag`, `lift.boundscheck`, `lift.free`
replace fixed-capacity-only coverage. 162 total recipes. Scan parameters:
P=pointer/base/size width, L=length width, E=max(P,L)+1, C=table capacity,
T=atop counter width; 0<=atop<=C. Separate pointer and length widths are
retained, and every prospective endpoint has the required extra bit.
Unused n/size formals for free can bind existing source bundles; they do
not affect operations or the free predicate. Dispatch uses selector width
A, cell count C=2^A, cell width W, address-order packed cells.

The independent reduced ordered BDD checker executes every Boolean input
simultaneously, comparing canonical Boolean functions to separately built
unsigned arithmetic/range/multiplexer semantics. Zero difference under the
legal atop domain is an exact full-domain proof, not random sampling.
Every lifetime boundary and complete reverse restoration is checked.
The BDD engine's own four-bit add/order/equality truth tables are exhaustive.

Dispatch: 80 configurations (five recipes, A and W independently 1–4),
**47,224,025,124,923,801,776,800 tuples** total; each configuration has
2^(A+2^A*W+W+1) tuples, including ambient control.
Scans: 32 configurations per recipe (P and L independently 1–4; C=1/2,
T=1/2). Each helper covers **6,239,116,800 tuples**; each row covers
**99,825,868,800 tuples**. Four recipes total **212,129,971,200 tuples**.
The exact domain per helper is (C+1)*2^(P+L+2CP+2C+2); the rows replace
an arbitrary predicate target by clean flag plus arbitrary faulted/code,
giving exponent +4. Largest dispatch configuration is 2^73 basis tuples.

### Structured bodies

Generic memory commit aa9448f; 54 Rust tests and harness 445/0/63.
Six structured recipes: qif, qloop, qloop.while, qpred.scope, ctl, adj.
Predicate scopes group matching qpred/qunpred pairs as §3.5 lexical scopes.
Bounds 0,1,2,3,17 at widths 1–4, full input domains, including every
17-bit while condition vector and ambient control: 7,865,220 tuples per
loop recipe. qif and nested fixpoint compositions each add 120 tuples.
Exact Q(√2,i) witnesses: 270 coherent scope/qif/adjoint matrix columns,
plus 600 Hadamard-loop columns (qloop bounds through 17; while bounds 0–3).
The large while bound has an exact permutation witness over its entire
basis; non-permutation leaf matrices and coherent scoped composition are
checked independently, not called basis permutations.
168 total recipes. Step/construction are blocked on b-isa-spec-lph:
whole-body capture-free splices cannot select heterogeneous row templates
or rebind the current frame. Generic call/activation interfaces remain
b-isa-spec-1a4; allocation/class outputs b-isa-spec-3xe; images afi.

### Ratified corrections, control rows, allocator and aliases

Current count before the native family: 178 recipes. Added qjump/qbranch/
qjumpr and successor.literal/flagged/dynamic, plus lift.alloc and positional
copy/exchange/select adapters. Branch/successor witnesses: 22,120 tuples;
qjumpr: 138,880 tuples over every nonempty function interval at widths 1–4.
Wide effective-cause witnesses: 14,592 tuples at F=4,5,8. Cause C=0 maps
to 4, including under faultif; core fault tests now cover all C=0..15
(3,072 tuples). These supplement, not replace, the available 0.8 harness.

All four allocator assignment values precede all four exchanges; inverse
restores atop before addressing table[atop]. Eight exact BDD configurations:
N=1–4, H=N+2, P=max(N+2,5), capacities 2/4, T=2/3, Base=16,
Limit=2^N, F=4; **1,512,125,731,994,490,556,645,376 tuples**. Domain per configuration:
(C+1)*2^(N+H+C*(2P+2)+P+F+2), legal atop 0..C. Packed entries are
base[P], size[P], live, pinned. E=max(H,N)+1; K=min(P,H), J=min(P,N).
These are derived width parameters; successful allocation sizes fit P.
Selectors may be wider than log2(C): only the C physical entries are
scanned, and bounds are checked before mutation.

Pinned free entries are excluded from both the validity predicate and the
mutation match (0.9 E17.13). Division computes frame.flag before any value
slot write (§3.6), then success-only assignment and fault writes. F≥4 is
an explicit parameter: the complete F=8 division circuit equals the F=4
circuit and touches no high code bit, proving exact tensor identity on all
high inputs and an additional **19,169,280** tuples. Both regressions were
observed red before their fixes. Strict cast widths exclude M=N (the old
value-helper checkpoint also tested identity-width mathematical extensions).
Updated core assignment/fault witness count: 1,318,032 tuples; repeated
location/destination aliases add 540, positional copy/exchange 1,360.
Input captures are distinct, so aliasing location reads cannot create an
invalid coincident-control CCX. All declared library scratch is balanced;
every release is now checked against reverse allocation order. Diagonal
and both swap targets obey §3.3 effective-control disjointness.

Additional blockers: b-isa-spec-1oa (typed mapped-region layout descriptors
for arbitrary lift.load/store); b-isa-spec-82x (qinplace's printed t/f
release order conflicts with §11.1 reverse allocation order). Neither is
silently replaced by a partial specialization. Complete-library hash/count
remain withheld while these and afi/1a4/3xe/lph are open.

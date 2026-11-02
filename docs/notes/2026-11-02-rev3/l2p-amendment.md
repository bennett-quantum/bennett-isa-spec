# l2p amendment: private measurement cleanup with checked relocation

Council-ready proposal for `b-isa-spec-bwm` (P1); publication and implementation
remain open. This note changes neither the published specification nor the
reference interpreter. The baseline is [ISA 0.9](../../../spec/Bennett-ISA-Spec-0.9.md),
through K84. Each BEFORE block below is an exact, unique span of that baseline;
each AFTER block is its complete replacement. Text outside those spans stays
as published, except the explicit version migration in §A. The checked patch
anchors make this a surgical amendment, rather than a replacement specification.

## A. Decision, release and scope

Accept the core as proposed **K85, PRIVATE-MEASUREMENT-CLEANUP**, or the next
unallocated K number at acceptance:

| Entry | Clauses | Semantic change | Tracking | Publication |
|---|---|---|---|---|
| K85 (proposed) | §§6.2, 6.4, 8.3, 11–12, 14–15, 17 | Admit uncontrolled `measure_fixup`, including delayed cleanup after its source window closes; verify the whole realised channel against legal source; bind private events, timing, resource charges and device certificate scope | b-isa-spec-bwm; council B1/B2 and L2P-1/L2P-2 | 0.10 |
| Separate optional entry | §§6.1, 11/K81, 12.1 | Exact clean-isometry boundary may establish `perm` provenance; no change to the emitted-gate permutation theorem | Efficiency only; soundness review | May accompany 0.10; not a prerequisite for K85 |

**Release choice: 0.10.** [spec/README](../../../spec/README.md) requires a full
version bump for semantic changes. Publish the core with ISA/object/sched/cert/
target `0.10`, launch/coverage `0.9` (their ABIs are unchanged), and `ref@0.10`.
The schema substitutions below are expressed against sched/cert/target 0.9 so
every old refusal and reserved `mbu` flag is accounted for. Their new fields
belong to the 0.10 schemas, never silently to an old 0.9 consumer. The new
object header carries this seven-component version vector; unchanged signature
ABI strings remain `0.9`. Section headings referring to these five changed
formats become 0.10; inherited historical examples remain explicitly 0.9.
This is the mechanical version migration, in addition to the exact semantic
replacements below. The publisher must generate actual new envelope hashes;
this note does not invent them.

A backported 0.9 semantic K repair would require an explicit council exception
to that version rule and a separately negotiated schema/expansion profile. It
is not the default proposed here. Keeping 0.9 published is compatibility, not
resolution: **b-isa-spec-bwm remains a current North Star blocker until the
repair is accepted, published and implemented by the required consumers**.
A future-version label alone does not discharge the promise or permit a
claim of complete North Star support with only 0.9.

The required capability is the uncontrolled measurement choice promised by
[North Star](../../../../bennett/docs/Bennett-NorthStar.md) §§4.2, 5 and 5.1.
The previous note overstated §9.6 (section 9, item 6): it identifies controlled
measurement cleanup as an open question. Under coherent control, unitary
cleanup remains the sanctioned fallback. This amendment preserves that
fallback and does not make controlled measurement mandatory. The optional
`perm` provenance rule below is an efficiency extension, judged on soundness;
North Star §5.1 still classifies the actually emitted H/T/measurement circuit
outside its permutation tier.

The mixed fat object remains reachable: an ordinary native host binary can
embed ISA objects for only marked modules and their linked device closures.
Sections D and E bind cleanup claims to those device closures. The standing
[shared scoping statement](../2026-11-02-rev2/fat-object-scope.md) is part of
this proposal's scope; neither this version migration nor an unimplemented
launch boundary requires whole-binary lowering.

## B. Core amendment: source validity and cleanup

### B1. §6.2: preserve legal source pairing; authorise checked relocation

Keep the first window-definition paragraph. Replace the remaining paragraphs:

<!-- before:6.2 -->
```text
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
```

<!-- after:6.2 -->
```text
Source cleanup is bound to its exact computing sequence and requires, at
its original source point, the external input versions that sequence
read (the versions in S for a perm value), and the version of g it
produced, to be current; g's pre-computation target versions need not be
current. Source computation and cleanup have the same effective control
set and control-wire versions. Every perm workspace value has a legal
source cleanup within its window and before its region exits. An other
value's source cleanup is the exact computing adjoint and is legal only
while the §6.1 outstanding-dependence condition holds. Source programs
that fail these conditions remain ill-formed.

reverse and recompute retain these version, window and control
conditions at realisation. recompute is legal only for perm values.
measure_fixup is a checked implementation of a well-formed source pair,
not a reinterpretation of an invalid late source adjoint. K81 pairing
and the source-version checks are performed at the original legal
source point before selecting this realisation.

A measure_fixup witness MAY delay the paired physical cleanup beyond
its source window, across the explicitly identified intervening source
operations. It binds the original K65 carrier, replaced source
occurrences, later realised location, actual operand versions and
extended workspace lifetimes. Only the dependency edges explicitly
listed by the witness are replaced; all other ordering, aliasing,
control, region-cell and lifetime rules remain mandatory. Operations
other than the replacement events MUST NOT access a source-released
lifetime. The checker validates the complete realised region channel
against the original well-formed source region under §6.4 and §12.1,
with every interface wire, region cell, borrowed system and external
correlation included. A closed source window at the realised cleanup
location does not prohibit that checked implementation. Workspace MUST
be reset and released before the realised region exits.
```

### B2. §6.4: complete validity condition

<!-- before:6.4 -->
```text
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
```

<!-- after:6.4 -->
```text
Cleanup realisations are reverse, recompute and measure_fixup. reverse
is the computing sequence's exact adjoint. recompute retains §6.2's
perm-only rule: recompute into fresh clean workspace, XOR into the
original, uncompute the fresh computation and release its workspace.

measure_fixup implements a source pair satisfying §6.2. It measures
only designated private allocated workspace, performs outcome-indexed
fixup, resets that workspace and releases it. It is admitted when the
effective coherent control context of the replaced pair is empty.
The algorithm's own quantum data controls (for example a and b in an
AND) are included in the channel subject; they are not an inherited
control context. Under any nonempty inherited or local coherent
predicate context, reverse or recompute remains the sanctioned fallback
until the controlled measurement contract is published. This is the
controlled-form deferral, not a deferral of uncontrolled measurement.

The default private measurement primitive is computational-basis
projective measurement of one workspace wire. X-basis measurement
expands as H followed by that primitive. For outcome m, reset is X^m
on the measured wire after any required correction. Every basis change,
measurement, conditional gate, reset and release is an explicit bound
expansion event. Outcomes are private classical bits, distinct from
quantum predicates. No outcome or record of it may enter a launch
measurement list, host result, effect tape, hostcall, source control
flow or any other externally visible output. These implementation
events do not create source-level CC measure frames.

Let A initialise all private workspace in its declared clean/image
state, V be the unchanged legal source region's reference isometry,
and K_m be the complete realised branch map after measurement, fixup,
reset and release, for outcome string m. Before deleting any workspace
factor the checker MUST establish its required clean/image restoration
and borrowed-system restoration on the entire declared input space.
Let Phi(rho) = sum_m K_m rho K_m^dagger on the complete retained
interface, including borrowed systems and inherited controls when
checking a containing region. Verify sum_m K_m^dagger K_m = I and the
§12.1 channel bound ||Phi - (rho -> V rho V^dagger)||_diamond <= e
for every actual invocation context. e is the checked channel error
charged under §12.3 and must fit the bound schedule accuracy. Exact
identity has e=0 and uses the exact route in §12.1. Restoration and
privacy are exact obligations, not consequences of an approximate
interface bound. Basis values or ancilla-zero alone are insufficient.

Every outcome string is included, including a branch of exact zero
probability. Each branch correction is checked for the actual
intervening operations and realised operand versions. In the special
case of a phase D followed by an intervening unitary U on retained
systems, the transported correction is U D^m U^dagger. This formula
does not replace channel verification for more general intervening
operations or multiple cleanups. Witnesses for several interacting
cleanups are checked in their composed region, not independently under
incompatible assumptions.

The witness MAY also bind a replacement of its owned source computing
sequence on its proved clean-input subspace. That replacement MUST have
the same exact clean-boundary isometry, including amplitude +1, on
every input column and leave all other wires unchanged. It is checked
as part of the whole-pair realisation and whole-region channel proof;
it grants no source perm provenance and is not a general target gate
scheme on dirty inputs. With no such replacement, source computation
is unchanged. No source occurrence may be replaced by two witnesses.

An adjoint invocation or reversible run uses a separately checked
realisation of the logical inverse channel with fresh private outcomes;
it does not reverse a measurement record. Exact unitary-equivalent
interface channels satisfy the reversible interface. Approximate
channels compose with their declared diamond bounds. Explicit source
measure continues to consume its register and bar reversal.

Schedules and certificate cleanup entries remain keyed by (region hash,
variant, inherited control count). Every reachable nonempty control
context has a unitary cleanup schedule for its key. A containing region
may itself be under control only if every affected cleanup is selected
for that actual context; an uncontrolled witness cannot be reused there.
```

The core can use legal permutation source `CCX(a,b;g); CX(g;y);
CCX(a,b;g); release(g)`. Its paired implementation can compute by the
four-T circuit C in §F and measure its cleanup, after the exact clean-boundary
and full-channel checks. It does not need the optional source-provenance
promotion. An implementation that keeps the source CCX compute can still use
measurement cleanup; its compute cost is the cost of that chosen CCX.

## C. Core amendment: target and schedule contract

### C1. §8.3: Boolean mbu, explicit private primitive cost

Replace the capability schema span:

<!-- before:8.3-capabilities -->
```text
  "capabilities": { "measurement": {"mid_circuit": bool, "feedforward_latency": n}, "mbu": false,     /* reserved, §6.4 */
                    "max_parallel": n | "unbounded", "max_predicate_depth": n | "unbounded", "classical_step_time": n },
```

<!-- after:8.3-capabilities -->
```text
  "capabilities": { "measurement": {"mid_circuit": bool, "feedforward_latency": n}, "mbu": bool,
                    "max_parallel": n | "unbounded", "max_predicate_depth": n | "unbounded", "classical_step_time": n },
  "private_cleanup"?: { "measure_z": { "cost": { "<resource>": n, "time": n } } },
```

Insert before the existing matching paragraph, retaining its first line:

<!-- before:8.3-matching -->
```text
**Matching.** An executed gate with opcode o, operand classes, **total
```

<!-- after:8.3-matching -->
```text
capabilities.mbu is a Boolean, default false when omitted by an older
target profile. mbu=true requires measurement.mid_circuit=true and a
private_cleanup.measure_z cost record. That record is required only
for mbu=true; unknown keys reject. Every declared resource has an
explicit nonnegative finite cost there (zero is allowed); time is
required. These costs are numeric constants, with no unbound cost
variables. measure_z is an exact private implementation event, not a
§4.4 gate and not a source measure instruction. Reset is an ordinary
reserved outcome-conditioned X gate; basis changes and fixups use
ordinary matched gates/schemes, their topology and their costs.
Private outcome dispatch is charged classical_step_time once per
measured bit, after measurement and before dependent gates. It emits
no host effect. feedforward_latency is an additional minimum delay,
not a substitute for that dispatch event or for any reset/gate cost.
A target with mbu=false still supports unitary cleanup. A schedule
requesting measure_fixup on it is E_SCHEDULE_INFEASIBLE; the executor
does not silently replace the requested implementation. mbu=true does
not itself prove the correction or enable measurement under coherent
control. Existing targets with mbu=false need no new cost record.

**Matching.** An executed gate with opcode o, operand classes, **total
```

### C2. §11 generated operations: explicit exception and expansion gate

<!-- before:11-expansion -->
```text
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
```

<!-- after:11-expansion -->
```text
**Generated operations and the expansion library.** Every row and
instruction expands into generated operations (gates, allocations,
releases). Source order is fixed by §3.6, §4.2, §4.5–§4.6 and §5.2;
only the checked measure_fixup transformation of §§6.2/6.4 may replace
the specifically recorded source-order dependencies for its owned pair.
The expansion library is a versioned document bound by
bindings.expansion = {name,version,hash}. It defines every instruction
and row expansion, its ordinals, scratch identities and removable set,
and, when supported, the private cleanup-event constructors below.
Unselected source operations retain their semantics, identities and
dependencies. A consumer lacking the exact bound library or event
profile rejects with E_SCHEDULE_INFEASIBLE. Two consumers implementing
it execute and charge the same reserved event set. Other §5.7
transformations retain ordinals and remove only library-marked leaves.
```

Replace the cleanup payload in the schedule schema:

<!-- before:11-schema -->
```text
                   "cleanup": { "realisation": "reverse" | "recompute" },
```

<!-- after:11-schema -->
```text
                   "cleanup": { "realisation": "reverse" | "recompute" | "measure_fixup", "witness"?: "<id>" },
```

Add the definition map beside interface placement in the schedule schema:

<!-- before:11-witness-map -->
```text
    "interface_vertices"?: [ [v, …], … ],
```

<!-- after:11-witness-map -->
```text
    "cleanup_witnesses"?: { "<id>": { "contexts": [ <D3 context record>, … ] } },
    "interface_vertices"?: [ [v, …], … ],
```

`witness` is required exactly for `measure_fixup` and is forbidden on the
other two kinds. It names one definition in `cleanup_witnesses` and the
matching certificate record in D3. The definition map commits all event,
placement, version, lifetime and dependency bytes to `schedule_hash`. It
contains no claim, certificate hash or channel subject. The certificate
repeats the same definition and adds its channel-claim index; the checker
requires byte-identical JCS `contexts` values. Thus the schedule fixes the
event set and the certificate can bind `schedule_hash` without a hash cycle.
The map is required for any selected `measure_fixup`; otherwise it may be
absent. Missing referenced definitions and conflicting duplicates reject.
Instantiated events derive deterministically from that bound definition and
the expansion profile and are checked before execution.

Replace Timing and Obligations:

<!-- before:11-timing -->
```text
**Timing**: a `start` time
MUST be ≥ the end of every operation the operation depends on (data,
wire lifetime, region cell version, effect order); an entry violating
this makes the schedule invalid. **Obligations**: satisfy §6; respect
capacity; report every resource; be reproducible from the request plus
scheduler version.
```

<!-- after:11-timing -->
```text
**Timing**: a start time MUST be at least the end of every predecessor
in the validated dependency graph (data, lifetime, region-cell version
and effect order). measure_fixup replaces only the explicitly bound
edges authorised by its whole-region channel check. Its graph MUST be
acyclic. A predecessor is an actual emitted event, never the
non-emitting cleanup carrier. A dependency on a measurement outcome
also requires start >= measurement end + feedforward_latency and
completion of the outcome-dispatch event. Absent explicit start
entries, emitted events execute sequentially in bound-expansion order
with those minimum delays. Invalid timing is E_SCHEDULE_INFEASIBLE.

Conditional slots are reserved events. Each retains its ordinary gate
duration, wire occupancy, resource cost and error reservation on every
outcome; a false condition performs the identity in that reserved slot.
Every private measurement and dispatch is charged, and workspace live
intervals extend through reset and release at their realised positions.
Resource counts, makespan, occupancy, peak, capacity and topology use
this padded event set and the extended lifetimes. Outcome-dependent
renumbering, early release or resource discounts are invalid.

**Obligations**: satisfy §6, every required channel check, target
capabilities and capacity; report every resource; reproduce the same
reserved events and graph from the request, scheduler version,
bound schedule definition and expansion. A missing/unverified cleanup
witness makes the requested schedule infeasible before its effects.
```

### C3. K65 and K80: ownership, coordinates and relocation

Replace K65's selection paragraph:

<!-- before:11-K65 -->
```text
Resolve the unique maximal cleanup entry or the reverse default before
expanding that cleanup, and apply its reverse/recompute realisation to
the entire paired cleanup, subject to §6's validity conditions. [deps: ISA-11-P6.S1, ISA-11-P6.S7, ISA-11-P6.S8, ISA-6.2, ISA-6.4]
```

<!-- after:11-K65 -->
```text
Resolve the unique maximal cleanup entry or the reverse default before
expansion. reverse/recompute apply to the entire K81 pair's cleanup.
measure_fixup binds that same original carrier and legal source pair;
its witness may replace the owned computing sequence on the exact
clean subspace and delay the physical cleanup under §§6.2/6.4.
Source pairing, captured source versions and inverse ownership are
unchanged by selection. The later insertion point is not another
carrier and cannot acquire the inverse occurrences a second time.
```

Replace the complete K80 normative paragraph; its existing multirow reverse/
recompute witness remains valid:

<!-- before:11-K80 -->
```text
**Cleanup leaf ownership (K80).** For `reverse`, emitted leaves retain the
written cleanup's source points and per-point ordinal and semantic-call
occurrence spaces, including the terminating release. For `recompute`,
consumers MUST assign the entire replacement's caller-level leaves and
call edges to the carrier's point and coordinates. The original
computation is unchanged; replaced inverse rows emit nothing. Materialise
fresh computation, XOR, fresh uncomputation and releases in §6.4 order;
number caller leaves contiguously from zero, excluding callee subtrees,
and number semantic-call occurrences independently from zero (§11.1/K63).
Fresh uncomputation has separate occurrences from fresh computation;
neither reuses the original computation's edges or scratch identities.
Callees retain their own source points, nested chains and ordinal/call
spaces under these fresh edges. Internal replay does not import the
original rows' source-qloop coordinates. Raw inverse traversal retains
forward paths, leaf ordinals, call occurrences and scratch identities;
step/phase follows §11 and source-adjoint formation remains K62. The carrier's ordinal 0 belongs only to
cleanup-kind matching; it does not consume emitted-leaf ordinal 0.
K64's oracle support partition is unchanged.
```

<!-- after:11-K80 -->
```text
**Cleanup leaf ownership (K80).** reverse retains the written cleanup's
source points, per-point ordinals and semantic-call occurrences.
recompute assigns all replacement caller leaves and call edges to the
original carrier's coordinates, in fresh-computation, XOR,
fresh-uncomputation, release order. Replaced inverse rows emit nothing.
Number caller leaves from zero excluding callee subtrees, and semantic
calls independently from zero. Fresh replay has fresh scratch and
call occurrences; callees retain their own points beneath those edges.
It does not import original source-qloop coordinates.

For measure_fixup, the original source computing sequence is unchanged
unless compute_events is supplied. Such replacement compute events
belong to the original computing points; their caller ordinals and
calls are numbered from zero within each replaced point in event order.
All measurement, dispatch, fixup, reset and delayed-release caller
events belong to the original cleanup carrier's chain, point,
iteration, instance, step and phase; number their leaves contiguously
from zero in declared reservation order. Their actual placement is
specified separately by after and dependencies. The carrier remains
non-emitting; its cleanup ordinal 0 does not consume emitted ordinal 0.
Outcome choice never renumbers a slot, removes its reservation or
changes a context key. Callee subtrees, if any, keep their own spaces
under fresh carrier edges. No source occurrence has multiple owners.
The unchanged K81 partition still excludes private constructor cleanup;
this amendment does not silently create carriers for those expansions.

Raw inversion of a unitary expanded trace retains forward coordinates,
call occurrences and scratch identities; step/phase follows §11 and
source-adjoint formation remains K62. A measurement trace has no raw
physical inverse. Its logical inverse is separately expanded, bound
and channel-checked under §6.4, retaining source occurrence identity
but using its own reserved events and fresh outcomes. K64 support
partitioning and all unchanged operation coordinates remain intact.
```

K81's source partition itself is unchanged in the core. In particular,
`qcopy; H; quncopy` is not made legal by a schedule selection. The permitted
source is `qcopy; quncopy; H`, with the physical cleanup delayed by this rule.

### C4. §11.1: new private-event profile, never a silent ref@0.9 edit

Keep the historical catalog statement, append the new publication contract:

<!-- before:11.1-library -->
```text
**ISA-REF-LIB** (K11; b-isa-spec-8r4, from bennett-vm-47t).
This is the decided 0.9 contract. The catalog bound as ref version 0.9
is the file `spec/ref-0.9.json` whose four-line envelope hashes to
`5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197`.
It publishes the complete 219-recipe catalog, byte-identical to the
accepted catalog snapshot.
```

<!-- after:11.1-library -->
```text
**ISA-REF-LIB** (K11; b-isa-spec-8r4, from bennett-vm-47t).
The historical ref@0.9 catalog is spec/ref-0.9.json, with 219 recipes
and envelope hash
5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197.
Its bytes, constructors and meaning are unchanged. It cannot implement
the new private cleanup events. Existing 0.9 consumers retain that
binding and reject an unimplemented expansion library.

The 0.10 publication MUST publish ref@0.10, its complete recipe catalog,
the private-cleanup event profile below and its actual envelope hash.
Use the existing four-line library envelope with isa=0.10 and
version=0.10; the event-profile definitions are part of the versioned
document's semantic contract. Its canonical document MUST include
event_profile:"private-cleanup-1". No missing hash is a wildcard.
Ordinary unitary recipes retain their existing algorithms unless a
separate accepted change explicitly replaces them.

The private-cleanup-1 event union is:
gate {op,wires,when?}; measure_z {wire,outcome};
dispatch {outcome}; release {wire}.
gate.op is a §4.4 gate with its usual parameter encoding (params is
required for a parametric gate and absent otherwise); wire operands
are ordered resolved wire/version records. measure_z has no quantum
controls, emits one exact outcome bit, and uses the target's private
measurement cost. dispatch consumes that bit and costs one
classical_step_time. when, if present, is {outcome,equals:0|1}; it
selects an ordinary gate or the identity in its fully charged slot,
and cannot gate allocation, measurement, dispatch or release.
Outcome names are unique within a witness and cannot escape it.
when is classical feedforward and adds no coherent control to target
gate matching or the region's inherited control count.
Reset is the explicit gate X with when:{outcome,equals:1}.
Allocation remains the existing clean allocation constructor and
must be declared in the ordinary frame/event stream; no undeclared
wire is created by a cleanup event. Each event has its stable id,
source coordinates and resolved operands under §11. No constructor
means source-level CC measure or changes the unitary gate registry.
Unsupported event-profile versions are E_SCHEDULE_INFEASIBLE.

The exact JSON encoding is the union above with an event discriminator
(`"gate"`, `"measure_z"`, `"dispatch"`, `"release"`) and the corresponding
fields; no additional fields except the universal `id` are allowed. `params`
is the gate-only optional field just specified. Source coordinates derive
from C3 and are not producer-selected event fields. `compute_events` below
wraps each event with its replaced source point so several computing points
can be distinguished. A release event consumes a lifetime only after its
exact reset proof succeeds. A fixup requiring a controlled gate uses that
gate's ordinary quantum controls in `wires`; `when` is solely classical.
```

## D. Core amendment: channel claims, domains and certificate layout

### D1. §12.1 channel row and exact route

<!-- before:12.1-channel -->
```text
| `channel` | `{ "subject": as above, "metric": "diamond", "precision_bits", "bound", "dual": [ { "Y": interval Hermitian matrix, "mu": decimal } … ] }` | for every invocation context of the subject, in enumeration order, verify its dual witness for the **diamond-norm SDP** below in interval arithmetic and `mu` ≤ `bound` (one witness per context; too few witnesses: reject) |
```

<!-- after:12.1-channel -->
```text
| `channel` | `{ "subject": as above, "metric":"diamond", "bound":decimal, "proof"?:"dual"|"exact", "precision_bits"?:n, "dual"?:[ {"Y":interval Hermitian matrix,"mu":decimal} … ], "witness"?:{"rule":"matrix_units"} }` | every actual invocation context: build the complete realised channel and legal source channel independently; default proof `dual` uses the existing rigorous SDP rule with required precision_bits and dual and no witness; proof `exact` requires bound 0, witness rule matrix_units, no precision_bits/dual, and the exact matrix-unit and restoration checks below; a failed or incomplete check discharges nothing |
```

Insert this exact route immediately before the existing SDP paragraph:

<!-- before:12.1-exact -->
```text
**Diamond-norm SDP** (Watrous, *Theory of Computing* 5 (2009), §4; the
```

<!-- after:12.1-exact -->
```text
**Exact channel identity.** proof:exact with witness rule matrix_units
requires the checker to construct every branch K_m and reference V
from the bound source, schedule, target and expansion, never from a
producer-supplied result matrix. On every interface matrix unit E_ij
it checks sum_m K_m E_ij K_m^dagger = V E_ij V^dagger exactly. The
matrix units range over the full interface and all borrowed systems;
workspace starts in its declared clean/image state, and its exact
restoration is checked before removal, on every branch, including
zero-probability branches on their support. Check trace preservation,
complete outcome coverage, privacy and every §6.4 side condition.
Exact arithmetic or an independently checked compositional exact
derivation must establish these equalities; floating-point agreement
is not an exact proof. This route has diamond bound zero and does not
use the strictly-positive-pivot SDP test. It checks off-diagonal as
well as diagonal matrix units, hence arbitrary external entanglement.
The rule name matrix_units denotes these obligations; an algebraic
checker may factor them compositionally without materialising them.
An uncompleted check is unchecked, never proof. The numerical dual
route remains available for nonzero error and keeps all its existing
strict positivity and interval requirements; Y=0, mu=0 still fails
that route. No exact-route permission weakens its positivity test.

**Diamond-norm SDP** (Watrous, *Theory of Computing* 5 (2009), §4; the
```

Insert device scope immediately before invocation-context enumeration:

<!-- before:12.1-scope -->
```text
**Invocation contexts.** A subject denotes the set of invocations of
```

<!-- after:12.1-scope -->
```text
**Mixed fat-object subjects.** A mixed fat object may retain a native
host binary and embed ISA objects only for marked device modules and
their linked device closures. A channel subject identifies a region
closure in one selected embedded device object; its complete device
interface and device callees, with external correlations, are covered.
Native main, argv/envp handling, unmarked native libc and ordinary host
effects stay native and require no ISA/channel certificate inventory.
The launch inventory associates the selected exported device root,
device object, schedule and certificate. A native comparison claim
uses the corresponding materialised marked entry and materialised
callee closure, layout and captured launch inputs, not unrelated host
code. Private cleanup outcomes are absent from host results and launch
measurement records. Each selected device root has its own bindings
and concrete invocation-context set, including default-only calls;
[] is relative to that root, never to native process entry.

**Invocation contexts.** A subject denotes the set of invocations of
```

### D2. §12.2: quantum claims are not basis-slice claims

<!-- before:12.2-domain -->
```text
A `proof` label requires the domain to equal D; otherwise the claim is
`evidence` with the covered fraction.
```

<!-- after:12.2-domain -->
```text
For exhaustive basis claims, a proof label requires the domain to
equal D; otherwise the claim is evidence with the covered fraction.
For channel claims the domain is the full operator space of the
declared interface, inherited controls and borrowed systems, for every
§12.1 invocation context. domain slicing, omitted-component zero
defaults and sampled basis replay do not restrict that quantum proof
obligation. Workspace retains its required clean/image initial state;
it is not an arbitrary dirty input. Every private outcome is included.
Exact matrix-unit verification assigns proof only after every equality,
restoration and context obligation succeeds. Host limits on exact or
dual checks produce unchecked (host limit), identifying the context
and limiting operation; they do not make the legal channel or target
capability ill-formed and discharge no execution/proof obligation.
```

The remaining basis-domain rules, including K69–K75, retain their existing
meaning. In particular, diagonal basis tests cannot discharge a channel claim.

### D3. §12.3 cleanup record, bindings and validation

<!-- before:12.3-cleanup -->
```text
            cleanup: { "<variant>:<inherited_controls>": [ { path, realisation } ] },
```

<!-- after:12.3-cleanup -->
```text
            cleanup: { "<variant>:<inherited_controls>": [ { path, realisation, witness? } ] },
```

Insert the following definitions before `versions`:

<!-- before:12.3-layout -->
```text
`versions` is the object `{ "isa", "object", "launch", "sched", "cert",
```

<!-- after:12.3-layout -->
```text
For reverse/recompute the cleanup entry remains {path,realisation};
witness is absent. For measure_fixup, witness is required and has:
{ id, contexts: [ { context, carrier, compute_points, cleanup_points,
                   compute_events?, after, measured, outcomes, events,
                   versions, lifetimes, dependencies } ... ], claim }.
id is a nonempty string unique within the bound root certificate and
is the schedule cleanup.witness reference. contexts MUST be identical
as JCS bytes to schedule.cleanup_witnesses[id].contexts; that definition
contains no certificate/claim/subject field. claim is a zero-based
index in this owning region record's claims array, pointing to a
channel claim also listed in verification.channel. Its subject has
this region closure, variant, inherited control count and the exact
bound root schedule_hash. One record covers each concrete carrier
selected by the schedule; wildcard schedule selections must resolve
to all of their concrete contexts before verification.

contexts is ordered by the bytewise JCS encoding of {context,carrier},
without duplicate pairs or omissions. Several carriers in one region
invocation have the same context and distinct carrier coordinates.
context is the §12.1 region invocation key,
carrier the complete terminal source coordinates with generated:0,
and compute_points/cleanup_points are the exact ordered K81-owned
source occurrences, not producer-chosen semantically similar gates.
source_point is {path,step,phase}; path is the resolved §11 source
chain with no wildcards, and step/phase are the terminal containing
invocation's concrete coordinates or null when absent. compute_points,
cleanup_points and after use source_point, before leaf expansion.
carrier adds generated:0 to its source_point. A source_leaf is
{point:source_point,generated:n}, naming a concrete bound-expansion
leaf rather than a source occurrence; the two forms are not conflated.
The checker derives this ownership and compares it before expansion.

after is the concrete unchanged source occurrence after whose emitted
events the delayed cleanup begins (or the carrier's source_point for no
delay); no placement may cross the containing region return. All
intervening source operations keep their order and identity. The
checker derives their list between the original cleanup and after.
If compute_events is present it is an ordered list of {point,event}
replacing exactly compute_points, using the private-cleanup-1 gate
events and the checked clean-boundary isometry of §6.4. If absent,
computation remains unchanged. It cannot replace allocation, region
calls with effects or another witness's source occurrences.

measured is an ordered list of {wire,outcome} with distinct private
workspace wires and distinct local outcome names. wire is
{allocation,bit,version}: allocation is {leaf:source_leaf,lifetime:n},
where n is the zero-based lifetime occurrence at that allocating leaf.
bit is the zero-based bit index. version identifies the checker-derived
current value origin: {entry:true}, {source:source_point,output:n}, or
{replacement:event_id,output:n}. output numbers modified output bits
in resolved operand order. A matched inverse restores its captured
origin rather than manufacturing a new unrelated version. Internal
static/SSA numbering is not a portable substitute for these origins.
Interface/borrowed identities instead use {group,slot,bit,version},
and region cells {group,slot,cell,bit,version}; allowed groups and
slots are the existing signature and borrow positions. These three
wire-record forms are disjoint; unknown or aliased identities reject.
outcomes lists every width-|measured| bit string exactly once in
lexicographic order, including zero-probability strings. events is
the complete ordered reservation list from basis changes through
measurement, dispatch, fixup, reset and release under §11.1. Each
event id is unique within the context. Its when clauses define the
branch gate list for each listed outcome; there is no hidden branch
or implicit reset. A bit cannot be read before its measurement and
dispatch. Each measured wire has exactly one measure_z event; its
explicit reset and release follow it. No branch may consume a source
interface, dirty borrowed or live nonworkspace wire by measurement.

versions lists {event,operands:[wire records]} for every replacement
event, in event order, with checker-derived operand versions at its
actual placement. Conditional reservation slots have fixed output
origins, even on an outcome where their gate is disabled; no outcome
renumbers them. lifetimes lists {allocation,source_release,
realised_release} for every delayed private lifetime. Each release
reference is a source_point or replacement event id as appropriate;
fresh/reused physical storage cannot alias an extended lifetime.
dependencies is {remove:[[event_ref,event_ref]...],
                 add:[[event_ref,event_ref]...]}.
event_ref is {source:source_leaf} or {replacement:event id}.
remove lists exactly the original source-derived ordering/version/
release edges made inapplicable by this replacement; edges involving
unreplaced source effects or allowing foreign access to a released
lifetime cannot be removed. add binds the complete replacement data,
outcome, timing and lifetime edges. Both lists are sorted by JCS
encoding, without duplicates, and validated against the independently
derived before/after graphs. The schedule graph is acyclic and all
remaining source edges are preserved. Whole-region channel verification
is required in addition to, not instead of, these structural checks.

The certificate schedule_hash, root invocation args/budget, target_hash
and expansion binding MUST match the schedule. Root/nested argument,
token and budget resolution uses K13 and K63, including default-only
contexts; a root binding cannot substitute for a callee binding. claim
must cover the containing region's complete realised operation, not
just the isolated correction gates. A referenced witness lacking a
completed successful check is not executable. Detached certificates
must be supplied with the schedule before validation; there is no
post-execution trust path and no certificate/schedule hash cycle.

For a mixed fat object the default layout is one device certificate
per embedded ISA object and selected-root binding, with an outer
versioned inventory mapping {module_id, object_hash, entry,
closure_hash, schedule_hash, certificate_hash}. Multiple roots in one
object have separate binding records; a shared callee is checked in
each actual root context. certificate_hash is SHA-256 of the complete
certificate JCS bytes, no trailing LF. module_id is an inventory-local
identifier, not native main; entry names the selected exported device
root. Inventory entries are sorted by module_id then entry, without
duplicates, and cover every device root claimed as executable by that
inventory. The inventory associates records; it grants no new launch
ABI. Native host entry, argv/envp, unmarked libc and ordinary host
effects need no ISA region/frame/channel record. Device owner and
link-proof coverage remains complete for each selected closure.
Native comparison covers the corresponding materialised marked
closure and captured launch inputs. A whole-host quantum certificate
is not required. Private cleanup outcomes never enter the host-visible
measurement inventory. Container identity, when supplied, is separate
from object_hash and cannot replace device closure bindings.

The mixed-launch contract still must publish container/object/root
association, host-entry-to-device-entry marshalling, budgets, opaque
handle lifetime and run/tape association. Host memory by reference
requires explicit address/layout mapping, range/liveness, aliasing,
ownership/synchronisation, writeback and reversal rules. Until that
versioned boundary exists these are bennett-4ht obligations; neither
whole-binary lowering nor omission of mixed execution is a substitute.

The outer inventory is a separate mixed-container record, not an undeclared
field inserted into the existing single-object certificate. Its version is
`"mixed-device-inventory-1"`, with a required `entries` array of exactly the
records above, optionally `container_hash` (SHA-256 of container bytes, defined
outside ISA source identity); all other fields reject. Its association can be
published independently of general host memory/launch mechanics. Certificates
for multiple roots can be grouped in a container, without merging their keys.

`versions` is the object `{ "isa", "object", "launch", "sched", "cert",
```

Replace the controlled-unitary/channel claim requirement:

<!-- before:12.3-metrics -->
```text
`operator_norm` conclusions (T6, controlled unitary
variants) require `operator_norm` bounds throughout. A controlled
unitary variant requires a `numerical` claim; an ε > 0 realisation
compared as a channel requires a `channel` claim.
```

<!-- after:12.3-metrics -->
```text
operator_norm conclusions require operator_norm bounds throughout and
are claims about measurement-free unitary realisations. A controlled
measurement-free unitary variant requires a numerical claim. Every
region containing a private measurement realisation requires a channel
claim, including at epsilon=0, on its complete interface and actual
invocation contexts; exact channel identity may use §12.1's exact
route. A channel-only proof cannot be reused as an operator_norm or
global-phase proof for a controlled unitary. Nonzero-error channel
realisations retain the dual route. Each local/inclusive value is a
verified common upper bound over its key's actual contexts; multiplicity
composition and the once-only operator_norm-to-diamond factor of two
are unchanged. No failure probability is substituted for approximation
epsilon. Reservations count their charged error on every outcome.
```

## E. Core amendment: formal model and defaults

### E1. §14.2 and T4: private outcomes and logical reversal

<!-- before:14.2 -->
```text
The state of a Q region is a vector in ⊗ over live wires in allocation
order (bundles little-endian). `qalloc` tensors |0⟩; `qfree` removes
clean factors; `qborrow` introduces a factor in unknown state possibly
entangled with a reference system R, and the region's operation must be
U ⊗ I on it (§5.6); gates act by §4.4 under the active predicate;
`measure` (CC) is the projective computational-basis measurement with
Born probabilities, consuming the wires.
```

<!-- after:14.2 -->
```text
The reference state of a Q region is a vector in the tensor product
of live wires in allocation order (bundles little-endian). qalloc
tensors |0>; qfree removes clean factors; qborrow introduces an
unknown factor possibly entangled with R, on which the operation
must be identity as in §5.6; gates act by §4.4 under the active
predicate. Source measure (CC) is projective computational-basis
measurement with Born probabilities, consuming its wires.

A private measurement-assisted implementation instead denotes the
channel rho -> sum_m K_m rho K_m^dagger after explicit fixup and
workspace reset/release. Workspace begins and ends clean/image;
private outcomes belong to the implementation environment and are
discarded. Its complete retained interface, borrowed systems and
external correlations are checked under §6.4. It creates no source
measure history frame. Logical reversal uses an independently checked
implementation of the inverse reference channel with fresh private
outcomes, not inversion of physical measurement events.
```

Replace only T4's quantum exactness hypothesis; its classical history theorem
and source-measure/unverified-free eligibility conditions remain:

<!-- before:14.3-T4 -->
```text
If, in addition, every region call is undone
  by the exact inverse of its realised forward operation (ε = 0
  realisations, or a realisation that is applied as its own adjoint),
  the quantum state of every register the run touched is restored
  exactly; otherwise to within the sum of the realisations' declared
  bounds (§12.3).
```

<!-- after:14.3-T4 -->
```text
  If every region call is undone by an exact inverse of its realised
  interface operation, or a separately verified exact channel for the
  inverse of its exact unitary-equivalent interface channel (§6.4),
  the quantum state of every register touched by the run is restored
  exactly, including external correlations; otherwise restoration is
  within the sum of the declared channel bounds (§12.3). Verified
  private cleanup events do not create a measure frame and alone do
  not remove reverse eligibility. Explicit source measure and an
  unverified free retain their existing prohibitions.
```

### E2. §17 items 2, 6, 7 and 9

<!-- before:17-2 -->
```text
2. **Gate-name registry.** Default: the names of §4.4 only; an unknown
   gate name is a load error.
```

<!-- after:17-2 -->
```text
2. **Gate-name registry.** Default: the unitary gate names of §4.4 only;
   an unknown gate name is a load error. The private measurement,
   dispatch, outcome-conditioned gate and reset events of the bound
   expansion profile are realisation events, not new source opcodes
   or additions to that gate registry.
```

<!-- before:17-6 -->
```text
6. **Measurement-based cleanup** (§6.4): the normative fixup, its
   controlled variant and its certificate kind. Default: unitary cleanup
   (`reverse`); `capabilities.mbu` is `false`.
```

<!-- after:17-6 -->
```text
6. **Cleanup policy and controlled measurement.** Uncontrolled
   measure_fixup, including checked delayed cleanup, is admitted by
   §§6.2/6.4 with private outcomes, transported correction, reset,
   target mbu=true and a complete channel claim. The scheduler may
   choose it or unitary cleanup. With no selected cleanup entry the
   deterministic default remains reverse; this is not a ban on the
   measurement alternative. mbu is a target Boolean, not universally
   false. The coherent-controlled measurement form remains open;
   until its contract is published that context uses unitary cleanup.
```

Append a new-version binding after the existing item 7's exact 0.9 binding:

<!-- before:17-7 -->
```text
   For a 0.9 object this is exactly
   `{"name":"ref","version":"0.9","hash":"5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197"}`,
   equally for omission and an explicit binding. [deps: ISA-11.1-P3, ISA-11.1-P4, ISA-11.1]
```

<!-- after:17-7 -->
```text
   For a 0.9 object this is exactly
   `{"name":"ref","version":"0.9","hash":"5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197"}`,
   equally for omission and an explicit binding. [deps: ISA-11.1-P3, ISA-11.1-P4, ISA-11.1]
   A 0.10 object instead defaults to ref@0.10 at its published complete
   envelope hash. measure_fixup requires an explicitly implemented
   private-cleanup-1 expansion profile; ref@0.9 cannot be selected for
   it. Missing publication or support is never treated as a wildcard
   hash or as permission to mutate historical catalog bytes.
```

Append deterministic conditional-slot counting to item 9:

<!-- before:17-9 -->
```text
   Omitting expansion on a 0.9 object selects the full-library hash in
   item 7, not the single-qxor witness hash. [deps: ISA-11.1-P3, ISA-11.1-P4]
```

<!-- after:17-9 -->
```text
   Omitting expansion on a 0.9 object selects the full-library hash in
   item 7, not the single-qxor witness hash. [deps: ISA-11.1-P3, ISA-11.1-P4]
   For private measurement cleanup N includes every reserved ordinary
   gate slot in the selected bound expansion, with the same exact-native
   test, whether or not that slot executes a gate on a particular
   outcome. Exact private measurement and dispatch events do not add
   to N. Error shares, charged bounds and reservation identities are
   independent of sampled outcomes.
```

## F. Exact worked witnesses and costs

All witnesses below were checked using **SymPy 1.12 exact arithmetic in
Q(sqrt(2), i)** by [verify_l2p.py](verify_l2p.py), not floating-point simulation.
The executable defines every gate, computes full columns, constructs branch
maps including reset and compares all interface matrix units. Its integer
wire order is a=0, b=1, y=2, g=3, optional diagnostic q=4; displayed kets here
use the named order `(a,b,g,y)` to match the council report.

Gidney's [primary paper](https://arxiv.org/abs/1709.06648v3) reports four T
resources for temporary AND computation and zero T resources for its erasure.
The [published article](https://doi.org/10.22331/q-2018-06-18-74) distinguishes
the controlled adder's cost. This explicit implementation is derived and
verified here; it is not asserted to be a transcription of the paper's diagram.

### F1. Four-T clean-input compute and dirty-target negative

Define C in time order:

```text
h g; t g; cx a g; tdg g; cx b g; t g;
cx a g; tdg g; h g; sdg g
```

| a | b | C on g=0 | C on g=1 |
|---|---|---|---|
| 0 | 0 | +|0> | -i|1> |
| 0 | 1 | +|0> | +i|1> |
| 1 | 0 | +|0> | -i|1> |
| 1 | 1 | +|1> | -i|0> |

Thus C restricted to clean g is exactly `|a,b,0> -> |a,b,ab>` with
amplitude +1. It is not arbitrary-target CCX. The checker verifies all eight
columns, not just the four clean columns; a general native-CCX substitution
fails. `C; CX(g,y); C†` returns g=0 and implements CCX on arbitrary y,
including coherent inputs. Its literal count is eight T/Tdg gates. That is
not an optimality claim or a count after magic-state recovery; the primary
paper's unitary resource-recovery alternative uses a different accounting.

### F2. Open-window measurement: 4 T total

Let W=CCX(a,b;y), D=CZ(a,b). Before measuring g, the encoded state is
`sum_ab,y alpha_ab,y |a,b,y XOR ab>|ab>_g`. Apply H(g), measure Z(g) with
outcome m, conditionally apply CZ(a,b) when m=1, then X(g) when m=1 and
release g. Before fixup the reduced branch map is `D^m W/sqrt(2)`.
After fixup/reset it is `K_m=W/sqrt(2)` for both m. Therefore
`sum_m K_m E_ij K_m† = W E_ij W†`, including off-diagonal matrix units;
`sum_m K_m†K_m=I`. The outcome distribution is input independent (1/2 each),
and released workspace is exactly zero. This proves preservation of any
external reference, not merely agreement on classical inputs.

The compute uses four T/Tdg gates; H, CZ, measurement, dispatch and X reset
add zero T. All their other target resources and feedforward latency remain
chargeable. For a construction with exactly W-1 independent such carry pairs
and no other T resources, widths W=1,2,3,5 give 0,4,8,16 T respectively.
These are algebraic counts of that stated construction, not an execution of
a sibling arithmetic generator or a claim about all adders.

### F3. Closed-window relocation: legal source and delayed schedule

Use the following legal source; C_src is native exact CCX(a,b;g):

```text
g = qalloc any 1
C_src
cx g y
C_src†
qfree g                     # original legal K65 carrier
h a                         # after: this concrete source occurrence
```

The source interface operation is `V=H_a W`. A core implementation can use
`compute_events` to replace C_src by C, after proving the clean-boundary
isometry. It replaces the owned source inverse/release by this delayed trace:

```text
compute C; cx g y; h a
h g; measure_z g -> m; dispatch m
when m=1: h a; cz a b; h a  # H_a D H_a, in time order H,CZ,H
when m=1: x g
release g
```

Each of the three correction gates is a reserved conditional slot. The
corrected branch maps are `K_m=H_a W/sqrt(2)`. Without the transported
correction, a stale CZ gives `D H_a D W/sqrt(2)` on m=1 and fails the channel.
For input `(a,b,g,y)=(0,1,0,0)`, the correct final vector is
`(|0,1,0,0>+|1,1,0,0>)/sqrt(2)`.

The literal sequence `C; CX(g,y); H_a; C†` is a different operation, yielding
`(|0,1,0,0>+i|1,1,1,0>)/sqrt(2)` on that input. It leaves g=1 with
probability 1/2 and is not the reference. With the optional rule in G,
the source may spell C and C† literally at the original legal points;
the late literal adjoint is still invalid. This distinction closes council
B1/L2P-1 without changing source bytes merely to mean a different ordering.

### F4. Minimal relocation without optional provenance

Legal source: `d=qcopy a; quncopy a d; h a`. Its interface operation is H.
Delayed realisation: `d=qcopy a; h a; h d; measure_z d -> m; dispatch m;
when m=1: x a; when m=1: x d; release d`. Since `H Z^m H=X^m`, each
corrected branch is `H/sqrt(2)`. Input zero yields `|+>_a|0>_d`.
Literal `qcopy; H; quncopy` instead entangles the two wires and leaves
d=1 with probability 1/2. This core example uses only existing `perm`
source rules and proves that G is not necessary for the closed-window repair.

### F5. Controlled diagnostic and logical inverse

As a diagnostic outside this amendment's admitted measurement context,
coherently controlling the compute by q encodes `q*a*b` in clean g.
The outcome-one correction would be CCZ(q,a,b), not an unconditional CZ(a,b).
Both q=0 and q=1 sectors produce random X-measurement outcomes even though
the q=0 workspace remains zero before H(g). The exact checker confirms the
CCZ branch identity on all q values and rejects missing q in the correction,
including off-diagonal q matrix units. It does not infer a zero-T controlled
CCZ correction, publish a controlled measurement contract, or treat that
diagnostic as conformance admission. Under control the implementation uses
the sanctioned unitary fallback with its separately matched costs.

For the uncontrolled open-window operation W=W†, a fresh checked measurement
implementation can implement its logical inverse. For the delayed operation
H_a W the inverse is `W H_a`, so the inverse source applies H_a first and
then the AND/use/cleanup implementation of W, with fresh outcomes. Each
inverse branch is `W H_a/sqrt(2)`; composing forward and inverse gives
`I/2` for each of four outcome pairs and exactly the identity channel.
There is no physical adjoint of a measurement trace.

## G. Optional efficiency amendment: clean-isometry provenance

Accept or defer this part independently. Deferring it cannot keep the core
measurement ban or close b-isa-spec-bwm. It does not assert that North Star
promises provenance promotion, or that a relative-phase gate becomes a
permutation on arbitrary inputs.

### G1. §6.1 insertion after the provenance definition

<!-- before:6.1-optional -->
```text
Every workspace bundle version, location version and cell version has
provenance `perm` (computed by permutation gates from its **dependency
set** S of versions) or `other`. Cell versions `(region, i, v)` advance on
every write. Π is the effective control context at the instruction.
```

<!-- after:6.1-optional -->
```text
Every workspace bundle version, location version and cell version has
provenance perm (computed by permutation gates or an exactly certified
clean-isometry boundary from its dependency set S of versions) or other.
Cell versions (region,i,v) advance on every write. Pi is the effective
control context at the instruction.

Optional clean_isometry rule: a contiguous source sequence C may be
certified as C|x,0_G,0_A> = |x,f(x),0_A>, with exact amplitude +1
on every x, where ordered x includes all read-only inputs, dependency
versions and effective coherent controls, G is allocated clean output,
and A is explicitly declared internal clean workspace. No other wire
or cell is modified. The certificate binds all concrete source points,
ordered wire/version identities, controls and clean lifetimes. All
columns on this clean-input subspace are checked exactly, with all
untouched components and internal workspace restored. For each input
column the claimed f(x) is a canonical fixed-width bit string; there
are no duplicate, missing or out-of-range columns. Finite exact checks
or independently checked compositional proofs may discharge the
equality; numerical or basis-value-only evidence cannot.

At that source boundary only, G has perm provenance with the union
of the declared input dependency sets, the input versions themselves
and effective control versions. Its source cleanup may be the exact
sequence adjoint within the resulting §6.2 window, including after
read-only uses permitted for perm values. Dirty-input behaviour is
not certified. Internal gates keep their actual classification;
process/unitary/channel claims still check their complete amplitudes.
This rule does not invoke T3 on a non-permutation emitted circuit.
```

### G2. §11/K81 sequence matching, before occurrence matching

<!-- before:11-K81-optional -->
```text
**Source cleanup-pair partition (K81).** Consumers MUST form the following
partition from the source before K65 selection and K80 expansion. Match
source operation occurrences by §5.3 and §6.2, against the most recent
```

<!-- after:11-K81-optional -->
```text
**Source cleanup-pair partition (K81).** Consumers MUST form the following
partition from the source before K65 selection and K80 expansion.
If the optional clean_isometry rule is accepted, a checked contiguous
computing boundary and its explicitly written exact sequence adjoint
are indivisible sequence occurrences for this partition. Match their
external versions and effective controls by §6.2 before considering
their internal inverse rows; those rows have no competing owners.
An unchecked boundary is not formed. Overlapping boundaries or any
double ownership of source rows are ill-formed. Every existing
intrinsic boundary and the joint-operation/first-release rules below
remain in force. This recognition establishes only the declared
clean-subspace boundary, not equality on a dirty target.
Match source operation occurrences by §5.3 and §6.2, against the most recent
```

### G3. §12.1 structural row

<!-- before:12.1-optional -->
```text
| `structural` | `{ "rule": "translation_lemma" \| "inverse_pair_library" \| "adjoint_table", "subclaims": [indices], "witness": … }`; for `translation_lemma`: `{ "V": [points], "W": [points], "X": [wires], "B": [wires], "frame": [first point, last point] }`; for `inverse_pair_library`: `{ "library_closure_hash", "claim_index" }`; for `adjoint_table`: `{}` | check the rule's side conditions syntactically against the object and the subclaims' results |
```

<!-- after:12.1-optional -->
```text
| `structural` | `{ "rule":"translation_lemma"|"inverse_pair_library"|"adjoint_table"|"clean_isometry", "subclaims":[indices], "witness":... }`; existing rule payloads unchanged; clean_isometry witness `{points,inputs,targets,internal,controls,table}` | existing rules retain their side conditions; clean_isometry checks §6.1's exact complete-column equality, lifetimes and untouched wires against source gate matrices, or a checked compositional derivation of that same equality in subclaims; table rows are `{input:"<fixed-width bits>",target:"<fixed-width bits>"}`, in lexicographic input order, exactly once per input |
```

`points` is the ordered concrete source-point list; `inputs`, `targets`,
`internal` and `controls` use D3's resolved wire/version records in declared
bit order. All are explicit arrays; inputs contain controls in their declared
positions and controls repeats their identities solely to check the context.
The exact equality restores controls unchanged on both branches, including
the disabled branch with amplitude +1. The claim binds its source closure
and K13 invocation (or proves parameter-uniform side conditions for all valid
arguments/budgets). It has no schedule-hash cycle. For C it proves only
the four clean columns in F1 and provides `perm` provenance at its endpoint.
It cannot authorise replacement of arbitrary-target CCX or stale source
versions, and supplies no channel proof for subsequent measurement cleanup.

## H. Conformance vectors and interpreter work

### H1. Required vectors (publish with the amendment)

Append before the existing Suite H/K77 paragraph:

<!-- before:15-vectors -->
```text
**Suite H / §11.1 K77 adapter vector.** Captured `oracle_invocation`
```

<!-- after:15-vectors -->
```text
**Private-cleanup conformance (K85).** Suites C/D/E/F/G/H include the
following positive and negative vectors. Exact channel checks include
zero-error private measurement realisations; suite D's old zero-pivot
SDP rejection remains a dual-route negative. Publish legal serialized
source, target, schedule, certificate, bound expansion identity,
expected reserved events/coordinates/resources and expected channel
for each executable vector. Algebra-only and adapter-generated
coordinates are labelled as such, not production-builder evidence.
Vectors include the qcopy delayed-cleanup bridge, four-T AND compute
and measurement cleanup, transported H/CZ/H closed-window correction,
inverse-channel reversal and per-root mixed-object inventory. Negative
vectors exercise phase, dirty targets, stale correction/versions,
missing outcomes/reset/claims, forbidden outcome escape, control
context reuse, conflicting ownership, illegal source and dependency
relocation, timing, capacity and hash binding. A basis-only or
host-limited result never discharges a channel execution obligation.

**Suite H / §11.1 K77 adapter vector.** Captured `oracle_invocation`
```

| Vector | Expected result and determining check | Status in this note |
|---|---|---|
| AND-8COL | Eight F1 columns, including all dirty g phases; C is not CCX | Exact replay passed |
| AND-INVERSE | 32 full C/C† columns with an independent untouched wire; eight clean compute/use/C† columns | Exact replay passed |
| AND-MBU | Eight logical input columns × two outcomes; both branches W/sqrt(2); all 64 matrix units | Exact replay passed |
| AND-DELAY | Legal source in F3; two branches H_a W/sqrt(2); all 64 matrix units; g clean | Exact replay passed |
| COPY-DELAY | Legal qcopy/quncopy/H source; delayed branches H/sqrt(2), input zero gives + with d=0 | Exact replay passed |
| LATE-INVERSE | Literal C/use/H/C† and qcopy/H/quncopy fail workspace cleanliness/source validity | Exact replay passed |
| CHANNEL-NEG | Missing CZ, stale CZ after H, and missing q on diagnostic controlled correction fail full channel | Exact replay passed |
| RESET-NEG | Omit outcome-one reset; retained private g is one | Exact replay passed |
| AND-ADJOINT | Source inverse W or W H_a; fresh outcome branches and four forward/inverse outcome pairs yield identity channel | Exact replay passed: 16 inverse branch columns, 32 round-trip branches |
| EXT-REFERENCE | Check off-diagonal E_ij for arbitrary external reference, including q coherences in diagnostic | Exact replay: 384 matrix units across the three AND contexts |
| CORE-NO-PROVENANCE | Native CCX-based legal source and qcopy-based legal source selectable without G | Contract/source fixture requirement; not interpreter-tested |
| OUTCOME-NEG | Missing/duplicate outcome string; outcome in host output, tape, hostcall or launch record; measure interface/borrowed wire | Required schema/privacy negatives; not interpreter-tested |
| PAIR-NEG | Invalid late source cleanup, wrong original carrier, reused inverse occurrence, overlapping compute replacement, implicit constructor carrier | Required production-source negatives; not interpreter-tested |
| BIND-NEG | Wrong schedule/target/expansion hash, missing nested/default-only context, wrong args/token tuple or versions | Required certificate negatives; not interpreter-tested |
| CONTROL-FALLBACK | Inherited q=0/1 and both qif polarities select unitary cleanup; uncontrolled witness reused under control fails | Required admission/fallback test; controlled channel algebra is diagnostic only |
| TIMING | Measurement ends at 2, feedforward latency 3: dependent correction starts >=5 and after dispatch; 4 rejects | Exact threshold arithmetic; scheduler vector required |
| RESERVATIONS | Both outcomes have identical reserved ordinals, padded time/resources; extended g lifetime may raise peak/capacity | Required event/resource vectors; not interpreter-tested |
| VERSION | 0.9 library/target cannot accept new events; 0.10 binds new envelope/profile; old 219 recipes unchanged | Old catalog hash/count replay passed; new publication pending |
| MIXED | Native main/libc plus marked AND root A and root B; root-relative [] and shared callee checked separately; no host quantum inventory | Required container association fixture; not interpreter-tested |
| OPTIONAL | Exact clean C table passes G; dirty-target waiver, missing phase, omitted controls, partial table and overlapping boundaries fail | F1 exact equality passed; schema/source tests pending optional acceptance |

These pending fixtures are implementation acceptance requirements, not claims
that this documentation job ran production conformance or implemented a
container. R3 forbids full gates; the focused verification here is limited to
the exact witnesses, source-anchor consistency and historical catalog identity.

### H2. sol-ref09 implementation impact (precise handoff)

The [current interpreter](../../../reference-interpreter/src/metadata_schema.rs)
enumerates cleanup as reverse/recompute and channel claims with the dual
payload. Its `tests/cleanup_partition.rs` explicitly labels its K81 carrier
formation a finite source adapter, not production Expander output.
sol-ref09 must change the following after contract acceptance;
none of these files is edited by this job:

1. Version admission: add the agreed 0.10 ISA/object/sched/cert/target vector
   and ref@0.10 hash/profile without changing 0.8/0.9 parsing or defaults.
   Keep seven-component version and signature ABI binding explicit.
2. `metadata_schema.rs`, `metadata.rs` and schedule-domain parsing: add
   measure_fixup/witness, Boolean mbu, private measurement cost record,
   event union, exact channel payload and D3's closed record schemas.
   Reject unknown fields, missing costs, aliases and wrong conditional kinds.
3. `source_constructor.rs`, `source_constructor/action.rs`, `validation.rs`
   and the production cleanup-partition path: implement/complete legal K81
   pair derivation before selection rather than importing finite-adapter
   carrier assignments. Use the source version/provenance checks and
   retain original source versions/carriers, then attach checked relocation
   and optional owned compute replacement. Do not admit literal late source
   cleanup or absorb excluded constructor cleanup. Preserve K63/K64/K80
   occurrence identities and unique ownership across every context.
4. `ref_library.rs` and `conformance/src/ref_language.rs` event expansion:
   implement the explicitly bound new
   profile; emit measure_z, dispatch, reserved conditional gates, reset and
   release with C3's coordinates; bind every operand/lifetime version and
   nested argument/token/budget from source. Outcome choice cannot renumber.
5. Scheduling/timing/resource path: validate removed/added graph edges,
   acyclicity, after placement, live versions, extended lifetimes, topology,
   capacity, padded charges, dispatch time, latency and outcome-independent
   epsilon-share population. Leave other dependency edges intact.
6. `quantum.rs`/`action.rs` and the realised-event executor: add private branch maps
   and deterministic exact verification of all outcomes, not a public CC
   measure opcode. Execution may sample with Born probabilities, while the
   verifier enumerates all branches. Reset and privacy checks happen before
   deleting workspace. Keep outcome storage separate from host/result/tape.
7. `certificate.rs` and exact arithmetic in `exact.rs`: build complete
   source and realised maps independently; add exact matrix-unit channel
   equality, trace preservation, exact restoration and context coverage;
   retain the rigorous dual route unchanged. Record host-limit checks as
   unchecked and forbid using them for executable verification. Branch
   operators are on complete interfaces, not just measured wires or basis
   populations. Channel proofs do not become operator-norm phase proofs.
8. `machine.rs` and launch/history handling: distinguish private
   cleanup from source measure frames; expand and verify the logical inverse
   with fresh private outcomes for adjoint calls, segment uncall and whole-run
   reversal. Keep explicit-source-measure and unverified-free restrictions.
9. Certificate loader/binding path: resolve schedule witness ids, require
   exact definition equality with the certificate, and bind all event bytes
   in schedule_hash without a hash cycle; require successful channel proof
   before affected effects;
   validate per-device object/root records and external inventory association
   without inventing an ISA record for native main or host libc. General
   mixed launch/host-memory mechanics remain with bennett-4ht owners.
10. If G is separately accepted, add source-bound exact clean_isometry
    proof validation, boundary transfer and indivisible sequence pairing;
    do not classify emitted H/T as permutation gates or alter T3.
11. Add H1 fixtures to the existing focused `isa09_source_schedule`,
    `cleanup_partition`, `cleanup_ownership`, `isa09_certificate`, quantum,
    version, resource and reversal tests, using the new version where
    appropriate. Confirm old-version negatives and compatibility fixtures.
    Keep adapter/algebra evidence distinct from production-builder results.

The file names above are navigation points, not permission to reuse another
repository's code across a published-spec boundary. The required behaviour,
record schemas and verification conditions are the contract.

## I. Hashes and catalog impact

The existing ref@0.9 envelope was independently recomputed from its bytes:
219 recipes, SHA-256
`5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197`.
The focused script checks both values. No catalog bytes or native gate
definition are changed by this note.

| Object | Impact when the proposal is published/used |
|---|---|
| Existing 0.9 source/object | Unchanged when left 0.9; no retroactive CCX reinterpretation |
| Same source migrated to ISA 0.10 | §9.2's `isa=` envelope line changes interface/closure hashes and their transitive callers/SCCs; regenerate canonical references and examples, never reuse the old identity |
| Signature ABI | Remains 0.9 absent a separate ABI change; this does not suppress the ISA-envelope change |
| Source cleanup relocation | Schedule/certificate implementation choice; for a fixed ISA version, no source-byte rewrite or semantic source-envelope field is added |
| Optional literal C boundary | Claims do not enter source identity; changing source from CCX to literal C does change source bytes/hashes; a source hash does not certify the boundary |
| Target | mbu/cost/profile/version byte changes alter target_hash; old mbu=false target remains valid for unitary cleanup |
| Schedule | New expansion identity, witness id, cleanup_witnesses definitions and entries change schedule_hash; every event/placement/dependency byte is committed and compared with the certificate |
| Certificate | Channel claims, cleanup records/resources/context sets and version vector change certificate bytes and certificate_hash; source identity excludes these metadata sections |
| Expansion | New ref@0.10 canonical document/profile and `isa=0.10/expansion=ref/version=0.10` envelope have a new published hash; no fabricated hash here, no mutation of ref@0.9 |
| Fat object | Device roots bind their own migrated object/closure/schedule/certificate identities; outer inventory changes separately; native-container hash is not object_hash |
| Resources | C compute four T; zero-T cleanup still charges H/CZ/X, measurement, dispatch, latency, occupancy and capacity; controlled correction has its own costs |

Acceptance requires simultaneous publication of normative clauses, the new
closed schema/profile, actual library bytes/hash, source/schedule fixtures and
focused consumer support. This note completes the amendment deliverable;
it does not close the live capability gap or the mixed-launch boundary bead.

## J. Council round-1 blocking-item disposition

The [round-1 report](../../../../bennett/docs/council/contract-l2p-2026-11-02/round-1/report.md)
exists and grades the earlier note (snapshot prefix ec16084f8a5b), with sol
and codex both requesting revision. This amendment does not claim a new
council acceptance. The user brief names sol+astra; the recorded report's
actual member labels are sol+codex, which are the labels mapped here.

| Recorded blocker | Text supplying the repair | Determining witness |
|---|---|---|
| sol B1 | B1/B2 and C2/C3/D3 bind the unchanged legal source pair at the original carrier, permit only explicit checked dependency relocation and account for actual versions/lifetimes | F3 correct source vs schedule and literal-late-inverse negative; F4 qcopy bridge independent of G |
| codex L2P-1 | Same source-to-schedule bridge; whole-region channel against the original source, no access to a source-released lifetime and no inference from the late literal adjoint | H1 AND-DELAY/COPY-DELAY/LATE-INVERSE, exact branch and matrix-unit checks |
| sol B2 | D1 and D3 delimit device channel subjects, per-object/per-root certificates, outer inventory, native host/argv/envp/libc and private measurement inventory | H1 MIXED fixture requirement; A's shared scoping statement and D3's remaining mixed-launch obligations |
| codex L2P-2 | Same device scope; selected exported root and materialised native comparison bind the marked closure and its callees, not the whole host | Native main may call ordinary libc, launch marked AND A, then continue natively; only A's device closure has this channel certificate |

The report's nonblocking corrections are also incorporated: A corrects §9.6;
B2/E1 distinguish private events and source measure; C1/C2 account for reset,
dispatch, latency and unconditional reservations; C3 fixes leaf coordinates;
D1/D2 add exact zero-error verification without weakening the SDP; C4/E2 keep
private events distinct from unitary gate names and historical library bytes.

## K. Self-check and evidence boundary

Run this focused command (2 GiB address-space cap, 90 CPU seconds, 120 seconds
wall timeout; headless; no /tmp artefacts):

```sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/docs/notes/2026-11-02-rev3/.tmp prlimit --as=2147483648 --cpu=90 timeout 120s python3 -B /home/tobias/Projects/bennett-workspace/bennett-isa-spec/docs/notes/2026-11-02-rev3/verify_l2p.py
```

The script checks the exact circuits/channels, all BEFORE spans against the
published file, required topic coverage, relative links, code fences and the
historical catalog envelope/count. It writes nothing. The resulting PASS
lines are recorded in the job Resume block after verification. No full gate,
reference-interpreter test, sibling generator or new-release catalog was run.
Those implementation/publication obligations remain open under b-isa-spec-bwm
and bennett-4ht; completing this note is not evidence they are implemented.

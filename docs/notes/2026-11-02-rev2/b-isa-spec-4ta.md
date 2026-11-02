# 4ta — one interface slot placed across several classes

Council proposal for `b-isa-spec-4ta` (P1).

## Round-1 blocking items and resolution

Read the [round-1 report](../../../../bennett/docs/council/pass2-4ta-2026-11-02/round-1/report.md)
and both `out-sol.json` and `out-codex.json` in that directory. The table
maps every blocking item to this revision; it claims a proposed resolution,
not a new council verdict.

| Item | Change and witness |
|---|---|
| sol 4ta-B1; codex B1 — §3.3 says one class per register but concat has no same-class condition | Replace the register/ledger definition and specify new/concat/slice/free/reversal and frame-binding vector equality below. Witness: new A[1], new B[1], concat yields [A,B], split maps 0 to 3; inverse execution restores original wires, classes, ids and handle states. |
| sol 4ta-B2; codex B2 — missing mixed-object descriptor scope | Append the §9.4 rule below. A native host launches a device CC wrapper that joins the two registers and calls the single split slot; host functions remain outside B5. |

## Defect and North Star scope

ISA 0.9 §11 “Topology placement” supplies one vertex **per bit**, in
§9.4 slot order, and callees inherit actual operand placements. §8.3
assigns a class to each vertex. Neither clause requires all bits of a
source interface slot to have the same class. B5 (§9.4), however, gives
each slot exactly one `class` and its full signature `width`; unknown
fields and duplicate slot identities are not an extension mechanism
(§9.1). A legal mixed placement has no truthful descriptor. This is a
contract defect under C4/C5/C8, distinct from [v2v](b-isa-spec-v2v.md):
4ta concerns different bits **within one invocation of one slot**.

North Star §§4.1/4.3/4.4 promises virtual memory, allocation classes and
target-driven placement/resource reporting. Splitting a virtual slot
must not require a user rewrite or a target with a larger homogeneous
class. Class capacities still constrain physical feasibility; no capacity
is manufactured by this repair.

## Alternatives and worked witness

An action `split(t:wire[2] mut)` applies `x t[0]` and `x t[1]`.
A target has two vertices, 0:A and 1:B, each class capacity 1, and exact
single-wire X entries of duration 1 for either class. Placement
`interface_vertices:[[0,1]]` is legal: there is no multi-wire adjacency
requirement to satisfy. Both bits stay live for two sequential slots.
For unit class rates, time=2, peak[A]=peak[B]=1 and
qubit_second[A]=qubit_second[B]=2. On the four values t=0,1,2,3, the
outputs are respectively 3,2,1,0; the adjoint restores every input.

One B5 record `class:A,width:2` lies about bit 1 and exceeds A's capacity;
class B has the symmetric problem. Two `mut:0` records lose uniqueness;
two new source slots change the interface and its hashes unnecessarily.

| Alternative | Tradeoff |
|---|---|
| Ordered per-bit class vector | Precise, but repeats one class name per bit. |
| Ordered maximal class fragments | Precise, compact for runs, retains the source slot. Recommended. |
| Require a homogeneous class or split the source signature | Removes this legal placement or changes the program interface. |

## Recommended exact spec diff

First replace §3.3's singular-class register definition and update every
ledger-frame reference to class as follows (including §10's allocation
cross-reference):

```text
A register denotes an ordered vector of n device-resident wires with a
resolved target class for each wire. The register ledger records that
class vector, width, nonzero id and live/viewed/consumed state. A single
class is shorthand for a homogeneous vector. qreg.new cls n and
bennett_qreg_new allocate n wires of cls and remain homogeneous requests.
qreg.concat accepts differing operand classes and concatenates their
wire and class vectors in operand order, with no movement, copy,
measurement or reallocation. Its existing distinct/live/non-view
preconditions and consumption/new-id transitions remain unchanged.
qreg.slice selects the corresponding wire/class subsequences, with its
existing view ownership and one-call consumption rule. A slice never
relabels a wire. Zero-width runtime registers have empty vectors.
Ledger frames retain enough information to restore the complete class
vectors, wire ordering, identities and handle states. In particular free
records/restores the vector along with the same id/width and cleanliness
status; concat reversal restores both operands and their prior vectors;
slice reversal restores the parent/view states. New-frame reversal keeps
its existing cleanliness/exclusivity hypothesis.
Binding a CC/R register to a region slot requires its ordered class
vector to equal that slot's selected, expanded §9.4 class vector, and,
with topology, its actual vertices to equal the inherited placement.
Check this with the existing width/disjointness preflight before any
effect or view consumption. Class vectors introduce no new alias or
storage-sharing permission, nor permission to relabel or move a wire.
```

Then in §9.4 replace the `slots` record production with:

```text
{id, role, width, class} |
{id, role, width, fragments:[{offset, width, class}, ...]}
```

Append:

```text
Each source slot still has exactly one record, its original id and role,
and its complete signature width. Exactly one of class and fragments
is present. class abbreviates one fragment at offset 0 covering width.
For the fragmented form, fragment offsets and widths are nonnegative
and positive integers respectively, using §9.1 integer encodings.
Fragments are in increasing offset order, start at 0, meet without gaps
or overlap, and end at the instantiated slot width. Each class is a
resolved declared target class. Adjacent equal-class fragments must be
merged; use the class form when there is only one fragment.

Flatten bundle bits LSB first, or region cells in address order and
each cell LSB first, exactly as for the existing slot. A fragment names
the class of that half-open bit interval; it is not a new source slot,
allocation, lifetime, alias or permission to share storage. Repeated
slot ids, wrong roles/widths, gaps, overlaps or out-of-range fragments
reject. Slot widths retain §8.1's c-only expression rules; fragment
widths/offsets describe the resolved fixed signature width.

With topology, every bit's fragment class must equal the class of its
§11 vertex; vertices and simultaneous live uses remain disjoint. A
callee's fragments must equal the inherited operand-class vector. In
the absence of topology, they bind that resolved vector without naming
vertices. Capacity, occupancy and peak accounting use each bit's actual
class, and charge no wire twice. Existing homogeneous records and
scratch/history/borrow/ancilla schemas retain their meanings.
```

The witness descriptor is one slot
`{id:"mut:0",role:"mut",width:2,fragments:[{offset:0,width:1,class:"A"},`
`{offset:1,width:1,class:"B"}]}`. This is proposed record notation;
the production's field names are exact. It is not accepted B5 today.

## Fat-object applicability

Incorporate [the shared scoping statement](fat-object-scope.md#citable-scoping-statement).
Append the following proposed normative paragraph to §9.4:

```text
In a mixed fat object, B5 descriptors cover device-region invocations
in the embedded or linked ISA objects, bound to their object, closure,
schedule and invocation identities. Native host functions and process
state require no ISA frame slots or certificate records. A native host
may launch a device CC entry under §10; that entry may concatenate
homogeneous A and B registers under §3.3 and bind their ordered class
vector to one source slot. The fragments describe that binding without
splitting its signature. Native main, argv/envp and host-only libc stay
outside the device-frame inventory. This rule supplies no new container
or general host-memory launch ABI.
```

The shared statement lists today's assumptions and the exact remaining
container, launch, budget, handle, tape and host-memory contract changes.
This note does not assume that the device object is the entire binary.

## Hashes, catalog and consumers

The register-class rule is a core clarification/extension requiring renewed
core council review; B5 alone cannot make the slot usable. The
slot/signature/source closure stays the same for the unchanged split body. A repaired certificate
changes bytes; its existing placement schedule and target need not.
No reference recipe, leaf ordinal or catalog hash changes. Circuits,
scheduler, estimator and checker must validate fragments against the same
bit ordering and retain class-qualified resource rows. This does not
alter the explicit class of a source `qalloc` or add routing.

## Register construction and reversal witness

In a device CC wrapper allocate a=qreg.new A 1 and b=qreg.new B 1,
then t=qreg.concat a b and action_call @split (t) () () (). The two
homogeneous allocations each fit capacity 1. Concat consumes a,b and
creates a fresh t over the same ordered wires with classes [A,B]; its
mut:0 descriptor expands to exactly that vector. From initial clean 0,
the call produces 3; on 0,1,2,3 it gives 3,2,1,0. Reverse the action,
then concat, to restore a,b live with their original ids, [A]/[B]
classes and wire order, with t no longer live; reversing allocations
then releases their clean wires under the existing ledger rule.
A slice of t inherits its selected class subsequence. A false [A,A]
frame fails before effects rather than relabelling the B wire.

## Conformance vectors and verification

Test the four outputs and inverse above, both topology and class-only
descriptors, a region[1,2] slot with four flattened bits, and a homogeneous
legacy record. Reject a gap, overlap, width zero, split duplicate id,
incorrect class for a vertex, reversed offset order and adjacent unmerged
runs. For region[1,2] use vertices/classes [A,B,A,B], capacity 2 per class;
offsets 0,1,2,3 denote cell0.bit0, cell0.bit1, cell1.bit0, cell1.bit1.
For a controlled invocation enlarge the target to vertices 0:A,1:B,2:C,
capacity 1 each, edges [0,2] and [1,2], and exact controlled-X leaves of
duration 1. The control is caller-owned, outside mut:0;
(c,t) maps to (c,t XOR (3*c)), time=2 and occupancy=2/peak=1 in each class. Exact finite arithmetic is
checked by [check_rev2.py](check_rev2.py); no target/scheduler implementation
is executed here.

The originating note cites circuits' `docs/design/2026-11-01-m5-contract-requests.md`,
“Heterogeneous interface-slot frames”. Its reported refusal records
incompleteness, not a new legality rule for source programs.

## Release and route

Recommend a **0.9 K/B5 encoding extension** alongside v2v; the gaps are
independent and both require vectors. Waiting for 0.10 leaves an explicit
blocker for complete 0.9 certificates, not permission to omit mixed
placements. Keep 4ta open for council and consumer work.

## Publication and validation boundary

This is a revision proposal, not council acceptance or a consumer implementation.
The default publication rule in spec/README.md requires a full version bump
for semantic changes. The recommended 0.9 K route requires explicit council
and maintainer authorization of its in-version compatibility rules; this note
does not grant it. All unchanged-hash claims assume unchanged header/source
and catalog envelopes. A full version publication recomputes affected symbol,
object, target, schedule, certificate and versioned ref-library identities and
consumer pins even when recipe bodies stay fixed. Published 0.9 bytes remain
available; older closed-schema readers require an upgrade for new fields.

The reproducible finite and document checks are in [check_rev2.py](check_rev2.py):
`timeout 120 python3 -I -B /home/tobias/Projects/bennett-workspace/bennett-isa-spec/docs/notes/2026-11-02-rev2/check_rev2.py`.
Executed for this revision: exit 0, all eight PASS result lines.
These finite checks verify witnesses and revision structure; they implement
no proposed-format checker or host shim. No full repository gate or sibling gate is claimed: scripts/pre-push
uses git/worktree writes, forbidden by this runner, and its setup changes paths
outside this job. Underlying beads remain open for publication and consumers.

# 4ta — one interface slot placed across several classes

Council proposal for `b-isa-spec-4ta` (P1).

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

In §9.4 replace the `slots` record production with:

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

## Hashes, catalog and consumers

The slot/signature/source closure stays the same. A repaired certificate
changes bytes; its existing placement schedule and target need not.
No reference recipe, leaf ordinal or catalog hash changes. Circuits,
scheduler, estimator and checker must validate fragments against the same
bit ordering and retain class-qualified resource rows. This does not
alter the explicit class of a source `qalloc` or add routing.

## Conformance vectors and verification

Test the four outputs and inverse above, both topology and class-only
descriptors, a region[1,2] slot with four flattened bits, and a homogeneous
legacy record. Reject a gap, overlap, width zero, split duplicate id,
incorrect class for a vertex, reversed offset order and adjacent unmerged
runs. Add a controlled invocation with the control charged separately,
not inserted into `mut:0`. The exact finite arithmetic is checked in
[INDEX](INDEX.md); target/scheduler implementation is not executed here.

Source context read: circuits' `docs/design/2026-11-01-m5-contract-requests.md`,
“Heterogeneous interface-slot frames”. Its named refusal is honest
incompleteness, not a new legality rule for source programs.

## Release and route

Recommend a **0.9 K/B5 encoding extension** alongside v2v; the gaps are
independent and both require vectors. Waiting for 0.10 leaves an explicit
blocker for complete 0.9 certificates, not permission to omit mixed
placements. Keep 4ta open for council and consumer work.

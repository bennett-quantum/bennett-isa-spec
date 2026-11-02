# rvu — bind borrowed bits to their actual owners

Bead: **b-isa-spec-rvu**. Amendment proposal, never a prerequisite for rungs 3/4; F30 stays deferred until rung 3 or 4 lands.

## Exact defect and clause citations

[ISA 0.9](../../../spec/Bennett-ISA-Spec-0.9.md) §9.4 says callees borrow from the caller's designated idle set; §8.2 charges the borrower at dirty_rate, replacing the owner's charge. §11 has neither an owner designation nor a deterministic implicit idle-set choice. A frame id records requirements, not a physical alias. Vertices alone cannot solve the no-topology case. This is distinct from [7s0's frame-proof binding](../2026-11-02-rev2/b-isa-spec-7s0.md), which does not select physical owners.

## North Star gap

Missing schedule mechanics for promised dirty borrowing, not permission to remove borrowing or replace T3 with selected basis tests. North Star §§4.3/5 requires correlation-preserving return and correct resource accounting.

## Alternatives and witnesses

Caller has three live interface bits t,y,x. A four-CX callee borrows one dirty bit for four unit-time events. Separate external dirty storage yields peak 4 and occupancy `3*4 + .5*4 = 14`; lending idle mutable y gives peak 3 and occupancy `2*4 + .5*4 = 10`. The bead's scheduler probe observed the former; this pass independently checks the arithmetic and all eight t/x/b basis cases of the four-CX translation, not that C ABI probe. Guessing an owner from a producer string or subtracting an arbitrary interface charge is unsound. Prefer explicit aliases with the current external requirement as an absent-field default.

## Recommended exact spec diff

Extend §11 `alloc` and `gate` decision objects:

```text
alloc: {class, vertices?, borrow_from?:[owner-bit,...]}
gate: {realisation,epsilon_share,ancilla_borrow_from?:[owner-bit,...]}
owner-bit is exactly one of:
 {invocation:<fully concrete §12.1 context key>,slot:<§9.4 slot id>,bit:n}
 {allocation:<fully concrete alloc-operation selector>,bit:n}
```

Append to §§9.4/11:

```text
An allocation selector uses concrete path/step/phase/generated coordinates
and kind=alloc, including fixed instance/call_index/iterations; no wildcard
or frame-name inference is permitted. borrow_from is legal only for a
dirty acquisition and has exactly its width items in bit order. Every
item resolves to a distinct live bit held by the direct caller: mutable
interface/region storage or a live allocation/lease, of the resolved class.
Exclude the callee's passed interface bits, inherited controls, caller
read-only bits, overlapping outgoing loans and owner accesses overlapping
the loan. An incoming lease may be re-lent while held; retain its complete
owner chain. Expired/out-of-range/wrong-context references are invalid.

An explicit list is the caller's designated idle set. Bind the logical
borrowed bits to these owner bits with no additional physical acquisition;
retain owner vertices, and require alloc.vertices to agree when supplied.
Return restores the owner's state and lease. The universal correlated
T3 obligation on the normative §5.6 domain remains; a physical alias is
not proof. Logical lifetime identities and acquire/release/gate ordinals
in the bound expansion remain unchanged.

For each outgoing loan, split the owner's charging interval at its exact
acquire/return boundaries and replace its rate by the borrower's dirty
rate during that half-open interval. Count the physical bit once in peaks.
Nested lending uses the currently active borrower, never sums all owners.
Access and event ordering at equal timestamps must respect lifetime
dependencies; half-open rate intervals do not license simultaneous use.

Absent borrow_from denotes a separate external dirty requirement with no
implicit alias/subtraction from unrelated caller storage. It does not
designate one of the caller's already counted interface bits. Such demand
still requires physical capacity and its full frame/proof obligations.

gate.ancilla_borrow_from applies the same rules to the selected scheme's
resolved dirty ancilla bits, in template allocation order; its length
equals that dirty demand. It is invalid for native or clean-only schemes.
The scheme's ordinary clean ancillae keep their independent acquisitions.
```

This chooses the explicit/default alternative in the bead; it does not retrofit a hidden idle-set heuristic. Dirty demand from a native externally supplied pool and aliasing an existing caller bit are different resource records.

## Hashes, catalog and consumers

Owner references enter the schedule hash. Certificates/frames/reports must reflect actual aliases and context-dependent charging; source/closure hashes and ref recipes/leaf ordinals do not change. Coordinate [4ta](../2026-11-02-rev2/b-isa-spec-4ta.md), [7s0](../2026-11-02-rev2/b-isa-spec-7s0.md) and topology fields. Consumers must validate leases rather than trust producer proof labels.

## Conformance vectors and verification

Require the 14-versus-10 witness and capacity 3 versus 4; partial bundles/regions; nested re-lending; class/vertex agreement; readonly/control/outgoing-argument exclusion; expired context/allocation; owner access during loan; zero-time boundaries; scheme dirty ancillae; full T3 including entangled external owners. The finite translation truth table establishes a permutation relation by linearity; test phase-sensitive T3 separately. Actual checks are listed in [INDEX](INDEX.md).

## K entry: 0.9 versus 0.10

Recommend **0.10 schema/default amendment**, decision `BORROW-OWNER-BINDING`. The absent-field default resolves an ambiguity with observable costs. An in-version 0.9 K route needs **council + maintainer authorization of compatibility rules**, including old cost/certificate handling. No numeric K or producer field is adopted here.

## Mixed fat object

Use [fat-object-scope.md](../2026-11-02-rev2/fat-object-scope.md) and [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F03/F06/F09. Owner selectors are device-root-relative invocations within a program ledger. They are neither native pointers nor permission to borrow a sibling program's register id. Native host/certificate inventory is unchanged; future cross-program transfers require the explicit ledger protocol. These amendments never gate rungs 3/4; F30 stays deferred.

# 8uz — serialize root interface classes

Bead: **b-isa-spec-8uz**. Amendment proposal, never a prerequisite for rungs 3/4; F30 stays deferred until rung 3 or 4 lands.

## Exact defect and clause citations

[ISA 0.9](../../../spec/Bennett-ISA-Spec-0.9.md) §11 serializes `interface_vertices` only with topology and `alloc.class` only for generated acquisitions. §9.4 requires concrete classes in frame slots; §11 defaults choose the first class in target JCS order with capacity. Without topology there is no decision field binding a nondefault root interface class. A producer cannot communicate its choice through certificate frame strings alone.

## North Star gap

This is a schedule expressibility/determinism defect, not an absent machine capability. North Star §5's target-aware execution requires one replayable class choice. Preserve multiclass scheduling and existing source domains; a consumer's current default is not permission to ban class search.

## Alternatives and witnesses

Target classes a,b each have eight qubits; only b supports x. Root `mut:0` has one wire and executes x. An empty-entry schedule defaults to a, so a producer choosing b only in its frame cannot claim reproducible OK. Restricting all roots to a loses valid mappings. Fake alloc decisions cannot select caller-owned root wires. Prefer a top-level class vector. With a.capacity=0 the default is b, demonstrating why gate availability must not secretly replace the capacity-only default. [verify.py](verify.py) checks this finite distinction, including remaining-capacity overrides and width accounting; no scheduler C ABI probe was rerun here.

## Recommended exact spec diff

In §11 Schedule document add beside `interface_vertices`:

```text
"interface_classes"?: ["<concrete target class>", ...]
```

Append after Topology placement:

```text
Root interface_classes has one item per prepended inherited control wire,
then one per §9.4 source interface slot in source order. Every bit/cell of
a slot has that concrete declared class. Resolve the vector before capacity
and gate matching. A callee inherits its actual operand classes, including
controls; it cannot override them with a new root vector.

Without interface_classes and without topology, assign each item in that
order to the first class in target JCS order with sufficient remaining
capacity for its complete width, using request capacity overrides and
already live demand. Failure of a later gate does not silently reassign
that root item. Existing generated alloc.class decisions keep their scope.

With interface_vertices, each slot/control array has one uniform vertex
class and determines its class. If interface_classes is also supplied it
MUST agree. Wrong vector length, unknown/"any" class, mixed vertex class,
class conflict, overlapping live placement or insufficient capacity makes
the schedule invalid. Without topology interface_vertices remains absent.
Certificate root slots and per-context frames must match these actual
classes; certificate metadata cannot supply a missing schedule decision.
```

## Hashes, catalog and consumers

The field changes schedule JCS/hash and dependent certificates; source object/interface/closure hashes and `ref@0.9` recipes remain unchanged. Bind vectors separately per selected root. Scheduler, VM, estimator, circuits and checkers must negotiate the new schema; current consumers must not accept unknown fields by accident. Coordinate heterogeneous frame/context proposals rather than invent a competing class derivation.

## Conformance vectors and verification

Require a/b witness with absent vector versus explicit `[b]`; zero a capacity; request overrides; two root slots exhausting a; multibit/region slots; inherited controls; callee propagation; topology/vector agreement and conflict; wrong lengths/classes; a gate unavailable in the default class. Test exact frame class agreement and unchanged generated alloc resolution. Actual evidence is finite-model/citation checking in [INDEX](INDEX.md), not a new scheduler implementation.

## K entry: 0.9 versus 0.10

Default **0.10 schedule-schema amendment**. Proposed decision `ROOT-INTERFACE-CLASSES`, no catalog edit. An in-version 0.9 K repair requires **council + maintainer authorization of compatibility rules**, covering unknown-field admission and consumer pins. Do not emit the new field before publication.

## Mixed fat object

Apply [fat-object-scope.md](../2026-11-02-rev2/fat-object-scope.md) and [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F03/F05. Vectors describe one selected device root and its invocation frames, never all native process arguments or native memory. Native main/argv/envp/libc require no frame slots; each root retains context `[]`, certificate and ledger. The outer inventory and native-memory adapter remain explicit separate amendments. Rungs 3/4 are independent; F30 stays deferred.

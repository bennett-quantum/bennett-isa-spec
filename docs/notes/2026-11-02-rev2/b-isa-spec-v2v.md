# v2v — frame descriptors for every realised invocation context

Council proposal for `b-isa-spec-v2v` (P1).

## Round-1 blocking items and resolution

Read the [round-1 report](../../../../bennett/docs/council/pass2-v2v-2026-11-02/round-1/report.md)
and both `out-sol.json` and `out-codex.json` in that directory. The table
maps every blocking item to this revision; it claims a proposed resolution,
not a new council verdict.

| Item | Change and witness |
|---|---|
| sol V2V-1; codex V2V-1 — frames lack a device-root boundary | Append the §12.3 rule below. Witness: device pair's root key is []; flip contexts belong to pair's tree; native main needs no descriptor. Separate root B has its own []. |

## Defect and North Star scope

ISA 0.9 §12.1 “Invocation contexts” includes every reached invocation,
including defaults, with resolved arguments, budget and choices. §12.3
stores one region record per closure hash and requires one B5 frame.
§9.4 requires each slot's concrete target class and substitutes parameters
and budget into width constants before forming c-only expressions.
The same callee can receive A in one call and B in another; no single
descriptor can express both. A `(variant,controls)` key alone also fails
when those values agree. This is a C4/C5 contract defect.

North Star §§4.2/4.4/5.1 requires freely nested regions and independently
checkable schedules/certificates. Refusing repeated mixed-class calls
would narrow this machine. [4ta](b-isa-spec-4ta.md) describes a different
case: a single slot fragmented within one context. Both repairs compose.

## Alternatives and worked witness

Use the source from `../bennett-sched/tests/pipeline.rs`, test
`context_varying_frame_classes_are_explicitly_deferred_instead_of_misreported`:
root `pair(t,u)` calls the same one-bit `flip` on t and then u; flip is X.
Place t in A and u in B, capacity 1 each. Each call is forward with zero
inherited controls and the same closure hash. Callee frame one has
`mut:0,class:A,width:1`; callee frame two has B instead.

For all four root basis inputs, the result is `(t XOR 1,u XOR 1)`.
At unit duration/rate the root time is 2 and each root interface bit is
live for both slots: occupancy[A]=occupancy[B]=2, peaks 1 each.
The per-invocation callee views have duration 1 and just their respective
class. Neither copying A into both records nor taking a class wildcard
describes the realised placement. This witness needs no heterogeneous
slot and no nonzero epsilon.

| Alternative | Assessment |
|---|---|
| Duplicate region records or rename the callee | Breaks closure-keyed identity or unnecessarily changes the source. |
| Use one wildcard/maximal frame | Cannot verify concrete classes and actual widths. |
| Exact context-indexed frames, uniform shorthand retained | Describes both calls and keeps old uniform certificates. Recommended. |

## Recommended exact spec diff

Replace the §12.3 region `frame (required)` entry by:

```text
frame?: <§9.4 descriptor>,
frame_contexts?: [{context_key:<§12.1 key>, frame:<§9.4 descriptor>}]
```

Append:

```text
Each scheduled region record contains exactly one of frame and
frame_contexts. frame is a descriptor valid for every reached invocation
of that region in the bound root realisation, after instantiating c for
each invocation. It may be used only if all its concrete classes and
resolved non-c-dependent widths describe every invocation truthfully.
frame_contexts contains exactly one record for each reached invocation,
in increasing bytewise order of the JCS encoding of context_key. The
key is exactly §12.1's fully resolved invocation chain; no wildcard,
variant/control-only key, omitted default-covered call or repeated key
is admitted. Every supplied frame_contexts array is validated, including a single-context
array. Coverage includes predicated-off and default-covered realised calls;
frame selection never inspects the quantum predicate value. The root invocation has key []. Resolve its variant,
controls, arguments, budget, placements and expansion choices from the
bound invocation tree, never from record order or a guessed root budget.

Select the context's descriptor before substituting c and check its
slots, classes, widths, allocation identities and lifetimes against
that invocation. Inherited controls remain caller-owned and separate
from callee slots. Distinct contexts have distinct dynamic frame
identities even when their static descriptor id strings agree. Identical
descriptor bytes do not authorize shared physical storage; §9.4's
live-range and disjointness rules remain required. A uniform descriptor must also have valid allocation order, scheme identities
and live ranges in every covered invocation. A standalone reference
record uses frame for its explicitly bound invocation or a descriptor
proved uniform over its declared invocation scope.
```

Here `context_key` uses the same spelling as the first-pass resource and
fault proposals. Their arrays and `frame_contexts` must cover the same
reached context set when applicable, rather than enumerate independently.
The originating bead's field spelling `context` is superseded by this
consistent proposed spelling; neither is published yet.

## Fat-object applicability

Incorporate [the shared scoping statement](fat-object-scope.md#citable-scoping-statement).
Append the following proposed normative paragraph to §12.3:

```text
In a mixed fat object, frame and frame_contexts describe the linked
ISA regions in the certificate's bound device-root realisation. Each
device root has its own object/root bindings and complete context set;
[] denotes that device root, never the native executable entry. Native
host functions, process argv/envp and host-only libc need no frame
records. Shared linked closures retain separate root-relative contexts.
A host may reach the device through §10's device CC entry; the general
mixed-container launch shim remains the explicit boundary work described
in the shared scope statement. Native comparison binds the corresponding
materialised device region and callee closure under its recorded configuration.
```

The shared statement lists today's assumptions and the exact remaining
container, launch, budget, handle, tape and host-memory contract changes.
This note does not assume that the device object is the entire binary.

## Hashes, catalog and consumers

No callee clone, source/interface/closure hash or catalog change is
required. Certificate bytes change. Existing schedules already distinguish
the two source call points; no new call edge is invented. Schedulers,
specialisers and checkers must retain full context keys across linked and
default-covered calls. Estimator frames and resource reports must select
the same context. A uniformly valid c-expression stays usable as `frame`;
the repair must not force needless duplication of uniform certificates.

## Concrete context witness

With H_pair denoting the pair closure hash, the two context keys are
`[{symbol:H_pair,point:"entry:0",iteration:[],instance:[],call_index:0}]`
and the same key with point `entry:1`. Their complete frame descriptors
are `{slots:[{id:"mut:0",role:"mut",class:"A",width:1}],scratch:[],history:[],borrow:[],ancillae:[]}`
and the identical descriptor with class B. Replace H_pair by the bound hash
and sort by unsigned UTF-8 bytes of each key's JCS encoding. These five
arrays are all required B5 arrays. An empty schedule entries array does
not omit either default-covered call or its frame. A separate device
root's [] is independent of pair's [].

## Conformance vectors and verification

Use the pair/flip case; check both class records and all four outputs,
then inverse restoration. Reject two A records, a missing default call,
duplicate/foreign key, both `frame` and `frame_contexts`, neither field,
and a width from another token budget at the same variant/control count.
Add a homogeneous two-call control accepted with old `frame`, and combine
one context with 4ta fragments. Finite output/resource arithmetic is
checked by [check_rev2.py](check_rev2.py). The originating note reports the sibling test's `Unsupported` expectation
as gap evidence. This revision executes no sibling test and claims no
completed consumer fix.

## Release and route

Recommend a **0.9 K certificate-context repair**, coordinated with
[cwq](../2026-11-02-pass2/b-isa-spec-cwq.md) and [jw7](../2026-11-02-pass2/b-isa-spec-jw7.md). Keep v2v open until
the encoding and consumer tests are accepted. A 0.10 deferral must retain
this explicit blocker; it cannot restrict the admissible call graph.

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

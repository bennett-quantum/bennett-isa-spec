# v2v — frame descriptors for every realised invocation context

Council proposal for `b-isa-spec-v2v` (P1).

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
is admitted. The root invocation has key []. Resolve its variant,
controls, arguments, budget, placements and expansion choices from the
bound invocation tree, never from record order or a guessed root budget.

Select the context's descriptor before substituting c and check its
slots, classes, widths, allocation identities and lifetimes against
that invocation. Inherited controls remain caller-owned and separate
from callee slots. Distinct contexts have distinct dynamic frame
identities even when their static descriptor id strings agree. Identical
descriptor bytes do not authorize shared physical storage; §9.4's
live-range and disjointness rules remain required. A standalone reference
record uses frame for its explicitly bound invocation or a descriptor
proved uniform over its declared invocation scope.
```

Here `context_key` uses the same spelling as the first-pass resource and
fault proposals. Their arrays and `frame_contexts` must cover the same
reached context set when applicable, rather than enumerate independently.
The originating bead's field spelling `context` is superseded by this
consistent proposed spelling; neither is published yet.

## Hashes, catalog and consumers

No callee clone, source/interface/closure hash or catalog change is
required. Certificate bytes change. Existing schedules already distinguish
the two source call points; no new call edge is invented. Schedulers,
specialisers and checkers must retain full context keys across linked and
default-covered calls. Estimator frames and resource reports must select
the same context. A uniformly valid c-expression stays usable as `frame`;
the repair must not force needless duplication of uniform certificates.

## Conformance vectors and verification

Use the pair/flip case; check both class records and all four outputs,
then inverse restoration. Reject two A records, a missing default call,
duplicate/foreign key, both `frame` and `frame_contexts`, neither field,
and a width from another token budget at the same variant/control count.
Add a homogeneous two-call control accepted with old `frame`, and combine
one context with 4ta fragments. Finite output/resource arithmetic is
checked in [INDEX](INDEX.md). The sibling test was read, not run; its
current `Unsupported` expectation documents the gap, not a completed fix.

## Release and route

Recommend a **0.9 K certificate-context repair**, coordinated with
[cwq](b-isa-spec-cwq.md) and [jw7](b-isa-spec-jw7.md). Keep v2v open until
the encoding and consumer tests are accepted. A 0.10 deferral must retain
this explicit blocker; it cannot restrict the admissible call graph.

# cwq — resource contexts must select the same frame and invocation

Incremental proposal for `b-isa-spec-cwq`. The defect, alternatives,
flip witness and exact `resource_contexts` proposal already appear in
[the first-pass resource note](../2026-11-02-contract-checker-resource-contexts.md).
This note adds frame/class integration and attribution precision.

## Defect and North Star scope

ISA 0.9 §8.2 defines accounting for an event list; §12.1 defines every
reached context; §12.3 publishes flat `resources`/`gates_used` without
selecting an invocation or defining an aggregation. An exact count for
one call is not a common resource bound like §12.3's error bounds.
This remains the same C4/C5 defect. North Star §§4.3/4.4/5.1 promises
the complete resource vector for general nested/controlled code; sums,
maxima or omission of contexts cannot silently replace that contract.

## Alternatives and worked witness

Reuse root a(t,c) calling flip once normally and once under c. At
unit duration/rate, the two invocation footprints have time 1 each,
one X-family leaf each, and live-wire footprints 1 and 2. Root t and c
are live throughout two slots: time=2, occupancy=4, peak=2. Adding
the two child peaks gives 3, which is not root peak. Adding child
occupancy to root occupancy counts wires/leaves again. Interface wire
**ownership charging** stays with the caller under §8.2; child views
are independently checkable invocation footprints, not extra charges.

Additional witness from [v2v](b-isa-spec-v2v.md): two forward, uncontrolled
flip contexts on A and B have identical time/count and different class
rows. A context key must choose both the right frame and resources.
[4ta](b-isa-spec-4ta.md) additionally requires per-bit fragment classes
within one of those frames. A variant/control-only resource key cannot
resolve either difference in general.

| Alternative | Result |
|---|---|
| Sum/max flat maps or choose a convenient context | Already rejected by the first pass; exact replay remains ambiguous. |
| Per-context reports with a deterministic legacy projection | Keep the first-pass recommendation. |
| Add child reports as costs to the root | Double counting; contradicts leaf-once aggregation. |

## Recommended exact spec diff

Apply the first-pass `resource_contexts:[{context_key,resources,
gates_used,estimate?}]` and first-canonical-context projection. Append:

```text
Resolve every resource_contexts entry from exactly the invocation tree
used for §12.1 claims and frame_contexts. Its key selects that context's
resolved frame, placements, borrowed lifetimes, controls, budget and
choices before any accounting. Class-fragment slots are expanded to
their actual bit classes. Validate resources for all declared target
classes, including zero-valued unused class rows where K12 applies;
never merge class rows into a replacement scalar.

Each context report describes its invocation subtree's event stream
and wire footprint. At a call boundary, interface wires remain charged
to their caller under §8.2; displaying them in a child invocation view
does not create a second billable allocation or event. Borrow charges
replace the owner's charge during the borrower interval as specified
there. Root totals are recomputed from the root event/live-interval
stream once. Neither resources nor gates_used are obtained by adding
the root view to its overlapping child views. Inclusive error-bound
multiplicities do not define a resource aggregation rule.

Missing, foreign or mismatched frame/resource contexts reject the
asserted report. Different record arrays must not be independently
zipped by position as an alternative to comparing their context keys.
```

This makes the first-pass “callee views” precise while retaining caller
ownership, borrowed-rate replacement and exact K12 outward display rules.
It introduces no new optimizer objective, routing or physical model.

## Hashes, catalog and consumers

The certificate/report changes bytes; source, target, schedule and ref
catalog need not. New model/alias/target changes are separate dependencies,
not inherent resource-context hash changes. Scheduler, estimator and
checker must retain identical keys and class rows, and must not copy an
estimate from the first context to another with equal variant/controls.

## Conformance vectors and verification

Reuse first-pass flip/root values; add A/B callee frames, a fragmented
slot, an idle class, a dirty borrow interval with owner-charge replacement,
and different budgets under one variant/control key. Reject swapped keys,
missing default-covered call, copied first-context estimate, or root-plus-
children totals. Exact footprint/root arithmetic is checked in
[INDEX](INDEX.md); no new scheduler/checker or sibling gate was run.
Origin read: checker `.tmp/CROSS_REPO_BEADS.md`, item 4, and sched's
two-class test named in v2v.

## Release and route

Recommend the **existing 0.9 K resource-context repair**, with v2v/4ta
and the shared context convention accepted together. Keep cwq open;
this note adds integration requirements, not another competing schema.

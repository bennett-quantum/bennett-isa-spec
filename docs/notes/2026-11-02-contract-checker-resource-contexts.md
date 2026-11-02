# Checker question 4 — resources for repeated invocation contexts

Proposal for council review; source request 4 in
`../bennett-conformance/checker/.tmp/CROSS_REPO_BEADS.md`.

## Defect and North Star scope

ISA §12.1 enumerates every reached invocation context, including default
coverage; §§11/12.3 allow the same region under different controls,
arguments, budgets and schedule choices. Error entries are common bounds
keyed by variant/control count with explicit multiplicity composition.
But §12.3 region.resources and gates_used are flat maps. §8.2 defines
accounting for an event list, without choosing which invocation's list
these maps describe or declaring a max/sum/common bound interpretation.
K12's estimate must bind the reported resources, so it cannot resolve
this choice either.

This is an S0 contract gap for repeated contexts under rubric C4/C5.
North Star §§4.3–4.4/5.1 requires the full resource vector and
substitutable scheduler/checker interfaces. Ignoring repeated contexts,
rejecting controlled calls or silently taking one context would narrow
the promised machine. An explicitly unchecked flat report is honest
until the contract is repaired, but not a verified resource claim.

## Alternatives and worked witness

Take `@flip(t)` with body `x t`; root `@a(t,c)` executes
`action_call @flip (t) () () ()`, then
`ctl c action_call @flip (t) () () ()`. On a target whose native X
supports total control counts 0..1, duration 1, unit occupancy rate,
the two flip contexts have:

| Context | X count | Time | Qubit-time | Peak |
|---|---:|---:|---:|---:|
| forward, 0 inherited controls | 1 | 1 | 1 | 1 |
| forward, 1 inherited control | 1 | 1 | 2 | 2 |

The inherited control is live even when its basis value is zero; these
are schedule resource charges, not counts of classically enabled gates.
For the root with both interface wires held throughout two sequential
unit slots, makespan=2, occupancy=4, peak=2 and two X-family native
events. Summing child peaks gives 3, not the root's peak; adding the
root plus its children would count the same leaves twice. These values
are hand-worked under the stated target, not new checker execution.

Even a key of (variant, inherited_controls) is insufficient in general:
different resolved token budgets in nested oracle calls create different
expansion sizes under the same key. Use the full §12.1 context identity.

| Alternative | Assessment |
|---|---|
| Sum every invocation into the region's map | Counts aggregate work but does not define peak/time or let a checker replay individual invocations. |
| Common maximum bound per resource | Could be a separate upper-bound report; is not exact §8.2 accounting and needs a different assertion kind. |
| Pick one context implicitly | Depends on traversal/implementation choice; not reproducible. |
| Exact keyed contexts (recommended) | Reuse §12.1 identity and replay each invocation's event list; root totals remain independently defined. |

## Recommended exact spec diff

Add to §12.3 region records:

```text
resource_contexts?:[{context_key,resources,gates_used,estimate?}]
```

Append after the schema:

```text
For a region reached in exactly one invocation context, resources and
gates_used report that invocation's exact §8.2 scheduled event accounting.
For more than one invocation context, resource_contexts MUST contain
exactly one record for every §12.1 context, in that section's canonical
enumeration order. context_key identifies the actual invocation, not
merely its variant or control count. The checker reconstructs its
arguments, budget, controls, placements, cleanup choices, schedule/default
decisions and native leaves, and recomputes every declared resource and
gate count. An estimate in that record uses §10.1 encodings and binds
exactly that context's values.

For compatibility the unkeyed resources/gates_used/estimate fields remain
the projection of the first canonical context; they MUST equal it and
discharge no other context's obligation. Missing, duplicate, reordered,
foreign or mismatched context records reject the report. A standalone
region with one explicitly bound invocation uses the single-context rule.

A root report counts its realised event list once, including semantic
callee leaves. Child context reports describe their own invocation
subtrees under §8.2 attribution; they are independently checkable views,
not extra costs to add to the root. This rule introduces no second
aggregation by summing child peak/time maps. Error-bound multiplicities
remain governed by their existing separate composition rule.
```

The request's alternative of leaving unkeyed maps semantically unused
would also avoid false verification, but still leaves meaningless data
required. The deterministic first-context projection above pins their
remaining role. Context records are exact reports, not a new optimizer
objective or a claim that every context has identical costs.

## Hashes, catalog and consumers

Certificate bytes and any report digest change. Object closure hashes,
target bytes, schedule bytes and ref catalog need not change. The root
schedule already determines contexts; the schema makes that binding
visible in resource reports. Scheduler, estimator, plugin and checker
must preserve full keys, canonical order and class-qualified K12 keys.
An old multi-context flat report needs recomputation/migration, not an
arbitrary max or sum guessed by the checker.

## Conformance vectors and verification

Use both flip contexts and the root totals above; require control-count
0 and 1 to differ in occupancy/peak. Add two default-covered invocations,
repeated call occurrences, an lproc fixed instance, and differing nested
token budgets under the same variant/control key. Swap context records,
omit one, duplicate one, relabel its call_index, or mismatch its estimate:
reject. A single-context certificate keeps the old map form. The scalar
accounting witness is checked by
[contract-verification](2026-11-02-contract-verification.md); no new
scheduler/checker test was run in this job.

## Release and after acceptance

Recommend a **0.9 K certificate context repair** alongside the fault-rate
binding repair, without changing their distinct meanings. The existing
cross-repo filing request should carry this exact encoding decision.
After council acceptance publish schema and repeated-context fixtures,
then migrate producers and checker independently. This note alone does
not close that obligation.

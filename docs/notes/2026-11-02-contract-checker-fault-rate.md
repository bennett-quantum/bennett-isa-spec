# Checker question 3 — bind fault recounts to an invocation

Proposal for council review; source request 3 in
`../bennett-conformance/checker/.tmp/CROSS_REPO_BEADS.md`.

## Defect and North Star scope

ISA §12.3 `verification.fault_rate` contains domain, faults_by_code and
excluded counts, but no invocation. K13 gives each standalone claim its
own args/budget and a scheduled certificate its root invocation. Several
claims in the same region record can legitimately bind different args
or budgets. Neither §12.3 nor §12.1 says which one a flat fault_rate uses.
§5.2/§14.1 makes fault outcomes depend on the resolved token and args.
Thus a recount cannot be unambiguously replayed for a standalone region,
or for several reached scheduled invocation contexts.

This is an S0 contract gap under C5: counts for different invocations can
both be correct, and the schema does not select one. North Star §5.1
requires reported fault rates and §8 requires zero on acceptance domains.
Do not drop the rate, presume a convenient budget, or label an unchecked
count verified. These remain obligations for the complete demo ladder.

## Alternatives and worked witness

A memoryless oracle with x:wire[2], target:wire[2], fault:wire[4],
classical a:i2 has body `r = lift.add x a; lift.retval r`.
For args=["1"] and budget ["1","0","0","0"], every input reaches
the step limit before retval: 256 full interface tuples give
`faults_by_code:{"1":"256"}`. With the same args and budget
["2","0","0","0"], all 256 succeed, so faults_by_code is empty.
The input/output target and fault words vary over all values. A nonzero
incoming fault word is not an execution fault: recount code(F_B), not
whether the final XOR-accumulated fault word happens to be nonzero.

If the region carries claims for both budgets, a flat `fault_rate` can
mean either result. Neither the first claim nor the root invocation of
an unrelated caller may be silently selected.

| Alternative | Assessment |
|---|---|
| Inherit from an arbitrary claim | Ambiguous; changes when claims are reordered. |
| Restrict every region to one claim/invocation | Removes existing certificate expressiveness. |
| Bind each rate explicitly (recommended) | Recounts exactly one invocation/domain, with context keys where scheduled. |

## Recommended exact spec diff

Extend §12.3's existing standalone record to:

```text
fault_rate:{invocation:{args,budget},domain,faults_by_code,
            undefined_excluded,layout_dependent_excluded}
fault_rate_contexts?:[{context_key,invocation:{args,budget},domain,
                      faults_by_code,undefined_excluded,
                      layout_dependent_excluded}]
```

Append:

```text
Every fault_rate recount binds its own invocation by K13's exact typed
args, budget, validation and encoding rules. It inherits no claim's
invocation. Counts are recomputed from F_B on this invocation and the
declared domain, with the existing independently validated exclusion
classes reported separately; a nonzero incoming fault-register word is
not a fault event. Counts are nonnegative exact integers and cannot
exceed the enumerated applicable domain count.

For a scheduled region reached in several §12.1 contexts, report exactly
one fault_rate_contexts record for each such context, in canonical context
enumeration order. Each explicit invocation MUST equal that context's
resolved values. The legacy fault_rate record is the first context's
record with context_key removed; it is a deterministic projection and
discharges only that context. With one context the legacy record suffices
and MUST bind that context. Standalone records have their explicit
invocation and no implied context. Missing, duplicated or conflicting
bindings reject the requested recount; old unbound rate data cannot
discharge a verified fault-rate obligation.
```

A future arrays-only schema may remove the projection. This proposal
retains the required old field while making its meaning deterministic.
It does not change oracle fault semantics, exclusion classification or
K13's distinction between token tuple and process-local id.

## Hashes, catalog and consumers

Certificates gain invocation/context bytes and change identity; the
object/region closure, target, schedule and ref catalog do not change.
Plugin and checker must bind the same table entries/actual token tuples.
Rung gates must select the exact invocation for their zero-fault check.
No resource or fault result for one budget covers a different budget by
guessing; monotonic success can be a separate proved statement.

## Conformance vectors and verification

Use both 256-tuple budgets above, reordered claims, args=[], tokens with
equal tuples/different ids, missing budget, conflicting scheduled context,
and duplicate/omitted context records. Check all fault values, including
an incoming word 1: budget-1 failure XORs it to 0 but is still code 1;
budget-2 success leaves it 1 but is not a fault. Counts for undefined
and layout-dependent exclusions must retain their own justified counts.
Scalar counts/XOR examples are checked by
[contract-verification](2026-11-02-contract-verification.md). The full
reference recount is a proposed acceptance test, not executed here.

## Release and after acceptance

Recommend a **0.9 K certificate binding repair**. The orchestrator's
existing cross-repo filing request should point at this exact change;
leave the obligation open until council accepts the encoding and checker
and producer owners implement the same standalone/context rules.

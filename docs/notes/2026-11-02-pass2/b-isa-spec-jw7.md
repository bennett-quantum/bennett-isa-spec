# jw7 — fault-rate binding, coordinated with applicability and context

Incremental proposal for `b-isa-spec-jw7`. Incorporate
[the first-pass fault-rate derivation](../2026-11-02-contract-checker-fault-rate.md),
including its two-budget witness and explicit standalone/context schemas.

## Defect and North Star scope

ISA 0.9 §12.3's `fault_rate` has a domain and counts but no invocation;
claims in the same record may bind different arguments/budgets by K13.
§12.1 also reaches multiple scheduled contexts. Which F_B is recounted
is therefore undefined. This remains the same C5 contract gap. North
Star §§5.1/8 requires the reported rate for the actual declared acceptance
invocation; it cannot be dropped or guessed from a convenient claim.

## Alternatives and worked witness

The first pass's two-row oracle (`lift.add`, `lift.retval`) at fixed
argument 1 and two-bit input/target, four-bit fault has 256 full interface
tuples. B=1 faults on all 256; B=2 succeeds on all 256. The same region
can carry claims for both. Reordering the claims cannot select the census.
With incoming fault word 1, the failing invocation leaves fault word 0
and the successful one leaves word 1: recount **code(F_B)**, not final
nonzero words. Claim inheritance, one-claim restrictions and an implicit
root budget retain the first pass's rejected status; explicit per-rate
binding remains recommended.

The missing second-pass interaction is [0jq](b-isa-spec-0jq.md): an
action's structural-only packet should have no own fault census. Making
the field optional must not make an oracle's requested census optional
to verify. [v2v](b-isa-spec-v2v.md)'s frame selector must resolve the same
invocation as this census.

## Recommended exact spec diff

Apply the first-pass `fault_rate.invocation` and `fault_rate_contexts`
text. Add to §12.3 immediately after the applicability repair:

```text
When a fault census is asserted for an oracle record, every supplied
rate binds its own K13 invocation and full §12.1 context where scheduled.
For a multi-context scheduled oracle, fault_rate_contexts covers every
reached invocation once in JCS context-key order; fault_rate is the
first-context projection specified above. Frame, resource and rate
arrays use the same bound invocation tree and context keys. A default-
covered invocation is not omitted because no schedule entry names it.

An action has no F_B outcome of its own. Absence of its fault_rate
is not a statement about nested oracles. Their rates, if asserted, are
recorded under their own closure/context identities. A structural
adjoint or borrow proof is not a fault census. Supplied counts must
still be independently recounted; absence cannot discharge a requested
zero-fault algorithm/rung acceptance obligation.

If token identity context is adopted, the bound invocation includes its
canonical alias relation wherever applicable; equal tuple encodings do
not silently substitute a different identity-dependent context.
```

The last paragraph depends on [ysw](b-isa-spec-ysw.md) acceptance; it is
not an unannounced change to current K13 bytes. For unbound legacy
reports, the first pass's unchecked census outcome remains honest
incompleteness, not approval of a complete executable certificate.

## Hashes, catalog and consumers

As in the first pass, certificate bytes change; existing source, closure,
target, schedule and ref catalog bytes do not change from this repair
alone. If ysw adds alias bindings, affected schedules also change under
that separate repair. Checkers, circuits, scheduler and rung harness must
select the identical invocation and preserve justified exclusion classes.

## Conformance vectors and verification

Reuse the full 256-tuple two-budget vectors and first-pass binding
negatives. Add a structural-only action with no rate, two nested oracle
contexts at B=1 and B=2, reordered claims, a default-only invocation, and
a census keyed to the other context's frame. Require independent counts
and reject a supplied conflicting invocation. The exact count/XOR
arithmetic is checked in [INDEX](INDEX.md); no new reference census or
checker implementation is claimed. Origin read: checker
`.tmp/CROSS_REPO_BEADS.md`, item 3.

## Release and route

Recommend the **existing 0.9 K fault-binding repair**, accepted with the
0jq applicability and common context rules. Keep jw7 open for that work;
do not duplicate the first-pass proposal or weaken fault-rate acceptance.

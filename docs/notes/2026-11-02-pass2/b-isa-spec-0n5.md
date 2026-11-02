# 0n5 — diagnostic portability is not a new semantic defect

Disposition for `b-isa-spec-0n5`. The first pass already resolves this
question in [checker diagnostics](../2026-11-02-contract-checker-diagnostics.md).
Keep that disposition; this note adds the new proposals' report boundaries.

## Defect and North Star scope

ISA 0.9 §12 requires specified checker verdicts, not a global rejection-
code registry. §11 K55 separately mandates `E_SCHEDULE_INFEASIBLE` and
its multiple-maximal-entry diagnostic. §10 executor statuses and §5.4
oracle faults are other interfaces. Missing portable local-code spellings
does not make two rejecting checkers compute different machines.
Rubric C5/C8 therefore supports an optional API/documentation improvement,
not an established S0 contract defect. North Star §§4.4/5.1 requires
independent checking and honest results; it does not prescribe E_CERT codes.

## Alternatives and worked witness

Reuse the first pass's missing-expected and duplicate-domain-component
certificates: both must reject, whatever their documented local code.
Additional boundary: a legal region[63,1] domain with `cells:"all"` and
sample count 1 can exceed host storage. K75 mandates **unchecked (host
limit)**, not rejection or successful evidence. A report adapter that
maps every non-proof to `rejected` would change a normative outcome even
if its local code were stable. A B5 schema error in an old binary is
likewise not an executed certificate proof failure; see [552](b-isa-spec-552.md).

| Alternative | Assessment |
|---|---|
| Test verdicts/clauses and implementation-local codes separately | Preserves current requirements. Recommended. |
| Publish an optional versioned report adapter | Useful when applications need common paths/codes. |
| Impose an inferred universal E_CERT registry | Adds an unstated interface and can reject conformant checkers. |

## Recommended exact spec diff

No semantic diff. Retain the first pass's optional §12.3 SHOULD wording
for documented local codes, failed obligation and certificate location.
If council adds a report-format appendix, add only this clarification:

```diff
+Report adapters preserve the specified distinctions between rejection,
+unchecked (host limit), unchecked implementation obligations, proof
+and evidence. A schema-admission error does not report a replayed proof
+failure. A local diagnostic code cannot change these outcomes or
+replace the executor and schedule codes mandated elsewhere.
```

“Unchecked implementation obligations” here describes disclosed tool
incompleteness; it is not a new permission to leave required verification
unfinished while claiming a complete conformant checker. No error-code
numbers or new verdict spellings are assigned by this note.

## Hashes, catalog and consumers

No object, interface, closure, target, schedule, certificate or catalog
byte changes are needed. A new optional report format has its own version.
Consumers should compare normative outcomes and clause locations across
checkers, and separately test each checker's documented code meanings.

## Conformance vectors and verification

Reuse first-pass negative certificates, add K75's legal host-limited
domain, K55's staged schedule tie and old-schema rejection versus replay.
The domain state size is 2^63 bits despite one requested sample; this
integer arithmetic is checked in [INDEX](INDEX.md). No huge domain is
allocated and no checker host-limit execution is claimed.
Origin read: `../bennett-conformance/checker/.tmp/CROSS_REPO_BEADS.md`, item 2.

## Release and route

**No new K semantic repair or 0.10 blocker.** Recommend council disposition
as optional diagnostics/report API guidance, as the first pass did.
The bead remains open for its owner's disposition; this notes-only job
does not change tracker status.

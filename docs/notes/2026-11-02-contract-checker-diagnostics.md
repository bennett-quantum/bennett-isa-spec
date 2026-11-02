# Checker question 2 — diagnostic-code ownership

Disposition proposal for council review; source request 2 in
`../bennett-conformance/checker/.tmp/CROSS_REPO_BEADS.md`.

## Defect classification and North Star scope

ISA §12 requires rejection of invalid certificates but defines no global
checker error-code registry. §11 separately mandates
`E_SCHEDULE_INFEASIBLE` and K55's multiple-maximal-entry diagnostic.
Launch §10 enumerates executor statuses; §5.4 enumerates program fault
outcomes. These are different interfaces, and the latter two lists do
not imply an unlisted E_CERT registry.

Two checkers rejecting the same invalid certificate under different
documented local codes compute the same specified verdict. The lack of
a registry is therefore **not an established contract defect**. It is a
possible diagnostics portability improvement (rubric C8/S1). North Star
§§4.4/5.1 requires independent checkability and honest labels, not a
particular spelling for every checker-local code. No capability is lost
by leaving those codes local. Inventing required codes would make
otherwise conformant checkers fail an unstated condition.

## Alternatives and concrete witness

Certificate A omits a table claim's required `expected`. Certificate B
has a complete table and invocation but repeats a component in its domain.
K13 requires A to reject; §12.2 K71 requires B to reject. Checker 1 may
use `E_CERT_RELATION` and `E_CERT_DOMAIN`; checker 2 may use
`missing_expected` and `duplicate_component`. All four diagnostic
spellings are illustrative local choices, not normative codes.

| Alternative | Assessment |
|---|---|
| Keep codes implementation-defined (recommended) | Preserve existing contracts; test local documented codes and normative verdicts separately. |
| Add a small report-location convention | Useful optional interoperability, without reserving every local code. |
| Publish a full ISA-wide E_CERT registry | New interface/versioning work; no evidence it is required for correctness. |
| Reuse launch statuses or quantum fault codes | Conflates checker rejection with run failure or quantum oracle outcome. |

## Recommended exact spec diff

No semantic change or mandatory rejection registry is required. If the
council wants explicit wording, append to §12.3:

```diff
+Checker diagnostic codes other than codes explicitly mandated elsewhere
+in this specification are implementation-defined. A checker SHOULD
+document its stable code meanings and report the failed normative
+obligation and certificate location with a rejection. Such codes are
+scoped by checker name and version; their spelling does not alter the
+verdict or any executor status/fault code. This convention does not
+replace mandated E_SCHEDULE_INFEASIBLE behavior or K55 diagnostics.
```

The original request's mandatory report schema would be a new API; it
should not be inferred from present text. An application needing that
API can version its checker-report format independently.

## Hashes, catalog and consumers

No source, object, target, catalog, schedule or certificate bytes need
change. Optional checker report fields affect only that report's format
and identity. Consumers must not compare an implementation's code against
an invented normative registry. A normative schedule failure still needs
its mandated code and validation stage.

## Conformance vectors and verification

Use A/B above as negative schema/domain tests; assert rejection and the
normative clause, and separately assert the implementation's documented
local code. For an otherwise valid schedule containing duplicate maximal
matching entries, retain K55's named diagnostic and staged rejection.
Do not turn a checker-local mismatch into a program fault or invalid
source verdict. This is a clause derivation; no checker test was run here.
The common document verification checks this disposition's required
sections and source links.

## Release and after acceptance

**No new K requirement and no 0.10 blocker.** Optional 0.9 editorial
clarification is sufficient. The orchestrator can disposition request 2
as documentation/API guidance; it should not file an ISA semantic bug
solely for missing E_CERT spellings. Existing checker tests may continue
to pin their own stable documented codes.

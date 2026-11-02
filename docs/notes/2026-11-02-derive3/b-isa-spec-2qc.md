# b-isa-spec-2qc — checker duty for qinplace coverage

Council proposal. References: [ISA 0.9](../../../spec/Bennett-ISA-Spec-0.9.md)
§4.5 K4, §§9.2–9.3, §12.1 “Invocation contexts”, §12.3 “Invocation
binding (K13)”, §16.3 example 7; related
[0jq owner proposal](../2026-11-02-pass2/b-isa-spec-0jq.md).

## Exact defect and North Star scope

§4.5 already requires an inverse-pair entry in the certificate record
of the region whose body contains qinplace. An unordered pair of exact
g/ginv closure hashes covers either orientation. Coverage also requires
entry.budget ≤ tk componentwise, exact token-argument tuples and
membership of the other arguments in the claim's args domain. The
executor must preflight every transitively executed qinplace before
any effect of a CC/R region call, or trap/fault 4.

§12.1 specifies verification of an entry's claim, and §12.3 specifies
binding resolution, but neither explicitly turns **coverage of sites**
into a checker duty. Validating a supplied inverse-pair proof and
checking that every applicable instruction has one are different tasks.
The request exposes this checker/executor responsibility ambiguity,
not a missing executor trap, inverse algorithm or proof owner. Resolving
all three questions needs static pair presence, bound applicability and
nested-context scope, rather than just “check inverse_pairs”.

North Star §§3.1, 4.2, 5.1 promises checked in-place bijections and honest
proof/evidence labels. Requiring those existing obligations at certificate
acceptance preserves that promise. A standalone proof with unresolved
call parameters may remain useful; it must not be labeled executable
acceptance or be rejected merely for lacking a schedule it never claims.

## Alternatives and concrete witnesses

Use §16.3 example 7, owner `addya`, with qinplace G/GINV, token tk and
classical argument a. The entry proves both compositions on the full
x,y domain, minimum tuple [3,0,0,0], a in 0..3.

| Case | Proposed checker result |
|---|---|
| No entry, or only {G,G} for a {G,GINV} site | Reject missing owner-local pair. |
| tk=[3,0,0,0], a=1, valid full-domain claim | Covered. |
| tk=[9,0,0,0], same a | Covered by primary-token monotonicity. |
| tk=[2,0,0,0] | Reject applicability for that bound invocation. |
| a=3 but the claim covers only 0..2 | Reject applicability. |
| G/GINV swapped in the entry | Covered by the same two-composition proof. |
| Correct entry, no resolvable standalone invocation | Report coverage unestablished; no executable-acceptance conclusion. |

For nesting, outer calls inner twice; inner owns the qinplace. Its tk
actuals are 3 then 2. An entry with minimum 3 covers the first context
and misses the second, even if both have the same variant/control count
and outer's root budget is 9. An entry in outer's claims array cannot
replace inner's. If a qinplace forwards a token argument k, an entry's
arg_tokens.k=[3,0,0,0] does not cover k=[4,0,0,0], however large tk is.

| Alternative | Consequence |
|---|---|
| Executor-only coverage | Safe execution still traps, but a checker can advertise an unusable certificate; requires an explicit limited verdict. |
| Check pair presence and root only | Catches common errors, misses the second inner context above. |
| Check owner presence plus every bound context | Aligns checker acceptance and existing preflight; recommended. |
| Require a universal proof for all possible calls of every standalone record | Unnecessarily rejects useful invocation-scoped/parameter-uniform claims. |

## Recommended exact spec diff

Insert after §12.3 Invocation binding's paragraph ending “not a target
realisation”, before CLAIM-BINDING witness:

```text
Qinplace coverage checking. For every qinplace source site in a supplied
region record, the checker MUST require an inverse_pairs entry in that
owner region's record for the exact pair of resolved g/ginv closure
hashes, in either orientation, and MUST reject a missing pair or invalid
referenced claim. Resolve imported owner records and library claims by
§9.3; indices name claims in their owner record, never in an outer caller.

When a root invocation is bound by bindings.invocation (or by a
standalone claim whose subject is that root), the checker MUST apply
§4.5's complete
covering test at every reached qinplace site in that invocation and its
transitive invocation contexts, rejecting any uncovered site. With a
schedule, use §12.1's complete realised invocation tree, including
default-covered calls, iterations and source-adjoint/control contexts;
without a schedule, use the bound reference invocation tree. Resolve
the site's tk and args from that context, not the root's tuple reused
for all callees. A disabled coherent predicate does not remove an
entered instruction's obligation. Entry claims must be established by
their specified exhaustive or linked-library rule; sampled agreement
does not cover. Do not treat two private generated oracle calls as new
owners of the containing source site's entry.

A standalone record with an entry but no resolvable invocation may
retain its established standalone claims. The checker MUST report its
qinplace applicability as unestablished and MUST NOT conclude executable
acceptance from it. Universal applicability may instead be established
by a checked parameter-uniform argument covering every admitted call.
If a required invocation, linked body or context cannot be resolved or
checked, report that obligation unestablished; an implementation work
limit is not proof of absence or applicability. Malformed bindings and
missing required pairs still reject. No successful verdict may imply
that an unestablished obligation was discharged.

These checks do not remove the executor's §4.5 pre-effect trap/fault
requirement. A verified result may be reused only for the closure,
entry proof and invocation/domain it actually covers; closure equality
alone does not establish applicability at another token/args binding.
```

“Every source site” makes the existing carried-entry requirement explicit
even for a zero-iteration source loop. Context-dependent applicability
is tested only at reached sites. This separates static inventory from
call-dependent coverage; it does not require executing an unreachable
quantum branch as a classical branch or searching all possible CC inputs.
The driver/executor supplies the actual root binding before use.

## Hashes, catalog and consumers

No source envelope, public signature, ref catalog or instruction expansion
changes. Correcting a missing entry changes certificate bytes and any
certificate digest. Wrongly accepted executable packets may now reject
or become explicitly incomplete; this is a compatibility decision, not
an invisible checker patch. Composition must retain each owner's claims
and root-relative contexts; a cache keyed only by closure cannot cache
call coverage. VM, specialiser, producer and independent checker owners
need the same applicability vectors. Fault code 4 and pre-effect timing
are already normative and do not change.

## Conformance vectors and actual verification

Ran the existing independent replay's `QinplaceCoverage` (eight tests)
with K13Witness (four), using [INDEX](INDEX.md)'s bounded command: exit 0,
12 passed. These exercise absent/wrong pairs, swapped orientation,
larger/shorter tk, args range and unbound handling. They do not exercise
the proposed complete nested traversal. The replay explicitly reports
non-root applicability unchecked. Its verdict is not a newly ratified
contract. The published checker's `docs/checker.md` separately states
that verification-index shapes do not discharge qinplace applicability.

Acceptance adds the two-inner-context witness, default-covered and
adjoint calls, qloop 0 versus repeated visits, inherited coherent control,
outer-only entry rejection, imported owner/library records, exact token
argument mismatch and sampled-claim noncoverage. For the executor check
fault 4 occurs before any region effect and preserves the reversible
prefix. A valid structural-only standalone claim must survive with
coverage explicitly unestablished. The finite INDEX predicate checks
the binding distinctions, not a new complete certificate checker.

## K entry and publication route

Recommend an explicit **0.9 K checker-obligation repair**, coordinated
with 0jq, if council authorizes the existing repair track. Proposed row:
`K(new) | §§4.5,12.3 | QINPLACE-CHECKER-COVERAGE: owner-local pair
presence plus all bound transitive contexts; unbound applicability is
unestablished; executor preflight retained | 2qc,0jq | checker semantics`.
Reserve no numeric K identifier. Under the published semantic-version
rule this belongs in **0.10** unless the council explicitly authorizes
an in-version repair; see [INDEX](INDEX.md). Keep 2qc open for that
decision and consumer implementations.

## Mixed fat object

[fat-object-scope.md](../2026-11-02-rev2/fat-object-scope.md) applies (North
Star §§2, 3.3, 4.4). Coverage begins at each selected embedded device
root, context [], and uses its linked owner records. Two native calls to
that root can bind different tk/args; neither inherits coverage merely
from the other's certificate. Native main, host-only libc and other
device roots need no artificial region inventory. The container-to-root
association, launch budget/handle negotiation and tape/run association
remain the exact boundary changes named in the shared note. This proposal
checks device invocations without requiring whole-binary lowering.

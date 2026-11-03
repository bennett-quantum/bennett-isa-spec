# 0jq — structural-only certificates and the owner of an inverse-pair proof

Council proposal for `b-isa-spec-0jq` (P1); no normative text is changed.

## Round-1 blocking items and resolution

Read the [round-1 report](../../../../bennett/docs/council/pass2-0jq-2026-11-02/round-1/report.md)
and both `out-sol.json` and `out-codex.json` in that directory. The table
maps every blocking item to this revision; it claims a proposed resolution,
not a new council verdict.

| Item | Change and witness |
|---|---|
| sol 0JQ-1; codex B1 — device-object/root inventory missing | Append the §12.3 mixed-object rule below and cross-reference it from §9.2. Witness: native main calls device A and B; the two object/root certificates inventory device owners only, with no main region. |

## Defect and North Star scope

ISA 0.9 §12.1 admits structural `adjoint_table` and `translation_lemma`
claims; §12.3 K13 explicitly admits standalone structural claims without a
schedule. However, §12.3's region schema requires `verification.oracle: i`
and `fault_rate`, without an applicability rule or a no-claim encoding.
An action has no fault outcome (§5.3), and an adjoint-table proof does not
assert an oracle relation or a fault census. In addition, §9.2 defines
`reference_program_hash` as **the oracle's** interface hash although the
required §12.3 binding also occurs in action certificates. These are
contract gaps (rubric C5/C8). The published controlled-action fixture
already uses `oracle:null`; that practice needs normative wording.

The ownership part of the request is **already substantially decided**:
§4.5 `qinplace` puts `inverse_pairs` in the certificate record of the
region whose body contains the instruction, including when that region is
nested. §9.3 requires linked certificate composition. K63's two call
occurrences and K65's private-cleanup exclusion do not move that owner to
either oracle or to an outer caller. A packet transport convention is
needed only to identify the linked owner record, not a new proof owner.

North Star §§4.2/4.4/5.1 requires general quantum actions, composable
certificates, and honest proof/evidence labels. Forcing a non-permutation
action to fabricate an oracle/T3 claim narrows that scope. The proposed
null encoding preserves each capability and every remaining obligation.

## Alternatives and worked witness

Take an action with body `h t`. The adjoint table establishes that its
source adjoint is `h t`; H²=I follows exactly from §4.4. It does not
establish a permutation assertion: H|0>=(|0>+|1>)/sqrt(2). Putting that
structural claim's index into `verification.oracle` would change its
meaning. A fault count of zero on an invented domain also asserts a
census that was never performed.

For ownership, let `outer` call `inner`, and let only `inner` contain
`qinplace @G @H tk x`. The covering entry belongs to
`regions[closure(inner)].verification.inverse_pairs`. Its `claim:i`
indexes **inner's** claims array. A same-numbered claim in outer is not
that claim. Two visits to inner resolve their own token/argument coverage;
a proof at budget 2 does not cover an argument-token tuple 3 merely
because the outer root uses 3. The existing componentwise bound for the
instruction's primary token and exact `arg_tokens` rule still apply.

| Alternative | Result |
|---|---|
| Require a dummy oracle claim or zero fault report | Misrepresents the assertion; reject. |
| Make the two verification entries nullable/conditional | Preserves structural-only packets and existing applicable checks. Recommended. |
| Put nested entries/claims in the outer region record | Contradicts §4.5's owner and makes claim indices ambiguous. |
| Resolve linked owner records by closure hash | Uses the existing `regions` map and §9.3 composition. Recommended. |

## Recommended exact spec diff

Replace §9.2's definition and the indicated §12.3 schema fragments:

```diff
-`reference_program_hash` = the oracle's interface hash.
+`reference_program_hash` = the interface hash of the certificate's
+root reference subject, oracle or action. It is not the hash of a
+claim's separate expected program. `closure_hash` remains required.

-verification: { oracle: i, ... fault_rate: { ... } }
+verification: { oracle: i | null, ... fault_rate: null | { ... } }
```

Append to §12.3:

```text
verification.oracle is null when this record makes no oracle/table/
program comparison claim. A non-null index must name an applicable
exhaustive or sampled comparison claim in this record's claims array;
its explicit expected relation determines its meaning. A structural
adjoint-table or translation claim is not such a comparison claim.

<!-- muse-fault-rate-paragraph -->
verification.fault_rate is required and has type null | RATE. For action and perm_action records it MUST be null and fault_rate_contexts MUST be absent. For an oracle record it MUST be a RATE whenever a fault census is asserted or is required to discharge an executable or scheduled acceptance claim; it MAY be null only when the record asserts no fault census, and null discharges no fault-rate obligation.
<!-- /muse-fault-rate-paragraph -->

For kind=oracle, verification.fault_rate MUST be a RATE whenever a fault census is asserted or is required to discharge an executable or scheduled acceptance claim (including every oracle in the complete invocation tree of a bound schedule, via fault_rate_contexts); it MAY be null only when the record asserts no fault census, and null discharges no fault-rate obligation and satisfies no zero-fault gate.

An action
has no fault outcome of its own; a nested oracle's census belongs to
that oracle and its resolved invocation. Applicable oracle-correctness,
fault-rate, borrowed-system and realisation obligations remain required
before an executable or algorithm acceptance claim can use them.
A non-null fault_rate must be validated even when no acceptance claim requires it.

The owner of a qinplace inverse-pair entry is exactly the region whose
source body contains that instruction (§4.5). After linking, select that
region's record by its exact closure hash in the composed regions map;
claim indices are local to that record. Imported proofs must be present
in the linked certificate, or reached by the existing checked
inverse_pair_library reference to its linked library certificate.
Conflicting records for one closure must not be chosen arbitrarily:
resolve them by §9.3 composition or reject the composition. An outer
record, an oracle callee record, and the two private qinplace call edges
cannot substitute for the owner. Before any effect of the enclosing CC/R region call, check coverage for
every transitively executed qinplace using its owner entry and resolved
token and classical arguments (§4.5). Failure is a CC trap or R
FAULT_UB_TRAP; validation cannot wait until after earlier effects.
```

This wording does not make an adjoint-table claim a proof of the
instruction's inverse-pair obligation, or turn numerical agreement into
an exact structural proof. `unitary` and `channel` retain their own
applicable claim types. Fault-rate invocation encoding is handled by
[jw7](../2026-11-02-pass2/b-isa-spec-jw7.md), not invented again here.

## Fat-object applicability

Incorporate [the shared scoping statement](../2026-11-02-rev2/fat-object-scope.md#citable-scoping-statement).
Append the following proposed normative paragraph to §12.3, cross-referenced from §9.2:

```text
For a mixed fat object, object_hash identifies the embedded or linked ISA
object, and bindings.closure_hash selects its certified device root.
reference_program_hash identifies that oracle or action root's interface,
not native main or a separate expected program. The container associates
each object/root pair with its certificate; several device roots may have
separate certificates. Each composed regions map contains the required
device-region owner records and checked linked-library proofs. Native-only
host code, argv/envp and host-only libc require no region record, oracle
comparison, fault census or inverse-pair entry.
```

The shared statement lists today's assumptions and the exact remaining
container, launch, budget, handle, tape and host-memory contract changes.
This note does not assume that the device object is the entire binary.

## Hashes, catalog and consumers

No source, interface, closure, target, schedule or `ref@0.9` catalog bytes
need change under an authorized in-version repair. Replacing an inapplicable
fault report with null changes certificate bytes. Linkers,
specialisers and checkers must preserve owner-local claim indices and
resolve linked library proofs before use. Old ambiguous standalone action
bindings need explicit validation, not silent reinterpretation.

## Conformance vectors and verification

Require structural H and translation-only packets with `oracle:null` and
no fault census; reject a structural index masquerading as an oracle
comparison. An absent census must not satisfy a rung's zero-fault gate.
For outer→inner→qinplace, accept the covering entry in inner; reject an
outer-only entry, dangling library reference, changed G/H closure,
uncovered token argument and claim index into another record. Test two
inner contexts, swapped G/H, inherited control, and default-covered calls.

The H² and non-permutation facts are exact algebra checked by
[check_rev2.py](../2026-11-02-rev2/check_rev2.py). The originating note cites:
`../bennett-circuits/docs/design/2026-11-01-m5-contract-requests.md`,
“Structural-only packets”. No new packet checker is implemented here. A composed root packet may
contain inner's correctly keyed owner record; packet containment does not
move the entry into the outer region record. For the published controlled-action
fixture migration, the root table claim at index 0 gives verification.oracle:0;
the numerical-only flip record retains null, and both action fault-rate
placeholders become null. These are proposed certificate changes only.

## Release and route

Recommend a **0.9 K applicability/hash-definition repair**. The owner rule
needs clarification and link/consumer tests, not a new proof mechanism.
Keep 0jq open for council acceptance and producer/checker implementation.
Maintainer D2 (2026-11-03) authorizes this in-version 0.9 K route only
after the joint 0jq/checker-fault-rate revision passes sol+astra and a
Muse seat accepts. Until both conditions hold, 0jq remains HELD and is
not ACCEPTED FOR SPEC. The publication rules below apply.

## Publication and validation boundary

This is a revision proposal, not council acceptance or a consumer implementation.
The default publication rule in spec/README.md requires a full version bump
for semantic changes. The recommended 0.9 K route has conditional maintainer
authorization under D2; it still requires the joint sol+astra round and
Muse acceptance stated above. This note grants no council acceptance. All unchanged-hash claims assume unchanged header/source
and catalog envelopes. A full version publication recomputes affected symbol,
object, target, schedule, certificate and versioned ref-library identities and
consumer pins even when recipe bodies stay fixed. Published 0.9 bytes remain
available; older closed-schema readers require an upgrade for new fields.

The reproducible finite and document checks are in [check_rev2.py](../2026-11-02-rev2/check_rev2.py):
`timeout 120 python3 -I -B /home/tobias/Projects/bennett-workspace/bennett-isa-spec/docs/notes/2026-11-02-rev2/check_rev2.py`.
Executed for this revision: exit 0, all eight PASS result lines.
These finite checks verify witnesses and revision structure; they implement
no proposed-format checker or host shim. No full repository gate or sibling gate is claimed: scripts/pre-push
uses git/worktree writes, forbidden by this runner, and its setup changes paths
outside this job. Underlying beads remain open for publication and consumers.

## Muse blocking items and resolution

This joint revision starts from the latest graded notes: 0jq round 2
(`19ffcda5e44a87d0878c89e357fa4ccea163d50883e031aff781fb3e43bd923e`)
and checker-fault-rate round 3
(`71a74c494872f94fa40271dd0d81ef140fa01d6095006ad859400a5b5037a189`).
It proposes resolutions, not council verdicts. Both notes carry the same
Muse paragraph above and the same conditional oracle sentence, copied
verbatim from [out-muse.json](../../../../bennett/docs/council/accept-muse-batch2-2026-11-02/round-1/out-muse.json).
RATE abbreviates `{invocation:{args,budget},domain,faults_by_code,
undefined_excluded,layout_dependent_excluded,exclusion_evidence?}`.
A non-null RATE binds its own exact invocation and full oracle domain;
it never inherits a convenient claim's budget. For a bound schedule,
every complete-tree oracle context, including defaults and predicated-off
calls, needs its recount; the legacy field is the first canonical
context's projection. Exclusions do not shrink the reference census.
The complete context and evidence contract is retained in
[checker-fault-rate.md](checker-fault-rate.md); these requirements neither
invent a census for a structural-only packet nor relax acceptance gates.

| Blocking item | Resolution and concrete witness |
|---|---|
| 0jq-M1 | The closed schema requires `fault_rate: null | RATE`; actions use null and omit `fault_rate_contexts`. Both action fault-rate placeholders become null. A missing key is malformed, and null proves no zero-fault result. The reference_program_hash definition and owner-local inverse-pair rules are unchanged. |
| checker-fault-rate-M1 | Oracle RATE is conditional on an asserted/required census, including every oracle context in a bound schedule. A structural-only oracle may use null; it discharges no census or zero-fault gate. All accepted recount, deterministic context projection and classifier-bundle rules remain. |

There are no other Muse blocking items for these two proposals. The
accepted mixed-fat-object scope, hash conditions and prior-round fixes
remain. Relative witness links now point to their original directories;
historical verification reports above remain historical.

### Structural-only oracle conformance vector

Take a standalone oracle with body `r = lift.add x a; lift.retval r` and
a checked structural `adjoint_table`/translation claim only, no comparison
claim, no asserted census and no bound schedule. Supply all ordinary
identity, signature, frame and claim fields as required. The following
JSON is the verification fragment under test, not a complete certificate
or a newly introduced certificate field:

<!-- structural-only-oracle -->
```json
{"kind":"oracle","verification":{"oracle":null,"fault_rate":null}}
```

| Request for the same structurally valid packet | Expected result |
|---|---|
| Standalone structural validation, no census asserted | Accept as a structural packet. |
| Executable acceptance requiring a fault census | Reject that acceptance claim: null discharges no census obligation. |
| Scheduled acceptance requiring all oracle contexts, or a zero-fault gate | Reject that acceptance claim: no RATE/context coverage is supplied. |
| Remove fault_rate instead of using null | Reject schema validation: required key missing. |
| Assert a census while retaining null | Reject the asserted census. |

With the same body, args=["1"] and B=1, a verified RATE counts
256 code-1 faults over the full 4×4×16 domain; with B=2 it counts zero.
Those are actual census results and cannot be inferred from the null
packet. The focused script checks these applicability outcomes, action
null versus omitted keys, the fixture migration, unchanged owner rules
and the retained recount/bundle text. It is a finite contract test,
not an implementation of the proposed production certificate checker.

### Paragraph byte identity and verification

Extract exactly the UTF-8 bytes between the two `muse-fault-rate-paragraph`
markers, excluding the marker lines and the paragraph's terminating LF.
There is no line reflow or Unicode normalization. These are the paragraph
bytes quoted by Muse in 0jq-M1; the extra conditional oracle sentence
is copied verbatim from checker-fault-rate-M1 in both notes.

| Note | Shared paragraph SHA-256 |
|---|---|
| 0jq.md | `b34376b04d0d2e295018245c1f7f3ce36b4f55ad5ed18e8f127119e36b9685b0` |
| checker-fault-rate.md | `b34376b04d0d2e295018245c1f7f3ce36b4f55ad5ed18e8f127119e36b9685b0` |

Verification: `timeout 30s python3 -I -B /home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp/sol-notefix-s4/check_notes.py joint`
exited 0. Both extracted hashes equal the values above and Muse's exact
paragraph. No source, fixture, spec or catalog was edited.

Every producer and closed-schema reader using these amended 0.9
certificate bytes must upgrade together for the required nullable field
and context/evidence fields; older readers cannot silently accept them.
Remaining council work: joint sol+astra review, then a Muse acceptance
seat. D2 keeps 0jq HELD until those conditions pass. Publication and
producer/checker implementation follow acceptance; this revision performs
none of those steps and reserves no K number.

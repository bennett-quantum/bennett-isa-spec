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
absence encoding preserves each capability and every remaining obligation.

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
+verification: { oracle: i | null, ... fault_rate?: { ... } }
```

Append to §12.3:

```text
verification.oracle is null when this record makes no oracle/table/
program comparison claim. A non-null index must name an applicable
exhaustive or sampled comparison claim in this record's claims array;
its explicit expected relation determines its meaning. A structural
adjoint-table or translation claim is not such a comparison claim.
fault_rate may be absent when no oracle fault census is asserted.
Its absence asserts neither zero faults nor a checked census. For action and perm_action records, fault_rate MUST be absent. An action
has no fault outcome of its own; a nested oracle's census belongs to
that oracle and its resolved invocation. Applicable oracle-correctness,
fault-rate, borrowed-system and realisation obligations remain required
before an executable or algorithm acceptance claim can use them.
An explicitly supplied fault_rate must be validated even when optional.

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

Incorporate [the shared scoping statement](fat-object-scope.md#citable-scoping-statement).
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
need change under an authorized in-version repair. Removing an inapplicable
fault report or choosing null changes certificate bytes. Linkers,
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
[check_rev2.py](check_rev2.py). The originating note cites:
`../bennett-circuits/docs/design/2026-11-01-m5-contract-requests.md`,
“Structural-only packets”. No new packet checker is implemented here. A composed root packet may
contain inner's correctly keyed owner record; packet containment does not
move the entry into the outer region record. For the published controlled-action
fixture migration, the root table claim at index 0 gives verification.oracle:0;
the numerical-only flip record retains null, and both action fault-rate
placeholders are omitted. These are proposed certificate changes only.

## Release and route

Recommend a **0.9 K applicability/hash-definition repair**. The owner rule
needs clarification and link/consumer tests, not a new proof mechanism.
Keep 0jq open for council acceptance and producer/checker implementation.
The publication rules below apply.

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

# ysw — tuple values do not encode token identity

Council proposal for `b-isa-spec-ysw`.

## Round-1 blocking items and resolution

Read the [round-1 report](../../../../bennett/docs/council/pass2-ysw-2026-11-02/round-1/report.md)
and both `out-sol.json` and `out-codex.json` in that directory. The table
maps every blocking item to this revision; it claims a proposed resolution,
not a new council verdict.

| Item | Change and witness |
|---|---|
| codex B1 — action_call has no separate invocation token (§4.7) | Replace the blanket region ordering and both ABI defaults: oracle prepends tk; action/perm_action/CC/R lists token parameters only. Witness: token-free flip has []; token-free oracle has [0]; an action(a,b) has [0,0] or [0,1]. |
| sol B1; codex B2 — missing mixed-object launch/certificate scope | Add the §§10/12.3 paragraph below. The device launch derives aliases from that program's valid token handles; native main/argv/libc have no device-alias inventory. |

## Defect and North Star scope

ISA 0.9 §5.3 matched pairs require equal token **identities and tuples**;
§5.4 mints fresh identities even at equal tuples. §4.1/§7.4 `uncall`
compares recorded boundary values. But K13 (§12.3), K12 (§10.1) and
§11 schedule bindings serialize token arguments only as tuples. They
cannot distinguish alias relations that affect execution and K81 cleanup
pair formation. This is a C3/C5 context-identity gap, separate from the
correct canonical tuple encoding and from token-to-oracle diagnostics.

North Star §§4.2/4.4/5.1 requires deterministic reversible execution,
parameterised scheduling and substitutable estimates. Requiring all equal
tuples to be one token, or banning repeated/equal token arguments, removes
currently admitted capabilities. Preserve identity semantics and encode
equality without serializing process-local handles.

## Alternatives and worked witness

Origin: `../bennett-estimate/tests/fixtures/token-alias-source-gap.bisa`
and `.tmp/sol-estimate/cross-repo-beads.md`, EST-CONTEXT-03. Its CC root
is `call @sink a; uncall @sink (b) (); ret`, with an R sink containing
only `ret`. Passing the same valid token at a and b takes four source
steps (call, sink ret, uncall, root ret). Two fresh identities with the
same tuple fault 4 at the third step. In both cases args is
`[["1","0","0","0"],["1","0","0","0"]]`, hence params_hash agrees.
These counts follow §7.4 by hand; they are not a reproduced VM run.

For an oracle estimate with its separately supplied primary token T0, the
three identities (T0,a,b) can have aliases `[0,1,1]` or `[0,1,2]` while
all tuples agree. Forwarding a preserves its class; a successful new
`budget.mint` must create a class distinct from both. In a CC entry with
no separately supplied primary token, the corresponding a/b arrays are
`[0,0]` and `[0,1]`. The bead's unconditional “position zero is primary”
must distinguish oracle invocations from actions and CC/R entries.

| Alternative | Assessment |
|---|---|
| Change token equality everywhere to tuple equality | Changes matched-pair/uncall semantics and fresh-mint meaning. Not a serialization repair. |
| Serialize process ids | Nonportable, invalid across loads and competing implementations. |
| Canonical equality partition alongside tuples | Preserves existing semantics and every alias pattern. Recommended. |
| Compute a worst-case estimate over all partitions | Useful explicitly conditional bound, not an exact estimate of a selected identity context. |

## Recommended exact spec diff

Add to §12.3 invocation bindings and §11 schedule bindings
`token_aliases:[nonnegative integer,...]`, with this rule:

```text
A resolved invocation carries a canonical token equality partition.
For an oracle invocation list its separate invocation token first, then
token-typed args in signature order. For an action, perm_action or CC/R
entry list only token-typed parameters in signature order: §4.7 action_call
has no separate invocation token. Budget fields do not create identities.
The separate token accepted by the estimation ABI supplies the budget,
but contributes an identity slot only for an oracle subject.
Equal labels denote one valid token identity and require equal tuples;
unequal labels denote distinct identities even when tuples agree.
Labels are assigned by first occurrence, starting at 0, without gaps.
An empty token list has []. A token-free oracle has [0]; a token-free
action/perm_action/CC/R entry has []. An action with a,b uses [0,0] for
one shared token identity and [0,1] for two distinct identities.
No process-local handle is serialized. params_hash remains SHA-256 of
the typed args array alone. Full invocation/schedule/cache matching
additionally compares token_aliases; params_hash alone is insufficient.

Executors derive this partition from actual live handles. Tuple-only
requests with token arguments must explicitly supply the partition;
malformed labels, wrong arity or tuple-inconsistent equality are E_ARG
before producing an estimate or performing an effect. Every consistent
partition is admitted. Source forwarding, lexical captures, lproc fixed
bindings and adjoint/inverse traversal preserve resolved identities.
A successful mint creates a fresh identity attached to that dynamic
defining occurrence; it is not reconstructed by tuple interning.
Nested contexts retain the identity mapping and canonicalize their own
ordered partition from it; equal local label numbers in two contexts
do not by themselves denote one global token.

This source-binding identity context is explicit metadata outside the
closed catalog JSON and its positional value arguments. K24's token
values and K77's exactly-one-field oracle_invocation token channel keep
their tuple encoding. Builders and captures additionally preserve the
identity context so source matching is not performed on tuple equality.

Claims used in an identity-dependent context bind its partition too.
Parameter-uniform structural proofs quantify over every consistent
partition as well as every argument vector/budget. Inverse-pair entries
bind the primary/argument token partition when relevant; coverage must
preserve it and all its tuple-equality constraints alongside the existing
primary-budget inequality and exact arg_tokens tests. A proof for one
partition cannot be reused for another merely because params_hash agrees.
```

Add an ABI entry rather than changing the existing function in place:

```c
bennett_status bennett_estimate_run_with_token_aliases(
    bennett_program_t*, const char* region, bennett_token,
    const char* variant, uint32_t inherited_controls,
    const uint8_t* params_jcs, size_t params_len,
    const uint8_t* aliases_jcs, size_t aliases_len, bennett_estimate_t** out);
bennett_status bennett_sched_request_set_token_aliases(
    bennett_sched_request_t*, const uint8_t* aliases_jcs, size_t aliases_len);
```

Both copy and validate the JCS array. Keep the old ABI callable; with no
token args infer [0] for an oracle and [] for an action/perm_action.
A legacy request with token arguments and no alias binding returns E_ARG;
the additive alias-aware function admits every consistent partition and
provides exact estimates of each resolved request. A bound over partitions
is not a K12 exact result. This request diagnostic does not make any valid
program or alias pattern illegal, and does not replace implementing the
full alias-aware capability. Publish
the extension under an explicit launch/sched ABI revision or feature
contract, including loader capability negotiation; do not change the
published C argument layout/symbol meaning silently.

## Fat-object applicability

Incorporate [the shared scoping statement](fat-object-scope.md#citable-scoping-statement).
Append the following proposed normative paragraph to §§10 and 12.3:

```text
In a mixed fat object, token_aliases binds invocations of the loaded
marked device ISA object and linked device closure. Execution derives
partitions from that program's valid live token handles and preserves
equality through device calls; tuple-based estimate/schedule requests
supply equality partitions explicitly. Serialization contains equality
classes, never host pointers or process-local handle ids. Alias-bearing
schedules and certificates bind that device object/root and invocation.
Native entry points, argv/envp and host-only libc need no device-alias
entries. The future mixed launch shim negotiates the alias-binding ABI
feature; §10 alone does not specify that shim or container interface.
```

The shared statement lists today's assumptions and the exact remaining
container, launch, budget, handle, tape and host-memory contract changes.
This note does not assume that the device object is the entire binary.

## Action and inverse-pair witnesses

The published token-free flip action's action_call has no token operand:
execution, estimates and replay all derive []. An action with token
parameters a,b has [0,0] for a=b or [0,1] for distinct equal-tuple tokens.
A CC sink wrapper has the same ordering. Only an oracle with primary T0
and token arguments a,b can use [0,1,1] versus [0,1,2].

For a Q witness use a memoryless identity oracle (`lift.retval x`) and an
action containing two consecutive oracle_call occurrences on the same
wires, with token arguments a then b. Both calls compose to identity.
The action's partition [0,0] permits the K81 matched pair and its K65
carrier at the second call; [0,1] supplies no such carrier. Each oracle
call's own context prepends its actual a or b token. No hypothetical
primary action token is introduced. A mint reversed and minted again at
the same tuple creates a new identity; it does not revive the revoked one.

An inverse-pair entry at primary budget (2,0,0,0), arg_tokens=(2,0,0,0)
and partition [0,0] does not cover primary (3,0,0,0) with that unchanged
argument tuple: [0,0] becomes tuple-inconsistent; [0,1] changes the
certified equality partition. The original componentwise budget condition
alone is insufficient when equal-token constraints also apply.

## Hashes, catalog and consumers

Canonical args and params_hash stay fixed, as do source envelopes and
catalog bytes. Alias-bearing schedule/certificate bytes change hashes;
cache keys must include the partition. A source-builder metadata repair
does not add a token formal to ref recipes or change their hash. Runtime,
estimator, scheduler, checker and circuits must independently preserve
identity through forwarding and fresh mints. ABI revision must allow old
callers/libraries to negotiate support explicitly.

## Conformance vectors and verification

Test same tuple/same handle versus same tuple/different handles, both CC
uncall and Q K81 pairing; compare identical params_hash with unequal
alias bindings. Reject `[1,1]`, `[0,2]`, negative labels, wrong arity
and equal labels at unequal tuples. Include primary/arg alias, three
tokens, forwarding through lprocs/body captures, fresh repeated mints,
reversal/revocation and reuse of a certificate from the wrong partition.
No valid consistent partition is a source error. The partition and hash
arithmetic is checked by [check_rev2.py](check_rev2.py). The originating VM object
hash is a reported pin, not a new execution result in this job.

## Release and route

Recommend a **0.9 K identity-binding repair with additive, explicitly
versioned ABI support**. If binary-interface publication waits for 0.10,
retain ysw as an exact-estimation/context blocker; conditional bounds do
not complete this capability. Coordinate fault/frame/resource bindings.

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

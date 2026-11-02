# ysw — tuple values do not encode token identity

Council proposal for `b-isa-spec-ysw`.

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

For a region estimate with a separately supplied primary token T0, the
three identities (T0,a,b) can have aliases `[0,1,1]` or `[0,1,2]` while
all tuples agree. Forwarding a preserves its class; a successful new
`budget.mint` must create a class distinct from both. In a CC entry with
no separately supplied primary token, the corresponding a/b arrays are
`[0,0]` and `[0,1]`. The bead's unconditional “position zero is primary”
needs this distinction outside region requests.

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
For region requests list the separately supplied invocation token first,
then token-typed args in signature order. For a CC/R entry with no
separate token, list only its token-typed parameters in signature order.
Equal labels denote one valid token identity and require equal tuples;
unequal labels denote distinct identities even when tuples agree.
Labels are assigned by first occurrence, starting at 0, without gaps.
An empty token list has []; a region with no token args has [0].
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
token args its partition is [0]. With ambiguous token args it cannot
claim an exact identity-dependent estimate without the new binding; an
explicitly uniform result may cover all consistent partitions. Publish
the extension under an explicit launch/sched ABI revision or feature
contract, including loader capability negotiation; do not change the
published C argument layout/symbol meaning silently.

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
arithmetic is checked in [INDEX](INDEX.md). The originating VM object
hash is a reported pin, not a new execution result in this job.

## Release and route

Recommend a **0.9 K identity-binding repair with additive, explicitly
versioned ABI support**. If binary-interface publication waits for 0.10,
retain ysw as an exact-estimation/context blocker; conditional bounds do
not complete this capability. Coordinate fault/frame/resource bindings.

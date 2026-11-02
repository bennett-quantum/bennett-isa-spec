# ojy — repair K82 fixture ordering to match normative body order

Disposition/proposal for `b-isa-spec-ojy`.

## Defect and North Star scope

ISA 0.9 §11.1 K82 defines the named interface “in body order”; §15
publishes the accepted K82 vector. The vector generator's `interface`
collects first uses in that order but returns `sorted(used)`.
Consequently its serialized arrays do not implement that clause.
An independent traversal in this job reproduced **31 of 54** differing
arrays, including `value.select`: body `[p,b,a,t]`, fixture `[a,b,p,t]`.
This is an implementation/fixture consistency defect, not the old unbound
wire defect already resolved by K82. The historical
[K82 derivation](../2026-11-01-ref-named-interface-k82.md) and first-pass
[b5b disposition](../2026-11-02-contract-b5b.md) remain the binding context.

North Star §§4.4/6.1 requires independent implementations of a common
contract. This mismatch threatens fixture/API interoperability; it does
not remove a machine capability, and correcting it needs no scope
restriction or catalog rewrite. The normative name-map semantics must
not become positional argument binding as a side effect.

## Alternatives and worked witness

For `value.select`, first uses are p, then b, then a, then t. A direct
map `{a:A,b:B,p:P,t:T}` binds those exact names regardless of JSON key
order. Reordering an **interface enumeration** must not bind p to A.
For p=0 the selected payload is b; for p=1 it is a. A positional zip
against sorted names could change that result and violates K82 today.

| Alternative | Assessment |
|---|---|
| Regenerate fixture arrays in body order | Follows accepted text, changes no semantic bindings. Recommended. |
| Change K82 to sorted order | Possible council editorial choice, but unnecessary divergence from current body-order consumers. |
| Treat arrays as unordered sets everywhere | Name lookup remains valid, but does not satisfy the published enumeration rule. |
| Rewrite recipes/formals | Unrelated to the defect; changes the catalog hash needlessly. |

## Recommended exact spec diff

```diff
# No normative spec change; retain §11.1 K82's body-order rule.
```

The fixture-owner patch should be exactly:

```diff
--- a/conformance/fixtures/accepted-k82-09/generate.py
+++ b/conformance/fixtures/accepted-k82-09/generate.py
@@
-    return sorted(used)
+    return used
```

Regenerate `k82.json` with that generator, changing the `interfaces`
arrays and affected `edges[].interface` arrays, and update any fixture
artifact checksum. Keep recipe order and unrelated sorted lists as they
are. Do not change interface maps, edge leaf digests or binding cases.

If council wants an explicit editorial gloss, append to K82:

```text
Body order is first free-wire occurrence in the written constructor
arrays, recursively in array order with duplicates removed at first
occurrence; lexical bindings retain their existing scope. Named-wire
actuals bind by name, never by the position of an interface enumeration.
```

A future named-interface invoke must account for its transitive free
names at that lexical site; the current 54 contain no such invoke. This
note's catalog count does not pretend to validate every future constructor.

## Hashes, catalog and consumers

`spec/ref-0.9.json`, its envelope hash, source envelopes, generated leaves,
ordinals and schedules/certificates remain unchanged. The fixture JSON
bytes and any checksum of that file change. The first-pass catalog hash
must not be confused with a hash of `k82.json`.

Consumer inspection: sched `src/ref_contract.rs` accumulates a first-use
Vec and binds each name in `named_scope`; reference's
`conformance/src/ref_language/named_interface.rs` also accumulates a Vec
and imports by name. They have a body-order enumeration API. VM's
`src/ref_library.rs` and `source.rs` were inspected, but this job did not
establish a public named-interface enumeration contract there. Do not
claim its order independence tested. VM, sched and reference owners
should run the corrected fixture; no competing consumer was edited.

## Conformance vectors and verification

The independent catalog traversal, in-memory generator correction and
original/corrected reference probe runs are recorded in [INDEX](INDEX.md).
They check 54 interfaces, 20 edges, 86 leaf digests and 16 binding cases;
the corrected fixture is generated only in job scratch. A successful
current `generate.py --check` proves self-consistency, not body-order
conformance. Add a direct `[p,b,a,t]` assertion and verify all name maps
and leaves survive array regeneration; reject a positional zip adapter.

## Release and route

**Existing 0.9 K82 fixture repair; no new K and no 0.10 deferral.**
If the optional wording is used, treat it as a K82 editorial clarification.
Keep ojy for the fixture owner/council disposition; sol-ref09's owned
implementation and all published fixtures stay untouched by this job.

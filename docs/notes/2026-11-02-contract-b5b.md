# b5b — implementation follow-up to accepted K82

Disposition proposal for council review; `b-isa-spec-b5b`.

## Defect classification and North Star scope

This bead is **not a new contract defect** in the current tree. ISA 0.9
§11.1 K82 is already accepted in round 10, and §15 points at
`conformance/fixtures/accepted-k82-09/`. The supplied bead and the prior
K82 proposal still describe the earlier pending state. K82 explicitly
imports exactly a named-interface recipe's free wire names, applies
K26/K27 wire-actual resolution, hides unrelated caller names and keeps
typed recipes/templates closed. It settles both reported divergences.

The reference stream's `.tmp/sol-ref09/PROGRESS.md` reports a current
implementation and red/green binding tests; this job does not repeat or
modify that owner's work. No North Star narrowing is needed. Rewriting
the catalog to compensate for a consumer's old binding behavior would
move an already-settled boundary and needlessly invalidate consumers.

## Alternatives and worked examples

Case 1: caller has wire z=r. Callee `open.local` has no interface names:
it allocates local z, applies X twice and releases z. K82 imports an empty
map, so the four leaves are `alloc #0; x #0[0]; x #0[0]; release #0`.
The unrelated caller r is untouched. Importing every caller wire would
spuriously report allocation shadowing.

Case 2: a K27 declaration q is acquired, then passed by its named-interface
wire-actual rule into `open.one` whose body is `x q[0]`. The interface is
exactly {q}; the active declaration resolves to its acquired wire. Follow
with X and relinquish: the same four leaves result. Not importing live
declarations would spuriously report unbound q. A declaration without
acquire still rejects; so does a typed recipe reading undeclared q.

| Alternative | Assessment |
|---|---|
| Implement accepted K82 (recommended) | Exact wire imports, no catalog or leaf drift. |
| Import all caller names | Captures unrelated wires and violates closure. |
| Rewrite helpers to typed formals | Possible future catalog revision, but unnecessary for this bead and changes its bytes/hash. |
| Add another permissive K exception | Duplicates or contradicts the accepted rule. |

## Recommended exact spec diff

```diff
# No change to spec/Bennett-ISA-Spec-0.9.md.
# K82 already states the needed binding rule and publishes its vector.
```

The implementation owner should replace legacy environment import with
the lexical interface set and ordinary wire-actual evaluation, including
active declaration bindings. This is an implementation disposition, not
authorization for this job to edit that code.

## Hashes, catalog and consumers

No object, catalog, ref envelope or accepted expected-leaf hash changes.
All 54 named interfaces and 20 invoke edges retain their existing
bindings; the vector has 86 finite edge instances and 16 binding cases.
VM/scheduler/checker implementations must independently follow K82.
K82 acceptance does not prove every consumer has implemented it.

## Conformance vectors and verification

Use the accepted vector's exact cases
`callee-local-is-not-captured-by-an-unrelated-caller-name` and
`live-declaration-binds-by-its-wire-actual-rule`, plus its negative
inactive-declaration/typed-closure cases. Verify all 16 cases and 86
edge digests, not just the two former failures. The independent generator
is runnable with:

```sh
python3 -I -B conformance/fixtures/accepted-k82-09/generate.py --check
```

The reference owner's report says 16/16 and 86/86 now pass; that is
reported implementation evidence, not this job's reference execution.
This job runs the independent generator and records its actual result
in [contract-verification](2026-11-02-contract-verification.md).

## Release and after acceptance

**Already a 0.9 K82 implementation fix; no new K or 0.10 deferral.** Keep
b5b with its current reference owner until the owner completes the
relevant gate and closes it with evidence. The fixture README's historical
14/16 result is a dated result, not proof that the live implementation
still fails. No second council contract decision is required here.

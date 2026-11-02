# n5a — give adjoint proofs a checked forward subject

Bead: **b-isa-spec-n5a**. Amendment proposal, never a prerequisite for rungs 3/4; F30 stays deferred until rung 3 or 4 lands.

## Exact defect and clause citations

[ISA 0.9](../../../spec/Bennett-ISA-Spec-0.9.md) §12.1 gives `adjoint_table` witness `{}` and integer subclaim indices. §12.3 stores claims per region; opaque bytewise variants do not identify a forward body. §16.3 example 3 relies on adjoint_table for the reversed four-CX borrow frame, which is not itself the forward translation-lemma syntax. No field or rule binds the reverse subject to an independently checked forward proof. [7s0](../2026-11-02-rev2/b-isa-spec-7s0.md) supplies frame/lifetime domains, not this cross-region proof dependency.

## North Star gap

A proof expressibility/soundness defect, not a new quantum capability. Keep every admitted adjoint and dirty-frame proof route; an unchecked forward/reverse correspondence must never discharge a North Star §5.1 obligation.

## Alternatives and witnesses

Forward frame `cx x b; cx b t; cx x b; cx b t` maps t to t XOR x and returns arbitrary b. Its full adjoint is the reversed four gates, with the same T3 factorisation. An unrelated identity-region proof also returns b, but cannot prove this reverse body. Opaque variant text `adjoint` and a matching signature do not distinguish these choices. Prefer an explicit closure/claim reference plus independent source-adjoint checking. Checking only basis probabilities misses a b-dependent Z phase. [verify.py](verify.py) checks all eight basis mappings, the reversed relation and the phase counterexample.

## Recommended exact spec diff

Replace §12.1's adjoint_table witness `{}` with:

```text
{"forward":{"closure_hash":<linked forward region>,"claim_index":n}}
```

Append:

```text
A claim in a region record has that region as subject except the explicitly
specified inverse_pair relation. Integer subclaims are local zero-based
indices; they do not implicitly name another region. adjoint_table has
no implicit local cross-region subclaims: its forward reference above is
the sole cross-region dependency and subclaims MUST be empty.

The checker resolves the forward closure in the selected linked device
program and independently checks the named claim. It checks that the
current complete body is the §5.3 source adjoint of that forward body,
modulo capture-free local renaming and positional interface/classical
parameter renaming only. Match signature widths/roles, types/bounds,
primitive adjoints, structured order/controls/iterations, unconditional
allocations/releases and borrowed lifetime correspondence. Interface
equality, variant names or agreement on selected basis values is not this
syntactic check. Dependencies must be acyclic and checked before use.

The forward proof must establish the required preservation/cleanliness
relation on the full applicable quantum domain at the corresponding
arguments, budget and invocation scope. A sampled/strict-slice result
cannot discharge it. Map every required current context and lifetime to
its checked forward counterpart; no absent context inherits a proof.
For restricted frame domains, additionally prove the current normative
entry domain is covered by the image of the forward proof's domain under
the checked forward operation. Equality of input domains alone does not
establish inverse applicability. Otherwise use a direct covering frame
proof, not an invented adjoint inheritance.

Source-adjoint correspondence derives source claims only. A schedule-bound
realised claim additionally requires the current realised operation to be
the exact adjoint of the forward realised operation in every mapped
context, with all chosen approximations/cleanups/controls checked. Source
adjointness alone does not prove arbitrary selected schedules adjoint.
Inherited evidence remains evidence; neither a dependency cycle nor an
unchecked structural label proves an obligation.
```

## Hashes, catalog and consumers

Witness bytes/dependency graphs change certificate hashes. Source/interface/closure identities and the catalog need no change; forward closure hashes are references, not substitutions for the current subject. Checkers/producers must implement cross-record resolution and context/domain applicability together. Existing empty witnesses cannot silently be interpreted from a variant string.

## Conformance vectors and verification

Positive four-CX reverse; alpha-renamed locals and positional arguments. Negative unrelated prior proof, changed gate/order/control/width/role, stale closure/index, cyclic dependency, sampled forward result, restricted domain with uncovered inverse image, missing context and nonadjoint realised schedules. Check phases and correlated b, not just target projection. Actual finite checks and their limits appear in [INDEX](INDEX.md); no certificate-checker implementation or sibling gate is claimed.

## K entry: 0.9 versus 0.10

Default **0.10 certificate repair**. Proposed decision `ADJOINT-FORWARD-BINDING`. A 0.9 K route requires **council + maintainer authorization of compatibility rules**, including disposition of old `{}` witnesses; do not grandfather an unsound proof. Coordinate 7s0 without duplicating its domain schema.

## Mixed fat object

Apply [fat-object-scope.md](../2026-11-02-rev2/fat-object-scope.md) and [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F02/F03. Resolve forward proofs inside the selected linked device program with each root's own context `[]` and complete callee coverage. An independent embedded program/native main is not an implicit proof owner; certificates never inventory host-only libc. Container/root association remains required. These amendments never gate rungs 3/4; F30 stays deferred.

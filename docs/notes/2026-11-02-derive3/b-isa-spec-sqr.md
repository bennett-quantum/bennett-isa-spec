# b-isa-spec-sqr — mcx controls named by a bundle

Council proposal, not an ISA implementation. References:
[ISA 0.9](../../../spec/Bennett-ISA-Spec-0.9.md) §§3.3 (indexed operands,
distinctness), 4.4 (gate forms), 11.1 (resolved gate wires), 16.2 (grammar).

## Exact defect and North Star scope

§4.4 writes `mcx (c_1..c_k) t` and “X on the target iff all controls
are |1>”. It does not say whether a NAME in that list must denote one
wire or may denote a wire[N] bundle. §3.3 defines bundles and indexed
operands, but supplies no gate-specific list expansion rule. §16.2's
parenthesized list admits bare NAME entries; it does not admit indexed
entries such as `(c[0], c[1])`. Thus the usual informal advice to list
those indices is not an existing grammar solution.

This is an **operand contract ambiguity**, not evidence that the abstract
machine lacks a many-controlled X gate. Explicit `ccx c[0] c[1] p` already
expresses the two-control case. North Star §§3.1, 4.2, 5.1 requires the
operation and interoperable interpretation, not this particular shorthand.
Either precisely defined expansion or a precisely stated scalar-only
syntax can preserve the machine capability. Expansion is recommended
because it preserves the admitted bundle spelling and needs no new list
grammar. Reading only the first bit has no defensible all-controls meaning.

## Alternatives and concrete witness

Read-only witness:
`../bennett-conformance/checker-replay/witnesses/mcx-bundle-control/`.
Its action has p,q:wire mut and c:wire[2] in, then:

```text
swap p q
mcx (c) p
```

With p=q=0, c=1 (bits c[0]=1,c[1]=0), expansion leaves p=0;
first-bit execution produces p=1. With c=3 expansion produces p=1.
The complete forward map is (p,q,c) -> (q XOR [c=3],p,c).
The adjoint applies the controlled X before the swap, restoring every
input. It is generally not the same source sequence run forward twice.

| Alternative | Meaning and cost |
|---|---|
| Flatten each bundle entry in increasing bit index | All named controls participate; k is the total wire count. Recommended. |
| Require every entry to be scalar | Legal alternative if explicitly published; bundles require another spelling/construction, and indexed list syntax would need a grammar amendment. |
| Broadcast independent X operations per control | XOR of controls, not conjunction; wrong at c=1 and c=3. |
| Inspect c[0] alone | Silently loses controls; wrong at c=1. |

## Recommended exact spec diff

Insert immediately after the forms sentence in §4.4:

```text
Each NAME entry of an mcx control list may denote wire or wire[N].
It contributes every wire of that bundle, in increasing bit index;
concatenate entries in written list order. The k in mcx(k) is the
number of resulting control wires, not the number of list entries.
The target denotes exactly one wire (a scalar name or indexed operand).
Apply §3.3's pairwise distinctness to the resulting control wires and
target before execution; overlapping or repeated entries reject as
ill-formed, never deduplicate them to repair an operand list. Existing
effective predicate-context rules still apply. The empty list has
zero explicit controls and applies X under the active predicate.
Flattening determines resolved primitive operands, gate cost and
realisation arity, but does not rewrite source identity or source points.
```

Append the normative witness:

```text
For c:wire[2], mcx (c) p has the same gate semantics as
ccx c[0] c[1] p: it changes p exactly for c=3, not c=1 or c=2.
mcx (c, c) p and a target c[0] with controls (c) are ill-formed.
```

No general §16.2 list-grammar change is required. This proposal does not
add indexed entries inside parenthesized lists; use the existing indexed
operands of ccx for the scalar two-bit comparison above. It also does
not legalise a multiwire target or change cx/ccx into bundle operations.

## Hashes, catalog and consumers

Retain the written bundle operand in source envelopes. Under an explicitly
authorized same-version repair, existing object/interface/closure hashes
are unchanged; the published witness binds object
`db3d3f92a79077074901449d83532f1a0d82ef1621bd833adb7ad14d1aedd467`
and action closure
`e38d5da64f9a0ae89a66243bcbe5a321cb807a5bbabf83952b63d3465dfab346`.
They identify bytes, not an implementation's formerly mistaken unitary.
Schedules that counted one control must be regenerated and certificates
rechecked; unchanged source hashes do not validate old execution caches.

The ref catalog's existing mcx wire-list recipes already operate on
resolved wires. Their bytes need no change: source adapters must supply
all flattened controls and k. If an adapter changes generated leaves,
update its expected leaf hashes and affected schedules, not `ref@0.9`
silently. K78 primitive identity uses the complete effective control set;
no new causal-matching exception is required. A 0.10 publication changes
version-prefixed envelopes and pins as described in [INDEX](INDEX.md).

## Conformance vectors and actual verification

The recorded older published-checker result rejects the witness table
because it flips p at c=1; the stored replay result proves all 16 inputs.
Re-ran the current replay CLI with `--max-elements 16`: exit 0,
claim proof, 16 elements, covered_fraction 1. Its separate unchecked
metadata obligations remain unchecked; this is only the relation claim.

Built the current reference interpreter read-only into the job target.
Its `check` on the **original** bundle object exits 1 with
`§3.3: c must name one wire`. It does not reproduce the older checker's
first-bit result. Current validation and action execution require scalar
list entries. That is evidence for the ambiguity, not proof that the
recommended expansion is implemented.

For an executable independent comparison, changed only a scratch copy
to `ccx c[0] c[1] p` and recomputed its declared interface hash. Reference
check/hash pass; object hash is
`c785ff3f310f968e2a26f68834e99ab892cf0d1bffe816cf26b17d45c33a65af`,
interface/closure hash
`f53238499e07824b6c62385c33426d4bc56bfaf900df61264ec65e0589061f39`.
All 16 forward and 16 adjoint basis cases pass the map above, with c
preserved. An initial attempt at `(c[0], c[1])` was rejected by the
current list grammar; it is not used as a passing witness. Reproduction
is in [INDEX](INDEX.md). No source interpreter or sibling fixture changed.

Acceptance adds bundle widths 1/2/3, mixed scalar/bundle entries, list
order, c=0/1/2/3, empty-list X, target/duplicate/control alias negatives,
active predicate and adjoint cases. Check exact +1 phase and a superposed
control entangled with a reference, full flattened arity/cost, source
hash stability and K78 matching against the equivalent scalar primitive.
Suite F must distinguish an operand rejection from a replay mismatch;
suite C/T6 checks the resulting unitary.

## K entry and publication route

Proposed row: `K(new) | §§3.3,4.4,11.1 | MCX-BUNDLE-CONTROLS: ordered
bundle expansion; distinctness and k over denoted wires; scalar target |
sqr | operand/expansion semantics`. Reserve no number. Recommend a
council-authorized 0.9 K repair only if the ambiguity-repair track is
explicitly approved; otherwise publish in **0.10** under the semantic
version rule. Do not silently extend accepted 0.9 syntax or claim all
current consumers already agree. The scalar-only alternative is a valid
decision if the council instead supplies its exact operand wording and
preserves the general many-control capability. Keep sqr open meanwhile.

## Mixed fat object

[fat-object-scope.md](../2026-11-02-rev2/fat-object-scope.md) and North Star
§§2, 3.3, 4.4 apply. Flatten within each marked device module's actual
wire bindings; wires and certificate domains do not include native host
code. Gate arity and budgets belong to that device invocation. Shared
device helpers may be linked under several roots without making native
main or host-only libc certificate regions. No launch, opaque-handle or
tape format changes follow from this operand rule. The shared note's
container/root and mixed-launch boundary additions remain required.

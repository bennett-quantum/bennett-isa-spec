# b-isa-spec-pe1 — K13's extra borrowed bit

Council proposal; published text and fixtures are unchanged. References:
[ISA 0.9](../../../spec/Bennett-ISA-Spec-0.9.md) §§4.2–4.3, 5.6,
12.1 “CLAIM-REPLAY witness”, 12.2 “Component resolution (K69/K70)”.

## Exact defect and North Star scope

The §12.1 subject is exactly the two-row oracle `r = lift.add x a;
lift.retval r`, with no inherited controls and an unused interface
region. The printed tuples nevertheless contain `borrowed:["1"]`,
the widths include `borrowed:[1]`, and the prose counts a 25-bit D and
coverage `1/33554432`. The lifted opcode set in §4.2 supplies no qborrow
instruction. This subject has no checked dirty-bundle lifetime.
§12.2 K70 expressly rejects a borrowed position beyond the checked
borrow requirement. The example cannot pass the rule it illustrates.

This is an inconsistent normative example, **not a missing borrow
capability** or a reason to relax K70. §4.3/§5.6 already supplies dirty
allocation and borrowed-system preservation where applicable. North
Star §§3.1, 4.2, 5.1 remains reachable, including dirty ancillae and
entanglement with an external reference. Correct the example rather
than adding a fictitious oracle input component.

## Alternatives and concrete witnesses

For x=1, a=2, target=2, fault=3, cells=[2,1], B=2, the two rows return
payload 3 and fault code 0. The resulting complete tuple is
`bundles:["1","1","3"], borrowed:[], regions:[["2","1"]]`.
The full basis domain has 2+2+4+16 = 24 bits. A singleton declared
domain, sampled four times with replacement, covers one of 2^24 full
tuples and remains evidence. Singleton ranges consume no random words.

Changing a to 1 changes target to 0. Reducing B to 1 changes the payload
to 0 and the final fault word to 3 XOR 1 = 2. An identity expected row
fails. Dropping the borrowed coordinate changes none of these results.
An invented one-bit coordinate must still reject under K70, even if
the table preserves it perfectly or calls it the same name as a real
interface bundle.

| Alternative | Assessment |
|---|---|
| Remove the extra borrowed coordinate and adjust D/fraction | Repairs the example without changing any machine/domain rule. Recommended. |
| Replace the subject by a checked borrowing action wrapper | Possible new example, but needs its actual body, frame, lifetimes and new domains; the printed two-row oracle cannot stand in for it. |
| Invent oracle borrowed inputs or infer them from unused labels | Changes the oracle ABI and breaks K70 positional resolution; unjustified. |
| Count a lift.perm_call callee's internal scratch as this oracle's borrowed interface | Needs a separate lifetime/tuple contract; no such call exists in this witness. |

## Recommended exact spec diff

Within the single §12.1 CLAIM-REPLAY witness JSON block, replace **all
six** tuple occurrences of `"borrowed":["1"]` by `"borrowed":[]`,
and make these additional replacements:

```diff
-  "widths":{"bundles":[2,2,4],"borrowed":[1],"regions":[[8,8]]},
+  "widths":{"bundles":[2,2,4],"borrowed":[],"regions":[[8,8]]},
-  "sample_count":4,"distinct_samples":1,"covered_fraction":"1/33554432","verdict":"evidence"
+  "sample_count":4,"distinct_samples":1,"covered_fraction":"1/16777216","verdict":"evidence"
```

Replace the indicated prose immediately below that block:

```diff
-with a preserved one-bit dirty bundle and an unused two-cell,
-eight-bit `in` interface region; no inherited controls.
+with an unused two-cell, eight-bit `in` interface region; no borrowed
+lifetimes and no inherited controls.
-full 25-bit D (2+2+4+1+16 bits).
+full 24-bit D (2+2+4+16 bits).
-words and cover one of 33554432 full tuples.
+words and cover one of 16777216 full tuples.
```

Append to that witness paragraph:

```text
The empty borrowed array is required here: the two-row oracle has no
checked borrow lifetime. Adding a borrowed coordinate to this witness
rejects under K70; unchanged borrowed contents alone do not establish
that a lifetime exists.
```

The replacements operate on the exact phrases across line wrapping;
do not change the separate general tuple encoding, sampler, layout,
arguments, target/fault widths or K70 rule.

## Hashes, catalog and consumers

The witness deliberately supplies no bound object/closure hashes, so
there are no printed subject digests to recompute. No source instruction,
interface or ref catalog changes. Any concrete certificate/table fixture
based on the invalid example must change its tuple/domain bytes and
coverage report; a certificate digest consequently changes. Consumers
following K70 already reject the old example. Do not change their
component resolver or call the repair a proof over dirty inputs.

## Conformance vectors and actual verification

The read-only replay suite's `tests.test_replay.K13Witness` has four
focused tests using the repaired empty-borrowed relation, including the
changed argument, short budget, wrong row and missing-cell cases.
Ran it together with `QinplaceCoverage` using the bounded command in
[INDEX](INDEX.md): all 12 tests passed, exit 0. The replay fixture uses
ptr_width=32 rather than the printed legal 2-bit layout, so this run
checks the relation/domain behavior, not literal byte reproduction of
that layout. It is also an exhaustive singleton-slice test, not the
printed four-draw sampled packet. The finite INDEX checks independently
check the bit count, all six tuple repairs and sampling calculation.

For acceptance publish a complete bound certificate for the repaired
2-bit-layout oracle, with the exact four-draw sampler and all region
cells. Expect evidence `1/16777216`; reject the extra borrowed position,
unchanged-argument/budget rebinding, identity row and omitted cell.
Retain a separate genuine qborrow action with an entangled external
reference to test dirty preservation; it must not be folded into this
oracle's D. A small example fix does not discharge suite C's full-D T3.

## K entry and publication route

Recommend a **0.9 K13 witness erratum**, not a new oracle ABI or 0.10
deferral. Proposed decision row: `K13 erratum | §12.1 | remove nonexistent
borrowed coordinate; D=2^24, coverage=1/16777216 | pe1 | example repair`.
Council assigns any new ledger number; this proposal reserves none.
The semantic rule already belongs to accepted K70. Keep pe1 open until
the editor and conformance owners publish the repair.

## Mixed fat object

Apply [fat-object-scope.md](../2026-11-02-rev2/fat-object-scope.md) and
North Star §§2, 3.3, 4.4. This tuple belongs to one embedded device
subject and its selected root. A native host or another marked module
does not contribute a borrowed coordinate, interface cell or certificate
record to this subject. No container, launch, budget, handle or tape
change is needed; the shared note's explicit mixed-launch association
work remains. The native binary need not become an oracle to repair K13.

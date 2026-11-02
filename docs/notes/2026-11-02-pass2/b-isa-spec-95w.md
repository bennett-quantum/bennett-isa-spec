# 95w — native models: incremental binding requirements

Second-pass proposal for `b-isa-spec-95w`. The defect, X/−X witness,
alternatives and model proposal are already derived in
[the first-pass native-model note](../2026-11-02-contract-checker-native-model.md).
That proposal is incorporated by reference; this note adds only the
context, exact-entry and publication boundaries it leaves to settle.

## Defect and North Star scope

ISA 0.9 §8.3 supplies native opcode/cost/epsilon metadata but no realised
operator. §12.1 requires independent construction of U/channel at every
resolved context, with controls first in its basis. An advertised bound
does not determine U. This is the same C3/C5 gap, not a second issue.
North Star §5.1 part 2's independent process comparison remains required
for approximate native gates, including coherent control; an unchecked
legacy obligation does not deliver that capability.

## Alternatives and worked witness

Use the first pass's alternatives and concrete X/−X controlled-interference
witness without another model-schema derivation here.

Additional exact-entry witness: the §8.1 expression `{"op":"-",
"args":[1,1]}` is identically zero. Its syntactic expression form does
not make an otherwise exact X entry approximate. Conversely an expression
that evaluates to zero for one parameter/control context and nonzero for
another needs the model in the latter context. Do not turn “expression-
valued epsilon” into a new, broader exact-entry rejection rule.

## Recommended exact spec diff

Apply the first-pass schema/model text, replacing its opening requirement
and adding these precise qualifications in §8.3:

```text
A native entry that can evaluate to nonzero epsilon in an admitted
resolved context must bind the versioned realisation_model described
here. For a context whose evaluated epsilon is zero, the operation is
exactly §4.4's opcode, including global phase and controlled form;
an omitted model denotes that operation. Expression syntax alone does
not remove this exact-entry convention. Any supplied model must agree
with every zero-epsilon context and satisfy the advertised bound in
every other context in which it is used.

Resolve model parameters, operand bit order and total effective control
context from the same matched target entry and invocation used for
cost/error accounting. The model's operand-to-wire map must explicitly
relate its declared basis to §12.1's controls-first allocation basis;
permutation of matrix indices is checked, never inferred from a vendor
name. A controlled model includes phase relative to the disabled branch.
For a channel use the specified controlled instrument/dilation, not
an arbitrary coherently controlled Kraus representation.

An externally stored model binds canonical bytes by digest and a
published model-format version; those bindings enter the target's JCS
bytes. A location/URL alone is not model identity. A model change thus
invalidates the target-bound schedule and certificate. A legacy target
without the required model remains executable under its declared target
assumptions, but the affected independent approximation obligation is
unchecked and cannot be relabelled checked by reference self-comparison.
```

The first-pass model grammar, exact/interval certificate requirements and
dimension/normalisation checks still apply. Unsupported model mechanics
must name the missing format, not remove approximate algorithms from
scope. Do not publish this prose without a versioned model schema and
positive/negative bytes accepted by independent consumers.

## Hashes, catalog and consumers

The first pass's hash analysis stands: new model bindings change
`target_hash` and dependent schedules/certificates, not source closures
or the ref catalog. Add cache tests for changed **model digest** at equal
opcode, epsilon and cost. Producers, target authors and checkers must
agree on controlled and parameter-dependent models, including models
reached through target schemes and default-only calls.

## Conformance vectors and verification

Reuse the first-pass X/−X vector. Add syntactic-zero epsilon, an expression
with zero and positive contexts, matrix basis reordering, a same-location
changed digest, disabled-control phase, and a non-trace-preserving Kraus
list. The exact Bell-state overlap and rational expression are checked
in [INDEX](INDEX.md). No new native-model consumer is run or claimed.
Origin read: `../bennett-conformance/checker/.tmp/CROSS_REPO_BEADS.md`, item 1.

## Release and route

Recommend the **same 0.9 K plus versioned target-model repair** as the
first pass. If schema publication waits for 0.10, retain 95w as the
explicit current blocker for independent approximate-native checking.
Do not create a duplicate bead or re-open exact §4.4 gate semantics.

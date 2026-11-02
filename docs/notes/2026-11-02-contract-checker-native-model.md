# Checker question 1 — realised native operators

Proposal for council review; source request 1 in
`../bennett-conformance/checker/.tmp/CROSS_REPO_BEADS.md`.

## Defect and North Star scope

ISA §8.3's native `gates` entries contain opcode, classes, controls,
parameters, cost and epsilon; §8.1 supplies arithmetic expressions, not
an operator description. §12.1 requires the checker to build realised U
or a channel independently for numerical/channel claims, and §4.4
defines only the ideal opcode plus an error obligation. Cost and an
advertised distance bound do not identify the realised operation.
This is a genuine S0 contract gap for independent native approximation
verification (rubric C3/C5), not grounds to accept reference self-comparison.

North Star §§4.4/5.1 part 2 promises independent phase-sensitive and
channel verification. Returning unchecked for an unsupported model is
honest checker evidence handling, but is not the missing model mechanism.
Exact logical native gates retain their declared §4.4 matrices; algorithms
with nonzero approximation remain in scope.

## Alternatives and worked witness

Take one native X entry with fixed advertised epsilon=2 and time=1.
Realisation A is X and realisation B is −X. Both meet that advertised
operator-norm bound, cost, opcode and class metadata; they produce the
same uncontrolled channel. Their controlled extensions differ: on
`|+>_q|0>_t` they produce `(00+11)/sqrt(2)` and `(00−11)/sqrt(2)`,
orthogonal states. An entry with no realisation bytes cannot tell the
checker which process to compare, even though both targets have the same
current JSON/hash. Certifying actual bound 0 under coherent control is
valid only for A. A conservative declared bound 2 can be composed as an
assumption, but that is not independent reconstruction of a tighter claim.

| Alternative | Assessment |
|---|---|
| Treat every native entry as its ideal opcode | Circular self-comparison for approximate entries. |
| Trust advertised epsilon | Explicit target assumption only; cannot discharge §12.1 independent reconstruction. |
| Hash-bound matrix/channel model (recommended) | Supplies the actual operation, controls and conventions needed for checking. |
| Model approximations as explicit existing schemes | Already works when every leaf is exact and the scheme lists the actual circuit; cannot describe an opaque approximate native leaf alone. |

## Recommended exact spec diff

Add to §8.3 gate entries `realisation_model?: {version,kind,arity,basis,
parameter_bindings,control_models,data}` and append:

```text
A native entry with nonzero or expression-valued epsilon MUST supply a
versioned realisation_model. Its canonical bytes are part of target_hash.
kind is unitary or channel. basis lists ordered operand bits; matrices use
the §12.1 computational basis with inherited controls first. Parameters
bind by their exact §4.4 angle values and declared operand order. data
contains the actual complex matrix, or an ordered Kraus list, as exact
expressions over rationals, i, pi, sqrt, exp, sin and cos, or certified
interval enclosures of an identified exact model. Real/imaginary rational
constants are numerator/positive-denominator decimal-string pairs.
Intervals are closed rational endpoints; enclosures need an independently
checkable certificate for the enclosed exact operation and its unitary
or completely-positive trace-preserving property. An unverified box of
numbers alone is not an operation model.

For each admitted total control count, control_models binds the actual
controlled implementation, including environment/measurement conventions.
A unitary model may use the exact controlled-U construction including its
global phase. A channel is not given a coherent controlled extension by
controlling arbitrary Kraus operators; it requires an explicit instrument
or dilation/control model. The checker resolves every parameter and
control context and constructs the realised operation from that model,
checks dimensions and normalization, and verifies the claimed distance
with §12.1's interval algorithm. The epsilon field remains a separate
accuracy/matching obligation and is not the operator definition.
Exact entries may omit the model and then denote §4.4's exact opcode,
including its exact controlled form. A supplied model must agree with
that contract. No absent model licenses ideal-vs-ideal self-comparison.
```

For legacy targets lacking a model, preserve parse/execution compatibility
but record affected numerical/channel obligations as unchecked (missing
realisation model). Do not issue a checked executable certificate from
those missing obligations. The new model version is an explicit target
format extension; unknown model versions receive unsupported-model
reporting, never a guessed matrix.

## Hashes, catalog and consumers

New native-model bytes change target_hash and every target-bound schedule
and certificate. No canonical source or reference catalog changes. Target
authors, scheduler and checker must resolve the same model and controlled
variant; a simulator using ideal opcodes alone is not a physical native
model checker. Physical stochastic failure probabilities remain separate
from logical approximation epsilon (see fd0).

## Conformance vectors and verification

Use X/−X with an outer control and compare matrix units/Choi operators;
test uncontrolled equal channels but unequal controlled processes. Exact
X without a model remains valid. Missing approximate model, wrong arity,
non-normalized Kraus list, parameter/control mismatch, a model-hash change
and a rigorously inconclusive interval claim must not pass the obligation.
The Bell-state sign witness is checked in
[contract-verification](2026-11-02-contract-verification.md). No native
target checker was executed for this note.

## Release and after acceptance

Recommend a **0.9 K contract repair with a versioned model schema**, or a
0.10 target extension with this current gap explicitly blocking independent
approximation claims. The checker question is already the cross-repo bead
filing request; the orchestrator should file/retain it in this repo with
this exact missing mechanism. Council must accept the model and migration
rules before a consumer advertises checked approximate native support.

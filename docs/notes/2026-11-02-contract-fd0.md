# fd0 — imported QEC models, parameters and hierarchy

Proposal for council review; `b-isa-spec-fd0`, originating at
`b-targets-gyp`. No published target or spec bytes are edited.

## Defect and North Star scope

ISA 0.9 §8.3 imports maps a model quantity to `[resource,factor]`.
§8.1 has a closed expression language and binds no code distance,
hardware parameter or imported quantity. §8.2 aggregates scheduled leaf
events, not a factory hierarchy or a symbolic algorithm call graph.
There is no binding rule from an imported QEC formula to a logical gate's
time or qubit class. A numeric mapping is implemented by the supplied
bead's account; it does not supply these missing semantics.

North Star §7, **Qualtran, Azure RE**, promises symbolic hierarchical
counting and imported models as target data. §4.3 requires all declared
resources with their own units. This is an S0 contract scope gap for that
import capability, rather than permission to reject such algorithms.
Scalar materialisation is an available 0.9 path for a completely bound
instance; it is not the full symbolic contract. Keep fd0 as the explicit
blocker for that missing contract, even if the new encoding is 0.10.

Primary evidence: Microsoft's [QEC architecture model example](https://quantum.microsoft.com/en-us/insights/blogs/resource-estimation/modeling-quantum-architecture-with-azure-quantum-resource-estimator)
uses code distance and hardware timing in its cycle formula. Qualtran's
own [call graph protocol](https://qualtran.readthedocs.io/en/latest/resource_counting/call_graph.html)
admits symbolic multiplicities. Its [release notes](https://github.com/quantumlib/Qualtran/releases)
say AlgorithmSummary was removed in favor of GateCounts; adapters must pin
the input format/version, not assume the old class is current. These
sources were read as model-format evidence, not imported code.

## Alternatives and worked example

Use a declared odd code distance d=5, hardware measurement time τ=2 ns,
and **illustrative** formulas chosen for this witness:
`cycle=3τd`, `physical=2d²`, `logical_risk=(1/10)*(1/100)^((d+1)/2)`.
The results are 30 ns, 50 physical qubits per logical qubit and 1/10^7.
The first expression can be numerically precomputed; it cannot reference
d in current §8.1. The risk expression needs exponentiation. It is a
physical-model risk, not a bound on phase-sensitive operator norm.

For three sequential cycles with one logical qubit, the model gives
90 ns, peak 50 physical qubits and occupancy 4500 physical-qubit·ns.
For three simultaneous independent logical qubits held for one cycle,
time is 30 ns and peak is 150, with the same occupancy. Adding all scalar
quantities would get the parallel time or sequential peak wrong.

| Alternative | Assessment |
|---|---|
| Bind d and hardware externally; emit a scalar 0.9 target | Sound concrete compatibility path; retain the original model and bindings in provenance. No symbolic selection or hierarchy is represented in the target. |
| Add parameters and formulas only | Solves evaluation, but not where a formula is consumed, hierarchical multiplicities, concurrency or double counting. Insufficient alone. |
| Bound target model plus separate hierarchical estimate (recommended) | Fix hardware/model choices in target identity; bind symbolic algorithm inputs and model accounting to the certificate; retain §8.2 leaf accounting separately. |
| Treat failure probability as gate epsilon or sum every quantity | Unsound error semantics or wrong units/aggregation. |

## Recommended exact spec diff

For the extended target/cert version, add §8.5 and a corresponding K entry:

```text
8.5 Imported model bindings
A target MAY contain model_parameters:{name:{value:number|null,unit:string}}
and each imports entry MAY additionally contain
{source_hash, source_format, parameters:{name:expr},
 quantities:{name:{unit,expr}}, bindings:{destination:quantity_name}}.
source_hash is SHA-256 of the source model's canonical bytes; source_format
includes a format version. References {param:name} resolve model_parameters;
{quantity:name} resolves a quantity of this model. Dependencies form a DAG;
cycles, duplicate definitions and undeclared references are invalid.
Formula expressions add pow(base,exponent) for integer exponents and
sqrt(x) for the nonnegative principal root, in model context only.
Rational operations and integer powers are exact; roots are exact
algebraic quantities, converted to a reported decimal only with a stated
outward bound. Division by zero, negative roots and nonfinite concrete
outputs are errors. No eps or accuracy-bucket variable is bound here.
Integer parameters declare their integer constraint; a code-distance
parameter additionally declares its permitted parity/range.
A null parameter is symbolic: an estimate retains it as a free parameter.
It MUST be bound before concrete target scheduling or numeric evaluation,
and every concrete binding is included in the resolved target's bytes.
Only an explicit bindings entry may substitute a model quantity for a
gate cost, class quantity or other target numeric field. Its destination
must name a schema-defined numeric field and have matching units and a
valid value. Do not infer bindings from vendor quantity names.
Unmatched quantities are preserved under new named resources using map
and its unit factor. An imported physical failure-risk quantity is a
separately named model output or additive count charge, never epsilon,
an operator-norm assertion, or an automatic probability composition rule.
```

Add to §12.3 region records:

```text
model_estimates?:[{model_hash,parameters,root, nodes,outputs,verification}]
```

And add this normative paragraph:

```text
A model estimate is a finite DAG of leaf model invocations and composition
nodes {kind:sequence|parallel|repeat,children,count?}. A repeat count is a
nonnegative integer expression. Children have explicit resource units,
duration and live-interval summaries. Sequence shifts each child's time
origin to its predecessor's end; parallel aligns child origins; repeat
concatenates count copies. Count resources sum, time is makespan, and
physical occupancy and peak are computed from the resulting simultaneous
live intervals, summing simultaneous demands. Shared factories require
explicit shared intervals, not duplicated independent children. Symbolic
results retain their DAG until parameters are bound. These reports are
distinct from required §8.2 resources and MUST NOT be added to the same
leaf counts again. The checker verifies model identity, parameter and unit
bindings, multiplicities and composition, recording checked obligations
separately from external-model assumptions. Unsupported symbolic source
expressions remain preserved external model obligations; they do not
invalidate execution of the object or justify an invented numerical
estimate. A concrete requested estimate with an unresolved obligation is
reported unchecked, with the exact expression and missing model support.
```

The external obligation path preserves the full imported artifact and
symbolic report; it does not claim a checked estimate. Concrete adapter
support remains required work under fd0. Code-distance search belongs to
the estimator/scheduler; its selected value is a resolved target binding,
and its search domain/objective/physical assumptions belong in the model
estimate. The target does not silently choose an optimum.

## Hashes, catalog and consumers

Resolved parameters, formulas, input-model identity and bindings change
target_hash. Algorithm-dependent hierarchy changes certificate bytes,
not source closure hashes. No reference catalog leaf needs to change.
Targets, estimator, scheduler and checker must agree on units, exact
evaluation, outward conversion and DAG accounting. Existing count
resources and exact logical-gate epsilon keep their present meanings.
Do not reinterpret a legacy count named failure_probability as a checked
physical-error model.

## Conformance vectors and verification

Check the d=5 example above using rational arithmetic; d=3 gives cycle
18 ns, physical 18 and risk 1/100000. A seconds conversion multiplies
nanoseconds by 1/10^9. Exercise null/bound parameters, odd-distance
validation, forward quantity references, cycles, zero divisors, roots,
incompatible units, unknown binding destinations and a source-hash
mismatch. Test sequence versus parallel peaks, factory sharing and
symbolic repeats. A nonzero risk with exact logical gate epsilon=0 must
leave the logical error bound at zero. The executable arithmetic checks
are in [contract-verification](2026-11-02-contract-verification.md);
no target consumer was executed here.

## Release and after acceptance

Recommend **0.10 target/certificate extension**, with a 0.9 K entry
explicitly recording the blocker and the scalar-instance compatibility
construction. The fd0 proposal in the bead is incomplete without binding
destinations, hierarchy and identity. Do not call imported symbolic models
implemented until those vectors and consumer obligations pass. Council
acceptance precedes schema edits, adapter work and any re-pinning.

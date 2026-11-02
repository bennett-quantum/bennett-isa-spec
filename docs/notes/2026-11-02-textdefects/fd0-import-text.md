# fd0 — exact import and hierarchical-estimate text for council review

For `b-isa-spec-fd0`, originating at `b-targets-gyp`. Proposal only:
**target/cert 0.10**, with full-version publication under the repository's
change rules. This does not amend 0.9, its catalog, or any consumer.

## Relationship to sol-contract and the defect

Read [the earlier fd0 derivation](../2026-11-02-contract-fd0.md) first.
This note supplies its missing field-level contract, worked serialized
import and acceptance vectors; it does not repeat its alternatives review.
The earlier note correctly identifies that 0.9 §8.3's
`imports.{model}.{version,map}` cannot bind code distance or formulas to
logical costs, and §8.2's leaf-event aggregation cannot express a factory
hierarchy. `bd show b-isa-spec-fd0` confirms the missing exponentiation,
code-distance selection and symbolic-count questions. Both published
versions' §8.1 languages lack the required parameter/quantity bindings.

Differences from the earlier sketch are deliberate:

- The extension is nested under `imports.<name>.model`; the old numeric
  import shape is retained. No new top-level `model_parameters` namespace
  or implicit vendor-name binding is needed.
- Source identity hashes **raw supplied bytes**, with the bytes embedded;
  a second hash identifies the canonical normalized model. Canonicalizing
  a vendor file first could lose significant numeric spelling or metadata.
- Units have dimensions and scales, not just labels. Parameters distinguish
  hardware, code distance and algorithm inputs. A code distance is positive
  integral; its declared domain chooses parity. An unconditional odd-only
  rule would wrongly exclude codes with even distance.
- A new `model` resource kind holds named non-additive outputs. A numeric
  physical/logical ratio is not summed as a qubit count, nor is per-cycle
  risk a gate epsilon. Legacy count charges keep their old meaning.
- Every numeric substitution has a JSON Pointer, unit conversion, rounding
  rule and checked resolved value. The hierarchy has an exact node grammar,
  half-open intervals and single-accounted shared-factory reservations.
- Symbolic models are valid inputs. Unresolved **operational** bindings
  prevent producing a concrete schedule, not loading/executing an otherwise
  valid object. Unsupported source expressions stay losslessly present as
  explicit obligations. That preservation is not a claim of full adapter
  implementation: fd0 remains a blocker until promised adapters work.

North Star §§4.3–4.4, §5.1 and §7's Qualtran/Azure RE row require all named
resources, independent checking, symbolic hierarchy and models as data.
All §8 demo rungs, especially Grover over unmodified SQLite with its
N=2^20 estimate, remain unchanged. A scalar-only subset, source rewrite,
mandatory gate materialization, or rejecting algorithms because a model
is symbolic would narrow that scope. The proposal below retains symbolic
models and a concrete instantiation path without making either executor
understand a vendor's implementation language.

## Publication edits

In the **new version**, extend §8.2's resource kind enumeration with
`"model"`; append `dimension` and `scale` metadata as specified below;
extend §8.3's `imports` entry with optional `model`; add §8.5 below; add
`model_estimates?` to §12.3 region records and the §12.3 paragraph below.
Add a §0 semantic entry referencing fd0/gyp and suite E/H vectors.
The text from “8.5” through “End normative insertion” is the proposed
normative insertion, including its tables. No unspecified field or
inferred default is part of it.

### 8.5 Imported models, parameters and hierarchical estimates (proposed normative text)

An import MAY retain the legacy form `{version:string,map:object}`. Such
an entry has its existing scalar import meaning; no QEC selection,
hierarchy, physical-error guarantee or new parameter is inferred from it.
An entry using this section has exactly `{version,map,model}`. `version`
is the nonempty imported-model version, not the ISA sub-version.
`model.schema` MUST equal `"bennett.import-model/1"`. All fields shown
below are required unless suffixed `?`; absent optional arrays/maps are
empty. Duplicate JSON keys, unknown fields, and unknown schema versions
are invalid model encodings, rather than instructions to ignore data.

**Scalar and unit types.** In the following schemas `S` is a nonempty
Unicode string compared bytewise, `H` is exactly 64 lowercase hexadecimal
digits, `Hex` is an even-length lowercase hexadecimal string (possibly
empty), and `Q` is a reduced rational encoded as a string: `0` or a
canonical signed decimal numerator, optionally followed by `/` and a
positive canonical denominator greater than one. No leading `+`, leading
zero, `-0`, denominator one, or reducible fraction is permitted. `Z` is
the integral subset of Q, `N` the nonnegative integers, and `P` the strictly
positive rationals. Mathematical integer ranges are unbounded; an implementation
resource limit is an explicit unfinished-operation diagnostic, not a
model-domain restriction or an executed program fault.

A `Unit` is `{dimension:{S:Q},scale:P}`. Its dimension map omits zero
exponents. Axis `s` denotes seconds; `physical_qubit` and `logical_qubit`
are distinct axes. Other axes are namespaced by the source format and
version, unless the importing target explicitly declares an equivalence.
Equal labels alone imply no equivalence. A numeric value x in this unit
has base value x*scale and the given dimension vector. Fractional
exponents are admitted. Dimensionless units have `{}`. Unit metadata is
not an approximation or an instruction to sum quantities.

A target resource participating in a typed map MUST declare
`dimension:{S:Q}` and `scale:P` in addition to its §8.2 `unit:string`.
Its `unit` remains the display label. These two fields are optional and
have no effect for resources not participating in this section. The
mandatory `time` dimension is `{s:1}`, scale 1. Ordinary logical peak is
`{logical_qubit:1}` and logical occupancy is
`{logical_qubit:1,s:1}`; physical model outputs retain physical axes.
A resource of kind `model` MUST supply this metadata; it has no automatic
§8.2 scheduled-event aggregation and is reported in `model_estimates`.
The required scheduled `time`, `peak_qubits` and `qubit_second` accounting
remains in force. An objective that references a model resource MUST supply an explicit
selector in the new optional `objective.model_outputs` map:

```text
objective = {weighted:{S:number},model_outputs?:{S:Selector}}
          | {lexicographic:[S,...],model_outputs?:{S:Selector}}
Selector = {model:S,node:S,metric:"duration"|"counts"|"occupancy"|"peak"|"quantity",
            pool?:S,resource?:S}
```

Each model-resource objective term has exactly one selector keyed by that
resource name; other selector keys are invalid. Model/node name the
hierarchy to evaluate under its resolved target and current algorithm
bindings. Duration has seconds units and neither optional field;
counts/quantity require resource and forbid pool; occupancy/peak require
pool and forbid resource. Quantity selects the named destination resource
in that model's map, counts selects its aggregate count, and pool metrics
select the model-qualified pool. The selected dimensions must agree with
the objective resource's and the value is converted to its scale. Existing
weighted/lexicographic rules then apply to that scalar. An unbound
objective is retained symbolically and has no silently assigned scalar
value or completed concrete optimization verdict.

**Normalized model schema.** The import key supplies the model name.
Names of units, parameters, quantities and nodes are local to that model.
`model_hash` is SHA-256 of UTF-8 JCS bytes of the complete `model` object,
without a final LF; `source.sha256` below hashes the decoded raw bytes.

```text
model = {
  schema: "bennett.import-model/1",
  source: {format:S,version:S,media_type:S,bytes:Hex,sha256:H},
  units: {S:Unit},
  parameters: {S:Parameter},
  quantities: {S:Quantity},
  bindings: [Binding,...],
  nodes: {S:Node},
  assumptions: [S,...]
}
Parameter = {
  role: "hardware" | "code_distance" | "algorithm" | "other",
  type: "integer" | "real", unit:S, value: ClosedExpr | null,
  domain: {min:Q|null,max:Q|null,parity:"any"|"odd"|"even"}
}
Quantity = {unit:S,role:"duration"|"demand"|"risk"|"count"|"scalar",expr:Expr}
Binding = {path:S,quantity:S,factor:P,rounding:"exact"|"upper"|"lower",places:N,value:Q|null}
Expr = Q | {lit:Q,unit:S} | {param:S} | {quantity:S}
     | {op:Op,args:[Expr,...]}
     | {external:{location:S,unit:S}}
Op = "+" | "-" | "*" | "/" | "min" | "max" | "ceil" | "pow" | "sqrt"
ClosedExpr = Expr with no param, quantity or external reference
```

Every unit reference MUST resolve in `units`. `source.format` and
`source.version` identify the exact input grammar; `media_type` describes
its bytes. The normalized formulas and assumptions MUST be retained with
the original source. The import's version and source version need not be
equal, but both MUST be preserved. Hash mismatch is invalid, even if a
file with the same vendor name is otherwise available. No file path,
network fetch, mutable URL, executable host callback, or environment
variable is an implicit source or parameter binding.

`value:null` denotes a free parameter, not zero or an inferred optimum.
Bounds are inclusive, in the parameter's declared unit; null endpoints
mean unbounded, and min MUST NOT exceed max. Parity other than `any`
requires integer type. `code_distance` requires integer type,
dimensionless unit with scale 1, and domain min at least 1; the domain
MAY admit odd, even or all distances. `real` includes exact algebraic real
values expressible by ClosedExpr. A bound integer value MUST be integral
in its declared unit. Bound values MUST satisfy all domain constraints.
No generic model can infer a distance from a risk budget.

A scheduler/estimator MAY select a distance from the declared domain.
Before it uses the value in an operational target binding it MUST write
that choice into the model's parameter value and validate it, thereby
creating the resolved target identity. The certificate records the chosen
value, searched domain, objective and assumptions in its model provenance;
a claim of optimality still requires the existing independently checkable
minimality/search certificate. Hardware and code-distance values cannot
be overridden under an unchanged target_hash. Algorithm values used only
in an estimate belong to that estimate's binding, as specified below.

**Expression evaluation.** A bare Q is dimensionless; a typed literal has
its declared unit. Parameter and quantity references denote base values
with their declared dimensions. Addition, subtraction, min and max require
identical dimensions; multiplication adds dimension exponents and division
subtracts them. Binary operators have exactly two arguments, except ceil
and sqrt which have one. `pow(base,exponent)` requires a dimensionless
integral exponent; it scales the base's dimensions by that integer.
Integer exponents may be positive, zero or negative; zero to zero and
zero to a negative exponent are invalid. `sqrt` is the nonnegative
principal root, halving all dimension exponents. `ceil` accepts only a
dimensionless argument and returns a dimensionless integer. Division by
zero and a negative root are invalid. Rational operations, integer powers,
roots and their order comparisons denote exact real algebraic values;
no binary64 evaluation or floating tolerance substitutes for them.

A quantity's inferred dimensions MUST equal its declared unit's dimensions;
its reported numeric value is its base value divided by that unit's scale.
The dependency graph of quantity references MUST be acyclic; forward
references are permitted. Parameters contain no quantity dependencies.
Unknown references and cycles are invalid even if a particular invocation
would multiply their result by zero. Duration, demand and count quantities
must be nonnegative; concrete demand/count quantities must be integral in
their declared unit. Risk quantities must be dimensionless, scale 1, and
in [0,1]. Scalar quantities may be signed. All concrete results must be
finite. Parameter-dependent validity conditions remain explicit symbolic
obligations until discharged for the requested binding.

These are model expressions, not extensions of every §8.1 expression
context. No gate control count, iteration variable, theta, `eps`,
`log2inv_eps`, `eps_decade` or table context is implicitly bound here.
Their old AST spellings are invalid in a model expression. A source
formula requiring further functions is represented by `external`, whose
`location` identifies the exact expression in the preserved source using
that source format's location convention. It has no invented numeric
value. The report MUST retain that expression, identity and the unresolved
interpretation/checking obligation. A pinned adapter may translate it to
the exact AST above; an independent checker must verify that translation
before calling the result checked. Source-model correctness and physical
assumptions are distinct from verification of that translation and its
arithmetic. Unsupported functions MUST NOT disappear, become zero, or
invalidate program execution. Complete support for promised source models
remains required; an unchecked report does not discharge that requirement.

**Named outputs and conversion.** `map` has exactly one entry for every
normalized quantity, using the §8.3 shape `quantity:[resource,factor]`.
The factor is a positive finite JSON number interpreted by exact decimal
text. In this typed form it MUST equal source-unit scale divided by
destination-resource scale, and dimensions MUST agree. Additional
multiplicity is expressed in the quantity formula, not hidden in this
unit conversion. Within a model, different quantities MUST have different
destination resources; reports are qualified by model name across models.
Any quantity without an existing aligned resource MUST receive a newly
declared named resource of kind `model` with the appropriate unit metadata;
it MUST NOT be silently dropped or coerced to an unrelated unit. A count
or risk quantity MAY instead map to a count resource, but is charged only
when explicitly present in a leaf's `counts`. Duration and demand are not
made additive by naming them resources. The report preserves each mapped
quantity independently of whether any target field consumes it.

**Operational bindings.** A binding path is a canonical RFC 6901 JSON
Pointer into the target document, using `~0`/`~1` escaping and canonical
nonnegative decimal array indices. It MUST name an existing field that
the target schema makes numeric or a §8.1 numeric expression. Missing
fields, duplicate paths, ancestor/descendant path conflicts and pointers
into imports themselves are invalid. Every path identifies one field;
array matching by opcode or vendor name is forbidden.

The following are the allowed destinations and their units/constraints:

| Destination | Unit and additional constraint |
|---|---|
| `gates[i].cost[r]`, each qram operation's `cost[r]` | Unit of declared resource r; r MUST NOT have kind model. Costs nonnegative. |
| `classes[c].capacity` | Logical qubits of class c; integer or existing literal unbounded. Only numeric capacities may be bound. |
| `classes[c].qubit_second`, `classes[c].dirty_rate` | Existing class occupancy multiplier, dimensionless and nonnegative. Physical/logical conversion instead uses the hierarchical interval report. |
| `resources[r].rate[c]` | Resource r unit divided by (logical_qubit·s); nonnegative; only occupancy resources have this field. |
| `capabilities.classical_step_time`, `capabilities.measurement.feedforward_latency` | Seconds, nonnegative. |
| `capabilities.max_parallel`, `capabilities.max_predicate_depth` | Dimensionless integer, subject to their existing constraints. |
| `gates[i].controls.min/max`, `schemes[i].realises.controls.min/max` | Dimensionless integer, existing inclusive-range constraints. |
| `schemes[i].ancillae.count` | Logical qubits, nonnegative integer. |

For all other quantities the named-model-output path remains available;
new numeric destination families require their own published schema and
unit rule, never an implicit substitution. This table does not authorize
binding an epsilon, boolean capability, topology, quantum operation,
interface width, source budget or certificate proof field. Imported
physical risk MUST NOT be substituted for gate epsilon, diamond norm,
operator norm, oracle fault rate, or any correctness assertion.

The binding's local `quantity` MUST resolve. Its dimensions MUST equal
the destination's; `factor` MUST equal source-unit scale divided by the
destination unit's scale. Let x be the exact quantity value times factor.
For `exact`, x MUST be rational and `value` MUST equal x. For `upper`,
`value=ceil(x*10^places)/10^places`; for `lower`,
`value=floor(x*10^places)/10^places`. These bounds use exact comparisons,
including at decimal boundaries. Nonintegral results at integral
destinations are invalid. `places` MUST be zero at such destinations.
Only exact binding is allowed for capability limits and control ranges;
capacity may use exact or lower, and cost/duration/rate fields exact or
upper. Thus rounding cannot overstate a capacity or understate a cost.

A concrete binding overrides the field at its explicit path; the original
field remains as serialized provenance, not a fallback if resolution fails.
Every nonnull `value` MUST be independently recomputed and checked against
the expression and rounding rule. Symbolic templates have `value:null`;
all operational bindings must have valid nonnull values and bound
parameters before concrete scheduling. Changing any operational parameter,
resolved value, formula, mapping, source identity or binding changes the
JCS target bytes and hence target_hash. Scheduling and launch reports bind
that resolved hash; consumers MUST NOT cache under the unresolved target's
hash or reuse a distance selected for another target instance.

**Hierarchy node grammar.** `nodes` is a finite, acyclic map of reusable
node templates. Child names refer to the same map; invoking another model
is explicit. All node and reference validation precedes evaluation,
including children of a zero-count repeat.

```text
Node = {kind:"leaf",duration:Expr,counts:{S:Expr},
        intervals:[Interval,...],shared_uses:[{id:S,slots:Expr},...]}
     | {kind:"sequence"|"parallel",children:[S,...]}
     | {kind:"repeat",child:S,count:Expr}
     | {kind:"invoke",model:S,node:S,arguments:{S:Expr}}
Interval = {pool:S,unit:S,start:Expr,end:Expr,amount:Expr}
```

Leaf duration and interval endpoints have time dimensions and are
nonnegative. Each interval is half-open `[start,end)`, relative to leaf
start, with `0<=start<=end<=duration`. Amount has the interval unit's
dimensions, is nonnegative, and is integral when it measures physical or
logical qubits. Pool identifies a model-qualified accounting pool; units
within a pool MUST have identical dimensions and convert to a common
base. Counts name declared count resources and have their dimensions;
values are converted from base units to the resource unit before summing.
Negative charges or any automatic probability composition are forbidden.
Physical risk summed as a charge is an additive model number, possibly
above one, not a declared failure probability or error guarantee.

Sequence places its children consecutively, starting at zero; parallel
places every child at zero; children may repeat and each occurrence is a
separate invocation, not memoized-away work. Empty compositions have zero
duration, counts, occupancy and peak. Repeat concatenates count copies;
count is a dimensionless nonnegative integer, with zero giving the empty
composition. Invoke instantiates the referenced model's node, retaining
its source/units/assumptions. Its arguments are evaluated in the calling model's environment and bind
only the referenced model's algorithm parameters, with matching dimensions
and conversion to the callee's declared units. They must satisfy the
callee's types/domains; free caller parameters remain symbolic. Other
parameters retain resolved target values. Unspecified algorithm arguments
inherit the report's binding for that model. Dependency cycles, including
cross-model invokes, are invalid; arbitrarily large finite repeats MAY
be evaluated symbolically without materializing their leaves.

Every private interval is instantiated separately at its occurrence's
time origin. Count resources sum once per leaf occurrence. Duration is
the largest leaf end after composition. Pool occupancy is the integral of
simultaneous demand over time; pool peak is its maximum. Concurrent demands
sum, and touching half-open intervals do not overlap. The result MUST be
identical to explicit expansion at any bound finite input, but expansion
is not required. No root total is obtained by summing child peaks or by
adding parent resource totals to their child totals.

Shared resources are declared once by the report's `shared` array, in
root time coordinates. A declaration has exactly
`{id:S,model:S,pool:S,unit:S,start:Expr,end:Expr,amount:Expr,slots:P}`;
model selects the expression/unit namespace, slots is a positive integer,
and other dimensional rules match Interval. Its interval must lie within
the root duration. A leaf shared-use id names that declaration and uses
an integral nonnegative number of slots for its full leaf duration. The
reservation must cover that use, and the simultaneous slot demand must
never exceed its slots. Its physical amount contributes **once** to
pool occupancy and peak, independent of the number of references. A
private interval and a shared declaration cannot claim the same physical
allocation; the producer MUST choose one ownership representation and
provide that ownership in its model assumptions. An idle reserved factory
may still consume physical space. Exclusive factories use slots=1;
parallel uses needing two slots are infeasible until separately reserved
capacity or serialization is supplied. No duplicate shared id is allowed.

**Reports, identity and checking.** Add optional `model_estimates:[Report,...]`
to a §12.3 region record. Existing region resources remain the §8.2
scheduled-leaf report. The additional report has this exact schema:

```text
Report = {
  schema:"bennett.model-estimate/1", target_hash:H,
  subject:{closure_hash:H,invocation:Invocation,context?:ContextKey},
  models:{S:{model_hash:H,parameters:{S:ClosedExpr|null}}},
  root:{model:S,node:S}, shared:[Shared,...],
  outputs:{duration:Result,counts:{S:Result},
           occupancy:{S:{unit:Unit,value:Result}},
           peak:{S:{unit:Unit,value:Result}},
           quantities:{S:{S:Result}}},
  verification:{status:"checked"|"symbolic"|"unchecked",
                obligations:[{model:S,location:S,reason:S},...],
                assumptions:[{model:S,text:S},...]}
}
Result = {exact:ClosedExpr}
       | {symbolic:{model:S,node:S,metric:S,pool?:S,resource?:S}}
       | {interval:{lo:Q,hi:Q,places:N}}
```

Invocation and ContextKey are the existing exact encodings of §12.3 and
§12.1; absence of context means a standalone invocation, not the first
reached context. A scheduled report MUST carry the actual context key and
root schedule binding through its containing certificate. Repeated
contexts cannot share one result unless independently shown equal.
The models map contains exactly the transitively referenced models and
their canonical model_hash values. Its parameters map lists every parameter
exactly once in its declared unit. Hardware/code-distance/other parameters
MUST equal target values; only algorithm parameters may replace null or
algorithm defaults in this report. A parameter's replacement also applies
to all direct uses of that model unless an invoke explicitly binds another
algorithm value. Such per-invoke arguments are part of the checked DAG,
not one misleading global multiplicity. No coherent state or qubit
measurement may supply a parameter during a coherent region.

Duration is reported in seconds; counts in their target resource units;
peak in the displayed Unit of the pool, occupancy in that Unit times
seconds. The quantity outputs contain, for every referenced model and
every mapped quantity, its resource name and converted value, in the
report-level parameter environment. Locally rebound invocations are
additionally identified by their DAG occurrence when requested individually;
their aggregate contribution follows hierarchy composition, not that
report-level quantity display. Missing metric keys are zero only when
recomputation establishes there is no contribution. Foreign or additional
asserted outputs must be checked, never silently ignored.

An exact result uses a dimensionless ClosedExpr for the numeric value in
the stated output unit. Interval endpoints are rational in that same
unit, lo<=hi; at `places` the checker requires the tight outward decimal
floor and ceiling of the exact result. An interval is a bound, not an
exact point. A symbolic result is a thunk for the specified root/node
metric under the report bindings; metric is exactly `duration`, `counts`,
`occupancy`, `peak` or `quantity`. counts/quantity require resource,
occupancy/peak require pool, duration permits neither; model and node
must resolve. It retains the entire dependent expression/hierarchy and
all domain obligations, including min/max and interval overlaps; no
unsupported scalar expression is fabricated. Free parameters may stay
null in a symbolic report, including an unresolved hardware design study.
Such a report cannot discharge a concrete operational binding.

The checker verifies all hashes, typed bindings, graph references, domain
conditions, conversions, rounding, multiplicities, reservations, outputs
and context identity independently. `checked` means these obligations
succeeded for concrete parameters; it does not establish that an external
physical QEC model is true. `symbolic` means structural/unit checks
succeeded and unresolved items are exactly the free-parameter obligations.
`unchecked` is required if any translation, external-function or requested
verification obligation remains unsupported; every such obligation MUST
appear with its model, source/AST location and reason. All imported
assumptions appear in the report's assumptions, qualified by model; they
are not proof labels. A claimed checked status with a missing obligation
is invalid. Results do not become permutation proofs, channel bounds,
operator-norm bounds or oracle fault guarantees by appearing in a checked
model report.

Malformed typed input, conflicting binding or a mathematically invalid
concrete expression produces an explicit model/report validation error
with the failed location. An unresolved operational binding prevents a
concrete schedule, with the missing parameter/expression reported. A
capacity/slot-constrained candidate schedule is infeasible by the existing
schedule interface, which permits another schedule. None of these creates
a new ISA fault code, changes F_B, rejects the program as outside the
language, erases an earlier valid loaded state, or licenses a fallback to
zero or a vendor default. An unused or unresolved analytical model does
not prevent execution under an independently valid concrete target.
Unsupported source-format interpretation yields a preserved unchecked
model plus its explicit implementation obligation; it is not a successful
estimate or permission to abandon the promised adapter capability.

End normative insertion.

### Corresponding §12.3 insertion (proposed normative text)

A region MAY carry `model_estimates` as specified in §8.5. Those reports
are separate, explicitly identified accounting views. They MUST NOT be
added to the region's required §8.2 scheduled-leaf resources, charged again
as parent-plus-child costs, or substituted for the region's logical error
or fault-rate obligations. An invocation's selected QEC distance and
hardware binding affect target_hash; algorithm inputs and hierarchy
reports affect certificate bytes. Physical failure charges and all
physical-model assumptions remain separate from epsilon and correctness
claims. No model output changes the object's instruction semantics.

## Worked serialized import

This is the **complete** `imports.surface` entry, not a complete target
or an accepted vendor fixture. Its preserved source is a small illustrative
formula JSON, with explicit normalized AST. It is not an Azure document
and no vendor adapter was run. The earlier fd0 example's formulas are
retained: d=5, tau=2 ns, p=1/1000, pth=1/10. A code with even distances
would use another declared parity/domain, not change generic validation.

Required target context: a native gate at `gates[0]` has `cost.time:0`
(the explicit binding overrides this provenance value) and `epsilon:0`.
Declare `time` with dimension `{s:"1"}`, scale `"1"`;
`model_cycle` (kind model, seconds); `model_physical_ratio` (kind model,
physical_qubit/logical_qubit); and `risk_charge` (kind count, dimensionless),
all with the metadata above. Retain normal mandatory logical occupancy
and peak declarations. Model hierarchy pool `surface:data` is physical;
it does not replace the logical register ledger or class capacity.

<!-- import-example -->
```json
{
  "map": {
    "cycle": [
      "model_cycle",
      1e-09
    ],
    "physical": [
      "model_physical_ratio",
      1
    ],
    "risk": [
      "risk_charge",
      1
    ]
  },
  "model": {
    "assumptions": [
      "Illustrative QEC formulas; no physical independence assertion."
    ],
    "bindings": [
      {
        "factor": "1/1000000000",
        "path": "/gates/0/cost/time",
        "places": "0",
        "quantity": "cycle",
        "rounding": "exact",
        "value": "3/100000000"
      }
    ],
    "nodes": {
      "algorithm": {
        "child": "cycle",
        "count": {
          "param": "N"
        },
        "kind": "repeat"
      },
      "cycle": {
        "counts": {
          "risk_charge": {
            "quantity": "risk"
          }
        },
        "duration": {
          "quantity": "cycle"
        },
        "intervals": [
          {
            "amount": {
              "args": [
                {
                  "quantity": "physical"
                },
                {
                  "lit": "1",
                  "unit": "logical"
                }
              ],
              "op": "*"
            },
            "end": {
              "quantity": "cycle"
            },
            "pool": "data",
            "start": {
              "lit": "0",
              "unit": "ns"
            },
            "unit": "physical"
          }
        ],
        "kind": "leaf",
        "shared_uses": []
      }
    },
    "parameters": {
      "N": {
        "domain": {
          "max": null,
          "min": "0",
          "parity": "any"
        },
        "role": "algorithm",
        "type": "integer",
        "unit": "one",
        "value": null
      },
      "d": {
        "domain": {
          "max": null,
          "min": "1",
          "parity": "odd"
        },
        "role": "code_distance",
        "type": "integer",
        "unit": "one",
        "value": "5"
      },
      "p": {
        "domain": {
          "max": "1",
          "min": "0",
          "parity": "any"
        },
        "role": "hardware",
        "type": "real",
        "unit": "one",
        "value": "1/1000"
      },
      "pth": {
        "domain": {
          "max": "1",
          "min": "0",
          "parity": "any"
        },
        "role": "hardware",
        "type": "real",
        "unit": "one",
        "value": "1/10"
      },
      "tau": {
        "domain": {
          "max": null,
          "min": "0",
          "parity": "any"
        },
        "role": "hardware",
        "type": "real",
        "unit": "ns",
        "value": "2"
      }
    },
    "quantities": {
      "cycle": {
        "expr": {
          "args": [
            {
              "args": [
                "3",
                {
                  "param": "tau"
                }
              ],
              "op": "*"
            },
            {
              "param": "d"
            }
          ],
          "op": "*"
        },
        "role": "duration",
        "unit": "ns"
      },
      "physical": {
        "expr": {
          "args": [
            {
              "lit": "2",
              "unit": "ratio"
            },
            {
              "args": [
                {
                  "param": "d"
                },
                "2"
              ],
              "op": "pow"
            }
          ],
          "op": "*"
        },
        "role": "scalar",
        "unit": "ratio"
      },
      "risk": {
        "expr": {
          "args": [
            "1/10",
            {
              "args": [
                {
                  "args": [
                    {
                      "param": "p"
                    },
                    {
                      "param": "pth"
                    }
                  ],
                  "op": "/"
                },
                {
                  "args": [
                    {
                      "args": [
                        {
                          "param": "d"
                        },
                        "1"
                      ],
                      "op": "+"
                    },
                    "2"
                  ],
                  "op": "/"
                }
              ],
              "op": "pow"
            }
          ],
          "op": "*"
        },
        "role": "risk",
        "unit": "one"
      }
    },
    "schema": "bennett.import-model/1",
    "source": {
      "bytes": "7b226379636c65223a22332a7461752a64222c22706879736963616c223a22322a645e32222c227269736b223a22302e312a28702f707468295e2828642b31292f3229227d",
      "format": "bennett.illustrative-qec",
      "media_type": "application/json",
      "sha256": "426daeed30fce3fa03234faff9d8ea543005c4681917f126b8dbaca6508c9be5",
      "version": "1"
    },
    "units": {
      "logical": {
        "dimension": {
          "logical_qubit": "1"
        },
        "scale": "1"
      },
      "ns": {
        "dimension": {
          "s": "1"
        },
        "scale": "1/1000000000"
      },
      "one": {
        "dimension": {},
        "scale": "1"
      },
      "physical": {
        "dimension": {
          "physical_qubit": "1"
        },
        "scale": "1"
      },
      "ratio": {
        "dimension": {
          "logical_qubit": "-1",
          "physical_qubit": "1"
        },
        "scale": "1"
      }
    }
  },
  "version": "1"
}
```

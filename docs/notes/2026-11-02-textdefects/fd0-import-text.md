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
are distinct axes. Other axes MUST include their source format and version in the axis
identifier, unless an explicit dimension-preserving conversion is present in the normalized
formula. An axis spelling itself is its identity; there is no implicit
unit-equivalence registry.
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
            pool?:S,resource?:S,quantity?:S}
```

Each model-resource objective term has exactly one selector keyed by that
resource name; other selector keys are invalid. Model/node name the
hierarchy to evaluate under its resolved target and current algorithm
bindings. Duration has seconds units and no optional selector field;
counts requires resource, quantity requires its local quantity name, and
occupancy/peak require pool; other optional selector fields are forbidden.
Quantity selects that model output and converts through its map, counts
selects its aggregate count, and pool metrics
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
values expressible by ClosedExpr. A parameter value is a dimensionless numeric ClosedExpr coefficient in
its declared unit; the declared scale is applied exactly once. A bound
integer value MUST be integral in its declared unit. Bound values MUST
satisfy all domain constraints.
No generic model can infer a distance from a risk budget.

A scheduler/estimator MAY select a distance from the declared domain.
Before it uses the value in an operational target binding it MUST write
that choice into the model's parameter value and validate it, thereby
creating the resolved target identity. The target parameter value/domain and target objective identify the
chosen instance. The report records the actually searched domain, search
method and objective as model-qualified verification.assumptions strings,
with no machine-checked optimality claim inferred from that prose. A claim
of optimality requires a separately published, independently checkable
minimality/search witness format; this section introduces no such claim
kind or verdict. Hardware and code-distance values cannot
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
normalized quantity, using the §8.3 pair shape `quantity:[resource,factor]`.
In this typed form factor is either P (a rational string) or a positive
finite JSON number interpreted by exact decimal text. Rational strings
allow exact conversions such as 1/3; no decimal approximation is required.
The legacy untyped form retains its existing JSON-number factor. In this typed form it MUST equal source-unit scale divided by
destination-resource scale, and dimensions MUST agree. Additional
multiplicity is expressed in the quantity formula, not hidden in this
unit conversion. Several quantities MAY map to one resource with aligned dimensions; each
remains separately identified by model and quantity name, without automatic
summation of their displayed values.
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
logical qubits. Pool identifies a local accounting pool. Report pool keys are the JCS
encoding of the two-string array [model,pool], used as a JSON object key;
selector pool fields use that same qualified encoding. This avoids aliases
when either name contains punctuation or Unicode. Units
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
a dimensionless integral nonnegative number of slots for its full leaf duration. The
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
           quantities:{S:{S:{resource:S,value:Result}}}},
  verification:{status:"checked"|"symbolic"|"unchecked",
                obligations:[{model:S,location:S,reason:S},...],
                assumptions:[{model:S,text:S},...]}
}
Result = {exact:ClosedExpr}
       | {symbolic:{model:S,node:S,metric:S,pool?:S,resource?:S,quantity?:S}}
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
every mapped quantity keyed by its local name, its destination resource
name and converted value, in the report-level parameter environment. Locally rebound invocations are
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
`occupancy`, `peak` or `quantity`. Counts requires resource, quantity
requires quantity, occupancy/peak require pool, and all other optional
selector fields are forbidden. Model and node must resolve. It retains the entire dependent expression/hierarchy and
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
capacity/slot-constrained candidate schedule is infeasible with
`E_SCHEDULE_INFEASIBLE` under the existing schedule interface, which permits another schedule. None of these creates
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
or an accepted vendor fixture. It is pretty-printed for reading; the complete target is canonicalized
with RFC 8785 before identity calculation. Its preserved source is a small illustrative
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
and peak declarations. Model hierarchy pool `["surface","data"]` is physical;
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

For a report bound to the referenced node `surface:algorithm`, bind N=3 in
`Report.models.surface.parameters`; retain every hardware/code-distance
value from the target, and set `shared:[]`. This changes the report's bytes,
not the target. N left null yields a symbolic repeat with duration
`N*30 ns`, count charge `N/10^7`, and occupancy `N*1500 physical-qubit·ns`.
Its peak is piecewise: 0 for N=0, 50 for positive N. Do not simplify it
to 50 without retaining the N>0 premise. A fixed distance selection d=3
instead creates new target bytes and requires recomputing `Binding.value`
as `9/500000000` seconds; an old 30 ns resolved value must reject.

| Bound instance / composition | Duration | Physical peak | Physical occupancy | Additive risk charge |
|---|---:|---:|---:|---:|
| One cycle, d=5 | 30 ns | 50 | 1500 qubit·ns | 1/10^7 |
| Three sequential cycles, d=5 | 90 ns | 50 | 4500 qubit·ns | 3/10^7 |
| Three independent parallel cycles, d=5 | 30 ns | 150 | 4500 qubit·ns | 3/10^7 |
| Three sequential cycles, d=3 | 54 ns | 18 | 972 qubit·ns | 3/10^5 |
| Zero repetitions | 0 | 0 | 0 | 0 |

The complete entry above converts cycle 30 ns to `3/100000000` seconds
using factor `1/1000000000`. Its physical quantity is 50
physical_qubit/logical_qubit, not 50 physical qubits until multiplied by
one logical qubit in the leaf interval. The risk formula is an illustrative
external physical-model assumption. Even when its risk is positive, the
logical gate's exact `epsilon:0` stays 0, and any checked oracle fault rate
is independently obtained from F_B and its declared domain.

For a shared-factory variant remove its private physical interval, add
`shared_uses:[{id:"factory0",slots:"1"}]` to each factory-using leaf, and
declare one typed shared reservation in the report:

```json
{"id":"factory0","model":"surface","pool":"factory","unit":"physical","start":{"lit":"0","unit":"ns"},"end":{"lit":"90","unit":"ns"},"amount":{"lit":"50","unit":"physical"},"slots":"1"}
```

Three sequential 30 ns uses fit that reservation: peak 50 and occupancy
4500 physical-qubit·ns, with no threefold factory space charge. Three
parallel uses of a one-slot factory are infeasible. A model that actually
supplies a three-slot factory may reserve 50 physical qubits for 30 ns
and assert capacity 3 as an external-model assumption; then peak 50 and
occupancy 1500 physical-qubit·ns are the checked **accounting**, not a proof
that such hardware exists. The private-data and factory pool names remain
distinct. The serialized source and normalized model hashes are recomputed
by the check below and recorded in the job's `model-results.json`; they
are fixture identities, not published pins.

## Conformance vectors

These are acceptance obligations for the proposed format. “Error” below
means model/report validation or requested-schedule failure as specified,
never a new source-program trap or quantum oracle fault.

| Vector | Required verdict / result |
|---|---|
| Serialized example, d=5/N=3 | Valid import; 30 ns binding; sequential totals above. |
| d=3 with updated resolved value and hash | 18 ns/cycle, 18 physical/logical, risk 1/10^5. |
| Change d without updating Binding.value | Error at stale resolved value, before concrete scheduling. |
| d=4 under this model's declared odd domain; d=0 | Domain error. Another model declaring even distance with appropriate formulas remains legal. |
| Null N; N=0; N=3 | Symbolic repeat / empty composition / three copies, respectively. |
| Null hardware parameter used by a binding | Valid symbolic template; incomplete concrete schedule, naming that parameter; unrelated object execution unaffected. |
| Forward quantity reference; quantity/invoke cycle even under repeat 0 | Forward reference valid; either cycle error. |
| Division by zero; negative root; nonintegral pow exponent; zero^zero | Exact expression error. Integer negative powers such as 2^−3=1/8 valid. |
| sqrt(9/4); sqrt(2) | Exact 3/2; exact algebraic expression or verified outward enclosure. At 3 places sqrt(2) is enclosed by [1.414,1.415]. |
| Exact unit conversion factor `"1/3"` | Preserve the rational factor without rounding or a JSON binary64 restriction. |
| 1/3 with upper/lower at 3 places | 0.334/0.333; exact rational mode retains 1/3. |
| Add time and dimensionless risk; bind physical/logical ratio into time | Unit error, no heuristic unit/name conversion. |
| Unknown target path; path into epsilon; duplicate path; bad escaping | Binding error, before any target realization is used. |
| Source byte/hash mismatch; same name with another source version | Mismatch error; distinct identity, respectively. |
| Missing mapped quantity; unaligned quantity with newly declared model resource | Missing output error; valid preserved named resource, respectively. |
| Two quantities mapped to one aligned resource | Preserve both quantity-keyed displays; charge only the explicitly composed counts. |
| Unknown expression field / `var:eps` / accuracy bucket | Invalid normalized expression, never a model epsilon binding. |
| Unsupported source expression at a retained external location | Unchecked report with complete source/obligation; no invented value or algorithm rejection. |
| Three independent sequential/parallel cycles | Same count/occupancy, different makespan and peak as above. |
| Duplicate occurrence of one DAG child | Counts and private demand multiply; DAG storage sharing does not share a physical instance. |
| Shared factory sequence / parallel with one slot | One factory charged once / infeasible simultaneous use; serialized alternative remains legal. |
| Shared interval outside root time or not covering use | Error at reservation/use. Touching intervals have no overlap. |
| Cross-model symbolic invoke of an algorithm argument | Convert dimensions/units, retain caller dependency, qualify identities; no hardware override. |
| Report/certificate with wrong target, context or model hash | Reject the asserted report; no first-context or old-hash cache fallback. |
| Positive physical risk with exact logical gates | Risk output/charge remains nonzero; epsilon and logical fault claims unchanged. |
| Model objective with and without an exact root selector | Bound scalar under ordinary objective rules / unresolved objective, never implicit scalarization. |

The executable check below covers the fixture's scalar arithmetic and
unit conversions; parameter/formula negative cases; forward reference and
zero-multiplied cycle; exact rational roots/powers and rational enclosure
inequalities; sequence/parallel/repeat/shared-accounting examples; hashes;
and document structure. Cross-model invokes, actual operational target
resolution, source adapter translation, generic algebraic evaluation,
context validation and the remaining consumer acceptance vectors remain **proposed**.
A small rational example checker is not an implementation of this format.

## Consumer impact, release and what remains for 0.10

For 0.9 retain concrete scalar target instances when fully bound, their
original model/provenance, and all promised algorithm scope. This is a
compatibility path, not completion of symbolic import support. Do not
pretend a new JSON field is already admitted by 0.9's closed schema. The
reference interpreter's current declared 0.8 surface also does not gain
0.10 admission through a documentation proposal.

For 0.10, target importers parse the pinned source version, preserve raw
bytes, normalize formulas/units/domains, and supply resolved bindings.
The scheduler selects distance using classical data, creates a distinct
resolved target, and evaluates model objectives without materializing all
gates. The estimator retains symbolic hierarchies and reports intervals,
shared factories and named quantities. The VM/circuit executors preserve
the ordinary logical machine semantics and use the resolved costs;
physical simulation is not inferred from a resource-risk output. The
independent checker verifies translation where supported, target/report
identity, arithmetic, accounting and truthful obligation labels. The
conformance harness must run all format vectors with separate producers
and checker; a consumer should not share a vendor importer as its sole
verification oracle across a published contract boundary.

No reference catalog recipe, source syntax or 0.9 envelope changes in
this proposal. Under the normal full-version 0.10 publication, affected
semantic envelopes and source hashes change because their version changes;
do not promise a global no-repin migration. Target hashes change for new
models/resolved distances; target-bound schedules and certificates must
be re-bound and checked. Algorithms' estimate parameters and hierarchy
change the report/certificate bytes, independently of source identity.
Catalog bytes/hash change only if separately revised; fd0 needs no such
recipe change. No artifact is re-pinned by this job.

Remaining 0.10 work is concrete: council acceptance of these new
encodings and unit/accounting rules; machine-readable target/certificate
schemas and vectors; source-format/version adapters including retained
external expressions; an independent translation checker or versioned
checkable witness mechanism for each such expression; scalable symbolic
DAG evaluation, distance-search implementation and a versioned checkable witness for
optimality claims when requested; operational-resolution and context
validation in consumers; compatibility and publication tooling. Routing,
physical pulse/noise simulation and a general proof that vendor QEC models
predict hardware are separate mechanics/assumptions, not hidden obligations
this arithmetic proposal claims to solve. It supplies no automatic
independence law for physical failures and no physical probability to
logical-norm conversion. A user may import an explicit, separately checked
risk-composition formula as model data; no such capability is prohibited.

Keep **b-isa-spec-fd0** open for that contract/adapter work and
**b-targets-gyp** open at its owning repository until the promised complete
path exists. Preserving an unchecked external obligation is honest interim
reporting, not a non-narrowing substitute for implementing it. Neither bead
was mutated by this notes-only job. The exact new mechanism is described
above; no new cross-repo issue is necessary to name the same blocker.

## Verification

All shell commands used the headless prefix and absolute working directory
shown in the [draw note](0h4-draw-tie.md#verification-and-reproduction).
No git command or tracker mutation was run. Both deliverables are proposals.
The full `scripts/pre-push` gate uses git/sibling worktrees and therefore
was read but not run under the runner's rules. No sibling gate was run.

The self-check script below is also saved under the named job path. Its
exact invocation is:

```sh
timeout 120 python3 -I -B /home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp/sol-isa-textdefects/check_notes.py
```

It is deliberately a finite rational witness calculation and document
check. The nonrational sqrt(2) check proves the claimed enclosure by exact
squaring; it does not claim a general algebraic-number library. Source
identity and normalized formula evaluation are checked separately from
external physical-model validity.

<!-- check-notes -->
```python
from pathlib import Path
from fractions import Fraction as F
from copy import deepcopy
from math import isqrt, ceil, floor
import hashlib, json, re
ROOT=Path('/home/tobias/Projects/bennett-workspace/bennett-isa-spec')
OUT=ROOT/'docs/notes/2026-11-02-textdefects'
JOB=ROOT/'.tmp/sol-isa-textdefects'
text=(OUT/'fd0-import-text.md').read_text()
entry=json.loads(text.split('<!-- import-example -->\n```json\n',1)[1].split('\n```',1)[0])
m=entry['model']
def q(x):
    assert isinstance(x,str)
    v=F(x)
    assert str(v)==x, ('noncanonical rational',x)
    return v
def dims(d):return {k:q(v) for k,v in d.items() if q(v)}
def dimcombine(a,b,sign=1):
    out=dict(a)
    for k,v in b.items():out[k]=out.get(k,F(0))+sign*v
    return {k:v for k,v in out.items() if v}
def unit(n):
    u=m['units'][n]
    assert set(u)=={'dimension','scale'} and q(u['scale'])>0
    return q(u['scale']),dims(u['dimension'])
class Unbound(Exception):pass

def ev(e,params,seen=()):
    if isinstance(e,str):return q(e),{}
    assert isinstance(e,dict)
    if set(e)=={'lit','unit'}:
        scale,d=unit(e['unit']); return q(e['lit'])*scale,d
    if set(e)=={'param'}:
        p=m['parameters'][e['param']]; x=params[e['param']]
        if x is None:raise Unbound(e['param'])
        v,d=ev(x,{},seen); assert not d
        scale,d=unit(p['unit']); return v*scale,d
    if set(e)=={'quantity'}:
        n=e['quantity']; assert n not in seen, ('cycle',n)
        v,d=ev(m['quantities'][n]['expr'],params,seen+(n,))
        assert d==unit(m['quantities'][n]['unit'])[1]
        return v,d
    if set(e)=={'external'}:raise Unbound(e['external']['location'])
    assert set(e)=={'op','args'}
    op=e['op']; a=e['args']
    assert op in ['+','-','*','/','min','max','ceil','sqrt','pow']
    assert len(a)==(1 if op in ['ceil','sqrt'] else 2)
    x,dx=ev(a[0],params,seen)
    if op=='ceil':assert not dx; return F(ceil(x)),{}
    if op=='sqrt':
        assert x>=0
        n,d=isqrt(x.numerator),isqrt(x.denominator)
        assert n*n==x.numerator and d*d==x.denominator, 'nonrational root: use exact algebraic AST/enclosure'
        return F(n,d),{k:v/2 for k,v in dx.items()}
    y,dy=ev(a[1],params,seen)
    if op in ['+','-','min','max']:
        assert dx==dy, ('unit mismatch',dx,dy)
        return {'+':lambda:x+y,'-':lambda:x-y,'min':lambda:min(x,y),'max':lambda:max(x,y)}[op](),dx
    if op=='*':return x*y,dimcombine(dx,dy)
    if op=='/':assert y!=0; return x/y,dimcombine(dx,dy,-1)
    assert not dy and y.denominator==1
    assert x!=0 or y>0
    return x**int(y),{k:v*y for k,v in dx.items()}

def defaults():return {k:p['value'] for k,p in m['parameters'].items()}
def validate_param(p,x):
    if x is None:return
    v,d=ev(x,{})
    assert not d
    if p['type']=='integer':assert v.denominator==1
    dom=p['domain']
    assert dom['min'] is None or v>=q(dom['min'])
    assert dom['max'] is None or v<=q(dom['max'])
    assert dom['parity']=='any' or (p['type']=='integer' and int(v)%2=={'odd':1,'even':0}[dom['parity']])
    if p['role']=='code_distance':assert p['type']=='integer' and unit(p['unit'])== (F(1),{}) and q(dom['min'])>=1

def fails(call):
    try:call()
    except (AssertionError,ZeroDivisionError,KeyError,Unbound):return
    raise AssertionError('negative vector did not fail')
assert set(entry)=={'version','map','model'}
assert set(m)=={'schema','source','units','parameters','quantities','bindings','nodes','assumptions'}
assert m['schema']=='bennett.import-model/1'
assert set(m['source'])=={'format','version','media_type','bytes','sha256'}
assert hashlib.sha256(bytes.fromhex(m['source']['bytes'])).hexdigest()==m['source']['sha256']
for p in m['parameters'].values():
    assert set(p)=={'role','type','unit','value','domain'}
    assert set(p['domain'])=={'min','max','parity'}
    validate_param(p,p['value'])
for n in m['units']:unit(n)
resource_units={'model_cycle':(F(1),{'s':F(1)}),'model_physical_ratio':(F(1),{'physical_qubit':F(1),'logical_qubit':F(-1)}),'risk_charge':(F(1),{})}
assert set(entry['map'])==set(m['quantities'])
assert len({a[0] for a in entry['map'].values()})==len(entry['map'])
for name,(r,factor) in entry['map'].items():
    u=m['quantities'][name]['unit']; sc,d=unit(u); rc,rd=resource_units[r]
    assert d==rd and F(str(factor))==sc/rc
params=defaults()
for d,ns,physical,risk in [('5',30,50,F(1,10**7)),('3',18,18,F(1,10**5))]:
    p=dict(params,d=d)
    assert ev({'quantity':'cycle'},p)==(F(ns,10**9),{'s':F(1)})
    assert ev({'quantity':'physical'},p)==(F(physical),{'physical_qubit':F(1),'logical_qubit':F(-1)})
    assert ev({'quantity':'risk'},p)==(risk,{})
b=m['bindings'][0]
assert set(b)=={'path','quantity','factor','rounding','places','value'}
assert b['path']=='/gates/0/cost/time' and b['rounding']=='exact'
cyc,dc=ev({'quantity':b['quantity']},params)
assert q(b['factor'])==unit(m['quantities'][b['quantity']]['unit'])[0]
assert cyc==q(b['value'])==F(3,100000000)
# Binding factors multiply quantities in their declared source unit.
assert F(30)*q(b['factor'])==q(b['value'])
fails(lambda:validate_param(m['parameters']['d'],'4'))
fails(lambda:validate_param(m['parameters']['d'],'0'))
even=deepcopy(m['parameters']['d']); even['domain']['parity']='any'; validate_param(even,'4')
fails(lambda:ev({'param':'N'},params))
for e in [{'op':'/','args':['1','0']},{'op':'sqrt','args':['-1']},{'op':'pow','args':['2','1/2']},{'op':'pow','args':['0','0']},{'var':'eps'},{'op':'log2inv_eps','args':[]},{'param':'absent'},{'op':'+','args':[{'param':'tau'},'1']}]:
    fails(lambda e=e:ev(e,params))
assert ev({'op':'pow','args':['2','-3']},params)==(F(1,8),{})
assert ev({'op':'sqrt','args':['9/4']},params)==(F(3,2),{})
# Sqrt(2) tight outward 3-place enclosure, without approximating sqrt.
lo,hi=F(1414,1000),F(1415,1000)
assert lo*lo<2<hi*hi and hi-lo==F(1,1000)
assert (ceil(F(1,3)*1000)/F(1000))==F(334,1000)
assert floor(F(1,3)*1000)/F(1000)==F(333,1000)
# Forward quantity references work; cycles reject even beneath a zero coefficient.
m['quantities']['alias']={'unit':'ns','role':'duration','expr':{'quantity':'cycle'}}
assert ev({'quantity':'alias'},params)==ev({'quantity':'cycle'},params)
m['quantities']['bad']={'unit':'one','role':'scalar','expr':{'op':'*','args':['0',{'quantity':'bad'}]}}
fails(lambda:ev({'quantity':'bad'},params))
del m['quantities']['alias'];del m['quantities']['bad']

nodes=deepcopy(m['nodes'])
nodes.update(seq={'kind':'sequence','children':['cycle']*3},par={'kind':'parallel','children':['cycle']*3})
def graph(n,seen=()):
    assert n not in seen
    a=nodes[n];k=a['kind']
    if k in ('sequence','parallel'):
        for c in a['children']:graph(c,seen+(n,))
    elif k=='repeat':graph(a['child'],seen+(n,))
    else:assert k=='leaf'
for n in nodes:graph(n)
def tree(n,p,origin=F(0)):
    a=nodes[n];k=a['kind']
    if k in ('sequence','parallel','repeat'):
        children=a.get('children')
        if k=='repeat':
            count,dd=ev(a['count'],p);assert not dd and count>=0 and count.denominator==1
            children=[a['child']]*int(count)
        duration=F(0);counts={};intervals=[];uses=[]
        for c in children:
            off=duration if k!='parallel' else F(0)
            dt,cs,ins,us=tree(c,p,origin+off)
            duration=max(duration,dt) if k=='parallel' else duration+dt
            for r,v in cs.items():counts[r]=counts.get(r,F(0))+v
            intervals+=ins;uses+=us
        return duration,counts,intervals,uses
    assert k=='leaf'
    dt,dd=ev(a['duration'],p);assert dd=={'s':F(1)} and dt>=0
    counts={r:ev(e,p)[0]/resource_units[r][0] for r,e in a['counts'].items()}
    ins=[]
    for i in a['intervals']:
        start,ds=ev(i['start'],p);end,de=ev(i['end'],p);amount,da=ev(i['amount'],p)
        assert ds==de=={'s':F(1)} and 0<=start<=end<=dt
        assert da==unit(i['unit'])[1] and amount>=0 and amount.denominator==1
        ins.append((origin+start,origin+end,amount))
    uses=[(origin,origin+dt,u['id'],ev(u['slots'],p)[0]) for u in a['shared_uses']]
    return dt,counts,ins,uses

def stats(ins):
    times=sorted({t for s,e,a in ins for t in (s,e)})
    peak=area=F(0)
    for s,e in zip(times,times[1:]):
        demand=sum((a for st,en,a in ins if st<=s<en),F(0))
        peak=max(peak,demand);area+=demand*(e-s)
    return peak,area
for node,expected in [('seq',(F(90,10**9),F(50))),('par',(F(30,10**9),F(150)))]:
    dt,counts,ins,uses=tree(node,params)
    peak,area=stats(ins)
    assert (dt,peak)==expected and area==F(4500,10**9)
    assert counts=={'risk_charge':F(3,10**7)} and not uses
p=dict(params,N='3')
assert tree('algorithm',p)==tree('seq',p)
p['N']='0';assert tree('algorithm',p)==(0,{},[],[])
nodes['badzero']={'kind':'repeat','count':'0','child':'missing'}
fails(lambda:graph('badzero'));del nodes['badzero']
# One reserved physical factory, three uses; private intervals are removed.
nodes['shared']=deepcopy(nodes['cycle']);nodes['shared']['intervals']=[]
nodes['shared']['shared_uses']=[{'id':'factory0','slots':'1'}]
nodes['sharedseq']={'kind':'sequence','children':['shared']*3}
nodes['sharedpar']={'kind':'parallel','children':['shared']*3}
def shared(node,slots):
    dt,cs,ins,uses=tree(node,params)
    times=sorted({t for s,e,i,a in uses for t in (s,e)})
    for s,e in zip(times,times[1:]):assert sum(a for st,en,i,a in uses if st<=s<en)<=slots
    assert not ins
    return stats([(F(0),dt,F(50))])
assert shared('sharedseq',1)==(50,F(4500,10**9))
fails(lambda:shared('sharedpar',1))
assert shared('sharedpar',3)==(50,F(1500,10**9))
# Identity changes are explicit, not hidden in caller defaults.
def modelhash(x):return hashlib.sha256(json.dumps(x,sort_keys=True,separators=(',',':')).encode()).hexdigest()
hash0=modelhash(m);changed=deepcopy(m);changed['parameters']['d']['value']='3'
assert modelhash(changed)!=hash0
assert hashlib.sha256(bytes.fromhex(m['source']['bytes'])+b' ').hexdigest()!=m['source']['sha256']
(JOB/'model-results.json').write_text(json.dumps({'model_hash':hash0,'source_hash':m['source']['sha256'],'cycle_seconds':'3/100000000','physical_per_logical':'50','risk':'1/10000000','sequential_ns':'90','parallel_ns':'30','sequential_peak':'50','parallel_peak':'150','occupancy_physical_qubit_ns':'4500'},indent=2)+'\n')
# Document checks cover both owned deliverables, without mutating specs.
for name in ['0h4-draw-tie.md','fd0-import-text.md']:
    path=OUT/name;body=path.read_text()
    assert body.count('\n```')%2==0,(name,'fences')
    assert not re.search(r'\b(?:TODO|TBD)\b',body.split('<!-- check-notes -->')[0])
    for target in re.findall(r'\[[^\]]+\]\(([^)]+)\)',body):
        if not target.startswith(('https://','http://','#')):
            assert (path.parent/target.split('#')[0]).exists(),(name,target)
    for line in body.splitlines():
        if line.startswith('|'):
            assert len(re.findall(r'(?<!\\)\|',line))==len(re.findall(r'(?<!\\)\|',previous_table)) if previous_table else True,(name,line)
            previous_table=line
        else:previous_table=''
    for word in ['Conformance','Verification','North Star']:
        assert word in body,(name,word)
assert 'strictly greater' in (ROOT/'spec/Bennett-ISA-Spec-0.9.md').read_text()
print('PASS: exact d=3/d=5 formulas and units; import identity/conversion; parameter/expression negatives; sequence/parallel/repeat/shared-factory vectors; two notes links/fences/tables')
```

## Recorded verification results and exact replay command

The build, two existing measurement tests, 28 reference draw vectors,
published K17 carrier and seven negative CLI vectors are recorded in the
[draw note](0h4-draw-tie.md#verification-and-reproduction). The model and
document check above exited 0. Its fixture hashes are:

```text
source raw bytes: 426daeed30fce3fa03234faff9d8ea543005c4681917f126b8dbaca6508c9be5
normalized model: c69312a9d3fc53728a6554b44bae48a9fab7eaefbcca3651c75ad9a0128e022f
```

These additional commands were run under the same headless prefix:

```sh
timeout 120 python3 -I -B /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/tools/ref_catalog.py --check
timeout 120 python3 -I -B /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/fixtures/accepted-k82-09/generate.py --check
patch --dry-run --batch -p1 --input /home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp/sol-isa-textdefects/0h4.patch
```

Results: catalog check passed **219 recipes**, envelope hash
`5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197`;
K82 generator passed **54 named interfaces, 20 edges, 86 instances and
16 cases**; patch dry-run accepted all three hunks without editing the
spec. The publication diff was independently regenerated with `diff` and
matched its checked-in bytes exactly. No new model consumer, general
algebraic evaluator, cross-model adapter or full repository gate was run.

The following exact Python command was run with the environment prefix.
It extracts the embedded scripts, executes them, dry-runs the embedded
proposal diff and verifies the published diff. The earlier cargo build is
a prerequisite; the scripts and artifacts it writes stay in the explicitly
authorized job scratch directory.

```sh
python3 -I -B - <<'PY'
from pathlib import Path
import subprocess
root=Path('/home/tobias/Projects/bennett-workspace/bennett-isa-spec')
job=root/'.tmp/sol-isa-textdefects'
notes=root/'docs/notes/2026-11-02-textdefects'
for name,marker,output in [('0h4-draw-tie.md','```python\n','draw_probe.py'),('fd0-import-text.md','<!-- check-notes -->\n```python\n','check_notes.py')]:
    code=(notes/name).read_text().split(marker,1)[1].split('\n```',1)[0]
    compile(code,str(notes/name),'exec')
    (job/output).write_text(code+'\n')
    subprocess.run(['timeout','120','python3','-I','-B',str(job/output)],check=True)
patch=(notes/'0h4-draw-tie.md').read_text().split('```diff\n',1)[1].split('\n```',1)[0]+'\n'
(job/'0h4.patch').write_text(patch)
subprocess.run(['patch','--dry-run','--batch','-p1','--input',str(job/'0h4.patch')],cwd=root,check=True)
r=subprocess.run(['diff','-u','--label','a/spec/Bennett-ISA-Spec.md','--label','b/spec/Bennett-ISA-Spec-0.9.md',str(root/'spec/Bennett-ISA-Spec.md'),str(root/'spec/Bennett-ISA-Spec-0.9.md')],capture_output=True)
assert r.returncode==1 and r.stdout==(root/'spec/isa-0.8-to-0.9.diff').read_bytes()
print('PASS: embedded scripts replayed; proposal diff dry-run; published 0.8-to-0.9 diff matches exactly')
PY
```

A read-only input audit compared 917 source/spec/conformance files with the
job's saved snapshot: the measurement sources, spec and catalog were
unchanged; two unrelated reference files (`oracle/program.rs` and
`tests/isa09_forwarding.rs`) changed concurrently during the job. The
recorded execution evidence is for the job's built binary, not a claim
that other agents stopped editing the checkout. This job writes only the
two deliverables and its authorized scratch/progress; it runs no git.

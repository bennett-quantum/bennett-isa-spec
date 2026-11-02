# fd0 — exact import and hierarchical-estimate text for council review

For `b-isa-spec-fd0`, originating at `b-targets-gyp`. Proposal only:
**target/cert 0.10**, with full-version publication under the repository's
change rules. This does not amend 0.9, its catalog, or any consumer.

## Round-1 blocking items and resolution

Read the [round-1 report](../../../../bennett/docs/council/td-fd0-2026-11-02/round-1/report.md)
and both out-sol.json and out-codex.json there, the convenor table and
bennett-4ht ruling. These are proposed resolutions; no new acceptance is claimed.

| Item | Change and semantic witness |
|---|---|
| sol FD0-1 — operational parameter can be rebound by report/invoke | Freeze every transitive operational dependency regardless of role. N=1 gives a one-second bound gate; report/invoke N=2 under that hash rejects. A new N=2 target recomputes value 2 and its hash. |
| sol FD0-2 — standalone subject lacks variant/controls | Require explicit variant and inherited_controls in Report.subject. The same X closure at forward/0 versus forward/1 controls can cost 1 versus 2 seconds; the subjects now differ. |
| sol FD0-3; codex B2 — missing selector called unresolved | Missing objective.model_outputs selector is a validation error. Present selector with free inputs remains symbolic; present selector with concrete inputs evaluates. |
| codex B1 — objective root, parameter bindings and shared reservations unbound | Add ModelInput in request and schedule.bindings.model_inputs, hence schedule_hash; certifying reports must reproduce it. One 90 ns factory reservation of 50 versus 100 qubits gives peaks 50/100, occupancy 9/2000000 versus 9/1000000, and distinct schedule hashes. |
| sol FD0-4 — full 0.10 publication leaves default ref catalog identity wrong | Republish ref isa/version metadata and envelope at 0.10 with all 219 recipes unchanged. Hash changes from 5de4e499… to 83cc9c17…; exact full hashes and reproduction below. |
| sol FD0-5; codex B3 — missing mixed-object report inventory | Add the §12.3 attachment rule and shared-scope citation below. Native main + one device module has device-only model reports; main/argv/libc are outside them. |

## Defect, alternatives and scope

This full revision supplies the field-level contract, serialized import,
publication route and acceptance vectors. ISA 0.9 §8.3's
`imports.{model}.{version,map}` cannot bind code distance or formulas to
logical costs, and §8.2's leaf-event aggregation cannot express a factory
hierarchy. `bd show b-isa-spec-fd0` confirms the missing exponentiation,
code-distance selection and symbolic-count questions. Both published
versions' §8.1 languages lack the required parameter/quantity bindings.

The alternatives are: (1) fully bind a model externally into a concrete
0.9 target, preserving source and hierarchy as provenance; this keeps
concrete estimates available but cannot interchange symbolic models;
(2) add parameter expressions alone, which leaves destinations, hierarchy
and resource ownership undecided; (3) the selected model-as-data plus
separately bound hierarchy report, specified completely below. Summing
all quantities or treating physical failure risk as epsilon is unsound.

Decisions in this proposal are deliberate:

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
Extend §11 requests/schedule bindings with model_inputs and the additive
setter below. The text from “8.5” through “End normative insertion” is the proposed
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
resource name; other selector keys are invalid. A missing required selector
is a model validation error at objective.model_outputs.<resource>, not a
symbolic or unresolved expression. Model/node name the
hierarchy evaluated using the resolved target and that objective term's
explicit ModelInput below; no analytical report implicitly supplies inputs. Duration has seconds units and no optional selector field;
counts requires resource, quantity requires its local quantity name, and
occupancy/peak require pool; other optional selector fields are forbidden.
Quantity selects that model output and converts through its map, counts
selects its aggregate count, and pool metrics
select the model-qualified pool. The selected dimensions must agree with
the objective resource's and the value is converted to its scale. Existing
weighted/lexicographic rules then apply to that scalar. An unbound
objective is retained symbolically and has no silently assigned scalar
value or completed concrete optimization verdict.

**Objective input binding (§11 extension).** The scheduling request and
schedule.bindings gain optional `model_inputs:{S:ModelInput}`. When there
are model-resource objective terms, its keys MUST equal exactly their
resource names; with none, omit it or use {}. Each entry is:

```text
ModelInput = {
  root:{model:S,node:S},
  models:{S:{model_hash:H,parameters:{S:ClosedExpr|null}}},
  shared:[Shared,...]
}
```

The root MUST equal the term's selector model/node. The models map lists
exactly the transitively referenced models, their model_hash and every
parameter once, in its declared unit. Parameter resolution, domain checks,
operational freezing, invoke bindings and Shared use exactly the report
rules below. This whole entry, including root, parameter expressions and
reservations, is copied/validated by the request and serialized in
schedule.bindings; it participates in schedule_hash. ModelInput supplies
only classical model data and never coherent input values. A missing
required entry gives an explicit incomplete-model-objective diagnostic,
with no completed concrete schedule or optimization verdict. A present
well-formed input with free dependent parameters retains a symbolic
objective. A malformed root/model/hash/selector/parameter/reservation is
an explicit validation error, not an arbitrary choice of another report.
Only a complete concrete input can support a completed concrete objective.

A report used to certify an objective MUST bind the same device subject
and context and reproduce that entry's root, models and shared values
byte-for-byte after JCS. Other analytical reports may coexist and cannot
retarget the objective. Changing only report outputs cannot change the
schedule-bound objective input. Different legal shared reservations or
algorithm bindings create different schedule identities even under one
target_hash. A template with unresolved operational bindings still needs
a newly resolved target before concrete scheduling, as specified below.

Add this symbol in the new sched 0.10 ABI; JCS bytes are copied and
validated, with the ordinary §11 ownership/status convention:

```c
bennett_status bennett_sched_request_set_model_inputs(
    bennett_sched_request_t*, const uint8_t* model_inputs_jcs, size_t len);
```

This extends an explicitly versioned interface, not an existing C layout.
The target selector identifies what to measure; ModelInput identifies the
exact inputs used to measure it. No model-input hash is supplied in place
of these bytes.

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
be overridden under an unchanged target_hash. Algorithm values outside
the operational dependency set may be analytically rebound as below;
Parameter.role never overrides operational dependency membership.

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

Define the operational dependency set as the model-qualified parameters
transitively referenced by any operational Binding.quantity, following
the complete quantity-reference DAG. Compute it syntactically before
arithmetic simplification, including dependencies below zero factors;
external expressions retain unresolved dependency/translation obligations.
Membership is independent of Parameter.role. Every member MUST have a
valid nonnull target value before a concrete schedule is produced. Reports,
ModelInputs and cross-model invokes MUST preserve these resolved target
values; a supplied value for a member is an equality assertion checked
exactly with dimensions/units, never a replacement. A changed value
requires a newly resolved target, recomputed Binding.value and target_hash.
An unsupported external dependency analysis cannot claim operational
resolution checked. Non-operational hardware/code_distance/other values
also retain target values; only non-operational algorithm parameters may
replace defaults/null or receive analytical invoke arguments.

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
the referenced model's algorithm parameters, with matching dimensions
and conversion to the callee's declared units. They must satisfy the
callee's types/domains; free caller parameters remain symbolic. Only
non-operational algorithm parameters may acquire new values. An explicit
argument for an operational algorithm parameter MUST equal its resolved
target value; it is an equality assertion. An unequal or unresolved
replacement is a validation error. Other
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
  subject:{closure_hash:H,variant:S,inherited_controls:N,
           invocation:Invocation,context?:ContextKey},
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
reached context. variant is a nonempty producer string as in §12.3;
inherited_controls is a nonnegative integer encoded by §9.1. Standalone
reports explicitly bind closure_hash, variant, inherited_controls and
all invocation arguments/budget. Scheduled reports MUST match all those
values resolved at the selected actual context; none is inferred from
another report or from record order. If the alias-binding repair is
adopted, Invocation also carries its token_aliases partition: oracle
prepends its actual invocation token, action/perm_action/CC/R uses token
parameters only (§4.7), with no invented primary action identity. A scheduled report MUST carry the actual context key and
root schedule binding through its containing certificate. Repeated
contexts cannot share one result unless independently shown equal.
The models map contains exactly the transitively referenced models and
their canonical model_hash values. Its parameters map lists every parameter
exactly once in its declared unit. Hardware/code-distance/other parameters and every member of the
operational dependency set MUST equal target values. Only algorithm
parameters outside that set may replace null or algorithm defaults in
this report or ModelInput. A parameter's replacement also applies
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
or fault-rate obligations. An invocation's selected QEC distance, hardware binding and every other
operational dependency affect target_hash; analytical non-operational
algorithm inputs and hierarchy reports affect certificate bytes, and
objective ModelInputs additionally affect schedule_hash. Physical failure charges and all
physical-model assumptions remain separate from epsilon and correctness
claims. No model output changes the object's instruction semantics.

## Fat-object applicability

Incorporate [the shared scoping statement](fat-object-scope.md#citable-scoping-statement).
Append to §12.3, in the new version:

```text
In a mixed fat object, model_estimates attach to the certificate of the
identified embedded/linked device ISA object and selected device-root
closure. The containing certificate's object_hash and closure_hash name
that device object and root, not the enclosing native binary. The fat
inventory associates each device object/root with its certificate and
target-bound reports; multiple roots retain independent contexts and
objective inputs. Native main, argv/envp and host-only libc remain native
and need no device resource report or certificate record. Native
comparison binds the corresponding materialised outlined-region/callee
closure and recorded configuration. This report asserts no whole-native-
program resource total. Budgets, handles, tapes and host memory follow
the device scope and explicit remaining launch obligations in the shared
statement; this extension implements no host shim.
```

Example: native main performs host-only work then calls marked module M
through a device CC wrapper. M's object/root certificate carries its
surface:algorithm report. Main and host-only libc acquire no model_estimates;
a second device root obtains its own context/input/certificate binding.
General host-memory-by-reference launch mapping remains explicit
bennett-4ht contract work, not an assumption supplied by this report.

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
distinct. The source and normalized model hashes are recomputed
by check_rev2.py; they are fixture identities, not published pins.

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
| Complete selector + complete ModelInput + concrete dependencies | Checked scalar under the ordinary objective rules. |
| Complete selector + well-formed ModelInput with free non-operational dependent parameters | Symbolic unresolved objective, with no invented value. |
| Missing required model_outputs selector | Validation error at objective.model_outputs.<resource>; not a symbolic input. |
| Missing required ModelInput | Explicit incomplete-model-objective diagnostic; no completed concrete optimization. |
| Root/model parameters/shared reservations changed in a certifying report | Reject if they differ from its schedule-bound ModelInput; no report-selected objective. |
| Algorithm parameter N operationally binds time, target N=1; report/invoke N=2 | Reject the changed operational value. A newly resolved N=2 target must have value 2 and a new hash. |
| Standalone X reports with 0 and 1 inherited controls | Explicit subjects differ; 1-second and 2-second estimates cannot be interchanged. |
| 90 ns one-slot shared factory, 50 versus 100 qubits, same selector | Legal inputs give peaks 50/100 and occupancy 9/2000000 versus 9/1000000 physical-qubit seconds, with distinct schedule hashes. |

The executable check_rev2.py covers the fixture's scalar arithmetic and
unit conversions; parameter/formula negative cases; forward reference and
zero-multiplied cycle; exact rational roots/powers and rational enclosure
inequalities; sequence/parallel/repeat/shared-accounting examples; hashes;
the seeded input-identity/freeze witnesses, versioned catalog hashes,
and document structure. Cross-model invokes, actual operational target
resolution, source adapter translation, generic algebraic evaluation,
context validation and the remaining consumer acceptance vectors remain **proposed**.
A small rational example checker is not an implementation of this format.

## Objective selector and input witness

Declare factory_peak as a model resource with physical_qubit dimension
and scale 1. Use the shared-factory variant specified above (its changed
nodes require its own recomputed model_hash), N=3, and this objective:

```json
{"weighted":{"factory_peak":1},"model_outputs":{"factory_peak":{"model":"surface","node":"algorithm","metric":"peak","pool":"[\"surface\",\"factory\"]"}}}
```

Its schedule.bindings.model_inputs.factory_peak contains root
`{model:"surface",node:"algorithm"}`, complete model/parameter bindings,
and the explicit shared reservation. With one slot over [0,90 ns),
reserving 50 or 100 physical qubits gives peak 50 or 100, and occupancy
9/2000000 or 9/1000000 physical-qubit seconds. Both inputs are legal and
have the same target/selector, but distinct schedule hashes. A report
for the 100-qubit reservation cannot certify the 50-qubit schedule input.
No external report selection supplies the reservation implicitly.

For the selector-validation vector, `{"weighted":{"factory_peak":1}}`
is malformed: objective.model_outputs.factory_peak is missing. The
complete selector above with a well-formed input retaining free N is
symbolic; bind N and compatible reservation endpoints to obtain a concrete
scalar. Missing ModelInput gives the separate incomplete-input diagnostic.
These are model/scheduling outcomes, never ISA fault codes. The test uses
a finite schematic schedule envelope to witness identity sensitivity;
publication still needs complete bound production fixtures.

## Consumer impact, release and what remains for 0.10

For 0.9 every fully bound invocation retains its §10.1 estimate of the
complete padded realisation, exact mandatory rows and checked certificate,
including unmodified SQLite at N=2^20. Materialise fully resolved operational
fields into ordinary scalar 0.9 targets, retaining original source, parameter
bindings and hierarchy as external provenance. Equivalent accounting need
not materialise every gate. estimate_nresources continues to enumerate only
scheduled count/time/occupancy/peak rows; new model-only resources use
model_estimates and add no scheduled scalar row. An unused model output
still remains visible in the model report. Preserve all promised algorithm scope. This is a
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

No reference catalog recipe body, source instruction syntax or published
0.9 envelope is edited by this proposal. Under the normal full-version 0.10 publication, affected
semantic envelopes and source hashes change because their version changes;
do not promise a global no-repin migration. Target hashes change for new
models/resolved distances; target-bound schedules and certificates must
be re-bound and checked. Algorithms' estimate parameters and hierarchy
change the report/certificate bytes, independently of source identity.
Normal full-version 0.10 publication MUST also republish the default ref
catalog with isa="0.10" and version="0.10", update its §11.1 envelope and
§17 item 7 default binding, and recompute dependent schedule/certificate
identities and pins. All 219 recipe bodies may remain unchanged. The
metadata-only in-memory transformation gives:

```text
ref@0.9:  5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197
ref@0.10: 83cc9c177bad054bd7af35dc1892645d869b9e4c0b9c968f35b8783ce2b54c42
```

These are versioned envelope hashes, not raw-file SHA-256. The 0.10
calculation uses the published JCS routine, changing exactly the catalog's
isa/version and envelope's isa/version while preserving recipes. Published
0.9 artifacts stay unchanged; this job writes no catalog and re-pins no
artifact. Unchanged recipes do not imply an unchanged versioned identity.
Non-operational objective ModelInputs change schedule_hash; every
model report remains bound to its specific device object/root in the
mixed-fat-object inventory, not a native-program aggregate.

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

## Verification and evidence boundary

Run this directory's read-only witness/document checker:

```sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER \
  TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp/sol-contract-rev2/tmp \
  timeout 120 python3 -I -B /home/tobias/Projects/bennett-workspace/bennett-isa-spec/docs/notes/2026-11-02-rev2/check_rev2.py
```

The command was executed for this revision: exit 0, all eight PASS result
lines. It checks exact d=3/d=5 formulas, source/model identities, sequence/parallel/
repeat/shared-factory arithmetic, operational-dependency freezing, explicit
subject fields, selector/input outcomes and catalog 0.9/0.10 hashes. It also
checks all eight documents, local links, council-item mappings and the other
notes' finite witnesses. It implements no vendor adapter, generic algebraic
library, schedule solver, operational target resolver, report checker or
mixed-launch shim. A full serialized Report with bound production hashes
remains a publication fixture obligation; H-shaped schematic subjects in
these finite checks are not accepted certificates.

The illustrative model is ASCII and string-valued, so its recorded hash
can be reproduced by the narrow fixture serializer. General model JCS uses
RFC 8785, including UTF-16 key ordering: U+10000 sorts before U+E000. Python's
default sort_keys order is insufficient; the checker tests that distinction.
The map's 1e-09 number is outside model_hash, which hashes only model.

```text
raw source: 426daeed30fce3fa03234faff9d8ea543005c4681917f126b8dbaca6508c9be5
model:      c69312a9d3fc53728a6554b44bae48a9fab7eaefbcca3651c75ad9a0128e022f
```

All claims are proposal-level. The repository gate scripts/pre-push invokes
git/worktree operations and setup outside owned paths, forbidden by this
runner, so no full repository/sibling gate is run or claimed. Beads fd0,
gyp and bennett-4ht remain open for publication and implementation; no
tracker mutation, spec/catalog edit or sibling-revision edit is made here.

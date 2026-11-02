# Checker native models — evaluated epsilon and phase-bearing unitaries

Full revised proposal for council review, 2026-11-02; checker request 1
in the existing cross-repo filing queue. Supersedes
`2026-11-02-contract-checker-native-model.md`; proposal only.

## Defect classification and North Star scope

ISA 0.9 §8.3 native entries supply opcode, classes, controls, parameters,
cost and epsilon, but no realised operator. §8.1's arithmetic expressions
are not operator definitions. §§4.4/12.1 require phase-sensitive realised
unitary/channel comparison; an advertised error bound cannot identify
which operation to reconstruct. This is an S0 contract gap for independent
native approximation verification. Assuming the ideal opcode for an
opaque approximate leaf is circular self-comparison.

North Star §§4.4/5.1 part 2 retains approximate algorithms and independent
phase/channel checks. Exact logical native gates still denote their
phase-fixed §4.4 operations. The repair supplies missing model data,
without changing accepted native-unitary semantics or mistaking a
physical stochastic failure rate for logical approximation epsilon.
Legacy unchecked reporting is migration behavior, not a replacement for
the mechanism or permission to omit approximate algorithms.

## Alternatives and concrete witnesses

| Choice | Consequence |
|---|---|
| Always substitute the ideal opcode | Does not reconstruct approximate realisation independently. |
| Trust advertised epsilon | An explicit target assumption only; cannot prove a tighter independent claim. |
| Phase-bearing hash-bound unitary model | Recommended; permits independent numerical and induced-channel checks. |
| Expand an explicit scheme into exact leaves | Already works when it describes the actual circuit; does not describe an opaque approximate native leaf. |

An X entry with advertised epsilon=2 and time=1 could realise X or −X.
Both meet the same bound/cost/class metadata, and their uncontrolled
channels are identical. Under one coherent control on |+>_q|0>_t, they
produce `(00+11)/sqrt(2)` and `(00−11)/sqrt(2)`, orthogonal states.
The phase relative to the disabled branch is observable. Without model
bytes the current target/hash cannot distinguish these implementations.
A zero-distance numerical claim against controlled X is true only for
X; a declared conservative epsilon=2 alone cannot establish it.

The evaluated-epsilon counterexample is exact X with
`epsilon={"op":"-","args":[1,1]}` and no model. §8.1 evaluates it to
zero exactly; §17 item 9's eps-independent exact-match convention applies.
It must remain exact and checkable. For `epsilon={"var":"c"}` on c=0..1,
c=0 is exact; c=1 needs a realised model for independent verification.
Expression syntax alone cannot determine that requirement.

The channel-admission counterexample has Kraus operators
`K0=X/sqrt(2), K1=ZX/sqrt(2)`. Their squared operators sum to identity,
but the channel maps |+><+| to I/2. A unitary preserves purity, so no
logical unitary represents this map. An explicit controlled dilation
does not make the reduced map unitary or supply an operator-norm epsilon
for it. In contrast, two Kraus operators both X/sqrt(2) represent Ad_X;
that representation is usable if it also binds phase-bearing U=X and
its actual controlled model.

## Recommended exact spec changes (§8.3)

Add `realisation_model?` with initial format
`bennett.native-model/1`. Extend the target schema with the following
rules; retain §§4.4/8.1 matching and §12.3's controlled numerical claims.

### Model requirements depend on evaluated epsilon

```text
Resolve the matched entry's k, c, requested eps and parameter bindings
using §8.1 before determining model requirements. At each resolved
context, if achieved epsilon evaluates exactly to zero, an omitted model
denotes precisely the §4.4 opcode, including global phase and the exact
controlled form. Positive evaluated epsilon requires a versioned
realisation_model to discharge independent realised-operation
verification. A model-complete target supplies one for every admitted
context where its epsilon can be positive. Expression syntax alone
neither requires nor exempts a model.

Any supplied model MUST agree with the exact §4.4 operation in every
zero-epsilon context and satisfy the advertised operator-norm bound
otherwise. Zero-valued expressions independent of eps retain §17 item 9's
exact matching convention. Existing legality rules for eps-dependent
expressions, log2inv_eps, eps_decade and target matching remain unchanged;
this does not make an otherwise illegal zero-epsilon expression legal.
```

### Native operation and error metric stay unitary

```text
Every native realisation model MUST identify a phase-bearing logical
unitary U for each resolved parameter/control context. Native epsilon
bounds ||U-V||op for the corresponding phase-fixed ideal §4.4 operation V.
Normalization and model identity alone do not establish accuracy, an
inverse, or phase-free permutation status.

The native model kind is unitary. Matrix, Kraus or instrument data may
be used as a representation only when their reduced logical map is
rho -> U rho U-dagger, and the phase-bearing U is supplied as part of
the same model. Kraus/instrument data alone are insufficient. Check
complete-positivity/trace preservation when applicable and equivalence
to the bound U on every matrix unit (or a checked symbolic proof of that
identity), as well as U-dagger U=I. Thus a CPTP map that mixes a pure
logical state is not a conformant native-unitary gate model under §4.4.

For every admitted total control count the model identifies the actual
controlled unitary, including phase relative to disabled branches.
Exact controlled-U construction is allowed when explicitly bound as
that actual implementation. Arbitrarily controlling a Kraus list does
not define it. Instrument/environment conventions, when present, must
reduce to the bound logical unitary in that control context.

Channel claims compare induced channels; they do not replace the
phase-bearing unitary, change epsilon to diamond error, or discharge
§12.3's required numerical claim for controlled unitary variants.
General logical measurement retains its existing CC/other published
mechanisms; stochastic hardware failures remain separate target-model
work. This repair makes no new nonunitary-native admission rule.
```

### Representation and independent reconstruction

```text
realisation_model has {version,kind,arity,basis,parameter_bindings,
control_models,data}. Its canonical target bytes enter target_hash.
After resolution, arity and basis determine the complete ordered logical
operand bits. Explicit control models give the full context matrix;
a controlled_U rule instead gives a base matrix and constructs identity
on disabled branches and that matrix on the all-enabled branch, retaining
its global phase. In that form arity/basis describe the base matrix and
control_models adds the resolved c control wires. All matrix dimensions
must match the resolved basis. No environment is silently counted as an
extra logical input or discarded from a claimed unitary interface.

parameter_bindings uses the same resolved k, c and eps as cost/error
matching and the exact angle binding (§3.1 and §11.1 K20). A finite
binary64 theta denotes exactly that binary rational, not a rounded
source decimal. An explicit operand-to-wire permutation maps the entry's
own-controls-then-effective-controls order (§8.3) to §12.1's allocation
basis, inherited controls first. Both data and control models use this
map; wrong parameter, order, dimensions or context bindings reject.

Complex matrix entries are [real,imaginary] exact expressions over
rational constants, i, pi, sqrt, exp, sin and cos with finite expression
trees; rational constants are [numerator,positive_denominator] canonical
decimal-string pairs in reduced form. Expression nodes are
{op,args}; variables name only the declared parameter/context bindings.
Data may additionally supply closed rational interval enclosures of an
identified exact operation, with independently checkable enclosure and
unitarity certificates. An unverified box alone is not an operation.

The checker independently constructs U from the exact model in each
invocation context, checks its normalization/control/accuracy obligations,
and applies §12.1's interval numerical/channel procedures to requested
claims. Inconclusive interval tests do not pass. The advertised epsilon
remains a separate matching/error obligation, never the operator's
definition. An absent approximate model never licenses ideal self-comparison.
```

For X, a concrete base model is below. Its rational/imaginary pairs are
exact; changing the two real off-diagonal numerators from "1" to "-1"
produces the −X model. `controlled_U` binds its actual controlled
implementation for every admitted c, not just equality of uncontrolled
channels. For non-parametric X, k=0, and eps is the requested share when
bound; a zero-epsilon context needs no eps-dependent model expression.

```json
{
  "version":"bennett.native-model/1",
  "kind":"unitary",
  "arity":1,
  "basis":["t"],
  "parameter_bindings":{"k":"k","c":"c"},
  "control_models":{"kind":"controlled_U","controls":"c","disabled":"identity"},
  "data":{"matrix":[
    [[["0","1"],["0","1"]],[["1","1"],["0","1"]]],
    [[["1","1"],["0","1"]],[["0","1"],["0","1"]]]
  ]}
}
```

Model-format version 1's default evaluation is exact expressions plus
§12.1 interval enclosure, without any dimensional cap. Larger matrices
may be generated symbolically from the same bindings rather than fully
materialized; finite host checking limits yield unchecked obligations,
not smaller supported algorithm domains.

## Migration, hashes and publication

Legacy targets without required models keep parse/execution compatibility.
At zero evaluated epsilon use the exact-opcode convention. At positive
epsilon, missing-model reporting leaves only the affected independent
numerical/channel obligations unchecked; it cannot yield a checked
executable certificate covering them. Unknown versions receive
unsupported-model reporting, never a guessed operation. A supplied model
that contradicts a zero-epsilon contract rejects that target claim.

Recommend an authorized 0.9 K repair with this versioned target extension.
Model bytes change target_hash and require regenerating every dependent
schedule_hash and certificate binding. Source/interface/closure/object
identities remain unchanged only under an unchanged 0.9 semantic envelope;
a full ISA-version publication requires rehashing those envelopes too.
The ref catalog is unchanged. Target authors, scheduler, simulator and
checker must agree on model resolution; an ideal-opcode simulator alone
is not native approximation verification. Existing request 1 remains open
for acceptance/implementation. No physical native checker was run here.

## Mixed fat object (bennett-4ht)

Add to §§9/10/12: "A mixed binary's native host runs main, argv/envp,
unmarked libc and ordinary code. Only marked modules are VM objects;
the launch manifest selects their exported entries. A native gate model
belongs to the target used for that VM closure and its reached native
leaves, not to the host machine instructions of the whole binary.
Certificate inventory and phase/control reconstruction cover that selected
closure's invocations. Each VM schedule pins the target/model hash even
when several modules share the target. Host qreg handles and angle values
are marshalled through the published launch bindings, not reinterpreted
as model basis indices. A native comparison uses the corresponding
materialized marked host entry and captured inputs; it does not require
every host instruction to have a quantum gate model."

Thus a marked module using controlled X distinguishes the X and −X models
while linked beside an ordinary host libc. No whole-binary VM translation
or certificate inventory is assumed.

## Round-1 blocking items and resolution

| Item | Change and concrete witness |
|---|---|
| sol B1 / astra B1: expression-valued zero both requires and omits model | Value-based, per-context requirement; 1−1 without a model is exact X. Legacy epsilon=c at c=0 is exact; c=1 is missing-model unchecked. |
| astra B2: general channel has no §4.4 unitary or controlled numerical claim | Keep native models unitary; a Kraus/instrument representation must identify phase-bearing U and prove reduced-map equivalence. X/√2, ZX/√2 mixes the plus state and fails that requirement; identical X/√2 Kraus operators with U=X can satisfy it. |

## Conformance vectors and exact verification

Keep X/−X under an outer control, all matrix units and external references;
exact X without a model, zero-valued epsilon AST, mixed-control epsilon,
missing approximate model, wrong dimensions/normalization/control or angle
bindings, target-model hash changes, malformed Kraus data, unitary-equivalent
Kraus data and rigorous inconclusive interval claims. Angle 0.1 binds
3602879701896397/36028797018963968, not 1/10.

Controlled X versus controlled −X has operator distance 2 and channel
diamond distance 2. §12.1's numerical procedure uses a Frobenius upper
bound 2√2, so its requested bound 2 rejects and bound 3 can pass at
sufficient interval precision. Controlled X versus itself can pass a
zero numerical bound. Equal uncontrolled channels do not make a zero-bound
channel SDP witness pass: retain §12.1's strict-positive-pivot rule and
strict-feasibility treatment. These distinguish mathematical distance
from the prescribed checker's acceptance procedure.

The script checks exact matrices, all uncontrolled matrix units, the
controlled phase witness, purity and rational angle/epsilon examples.
It neither evaluates the proposed model schema nor runs a numerical/SDP
checker; no new native-target implementation is claimed.

```python
from fractions import Fraction as F

I = [[1, 0], [0, 1]]
X = [[0, 1], [1, 0]]
Z = [[1, 0], [0, -1]]

def mul(a, b):
    return [[sum(a[i][k]*b[k][j] for k in range(len(b)))
             for j in range(len(b[0]))] for i in range(len(a))]

def tr(a):
    return [list(r) for r in zip(*a)]

def neg(a):
    return [[-x for x in row] for row in a]

def ad(u, rho):
    return mul(mul(u, rho), tr(u))  # All entries in this witness are real.

assert mul(tr(X), X) == I and mul(tr(neg(X)), neg(X)) == I
for i in range(2):
    for j in range(2):
        e = [[int((a, b) == (i, j)) for b in range(2)] for a in range(2)]
        assert ad(X, e) == ad(neg(X), e)

# Basis 00,01,10,11: inherited control first.
def controlled(u):
    return [[(int(i == j) if i < 2 and j < 2 else
              u[i-2][j-2] if i >= 2 and j >= 2 else 0)
             for j in range(4)] for i in range(4)]

cx, cmx = controlled(X), controlled(neg(X))
start = [[1], [0], [1], [0]]  # sqrt(2) times |+>|0>.
a, b = mul(cx, start), mul(cmx, start)
assert a == [[1], [0], [0], [1]] and b == [[1], [0], [0], [-1]]
assert sum(a[i][0]*b[i][0] for i in range(4)) == 0
assert F(a[0][0]*a[3][0], 2) == F(1, 2)
assert F(b[0][0]*b[3][0], 2) == F(-1, 2)
# Frobenius squared of V-dagger U - I is 8; operator norm is 2.
r = mul(tr(cx), cmx)
assert r == [[1,0,0,0],[0,1,0,0],[0,0,-1,0],[0,0,0,-1]]
frob2 = sum((r[i][j]-int(i == j))**2 for i in range(4) for j in range(4))
assert frob2 == 8 and 2**2 < frob2 < 3**2

# Kraus X/sqrt(2), ZX/sqrt(2): exact weighted-channel computation.
zx = mul(Z, X)
plus = [[F(1, 2)] * 2 for _ in range(2)]
one, two = ad(X, plus), ad(zx, plus)
mixed = [[(one[i][j]+two[i][j])/2 for j in range(2)] for i in range(2)]
assert mixed == [[F(1,2), 0], [0, F(1,2)]]
assert sum(mul(mixed, mixed)[i][i] for i in range(2)) == F(1, 2)
assert sum(mul(plus, plus)[i][i] for i in range(2)) == 1
assert [[F(mul(tr(X), X)[i][j]+mul(tr(zx), zx)[i][j], 2)
         for j in range(2)] for i in range(2)] == I
# Equivalent two-Kraus representation, with explicit U=X, on all units.
for i in range(2):
    for j in range(2):
        e = [[F(int((a,b) == (i,j))) for b in range(2)] for a in range(2)]
        u = ad(X, e)
        assert [[(u[a][b]+u[a][b])/2 for b in range(2)] for a in range(2)] == u
assert F(1)-F(1) == 0
assert [F(c) for c in range(2)] == [0, 1]
assert F.from_float(0.1) == F(3602879701896397, 36028797018963968)
assert F.from_float(0.1) != F(1, 10)
print('PASS native-model: exact zero epsilon; four matrix units; controlled phase; CPTP nonunitary/equivalent Kraus witnesses')
```

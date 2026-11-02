# Checker resources — complete contexts and exact-rational display rules

Full revised proposal for council review, 2026-11-02; checker request 4
and `b-isa-spec-cwq`. Supersedes
`2026-11-02-contract-checker-resource-contexts.md`; proposal only.

## Defect classification and North Star scope

ISA 0.9 §12.1 enumerates complete invocation contexts, including default
coverage. §§11/12.3 permit a region under different arguments, budgets,
controls and schedule choices. Error entries already use common bounds
and explicit multiplicities. In contrast, flat region.resources and
gates_used maps do not choose an invocation or define a max/sum/common
bound. §8.2 defines event accounting but does not select the event list
for those maps. K12 binds an estimate's resources; it cannot choose the
meaning of another context's flat report.

A second gap predates this note: §12.3's resources {name:decimal} schema
has no encoding/comparison rule for non-terminating rational costs when
estimate is absent. §8.1 permits division, §8.2 aggregates rationals, and
§10.1 currently defines upward decimal encoding in the estimate contract.
The repair applies that encoding to every resources map, including old
single-context maps, while preserving exact accounting.

These are S0 contract gaps. North Star §§4.3–4.4/5.1 requires the complete
resource vector and substitutable scheduler/checker interfaces. Repeated,
controlled, default-covered and differently budgeted invocations stay
supported. Ignoring contexts, silently picking one or rejecting valid
rational costs cannot meet that scope. Unchecked reporting describes
verification status; it does not replace the repair.

## Alternatives and worked witnesses

| Choice | Consequence |
|---|---|
| Sum every child map | Misdefines peaks/makespan and double-counts nested leaves. |
| Common per-resource maximum | Could be a separately specified bound assertion; not exact event accounting. |
| Pick one context implicitly | Depends on traversal order and is not independently replayable. |
| Exact full-context records | Recommended; replay each event list and derive the root separately. |

Take `@flip(t)` with body `x t`. A root holds mutable t and in-role c
and executes these canonical instructions:

```text
action_call @flip (t) () () ()
qpred c
action_call @flip (t) () () ()
qunpred
```

At source points entry:0 and entry:2, each call contributes semantic
call_index=0 and instance=[], with no loop iterations. A native X entry
supports total controls 0..1, unit time, unit occupancy rate and sufficient
capacity in class fast. There are no other timed leaves; predicate-stack
instructions supply structural control. Its views are:

| Report | X native leaves | Time | Qubit-time | Peak |
|---|---:|---:|---:|---:|
| flip, forward, inherited controls 0 | 1 | 1 | 1 | 1 |
| flip, forward, inherited controls 1 | 1 | 1 | 2 | 2 |
| root, both interface wires live throughout | 2 | 2 | 4 | 2 |

The second child occupies root time [1,2) but reports its invocation-local
time 1. A zero-valued control still occupies its wire and scheduled gate
slot; these costs do not depend on a quantum predicate's basis value.
Summing child peaks gives 3 rather than 2; adding root and children
counts the same leaves twice. The second child includes its inherited
control in its footprint without transferring ownership from the caller.

Now set native duration to the legal expression {"op":"/","args":[1,3]}.
Child times are exactly 1/3; root time is 2/3. Child occupancies are 1/3
and 2/3; root occupancy is 4/3, with unchanged peaks. No finite decimal
can equal any of these thirds. Without estimate, exact decimal-vs-rational
equality would reject every legal time; the rule below instead compares
canonical displays of exact recomputed values.

As a borrow check, one wire lives over [0,2), borrowed during [0,1),
with owner rate 1 and dirty rate 1/4. Combined occupancy is 1/4+1=5/4,
not 9/4: borrower charges replace the owner's during that interval.

## Recommended exact spec changes (§12.3)

Add to each region record:

```text
resource_contexts?:[{context_key,resources,gates_used,estimate?}]
```

Add these normative rules, and a §0.2 repair entry RESOURCE-CONTEXTS:

```text
For exactly one invocation context, resources and gates_used report its
§8.2 scheduled event accounting. resource_contexts may be omitted; if
present it MUST have exactly one matching record. The root context_key
is []. A standalone verified resource report binds one invocation and
an effective §11 schedule, including an explicit empty-entries schedule
when defaults supply all decisions. A schedule-free reference claim
retains K13's reference-only meaning and is not target-resource evidence.

For more than one invocation context, resource_contexts MUST contain
exactly one record for every §12.1 context, in increasing bytewise order
of the JCS encodings of their complete keys. Include default-covered and
predicated-off invocations. Full identity includes resolved source
point, loop indices, fixed-binding instance, forward semantic-call
occurrence and oracle step/phase where applicable, not just variant and
inherited-control count. Resolve each context's own args, token budgets,
placements, cleanup, epsilon shares, schedule/default decisions, native
leaves and wire intervals. Recompute every declared resource and gate
count from that context. gates_used counts native leaves after scheme
expansion, each once, with existing scheme/error attribution unchanged.

For compatibility, unkeyed resources/gates_used/estimate MUST be the
first canonical context's projection. An optional estimate's presence
and value agree with that projection. The projection discharges only
that context. Missing, duplicate, reordered, foreign or mismatched
context records, or a conflicting projection, reject the requested
report. A singleton array, when present, follows the same validation.
```

### Exact arithmetic and canonical decimal comparison, even without estimate

Apply this paragraph to EVERY resources map: single-context fields,
resource_contexts entries, root maps and unkeyed projections. It repairs
the pre-existing decimal gap rather than leaving it to a separate issue.

```text
Whether or not estimate is present, accounting uses exact nonnegative
rational arithmetic. Each resources value is a canonical decimal string
using §10.1's 34-significant-digit upward encoding. For q>0, find integer
e with 10^e <= q < 10^(e+1), then display
ceil(q / 10^(e-33)) * 10^(e-33). Zero is "0". Spelling has ASCII digits,
an optional decimal point, no exponent/plus sign, leading integer zeros
except "0", trailing fractional zeros or negative zero.

The checker compares the producer's canonical string with the canonical
encoding of its independently recomputed exact rational value, not the
numeric displayed decimal with that exact rational. Noncanonical strings
or differing encodings reject. Resource-row keys and class qualification
retain §8.2/§10.1's vector and escaping requirements; no scalar sum
replaces a required per-class row. Capacity, matching, objectives and
resource calculations continue to use exact rationals. Never add rounded
child displays to compute an exact parent value.

When estimate is present, precision_digits is 34 and its reduced
exact_resources fractions MUST also equal the recomputed values, with
nonnegative numerator and positive denominator decimal strings and zero
as 0/1. K12's existing mandatory-estimate cases and complete class rows
remain mandatory. Optional estimate here does not weaken K12. Omitting
estimate elsewhere does not mean a decimal must be an exact rational
representation; it is the defined upper display enclosure.
```

Examples: 1/3 displays as
`0.3333333333333333333333333333333334`; 2/3 as
`0.6666666666666666666666666666666667`; 4/3 as
`1.333333333333333333333333333333334`.
An estimate's exact time for the first case is
`{"numerator":"1","denominator":"3"}`. These same displays are correct
when estimate is absent. Two rounded 1/3 displays sum to a different last
digit from the canonical encoding of exact 2/3; the latter is required.

### Root totals, child footprint views and time origin

```text
The root counts its realised event list once, including semantic callee
leaves. Child reports are independently checkable subtree/footprint views,
not extra costs added to the root. Measure child timing relative to that
invocation's entry, preserving internal delays in the effective schedule.
Restrict caller-owned interface and inherited-control lifetimes to that
invocation interval when displaying the child view. This neither
transfers ownership nor creates an additional charge to the root.
Within each view apply §8.2 dirty-borrow replacement: the borrower replaces
the owner's charge for that wire during the borrowed interval. There is
no second aggregation by summing child peak/time maps. Error-bound
multiplicities retain their existing separate composition rule.
```

## Concrete context/report fragment

Use a synthetic root closure hash of 64 `a` characters solely to display
context identity; it is not asserted as an actual object hash. The two
full child keys are:

```json
[
  [{"symbol":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","point":"entry:0","iteration":[],"instance":[],"call_index":0}],
  [{"symbol":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","point":"entry:2","iteration":[],"instance":[],"call_index":0}]
]
```

They sort in the displayed order. For duration 1, the corresponding
resource_contexts entries use these keys and respectively:

```json
[
  {"resources":{"peak_qubits[fast]":"1","qubit_second[fast]":"1","time":"1"},"gates_used":{"x":{"count":1}}},
  {"resources":{"peak_qubits[fast]":"2","qubit_second[fast]":"2","time":"1"},"gates_used":{"x":{"count":1}}}
]
```

The child's unkeyed projection is the first record. Root unkeyed fields
are `resources={"peak_qubits[fast]":"2","qubit_second[fast]":"4","time":"2"}`
and `gates_used={"x":{"count":2}}`; its own context key is []. These are
schema fragments, not a complete bound certificate. Actual source,
target, schedule hashes, claims and frames must be supplied before
checker execution; fabricated hashes do not verify an executable.

## Mixed fat object (bennett-4ht)

Add to §§9/10/12: "A native host binary can embed VM objects only for
marked modules. A launch manifest selects an exported VM entry and its
schedule/target; that selected VM invocation is the resource-report root,
with context_key=[]. Its certificate inventory covers its VM closure and
all generated invocation contexts, not every native entry or libc routine.
Unmarked host main, argv/envp and ordinary host work remain native. A host
shim supplies the bound handles, args, images and budgets before launch.
Only costs within the bound VM realisation, including effects/transfers
actually represented by that contract, enter its vector. Separately
reported host/launch overhead must have an explicit scope and is not
silently claimed as included. Native comparison selects the corresponding
materialized marked entry; resource reporting does not require a whole
native-binary schedule or VM certificate."

For the two-flip witness, the host calls a marked root in one module;
its report still includes both child contexts even if the surrounding
binary contains unrelated native modules. A second marked launch has its
own root binding and certificate; reports are not merged by region name.

## Round-1 blocking items and resolution

| Item | Change and concrete witness |
|---|---|
| astra RC-1: no non-terminating rational encoding without estimate | Apply §10.1 upward decimal encoding and recomputed-encoding comparison to every resources map; optional exact_resources must equal the rational too. Native duration 1/3 gives valid child/root thirds both with and without estimate. |

All accepted context/projection, root aggregation, default coverage and
error-composition rules remain. Round-1 footprint, time-origin,
singleton and borrow clarifications are made explicit above.

## Hashes, publication and consumer work

Recommend an authorized 0.9 K certificate repair alongside the distinct
fault-rate binding repair. New context/encoding bytes change certificate
and report digests. With an unchanged semantic envelope, §9.2 excludes
certificate bytes from object_hash: source/interface/closure/object,
target, schedule and ref catalog bytes/identities need not change.
A full-version publication instead regenerates version-envelope identities
and dependent pins. The note performs neither publication nor migration.

Scheduler, estimator, plugin and checker retain full keys, canonical
ordering and K12's class-qualified keys. Old repeated-context flat reports
need recomputation, never a guessed max/sum. Existing exact integer
reports can be migrated to canonical strings; non-terminating costs use
the same explicit rule regardless of estimate presence. The existing
request/cwq implementation obligation remains open; accepted source
semantics and the full demo ladder are unchanged.

## Conformance vectors and verification

Require integer and duration-1/3 cases, estimates present/absent, singleton
arrays, root [], unused target classes, borrowed intervals, repeated
semantic occurrences, lproc fixed instances, default coverage and nested
token budgets differing under one (variant,controls) pair. Swap/omit/
duplicate/relabel a context or mismatch its estimate/projection: reject.
Reject truncated thirds, noncanonical displays, wrong exact fractions,
rounded-display summation and duplicate leaf charging. Root and child
resource charges are independent of control basis values.

The script independently checks rational accounting, decimal encodings
and concrete context identities. It does not execute a new scheduler or
certificate checker, and the fragments above are not claimed as an
accepted fixture.

```python
from fractions import Fraction as F
import json

def pow10(e):
    return F(10**e) if e >= 0 else F(1, 10**(-e))

def display(q):
    q = F(q)
    assert q >= 0
    if not q:
        return '0'
    e = 0
    while q < pow10(e):
        e -= 1
    while q >= pow10(e+1):
        e += 1
    unit = pow10(e-33)
    z = q/unit
    ceiling = (z.numerator + z.denominator-1)//z.denominator
    rounded = ceiling*unit
    k = max(0, 33-e)
    n = rounded*10**k
    assert n.denominator == 1 and rounded >= q
    s = str(n.numerator)
    if k:
        s = s.zfill(k+1)
        s = (s[:-k] + '.' + s[-k:]).rstrip('0').rstrip('.')
    return s

expected = {
    F(0): '0', F(2): '2', F(1,8): '0.125',
    F(1,3): '0.3333333333333333333333333333333334',
    F(2,3): '0.6666666666666666666666666666666667',
    F(4,3): '1.333333333333333333333333333333334',
    F(1,30): '0.03333333333333333333333333333333334',
    F(10,3): '3.333333333333333333333333333333334'}
for q, encoded in expected.items():
    assert display(q) == encoded
assert F(display(F(1,3))) != F(1,3)
assert 2*F(display(F(1,3))) != F(display(F(2,3)))
for d in (F(1), F(1,3)):
    # (time, occupancy, peak), wire live intervals as declared above.
    children = [(d, d, 1), (d, 2*d, 2)]
    root = (2*d, 4*d, 2)
    assert sum(c[0] for c in children) == root[0]
    assert sum(c[2] for c in children) != root[2]
    assert root[1] == 2*root[0]
    if d == F(1,3):
        assert [display(v) for v in root] == [expected[F(2,3)], expected[F(4,3)], '2']
assert F(1,4)*1 + 1*1 == F(5,4)
assert F(5,4) != F(9,4)
keys = [[{'symbol':'a'*64, 'point':f'entry:{p}', 'iteration':[],
          'instance':[], 'call_index':0}] for p in (0, 2)]
jcs = lambda v: json.dumps(v, sort_keys=True, separators=(',', ':'))
assert keys == sorted(keys, key=lambda k: jcs(k).encode())
assert jcs(keys[0]) != jcs(keys[1])
print('PASS resources: integer/thirds context and root costs; eight canonical displays; borrow replacement; full keys')
```

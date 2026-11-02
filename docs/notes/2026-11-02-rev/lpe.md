# lpe — classical width families with explicit call shapes

Full revised proposal for council review, 2026-11-02; `b-isa-spec-lpe`.
Supersedes the proposal in `2026-11-02-contract-lpe.md`; no spec edits or
council acceptance are implied.

## Defect classification and North Star scope

ISA 0.9 §3.3 provides qreg[*], qreg.width and checked transfer into fixed
slots; §§5.1/9.1 give region slots literal widths. Dialect 0.3 draft
§8.4 can construct a fixed-shape oracle for each valid width/extent and
CC-dispatch between them. Its §24 D34 prohibits interpreting emission
order as a runtime validity restriction; §25.3 G-17 requests a compact
representation. This remains an efficiency/interface extension, not a
proved lack of denotational expressiveness in 0.9.

A dispatch table of up to 2^64−1 positive runtime widths is not a practical
completed build. A finite emitted prefix cannot certify an unrestricted
program. North Star §§0/3.3/4.2 retains arbitrary defined, bounded
programs and classical runtime budgets. Missing specializations cannot
become T-IFACE, an invalid-width fault or a required source rewrite.
Literal-width 0.9 programs and their unlimited mathematical wire/region
widths remain available; this compact runtime family mechanism adds no
new cap to them.

## Alternatives and worked witnesses

| Construction | Consequence |
|---|---|
| Complete static dispatch | Existing 0.9 semantics; an emitted prefix remains unfinished. |
| Finite load-time instance list | Suitable when the source program itself has that finite domain; cannot restrict an unrestricted program. |
| Symbolic family, instantiated classically on demand | Recommended; one linkable family, with a fixed shape during each coherent invocation. |

The memoryless family `xor_width[n]` has input/target widths n, fault
width 4 and body `lift.retval x`. At B≥1 it maps
`(x,t,f) -> (x,t XOR x,f)`; at B=0 it maps
`(x,t,f) -> (x,t,f XOR 1)`. A false outer control is identity. Thus n=1
maps (1,0,0) to (1,1,0), and n=3 maps (5,3,0) to (5,6,0). The target,
fault and control values are quantified, not initialized conveniently.

The ledger width of an eight-wire register does not choose its memory
shape. `width_args=["1","4"]` for a memory family with parameters [A,W]
selects region[1,4]; `["2","2"]` selects region[2,2]. Both occupy eight
wires. A legal qstore XORing integer 1 into cell 1 of the zero image
produces flattened integer 16 in the first shape and 4 in the second.
The address/value operands have the corresponding resolved widths in
each access. This packing witness does not infer either shape from 8.
For six wires, [0,6] and [1,3] similarly remain distinct. A=0 is legal
for a region and retains K79's empty address actual `()`, never wire[0].

Finally, one fixed root action can call xor_width[1] and xor_width[3]
on disjoint operands with two explicit token arguments, then return.
The child signatures must have widths [1,1,4] and [3,3,4] in the same
certificate, with one forward call occurrence apiece. Their common
family closure hash cannot be the sole certificate record key.

## Recommended exact spec changes

Publish as ISA/object/launch/sched/cert/coverage 0.10; amend §§3.3, 4.7,
5.1, 9.1–9.4, 10–12 and 16.2 together. The following is normative
replacement/addition text for that release.

### Family signature and substitution (§§5.1, 9.1, 16.2)

```text
A region header MAY declare [n,...] immediately after its symbol name.
Parameters are distinct unsigned-i64 classical shape values, in a
separate width namespace. signature.width_params lists, in header order,
{name,bind:{group,slot,axis?}}. A wire binding names a wire slot; an extent
binding names a mem slot and axis "A" or "W". A symbolic size is
{param:name}, or the existing literal integer. Parameter occurrences are
permitted in wire[N], reg[N], region[A,W], size-bearing opcode suffixes,
allocation dimensions, and loop/bounds counts. Substitution is
capture-free and precedes all ordinary type, range and opcode checks;
all other integer positions retain their existing literal rules. This
adds no implicit arithmetic, inferred formula or quantum-dependent size.

A signature admits exactly shape values satisfying its declared size
constraints: wire/cell widths are positive, region A may be zero, and
existing per-opcode constraints still apply after substitution. Only the
signature determines the admitted shape set; body validation cannot
silently remove a shape from it. The producer must supply a
well-formed substituted body for every shape the signature admits.
An ill-formed body at an admitted shape is an invalid family, not an
invalid caller width. The resolved denotation is exactly the literal
substitution, including full-domain, control, adjoint, prefix-reversal,
borrow-frame and local-restoration obligations.
```

### Explicit call-shape channel (§§3.3, 4.7, 10, 16.2)

```text
Every family call has a separate ordered width_args vector, not part of
ordinary args. Text syntax puts [s0,...] immediately after @callee in
all region-call forms, including lifted calls and both qinplace callees;
a family header uses the same brackets for parameter declarations.
At CC/R entry, si is a literal unsigned-i64 or a classical i64 value read
as unsigned. Every parameter has exactly one supplied value. Wire-bound
values MUST equal the actual's ledger width. Memory-axis values are
supplied explicitly: substitute them into the callee mem signature and
require width(q)=2^A*W for every qreg memory actual. Compute this equality
in mathematical integers without wrapping. No A or W is inferred from
total register width, and no pre-existing caller descriptor is presumed.

Check all values, role constraints, disjointness, ledger states and
resolved shapes before any effect, including consuming a view. A bad
CC/R binding is CC trap / R fault FAULT_UB_TRAP under the existing rules.
In coherent callers, shape arguments are literals or already resolved
classical shape parameters, never coherent data. Their memory actuals
are declared interface/local regions: the substituted A and W must each
equal the caller region's resolved A and W, not just its total size.
Adjoint, matched uncall and R history retain the same vector. Nested
body/template/argument forwarding preserves each callee's vector just
as it preserves the invocation-token context.

A host launch, scheduling request or estimate for a family supplies
width_args explicitly, independently of params_jcs/ordinary args and
without needing live register ids to estimate it. Launch and estimator
validation use E_ARG for invalid request bindings before ownership or
execution; nested CC/R calls use the trap/fault rule above.
```

The launch 0.10 request structure gains `width_args_jcs` and its byte
length: a copied JCS array of the canonical decimal strings described
below. The family entry and its ordinary args/token/register slots are
selected explicitly by that request. Existing non-family entry requests
use an empty vector. This also provides a concrete transport for A/W at
the native-host boundary.

### Identity, expansion and schedules (§§9.2, 10.1, 11–11.1)

```text
interface_hash covers the symbolic signature, parameter order and bind
sites. closure_hash identifies the symbolic source and dependencies.
width_args is the header-ordered array of canonical unsigned decimal
strings, no leading zeros except "0". Define:
instance_hash = SHA-256(UTF-8("width-instance\n" +
  lowercase-64-hex-family-closure-hash + "\n" + JCS(width_args) + "\n"))
Here \n denotes one LF byte, and the family hash is ASCII hexadecimal,
not raw digest bytes. No trailing bytes follow the last LF.

Concrete executables, schedules, estimates, certificate subjects and
invocation contexts carry closure_hash, width_args and instance_hash.
Consumers recompute and validate instance_hash before expansion/replay.
Binding equality adds these fields to all existing budget, params_hash,
variant, controls, accuracy, target and expansion bindings; params_hash
continues to hash only ordinary args. A callee context retains its own
instance, not the root's instance. Captured templates and matched
forward/reverse occurrences retain that instance as well.

The expansion default for resolved 0.10 instances is the unchanged
ref@0.9 library, expansion hash
5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197,
unless a different library is explicitly bound. Its descriptors receive
literal substituted sizes. A cache key includes instance_hash, target,
expansion identity and every existing schedule binding. Imports compare
symbolic signatures including parameter order/bind sites.

A supported family implements every admitted shape. Caching, lazy
materialisation and finite warm-up lists are implementation choices;
a miss is not invalid source input. Executor resource exhaustion follows
the existing executor-resource contract (§17 item 16), not a new source
fault. A finite instance inventory is not a claim of family completeness.
```

### Concrete certificate records and frames (§§9.4, 12.1–12.3)

```text
regions is keyed by closure_hash for non-family regions and by
instance_hash for concrete family instances. A family instance record
carries closure_hash, instance_hash and width_args; each has its own
substituted signature, bounds, domains, required frame and verification
records. There is no abstract family record standing in for these
concrete records. Width substitution precedes §9.4 frame construction;
slot widths are resolved literal widths, with c remaining the only
frame-expression variable as before.

Family claim subjects and g/ginv inverse-pair references explicitly
identify closure_hash, width_args and instance_hash. An expected.program
or library-claim reference to a family likewise uses an explicit
{closure_hash,width_args,instance_hash} reference instead of a bare family
hash, with resolved signature compatibility checked. Composition keys
use instance_hash in the hash position for a family, including error
multiplicities. A concrete claim or inverse proof for one instance
cannot discharge another. A width-uniform structural claim may cover
several records only with explicit shape quantification and checked side
conditions over its entire admitted domain; each use records its shape
substitution. The default is separate concrete claims. Region-context
reports distinguish full callee instance identity in addition to variant,
controls, parameters, budget and semantic call occurrence.
```

For the mixed-width root, `regions[H1]` has
`closure_hash=F,width_args=["1"],instance_hash=H1` and input/target/fault
widths [1,1,4]; `regions[H3]` has the corresponding ["3"] and [3,3,4].
Their required slot frames have those widths, with separately derived
scratch/history descriptors at each invocation. Composition has one
`H1:forward:0` and one `H3:forward:0`. F is the common actual family hash.
H1/H3 below use a synthetic all-zero F solely to test hash bytes.

Append to dialect §8.4's 0.10 map: "A classical shape not finitely
enumerated by compiler value analysis lowers to this width family.
Shape-width lists affect instantiation order only." Formula-based layout
beyond direct substitution requires an explicit expression contract;
existing literal specialization remains a valid construction meanwhile.

## Mixed fat object (bennett-4ht)

Add to §§9/10: "The native host binary may contain only marked modules
as embedded VM objects. A host dispatch manifest selects a VM module,
exported family symbol and launch 0.10 request; its classical shim supplies
width_args from host sizes and qreg handles before coherent entry.
Certificate inventory covers the selected VM closure and all its reached
instances, not every host function. Ordinary main, argv/envp handling,
unmarked libc and unrelated host code remain native. A marked module's
required dependencies are linked into its VM closure through published
imports; this does not require translating the whole binary. A native
comparison calls the corresponding materialized host entry with the same
shape, args, image and budget conventions, rather than comparing an
unrelated whole-program entry. Shape identity is independent of native
addresses and process-local qreg/token ids."

## Round-1 blocking items and resolution

| Item | Change and concrete witness |
|---|---|
| sol LPE-1 / astra B1: CC/R has no descriptor for A/W | Separate explicit call/launch width_args; pre-effect axis/width checks. Eight wires with [1,4] versus [2,2] produce flattened cell-1 values 16 versus 4. |
| sol LPE-2 / astra B2: one closure record cannot hold several instances | Instance-keyed concrete records, resolved frames, subjects, inverse references and composition; preserved nested shape captures. One root contains [1,1,4] and [3,3,4] child records. |

## Hashes, release and consumer work

This is a 0.10 extension, not an incompatible 0.9 K repair. Every symbol
re-emitted with isa=0.10, including non-family symbols, receives its new
§9.2 version-envelope identity. New interchange versions carry the shape
channel and instance records; target 0.9 is reused when otherwise
unchanged. Published 0.9 object/catalog bytes remain frozen. The catalog
default above explicitly avoids inventing an unprovided ref@0.10.
VM, linker, scheduler, estimator, plugin and checker must publish matching
0.10 grammar/ABI/schema vectors before adoption. G-17/lpe implementation
work remains open; these proposals do not certify complete emission.

## Conformance vectors and verification

Besides the witnesses above: mismatched operand widths or false
instance hashes fail before effects/replay; n=0 fails a positive wire
signature. Different n invalidates a cached schedule; distinct qreg ids
with equal shapes share instance identity. Test imports with swapped bind
sites, nested forwarding, control/adjoint, A=0 empty address, a valid
uncached shape, and uniform proofs whose side conditions fail at another
shape. Preserve arbitrary target/fault/dirty systems and full-D checks.

The following independently executable script checks finite mathematical
witnesses, not an implementation of the proposed 0.10 certificate schema.
Run it by extracting this note's Python block with `python3 -I -B -`.

```python
import hashlib
import itertools
import json

family = '0' * 64

def instance(values):
    a = [str(v) for v in values]
    data = 'width-instance\n' + family + '\n' + json.dumps(a, separators=(',', ':')) + '\n'
    return hashlib.sha256(data.encode()).hexdigest()

h1, h3 = instance([1]), instance([3])
assert h1 == '3486e3dc5d4e3a80a35d4338f5ff126b787ded89baaee9f66aca083e7025cf6e'
assert h3 == '9ae5c34acce5452f6c24f31e97baa9ddfc027a44c609a5e77f4eb54759feacd3'
assert h1 != h3
records = {h1: (family, ['1'], [1, 1, 4]), h3: (family, ['3'], [3, 3, 4])}
assert len(records) == 2 and records[h1][2] != records[h3][2]
assert (1 ^ 0, 5 ^ 3) == (1, 6)
assert 2**1 * 4 == 2**2 * 2 == 8
assert (1 << (1*4), 1 << (1*2)) == (16, 4)
assert 2**0 * 6 == 2**1 * 3 == 6
cases = 0
for n in (1, 3):
    for x, t, f, c, b in itertools.product(range(2**n), range(2**n), range(16), range(2), range(2)):
        payload, code = (x, 0) if b else (0, 1)
        after = (x, t ^ payload, f ^ code) if c else (x, t, f)
        back = (after[0], after[1] ^ payload, after[2] ^ code) if c else after
        assert back == (x, t, f)
        if not c:
            assert after == (x, t, f)
        cases += 1
assert cases == 4352
print('PASS lpe: 4352 width/control/budget tuples; two instance keys; explicit memory shapes')
```

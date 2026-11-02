# lpe — region signatures parameterised by classical width

Proposal for council review; `b-isa-spec-lpe`.

## Defect classification and scope

ISA 0.9 §3.3 accepts qreg[*], qreg.width and checked transfers to fixed
slots; §§5.1 and 9.1 give region slots literal widths. Dialect 0.3 draft
§8.4 constructs one fixed-shape oracle for every valid width/extent, with
CC dispatch, and §24 D34 expressly forbids treating an emission-order
hint as a run-time validity restriction. §25.3 G-17 requests a more
compact representation. This is an **efficiency/interface extension**,
not a proved absence of denotational expressiveness in ISA 0.9.

There is nevertheless a real delivery limitation: as many as 2^64−1
positive run-time widths cannot practically be emitted as separate
objects. An unfinished build is not a completed universal implementation.
North Star §§0, 3.3 and 4.2 retain arbitrary defined bounded programs and
classical run-time budgets; it would narrow scope to replace missing
shapes by T-IFACE, reject a valid run-time width, or require users to
rewrite the program. Finite dispatch is a compatibility construction,
not evidence that a plugin has emitted the full domain.

## Alternatives and worked witness

Consider a lifted oracle `xor_width(x,n)` with n input bits, n target
bits and four fault bits, computing the identity payload. At width n,
it maps `(x,t,f)` to `(x,t XOR x,f)` under a sufficient token. For n=1,
`(1,0,0)->(1,1,0)`; for n=3, `(5,3,0)->(5,6,0)`. A qreg[*] of width 3
cannot fill a literal wire[1] slot: the existing width check faults
before mutation. A real width-3 specialization succeeds. Controls,
arbitrary target/fault values and local workspace stay quantified.

| Alternative | Tradeoff |
|---|---|
| Static dispatch over the complete valid shape set | Existing 0.9 semantics; huge representation, and a finite prefix is unfinished work. |
| Load-time finite instantiation list | Useful when the input program itself has that finite domain; cannot make a user-supplied list the valid domain of an unrestricted program. |
| Symbolic family with on-demand classical instantiation (recommended) | One linkable family and a specialization keyed by resolved width; widths fixed before coherent entry. |

## Recommended exact spec diff

For ISA/object 0.10, extend §§5.1, 9.1, 9.2, 11 and 16.2:

```text
A region header MAY have [n,...] immediately after its symbol name.
These are distinct unsigned-i64 classical parameters. Wire widths and
cell widths are positive; address exponents may be zero. A
signature adds width_params:[{name,bind:{group,slot}}], in header order.
Each bind names a wire operand whose run-time qreg.width binds that
parameter at the CC/R region call. Other slots may have either a literal
width or {param:name}; all occurrences of a parameter must resolve to
the same width before any effect. A coherent caller forwards a width
already bound at its own entry; no superposed value binds a parameter.
Source wire[n] refers to that parameter. Loop/allocation bounds may use
it where a literal integer was required, becoming literal in the resolved
instance. Other arithmetic on widths is not implicit. Extent parameters
for region slots use distinct typed bindings {group:"mem",slot,axis:"A"|"W"}
from the caller's resolved region descriptor, never by guessing an
exponent from a byte count. All existing disjointness, width, budget,
local-region restoration and cleanup conditions apply after substitution.

The interface hash covers the symbolic signature, including parameter
order and bind sites. The closure hash covers the symbolic source and
dependencies. A concrete instance has instance_hash = SHA-256 of UTF-8
"width-instance\n" followed by the family closure hash, LF, the JCS array
of resolved parameter values as unsigned decimal strings, and LF.
Executables, schedules, estimates and certificates for a family add
width_args and instance_hash to their bindings. Existing closure_hash
continues to identify the family. Binding equality includes these new
fields in addition to budget, params_hash, variant and inherited controls.
Widths are not inserted silently into the ordinary classical args array.

Instantiation substitutes every width parameter and checks the ordinary
region rules at that shape. A consumer may cache or lazily materialise
the resulting expansion; cache keys include instance_hash, target_hash,
expansion identity and all existing schedule bindings. A supported family
must implement every shape its signature admits, without treating a
cache miss or a finite warm-up list as an invalid width. Resource
exhaustion is reported under the existing executor resource contract;
it is not an invented source fault for a supported shape. Imports match
the symbolic signature and parameter binding order.
```

Append to dialect §8.4 for its ISA-0.10 map:

```text
Where a shape is classical but not statically finite, lower to the width
family above. Shape-width lists control instantiation order only. The
0.9 map and its full-domain obligations remain published separately.
```

This is a proposed compact family contract, not a runtime-dependent
quantum register size. Body/data-layout formulas beyond direct parameter
substitution require a further explicit expression grammar; no implicit
formula inference is proposed.

## Hashes, catalog and consumers

Symbolic signatures/bodies are new object bytes and have new envelope
hashes under 0.10. Old 0.9 objects and ref catalog stay frozen. Family
instances use the explicit instance hash above; n=1 and n=3 must have
different bindings even when ordinary args are empty. Catalog descriptors
may continue to receive resolved integer widths. VM, linker, scheduler,
plugin, launch estimator and checker must agree on the family/instance
distinction and specialization support; no consumer can ignore width_args.

## Conformance vectors and verification

Use n=1 and n=3 with all input/target/fault values, coherent outer control,
adjoint, and a caller forwarding n. Mismatched operand widths fail before
any effect; width 0 is outside a positive wire signature. Different n
must invalidate a cached schedule. Test equal values from distinct qreg
ids sharing an instance hash, imports with swapped bind sites, and a
previously uncached valid n. Exercise A=0 region addresses separately;
they remain ISA K79's empty address actual, not wire[0]. Hand values above
are checked by [contract-verification](2026-11-02-contract-verification.md).
No plugin completeness or consumer execution is claimed.

## Release and after acceptance

Recommend **0.10**, rather than a 0.9 repair to literal signatures. Keep
G-17/lpe as tracked implementation efficiency work; do not certify a
finite emitted prefix as complete. After acceptance publish grammar,
family identity and schedule vectors together before consumers adopt it.

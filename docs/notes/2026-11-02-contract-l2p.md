# l2p — temporary AND and measurement-assisted cleanup

Proposal for council review; bead `bennett-arith-l2p`. No interpreter,
catalog, target or normative spec is changed by this note.

## Defect and North Star gap

ISA 0.9 §6.1 marks a workspace wire `other` after H. §6.2 permits its
release by the exact computing adjoint only when no remaining value or
phase depends on it. Thus a clean-target AND built with H/T/CX, read into
a lasting result, and then reversed is rejected even when the whole
clean-target map is exactly phase-free. This is a conservative static
discipline, not evidence that the circuit is wrong. §4.4 correctly forbids
replacing arbitrary-target CCX by a relative-phase gate.

There is a separate, definite scope defect. North Star §4.2, paragraph
**Uncomputation validity**, explicitly admits measurement cleanup after a
dependency window closes, with the correction transported through the
actual intervening operation. §5 says the compiler chooses measurement
versus unitary cleanup; §5.1 part 2 specifies channel verification,
including external entanglement; §9 item 6 requires the controlled form.
ISA §6.2 instead calls closed-window measurement cleanup ill-formed;
§6.4 prohibits all measurement cleanup in sched/cert 0.9, forces `mbu:false`,
and §17 item 6 defaults to unitary cleanup. That default removes a
promised capability. Under rubric C3/C7 this is S0 even if unitary
arithmetic computes the same classical function at a higher cost. A
future-version label alone does not repair it. Existing bead l2p must
carry this ISA blocker until the mechanism is accepted and implemented.

## Primary-source check

Craig Gidney, [Halving the cost of quantum addition, Quantum 2, 74
(2018)](https://doi.org/10.22331/q-2018-06-18-74),
[arXiv:1709.06648v3, pp. 1–3, Figures 1–3](https://arxiv.org/pdf/1709.06648):
the temporary AND consumes four T resources to compute and uses an X-basis
measurement with a conditional Clifford correction to erase it. Figure 1
has 4n−4 T gates; the controlled adder is a different construction costing
8n+O(1). These costs do not include general coherent controls for free.
The input magic state in Figure 3 counts as a T resource. The paper
credits Jones for an equivalent construction. PDF text and captions were
read; the web screenshot facility failed, so the diagram's wire routing
was not visually inspected. The explicit circuit and algebra below were
derived independently, not transcribed from that diagram. The paper also
describes a six-net-T unitary alternative when recovered magic-state
resources are credited. The eight-T C/C† count below is the literal
T/Tdg count of the stated gate list, without such recovery credit; it is
not a unitary optimality claim.

## Worked circuit and alternatives

For clean g, use this ordered gate list (a,b read-only):

```text
g = qalloc any 1
h g
t g
cx a g
tdg g
cx b g
t g
cx a g
tdg g
h g
sdg g
cx g y
```

Call the ten-gate compute sequence C. It has four T/Tdg gates. Its exact
action for g=0 is:

| a | b | C on g=0 | amplitude |
|---|---|---|---|
| 0 | 0 | 0 | +1 |
| 0 | 1 | 0 | +1 |
| 1 | 0 | 0 | +1 |
| 1 | 1 | 1 | +1 |

Its g=1 action is respectively `−i|1>`, `+i|1>`, `−i|1>`,
`−i|0>`. Consequently C is **not** CCX on all eight basis states.
Its clean-input restriction is the isometry
`|a,b,0> -> |a,b,ab>`, including exact phase, so linearity establishes
it for coherent inputs and an external reference. Applying C† after
`cx g y` clears g and retains `y XOR ab`, at eight T gates for the
compute/cleanup pair. Treating C as a general native CCX would be unsound.

For measurement cleanup, let `D=diag(1,1,1,−1)` on (a,b). Measuring g
in the X basis gives branch operator `D^m/sqrt(2)` on the encoded
logical state. Apply CZ(a,b) iff m=1. Both corrected branches are
`I/sqrt(2)`, so summing them gives the identity channel on the data,
including every matrix unit and external entanglement. Reset the measured
workspace to zero before release. This cleanup uses zero T gates.
The complete compute/use/cleanup realizes CCX(a,b;y) with four T gates.

Closed-window witness: after `cx g y`, apply `h a`, then clean g. The
required correction is `H_a D^m H_a`, not D^m. The former cancels
the measurement phase after H; the latter fails because H and D do not
commute. All classical inputs and both outcome branches remain in scope.

Controlled witness: with coherent control q and a clean g, the encoded
function is `q*a*b`. The m=1 correction is the phase
`(−1)^(q*a*b)`, a CCZ on (q,a,b), before any required transport.
An unconditional CZ(a,b) would distinguish/dephase the q=0/1 branches.
Even the q=0 branch measures the zero workspace in X, producing a random
outcome; no measurement outcome may reveal q. A controlled implementation
must have its own resource estimate and channel check.

| Alternative | Consequence |
|---|---|
| Keep exact 7-T CCX and reverse it | Sound arithmetic at 14(W−1) T for W−1 compute/undo pairs under that decomposition; does not restore the promised measurement capability. |
| Certified clean-input C, unitary C† cleanup | 8(W−1) T for those pairs; useful separately, but still no measurement mechanism. |
| Certified C and measurement/fixup (recommended) | 4(W−1) T for Gidney's corresponding uncontrolled carry construction; other Clifford work, workspace, measurement and latency remain chargeable. |
| Declare any basis-correct/relative-phase sequence `perm` | Unsound: basis values omit phase and dirty-target behavior. Reject this proposal. |

These arithmetic counts assume exactly W−1 such carry pairs and no other
T gates; they are not a new execution of the sibling adder generator.

## Recommended exact spec diff

Add a semantic K entry covering §§4.3, 6, 8.3, 11, 12 and 17. Use the next
free K number; the following is proposed replacement/addition text.

Replace the last two sentences of §6.2 by:

```diff
-Every perm value MUST be
-cleaned within its window and before its region exits; otherwise the
-region is **ill-formed** (closed-window measurement-based cleanup is not
-admitted). `other` values are released only by the adjoint of their
-computing sequence, and only while no other live value, interface wire
-or outstanding diagonal phase depends on them, as defined in §6.1.
+Every workspace value MUST be cleaned before its region exits. Unitary
+reverse/recompute cleanup retains the window, version and effective
+control obligations above; recompute remains legal only for perm values.
+Unitary cleanup of an uncertified other value requires the exact
+computing adjoint and no outstanding dependence as defined in §6.1.
+A closed window does not forbid a measurement/fixup realisation whose
+complete channel satisfies §6.4. A certified clean-isometry boundary may
+establish perm provenance as defined below. Neither exception licenses
+an uncertified release or a basis-only phase assertion.
```

Append to §6.1 and add the indicated §12.1 structural rule:

```text
A clean-isometry boundary is a contiguous computing sequence C identified
by source points, with ordered read-only inputs x, clean allocated target
g, and no other modified wires. A structural claim has
rule:"clean_isometry", witness:{points,inputs,targets,table}, where each
table row gives x and g(x) as canonical bit strings. The checker verifies
all input rows exactly against §4.4's matrices, including amplitude +1,
all untouched wires, and restoration of any internal workspace. Only
exact cyclotomic gate expressions need this finite table rule; other
gate families may use a compositional library proof of the same equality.
An incomplete or numerical-only check supplies no static provenance fact.
The checked equality is C|x,0>=|x,g(x)> on the declared clean-input
subspace, not equality to a permutation on arbitrary g. Its target at
that boundary has perm provenance with dependency set x and the effective
control versions, and may be read and cleaned as such. Internal gates
retain their actual non-permutation classification for process claims.
For source-pair formation the certified compute and its explicit exact
adjoint are indivisible sequence occurrences, subject to §6.2; K81's
unique ownership rule still applies. Their source leaves and hashes are
unchanged. Controlled boundaries must establish the equality including
every inherited control value and exact phase.
```

Replace §6.4's blanket prohibition and §17 item 6's unitary-only default:

```text
Cleanup realisations are reverse, recompute and measure_fixup. The first
two retain their present rules. measure_fixup names a certificate cleanup
witness {compute_points,cleanup_points,measured_wires,outcomes,fixups,
claim}; points are concrete source occurrences with §11 coordinates,
outcomes enumerates all bit strings in measured-wire order, and each fixup
is an ordered gate list on identified live wires after the measurement.
The lists bind the actual schedule, operand versions and control context.
Every branch measures only the designated private workspace, performs
its declared fixup and resets/releases that workspace. No outcome is
exposed through the launch measurement list or any external effect.
The checker constructs every branch Kraus operator from the realised
gates and measurements and verifies the complete induced channel against
the source's unitary-cleanup semantics, using §12.1 channel claims on all
invocation contexts with inherited controls included. A structural exact
proof may discharge a zero-error channel identity. A basis table alone
never discharges this obligation. Conjugated corrections are established
for the actual intervening operations; a stale computational-basis phase
fixup is not presumed correct. These witnesses may cover closed windows.
An eligible reverse/adjoint invocation implements the logical inverse
channel with its own checked cleanup realisation and fresh private
outcomes; it need not physically undo a measurement. Only verified
unitary-equivalent data channels satisfy the reversible interface.
```

In §8.3 change `mbu:false` to a Boolean; `measure_fixup` requires
`mbu:true`, mid-circuit measurement and declared measurement/reset gate
costs and feedforward latency. Extend §11 and §12.3 cleanup `realisation`
enumerations and add the witness above beside each selected cleanup.
The channel branch lists and all private measurement/reset/fixup slots
belong to the bound expansion document: publish a new expansion version,
not a silent alteration of `ref@0.9`. Charge every scheduled conditional
fixup slot and its reserved duration regardless of outcome, so the padded
resource report is deterministic; measurement and feedforward introduce
explicit timing dependencies. §12 error composition remains in diamond
norm for channels. Extend §14/T4 to state the logical-inverse hypothesis
above for private measurement-assisted realisations; source `measure`
continues to consume its register and bar reversal.

This is a coupled contract repair. Council must grade the whole mechanism,
not accept a provenance waiver while retaining the measurement ban.

## Hashes, catalog and consumers

Existing canonical source and `ref@0.9` bytes need no change; existing
source closure hashes do not become different CCX definitions. New
certificate witness bytes and schedules change their hashes; targets
enabling measurement change target_hash. A new expansion document has a
new version and hash and requires explicit consumer support. Native CCX
retains its all-input contract. Arithmetic may publish distinct checked
implementation variants; VM, scheduler, circuits and certificate checker
must agree on private outcomes, reset, control, reversal and accounting.
No logical failure probability is substituted for approximation epsilon.

## Conformance vectors and verification

Require exact clean-subspace columns for all four (a,b) inputs, a negative
g=1 CCX substitution, all eight (a,b,y) inputs for C/use/C†, and channel
matrix units for both measurement outcomes. Repeat with coherent q,
external reference, adjoint and H_a closed-window witness. Missing CZ,
missing q control, untransported correction, incomplete outcome list and
unreset workspace must fail. W=1,2,3,5 give carry T counts 0,4,8,16 under
the stated construction. Check both outcomes' timing and padded charges.

An initial eight-column complex-amplitude calculation was run with
`python3 -B -` and matched the table above; floating-point agreement is
evidence only. The exact ring and channel verification is reproduced by
the Python block in [contract-verification](2026-11-02-contract-verification.md).
No reference or sibling arithmetic test was run for this proposal.

## Release and after acceptance

Recommend a **0.9 semantic K repair** for the missing North Star
capability, with explicitly versioned new expansion bytes. A 0.10-only
mechanism is an alternative only if l2p remains an explicit blocking
contract gap for the current scope; it cannot be called a harmless
efficiency deferral. After council acceptance, publish the schema and
positive/negative vectors together, then let the interpreter and consumer
owners implement and validate them. No existing contract bead is closed
by this proposal.

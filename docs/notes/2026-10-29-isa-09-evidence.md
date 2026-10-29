# Evidence and derivations for ISA 0.9

2026-10-29. Review of council-accepted ISA 0.8; **proposal only**. Nothing in `spec/` is changed. The dialect 0.2 outline is ratified as an outline, not a normative ISA extension. Implementation of 0.9 still waits for dialect acceptance and the maintainer's go.

| Item | Verdict | Rung dependency | Hash impact |
|---|---|---|---|
| BLK-1 / ISA-4 | gap: dynamic register type and interoperable handle lookup | none at fixed demo widths | V |
| BLK-2 / ISA-1 | gap in device minting; finite dispatch already exists | none | V |
| BLK-3 / ISA-3 | gap in certified parameterised qinplace | none; general controlled arithmetic | V |
| BLK-4 / ISA-12 | gap in R allocation | none | V |
| ISA-2 | already-realised: counter instrumentation; named fault needs ISA-7 | optional R budgets | — / V |
| ISA-5 | efficiency: joint action avoids simultaneous slices | none | — |
| ISA-6 | already-realised: measurement instrument and re-preparation | feedback; no particular rung | — |
| ISA-7 | gap: fault-code contract plus explicit CC/R fault | no fault-free rung depends on it | V |
| ISA-8 | gap: reverse FAILED prefixes; propagation wording | fault recovery, beyond fault-free 3/6 | V |
| ISA-9 | efficiency: clean fallback or Q borrow frame | none | — |
| ISA-10 | wording-only: distinguish acyclic and recursive hashes | all object consumers | — |
| ISA-11 | efficiency: promotion preserves programs; native placement absent | no rung | — |
| G-12 | efficiency: allocation replay and byte copy | 7 runtime-snapshot cost; optional 2/4 | — |
| G-13 | efficiency: static mapping with address translation | none | — |
| G-14 | efficiency: finite structured expansion, subject to unitary cleanup | none; general action control | — |
| G-15 | efficiency: sliced oracles sharing a fault accumulator | wide OUTs; no scalar-predicate rung | — |
| G-16 | efficiency: saved continuations and polling | 5/6/7 mechanisms; native fibers unnecessary | — |
| I1 | efficiency: explicit reversible pebbling expansion | 4(c)/7(c) space, not feasibility | V if adopted |
| I2 | efficiency: structure recovery, with immutable selectors | 4(c) usefulness; not 4(a)/(b) feasibility | — |
| I3 | already-realised: host budget plus bounded wrapper | 2/4/7/8 | — |
| I4 | wording-only: logical history versus physical storage | 3/5/6 space | — |
| I5 | efficiency: identity idle step with encoded history | variable-time infrastructure; no rung | V if adopted |
| I6 | efficiency: atomic budget ladder exists; automatic composition unverified | none | — |
| I7 | wording-only plus deferred fixture/library work; triage below | all; no new rung blocker | — |
| I8 | wording-only: row indices | every lifted rung | — |
| SQLite E1 | efficiency: stack layout; reported saving unverified | optional 4(c) | — |
| SQLite E2 | efficiency: frame layout; reported width unverified | optional 4(c) | — |
| SQLite E3 | efficiency: compare tree and static call | 4/5/6/7 use construction | — |
| SQLite E4 | efficiency: memory-copy lproc | 2/4/5/6/7/8 use construction | — |
| SQLite E5 | efficiency: classical QROM specialisation | 4(c) | — |

**Hash legend.** `—` means no change to any of the four published 0.8 hashes is required by this item's construction or proposed editorial wording. `V` means a semantic 0.9 change requires the full version bump. ISA §9.2 says `"isa=" <full version string, e.g. 0.8>` is envelope part (1); §1 says “drafts are not compatible with each other.” Consequently **all four examples acquire new hashes when republished as 0.9**, even when their bodies are unchanged. The old 0.8 vectors remain valid historical vectors. No proposal below silently rewrites those bytes or supplies uncomputed 0.9 digests. New bodies/references/signatures also change their own hashes; runtime argument values and budgets bind schedules/certificates, not function bytes.

## Reading and evidence conventions

Sources read in full: [project rules](../../CLAUDE.md), [handoff](../../HANDOFF.md), [spec change rules](../../spec/README.md), [ISA 0.8](../../spec/Bennett-ISA-Spec.md), [budget note](2026-10-28-budgets-loops-variable-time.md), [dialect outline](../../../bennett-dialect-spec/spec/outline-0.2.md), [SQLite report](../../../bennett-dialect-spec/docs/spikes/sqlite/REPORT.md), [narrowing audit](../../../bennett-dialect-spec/docs/audit/2026-10-29-dialect-0.1-narrowing-audit.md), [north star](../../../bennett/docs/Bennett-NorthStar.md) and [plan](../../../bennett/docs/plan/2026-10-29-path-to-north-star.md). Also read interpreter `SPEC-QUESTIONS.md`; implementation choices there are evidence of ambiguity, not amendments.

`NS` below is north star iteration 7; unqualified section numbers are ISA 0.8. Quotes are from the local files, with line wrapping normalised. A canonical **fragment** is function/instruction text using §16 spelling, not a complete linkable object: symbol metadata, layout and certificates supplied in prose must be filled before it is an executable fixture. Blocks have no comments or indentation. Proposed instructions are labelled **0.9**, not presented as existing opcodes. No parser execution of those proposals is claimed.

`already-realised` concerns observable software capability. `gap` identifies the precise missing contract, not an impossibility of universal simulation. Finite enumeration can express surprisingly much; absence of one opcode does not prove absence of every construction. `efficiency` retains the 0.8 construction and defers an optional primitive. The quantitative comparisons below count reference rows separately from expanded gates and physical wires. A Q oracle call with budget B has B forward and B inverse machine steps plus commit/initialisation; its reference interpreter executes at most B rows. A lifted callee oracle is **one caller row**, but its own full circuit is charged too. CC/R frames are classical history, not B quantum frames.

The plan's R1 is addressed by actions calling outlined oracles, R2 by finite indirect dispatch (E3), R3 by the item reviews, R4 remains research with NS §3.1's explicit “unitary uncomputation is the fallback under control,” R5 by I2 and E5, and R6 by keeping this review separate from accepted text. None licenses narrowing a rung.

## BLK-1 (ISA-4): dynamic quantum registers and handle tables

**Promise and users.** NS §3.1: “Collections of handles are supported through a **handle table**: an aggregate or array of handles indexed by a classical value”; §4.2: “data-dependent trip counts, real allocation.” This covers `Vec<Handle>`, Bennett.jl arrays, and recorded generations whose live register set grows at runtime. Rung 4 can use a fixed-width rowid register and does not require this generality.

**0.8 evidence/construction.** §3.3 gives `qreg[n]`; §16.2 gives `"qreg[" INT "]"`, and §4.8 gives no qreg-to-integer conversion or qreg memory instruction. §10 binds “`QREG` to `qreg[n]` with the register's n.” For a finite set of roots, pass registers as extra CC parameters and dispatch with `icmp`/`br`; no qubit value is inspected:

```text
fn cc @choose (a:qreg[1], b:qreg[1], i:i1) () () () ()
^entry():
br i ^right ^left
^left():
action_call @flip (a) () () ()
ret
^right():
action_call @flip (b) () () ()
ret
action @flip (a:wire) () () () ()
^entry():
x a
```

Each call executes one branch, one region call, one return; the call's X cost is one target gate. For M statically named handles and arity k, direct tuple specialisation has at most M^k leaves; balanced dispatch has O(k log M) classical comparisons, but the finite register file still has M named slots. A recursive CC activation can retain another fixed-width register: therefore **unbounded instance count alone is not a proof that no 0.8 simulation exists**. What is missing is the promised ordinary table representation with runtime id lookup, and a uniform type for a single runtime-sized register. The grammar forbids `qreg[*]`, and the exhaustive opcode schema forbids loading a qreg value from an integer id. Concatenating dynamically would require a result type absent from this finite program's literal type set. This is the narrow counter-argument, not a claim that classical recursion has bounded storage.

**Exact proposed 0.9 sentences** (§3.3, §4.1/4.8, §7.4, §9.1, §10, §16.2):

> `qreg[*]` is a CC/R handle type whose width is a runtime unsigned 64-bit count. `q = qreg.new cls n`, with n an i64 value, produces qreg[*]; the literal form continues to produce qreg[n]. `w = qreg.width q` yields its actual width as i64 without changing the handle. `id = qreg.id q` yields its nonzero launch register id as i64 without consuming it. `q = qreg.from_id id` yields qreg[*] referencing that same register and traps if the id is unknown, consumed, or viewed. These operations do not copy wires or grant another owner. All aliases refer to the same live/viewed/consumed state.
>
> Qreg types may be passed unchanged through CC/R call, ret, and block parameters. Their handles remain in the classical environment; only the integer id is stored with ordinary integer load/store. A qreg[*] region operand is checked against the callee's concrete slot width, n or 2^A·W, before any effect; mismatch is FAULT_UB_TRAP. All operands are checked for wire overlap, including aliases and effective controls. A QREG launch argument may bind to qreg[*] without a width restriction; fixed qreg[n] keeps its equality check.
>
> `qreg.width` and `qreg.id` defining frames are pure; `qreg.from_id` has an impure frame recording the reference, whose reversal removes that dynamic instance without freeing the register. Register ids are not reassigned to another register while a live run or retained history can refer to them. Reversal restores the same ids and alias states. Concat and slice propagate actual widths; a slice's length is a literal for a fixed-width result, or i64 for qreg[*]. The one-call view lifetime continues to apply. `measure.iN q` for qreg[*], 1≤N≤128, requires actual width N and returns iN; mismatch traps before measurement.

Add `"qreg[*]"` to the type grammar/JCS type strings. Add these opcodes to their tier/schema tables; faults use code 4, allocation failure uses the existing allocation-error convention clarified by ISA-7. No new numeric fault code. Allocation and reference frames retain the pre-state and are reversed once under §7.4. Literal slices may also take a runtime `lo:i64`; bounds are evaluated without wrap. A schedule/certificate binds the **concrete** widths, arguments, budget, variant and inherited controls at the call, and is validated then; a width mismatch cannot be excused by an existing static schedule. Runtime scheduling sees classical counts only, satisfying §3.1. Schemas must distinguish the static type `qreg[*]` from the realised slot dimensions; the latter are not wildcards in a gate schedule.

**Identity/tests.** The new opcode/reference tokens and signature spelling enter §9.2 normally; dynamic ids/widths do not enter immutable function identity. A→new width 2→id→store.i64→load.i64→from_id→two-bit action→inverse action→free restores memory and handle ledger under reversal (A/G). F rejects overlapping aliases to two disjoint slots; A/G report code 4 for stale id, viewed parent lookup, and width 3 bound to wire[2], with unchanged wires/history before the fault frame. H accepts two separate schedules for widths 2 and 3 and rejects reusing the wrong concrete binding. With inverse actions, allocation reversal sees |00⟩. Tests must include aliases becoming consumed together and exact id restoration. Hash impact V.

**Verdict: gap — dynamic qreg typing and interoperable id lookup; no demo rung requires it at fixed widths.**

## BLK-2 (ISA-1): budgets computed on the device

**Promise/users.** NS §4.2: “run-time budgets come from the host or from measurement results, never from superposed data.” Adaptive classical feedback is the missing direct case; the eight demos may obtain all budgets from the host.

**0.8.** §5.4: “Budget = (B, heap_limit, stack_limit, depth_limit), classical.” §10 exposes `bennett_budget(... region, uint64_t steps, uint64_t heap, uint64_t stack, uint64_t depth, bennett_token* out)` and “`TOKEN` to `token`.” §4.1/4.8 has no token-producing device instruction. Host-supplied values and any pre-minted finite family already work:

```text
fn cc @select_budget (b:i1, small:token, large:token, x:qreg[8], t:qreg[8], f:qreg[4]) () () () ()
^entry():
br b ^large ^small
^small():
oracle_call @inc8 small (x) t f () ()
ret
^large():
oracle_call @inc8 large (x) t f () ()
ret
```

With tokens B=2 and B=3, §16.3 derives respectively `(t,f)→(t,f⊕1)` and `(t⊕(x+1 mod 256),f)`. Measuring some earlier register to choose b changes nothing about this legality. For K tuple alternatives, pre-mint K tokens and emit O(K) leaves and O(log K) balanced comparisons. Indeed uint64 components have a finite domain, at most 2^256 tuples before validation: an absolute “no candidate realisation” claim is false if exhaustive pre-minting is admitted. **The genuine contract gap is uniform in-device token creation without requiring that enumeration or returning to a host service.** There is no numerical law in 0.8 rounding B up to a power of two; doing so changes F_B.

**Exact 0.9 edit** (§4.1/4.8, §5.4, §7.4, §9.2, §10/11):

> `tk = budget.mint @o s h st d` is a CC instruction with four i64 operands and a linked oracle reference. Before mutation it applies exactly the §5.4 checks of bennett_budget to their unsigned values and @o's layout. Failure records FAULT_UB_TRAP. Success produces a token containing that validated tuple, program and region binding. It is scoped to the run, cannot be used after that run's reversal or close, and is not constructible by integer conversion. The frame is impure and records the token and prior token ledger; reversal revokes the token and removes its dynamic instance. Reversal of a later Q call uses its recorded token tuple before the mint frame is reversed. Replaying a reversed segment cannot silently revive an old id.
>
> At each invocation, the exact tuple and resolved classical arguments bind its schedule/certificate. A missing matching static schedule is handled by ordinary classical scheduling or by E_SCHEDULE_INFEASIBLE before invocation; a consumer MUST NOT substitute a different B. Within an oracle and every matched inverse, the token tuple is immutable.

The opcode's `@o` is a symbol reference, so closure identity follows §9.2; the minted tuple is runtime binding data. Retain host-minted tokens, with their existing program lifetime. No Q `budget.mint`: otherwise wire-dependent allocation and step counts would violate §3.1. No new fault code or step-machine row.

**Consequence/proof/tests.** Fix a measured classical outcome m and let B(m) be the minted value. The called circuit is a composition Π_1…Π_B(m), commit, Π_B(m)^−1…Π_1^−1; cancellation proves T2 separately for each m. This is classical selection of circuits, not a coherent superposition of circuit lengths. A nested oracle with budget b costs its own 2b expanded steps each time its caller row executes; its tuple is recorded for prefix reversal. T1 is unchanged because minting is outside that reference body; T6 and H must bind the same tuple. A/G: mint 2 or 3 after a classical branch, get the hand-derived inc8 outcomes above, reverse the Q call then the mint and check ledger restoration. Invalid heap limit in a memoryless object gives code 4 with no token. F rejects a Q-tier mint and non-token call operand; H rejects a B=3 schedule used for a B=2 call. Hash impact V.

**Verdict: gap — uniform device minting; finite dispatch is already realised; no rung depends on minting.**

## BLK-3 (ISA-3): parameterised in-place bijections

**Promise.** NS §4.2: “an in-place update x ← g(x) requires a total bijection with a bounded inverse”; the stated founding example is “`when(q) do x += a end`.” Quantum y, memory parameters and runtime a must remain available, with no per-value source rewrite.

**0.8.** §4.5's entire operand form is `qinplace @g @ginv tk x`; it requires “certificates that under tk the fault code is FAULT_NONE on every input and g∘ginv = ginv∘g = id.” §4.7 already has `(ins..) (mem..) (args..)` for oracles. §6.2 says “Each cleanup is bound to its exact computing sequence”; §6.4 permits only `reverse` or `recompute`. Thus blindly spelling compute(g), swap, compute(ginv), free does not establish ordinary workspace cleanup by that exact sequence. The **certified macro** is what authorises this change of inverse inputs. Missing operands forbid directly invoking a parameterised certified pair, not every equivalent circuit.

Examples that already work: specialise a finite classical parameter set; or use a library permutation action such as

```text
perm_action @xor2 (x:wire[2]) (y:wire[2] in) () () ()
^entry():
cx y[0] x[0]
cx y[1] x[1]
```

It costs two exact CX gates, maps x→x⊕y and is its own inverse for all 16 pairs. For an arbitrary small finite encoding, a producer could synthesise the complete permutation over x,y into exact gates, or exhaustively specialise classical parameters. That is a potential enormous-cost construction, so no universal impossibility theorem follows from this opcode omission. The missing contract is general reuse of the **declared certified pair** with its parameter interface and cleanup rule.

**Exact 0.9 edit** (§4.5, §5.1, §6.1/6.2, §12.3, §16):

> `qinplace @g @ginv tk x (ins..) (mem..) (args..)` accepts oracles whose first input has x's width, whose remaining inputs, in-regions and classical arguments have the supplied signatures, and whose target has x's width and fault widths agree. For every supplied classical argument tuple and every basis value of ins and mem, both oracles are fault-free under the token tuple and induce mutually inverse total permutations of x. The tuple is validated against both callees' layouts. The expansion is the §4.5 sequence with `(x, ins..)` and the identical mem and args lists passed to both calls. No parameter wire or in-region cell is changed between those calls.
>
> The certified inverse-pair expansion is a permitted cleanup of its private t and f even though the second call uses ginv and the successor x. This exception applies only to that expansion with its checked inverse-pair claim. Its adjoint exchanges g and ginv and retains every operand list. Transfer: if x was perm, its successor is perm with dependency set S_x ∪ ins ∪ cells(mem) ∪ Π, including the versions read; if x was other it remains other. Classical arguments are fixed at entry and are part of the bound inverse-pair claim.

Preserving `other` avoids falsely laundering a Hadamard-produced value into recomputable garbage. A parameter wire's later non-diagonal operation or an un-restored memory version change closes dependent cleanup windows under §6.2. The original transfer omits parameters because the opcode had none; copying it unchanged is unsound. The two references and all new operand bytes enter §9.2; args values enter params_hash and certificate bindings. No new frame: Q macro scratch is private, CC/R region-call frames already reverse it by adjoint; no new fault code because the pair is required total. A failing certificate is ill-formed, not a data-dependent “in-place fault.”

**Derived fixture.** Let g(x;y,a)=x+y+a modulo 4, ginv(x;y,a)=x−y−a modulo 4; two arithmetic rows and retval suffice with an extra temporary, so each oracle needs B≥3. For x=3,y=2,a=1: g=2; after compute t=2, exchange gives x=2,t=3; ginv(2;2,1)=3, so t=3⊕3=0, f=0. Generally t_old=x and ginv(g(x;y,a);y,a)=x, hence private target is zero. Under external control 0 every gate is disabled and scratch remains zero; under 1 the same proof applies. C exhausts x,y∈[0,3], a∈[0,3], both controls and checks inverse/clean scratch. F rejects a parameter alias of x or an incorrect inverse claim; §6/F rejects cleaning a dependent copy after changing y. C includes arbitrary initial x and a fault-free pair at the exact budget boundary. Hash impact V.

**Verdict: gap — parameterised certified inverse-pair macro and transfer rule; no demo rung requires this general pair interface.**

## BLK-4 (new ISA-12): allocation inside R closures

**Promise/users.** NS §3.1 lists “allocate (clean / dirty / class)” as ordinary markers; NS §4.2 gives device reversible execution with “real allocation.” R calls Q (§3.1), so an R routine using quantum scratch must be able to allocate its runtime number of registers. The ordinary rung 3/5/6 programs do not themselves allocate qubits.

**0.8.** §4.1 says `qreg.new/free/slice/concat` are “CC only”; §4.8 repeats that tier set. §3.1's graph permits R→R,Q, not R→CC allocation helpers. Hoisting a statically bounded set of allocations to the CC caller works, then passing those qregs to R calls works; hoisting a number selected during R execution has no such direct instruction. This tier restriction is the exact counter-argument for an R allocation, independent of BLK-1's width issue.

**Exact 0.9 edit** (§3.3, §4.1/4.8, §7.4, §10, §17.14):

> `qreg.new`, `qreg.free`, `qreg.slice`, and `qreg.concat` are permitted in CC and R with the same handle preconditions and transitions. Their R failures are terminal fault records using §4.1's cause mapping. Their history frames retain ids, class, width, ownership and prior handle/view states. Undoing new releases the same register only after proving it clean; undoing free restores the same id, width, class and clean wires. Undoing slice/concat restores the exact parent/child ownership and ordering, without copying or measuring a wire. An alias observes these same transitions. A qreg new frame is not merely an environment-instance deletion.

`measure` stays CC-only, as the existing reversible-tier contract requires. Reversal failures obey the existing simulator cleanliness check and §17.14 hardware fault rule; no imagined nondestructive quantum test is introduced. For segment uncall, §7.4's exclusion of unverified frees remains necessary. Extending these handle operations into R changes the function body/tier schema, hence V; no new Q opcode, budget unit, or fault number.

**Consequence/fixture.** Proposed R fragment (0.9):

```text
fn r @scratch () () () () ()
^entry():
q = qreg.new any 1
action_call @flip (q) () () ()
action_call @flip (q) () () ()
qreg.free q
ret
```

Forward scratch is |0⟩→|1⟩→|0⟩→released. Reverse traverses free (recreate |0⟩), second X (|1⟩), first X (|0⟩), new (clean release). **It is not legal to reverse new by discarding an unknown state.** If R returns a qreg and CC later changes it, segment uncall cannot cross those impure suffix frames; whole-run reversal must undo them first. A/G test these traces, dynamic repeated allocations, concat/slice restoration, and hardware-unverified free exclusion. C checks entangled interface data is unchanged by an allocate/use/uncompute/free R helper; F accepts R allocation and continues to reject R measurement. Hash impact V.

**Verdict: gap — permit and reverse R qreg allocation operations with cleanliness obligations; no demo rung requires quantum allocation inside R.**

## ISA-2: enforcing an R-tier budget

NS §3.1 promises “A region-level classical budget: dynamic steps, heap bytes and recursion depth”; §4.2 calls classical control “genuinely dynamic.” ISA §3.4 says “Sequential small-step execution … dynamic control flow and allocation”; §5.4's budget is a Q budget. An R budget can be software instrumentation:

```text
fn r @countdown (left:i64) (r:i64) () () ()
^entry():
zero = const.i64 0
one = const.i64 1
br ^test(left)
^test(k:i64):
empty = icmp.eq k zero
br empty ^exhausted ^work
^work():
next = sub k one
br ^test(next)
^exhausted():
trap
```

For this intentionally non-returning work loop and input B, the uninstrumented work executes B times; the generated trace has 3 + 4B + 2 + 1 = **4B+6 instructions**, including the trap. k is decremented only when nonzero, so no unsigned wrap. A general lowering threads remaining steps through calls/returns and checks before each **original reference operation**; generated checking instructions are not recursively charged. Fixed-width decrement handles the entire initial unsigned range. Heap instrumentation uses cumulative allocations, not live bytes; depth increments on entry and decrements on return. All inserted state is ordinary dynamic SSA/memory history and reverses under §7.4. A zero budget reaches the trap before work.

§4.1 provides only `trap` (“R fault record”), so exact code **1** for exhaustion is not obtained from this construction until ISA-7's explicit fault instruction. Replace the final trap with proposed `fault 1`; do not silently call every trap budget exhaustion. No extra R budget object or counter ISA is necessary. A can test B=0,1,2 work counts and reverse prefixes; after ISA-7 they all end with code 1. Q T1/T2 and published bodies are unaffected.

**Verdict: already-realised — R budget instrumentation; exact named exhaustion uses ISA-7; no extra rung blocker.**

## ISA-5: simultaneous slices of one register

NS §3.1 promises ordinary “gate” markers and handle collections. §3.3 states “q2 MUST be the operand of exactly one region call and of nothing else” and “any use of q while viewed is ill-formed.” This forbids taking a second slice of that parent before the first call. It does **not** forbid a joint action on the parent:

```text
action @cx_0_7 (q:wire[8]) () () () ()
^entry():
cx q[0] q[7]
```

A CC caller passes its live qreg[8] to this one action, so no slice exists. Exact map q_7→q_7⊕q_0, all other bits unchanged; one CX. Runtime classical indices can dispatch to the n(n−1) ordered distinct-pair actions; a binary tree uses O(log(n(n−1))) comparisons and O(n²) static leaves, **one** quantum CX at execution. For triples there are n(n−1)(n−2) leaves. Alternatively compute selector bits into wires using classical args, perform a fully enumerated controlled action and uncompute selectors. No O(n) general routing bound is claimed without constructing that routing network: the audit's bare O(n) assertion is not verified here.

The smallest 0.9 action is **no semantic change**: use joint actions. Clarify §3.3: “The slice lo operand may be an unsigned classical i64 value; lo+len is checked without wrap. With literal len the result has qreg[len]. The parent becomes live when the one region call completes, including on its recorded inverse.” This is proposed wording for an underspecified operand kind; conservative 0.8 producers can dispatch literal lo values meanwhile. `qreg.slices` would be optional efficiency work; no new multi-view instruction is proposed in this review. A/C/F tests compare all 56 two-bit choices in an eight-bit parent, reject overlapping controls/targets and a second simultaneous view. No new faults/history beyond existing code-4 handle failure and view frames. Existing hashes unchanged.

**Verdict: efficiency — the joint-parent action realises multi-wire gates on 0.8; no rung dependency.**

## ISA-6: non-consuming computational-basis measurement

NS §3.1: “Marker call returning i1, flowing into ordinary br / select / phi, outside unitary regions.” ISA §3.3 says `r = measure q` consumes q and releases its wires; §14.2 specifies projective computational-basis measurement with Born probabilities. For one wire:

```text
fn cc @measure_keep (q:qreg[1]) (out:qreg[1], m:i1) () () ()
^entry():
m = measure q
out = qreg.new any 1
br m ^one ^done
^one():
action_call @flip (out) () () ()
br ^done
^done():
ret out m
```

Write an arbitrary joint input as |0⟩|e_0⟩+|1⟩|e_1⟩. Outcome m has probability ||e_m||² and post-state |m⟩|e_m⟩ after normalisation. Destructive measurement followed by fresh |0⟩ and conditional X gives **the same instrument on data and reference**, not just the same marginal probabilities. Replace the dialect handle's id with out; any handle table must update that root consistently (BLK-1 for general dynamic tables). This costs one measurement, one clean wire and at most one X, plus 5 instructions on outcome 0, 7 on outcome 1. General n≤128 uses a fixed preparation action with m's bits materialised from args; arbitrary-width partial observation uses a one-bit CNOT copy into a declared output register, then measurement, leaving the source collapsed. That output is not private workspace freed by the copying action.

Neither construction enables reversal across measurement: §7.4 and §10 still exclude it. No new opcode, frame, fault code, identity change, or normative measurement rule is needed. A/G test outcome 0/1 and consumption; D tests an entangled reference state, with the conditional reference states above as expectations. No particular demo requires retaining a measured qreg.

**Verdict: already-realised — destructive measure plus exact re-preparation reproduces the instrument; no rung blocker.**

## ISA-7: trap causes and explicit classical faults

NS §0 promises “a named fault outcome … never a silent wrong answer”; §4.2 names distinct causes. ISA §4.1 says `udiv` gives “CC trap; R fault record” and `trap` likewise; §10 requires launch faults to be “codes of §5.4.” §7.1 explicitly gives allocation/free codes and §17.14 gives unclean-free code 4, but the missing CC cause mapping lets implementations disagree. §4.8 currently confines `fault` to Q oracle/lproc rows. A `hostcall bennett.fault` can export a requested code before a trap, but does not define `bennett_result_fault`'s code and is CC-only. Thus it is not a full realisation.

**Exact edit** (§4.1/4.8, §5.4, §7.4, §10):

> In CC and R a failing instruction validates before mutation and appends one terminal fault record. Its cause is: zero divisor → FAULT_UB_DIV0 (5); signed MIN divided or remaindered by −1 → FAULT_UB_SDIV_OVERFLOW (6); invalid classical load/store/free or pinned free → FAULT_UB_MEMFAULT (7); allocation capacity/table exhaustion → FAULT_BUDGET_HEAP (2); invalid handle state, call binding, segment uncall, or explicit trap → FAULT_UB_TRAP (4); device failures use §7.3's named causes. The record is returned unchanged by bennett_result_fault. No result values are exposed on failure.
>
> `fault C` in CC/R takes a literal unsigned 64-bit code and appends that terminal record without other state mutation. It is impure; reversal removes the record and restores the pre-fault control position. In Q its existing F-bit literal rule and row semantics are unchanged. Code 10 names entropy exhaustion in virtualised coherent state as well as in R; remove the parenthetical “R only.”

The unsigned width is derived from the existing `uint64_t bennett_result_fault`, not a new numerical cause assignment. Retain literal `fault 0`'s existing Q semantics rather than inventing a new invalid-code rule here; consumers must distinguish terminal status from code value. For R instrumentation `fault 1`, `fault 2`, `fault 3` now report exactly those causes. A/G tests division by 0→5, −128/−1 at i8→6, null load→7, exhausted allocator→2, trap→4, explicit fault 1→1, each with unchanged pre-fault data. B tests a virtualised exhausted entropy oracle's `fault 10`, target unchanged and fr⊕10, all scratch reversed. F checks literal width/tier schema. Function text containing new classical faults changes its hash; the cause mapping is semantic, so V applies to 0.9 publication. No new fault numbers.

**Verdict: gap — exact CC/R fault causes and an explicit classical fault instruction; no fault-free rung depends on it.**

## ISA-8: terminal R faults and reversal of failed prefixes

NS §4.2: reversible effects are “recorded on the effect tape forwards, replayed backwards”; §5 says the kernel “un-boots to the reset vector.” ISA §7.4 says “terminal faulted state, reversed like any prefix,” but §10 says “Reverse eligibility: COMPLETED” and lists only `COMPLETED → REVERSED`. It cannot expose failed-prefix reversal through that ABI. The fault-free rung 3/6 paths are unaffected.

The caller behavior can be made precise without adding an exceptional return value to **every** R function's signature. The outline §17.6 already selects terminal failure, not catch-and-continue. **Exact 0.9 edit** (§4.1, §7.4, §10, §14.3):

> An uncaught R fault terminates the invocation tree and its CC run as FAILED; the CC caller executes no successor instruction and receives no ordinary callee results. bennett_wait returns that failure and its ordered fault records. Retained invocation boundaries describe an incomplete call as well as a returned call.
>
> COMPLETED or FAILED may transition to REVERSED if the entire executed prefix has retained reversible history, contains no measurement, contains no unverified quantum release, and satisfies the existing exclusive-ownership and subsequent-use checks. Reversal visits the retained prefix, including the terminal fault record and incomplete activations, and restores its entry state. A failed validation at the faulting instruction has no partial effect beyond its fault record. E_NOT_REVERSIBLE applies when those conditions fail.

This extends recovery, preserves every prior completed-run behavior, and does not manufacture success results. A future catch API is not needed for this capability. All remaining terminal failure paths must retain a prefix or fail the explicit eligibility test; hardware loss of an unknown state cannot be reversed by pretending its frame was classical. No new numeric fault. The T4 wording in I7 distinguishes logical state restoration from retained audit records.

Fixture A/G: CC calls R; R performs clock.read to a live eight-byte buffer, then `fault 4` (or existing trap with ISA-7 mapping). Forward c=1 and the buffer contains 0; run FAILED, result fault=4, no CC continuation store. Reverse fault, clock.read (restore overwritten bytes, c=0), and allocation/call prefix; entry memory and device state restored. Include a trap inside recursion before ret and a failed run after measurement (E_NOT_REVERSIBLE). H/schedules need no new entry kind. §9.2 schemas unchanged locally; launch semantics requires V globally.

**Verdict: gap — reverse eligible failed prefixes; terminal propagation is clarified, not a new R return convention; no fault-free rung dependency.**

## ISA-9: CC-spanning dirty borrowing

NS §4.3 calls dirty borrowing “a first-class resource” and requires “restored correlations.” ISA §4.3: `qborrow` gives “n wires in unknown state; usable only inside a borrow frame”; §4.8 places it in Q actions; §9.4 says “callees qborrow only from the caller's designated idle set.” Direct CC qborrow is absent, but a correct user must implement U⊗I_B for **every** borrowed state, including |0⟩. Clean CC allocation is therefore a valid semantic fallback; existing dirty owners remain untouched with their correlations preserved. The fallback does not justify charging a dirty rate for clean allocation.

Concrete Q borrow construction is I7's `borrow_demo`: b borrowed; `cx x b; cx b t; cx x b; cx b t`; return b. On basis b, t receives (b⊕x)⊕b=x and b is restored. Linearity gives identity on any external reference entangled with b. This is already a structural translation-lemma proof. A spanning CC user with an equivalent circuit can group it into a Q region, or allocate a clean qreg and do two CX calls via a joint action, then free only after restoration. Cost of clean fallback is n extra clean live wires for its whole lifetime, occupancy n·duration·class_rate instead of the borrower's dirty rate; that formula is §8.2's live-interval integral, not a gate-cost guess.

No new CC borrow primitive is proposed for 0.9: a borrow contract spanning dynamic CC calls would need an owned idle-set reference, prohibition of intervening owner use, and a full-domain frame claim, not merely new syntax. Capability remains via the constructions above. C/F tests the four-gate positive vector and existing negative vectors; E charges the actual clean/dirty class correctly. No new faults, history or identity change.

**Verdict: efficiency — clean fallback or an existing Q borrow frame; no rung dependency.**

## ISA-10: acyclic hashing

NS §4.4 promises “serialisation and versioning rules”; §5 promises linkable certified libraries. ISA §9.2 says “member hash = SHA-256 over the ASCII bytes of <scc_hash hex>:<decimal index>.” The normative §16.3 vectors distinguish an acyclic symbol from a **recursive** one-member SCC. Independently hashing the printed inc8 envelope gives:

```text
direct = 8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93
wrapped = 5f3e8bcd2ec81818085049dbc03dcea6f470c26e1de0bfb231d389985a8c8ee8
```

The published hash is direct. For recursive sum the printed envelope digest is `c6d549be70e7018cd6d0b6bc5f7dd1fa3d869e5b1263c661f12ad5b11b49bc31`; hashing its ASCII hex plus `:0` gives the published `74f110bbafde68a4785025e7e571ef871ddd83a09465ab9c1604f653a0cfcd92`. The seed implements this exact distinction, independently of the audit's assertion.

**Exact editorial sentence** (§9.2): “An acyclic symbol's hash is SHA-256 of its own envelope. The SCC envelope concatenation and member wrapper apply to a cyclic SCC, including a one-member SCC with a self-reference.” F checks inc8, zero and sum_db direct, sum wrapped, and a mutual SCC; C keeps all four published 0.8 vectors. No instruction, frame, fault or capability change; no local hash changes. Suggestion zsw.11 as written is rejected in I7 because it contradicts these bytes. A full 0.9 version bump still changes all republished hashes.

**Verdict: wording-only — direct acyclic hashes and wrapped cyclic hashes, preserving the 0.8 vectors; all object consumers depend on the rule.**

## ISA-11: pointer-rich host memory and native placement

NS §5: “Partial compilation is a region marker”; NS §3.3: “native host code for the classical tier, ISA object(s) for the device tiers.” ISA §10's positional kinds are INT,F64,QREG,TOKEN and “Arguments and the entropy tape are copied”; §7.1 initially has only globals and a monotonic bump heap. The precise missing direct interface is mapping arbitrary **existing host objects at their numerical addresses** into CC memory. It cannot be expressed as a MEM launch argument in the closed §10 schema.

The outline's promotion construction is valid for bitcode-available programs: promote the transitive host callers sharing pointer-rich state to CC, create every node with CC alloc, and leave genuine host effects behind hostcall. Example list creation fragment:

```text
n = const.i32 16
p = alloc n
q = alloc n
v = const.i64 7
store.i64 q v
bits = ptrtoint.i64 q
store.i64 p bits
r = load.i64 p
next = inttoptr r
answer = load.i64 next
```

At ptr_width=64 with two disjoint live 16-byte allocations, answer=7; both pointer values are preserved, with no guessed relocation of untyped bytes. Ten instructions, 32 bytes cumulative heap, two unpinned table entries; appending free q/free p is reversible and does not reclaim the heap. For a whole program, promotion adds one history frame per executed instruction and hostcall marshalling proportional to actual argument/reply bytes; it need not copy a whole graph on every Q call. Non-bitcode host libraries require the already-promised runtime shims, not rejection of pointer-rich source programs.

A native-placement optimisation would need a new launch MEM argument, authoritative live-object bounds, aliasing rules, exclusive ownership while a run is open, copy-back/reversal timing, and a host-window partition of §7.1. `alloc.at` alone is **not** a complete proposal: it does not supply bytes, table state or shared ownership. No such new primitive is accepted for 0.9 here; the program capability is kept by promotion, while staying at native speed is deferred mechanics. If an already running foreign process must pass arbitrary pointer graphs without moving its execution, that exact interoperation remains **unverified**, not promised by this construction. No rung uses that boundary. A/G test promoted graph results and byte-exact reversal; hostcall copies remain tested separately. No new fault/history/hash rules proposed.

**Verdict: efficiency — promotion realises pointer-rich programs on 0.8; direct mapping of an already-existing foreign heap remains absent; no rung dependency.**

## G-12: runtime snapshots at native addresses

NS §8 rung 7: “Kernel state snapshot fixed classically after boot”; §4.2: “every other write being restored workspace.” ISA §3.7 says “storage of mem all zero except global images”; §7.2 requires “all ranges disjoint and within [0, 2^H).” These forbid directly overlaying a snapshot register onto an already-declared global/heap range. They do not forbid allocation replay and copying.

**Construction.** Record **every** past allocation in bump order, including zero-sized allocations and those later freed. Replay `lift.alloc` with those sizes; copy each live object's bytes and mutable globals from a disjoint snapshot `in` region, then replay dead live bits with `lift.free`. For allocation j the address is

`p_j = heap_base + Σ_{i<j} max(n_i,1)`.

Induction on j proves identical pointers and bump. Live-only replay is wrong: alloc(0), alloc(4), free(first) places the second object at heap_base+1, not heap_base. Reordering allocations or ignoring dead entries also changes table-capacity faults. An executor can copy fewer bytes based on a proved read set, but must retain the allocation prefix that determines addresses.

Canonical byte-copy fragment for a global g of positive size M, a snapshot region snap whose first M bytes hold g, ptr_width=32:

```text
s = lift.addr @snap
d = lift.addr @g
i = lift.const.i32 0
one = lift.const.i32 1
limit = lift.const.i32 4
qjump ^copy
^copy():
sa = lift.add s i
v = lift.load.i8 sa
da = lift.add d i
lift.store.i8 da v
i = lift.add i one
c = lift.icmp.ult i limit
qbranch c ^copy ^continue
```

Here limit=4 is the concrete instance; replace its literal by M when generating another fixed object. Declarations require s,d,i,one,limit,sa,da:reg[32], v:reg[8], c:reg[1]. Six entry rows and seven per copied byte give **6+7M** rows before continuation; both accesses already have inherent §4.2 checks. A dialect emitting explicit boundschecks adds 2M rows. For J allocations, J_f frees and M copied bytes, a general streamed prologue costs O(J+J_f+M) reference rows plus decoding; cumulative replay heap is Σ max(n_i,1), table entries J+globals+regions, **not live object count**. Prefix reversal restores g and heap and the snapshot input exactly. Each memory row additionally expands into address/table scans: row count is not gate count.

CC preparation using 128-bit chunks requires C=ceil(snapshot_bytes/16) leaf qregs/oracle calls and C−1 concats, O(snapshot_bytes) bit preparation work; padding must be zero. For a power-of-two byte length 2^A≥16 there are A−4 **concat levels**, not A−4 total joins. The snapshot metadata, ids and unused cells must be sized explicitly; it is not an implicit launch copy.

**No mandatory 0.9 change.** An overlay would be an efficiency primitive requiring a precise table image (including freeable entries), bump/heap accounting and borrowed-state restoration. The closed 0.8 layout forbids the direct overlay, but not this construction. Adding a vague `overlay` field is insufficient to specify those obligations, so it is not accepted this round. The C′ route in the SQLite report also requires its arena to be outside the ISA heap range: “arena at heap_base and mapped there” would violate disjointness. Route A avoids this issue entirely. B/C tests a mutable global, replay of sizes 0 and 4 with a dead first object, pointer equality and prefix cleanup. No new code/frame/identity; 7(c) runtime-snapshot costs remain a real optimisation target, not a loss of rung 7.

**Verdict: efficiency — explicit replay/copy realises runtime snapshots; rung 7 cost, not feasibility; no rung-4 dependency.**

## G-13: interface bases chosen at call time

NS §4.2 promises coherent memory with “classical inputs … fixed before entry.” ISA §9.1 fixes `interface_regions.<oracle>.<region>.base` in layout; §7.2 requires disjoint ranges. A dynamic base cannot be substituted into that field without changing object identity. Static arena placement already supports every demo.

A compiler may preserve **logical pointer values** and translate only at each access. Let logical p address an L-byte interface starting at classical runtime base b, whose static mapped base is a. With a valid logical range check b≤p and p+length≤b+L at widened precision, perform:

```text
off = lift.sub p logical_base
physical = lift.add mapped_base off
value = lift.load.i8 physical
```

Here p, logical_base, mapped_base, off and physical have ptr_width; logical_base is a classical arg and mapped_base is a location from `lift.addr @db`. For a concrete L=16, a=4096,b=8192,p=8195: off=3, physical=4099, so the byte is db[3]. Pointer comparisons, hashes and stores still use logical p. At most two translation rows per access after selecting its range; O(r) compare tests for r mapped logical ranges, or a balanced tree if range order is known. Range classification must distinguish globals, heap and interfaces and preserve source object bounds; aliasing logical regions must be represented by one shared backing, not independent copies. Full pointer-layout preservation for arbitrary mixed heaps without that transform is **not proved** by this small fragment.

No dynamic-base ISA extension is needed for this addressed case; use static placement or this explicit transform, recording it in the reference body so row budgets include it. Native static addressing is cheaper. F/B test the concrete byte and disjointness, including no-wrap checks near the address limit. A new dynamic layout leaf would require §9.2 params/schedule binding and validation before entering Q, but is deferred here. No new frame or fault code; failed logical range checks use 7.

**Verdict: efficiency — static arenas and explicit logical-address translation; no demo rung dependency.**

## G-14: general control inside quantum actions

NS §4.2: “A quantum action whose body contains coherent internal control is, by definition, its bounded padded unitary under the declared bound.” ISA §5.3 lists gates, calls and structured `qif qloop`, but no action `qjump`; §4.6's while selector is “computed before the loop from wires the body does not modify.” Direct arbitrary CFG syntax is absent. A finite bounded gate/event expansion is already an action:

```text
action @two_stages (x:wire) (c:wire in) () () ()
regs { } bounds { 2 }
^entry():
qloop 2 {
qif c {
h x
} {
s x
}
}
```

For c=1 this is H²=I; c=0 it is S²=Z. The adjoint uses H or S† at each reversed iteration. Both branches are charged, even on a basis c. For a bounded controller, unroll K steps and enumerate its finite labels and opcode choices with immutable per-step selectors; store predecessor/selector history in declared mutable wires where needed. Size O(K·n·g) for n candidate rows each expanding to at most g gates; path-specific static expansion can be larger. This is expensive but finite.

**Important limit of the argument.** It is not valid to run H-containing actions and then apply the classical oracle's prefix reversal to erase their controller: that reverses the desired gates too, or leaves changed dependencies. If a selector depends on data a later H changes, §6.2 closes its window. Such control must meet injectivity/synchronisation, retain declared history, or have an independently proved cleanup. Declaring new action jumps does not solve this. General QCM cleanup for every front-end CFG is **unverified here**; no sentence of 0.8 forbids a valid finite unitary gate expansion. The absence of jumps is therefore not evidence of a capability gap. C/D/F can test two_stages, its controlled adjoint, and reject an active-selector mutation. No new 0.9 instruction or fault semantics is proposed. A padded action has no implicit oracle fault; any fallible computation remains a called oracle with declared fault wires.

**Verdict: efficiency — structured bounded action expansions exist; general controller-cleanup claims require proof; no named rung requires an action step-machine opcode.**

## G-15: payloads wider than 128 bits

NS §0 promises “Every program”; §4.2 says “declared outputs are XORed into caller-supplied targets.” This includes multi-field results, although rungs 4/7/8 need only predicate bits. ISA §3.3 bounds integer scalar widths to 128; §4.2's `lift.retval v` XORs one result_val. A direct wide scalar return has no such row. Re-evaluating the same source for each slice realises wide results.

The outline's scratch-fault `qxor` cleanup would additionally have to satisfy §6.2's exact-sequence rule. A simpler construction avoids **all private fault cleanup**. Give every slice variant an identical row-count prefix, identical faults, and one final retval selecting its slice. Call all k variants with the **same caller-owned fault register**. Each contributes code c, so the result is fr⊕(k mod 2)c. If k is even, call one extra variant with zero payload and the same c. Thus k calls for odd k, k+1 for even k, always the required fr⊕c. No scratch fr is freed. Caller targets are not inputs to the computation, so prior slice writes do not affect later c.

Concrete 4-bit payload from two 2-bit slices, f(x)=(x, x⊕3): all three oracles share this five-row reference body and differ only in the operand of retval (lo, hi, or z):

```text
oracle @wide_lo (x:wire[2] in) (t:wire[2]) (f:wire[4]) () ()
regs { mask:reg[2], lo:reg[2], hi:reg[2], z:reg[2] }
^entry():
mask = lift.const.i2 3
lo = lift.mov x
hi = lift.xor x mask
z = lift.const.i2 0
lift.retval lo
```

`wide_hi` returns hi, `wide_zero` returns z. The enclosing canonical action fragment is:

```text
perm_action @wide (lo:wire[2], hi:wire[2], f:wire[4]) (x:wire[2] in) () (a:token, b:token, c:token) ()
^entry():
oracle_call @wide_lo a (x) lo f () ()
oracle_call @wide_hi b (x) hi f () ()
oracle_call @wide_zero c (x) lo f () ()
```

Tokens a,b,c have identical tuples validated for their respective callees. At B≥5, lo⊕=x, hi⊕=x⊕3, f unchanged. At B<5, payloads are zero and f⊕=1⊕1⊕1=f⊕1. For every initial lo,hi,f this is the required wide XOR embedding; reversing the action reverses calls in order and restores everything. General cost is k or k+1 complete bounded computations, peak scratch of one variant plus all interface targets. To retain **the same F_B**, return selection/serialisation work must have equal row counts across variants; padding or different wrapper epilogues cannot silently change the intended budget. The reference source mapping must account for that common prefix.

C tests all 4·4·4·16=1024 interface configurations at B=4 and 5, including target/fault seeds. F checks equal signatures and exact permutation gates. This construction needs no new opcode, frame, code or hash changes to existing examples; a future multi-location retval is an optional efficiency change.

**Verdict: efficiency — sliced variants plus shared fault parity realise wide payloads on 0.8; no scalar-predicate rung dependency.**

## G-16: fibers, nonlocal exits and interrupts

NS §8 rung 6 promises kernel boot/unboot and “tape replay byte-identical”; rung 5 includes Julia runtime idioms; §9.11 lists signals and setjmp as open mechanics. ISA §3.2 puts the “activation stack” in the classical environment, and §4.1 has no stack-switch instruction. It does have loads/stores, br, call and ret with reversible frames. A transformed fiber is an explicit continuation record, not a replacement of the executor's hidden activation stack.

Concrete two-continuation R dispatcher (resume/current slots are live globals; their addresses come from ordinary integer constants/inttoptr in its prologue):

```text
^dispatch():
which = load.i8 current
zero = const.i8 0
is_a = icmp.eq which zero
br is_a ^a ^b
^a():
one = const.i8 1
store.i8 current one
br ^dispatch
^b():
ret
```

The a→b switch is recorded by the store's displaced byte; reversal restores current=0. Real functions spill locals and a resume label to memory and unwind to the dispatcher; rewinding reloads frames and branches to resume blocks. For a switch crossing depth d and saving L bytes, a scalar implementation needs O(d+L) rows and O(L) stored bytes plus its ordinary history, with exact constants determined by the transform. A hand count for the printed a→b instance is 6 instructions in dispatch/a (load,const,icmp,br,const,store), one branch, then four dispatcher instructions and ret = 12, excluding the prologue. It is not a hidden constant-time switch.

Interrupt delivery is a classical poll at a defined materialised point, reading a deterministic timer/pending-state model, storing CSRs and calling a static-dispatched handler. The program's native comparison build must use the same platform model and poll schedule. Claiming equality to arbitrary asynchronous wall-clock hardware timing is **unverified and not implied**. Let P be poll points reached and h handler invocations; overhead is P·cost(poll)+Σ cost(handler_i), with memory/effect writes retained for reversal. Call-site setjmp/longjmp can similarly propagate an explicit frame token and unwind flag; only jumps to active frames are defined source executions. No need to name an unnameable old SSA activation.

The outline ratifies rewindable frames plus polling, so old beads “blocks rung 6 unless accepted” is now stale as a scope statement. The xv6/Linux platform inventory, assembly shims and independent QEMU comparison are still **unverified implementation work**, not an ISA blocker. A tests ping-pong current/resume records and reversal; A/G tests clock/effect state at poll entry; conformance for actual kernels remains the platform spike. No native fiber/interrupt opcode is proposed for 0.9, and no new frame/fault/hash rule is needed.

**Verdict: efficiency — explicit continuations and defined polling preserve the kernel/runtime capability; 5/6/7 need the transform, not an ISA fiber opcode.**

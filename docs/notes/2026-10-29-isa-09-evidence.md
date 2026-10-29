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
| New defect: SCC group ordering | wording-only | linked recursive programs | — |
| New defect: empty data spelling | wording-only | no specific rung | — |
| New defect: explicit fault code zero | gap: make the failure observable | oracle fault contract | V |

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

The smallest 0.9 action is **no semantic change**: use joint actions. Clarify §3.3: “The slice lo operand may be an unsigned classical i64 value; lo+len is checked without wrap. With literal len the result has qreg[len]. The parent becomes live when the forward region call completes; reversing that call restores its pre-call view state before the slice frame is undone.” This is proposed wording for an underspecified operand kind; conservative 0.8 producers can dispatch literal lo values meanwhile. `qreg.slices` would be optional efficiency work; no new multi-view instruction is proposed in this review. A/C/F tests compare all 56 two-bit choices in an eight-bit parent, reject overlapping controls/targets and a second simultaneous view. No new faults/history beyond existing code-4 handle failure and view frames. Existing hashes unchanged.

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

Write an arbitrary joint input as |0⟩|e_0⟩+|1⟩|e_1⟩. Outcome m has probability ||e_m||² and post-state |m⟩|e_m⟩ after normalisation. Destructive measurement followed by fresh |0⟩ and conditional X gives **the same instrument on data and reference**, not just the same marginal probabilities. Replace the dialect handle's id with out; any handle table must update that root consistently (BLK-1 for general dynamic tables). This costs one measurement, one clean wire and at most one X, with 4 instructions on outcome 0 and 6 on outcome 1. General n≤128 uses a fixed preparation action with m's bits materialised from args; arbitrary-width partial observation uses a one-bit CNOT copy into a declared output register, then measurement, leaving the source collapsed. That output is not private workspace freed by the copying action.

Neither construction enables reversal across measurement: §7.4 and §10 still exclude it. No new opcode, frame, fault code, identity change, or normative measurement rule is needed. A/G test outcome 0/1 and consumption; D tests an entangled reference state, with the conditional reference states above as expectations. No particular demo requires retaining a measured qreg.

A stronger variant preserves the original **launch id**, avoiding even the logical handle update above:

```text
fn cc @measure_same (q:qreg[1]) (m:i1) () () ()
^entry():
copy = qreg.new any 1
action_call @basis_copy (copy) (q) () ()
m = measure copy
ret m
perm_action @basis_copy (t:wire) (q:wire in) () () ()
^entry():
cx q t
```

The copy is a declared mutable interface target, so the action returns with no unclean private workspace. Measuring it applies the same projector P_m to the original q and any reference; q's handle stays live with its original id and all aliases. Cost: four CC instructions, one CX, one temporary clean wire and one measurement. This is the preferred lowering when identity comparisons or stored aliases are observable. It also supports partial observation of larger registers through an action on the whole parent, without measuring a view.

**Verdict: already-realised — destructive measure plus exact re-preparation reproduces the instrument; no rung blocker.**

## ISA-7: trap causes and explicit classical faults

NS §0 promises “a named fault outcome … never a silent wrong answer”; §4.2 names distinct causes. ISA §4.1 says `udiv` gives “CC trap; R fault record” and `trap` likewise; §10 requires launch faults to be “codes of §5.4.” §7.1 explicitly gives allocation/free codes and §17.14 gives unclean-free code 4, but the missing CC cause mapping lets implementations disagree. §4.8 currently confines `fault` to Q oracle/lproc rows. A `hostcall bennett.fault` can export a requested code before a trap, but does not define `bennett_result_fault`'s code and is CC-only. Thus it is not a full realisation.

**Exact edit** (§4.1/4.8, §5.4, §7.4, §10):

> In CC and R a failing instruction validates before mutation and appends one terminal fault record. Its cause is: zero divisor → FAULT_UB_DIV0 (5); signed MIN divided or remaindered by −1 → FAULT_UB_SDIV_OVERFLOW (6); invalid classical load/store/free or pinned free → FAULT_UB_MEMFAULT (7); allocation capacity/table exhaustion → FAULT_BUDGET_HEAP (2); invalid handle state, call binding, segment uncall, or explicit trap → FAULT_UB_TRAP (4); device failures use §7.3's named causes. The record is returned unchanged by bennett_result_fault. No result values are exposed on failure.
>
> `fault C` in CC/R takes a literal unsigned 64-bit code and appends that terminal record without other state mutation. It is impure; reversal removes the record and restores the pre-fault control position. In Q its existing F-bit literal rule and row semantics are unchanged. Code 10 names entropy exhaustion in virtualised coherent state as well as in R; remove the parenthetical “R only.”

The unsigned width is derived from the existing `uint64_t bennett_result_fault`, not a new numerical cause assignment. The newly found zero-cause defect is repaired by the effective-cause rule below; consumers must still distinguish terminal status from ordinary results. For R instrumentation `fault 1`, `fault 2`, `fault 3` now report exactly those causes. A/G tests division by 0→5, −128/−1 at i8→6, null load→7, exhausted allocator→2, trap→4, explicit fault 1→1, each with unchanged pre-fault data. B tests a virtualised exhausted entropy oracle's `fault 10`, target unchanged and fr⊕10, all scratch reversed. F checks literal width/tier schema. Function text containing new classical faults changes its hash; the cause mapping is semantic, so V applies to 0.9 publication. No new fault numbers.

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

## I1: checkpointed and pebbled oracle realisations

NS §4.2: “a checkpoint of a coherent region is a point from which it is recomputed”; §5: “The compiler chooses adders, pebbling … under a space budget.” ISA §5.2 explicitly allocates “B frames”; §5.7 permits transformations producing “the same final interface tuple”; §9.4 nevertheless describes retained history as “B × frame width.” §11 binds a versioned expansion library and permits removing only operations that library marks removable. A new replay policy must therefore publish an operation set and account for every replay; final-tuple agreement alone is not a cost specification.

**Construction/proof, without a complexity slogan.** Let F be a total classical step on an S-bit configuration, stuttering at its terminal state. Let P_0(a,b) perform b⊕=F(a), preserving a and cleaning its private scratch. Define

`P_{j+1}(a,b) = P_j(a,c); P_j(c,b); P_j(a,c)`, c initially zero.

After the first call c=F^(2^j)(a), the second XORs F^(2^(j+1))(a) into b, and the third clears c. The first and third are the identical XOR oracle, not a copy of an unknown quantum state. On a superposition the configuration copies are entangled CNOT copies of basis encodings; the final uncompute removes that entanglement. With k=ceil(log2 B), pad F to 2^k terminal-stuttering transitions. Base-call count obeys T(k)=3T(k−1), T(0)=1, so T=3^k=O(B^(log2 3)), **not** B times a polylogarithm from this derivation. There are at most k temporary S-bit checkpoints plus the root/output, and base scratch. A compute/copy-result/uncompute implementation doubles that count. S itself may grow with the program/budget (for example the monotonic stack); O(S log B) is not O(log B) qubits.

A canonical two-step instance with F=inc8, one token bound to the total B≥3 oracle, and two caller-owned fault registers is:

```text
perm_action @inc_twice (b:wire[8], f0:wire[4], f1:wire[4]) (a:wire[8] in) () (tk:token) ()
^entry():
c = qalloc any 8
oracle_call @inc8 tk (a) c f0 () ()
oracle_call @inc8 tk (c) b f1 () ()
oracle_call @inc8 tk (a) c f0 () ()
qfree c
```

No intermediate call touches a or f0, so the first/third are a matched exact cleanup of c. For arbitrary b and fault seeds, b⊕=a+2 modulo 256, both fault registers unchanged, c=0. Each base call has 3 forward+3 inverse machine steps; three calls have 18 such steps, not “two constant-cost operations.” General checkpointing uses the full step function and its resource cost, not inc8's cost.

**Smallest exact 0.9 proposal** (§5.7, §9.4, §11, §14.3, §17.7):

> An expansion library MAY specify a checkpoint/replay or pebbled realisation of §5.2. It MUST name every generated allocation, copy, forward evaluation, inverse evaluation and release in a deterministic order, including every repeated evaluation, with distinct generated ordinals and scratch identities. The schedule binds that library's version and hash; all replay events are executed and charged. The reference body, F_B and budget unit are unchanged. The library MUST supply an exact permutation equivalence argument with clean private scratch on the full domain D; interfaces and borrowed correlations obey §5.2/5.6.
>
> The frame descriptor reports the realised retained history and checkpoint wires from the bound expansion; B×frame_width describes the full-history reference expansion. The default remains the full-history ref library. T2 applies to that reference expansion; a checkpointed expansion establishes the same postcondition by its checked equivalence and its compute/uncompute construction.

This uses the existing expansion binding rather than inventing an unrelated schedule kind. §11's preservation rule continues to apply to each bound library's own operation set. No new instruction/fault/frame semantics; checkpoint live intervals and all replay leaves enter §8.2. C exhausts a toy two-bit increment P_1 (b⊕=a+2 mod4), arbitrary b/fault seeds; E/H require three evaluations, one temporary and its full live interval, and reject an omitted replay charge or unknown expansion hash. The recurrence proves only the stated tradeoff. The beads claim “polylog time” for O(log B) checkpoints is **not verified** and is not adopted. Hash impact V if the transformation/schema wording becomes normative 0.9; original reference_program_hash body remains unchanged apart from versioning.

**Verdict: efficiency — explicit reversible pebbling exists; publish its exact expansion and cost binding, not an unsupported polylog-time rule; 4(c)/7(c) space only.**

## I2: recovering bounded data-dependent loops

NS §4.2 calls structured loops “optimisations … where the CFG is structured”; §4.3 warns a step machine costs “the budget times the block count unless the specialiser recovers structure.” ISA §5.7 explicitly names “structure recovery,” while §4.6 requires the while conditions be “computed before the loop from wires the body does not modify.” A Collatz exit computed from its evolving state is not such a while selector. Nor can an active `still_running` wire be updated under its own predicate: §3.3 makes controls distinct from targets.

There are legal constructions. Unroll K candidate iterations, compute each selector c_i as a function of the **immutable initial inputs** (by a total prefix oracle), run that iteration under c_i, then re-run the selector oracle to clean it. If a prefix evaluation costs at most r·i rows, selectors cost O(rΣ_i i)=O(rK²) rows in total, with no dispatch over unrelated program rows. This proves expressibility, not an efficient universal loop recovery. A better transform retains per-iteration predecessor/condition history and proves its later reversal, or recognises a structure-specific loop invariant. The selector must be pushed only while it stays unchanged. A final oracle embedding still reproduces **the original** row-budget fault boundary, including partial-loop exhaustion.

Concrete immutable-selector fragment, costing one controlled CX per iteration and no private history:

```text
perm_action @repeat3 (t:wire) (c:wire in, s:wire in) () () ()
regs { } bounds { 3 }
^entry():
qloop 3 {
qpred c
cx s t
qunpred c
}
```

Each CX has c as effective control; there are three CCX-equivalent controlled flips, so t⊕=3cs mod2=t⊕cs. There are **three gates** total; push/pop carry no independent unitary. This example pins predication legality; the general prefix-oracle construction explains evolving conditions. Dispatch-free cost is Σ_i gate_cost(body_i+condition_i), whereas literal dispatch includes every static row expansion at every B step. There is no target-independent numeric reduction ratio.

**Exact explanatory addition** (§5.7, no new opcode): “Bounded loop recovery may replace reference dispatch by structured predication and retained condition/predecessor history. Every active selector remains unchanged until its pop; cleanup restores its dependencies. The transformed construction MUST preserve the original reference-row budget boundary, first fault, arbitrary target/fault XOR behavior and full-domain cleanup.” C compares K=0,1,2 and budget cuts inside the loop with the literal machine; F rejects flipping a pushed selector. This is an existing T6 obligation stated concretely, with no identity/fault/history-format changes.

Rung 4(a)/(b) can execute the reference by branch enumeration. Rung 4(c) can report an enormous but honest reference estimate; usefulness of a smaller estimate depends on implementation work and proof, not a new ISA loop. The SQLite report's observed invariance is evidence at measured inputs, not a universal static bound.

**Verdict: efficiency — structured recovery is permitted; immutable selectors or retained history are required; 4(c) cost depends on the implementation.**

## I3: provenance of B and bounded wrappers

NS §3.1: “No per-loop source annotation is ever required”; §9.4 calls for “out-of-band budget tables; scalar-evolution refinement.” ISA §5.4 says “B is enforced by running exactly B steps”; §10 already mints host tokens. Compiler-inferred, table-supplied and host-supplied bounds all supply the same tuple. The origin of a value is not an extra quantum semantic rule. Device-computed values are BLK-2.

Concrete bounded wrapper fragment (K:i32 classical arg, target width 32):

```text
oracle @bounded_counter () (t:wire[32]) (f:wire[4]) () (K:i32)
regs { i:reg[32], one:reg[32], c:reg[1] }
^entry():
i = lift.const.i32 0
one = lift.const.i32 1
qjump ^test
^test():
c = lift.icmp.ult i K
qbranch c ^body ^done
^body():
i = lift.add i one
qjump ^test
^done():
lift.retval i
```

There are 3 entry rows, K successful tests (2 rows), K body/backedge pairs (2 rows), one unsuccessful test (2), one retval: **4K+6**. i never increments above K, so even K=2^32−1 does not wrap; the uint64 B API can represent 4K+6. For B≥4K+6, F_B=(K,0); for smaller B, no successful halt and F_B=(0,1). Registers total 65 location bits; n=8 rows, HALT=8, P=4; there are B frames of width 2P+max_assignment_slot+1=8+32+1=41 bits under the reference per-slot convention, with no flag/succ allocation elision asserted. Stack/heap/depth limits are zero because it is memoryless. The result_val has 32 bits in addition to the locations; do not count it as a third scalar location or forget it.

For a kernel, replace the body with K units of modelled work and return a panic flag; **K is the program's bounded-work parameter, B bounds the complete wrapper trace**. A wrapper reaching its return at B is success, not budget exhaustion. Changing B while leaving K fixed changes only operational F_B; changing K changes the bound/parameterised function semantics and its parameter binding. No inferred bound may omit prologue, calls, boundschecks or output delivery rows. C/B test K=0 (B=6 succeeds, B=5 faults) and K=2 (B=14 succeeds, B=13 faults). No new ISA sentences beyond referring the dialect to these existing obligations; no hash change.

**Verdict: already-realised — budget provenance belongs in dialect/driver documentation; rungs 2/4/7/8 need complete bounds, not a new ISA value.**

## I4: logical run history versus checkpoint/replay storage

NS §4.2 names “history layers, checkpointing and pebbling policy”; ISA §3.2 and §7.4 say “one frame per executed instruction” and “dynamic instances … retained in the run history until reversal.” The logical model is clear; requiring an executor to allocate one physical record per logical frame would contradict the explicit policy freedom. This is distinct from quantum oracle pebbling (I1).

Construction: take a classical checkpoint every c instructions, retain deterministic input/effect replies and logical segment/instance identifiers, and reconstruct the next ≤c logical frames by replay before reversing that block. Over a straight trace of L instructions and state size S, naïve full checkpoints cost ceil(L/c)·S plus at most c·d bytes of reconstructed deltas, where d bounds a logical frame. Replaying each block once costs at most L extra forward instructions before the ordinary L reversals. This is one concrete policy, not a claim that every control/effect trace costs exactly 2L. Effects are **replayed from the tape**, not observed again; host requests are not reissued to reconstruct old bytes. Quantum frames retain their register ids/inverses: reconstruction must not duplicate a quantum state or execute an extra physical Q call merely to recover its classical metadata.

Canonical trace fragment:

```text
r = call @f (a)
kept = xor r zero
uncall @f (a) (r)
```

The xor frame is pure, so it may keep r's value while f's segment is cancelled. Reconstructed history must retain that cancellation state: doing f again during whole reversal would duplicate its clock or quantum effects. **Exact editorial addition** (§7.4): “Frames and dynamic instances specify logical history. An executor MAY store them by checkpoints, deltas or deterministic replay, provided it reconstructs the same logical instances, cancellation markers and reversal order without repeating external effects or copying quantum state, and satisfies T4.” Add the active/audit distinction from I7. A tests a checkpoint spanning call/copy/uncall with one clock read and requires c=0 after full reversal, not −1 or 1. No new instructions, faults, identity bytes, or physical checkpoint wire rule. Rungs 3/5/6 need exact reversal; this changes only their storage policy.

**Verdict: wording-only — logical history permits a verified checkpoint/replay representation; no new rung capability.**

## I5: done-invariant idle steps

NS §4.2 requires prefix reversal and synchronised halt; it does not require a lower variable-time algorithm bound. ISA §3.6 step 3 explicitly says “Under at_HALT: XOR HALT into pc'” and step 4 performs both exchanges. Starting from pc=HALT=n, pc'=old_pc=0, it leaves old_pc=n, so an idle step is **not** the identity on a clean frame. Existing T2 still holds because the inverse clears that record. This independently confirms the budget note's correction and disproves its earlier broad idle-frame-zero sentence.

Merely removing the HALT successor and controlling both exchanges is insufficient: an active step from pc=0 to HALT could leave old_pc=0, indistinguishable from an idle frame. This loses the inverse's selector. One extra retained active bit would fix that, but is unnecessary. Existing P-bit history has a spare code because 2^P>n. Encode an **active old index** k<n as `k XOR HALT`; this is nonzero, while an idle frame stays zero. This reuses the same width and works even when n is a power of two; XOR is a bijection and k≠HALT.

**Exact 0.9 replacement rule** (§3.6, §11 reference expansion, §14.3 new T7):

> At forward-step entry compute a=(pc≠HALT) into clean scratch and dispatch only k<n. Run rows as before. The successor XORs for k<n and both pc commit exchanges run under a; there is no HALT successor XOR. After the exchanges, XOR HALT into frame.old_pc under a. Clean dispatch by comparing frame.old_pc XOR HALT to k, controlled on a. Clear a by recomputing (frame.old_pc≠0). Thus old_pc is zero for an idle step and otherwise encodes the old pc XOR HALT.
>
> For the inverse, compute a=(frame.old_pc≠0). Recompute dispatch under a from frame.old_pc XOR HALT. XOR HALT into frame.old_pc under a, then perform the two inverse commit exchanges under a. Undo the k<n successor XORs and rows in the existing inverse order. Clean dispatch from restored pc under a, and clear a from (pc≠HALT). No HALT comparator is reconstructed from an idle zero frame.
>
> T7: On the subspace pc=HALT with a fresh zero frame and scratch, a forward step is the identity on all quantum machine and interface state and leaves that frame and scratch zero. The external classical step counter still advances. The inverse obeys the same identity on that subspace.

**Proof.** Idle gives a=0 and all row selectors zero, so no write occurs. Active k yields old_pc=k⊕n≠0; inverse recovers a=1 and k, decodes before exchanging, restores pc=k and clears pc' and old_pc exactly as the old inverse did. Row flag/succ uncomputes use the restored old state as before. Finally pc≠HALT clears a. This is a permutation with its printed inverse, satisfying T2; reference row count, first faults and commit F_B are unchanged. The done projector is Π_done=|HALT⟩⟨HALT|_pc⊗I; T7 establishes U_tΠ_done=Π_done on the fresh-frame domain. No new fault, frame bits, register-width inequality or budget formula is introduced. Expanded compare/XOR controls change resource counts and must be in the versioned ref expansion.

B tests idle at n=1,2,3,4, active pc=0→HALT and a fault→HALT, every old frame becoming clean on inverse. For n=1, active pc=0 writes encoded old_pc=1; idle pc=1 writes 0. Include a coherent superposition of these branches; amplitude comparison must agree, with no implicit relative phase. C compares all target/fault seeds and F_B to 0.8. Hash impact V if adopted (new reference realisation even with equal final F_B). An optimiser could already use this as a §5.7 transformation; making it the reference is an efficiency/readiness choice, not a rung blocker.

**Verdict: efficiency — a correct identity-idle reference variant exists with no added history width; no demo rung requires T7.**

## I6: step-granular access and variable-time search claims

NS §5.1 promises “Grover, amplitude estimation and friends … state-vector simulation at small instance size”; NS §9.4 leaves control/budget mechanics open. ISA §4.7 exposes whole region calls and §5.2 runs exactly B steps per oracle call; there is no public start/advance/finish state handle. That prevents directly invoking an **internal** prefix as a reusable instruction, but not algorithms assembled from total bounded oracles.

Concrete ladder source has T(0)=3,T(1)=4:

```text
oracle @ladder_source (x:wire in) (t:wire) (f:wire[4]) () ()
regs { z:reg[1] }
^entry():
z = lift.const.i1 0
qbranch x ^slow ^fast
^fast():
lift.retval z
^slow():
qjump ^last
^last():
lift.retval x
```

At B=3: x=0 gives payload 0,code0; x=1 gives payload0,code1. At B=4: payload=x,code0. An action can call these with host-minted tokens and distinct declared fault outputs, then their inverses:

```text
perm_action @ladder (t:wire, early:wire[4], full:wire[4]) (x:wire in) () (a:token, b:token) ()
^entry():
oracle_call @ladder_source a (x) t early () ()
oracle_call @ladder_source b (x) t full () ()
oracle_call @ladder_source b (x) t full () ()
oracle_call @ladder_source a (x) t early () ()
```

Between the middle calls an algorithm may apply phase/amplification operations while respecting cleanup windows. The printed round trip is identity on arbitrary interface seeds. Its call budgets sum to 3+4+4+3=14, so **28 forward/inverse machine steps**, plus four commit/initialisation sequences. A ladder with multiplicities m_j costs Σ_j m_j·C(B_j), not Σ_i T_i² by fiat. Here sqrt(3²+4²)=5 is a reference running-time norm, not the circuit's derived cost. No multiplicities realising an Ambainis bound have been established by this toy example.

The budget note's claim that token ladders automatically reproduce the general ℓ2 bound, and its “no circuit … less work than the slowest input” as a universal circuit lower bound, are **unverified and not adopted**. Algorithm-specific structure/partial evaluation can defeat such a blanket lower bound. A paper citation or token ladder alone supplies neither an implemented amplitude-amplification schedule nor its proof. Automatic loop composition and a reusable step-state ABI are deferred mechanics, not a capability removed from any rung. No new ISA instruction is proposed without its state ownership, controlled inverse, halt and finish-commit contract. C/H can already test this ladder, exact thresholds and charges. Existing published hashes are unaffected; these would be new fixtures.

**Verdict: efficiency — whole-oracle budget ladders are expressible; step API and claimed general ℓ2 construction remain unverified; no rung dependency.**

## I7: the sixteen council round-8 suggestions

NS §4.4 promises public “certificate schema,” “serialisation and versioning rules, and the conformance suite”; §5.1 requires proof versus evidence. These are cross-rung contract hygiene, not new program capabilities. Read each `bd show b-isa-spec-zsw.N`; no suggestion is treated as proof of its own correctness. Decisions: **11 accept, 1 reject, 4 defer**. The exact proposed edits for accepted suggestions follow. None changes function envelopes locally; a later semantic full-version bump changes every republished example as explained above.

| Bead suffix | Decision | One-line derivation |
|---|---|---|
| .1 | accept | §4.1 omits restrictions already normative in §7.4; use one authoritative traversal. |
| .2 | accept | §11 defaults create invocations with no explicit schedule entry; contexts must enumerate actual invocations. |
| .3 | accept | §7.4 retains marked frames, so T4 cannot restore the physical audit array to its old length. |
| .4 | defer | A complete controlled target/schedule/certificate fixture needs bound expansion bytes; the small rigorous witness below is ready but does not supply that whole fixture. |
| .5 | defer | Bibliographic verification of the proposed 2013 §3.2 reference was not performed; no unverified citation is promoted to normative text. |
| .6 | accept | Exact positive definiteness alone need not give strictly positive interval pivots at finite precision. |
| .7 | accept | Same context defect as .2; sorting only explicit entry paths can omit default-covered calls. |
| .8 | accept | §10 excludes measurements; T4's unqualified “every finite valid CC/R trace” does not. |
| .9 | accept | Same missing summary restrictions as .1, including unverified frees. |
| .10 | accept | A marker is always skipped; “retired” must not imply re-executing its cancelled call. |
| .11 | reject | Acyclic wrapper hashing gives inc8=5f3e…8ee8, contradicting published 8ce5…be93. |
| .12 | accept with correction | Use certified residual margins and actual interval pivots; 2^−precision_bits is not a universal error bound. |
| .13 | accept | State the input-first trace convention; transposition ranges over the same density matrices. |
| .14 | defer | ref needs a complete published, hashed operation/ordinal library; the current prose is not that artefact. |
| .15 | accept | A four-CX dirty frame computes t⊕x and restores b on every basis input, including under inherited control. |
| .16 | defer | §8.3 has no topology schema and §9.1 rejects unknown fields; routing cannot acquire a default from an undeclared field. |

**.1/.9 exact §4.1 replacement for the uncall summary:**

> Segment uncall is legal exactly under §7.4, including equal boundary arguments/results, a pure suffix, no measure at any nesting depth, and no unverified qreg.free in the segment. It uses §7.4's cancellation-aware history traversal; already reversed frames and uncall markers are skipped. Otherwise it traps/faults with FAULT_UB_TRAP and no state change beyond the fault record.

The last phrase harmonises the old “no state change” with its required fault record, not with partial undo. A tests measured nested segments, unverified frees, impure suffixes and nested cancelled clock reads.

**.2/.7 exact §12.1 replacement for context enumeration:**

> Invocation contexts are enumerated from the root realisation's complete invocation tree, including invocations selected only by defaults, not from schedule entries. A context key is its concrete outermost-first invocation chain with resolved loop iteration and oracle-step indices and classical arguments. Enumerate distinct keys in bytewise order of their JCS encodings. A subject claim checks every matching context; its bound is their common upper bound. Wildcard selectors are resolved before this enumeration.

This also disambiguates repeat invocations at one point. D/H fixture: an action calls A twice, one explicitly scheduled and one defaults-only; a claim with a witness for only the explicit call rejects. Two complete witnesses accept only when both bounds pass. No invocation is silently omitted.

**.3/.8 exact §7.4 and T4 edits:**

> Active history consists of unreversed semantic frames; reversed frames and uncall markers may remain as immutable audit records. Undoing a defining frame removes its dynamic instance from the active environment, even if its audit record is retained.
>
> T4: For every finite CC/R prefix eligible for reversal under §10, containing no measure and no unverified quantum release, traversal per §7.4 restores the active environment, memory, allocation table, device-model state and active history to their entry values. Retained marked audit records need not be deleted. Observed external events are not retracted.

The prefix formulation includes ISA-8's eligible failed traces. It does not claim that uncall markers are reversible effects or that failure after measurement can be undone.

**.10 exact §7.4 replacement for the parenthesis using retired:**

> An uncall marker remains a skipped audit record; retiring its segment removes no active effects a second time and never re-executes or redoes the segment.

A's nested f/g clock fixture must leave c=0 after uncall f and after subsequent whole reversal. Audit records may grow; active state must not.

**.6/.12 exact §12.1 addition:**

> The stated δ perturbation assumes an exactly feasible witness. For an approximate witness, first certify lower bounds −η_+, −η_− and −η_s on Y−J, Y+J and μI−Tr_outY. Choose δ>max(η_+,η_−,η_s), set Y′=Y+δI and μ′=μ+(d+1)δ, and require μ′≤bound. These inequalities establish exact positive margins, not an interval-factorisation pass. At the declared precision every actual pivot enclosure MUST still have a strictly positive lower endpoint; otherwise the checker rejects. There is no universal additive 2^−precision_bits allowance.

Derivation: Y′±J≥(δ−η_±)I; Tr_out(δI_d²)=δdI_d, so μ′I−Tr_outY′≥(δ−η_s)I. Interval pivot error depends on conditioning, dimension and actual rounded operations; a single ulp does not bound it. Rigorous ready-to-add D witness: d=2, identical channels so J=0, Y=I_4/8, μ=bound=3/8. Both Y±J have exact LDL† pivots 1/8; Tr_outY=I_2/4, leaving slack I_2/8. All entries are binary rationals; interval LDL† is conclusive with exact representable arithmetic. A witness with Y=0, μ=0 is rejected by the mandated strict-pivot test even though the exact channels agree. This is a test expectation, not a claim that a checker was implemented here.

**.13 exact non-normative §12.1 sentence:**

> With the input-first Choi convention, Tr_out traces the second tensor factor: Tr[(ρ⊗I)Y]=Tr[ρ Tr_outY]. If a primal convention labels this input factor ρᵀ, it ranges over the same set of positive trace-one matrices, so the scalar upper bound is unchanged.

**.15 exact new §16.3 function fragment and result statement** (add to a complete object with its signature and new computed hash):

```text
perm_action @borrow_demo (t:wire) (x:wire in) () () ()
regs { } bounds { 1 }
^entry():
b = qborrow any 1
qloop 1 {
cx x b[0]
cx b[0] t
cx x b[0]
cx b[0] t
}
qreturn b
```

> The action has one loop bound, 1. On every x,t,b∈{0,1}, it maps (x,t,b) to (x,t⊕x,b). V targets only borrowed b with control x; W targets only t with one borrowed control, so the translation lemma applies. Under inherited control w the result is (x,t⊕wx,b). Its adjoint has the same full permutation.

C checks all eight configurations and all sixteen controlled configurations, with an entangled borrowed reference in D; F checks bounds `[1]` and the V/W side conditions. This adds a new vector without changing any existing body/hash. No new fault/history/instruction semantics are introduced by these accepted suggestions.

Deferred .4/.14 remain concrete work: publish ref's exact bytes/ordinals and hash, then attach the full controlled-action fixture bound to them and add it to the precheck before council grading. No claimed target-independent “one step” charge substitutes for that library. Deferred .16 retains the stated logical all-to-all model until a topology contract is designed; no topology-related capability is deleted. The north star names target resource extensions, not an already specified physical routing ABI.

**Verdict: wording-only — 11 accepted exact edits, .11 rejected, .4/.5/.14/.16 deferred; cross-rung contract hygiene, no capability narrowing.**

## I8: blocks versus rows in the index space

NS §4.2 states “The budget unit is steps of the ISA formal-model interpreter” and promises the general predicated machine for arbitrary CFGs. ISA §3.6 calls I[0..n) “the oracle's blocks in text order, each contiguous,” but §4.2 says “Every row is one budget step; k is its own index.” `inc8` has one block with three rows and needs B≥3; treating that block as one index cannot produce k+1 fallthrough or that result. §3.7 records return index k+1. This is an editorial inconsistency with a uniquely pinned row interpretation.

**Exact edits** (§3.6/4.2): “I[0..n) is the oracle's rows in block text order, then the rows of its lproc closure. Each function occupies a contiguous range of row indices; a block label denotes its first row.” In qjumpr's flag definition, replace “indices of the containing function's blocks” by “row indices of the containing function”; label values still come from labels, not arbitrary cross-function control. Keep HALT=n and 2^P>n. No row count, frame width, instruction semantics, fault code or hash changes locally.

Concrete §16.3 inc8: const index0, add index1, retval index2, HALT3, minimum P=2. I3's wrapper has indices 0,1,2 at entry, 3,4 at test, 5,6 at body, 7 at done, HALT8 and P=4. F tests multi-row blocks and lift.label preserving these values; B checks B=2/3 inc8 boundaries. The separate cyclic-SCC ordering defect below must also be specified so linked label values are reproducible.

**Verdict: wording-only — one index per row and labels at block entry; every lifted rung depends on this already-pinned reading.**

## SQLite E1: uniform location-stack cells

NS §4.3 promises honest qubit-time and “coherent allocation … padded to its bound”; rung 4(c) requires “a resource estimate … reporting the budget and the padded cost.” ISA §3.7 defines `R_f=3+m′` cells and `W_cell=max(P,Sc,Did,W_max)`; “The stack is monotonic (sp only grows; records are history).” Recursive return does not reclaim those quantum history cells. Those rules already realise SQLite, but may cost much more than peak live native stack.

The concrete published sum_db example has m′=13, R_sum=16; input n makes n+1 calls. Stack cells=16(n+1), so n=16 needs 272, not 256, and depth≥17. Its lproc base case executes 6 rows and each non-base level adds 14, so total oracle rows=14n+10=234 at n=16 (provided all memory checks pass). The current published test already pins the stack boundary.

For a call record whose padding is proved unused, the occupied payload bits could be `P+Sc+Did+Σ_{ℓ∈L_f}width(ℓ)` instead of `(3+m′)W_cell`. This is a representation transformation, not a change in the logical R_f stack budget. Example dimensions P=4,Sc=4,Did=2,W_cell=8,m′=2 with two one-bit locations: padded record 5·8=40 bits, packed payload 4+4+2+1+1=12. A physical implementation still needs addressing/control scratch, and on a superposition must reserve enough for **every reachable call shape**, not the native trace's average. It must decode to the same logical stack for forward and inverse delivery.

The report's 1,214,248 cells×128 = **155,423,744 bits**, before its 10% margin. A 128-bit W_cell follows **if** the mapped closure actually contains a 128-bit location and P,Sc,Did≤128. No emitted full SQLite ISA object or per-function width histogram was available to verify the proposed “3–4×” saving, so that factor is **unverified**. All seven deliverables of the other interpreter job are outside this job's edit scope; no Rust tests were changed.

No new 0.9 stack semantics are proposed. A bound expansion can pack proved padding under §5.7/T6, with I1's realised-frame reporting if adopted. B/C compare logical snapshots and single-step inverses of differently sized recursive callees; E charges actual bits/live intervals, while reference stack exhaustion remains at R_f sums. No new fault code; `FAULT_BUDGET_DEPTH` still occurs on `sp+R_f>stack_limit`, not on packed-byte occupancy. Existing example hashes unchanged.

**Verdict: efficiency — heterogeneous physical packing may reduce 4(c) space; logical stack semantics already realise SQLite; measured saving unverified.**

## SQLite E2: uniform per-step history frames

NS §4.3: “Garbage carries an occupancy cost per qubit-second.” ISA §3.6 fixes `(old_pc, disp_1..disp_kmax, flag, succ)` per oracle. Its reference frame width is

`W_frame = 2P + 1 + Σ_j W_j`,

where W_j is the maximum width of slot j across the full closure's row uses. Example inc8 has P=2, one assignment slot of width8, hence W_frame=4+1+8=13 bits; result_val=8 and locations=16 bits are separate allocations. At B=3 it retains 39 frame bits; B=2 retains 26, even though the latter call faults. These are directly derived from the printed model, independent of any gate library.

With the report's assumed width 740 and profiled estimated B=289,590, the product is **214,296,600 bits**. At ceil(1.10B)=318,549, it is **235,726,260 bits**, explaining its approximate 235M claim. The 740-bit width itself is **unverified** until the mapped reference body's complete slot maxima are known; 13 arguments totalling 608 bits are not by themselves a derivation of it. At n≈324,000 rows P=19 because 2^18<n<2^19; slot sizing must include every call capture, allocator slot and parallel move, including fault-only paths.

Rows can share physical fields if their complete live ranges are disjoint; after each inverse its frame is zero. Across quantum-superposed row choices at a fixed step, narrower storage requires a reversible tagged/encoded representation, not observing which row ran. Statically proven reachable-row sets can reduce a step's maxima; observed classical traces cannot. General space reduction is I1 checkpointing. Neither changes the original row budget or fault boundary. No new row/frame schema is necessary; a named expansion must enumerate the actual representation and resource charge. B/C compare full decoded tuples and faults at B=0, the exact halt boundary, and early HALT; E/H compare live-interval occupancy. Hash impact none unless the separate I1/I5 proposal is adopted.

**Verdict: efficiency — frame representation is costly but correct; 4(c) may benefit, and no new capability is needed.**

## SQLite E3: indirect-call row

NS §0: “If it compiles with clang, you can put it under a qubit”; §3.2 requires function addresses equal the comparison link's layout. This matters for SQLite VFS/SQL callbacks and rung 5 dispatch and rung 6/7 operation tables. ISA §4.2 has static `lift.call`; `qjumpr` is constrained to the containing function. The latter forbids using a cross-function return address as an indirect call; it does not forbid finite dispatch.

Concrete canonical oracle fragment, with fp:wire[32] and x:wire[8] inputs, target8 and fault4; a,b:reg[32], p:reg[1], r:reg[8]:

```text
a = lift.const.i32 256
b = lift.const.i32 512
p = lift.icmp.eq fp a
qbranch p ^fa ^testb
^testb():
p = lift.icmp.eq fp b
qbranch p ^fb ^bad
^fa():
r = lift.call @add1 (x)
qjump ^done
^fb():
r = lift.call @sub1 (x)
qjump ^done
^bad():
fault 4
^done():
lift.retval r
```

Addresses 256/512 are the concrete comparison-link values of the two candidates. Each lproc has param x:reg[8], result r:reg[8], one:reg[8]; its body const1, add/sub, lift.ret r has three rows and R_f=3+3=6 cells. Totals: candidate add1 executes 4+1+3+1+1=**10 rows**; sub1 executes 4+2+1+3+1+1=**12**; no match executes 4+2+1=**7** and code4. With sufficient budget, x=255 gives 0 for fp=256, 254 for fp=512, and payload0,code4 for any unmatched pointer. One call depth and six cells suffice for either candidate, in addition to any caller closure history.

A balanced tree over K known addresses has O(log K) comparisons and branches on a chosen path, plus equality/no-match handling, the static call/return and **constant initialisation**. The report's `2ceil(log2 K)+1` is an approximation, not an exact universal row formula; e.g. it omits these two constants and an unmatched leaf. All candidate bodies enlarge n, and the literal step machine pays for the enlarged closure at each B step. Type-based candidate soundness across every LLVM ABI is **not proved here**; the report's counts are a concrete inventory, and the dialect's independent validator must establish candidate completeness for its materialised program.

No `lift.callr` is needed for 0.9 capability. A new indirect row would have to define function-address mapping, validated homogeneous signatures, no-match code, SCC closure references, canonical indices and an ordered capture/push/bind/delivery inverse list. That is larger than a claim that `qjumpr` can call. Keep compare trees; add B/C/F tests of both candidates, no match, caller-local target preservation and exact B=9/10 and 11/12 boundaries. No new faults/hash rules.

**Verdict: efficiency — compare-and-branch plus static lift.call already realises indirect calls; rungs 4/5/6/7 depend on that lowering, not a new row.**

## SQLite E4: block copy

NS §3.1 explicitly includes “The ordinary loads and stores of lifted classical code (all of SQLite's)”; rung 2 has byte memory and rung 4's memdb copies pages. ISA §4.2 provides `lift.load`, `lift.store`, arithmetic and branches, with inherent bounds checks. Lack of one block-copy row is not lack of memcpy.

Concrete canonical lproc, ptr_width=32, copying n bytes between **non-overlapping** live allocations:

```text
lproc @copy_bytes (d:reg[32], s:reg[32], n:reg[32]) (r:reg[32]) () () ()
regs { one:reg[32], i:reg[32], c:reg[1], sa:reg[32], da:reg[32], v:reg[8] }
^entry():
one = lift.const.i32 1
i = lift.const.i32 0
qjump ^test
^test():
c = lift.icmp.ult i n
qbranch c ^body ^done
^body():
sa = lift.add s i
da = lift.add d i
v = lift.load.i8 sa
lift.store.i8 da v
i = lift.add i one
qjump ^test
^done():
lift.ret i
```

It has m′=3 params+1 result+6 regs=10, R_f=13 cells. For n=M bytes, 3 initial rows, 2(M+1) test rows, 6M body rows and one return give **8M+6** callee rows, plus one caller lift.call. The largest M=2^32−1 still increments only to n, without wrap. The valid-allocation premise guarantees address addition does not wrap within the accessed range. With explicit dialect boundschecks add 2M. Each store records its displaced byte and prefix reversal restores destination, source, locals and call record even if a later access faults. Copy is not quantum cloning of an arbitrary state: this is the lifted classical assignment embedded in an oracle whose workspace memory is later reversed.

For source [1,2,3,4], destination [9,8,7,6], n=4, forward local destination=[1,2,3,4], r=4 after 38 callee rows; the complete oracle preserves caller `in` regions and only returns its declared output. Inverse restores [9,8,7,6] byte-for-byte. n=0 uses 6 rows and makes no access. `memmove` needs direction selection or a temporary: d=s+1 with source bytes [1,2,3] must produce [1,1,2,3], whereas naïve forward copy produces [1,1,1,1]. A two-pass temporary-buffer implementation is O(M) rows plus M temporary bytes and allocation metadata; it still fits 0.8. No atomic block-copy row or inferred constant gate charge is proposed.

B/C test zero length, non-overlap, overlap both directions via memmove, range failure after a prefix, and inverse displaced-byte restoration. F checks parameter and access widths; E charges all dispatch/table scans from the bound expansion. No new frame/fault/hash rule. The report's 49KB of copy activity is a profile measurement, not a new row count.

**Verdict: efficiency — byte/word copy lprocs already realise memcpy/memmove; optional 4(c) cost optimisation only.**

## SQLite E5: a classical-content hint for input regions

NS §8 rung 4 fixes the database “classically before entry”; §4.3 says quantum-addressed memory costs O(N) gates “unless the target provides QRAM.” ISA §5.2 nevertheless defines D as “every cell of every in region”; §4.6 qload XORs the selected cell and §5.7 allows extensionally equal specialisation. A hint must not silently replace arbitrary region contents by a classical ROM or exclude part of D.

For known four-byte image [0x12,0,0x80,0x03], address a:wire[2], the QROM construction computes e_j=(a==j), XORs the known byte into target under e_j, and uncomputes e_j, j=0..3. E.g. the j=0 fragment after its equality flag e0 is computed is:

```text
cx e0 t[1]
cx e0 t[4]
```

j=2 flips t[7]; j=3 flips t[0],t[1]; j=1 flips nothing. Thus 5 controlled data flips plus four equality computations and their inverses produce t⊕image[a], for every a,t. The general data-flip count is Σ_j popcount(image[j]); equality cost is the actual expansion of two-bit or A-bit comparisons, not a target-independent Toffoli number. To use this as qload's value d, allocate d clean and uncompute by the same circuit before freeing it. A schedule can choose the known-byte implementation only when entry preparation and exclusive ownership establish those exact bytes. If a region may be modified coherently, use an overlay or the general memory circuit. No nondestructive runtime “is classical?” measurement is available.

The hint can be non-semantic metadata or executor provenance; it must leave a valid general path for every other allowed input, with no new fault or rejected source program. The object schema's unknown-field rejection means a producer cannot invent a new normative hint field in a 0.8 object; it can specialise an ordinary data-image object or retain executor-side facts from `bennett_qreg_prepare`. The exact bytes and dynamic preparation/parameter binding must be reproducible in the certificate used for that estimate. No optional new field is proposed here until its binding schema is supplied.

**Report correction, independent of E5.** A `region[25,8]` has 2^25 bytes=2^28 wires. The measured file size 33,492,992 padded to 33,554,432 leaves **61,440 zero bytes**. §7.2 requires its base plus 2^25 within [0,2^H), disjoint from a positive null region, globals and heap. Therefore **H≥26**, not the report's H=18; a positive base prevents fitting in H=25 too. A conformant shape could put null/globals low, a bounded heap below 2^25, and db at base2^25, ending at2^26. Literal mem then has 2^26 byte cells (before any proved optimisation), not 2^18. Hc≥H+2 is at least28. Consequently the report's 1.3e19 literal gate estimate and its 6.9e8-qubit table do not describe a valid §7.2 object. Their recomputed conformant costs are **unverified**; no replacement total is invented.

The report's profiled row weights (FP=150, switch=3, indirect call=9, etc.) and sampled 2^20 maximum are also **not proved ISA budgets**. Its 4e7 structure-recovered estimate uses observed address sets/invariance; full-domain static proofs and a named concrete target/expansion remain outstanding. These limitations do not refute the feasible route-A construction: interface image, ordinary lifted SQLite and a sufficiently large validated tuple already exist in 0.8. They prevent treating a spike estimate as a conformance certificate or a demonstrated zero-fault budget for all 2^20 inputs.

C/D compare the QROM tuple and phases to general qload, including superposed addresses; F/B check H=18 rejection and a valid H=26 mapping. E/H require every comparator/data flip and live interval to be charged. No new ISA semantics, fault code, frame or existing hash change.

**Verdict: efficiency — verified classical-data specialisation is permitted; 4(c) needs sound bindings and corrected layout/cost evidence, not a capability restriction.**

## Additional ISA 0.8 defects found during the review

Three new issues were created; I8, ISA-10, I7's context/history defects and ISA-7/8 already have issues. All three new issues carry `isa-0.9-candidate`. These are text proposals, not repairs made to accepted 0.8.

**b-isa-spec-was — SCC ordering (§3.6).** Exact text says “lprocs ordered by closure hash with SCC members in SCC index order.” SHA-256 member digests are not ordered by member index, so sorting every member's closure hash can conflict with keeping its SCC in index order. Proposed exact clarification: “Partition the lproc closure into SCC groups. An acyclic singleton group's order key is its closure hash; a cyclic group's order key is its closure-envelope scc_hash. Sort groups by those keys, then emit each group's members in SCC index order. Within each function concatenate rows in block text order.” §9.2's name ordering determines the SCC member indices. No new frame/fault/width. F fixture: mutual f,g, member-hash order opposite name order; canonical f-before-g within that group and stable lift.label values, independent of load/link order. No existing example has a multi-member SCC; all four 0.8 vectors remain unchanged. NS §4.4's public linking contract and all recursive lifted programs depend on deterministic labels. Verdict: wording-only; no extra rung blocker.

**b-isa-spec-oim — empty data (§16.1/16.2).** HEX is `([0-9a-f]{2})*`, allowing empty; dataline is `NAME SP HEX NL`; §16.1 forbids trailing spaces. A size-zero image therefore requires and forbids its separator space. Proposed exact sentence: “The mandatory separator space in a data line with empty HEX is the sole exception to the no-trailing-spaces rule.” Keep the existing dataline grammar; do not disallow zero-sized data to escape the inconsistency. F accepts exactly NAME, one space, LF for size0; rejects two spaces and size/image mismatch. The hash includes the mandatory space. No instruction/frame/fault changes or existing example changes. NS §0's unrestricted programs and §4.4 serialisation apply; no rung specifically needs an empty image. Verdict: wording-only.

**b-isa-spec-7qu — unflagged explicit fault (§3.3/4.2/5.4).** `fault C` and `lift.faultif p C` require C<2^F, but not C≠0; §5.4 assigns 0 to FAULT_NONE. For this legal 0.8 fragment:

```text
oracle @zero_cause () (t:wire) (f:wire[4]) () ()
^entry():
fault 0
```

At B=1 the row sets faulted=1 and code=0. §5.2 commits no payload and XORs 0 into fr; the final interface is indistinguishable from a successful zero result for every target/fault seed. This violates NS §4.2's “distinct code per cause” and the fault-clear acceptance contract, despite perfect prefix reversal. **Exact proposed semantic edit:** “For an explicit fault or a triggered lift.faultif, the effective cause is FAULT_UB_TRAP when the literal C is zero, and C otherwise. Both forward and inverse fault writes use that effective cause. The same rule applies to CC/R fault.” Syntax remains accepted and no meaningful fault capability is removed. F=4 already represents code4; no new fault number or width condition. B/C check all target/fault seeds: B=0 gives fr⊕1, B=1 gives fr⊕4 and target unchanged; inverse scratch is clean. With an untriggered faultif, no cause is committed. A/G check proposed CC/R fault0 gives terminal record4. The one-row oracle's history still uses the ordinary fault-row inverse. Existing §16.3 bodies contain no zero-cause fault, but the semantic 0.9 version bump changes their republished hashes. Verdict: gap; fault observability, no additional demo blocker.

## Verification record and remaining evidentiary limits

Read-only baseline: headless `python3 -B reference-interpreter/isa_check.py` passed: two complete published objects parse/hash and execute, both printed envelopes verified. Inline `python3 -B`/hashlib independently checked the direct/wrapped envelope distinction, arithmetic and the constructions listed below. These checks are evidence for this note, not a claim that proposed 0.9 syntax passes the unchanged 0.8 checker. No LLVM command, Rust build, git command, spec edit or interpreter edit was performed by this job.

The following self-contained arithmetic verifier reads the actual printed loop fragments. Run from the repository root with the headless environment and `python3 -B`, extracting this `python verify=isa09` fence. It does not write files or import interpreter code. Its small row evaluator verifies the hand-derived counts; its permutation checks verify full tuples, not only ancilla zero. It is not a replacement ISA implementation.

```python verify=isa09
from pathlib import Path
from itertools import product
from fractions import Fraction as Q
import hashlib, re

note = Path('docs/notes/2026-10-29-isa-09-evidence.md').read_text()
spec = Path('spec/Bennett-ISA-Spec.md').read_bytes()
assert hashlib.sha256(spec).hexdigest() == '90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e'

def fragment(name):
    for body in re.findall(r'```text\n(.*?)```', note, re.S):
        if re.search(r'^(oracle|lproc) @' + name + r' ', body):
            return body
    raise AssertionError(name)

def rows(text, initial, memory, budget):
    labels, code, widths = {}, [], {}
    for n, w in re.findall(r'(\w+):reg\[(\d+)\]', text):
        widths[n] = int(w)
    for line in text.splitlines()[1:]:
        if line.startswith('regs '):
            continue
        if line.startswith('^'):
            labels[line[1:line.index('(')]] = len(code)
        else:
            code.append(line)
    state, mem, displaced = dict(initial), dict(memory), []
    pc = 0
    for count in range(1, budget + 1):
        lhs, rhs = code[pc].split(' = ') if ' = ' in code[pc] else (None, code[pc])
        op, *args = rhs.split()
        if op.startswith('lift.const.i'):
            state[lhs] = int(args[0])
        elif op == 'lift.add':
            state[lhs] = (state[args[0]] + state[args[1]]) % (1 << widths[lhs])
        elif op == 'lift.icmp.ult':
            state[lhs] = int(state[args[0]] < state[args[1]])
        elif op == 'qjump':
            pc = labels[args[0][1:]]
            continue
        elif op == 'qbranch':
            pc = labels[args[1 if state[args[0]] else 2][1:]]
            continue
        elif op == 'lift.load.i8':
            state[lhs] = mem[state[args[0]]]
        elif op == 'lift.store.i8':
            address = state[args[0]]
            displaced.append((address, mem[address]))
            mem[address] = state[args[1]]
        elif op in ('lift.ret', 'lift.retval'):
            return state[args[0]], 0, count, mem, displaced
        else:
            raise AssertionError(op)
        pc += 1
    return 0, 1, budget, mem, displaced

for k in range(33):
    expected = 4 * k + 6
    result = rows(fragment('bounded_counter'), {'K': k}, {}, expected)
    assert result[:3] == (k, 0, expected)
    assert rows(fragment('bounded_counter'), {'K': k}, {}, expected - 1)[:2] == (0, 1)
for size in range(17):
    memory = {16+i: (i*3+1) % 256 for i in range(size)}
    memory.update({128+i: 199 for i in range(size)})
    result = rows(fragment('copy_bytes'), {'d':128, 's':16, 'n':size}, memory, 8*size+6)
    assert result[:3] == (size, 0, 8*size+6)
    assert [result[3][128+i] for i in range(size)] == [memory[16+i] for i in range(size)]
    for address, value in reversed(result[4]):
        result[3][address] = value
    assert result[3] == memory

for x, lo, hi, fault in product(range(4), range(4), range(4), range(16)):
    for budget in (4, 5):
        payload = (x, x ^ 3, 0) if budget >= 5 else (0, 0, 0)
        code = 0 if budget >= 5 else 1
        actual = (lo ^ payload[0] ^ payload[2], hi ^ payload[1], fault ^ code ^ code ^ code)
        expected = (lo ^ payload[0], hi ^ payload[1], fault ^ code)
        assert actual == expected
for x, y, a, control in product(range(4), range(4), range(4), range(2)):
    out, scratch = x, 0
    if control:
        scratch = (x + y + a) % 4
        out, scratch = scratch, out
        scratch ^= (out - y - a) % 4
    assert scratch == 0 and out == ((x+y+a) % 4 if control else x)
for x, t, b, w in product(range(2), repeat=4):
    dirty, out = b, t
    dirty ^= w*x
    out ^= w*dirty
    dirty ^= w*x
    out ^= w*dirty
    assert (dirty, out) == (b, t ^ (w*x))

def pebble(level, a, b, calls):
    if level == 0:
        calls[0] += 1
        return b ^ ((a+1) % 4)
    c = pebble(level-1, a, 0, calls)
    b = pebble(level-1, c, b, calls)
    c = pebble(level-1, a, c, calls)
    assert c == 0
    return b
for level, a, b in product(range(6), range(4), range(4)):
    calls = [0]
    out = pebble(level, a, b, calls)
    assert out == b ^ ((a + (1 << level)) % 4)
    assert calls[0] == 3**level

idle_checks = 0
for halt in range(1, 17):
    for oldpc, oldvalue in product(range(halt+1), range(4)):
        pc, value, pcprime, old, disp, successor = oldpc, oldvalue, 0, 0, 0, 0
        active = int(pc != halt)
        if active:
            successor = (pc + oldvalue + 1) % (halt+1)
            disp ^= (3*value + pc) % 4
            value, disp = disp, value
            pcprime ^= successor
            pc, pcprime = pcprime, pc
            pcprime, old = old, pcprime
            old ^= halt
        active ^= int(old != 0)
        assert active == 0
        if oldpc == halt:
            assert (pc, value, pcprime, old, disp, successor) == (halt, oldvalue, 0, 0, 0, 0)
        active = int(old != 0)
        if active:
            row = old ^ halt
            old ^= halt
            pcprime, old = old, pcprime
            pc, pcprime = pcprime, pc
            pcprime ^= successor
            value, disp = disp, value
            disp ^= (3*value + row) % 4
            successor ^= (pc + value + 1) % (halt+1)
        active ^= int(pc != halt)
        assert (pc, value, pcprime, old, disp, successor, active) == (oldpc, oldvalue, 0, 0, 0, 0, 0)
        idle_checks += 1

image = [0x12, 0, 0x80, 0x03]
assert sum(v.bit_count() for v in image) == 5
for address, target in product(range(4), range(256)):
    out = target
    for j, byte in enumerate(image):
        if address == j:
            out ^= byte
    assert out == target ^ image[address]
for outcome in range(2):
    state = [1, 2j, 3, -4j]
    direct = [v if (i & 1) == outcome else 0 for i, v in enumerate(state)]
    copied = [0j] * 8
    for i, value in enumerate(state):
        copied[i + 4*(i & 1)] = value
    measured = [copied[i + 4*outcome] for i in range(4)]
    assert measured == direct
for target, fault in product(range(2), range(16)):
    for budget in (0, 1):
        cause = 1 if budget == 0 else 4
        assert (target, fault ^ cause) != (target, fault)
assert Q(3,8) - 2*Q(1,8) == Q(1,8) > 0
assert 1214248 * 128 == 155423744
assert 740 * 289590 == 214296600
assert 740 * ((289590*11+9)//10) == 235726260
assert (1 << 25) - 33492992 == 61440
assert 1 + (1 << 25) > (1 << 25)
assert (1 << 25) + (1 << 25) <= (1 << 26)
print('PASS: wrapper/copy row counts and boundaries; 2048 wide-output tuples; parameterised inverse; controlled dirty borrowing; pebbling recurrence;', idle_checks, 'active/idle round trips; QROM; retained-register measurement instrument; nonzero effective faults; rational witness and resource arithmetic; spec unchanged')
```

Source fingerprints at reading (SHA-256):

```text
ISA 0.8                 90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e
budget note             ff076c63a2a2f1d8f25ee66c57f877ab57f838a42a3bf671596a7fee86cb9119
dialect outline         bdcad1ac8ff141f75300f532328c6f04c27465a197fadd0851976801d4eb689c
SQLite REPORT           0daae7a070058f7c0a12614764840e6438727423026340c4c28119ab41e52871
narrowing audit         ee6b60b351f0068e4e75d8c4b3f6b1995d2249822a182f3951ad965d31292a34
north star              48d41e6a0094f6710fca5f01b5601f1a9f721bdcac2612211fb7511712019799
plan                    faa2fbd2eb4523f240fd549120947d8331dfb196a93a34fae9ead0895672f86c
```

Not reproduced here: SQLite source builds/profiles and complete mapped ISA closure, real target gate counts, the generic static taint/control/ABI completeness proofs, an Ambainis amplification schedule, approximate witness acceptance at arbitrary precision, or the 2013 bibliographic attribution. Each numerical rule used in a proposal is derived in its section. Beads notes carry verdicts, not closure claims about the later spec implementation. See the paired progress file for the final verification commands and issue-note audit.

## Proposed ISA 0.9 change list

**Order by rung dependency.** There is no proved missing ISA capability blocking the fixed-shape demo ladder. The implementations and proof obligations supporting rungs 1–8 are given above; this does not assert that their plugin, platform or executors are implemented. First address shared correctness/contracts used by early rungs (nonzero fault outcomes, row indices, acyclic hashing, SCC ordering, call/uncall history and complete certificate contexts). Next prioritise rung-4/7 cost work **outside normative edits**: corrected layout and full mapped budgets, structure recovery (I2), QROM bindings (E5), physical stack/frame accounting (E1/E2), and optional pebbled expansion (I1). Rungs 5/6 require the platform/continuation transform and independent native tests; G-16 is optional efficiency. Runtime-snapshot efficiency (G-12) follows for 7(c). None of E1–E5 warrants inventing a new ISA numerical rule.

**Fewest council rounds: one combined 0.9 proposal**, after dialect acceptance. Group the dependency-ordered contents in that single dossier:

1. Editorial repairs with unchanged local semantics: I8, ISA-10's direct/cyclic distinction, SCC ordering, empty data, I4, accepted I7 wording and its positive borrow vector; call/ret/branch qreg threading and literal/runtime slice-offset wording. New vectors and their expected values must enter the precheck before grading, per spec/README. Include entropy code10's clarified use and the zero-cause effective-fault repair.
2. Small capability contracts, independent of demo feasibility: BLK-1 dynamic handle type/id ledger and concrete schedule binding; BLK-4 R allocation using the same ledger and cleanliness inverse; BLK-2 device minting with immutable per-invocation tuples; BLK-3 parameterised certified inverse pairs plus provenance/cleanup exception. Follow with ISA-7's cause mapping/explicit fault and ISA-8's reversible failed prefixes, using the shared T4 eligibility wording. Their exact sentences and suites are in their item sections. No new numerical fault codes, no LLVM changes, no weakening of any NS clause.
3. Optional efficiency semantics in the **same** dossier if their expansion fixtures are ready: I5's encoded old_pc reference variant and I1's realisation/resource bindings. They are not prerequisites to sending the mandatory capability/wording proposals. Default remains full history until a checked alternative is bound. Publish ref's actual operation set/hash and run each changed/new vector before acceptance; a prose declaration of its existence does not discharge that work.

Use a full 0.9 version consistently and recompute all four republished example hashes; retain 0.8 vectors unchanged. This note deliberately proposes no callr, copy row, overlay, dynamic-base field, CC borrow ABI, fiber ABI or native-memory mapping until evidence establishes the efficiency case and their full cross-repository contracts. Every associated software capability is retained by a construction, with unverified generalisations marked explicitly. Four deferred round-8 suggestions stay open; they are not deleted or relabelled as implemented.


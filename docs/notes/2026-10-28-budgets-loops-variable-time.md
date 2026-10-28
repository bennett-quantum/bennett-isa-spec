# Budgets, data-dependent loops, and variable-time algorithms

Design note, 2026-10-28. Written from the maintainer's questions after the
ISA reached council consensus (draft 0.8), and from a reading of Jeffery,
Mamindlapally and Nguetsa Tankeu, *Loop Composition in Quantum
Algorithms*, arXiv:2605.07518 (8 May 2026). Nothing here changes the spec;
it records what the spec already provides, what it does not, and the open
items that follow. Section references are to `spec/Bennett-ISA-Spec.md`.

## 1. Two different Collatz problems

**Per-input reversible execution (R tier, §3.4, §7.4).** A program with a
data-dependent loop runs as many steps as its input demands, builds a run
history, and is reversed by LIFO traversal (T4). No budget exists in this
tier: the budget is a component of Q-tier state only (§3.2, §5.4). This is
the problem BennettVM.jl solved (`collatz_steps`, SC9 Case D, milestone
M13) and the ISA preserves it unchanged. Checkpoint-plus-replay, delta
logs and pebbling policy are executor choices inside this tier (north
star §4.2); see open item I4 on saying so in §7.4.

**One circuit for a superposed input (Q tier, lifted oracle, §5.2).**
The map |n⟩|0⟩ ↦ |n⟩|c(n)⟩ is a permutation and exists for every finite
register width. A circuit implementing it is one fixed gate list, so it
must contain the gates every input in the domain needs, each controlled
so that inputs which do not need it are unaffected. The reversible
interpreter's per-input *trace* has input-dependent length; the
*circuit* has the length of the longest trace in the domain. Turning the
family of traces into one circuit is exactly the predicated step machine
(§3.6): a quantum program counter, every row executed under "the counter
names me", for B steps, with finished branches idling at HALT. B is the
longest trace over the declared domain. Linearity then gives the
superposed map for free; it does not give a shorter circuit, because no
circuit for this permutation can do less work than the slowest input.

Vocabulary that keeps the two apart: *trace* (per input, variable
length) versus *circuit* (one object, fixed length). The R tier executes
traces; the lifted oracle is the circuit.

## 2. What the construction guarantees at the wall

- Early branches idle at HALT; their result is already in `result_val`
  and they need no fault (§3.6 step 3 under `at_HALT`).
- Branches not finished at B receive target ⊕ 0 and fault ⊕
  `FAULT_BUDGET_STEPS` (§5.2 commit) and still run the prefix reversal, so
  their ancillae are as clean as a finished branch's (T2; north star:
  "fault predicate off, history reversed exactly as on a clean branch").
- On every branch, frames for steps never reached stay |0⟩ (the gates that
  would write them are identities there) and frames that were used are
  uncomputed. After the circuit all ancillae are |0⟩ on every branch, which
  is what lets them factor out of Σ|n⟩|c(n)⟩. During the circuit they are
  entangled with n; this is why nothing may be measured or stopped mid-way.
- If B covers the domain, the fault register stays |0⟩ everywhere and
  factors out (the north star's zero-fault-rate acceptance rule).

## 3. The wrapper pattern (kernel fuzzing, rung 7)

A program that never returns (a kernel) must not be the oracle body, or
every branch faults and the oracle answers nothing. The oracle body is a
*wrapper*: ordinary code that runs the snapshot for K units of bounded
work and returns a predicate (the panic flag). K is part of the function's
meaning and identity; B is the ISA budget, operational, set by the host at
launch (`bennett_budget`, §5.4, §10) to at least the wrapper's longest
trace, and the fault register reports when that assumption was wrong. The
boot itself runs in the R tier, unbounded and reversible; the region after
the classical snapshot runs coherently. Nothing in the kernel knows.

## 4. The paper, and how it maps onto the ISA

### 4.1 What the paper does

- **Model (§2.2).** A quantum algorithm is U_1…U_T on H_I ⊗ H_A ⊗ H_Z,
  each U_t = Σ_i |i⟩⟨i| ⊗ U_t^i (controlled on the input register, never
  changing it). It is *variable-time* when there are nested "done"
  subspaces {0} = H_0 ⊆ H_1 ⊆ … ⊆ H_T = H_Z with U_t Π_{≤t−1} = Π_{≤t−1},
  and the projectors commute with the computational basis of H_Z, so
  "done by t" is a classical predicate on basis states and later steps act
  as the identity on done branches. The running time on input i is a
  random variable T_i. The unit-cost assumption is that
  Σ_{i,t} |i⟩⟨i| ⊗ |t⟩⟨t| ⊗ U_t^i costs one step (the paper notes this is
  not O(1) local gates in general, but reasonable in a quantum random
  access model).
- **A worked construction (§2.2, example).** Any block-structured
  algorithm becomes variable-time by adding a flag register F and a block
  counter C: each step runs its unitary controlled on F = 0; at a block
  boundary it coherently performs the "done" test and XORs the result into
  F, then increments C controlled on F = 0.
- **Results.** Naive Grover with a variable-time oracle costs
  Õ(√N · max_i T_i) (eq. 1). Ambainis [Amb10] achieves the ℓ2 bound
  Õ(√(Σ_i T_i²)) (eq. 2). Composing Grover with the oracle by the generic
  straight-line composition theorem (Theorem 2.1 of [Jef22]: Õ(Q·T_avg +
  L)) gives only Õ(√N · max_i E[T_i]) (Lemma 3.2, eq. 11): no better than
  naive, because the bound must hold for every possible marked element.
  **Loop composition** (§5, Theorems 5.1 and 5.7), which composes the
  subroutine into a loop-shaped overlap graph rather than an unrolled
  line, recovers the ℓ2 bound and also the ℓ1 and ℓ0 variants of [Jef22]
  (Table 1). The authors' conclusion is that modelling program control
  flow properly matters for algorithm design.

### 4.2 Where the ISA already is

- **The lifted oracle is a straight-line subroutine of length B.** Every
  branch is padded to B steps and the whole call is charged as B
  generated steps (§8.2 aggregation). This is the regime of eq. (1):
  max over the domain, by construction.
- **The step machine is a variable-time algorithm in the paper's sense,
  up to one detail.** The "done" predicate is `pc = HALT`, a classical
  predicate on basis states, and it is monotone: once HALT, always HALT
  (§3.6 step 3). The answer register is `result_val`. The paper's
  condition U_t Π_{≤t−1} = Π_{≤t−1} asks for later steps to be the
  *identity* on done branches. In §3.6 as written an idle step still
  writes `frame_t.old_pc ← HALT` and clears it again on the inverse, so
  the idle step is not literally the identity on the frame. A variant
  that controls the successor XOR and both commit exchanges on `¬at_HALT`
  is the identity on done branches, leaves idle frames at |0⟩, and
  produces the same final tuple (a §5.7 transformation). See open item
  I5: state done-invariance as a property, make the `¬at_HALT`-controlled
  step the reference, and test idle steps as identities.
- **The fault register is a coherent "unfinished at B" flag.** `FAULT_
  BUDGET_STEPS` is set exactly on branches whose trace exceeds B; budgets
  are classical per call (`bennett_budget` mints a token per region and
  budget; `lift.oracle_call @o tk …` takes it as a classical argument,
  §4.2). So a single action may call the same oracle under a ladder of
  budgets B_1 < B_2 < … and amplify on "fault clear". That is the
  primitive Ambainis's variable-time search is built from (padded runs
  to geometric thresholds, amplification concentrated on the branches not
  yet done). Written as an action over budget tokens, Ambainis's
  algorithm is expressible against the ISA as it stands, and the §8.2
  accounting of such an action (each call charged B_j × rows) reproduces
  the ℓ2 cost up to logarithms. The fault register is therefore not only
  error handling; it is what makes an oracle's running time addressable
  from inside an algorithm.
- **The paper's loop composition is a statement about automatic
  composition**, which nothing in the ISA performs today: the ISA
  executes what the plugin and the algorithm author wrote. If a future
  layer composes an outer algorithm with an inner variable-time oracle
  automatically (as [Jef22]'s theorem does), it should do so loop-aware,
  and would need the oracle exposed at step granularity (an interface
  like "advance the oracle one step under this control", which the ISA
  does not have; a region call is atomic). Recorded as I6, not proposed
  for 0.8.

### 4.3 What the paper does not change

The worst case over the domain remains the cost of *one* padded oracle
call. What the variable-time literature changes is the cost of an
*algorithm* that calls the oracle many times: it can pay an ℓ2 average of
the per-input times instead of N times the maximum. The ISA's flat
per-operation accounting is consistent with that, because the algorithm
is written as a sequence of calls at increasing budgets and each is
charged what it costs.

## 5. Open items filed from this note

Numbered as in the worklog's open-items list (I1–I8). In priority order
for the spec: I5 (done-invariance), I1 (pebbled realisations), I4 (§7.4
wording), I2 (bounded-loop recovery), I3 (budget provenance in the dialect
spec), I8 (§3.6 "blocks" wording), I6 (step-granular interface, future),
I7 (the council's round-8 S1 suggestions).

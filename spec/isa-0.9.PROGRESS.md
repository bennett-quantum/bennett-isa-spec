# ISA 0.9 draft — progress file (resumable)

Job: bead `b-isa-spec-so1` (claimed). Brief: `docs/briefs/2026-10-29-isa-09-draft.md`.
Owned paths: `spec/Bennett-ISA-Spec-0.9-draft.md`, `spec/isa-0.8-to-0.9.diff`, this file, bd notes/creates
(label `isa-0.9-candidate`). Never touch `spec/Bennett-ISA-Spec.md`, `spec/README.md`, `reference-interpreter/`,
`docs/`, `worklog/`, `HANDOFF.md`, `CLAUDE.md`. Commit owned paths after every group
(`git commit -- <paths>`, retry on index.lock); never push. Headless.

Method: the draft started as a byte-for-byte copy of 0.8 (sha256 `90e61d9c…605e`); every change is a surgical
exact-string replacement applied by a script that asserts each old string occurs exactly once
(scripts kept in the job scratchpad; the diff `spec/isa-0.8-to-0.9.diff` is regenerated before each commit).

## Sources read in full (2026-10-29)

CLAUDE.md; spec/README.md; spec/Bennett-ISA-Spec.md (0.8, all 1,675 lines); the evidence note and its progress
file; the budgets note; reference-interpreter/SPEC-QUESTIONS.md (13 entries at reading); the interpreter council
round-1 report (fable R1-B1..B3, codex B01..B20, muse B1..B7 and all suggestions); dialect outline §6, §11, §17,
§25 and part B B-BLK; north star §0, §3.1, §4, §5, §8, §9; rubric-isa.md; HANDOFF "What made the ISA rounds
converge"; all open beads (`bd list`; I-, ISA-, G-, BLK-relevant and the 16 r8 children).

## Plan of §0 entries

Group 1 (editorial repairs, local semantics unchanged unless marked):
- V   version 0.9 everywhere (header, status, every sub-version, abi, device versions, examples).
- E1  I8 row indices (§3.6, §4.2 qjumpr, SPEC-Q 1).
- E2  SCC group ordering of the index space (§3.6; b-isa-spec-was; SPEC-Q 7).
- E3  ISA-10 direct versus cyclic hashes (§9.2; rejects zsw.11).
- E4  empty data spelling (§16.1; b-isa-spec-oim; SPEC-Q 9).
- E5  I4 logical history; active versus audit history; "retired" (§7.4; zsw.3, zsw.10).
- E6  uncall summary aligned with §7.4 (§4.1; zsw.1, zsw.9).
- E7  invocation contexts from the realised invocation tree (§12.1; zsw.2, zsw.7).
- E8  approximate SDP witnesses, δ margin (§12.1; zsw.6, zsw.12) and Tr_out convention note (zsw.13).
- E9  T4 hypotheses (§14.3; zsw.3, zsw.8) — final wording after ISA-8 (group 2).
- E10 borrow_demo positive vector (§16.3 example 3; zsw.15); suite D witness vectors (zsw.4 part).
- E11 qreg values through call/ret/br; per-register state; aliases (§3.3; dialect wording item).
- E12 qreg.slice lo literal or run-time (§3.3; ISA-5 wording).
- E13 entropy code 10 not "R only" (§5.4; dialect §11.3).
- E14 effective fault cause for literal 0 (§3.6, §4.2, §5.4; b-isa-spec-7qu) — SEMANTIC.
- E15 call-tier policy table (§3.1/§4.1; council B20).
- E16 SPEC-QUESTIONS closures 2,3,4,5,6,8,10,11,12,13 (each its own sub-entry).
- E17 council-found ambiguities: widths (R1-B1/B01/muse B1), canonical literals everywhere (R1-B2/B12/muse B2),
      region call graph acyclic incl. lifted calls and qinplace (B02), local regions as call operands (B06),
      matched uncall LIFO matching (B08), pointer args (B10), definition order independent of block text
      order (B11), load/store suffixes iN only (B12), call operand spelling (fable s2), interface-region names
      distinct from symbols (fable s9), qstore value / address operands as `in`-wire uses (fable s10, muse B4),
      param role suffixes (muse s3), lift.free matches unpinned entries only (fable s1) — the last SEMANTIC (corner).
- D   r8 suggestions deferred: zsw.4 (rest), zsw.5, zsw.14, zsw.16 → §17 with defaults.

Group 2 (capability contracts, SEMANTIC):
- K1 BLK-1 qreg[*], qreg.new run-time count, qreg.width, qreg.id, qreg.from_id, register ledger, binding checks.
- K2 BLK-4 qreg.new/free/slice/concat (and K1's ops) in R, with history and reversal.
- K3 BLK-2 budget.mint (CC and R), token rules.
- K4 BLK-3 qinplace with (ins..) (mem..) (args..), inverse-pair claim, §6.1 transfer, run-time claim check.
- K5 ISA-7 CC/R cause table (§4.9), explicit `fault C` in CC/R.
- K6 ISA-8 terminal R faults, FAILED → REVERSED, T4 eligibility.
- Examples 4–9 (one per family) + republished 1–2.

Group 3 (optional efficiency): I5, I1 → §17 with full-history default and the hook; every other `efficiency`
item recorded in §0 as considered and deferred with its 0.8 construction.

## Status

- [x] group 1   - [x] group 2   - [x] group 3   - [x] hostile reread and fixes (own + independent reviewer)   - [x] final diff + commit

## Decisions (with reasons) — appended as taken

Group 1:
- G1-a (E2, SCC order). Key of a one-member SCC = its member's closure hash (self-recursive or not); key of a
  multi-member SCC = its closure scc_hash. This is the reading SPEC-Q 7 implemented and muse round-1 s1 proposed.
  DEPARTURE from the evidence note, which keys every *cyclic* group (including a self-recursive singleton such as
  `sum`) by scc_hash: both are total; the interpreter's keeps 0.8's literal "ordered by closure hash" for every
  singleton, so no existing single-lproc object changes its labels under either reading, and it is what the
  only implementation does (brief: interpreter reading unless it contradicts the spec).
- G1-b (E16.9, const literals). SPEC-Q 12 reads every const literal modulo 2^N. DEPARTURE from SPEC-Q 12: the draft
  requires −2^(N−1) ≤ c < 2^N (value c mod 2^N), following round-1 fable R1-B2 (`lift.const.i8 257` silently became
  1). Stricter than the interpreter, not than 0.8's text (0.8 states no range); no value becomes unwritable
  (every residue has a spelling in range). North-star citation for the stricter rule: §0 "never a silent wrong
  answer"; CLAUDE.md rule 1. Open question Q2.
- G1-c (E17.9, call spelling). `call @f a b` (bare operands, the §4.1 form and the interpreter fixtures); the
  parenthesised form is ill-formed for `call` (it remains the form of `uncall`'s two lists, of `lift.call` and of
  region calls). Round-1 fable s2 found both accepted.
- G1-d (E17.5, matched uncall). LIFO matching; suite F rejects a pair only for an intervening write to an `ins`
  location. 0.8 also listed "cells" as a static check; dropping it is safe because `lift.oracle_uncall` is the same
  permutation as the call (§4.2) and prefix reversal undoes an unmatched clear. Broader than 0.8 (accepts more).
- G1-e (E17.13, lift.free). match_i gains ¬pinned_i; the flag is the same boolean as 0.8's. Derivation D4.
- G1-f (E14, fault 0). Effective cause C′ = 4 for literal 0 (evidence note proposal, verified D8). Syntax kept.
- G1-g (E15, tiers). Table: CC calls CC/R; R calls R; uncall of CC or R segments from CC, of R from R. R→CC is
  ill-formed (0.8 §3.1 draws none; the interpreter's registers.bisa relies on it: flagged to the interpreter
  agent via a bead, not edited).
- G1-h (E17.12, role suffixes). Optional on wires, equal to the group role when present; required on regions.
  The permissive reading keeps both spellings seen in the evidence note and the fixtures legal.
- G1-i (E16.8, frame descriptor JSON). Scratch is always `{clean, dirty}` per class (SPEC-Q 11 said "may separate";
  one canonical shape avoids two encodings of one descriptor).
- G1-j zsw decisions: accept .1 .2 .3 .6 .7 .8 .9 .10 .12 .13 .15; .4 accepted in part (the two witness vectors go
  into suite D; the controlled-action target/schedule/certificate fixture waits for `ref`, §17 item 7); reject .11
  (contradicts the published inc8 bytes; verified D1b); defer .5 (citation not verified here), .14 (ref library
  artefact), .16 (topology: §17 item 15).

## Derivations (written before any number is stated in the draft)

D1 (E1, inc8 indices). inc8's one block has rows const, add, retval at indices 0, 1, 2; n = 3, HALT = 3; the
smallest P with 2^P > 3 is P = 2. Each row is one step: retval executes at step 3, so B ≥ 3 succeeds and B = 2
ends with pc = 2 ≠ HALT, committing FAULT_BUDGET_STEPS (the published results, unchanged).

D1b (E3, hashing). Recomputed here with hashlib from the printed 0.8 envelopes: SHA-256(inc8 envelope) =
8ce5e64f…be93 = the published hash (direct); SHA-256(hex ‖ ":0") would be 5f3e8bcd…8ee8 (≠ published), so zsw.11
contradicts the vectors. SHA-256(sum envelope) = c6d549be…bc31 and SHA-256("c6d5…bc31:0") = 74f110bb…cd92 = published
(wrapped). zero: SHA-256("isa=0.8\nptr_width=32\n{…data sig…}\nzero 00\n") = 7002ed5e…67e6 = published (direct).

D3 (E16.3, counter widths). Reachable bounds, from the flags of §3.7/§4.2 (a counter is only advanced when its
flag is clear): heap_used ≤ heap_limit ≤ heap_limit_max, and the heap range lies in [0, 2^H) (§7.2), so
heap_used < 2^H < 2^(H+2); atop advances only while atop < 2^A_tab, so atop ≤ 2^A_tab < 2^(A_tab+1); sp + R_f ≤
stack_limit ≤ 2^S_cells, so sp ≤ 2^S_cells < 2^(S_cells+1), and fp ≤ sp; depth ≤ depth_limit, and Dc =
⌈log₂(depth_limit + 2)⌉ gives 2^Dc ≥ depth_limit + 2, so depth ≤ 2^Dc − 2 (which also makes §5.4's check
depth_limit < 2^Dc − 1 hold for the minimum Dc). Hence every value fits the minimum widths and a wider counter
holds the same value: F_B does not depend on the choice; only W_cell = max(P, Sc, Did, W_max) and the resource
count do, so the reference realisation fixes the minimum.

D4 (E17.13, lift.free). Heap entry j has base heap_base + Σ_{i<j} max(n_i, 1) (§7.1, §4.2 lift.alloc): strictly
increasing in j because max(n_i,1) ≥ 1, so heap bases are pairwise distinct. Pre-entries (globals, interface
regions) are pinned. Hence at most one unpinned entry has base p, and match_i = (i < atop) ∧ (base_i = p) ∧
¬pinned_i selects at most one i. flag = ¬∃i: match_i ∧ live_i equals 0.8's ¬∃i: (i<atop)∧(base_i=p)∧live_i∧¬pinned_i.
Under ¬flag, live_i ^= match_i flips exactly that entry. match_i reads atop, base_i, pinned_i, none of which the row
changes, so recomputing it uncomputes the scratch (the D-row inverse is the same three sub-operations reversed;
including live_i in match_i would break this, which is why the fix uses ¬pinned_i). 0.8's flip set additionally
contained every pinned live entry with base p: a size-0 global at the freed base (size-0 ranges are disjoint from
every range, §7.1) lost its live bit, after which `lift.boundscheck g 0` (inrange(g,0) needs a live entry with
base ≤ g ≤ base) faulted where it should not. No published object has a size-0 global.

D5 (E16.9). The range −2^(N−1) ≤ c < 2^N contains exactly the signed and unsigned spellings of the 2^N residues.

D6 (E10, borrow_demo). Wires (t, x, b); gates in order V = cx x b, W = cx b t, V⁻¹ = cx x b, W = cx b t.
b → b⊕x; t → t⊕(b⊕x); b → b; t → t⊕(b⊕x)⊕b = t⊕x. Final (t⊕x, x, b) for all 8 basis inputs; b restored, so by
linearity identity on any reference entangled with b. Under an inherited control w every gate is controlled by
w: b⊕wx; t⊕w(b⊕wx); b; t⊕w(b⊕wx)⊕wb = t⊕w·x (w² = w). Side conditions: V's only gate targets b ∈ B with control
x ∈ X = {x}, X ∩ B = ∅; W's gate targets t ∉ X ∪ B ∪ {b} (W's controls), W writes no wire of X; W's gate has one
control in B; V⁻¹ is V reversed; no swap. The map is an involution (applying it twice gives t⊕x⊕x = t), so the
adjoint (reverse sequence cx b t; cx x b; cx b t; cx x b: t⊕b; b⊕x; t⊕b⊕b⊕x = t⊕x; b) realises the same permutation;
its frame is not of the lemma's V;W;V⁻¹;W shape (it starts with a gate targeting t), so it is covered by the
`adjoint_table` structural rule, not by the lemma. bounds = [1] (one qloop with trip count 1).

D7 (E10, suite D witness vectors). d = 2, identical channels, J(Φ) = 0. Y = I₄/8: Y ± J = I₄/8, LDL† pivots all
1/8 > 0. Tr_out(I₂ ⊗ I₂ / 8) = (2/8)·I₂ = I₂/4. μ = 3/8: μI − Tr_out Y = (3/8 − 2/8)I₂ = I₂/8, pivots 1/8 > 0;
μ = 3/8 ≤ bound = 3/8: accept; then ‖Φ‖◇ ≤ 3/8 (true, Φ = 0). All entries are dyadic, so interval arithmetic at any
precision_bits ≥ 4 is exact. Negative: Y = 0, μ = 0: Y ± J = 0 has pivot 0, not strictly positive: reject (although
the channels agree), exactly as §12.1's strict-pivot rule requires.

D8 (E14, effective cause). Q: row writes code ^= flag·C′ with C′ = 4 when C = 0; F ≥ 4 so 4 < 2^F. The first
fault halts (successor HALT), so code holds exactly one C′ ≠ 0 on a faulted path; commit XORs it into fr. Inverse
uses the same C′, so prefix reversal is unchanged in form. Example 9 (group 2) derives its outcomes.

D9 (E8, approximate witnesses). Given certified λ_min(Y−J) ≥ −η₊, λ_min(Y+J) ≥ −η₋, λ_min(μI − Tr_out Y) ≥ −η_s and
δ > max(η₊, η₋, η_s): Y′ = Y + δI_{d²} gives Y′ ± J ⪰ (δ − η±)I ≻ 0; Tr_out(δI_{d²}) = δ·d·I_d, so with μ′ = μ +
(d+1)δ, μ′I − Tr_out Y′ = (μI − Tr_out Y) + δI ⪰ (δ − η_s)I ≻ 0. (0.8's recipe is the case η = 0.)

## Hash placeholders

Every hash in §16.3 of the draft is `<to be computed by the 0.9 precheck>`: the reference binary
(`reference-interpreter/target/debug/bennett-isa-ref`) hard-codes `0.8` in its header, signature and envelope
checks (src/object.rs:708, :975, :1017; src/identity.rs:258), so it does not accept the 0.9 version string even
for unchanged bodies, and it has no 0.9 instructions. The crate was not edited.

Placeholders are **distinct per example and symbol** (`<to be computed by the 0.9 precheck: example k, name>`),
because the seed's `--repatch` maps old string → new hash by plain replacement: one shared placeholder would map
every symbol to the last hash computed. Full list (count of occurrences in the draft):

| placeholder (example, symbol) | occurrences | where |
|---|---|---|
| 1, inc8 | 2 | ex 1 symbols line; ex 2 hash list |
| 2, zero | 4 | ex 2 symbols line, hash list, sum envelope (`@…` and part-(6) line) |
| 2, sum | 2 | ex 2 symbols line, hash list |
| 2, sum_db | 2 | ex 2 symbols line, hash list |
| 3, borrow_demo | 1 | ex 3 symbols line |
| 4, ones / 4, flip | 1 each | ex 4 symbols line |
| 5, main / 5, scratch / 5, flip | 1 each | ex 5 symbols line |
| 6, adaptive / 6, overdraw / 6, inc8 | 1 each | ex 6 symbols line |
| 7, g / 7, ginv / 7, addya / 7, run | 1 each | ex 7 symbols line (plus `<closure hash of g>`, `<closure hash of ginv>` in the prose certificate fragment, which the precheck fills when it generates the certificate) |
| 8, main / 8, work | 1 each | ex 8 symbols line |
| 9, zero_cause / 9, zero_cc | 1 each | ex 9 symbols line |

**Cross-check values (NOT normative, not in the draft).** Computed by importing the *unmodified* seed
`reference-interpreter/isa_check.py` (functions `extract_examples`, `parse_object`, `compute_hashes`) on the
draft text before the placeholders were made distinct. The seed has no CC/R closure set and no part-(5c) device
lines, so examples 4 and 8 (CC/R functions with a layout; example 8 also a device) are omitted as unreliable;
for the others the seed's model is complete (oracle, lproc, data, action, and memoryless CC/R with no devices).
The 0.9 precheck must compute its own values; these are only for comparison.

```
ex1 inc8        ed9865300b85a519c2379de98e36e635e72b5b78d8bdfca03dccf9a09aa3df5e
ex2 zero        43e032f98d7aff20f6864d6368b20ab4ef9d810938999acff8d71ed16e6b17d7
ex2 sum         069abdb5b7239c403c2687bcd317c0de85710f955de3cb790444ccbc9aa09592
ex2 sum_db      e78f97e074907edfc724df7577dfb12684a05db21656477f9235fe71ed0572d8
ex3 borrow_demo 6f4782e9bde4267acd1afddd650093722a056d9d50ee465b79cb86a4606b2a94
ex5 flip        3860ea712b2d70fc495834586bb397656a467c85235aa069768318a06e9fa89b
ex5 scratch     05525834cbe5a518aa97444912254f3c12c75dd86314a98ba550f45187208a0b
ex5 main        5cebd5a72ee7873edda0d2c35afed2a5ab79eba60f2b242b2a856d5fa4074634
ex6 inc8        ed9865300b85a519c2379de98e36e635e72b5b78d8bdfca03dccf9a09aa3df5e (= ex1, as it must be)
ex6 adaptive    b80aed1639a9ac6a6856f02b0b3264d905e4eea3fa3f9081c01e56283ea398cf
ex6 overdraw    c0d9835844fa882c1864044acc6fa26dba3935e0d6a9d33f9cdbf1e7c877d250
ex7 g           0d65d926275688cfebf2831c1de960b03a6b8ea6400866e6d9fef8f393ca2c2e
ex7 ginv        786f1970ca3d78ef3d5de795411ee276286711c89b756872757c24f9bd569cc9
ex7 addya       5999b6d3d2362f8c1971ef6f37b404766aa5c18f6ba1880d767bfde351e47392
ex7 run         562ab7270a0f508d6a2f4a757f3f90964da0b60541bac2c5efa421bad009056a
ex9 zero_cause  badf7dd2a59afa879f092921946539dfb918cedd35f6fbbaa838c630577eaed4
ex9 zero_cc     0403d76ff065acab57cb9165cfe1d3d73bbf8dad60bbc1348fb0305ee4f37611
```

Precheck notes: the seed's printed-table regex needs 64-hex values (the placeholders will fail it until
patched, as intended); its `--repatch` and its executable checks of inc8/sum_db need `0.9` acceptance; examples 4–9
need a 0.9 interpreter (new instructions) — the brief forbids changing the crate here.

## Open questions for the maintainer (each with a recommendation)

Q1 SCC order key of a self-recursive singleton (E2): its member closure hash (SPEC-Q 7, adopted) or its scc_hash
   (evidence note)? Recommend the adopted reading: it is what the interpreter does and keeps 0.8's wording.
Q2 Const literal range (E16.9): reject literals outside [−2^(N−1), 2^N) (adopted, RC fable R1-B2) or read every
   literal modulo 2^N (SPEC-Q 12)? Recommend the range; the interpreter fix loop should align.
Q3 R faults terminal (K6) versus "an R call returns a fault status to its CC caller" (DL B-BLK ISA-8 row, which
   contradicts DL §17.6). Recommend terminal now; a catch API, if ever wanted, is an addition, not a change.
Q4 budget.mint in R as well as CC (K3; departs from EN and DL). Recommend yes (BLK-4 analogue; R is classical).
Q5 qreg.from_id pure (K1; departs from EN and DL). Recommend pure: it keeps the compute–copy–uncall idiom able
   to look up handles; dangling aliases after an uncall trap deterministically.
Q6 qinplace inverse-pair check at the CC/R call (run-time trap 4) rather than "ill-formed object" (EN). Recommend
   the run-time check: tokens and args are run-time values, so applicability is only decidable at the call.
Q7 Token-to-region binding (K3, bd 8wn): any region call of the program may use any token. Recommend as drafted
   (the §5.4 checks are object-level); the alternative (trap on region mismatch) adds a check with no benefit.
Q8 Registers wider than 128 bits cannot be measured directly (bd 04w). Capability kept by copy + measure + reset;
   recommend leaving `qreg.split` / wide measurement as an efficiency item for 1.0.
Q9 §17 item 14 (K7): accept the reconciliation (hardware releases unverified, excluded from uncall/reversal)?
   Recommend yes; 0.8's default was untestable.
Q10 E17.5 drops the static "cells" check between matched lift.oracle_call/uncall (accepts more). Recommend yes
   (sound: the uncall is the same permutation; prefix reversal cleans an unmatched clear).
Q11 E17.13 lift.free/free flip only unpinned entries (semantic corner change). Recommend yes (0.8 killed a pinned
   size-0 global's entry, making a later boundscheck of length 0 fault wrongly).
Q12 Hashes: keep placeholders until the 0.9 precheck exists (as the brief says), or accept the cross-check values
   above now? Recommend placeholders: the seed lacks CC/R closure sets and device lines (examples 4, 8).
Q13 Deferred r8 items (zsw.4 rest, .5, .14, .16): recommend publishing `ref` (zsw.14) before grading 0.9,
   since the controlled-action fixture (zsw.4) depends on it.

### Group 2 decisions

- G2-a (K1). `qreg[n]` may be used where `qreg[*]` is expected; `qreg[*]` → `qreg[n]` (parameter, block parameter,
  `ret`, region slot, launch) is checked at the transfer (width = n, else trap 4, no effect). `measure` keeps the
  0.8 form for `qreg[n]`; `qreg[*]` needs `measure.iN` (width checked before measuring). Run-time width 0 is legal
  (an empty `Vec<Qubit>`); such a register can be freed, passed, concatenated, never measured (N ≥ 1) or bound to a
  slot (slot widths ≥ 1). Ids are never reassigned while the program is loaded (simplest sufficient rule for
  "reversal restores the same ids"). `qreg.slice` `len` may also be run-time (result `qreg[*]`).
- G2-b (K1). DEPARTURE from the evidence note and DL: `qreg.from_id` frames are **pure** (it changes only the
  instance set; ownership acquisition is launch bookkeeping released at CLOSED). Reason: purity lets the
  compute–copy–`uncall` idiom (§7.4) look a handle up after the call; impure would forbid it. Soundness: if the
  uncalled segment created the register, the retained alias is dangling and every use traps 4 (deterministic).
- G2-c (K2). Reversal of `qreg.new` releases without a test: the register is clean because every later frame
  touching it is undone first (LIFO) and the run owns it exclusively; simulators verify. Reversal of `qreg.free`
  re-creates it clean with the same id/class/width.
- G2-d (K3). DEPARTURE: `budget.mint` is CC **and R** (evidence note and DL: CC). Reason: the BLK-4 analogue —
  an R routine computing a budget from R data mid-run cannot hoist it to CC; R is classical, so §3.1's invariant
  holds. Mint frames are impure (they change the token ledger).
- G2-e (K3, new closure found while drafting). 0.8 never says whether a token validated for region @p may be given
  to `oracle_call @o`. The §5.4 checks depend only on the object's layout, and the evidence note's own fixtures pass
  one token to several calls. Draft: a token MAY be supplied to any region call of the program. Bead filed.
- G2-f (K4). The inverse-pair obligation is carried by a certificate claim with a budget tuple and an args domain,
  checked at the CC/R region call before any effect (trap 4 otherwise). DEPARTURE from the evidence note ("a failing
  certificate is ill-formed"): tokens and args are run-time values, so the claim's applicability can only be checked
  at the call; a run-time trap is the more permissive outcome. Monotonicity lemma D14 makes one claim cover every
  larger tuple. The short form `qinplace @g @ginv tk x` is the spelling when all three lists are empty.
- G2-g (K4). §6.1: x stays `other` if it was `other` (0.8's row gave "perm with S_x ∪ Π", undefined for other x).
- G2-h (K5). "CC trap" and "R fault record" are one event: a terminal fault record; a FAILED run has exactly one
  (0.8: "at least one"; no second can arise once the first is terminal). Every CC/R block ends in `br`, `ret`,
  `trap` or `fault` (0.8 had no CC/R terminator rule). Unclean `qreg.free` on a simulator: record 4.
- G2-i (K6). R faults are terminal (EN; DL §17.6). DL's B-BLK row for ISA-8 says "an R call returns a fault status
  to its CC caller"; that contradicts DL §17.6 and is not adopted (open question Q3).
- G2-j (K8). §17 item 14's 0.8 default ("FAILED with FAULT_UB_TRAP", for a condition hardware cannot test)
  contradicts §7.4's "qreg.free frame whose cleanliness was not verified (hardware, §17 item 14)". Reconciled: the
  hardware release proceeds unverified, its frame is marked so (no uncall, no reversal), and releasing an unclean
  register discards (traces out) its wires.

### Group 2 derivations

D10 (example 4, `ones`, n = 3). qreg.new any 3 → register R, width 3, state |000⟩, fresh id r. w = 3; id = r.
p = alloc 8 → 16 (bump starts at heap_base = 16; entry (16, 8, live, unpinned); bump 24). store.i64 16 r;
load.i64 16 → r; from_id r → q2, alias of R (live, not a view, owned by this run). Loop i = 0, 1, 2: slice [i, 1)
of R (lo run-time, len literal 1 → `qreg[1]` view), R viewed; action_call @flip flips wire i (X); view consumed,
R live. i = 3: icmp.ult 3 3 = 0 → ^done. measure.i3 q2: width 3 = N; state |111⟩ (wire i = bit i), so outcome
0b111 = 7 with probability 1; R consumed. ret m = 7: COMPLETED, one INT result of width 3 (⌈3/8⌉ = 1 byte 0x07),
one measurement record (R, bits 1 1 1). Reverse: E_NOT_REVERSIBLE (a measure frame).
n = 2: the loop flips wires 0 and 1; measure.i3 sees width 2 ≠ 3 → trap 4 before measuring: FAILED, one record 4,
no measurement, R live in |11⟩. That run is reverse-eligible (no measure frame, no free): reversal undoes the record,
then i = 1's call by adjoint (X) and slice, i = 0's likewise, from_id, load, store, alloc, the consts, width, id
and finally qreg.new (R is |00⟩ again: release). n = 0: no iteration; measure.i3 sees width 0 → FAILED, record 4.

D11 (example 5, `scratch`, R allocation). For n: R creates register Q (width n, |0…0⟩). Iteration j < n: slice
[j,1) → view, flip (wire j: 0 → 1), view consumed; slice again, flip (1 → 0). After the loop Q is |0…0⟩;
qreg.free Q: clean (simulator verifies) → consumed. ret; main ret: COMPLETED, no values. Whole-run reversal (LIFO):
main's ret; call frame boundary; @scratch's ret; qreg.free undone → Q re-created clean, same id; iteration n−1
undone: br, add, second flip's adjoint (wire n−1: 0 → 1), its slice (view gone, Q live), first flip's adjoint (1 →
0), its slice; … iteration 0 likewise; consts; qreg.new undone → Q released clean. REVERSED; ledger as at entry (Q's
id is not reused). Midway, just after the first undone flip, wire n−1 is |1⟩, which checks the order. On hardware
the free is unverified → E_NOT_REVERSIBLE. Under 0.8 the R function is ill-formed (qreg.* CC only).

D12 (example 6, `adaptive`). b = 1: select → 3; mint (3, 0, 0, 0) for @inc8: memoryless object, heap = stack =
depth = 0 required: passes. inc8 at B = 3 (D1): t ⊕= (x + 1) mod 256, f unchanged. b = 0: B = 2: (0,
FAULT_BUDGET_STEPS) → t unchanged, f ⊕= 1. Both runs COMPLETED (an oracle fault is a fault-register outcome, not a
run fault). Reverse: the region-call frame re-applies inc8 with its recorded tuple (the oracle is an involution):
t, f restored; the mint frame is undone (token revoked); select and consts removed. `@overdraw`: mint (3, 1, 0, 0)
fails heap_limit = 0 for a memoryless object → FAILED, record 4, no token.

D13 (example 7, `addya`). g: rows s = x + y, r = s + a, retval r: 3 rows, F_B = ((x+y+a) mod 4, 0) for B ≥ 3, (0, 1)
for B ≤ 2. ginv: s = x − y, r = s − a: ((x−y−a) mod 4, 0) for B ≥ 3. For fixed (y, a) both are bijections of
Z/4 and mutually inverse: (x+y+a) − y − a = x. Macro (token B ≥ 3): t = 0, f = 0; g: t = (x+y+a) mod 4, f = 0; qexch:
x' = (x+y+a) mod 4, t = x; ginv on x': t = x ⊕ ((x'−y−a) mod 4) = x ⊕ x = 0, f = 0; free t, f clean. E.g. x = 3, y =
2, a = 1: x' = 6 mod 4 = 2; ginv(2; 2, 1) = −1 mod 4 = 3; t = 3 ⊕ 3 = 0. Adjoint (pair swapped): x ← (x − y − a) mod 4.
Under an external control 0 no gate acts and scratch stays 0. Exhaustive claim: 4 · 4 · 4 = 64 elements (x, y, a),
both oracles at (3, 0, 0, 0). A token (2, 0, 0, 0) is not ≥ (3, 0, 0, 0): trap 4. Without a certificate: trap 4.

D14 (monotonicity, used by K4). Reference interpreter at tuple β₀ = (B₀, h₀, s₀, d₀) reaching `lift.retval` at
step T ≤ B₀ without a fault: the only limit-dependent decisions are the flags heap_used + max(n,1) > heap_limit,
depth + 1 > depth_limit, sp + R_f > stack_limit (all false on this trajectory) and the step bound (not reached).
For β ≥ β₀ componentwise each flag is still false (its right-hand side only grew), so the trajectory is identical
and halts at T ≤ B₀ ≤ B with the same result_val: F_β(x) = F_β₀(x) = (f(x), 0). (A faulting or exhausted run at β₀
gives no such implication, which is why claims state a minimum tuple.) Nested `lift.oracle_call`s use tokens from
classical arguments, which the claim's args domain fixes.

D15 (example 8, failed prefix). Initial: bump 16, no table entries, clock c = 0, tape empty. main: n = 8; alloc 8 →
p = 16, entry (16, 8, live, unpinned), [16, 24) zero, bump 24; call @work 16; v = 7; store.i64 16 7 → [16, 24) =
07 00 00 00 00 00 00 00 (frame keeps 00 × 8); clock.read 16 → writes c = 0 LE: 00 × 8 (frame keeps 07 00 × 7), c = 1,
tape (clock, read, 16); fault 21 → C′ = 21: terminal record, FAILED. Observable: nfaults = 1, fault 0 = 21,
nvalues = 0, [16, 24) = 00 × 8, c = 1, tape length 1; main's ret never executes. Reverse: record removed; clock.read
undone ([16, 24) = 07 00 × 7, c = 0, tape empty); store undone ([16, 24) = 00 × 8); v; the open activation of @work
and main's call frame; alloc undone (bump 16, entry removed); n. REVERSED; memory, table, clock and tape at entry.
0.8: FAILED → E_NOT_REVERSIBLE.

D16 (example 9, zero cause). `@zero_cause`: rows faultif (index 0), retval (1); n = 2, HALT = 2, P = 2 (2^2 > 2). B =
0: every x → (0, 1). B = 1: x = 0: flag 0, pc = 1, not halted → (0, 1); x = 1: flag 1, code ^= C′ = 4, faulted,
HALT → (0, 4). B ≥ 2: x = 0 → retval 0: success, result 0 → (0, 0) (t unchanged, f unchanged); x = 1 → (0, 4). In
0.8 the x = 1 outcome at B ≥ 1 was (0, 0), identical to x = 0's success. `@zero_cc`: `fault 0` → record C′ = 4,
FAILED.

Beads filed by this job: b-isa-spec-8wn (token-region binding), b-isa-spec-exc (§17 item 14 vs §7.4),
b-isa-spec-04w (measuring registers wider than 128 bits; efficiency, construction recorded).

### Group 3 derivations and decisions

D17 (I5, identity-idle variant, verified by hand before citing EN's 608-case check). Forward: a = (pc ≠ HALT);
at_k for k < n from pc (all 0 when idle); rows; successor XORs for k < n and both exchanges under a (no HALT
successor); then old_pc ^= HALT under a. Active from k: after the exchanges pc = succ, pc' = 0, old_pc = k; then
old_pc = k ⊕ HALT ≠ 0 since k ≠ HALT. Clean dispatch: at_k ^= (old_pc ⊕ HALT == k) ∧ a. Clear a: a ^= (old_pc ≠ 0) =
1 → 0. Idle: a = 0, no write, old_pc = 0, a ^= (0 ≠ 0) = 0. Inverse: a = (old_pc ≠ 0); at_k from old_pc ⊕ HALT under a;
old_pc ^= HALT under a (→ k); inverse exchanges under a (pc = k, pc' = succ, old_pc = 0); successor XOR undone (pc' =
0); rows undone as printed; at_k uncomputed from pc under a; a ^= (pc ≠ HALT) = (k ≠ HALT) = 1 → 0; idle: all zero.
A step ending in HALT (retval, fault) still leaves old_pc = k ⊕ HALT ≠ 0, so it is never confused with idle. Same
frame width (P bits), same F_B. Decision: §17 item 17 with the printed step as default (brief: no expansion fixture
with hand-derived gate-level values exists, ref unpublished).

D18 (I1). P_{j+1}(a, b) = P_j(a, c); P_j(c, b); P_j(a, c) with c clean: T(j+1) = 3T(j), T(0) = 1 → 3^k base
evaluations for k = ⌈log₂ B⌉ (after padding F to 2^k stuttering steps); one temporary configuration per level live
during the inner calls → at most k temporaries of S bits. EN's verifier checked the recurrence to level 5 on Z/4.
The bead's "polylog time" is not adopted. Decision: §17 item 18, default full history, hook in §9.4/§11.

## Verification record

- Hand derivations D1–D18 above, written before the numbers entered the draft.
- Independent hashlib check of 0.8 hashing (D1b); the printed 0.9 inc8 envelope hashes to the seed's 0.9 inc8
  value (consistency of the republished envelope).
- A small independent Python model (job scratchpad, not the interpreter) re-checked the stated results of
  examples 3 (all t, x, b, w), 4 (n = 0, 2, 3), 6 (all x, 6 target values, all f, both b, with reversal),
  7 (all x, y, a, both controls, adjoint, B = 2 failure), 8 (byte trace and reversal) and 9 (B = 0..5): PASS.
- Cross-reference check: every § reference resolves except DL §17.6 / §25 (dialect sections, labelled DL);
  §17 items 1–18 exist and every "§17 item n" reference resolves.

## Hostile review (independent reviewer agent, read-only, rubric-isa) and fixes

The reviewer re-derived every stated result of examples 1, 3–9, the suite D vector, the E8 margin algebra, the
idle-step variant, the counter widths and the lift.free match: all correct. It found 9 S0 and 13 S1 issues; each
was checked against the text and all were accepted:

- S0-1 qjumpr into a block's interior vs the kept "any other index faults" → E17.16 (range check only; interior
  rows reachable; re-indexing preserves reachable rows).
- S0-2 "ill-formed where suite F decides it" was circular → E11 now states the exact static rule.
- S0-3 inverse-pair coverage only by exhaustive claims was a narrowing (NS §3.1 promises library certificates;
  Int64 `x += a` would need ~2^128 runs) → K4 admits `inverse_pair_library` claims (no args domain = all values),
  sampled never covers.
- S0-4 monotonicity ignored nested lift.oracle_call (a nested outcome can change with its own budget) → entries
  carry exact `arg_tokens` tuples; the note now says nested rows use their own tokens.
- S0-5 orientation: adjoint `qinplace @ginv @g` would not match an entry {g, ginv} → an entry covers both.
- S0-6 minted tuples vs the single schedule line → invocations whose bindings match run the schedule, others the
  §11 defaults; tokens encoded in params_hash as their tuples (this also closes the same gap for host tokens).
- S0-7 effect memory operand out of range had no code → 7.
- S0-8 T4's quantum-state clause false for ε > 0 adjoints → exactness hypothesis stated; ledger claim restricted to
  registers the run created or touched; qreg.new reversal releases without a test (residual within bounds).
- S0-9 lifted-call args from locations would be superposed "classical" values → E17.14 (classical args only);
  consequence filed as bd 7u6 (lprocs have no token/args source), open question Q14.
- S1s: from_id ownership not undone, host calls on owned registers E_INVALID_STATE; measure.iN on qreg[n] with
  N = n; literal qreg.new 0 ill-formed (literal n ≥ 1); slice out of range traps even when literal; executor class
  includes inverse-pair checking; "verifying executor" defined (E17.17) and used for qreg.free and host calls;
  item 16 faults before mutation; @o's role stated; ids executor-chosen; effect undo removes its tape entry;
  §0 Group 1 retitled, E11 → S, no-narrowing list completed; G-15 now realised directly (E17.1, E17.15); lifted
  suffix widths unbounded (E17.15); unmatched or path-dependent uncalls ill-formed; nits (suite letters, §3.2 wire
  store, example 4/5/8 wording).

Not changed: T1's "halts within B + 1 rows" (0.8 text, counts the halting check; no conflict with "exactly B rows
executed"); `fn` result names (0.8 convention); `bennett_qreg_new`'s `uint32_t n` (host-side allocation; device
widths are unbounded through `qreg.new`).

Q14 (new). Lprocs cannot obtain tokens or classical args for lifted calls (0.8 limitation made explicit by E17.14;
bd 7u6). Recommend raising it as a blocker candidate for the dialect council: either lprocs read the enclosing
oracle's classical arguments as pseudo-locations, or lprocs take token/classical params.

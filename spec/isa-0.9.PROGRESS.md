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

- [x] group 1 (commit below)   - [ ] group 2   - [ ] group 3   - [ ] hostile reread and fixes   - [ ] final diff + commit

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
for unchanged bodies, and it has no 0.9 instructions. The crate was not edited. List: below.

## Open questions for the maintainer

(see below)

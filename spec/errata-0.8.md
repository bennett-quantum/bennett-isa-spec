# ISA 0.8 erratum — borrowed frame domain (§5.6)

Binding maintainer interpretation, 2026-11-01; b-isa-spec-73y, ISA 0.9
editorial pass V. The accepted 0.8 file remains byte-for-byte unchanged.

The sentence is: “The frame's realised permutation P_F on its wires MUST
factor as `P_F = P' ⊗ I_B`, established by a T3 claim over the frame's full
domain (exhaustive, or a library certificate, §12 `borrow_frames`) or
**structurally by the translation lemma**”.

Reading A takes “full domain” as the independent cube of every frame wire,
regardless of allocation or the region prefix. Reading B includes every
borrowed/interface value but admits established allocation/prefix facts
for local wires. Council intent is B: `CX(0,b)` is identity on arbitrary
b and an external entangled reference.

The binding interpretation is the precise D_F definition, normative
constant/causal-adjoint entry analysis, restricted T3 equality and checker
support obligation in [ISA 0.9 §5.6](Bennett-ISA-Spec-0.9.md#56-borrowed-systems-and-borrow-frames),
K76. That text is incorporated here: no stronger polynomial or quantum
entry analysis may narrow D_F. Exact permutation proofs inside frames
remain available; borrowed and interface components remain unrestricted.
Linearity extends equality on span(D_F) to each reachable entangled state.

Affected accepted reference fixtures (all remain accepted):
`r9-sol-oracle.bisa`, `r9-codex-oracle.bisa`, `r9-loop.bisa`, and
`r9-sol-qinplace.bisa`, in `reference-interpreter/tests/fixtures/`.
A polynomial-only interleaved CNOT prefix zero is outside this entry
analysis and now rejects with §5.6. This erratum does not admit 0.9 objects
through the reference interpreter's existing 0.8 version gate.

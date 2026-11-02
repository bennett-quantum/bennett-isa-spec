# bennett-arith-4yg.1 — PPC parity where the low component becomes zero

Council proposal, not a published amendment. Owner: dialect specification;
implementation follow-up: `bennett-arith-4yg`. References are to maintained
[dialect 0.2.12](../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md)
§§13.1, 13.3–13.5 and the corresponding
[0.3 draft](../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md).

## Exact defect and North Star scope

§13.1, “PPCDoubleDouble priority and rounding”, defines canonical finite
pairs by h = binary64 roundTiesToEven(h+l), with l the exact residual.
It then requires the nearest canonical sum, “with ties to the even
low-component significand, using the high significand when the low
component is zero”. Adjacent canonical candidates can both be odd under
that rule. Thus a required result is not uniquely specified for valid
FMA and incoming IEEE conversion inputs. This is a numerical contract
defect, rather than an absent softfloat feature or an ISA operand defect.

§13.1 already determines the flags: only final rounding counts; underflow
requires a tiny rounded total and inexactness. The observed FMA flags 0
and conversion flags 24 are implementation defects independently of the
tie decision. They must become 16 whichever nearest candidate is chosen.

North Star §§3.2, 5.1 requires deterministic materialised numerical
semantics and independent comparison. The missing tie decision obstructs
that contract; it is not a missing North Star machine capability. All
seven formats, full input domains, conversions, FMA and the demo ladder
remain required. §13.3 expressly forbids treating missing numerical
implementation as a supported-format restriction.

## Alternatives and concrete witnesses

Let u = 2^-1074, h = 1+2^-52 (binary64 `3ff0000000000001`). The
canonical inputs a=(h,u), b=(1/2,+0), c=(h/2,+0) have exact fused result
h+u/2. Its adjacent canonical results are L=(h,+0), U=(h,u).
Every finite binary64, hence every pair sum, is an integer multiple of u;
their separation u proves adjacency, and both distances are u/2.
L's published high fallback is odd; U's low significand is 1, also odd.

For conversion, take H with binary64 bits `0030000000000001`.
V=H+u/2 is exactly binary128 bits `3c030000000000001200000000000000`.
The adjacent pairs (H,+0), (H,u) have the same odd/odd problem. H is
normal, so this conversion cannot raise underflow merely because its
unrepresentable residual is tiny. These are two different operation paths
to the same decision, not a speculative rare input.

| Alternative | Consequence |
|---|---|
| Compare low significands for both candidates if either low is nonzero; use high only if both are zero | Zero has even low parity, selecting L in both witnesses. Recommended. |
| Always compare high significands | Both highs are the same here; fails to settle these ties. |
| Always compare low significands | Both lows are zero for adjacent small subnormal totals; needs another rule there. |
| Specify nearest ties toward zero everywhere | Deterministic, but changes existing ties with a unique even candidate unnecessarily. |
| Defer to APFloat or use a 106-bit grid | Violates the explicit exact-value priority and loses admitted values. |

The recommendation includes a deterministic fallback if both candidates
receive the same parity. It does not claim an exhaustive theorem that
such a fallback is unreachable across all canonical high-bin boundaries.
This small added policy keeps the published rule total without making
implementers invent a second tie decision.

## Recommended exact spec diff

In §13.1's finite-canonical-pair paragraph in both dialect tracks, replace:

```diff
-with ties to the even low-component significand, using the high
-significand when the low component is zero,
+with nearest ties resolved as follows. Let L and U be the adjacent
+canonical finite values bracketing the exact result. If either low
+component is nonzero, compare the parities of BOTH low-component
+integer significands, treating a zero significand as even. Otherwise
+compare the high-component integer significands. The integer
+significand is the binary64 significand including the implicit leading
+bit for a normal value and without it for a subnormal; its sign does
+not affect parity. Choose the candidate of even parity when exactly
+one is even; if the parities coincide, choose the candidate whose
+exact sum has smaller absolute value. Existing exact-zero sign and
+overflow rules apply separately. This tie rule applies wherever this
+section requires nearest-even canonical-pair rounding, including
+incoming IEEE conversions and FMA; directed and nearest-away modes
+retain their specified rules,
```

The surrounding sentence should retain its existing NaN/inf and overflow
clauses. No change to §13.5's flags table or the exact-sum conversion
paragraph is needed. Add this explicit normative witness immediately
after that paragraph:

```text
For u=2^-1074 and h=1+2^-52, fma((h,u),(1/2,+0),(h/2,+0))
in nearest-even mode returns (h,+0). Conversion of binary128 bits
0x3c030000000000001200000000000000 to PPC returns
(binary64 bits 0x0030000000000001,+0). Each constrained operation,
with initial flags zero and exceptions observed, raises exactly inexact
(16). The unconstrained operations return the same bits without
modifying the floating-point environment.
```

## Hashes, catalog and consumers

This changes numerical semantics at an unresolved tie, not carrier types
or helper names. New arith bitcode and manifest hashes must be published
and driver/plugin consumers re-pinned. Materialised device/native closure
bytes, ISA object identities and certificates can consequently change;
do not reuse old result proofs against new linked bodies. The ISA
`ref@0.9` recipe catalog needs no change. A dialect version change also
changes mapping-version bindings. Keep old publication pins reproducible.

## Conformance vectors and actual verification

Ran `timeout 30 python3 -I -B -u
../bennett-arith/tests/ppc_zero_low_witness.py`, with the 1 GiB address-space
bound and headless environment in [INDEX](INDEX.md): exit 0. Exact rational
adjacency and binary128 representability passed. Both LLVM 18 and 23
native libraries returned L with FMA flags 0 and conversion flags 24.
This successful witness run proves the defect; it is not arithmetic
conformance. Sources and native libraries were read-only.

Acceptance vectors must test both witnesses and their negatives, odd/even
highs, low=0/±u, two-zero-low subnormal neighbors, low-subnormal/normal and
high-bin transitions, exact representable values and both signs. Check
nearest-even, nearest-away and all directed modes, sticky unrelated
flags, observed versus ignored exceptions, final-total underflow and
overflow, and every named conversion route reaching PPC. Include a direct
test of the equal-parity fallback. Do not substitute host double arithmetic
for the exact midpoint. Full-domain arithmetic work remains with 4yg.

## K entry and publication route

**No ISA 0.9 or 0.10 K entry is needed:** the ISA executes the linked
integer implementation and does not own PPC value selection. Proposed
dialect decision row: `PPC-ZERO-LOW-PARITY | §13.1 | compare both lows,
including zero; high fallback only for two zero lows; deterministic
same-parity fallback | 4yg.1 | semantic repair`.
Recommend the next published 0.2 patch (0.2.13 if available) and a matching
0.3-draft amendment, through the dialect council. This proposal does not
silently modify 0.2.12 or close 4yg.1. The existing cross-repo request
already routes the decision; no duplicate blocker is needed.

## Mixed fat object

Apply [fat-object-scope.md](../2026-11-02-rev2/fat-object-scope.md), especially
its separate device-closure and native-container identities (North Star
§§2, 3.3, 4.4). Only marked modules and their required linked helpers need
the revised PPC bodies and device certificates. The corresponding native
comparison uses those same materialised bodies. An ordinary native host
binary and host-only libc need no PPC certificate inventory. No launch,
handle, budget or tape format changes are introduced; the shared note's
container/root association and mixed-launch obligations remain explicit.

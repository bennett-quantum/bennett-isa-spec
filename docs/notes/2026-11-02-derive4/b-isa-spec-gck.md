# gck — retain source diagonal inverse ownership

Bead: **b-isa-spec-gck**. Amendment proposal, never a prerequisite for rungs 3/4; F30 stays deferred until rung 3 or 4 lands.

## Exact defect and clause citations

[ISA 0.9](../../../spec/Bennett-ISA-Spec-0.9.md) §6.1 leaves perm provenance unchanged by diagonal use; §6.4 recompute uses a fresh computing sequence/XOR/adjoint. §11 K81 coalesces the maximal release-ended inverse chain, with each inverse owned exactly once. If that chain absorbs a diagonal inverse into a value-only fresh recomputation, its phase no longer cancels the phase on the original wires.

## North Star gap

A cleanup transformation correctness defect, not a new source capability. North Star §§4.3/5.1 requires the written unitary, including phases. A blanket ban on Z-containing computing sequences or their source regions narrows existing perm cleanup; successful value cleanup alone is not conformance.

## Alternatives and witnesses

The bead's schematic source is `d=qalloc any 1; cx x d; Z d; Z d; cx x d; qfree d`, where Z denotes diag(1,-1). **Literal `z` is not an ISA §4.4 opcode**: the first reference attempt rejected it. The admitted [diagonal-source.bisa](diagonal-source.bisa) expands the forward Z as `s d; s d` and its inverse as `sdg d; sdg d`. Its exact unitary on x is I. Let C=CX followed by S². Fresh C / XOR / C-adjoint clears d but leaves the original phase (-1)^x, hence Z on x. Basis values alone agree; a superposition distinguishes I from Z. [verify.py](verify.py) checks exact signed columns and the reference's forward/adjoint/reverse execution on `0.6|0>+0.8|1>`, with a spectator mutable wire zero. This normalization uses published gates and preserves the counterexample; it is not a parser change.

| Alternative | Assessment |
|---|---|
| Refuse every diagonal source/recompute | Avoids this bad transform by narrowing; unacceptable. |
| Factor C into a monomial permutation and a separately synthesized phase correction | Sound with a complete phase decomposition/placement algorithm, but larger than needed for source-matched diagonals. |
| Keep diagonal inverses at their source points and split value cleanup around them | Preserves source phase order and reuses existing nonzero-history recompute reasoning. Recommended. |

Simply leaving a phase inverse in place while moving *all* surrounding value inverses to the final release is insufficient: the phase may then see a different workspace value. Both sides of a phase boundary need their original intermediate versions.

## Recommended exact spec diff

Append to §6.4 and replace §11 K81's release-chain coalescing rule with the following qualification:

```text
Cleanup has distinct value-restoration and source-phase obligations.
Diagonal operations do not change perm value provenance, but their
matched inverse occurrences remain executable phase operations on the
original source wires under their original effective controls.

When forming a release-ended terminal chain, split it at every matched
diagonal/phase inverse occurrence touching the restored lifetime or its
dependencies. Preserve those inverse leaves at their original source
positions, with their exact primitive/call adjoints and phase parameters.
They are not absorbed into a fresh value-only computation. Do not enter
semantic callees to guess that an arbitrary call is phase-free; only an
exact +1 permutation contract or a checked phase decomposition supplies
that fact.

The maximal intervening phase-free value-inverse segments are separately
owned cleanups. A segment ending before a phase inverse has its ordinal-0
K65 carrier at its last inverse occurrence; the final segment uses the
release carrier. A nonempty primitive segment therefore has such a carrier
even when it would formerly have been absorbed at release. Empty final
segments retain the release carrier. Each source inverse/phase occurrence
has exactly one owner; intrinsic boundaries and joint first-release
ownership remain unchanged within each segment.

At each segment boundary restore exactly the source's intermediate target
versions before executing the next phase leaf. reverse uses the exact
segment adjoint. For perm segments, recompute uses the source-derived
pre-segment input history and the reversible value delta, just as for
K81's independent nonzero-target call pair; it must restore the recorded
pre-computation target values, not necessarily zero. Fresh preparation
and its adjoint cancel their own phases; they do not own an original
source phase inverse. Preserve source ordering, external input versions
and §6.2 windows. Pair/segment formation is independent of the schedule's
chosen realisation and adds no source-program restriction.

K80 attaches fresh replay leaves/call_index values to that segment's
carrier; original phase leaves retain their own source path/point and
caller/callee address. An empty value segment emits no invented gate;
a zero-width value does not erase a nontrivial controlled/global phase.
Disjoint operations are not absorbed, and operations changing a value
seen by the next phase cannot be reordered across it.
```

For the concrete witness the SDG² inverse remains at its original occurrences, cancels the first S², and the final carrier recomputes only CX's value cleanup. With multiple interleaved phases, the intermediate carriers preserve the values each phase originally observes. A more general phase-factorization optimization remains possible under §5.7 after proving the exact unitary and binding its own expansion; it cannot be assumed from perm provenance alone.

## Hashes, catalog and consumers

Source identities stay unchanged in an authorized same-version repair; new-version prefixes change them. Segment/carrier ownership, generated call occurrences, selected schedules, costs and proof subjects change and must be regenerated. No arithmetic recipe is changed by this proposal, but the expansion contract/version and any catalog ownership annotations must be published consistently; do not reuse an old schedule hash with new leaves. VM, scheduler, estimator, circuits and checkers require the same partition.

## Conformance vectors and verification

Require the Z witness on both basis columns and superposition; S/S-adjoint and controlled phases; several interleaved value segments; nonzero pre-targets; joint multi-output/first release; intrinsic boundaries; calls with own addresses/call_index; unrelated disjoint rows; inherited controls; empty/zero-width cases. Assert one owner per inverse and unchanged full unitary, not merely clean released wires. Actual finite/reference checks are in [INDEX](INDEX.md); this note does not claim a phase-aware scheduler implementation.

## K entry: 0.9 versus 0.10

Recommend **0.10 K81 cleanup-partition amendment**, decision `PHASE-PRESERVING-CLEANUP-SEGMENTS`. An in-version 0.9 K route needs **council + maintainer authorization of compatibility rules**, covering changed carriers and invalidated cost/proof assumptions. The current VM's conservative implementation behavior is evidence of caution, not authority to narrow the source semantics.

## Mixed fat object

Apply [fat-object-scope.md](../2026-11-02-rev2/fat-object-scope.md) and [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F03/F09. This transformation is local to a selected device closure/root; native main, argv/envp and libc do not become cleanup carriers or proof records. Per-root budgets/ledger/run/tape and future native-memory ownership remain intact. These amendments never gate rungs 3/4; F30 stays deferred.

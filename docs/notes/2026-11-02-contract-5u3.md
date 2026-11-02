# 5u3 — exact pointer fixpoints without verdict-changing caps

Proposal for the dialect council; `b-dialect-spec-5u3`, linked to
`bennett-plugin` bp-fj1. This job writes no dialect or plugin source.

## Defect classification and North Star scope

Both dialect `Bennett-Dialect-Spec.md` (draft 0.2.12) and the 0.3 draft
§5 require every verdict/candidate/dispatch domain to come from the
specified reference analysis. §5.1 caps INT atoms at K=4096, but does not
cap explicit object offsets or PT6–PT8 memory intervals. PT2/PT5 use
finite-width modular pointer arithmetic. Hence an explicit enumeration
is finite but can have 2^61 members. §5.1 already admits residue classes
and `*`; it does not require enumerative storage.

The bead establishes a severe implementation/performance problem, **not
by itself an inconsistent or undefined reference semantics**. A sound
over-approximation may still change a compile-time verdict, which §5
forbids. The proposed thresholds (4096 offsets, 4 outside extent, 32 for
unknown extent; max(64,extent/4) slots) are not justified as exact. They
must not become normative merely because one plugin uses them.

North Star §§0/3.3/6.1 require arbitrary defined bounded programs,
whole-program analysis and substitutable compilers; §8 rung 4 requires
unmodified SQLite. Widening that introduces a false handle-use/escape
rejection narrows those capabilities. Removing SQLite features, changing
its source, suppressing vectorisation or declaring valid pointer patterns
unsupported does not resolve the issue. Keep bp-fj1 open for practical
convergence; do not declare the rung delivered from a coarse fixpoint.

## Plugin evidence actually inspected

`../bennett-plugin-rung4/.tmp/rung4/PROGRESS.md` reports 177381 slots
plateauing by round 7 on the N=16, LLVM-18 closure, with additional atoms
still growing. The tail of `.tmp/rung4/prof/trace-k.log` shows round 39,
177381 slots, growth in SQLite integer/value sets, then
`§5.2 PT8/PT9/PT12 budget exhausted` at a memcpy in
`@sqlite3CreateForeignKey`, exit 1 after 3401.31 seconds. It does **not**
demonstrate a converged exact reference fixpoint or prove that remaining
growth is only a pointer chain. The supplied earlier 800k-slot report is
historical bead evidence, not reproduced here. No full sibling gate was
run, and no plugin implementation was copied.

## Alternatives and worked recurrence

Take width W=64 and an unfiltered inclusion cycle with one provenance X:

```text
S initially contains offset 0
S includes { (o+8) mod 2^64 : o in S }
```

Its exact least solution is `{o: o=0 mod 8, 0<=o<2^64}`. There are
`2^64/gcd(8,2^64)=2^61` offsets, representable by one residue class.
For W=8 the set is exactly 0,8,...,248 (32 offsets), followed by 0.
Starting at 3 instead gives 3,11,...,251; adding a seed at 4 gives two
cosets, not `*`. Extent=16 does not permit dropping offsets ≥16:
PT2 retains original provenance and adds EXT/top for possibly invalid
addresses; it does not turn numerical out-of-range into host escape.

A full-width same-object memory copy from offset o to o+8 can similarly
propagate one stored atom along that relation. An exact periodic memory
summary must retain storage width, intervals and value/root provenance;
merging all fields into one slot may turn a nearby integer field into a
handle-derived value. That would be a different verdict even though it
is conservative for aliasing.

| Alternative | Assessment |
|---|---|
| Unbounded explicit enumeration | Same reference semantics, often impractical. Not required by the existing text. |
| Normative widening with the bead's thresholds | Deterministic only with further rules for ordering/stickiness/joins; can reject previously admitted programs. Not acceptable without a capability-preserving lowering for every newly coarse case. |
| Exact symbolic sets and translation-cycle closure (recommended) | Preserves verdicts and handles this witness compactly. Does not claim every SQLite recurrence has this form. |
| Resource/time cap with an explicit unfinished build | Honest operational outcome, not a completed analysis or a source rejection. Remains implementation work. |

## Recommended exact spec diff

Add a dialect K/clarification adjacent to §5.1's K and §5.2's least
fixpoint, in both maintained maps:

```text
The reference bounds INT atoms as above; it imposes no additional
cardinality bound on pointer-offset sets or memory intervals. Implementers
MAY use exact symbolic sets, shared interval relations and SCC
acceleration instead of enumerating represented offsets or slots.
All reference sets and byte/representation-width distinctions must be
preserved for verdicts, candidate sets and dispatch domains. A coarser
summary is not the reference result merely because it is sound for
aliasing. An unfinished analysis reports an unfinished build with the
rule and exhausted implementation resource, not a source rejection,
completed verdict or run-time fault for a valid input.

In a pure modular-translation inclusion SCC, if every cycle transfer is
an unfiltered fixed displacement at one bit width W and preserves one
base/provenance coordinate, close the generated displacements as a
subgroup of Z/(2^W). For a single self-cycle of stride s, each seed a
denotes the coset a modulo gcd(s,2^W). Union distinct seed cosets. For
multiple vertices, retain vertex-specific path displacements; do not
merge their seed coordinates. Apply the same rule to a memory relation
only when every transfer preserves the complete storage interval and
representation width and all PT6–PT8 read/overlap results are identical
to the represented individual slots. Retain every ROOT and FN alternative
explicitly. Otherwise use the original rules or another exact method.
PT2's pointer-index width and preserved high pointer bits still apply;
this rule does not replace them by unrestricted full-pointer addition.
```

Add to §24:

```text
Analysis representation/acceleration is implementation policy; the
default result is the exact §5 least fixpoint, including existing K=4096
INT abstraction. Time or memory caps do not alter that result. A report
of unfinished analysis keeps the remaining implementation obligation open.
```

This recommends an exact acceleration and explicitly **declines new
verdict-affecting bounds**. If council instead wants a new abstract
domain, it must supply canonical joins, evaluation order independence,
ROOT/partial-access rules and a lowering that preserves all programs
previously in scope; the current bead does not establish those properties.

## Hashes, catalog and consumers

An exact representation clarification changes no materialised program,
ISA object/certificate or catalog bytes. A dialect publication still
gets its required revision/mapping identity. A threshold widening could
change descriptors, dispatch domains, diagnostics and emitted object
hashes, so cannot masquerade as a local plugin optimization. Plugin and
independent dialect checker should compare normalized exact results on
small-width cases, not implementation data-structure counts.

## Conformance vectors and verification

Enumerate all W=8, stride-8 cases above and compare their symbolic cosets;
W=64 is a symbolic cardinality test, never a 2^61-element allocation.
Test stride 0, stride 1, multiple seeds, a cycle with a filter (must not
use the pure-cycle rule), and GEP with I(a)<P(a). Memory vectors include
same-width full copies, overlapping partial copies and adjacent handle/
integer fields; retain PT6's non-handle top and ROOT distinctions. Check
identical verdicts independent of worklist order. Mathematical cosets are
checked by [contract-verification](2026-11-02-contract-verification.md).
The SQLite exact fixpoint and production gate remain unverified here.

## Release and after acceptance

Recommend a **dialect K clarification on the current 0.2/0.3 tracks**,
not an ISA-0.10 dependency. As stated, 5u3 is primarily an implementation
scalability request. After acceptance, add the finite vectors and let the
plugin owner establish exact convergence on both LLVM majors and the
full rung-4 closure. No new source restriction is authorized by this note.

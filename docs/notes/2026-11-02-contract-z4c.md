# z4c — addresses for action-local region support

Proposal for council review; not a spec edit or an implementation claim.
Bead: `b-isa-spec-z4c`; blocks reference slice `b-isa-spec-fuk.3`.

## Defect and scope

ISA 0.9 §4.6, `qregion`, requires unconditional function-entry allocation
and image preparation, function scope, and function-exit restoration and
release. §16.2 permits a declaration inside either qif arm and inside a
qloop, including a zero-trip loop. §11 Matching requires every generated
leaf to end at a point whose iteration vector has the point's lexical loop
depth. K64 supplies non-row addresses only in a containing **oracle**.
K65 explicitly excludes implicit region restoration from cleanup carriers.

Consequently the following admitted action has mandatory leaves without
legal terminal addresses:

```text
action @gap (t:wire mut) () () () ()
regs {  } bounds {0}
^entry():
qloop 0 {
qregion local 0 1
}
x t
```

The associated signature has `mut:[1]`, `bounds:[0]`. The declaration is
`entry:1`; there is no executed iteration. `iteration:[]` has the wrong
depth and `[0]` is outside a zero-trip loop. Wildcard selectors can match
existing coordinates but cannot create a coordinate. Omitting the leaves
violates §4.6 even for an entirely default schedule. Calling them oracle
support contradicts K64's validation domain.

This is an S0 contract defect under rubric C5/C8: implementers must invent
an address or reject valid source. North Star §4.2 requires structured
coherent regions with unconditional workspace support; §4.4 requires an
independently implementable scheduler contract. Forbidding nested region
declarations would narrow the existing ISA. The fix preserves every
program, including the full SQLite/demo scope; no placement restriction is
proposed.

## Alternatives and worked witness

| Rule | Result on the witness |
|---|---|
| Action-root support pseudo-points (recommended) | Allocate local once at `$construct/[]/0`; execute X once at `entry:2/[]/0`; release local once at `$release/[]/0`. |
| Give declarations a separate hoisted coordinate type | Sound if it explicitly separates declaration coordinates from body iterations; requires another selector encoding and validation rule. |
| Use the lexical point with an invented iteration | `[]` violates depth; `[0]` names a nonexistent iteration. Requires a different normative rule, not an implementation convention. |
| Omit support or reject this source | Violates §4.6; rejected. |

Hand execution: for t=0, forward produces t=1; for t=1, t=0. The local
wire is allocated at zero and released at zero in either case. The inverse
restores t exactly and also allocates/releases the local wire once. The
loop body executes zero times. Resource counts before native decomposition
are one allocation, one X, one release. This is a derivation from the
clauses, not a newly run interpreter test. The bead reports a reference
test and diagnostics; those runs were not repeated in this job.

## Recommended exact spec diff

Insert after §11 **Non-row addresses (K64)**, before Whole-cleanup selection:

```diff
+**Action-local region support (K-new).** For each action or perm_action
+invocation, collect all function-local qregion declarations, including
+declarations nested in zero-trip qloops and in either qif arm. Each
+declaration creates one region per invocation, not one per loop visit.
+Traverse declarations in canonical function text order, recursively
+visiting the then arm before the else arm and a loop body once, for
+function-entry allocation and image preparation. Their terminal point is
+$construct in that action. Function-exit image restoration and release
+traverse the declarations in reverse order and have terminal point
+$release. Within a declaration, use the bound library's region creation
+expansion; restoration/release is its exact inverse. These support leaves
+are unconditional as specified in §3.5 and §4.6.
+Number leaves contiguously from zero within each of these pseudo-points
+in forward instantiated support order, including allocation, image gates
+and release leaves, excluding the action body and semantic callee
+subtrees. Each leaf retains its allocation identity. Their own instance
+and source-qloop iteration vectors are []; outer invocation edges retain
+their actual coordinates. At that action level the support has no step
+or phase and matches only wildcard step and phase, under §11 Matching.
+A selector ending at an enclosing oracle level retains that level's
+existing step/phase interpretation.
+Raw inversion of an already expanded trace retains forward pseudo-points,
+ordinals, invocation edges and allocation identities while adjointing
+operations. Source-adjoint formation continues to follow K62.
+Path validation admits $construct and $release for action and perm_action
+support. $step and $commit remain oracle-only. A reserved point with no
+leaves has an empty generated domain. K64's oracle partition and K65's
+exclusion of implicit region cleanup are unchanged.
```

Add to §0.2, assigning the next free K number at application time:

```text
K-new | §4.6, §11, §15 | ACTION-REGION-SUPPORT: action-local qregion
construction/release have reserved root support addresses, including
zero-trip declarations; no leaf changes. | b-isa-spec-z4c | S | –
```

The traversal rule resolves several declarations and both arms without
mistaking source-loop visits for declarations. It adds addresses, not
cleanup realisations or source instructions.

## Hashes, catalog and consumers

No change to canonical object text, semantic envelopes, `ref-0.9.json`, its
hash or generated leaf order is intended. Unaffected object closure hashes
stay fixed; the bead's witness closure hash is reported evidence, not
recomputed here. Address-bearing schedules/certificates for affected
actions must use the new coordinates, so their own serialized hashes
change. Published 0.8 behavior is untouched. Reference, VM, scheduler and
checker must implement identical action support partitioning; clause
extraction/coverage must include the new paragraphs. A catalog change is
unnecessary if its existing region construction already meets §4.6.

## Conformance vectors and verification

After acceptance, add these vectors to suite F and schedule suite H:

1. The zero-trip witness above: both t basis values, forward/inverse;
   exactly the three leaves and coordinates given above; no loop-body leaf.
2. Two declarations, one inside each qif arm: both constructed once in
   then/else text order regardless of selector value, released in reverse
   order. Nested zero-trip loops add no iteration coordinate to support.
3. `qregion r 0 8 @image` with one-byte image `01`: construction emits its
   allocation then X on bit 0; release undoes image then allocation. Check
   contiguous per-point ordinals and raw inverse identity retention.
4. A caller loop invokes this action twice: callee support has `[]`, caller
   edge has `[0]` or `[1]`; no alias of allocation identities across calls.
5. Concrete action `$step`/`$commit` reject; action support with nonempty
   own iteration/instance does not match; duplicate maximal matching
   entries on an actual support leaf reject by K55. Empty support domains
   cannot witness a tie.

These are proposed vectors, not claims of a passing implementation.
The exact commands used for the derivation were `sed -n '954,1617p'
spec/Bennett-ISA-Spec-0.9.md` and `sed -n '2401,2842p'
spec/Bennett-ISA-Spec-0.9.md`, plus inspection of the supplied bead.

## Release and after acceptance

Recommend a **0.9 K-entry repair**, ahead of the blocked reference slice.
Waiting for 0.10 would leave an existing legal 0.9 source without an
address. Council assigns the K number, applies the text, publishes the
vectors, regenerates the 0.8-to-0.9 diff and clause indexes, and authorizes
the reference owner to finish slice .3. The bead stays open until that
contract/implementation acceptance work is complete.

# 7s0 — bind a restricted-domain proof to an actual borrow frame

Council proposal for `b-isa-spec-7s0` (P1).

## Round-1 blocking items and resolution

Read the [round-1 report](../../../../bennett/docs/council/pass2-7s0-2026-11-02/round-1/report.md)
and both `out-sol.json` and `out-codex.json` in that directory. The table
maps every blocking item to this revision; it claims a proposed resolution,
not a new council verdict.

| Item | Change and witness |
|---|---|
| sol B1; codex B1 — entry_products has no sampled draw/word-consumption rule | Define one union-rank rejection draw, complete B-fiber replays with no random words, and primary-tuple coverage below. Both published seeds select rank 2 / (1,1,0), which CX(c,b) fails. Overlap witness has six distinct tuples, not eight. |
| sol B2; codex B2 — missing mixed-object proof inventory | Append the §12.3 attachment rule below. Frame lifetimes/occurrences resolve in the launched device closure; native main has no borrow-frame claim. |

## Defect and North Star scope

ISA 0.9 §5.6 K76/K78 defines a borrow frame's entry domain D_F and
explicitly permits restricted-domain T3 factorisation proofs. §12.2
says to use D_F for that obligation. But K69/K70 resolves `bundles` and
`regions` only to interface components, and `borrowed` only to lifetime
positions. There is no field selecting a frame or a local workspace
entry value. §12.3 `borrow_frames:[i]` gives only claim indices.
The structural translation witness selects `frame` and B, but its special
V;W;V⁻¹;W syntax cannot represent every admitted finite T3 proof. This is
a concrete certificate expressibility gap (C3/C5/C6), not a defect in
K76's already specified analysis.

North Star §§4.3/5.1 promises dirty borrowing with restored correlations
and frame-local proofs. Neither banning these frames nor promoting a
whole-region basis check into a frame proof preserves that capability.
All interface and borrowed values remain independent and full; only the
normative entry analysis may restrict allocated local values.

## Alternatives and worked witness

Allocate clean c, borrow b, execute the single-gate frame `cx c b`, then
return b and free c. K76 establishes c=0 at frame entry. The two states
`(c,b)=(0,0),(0,1)` are unchanged with exact +1 amplitude, so on D_F
the map factorises as I_c⊗I_b, including arbitrary external entanglement.
This is not a four-part translation-lemma witness. An interface domain
cannot name c; choosing a whole-region input domain does not select
the CX frame either.

Negative prefix: `cx a c; cx x c; cx a c; cx x c` clears c by Boolean
algebra, but K76's all-operand causal stacks do not establish that
constant (the a and x events interleave on c). Thus c is unknown in
normative D_F: all four `(c,b)` values must be checked. At c=1 the
frame flips b, so the borrow proof fails. K78's `cx`/`mcx` identity
normalisation enables exact adjacent adjoint pops, not this stronger
polynomial inference. A phase depending on b would also fail identity
on its entangled reference even if printed basis values agreed.

Union witness: two visits can establish local `(c,d)=(0,0)` and `(1,1)`
respectively. The frame `cx c b; cx d b` restores arbitrary b on both
entry products, hence on their union. It flips b on `(0,1)` or `(1,0)`;
replacing that non-Cartesian union by its rectangular hull could produce
a false counterexample. Conversely checking only the first product
cannot establish the required union. A two-trip body that borrows/returns
b and then flips both initially clean locals supplies these entry facts
and restores the locals after the second trip.

| Alternative | Assessment |
|---|---|
| Strengthen entry analysis to prove the negative prefix clean | Changes K76 and conceals a failing normative frame proof. |
| Restrict proof syntax to the translation lemma | Drops valid CX(c,b) at c=0. |
| Add a selected-frame relation and local entry components | Expresses exactly the existing obligation. Recommended. |

## Recommended exact spec diff

Add to §12.1's exhaustive and sampled productions:

```text
expected:"borrow_frame",
borrow_frame:{lifetime:k,first:<resolved source occurrence>,
              last:<resolved source occurrence>,
              context_key?:<fully resolved §12.1 invocation key>}
```

Add to §12.2 domains, only for this expected relation:

```text
workspace:[{allocation:<resolved allocation occurrence>,offset:n,
            width:n,lo:n,hi:n}, ...]
```

For a non-Cartesian D_F, additionally permit
`domain:{entry_products:[<product-domain>,...]}` instead of one product;
each product has the existing component fields plus `workspace` above.

Append the following normative text:

```text
A borrow_frame claim identifies the checked borrowed lifetime k in
K69/K70 allocation order and the exact maximal §5.6 frame realisation
being checked. Source occurrences include the full
invocation chain, fixed instance and source-loop iteration; oracle
step/phase are included where applicable. They are source coordinates,
not target-scheme leaf ordinals. The first/last points must equal the
frame determined by §5.6, not a selected subinterval. A schedule-bound
claim MUST carry context_key, equal to the fully resolved §12.1 subject
invocation key ([] at the device root); first and last belong to that
invocation. A standalone claim MUST carry the complete K13 invocation
binding for all classical arguments and budget components. Missing or
conflicting bindings reject before replay.

workspace components resolve to allocated local wires live at frame
entry, including flattened local-region storage, excluding interface
and borrowed wires. allocation is the source/expansion allocation
occurrence with its current lifetime, never a reusable storage address.
Offsets/widths select nonempty disjoint intervals within that allocation.
All components use K71's exact width/range checks. In a product enumerate
bundles, borrowed, regions and then workspace in their listed order, last
component fastest; region cells retain §12.2 ordering. This order is also
the component order for Cartesian sampled borrow_frame domains.
The checker computes D_F by K76/K78, including its normative
union over repeated visits, never from the claimed constants. Correspond
local identities by their source lifetime and bit position across visits.
entry_products denotes the union of its product domains: list products
in normative visit order, enumerate each by §12.2, and skip complete
entry tuples already enumerated. It cannot be combined with top-level
product fields. To discharge the frame obligation this union must equal
the normative D_F, with every interface and
borrowed component full and independent and every local entry component
explicitly covered. A strict subset is evidence only. An omitted local
component is fixed at zero for replay and cannot silently supply full
coverage of an unknown component.

For sampled borrow_frame claims using entry_products, form the ordered,
deduplicated enumeration E above and let N=|E|. Each primary draw uses
§12.1's existing rejection sampler on one synthetic integer component
[0,N-1], then selects E[rank], with replacement. Consume no product-choice
or per-component words in addition to that rank draw; N=1 consumes no
words. This rule is exact even when a host must report K75 rather than
materialise E. It does not cap N.

For every primary draw (from a product or entry_products), fix its
non-B entry values and replay the entire B fiber of the claimed domain.
Check B restoration, exact +1 phase and identical complete non-B outputs
across the fiber. Auxiliary B replays consume no generator words and are
not additional primary draws. A changed B value or phase rejects even
before other fiber comparisons. A missing B value cannot discharge the
full-frame obligation. Record exactly sampler.count primary draws and
coverage = number of distinct primary complete tuples / |D_F|; auxiliary
fiber replays do not increase that count or fraction. Overlapping products
never count the same complete tuple twice. Sampled success is always
evidence, including coverage 1; requesting proof rejects.

Replay the selected realised permutation frame from each exhaustive entry element.
Record every surviving output and check all internal release obligations.
Locals surviving this frame need not be zero at its boundary: they are
complete non-B outputs. Region-exit cleanup remains mandatory.
For fixed non-B entry values, varying B over all its values must leave
B unchanged and produce the same complete non-B output, with exact +1
basis phase. This establishes P' tensor I_B on span(D_F); an oracle
relation, output projection or same-input self-comparison does not.
Non-permutation frames cannot discharge this T3 relation.

An exhaustive successful complete-domain claim receives proof for that
selected frame realisation; sampled claims remain evidence and cannot discharge it.
verification.borrow_frames must resolve to covering proofs for every
required maximal frame and every visit/context, or to an applicable
existing structural/library proof. Each selected realisation must pass
on the complete normative D_F, not only on the entry states reachable
at that one visit. A checker may share one verified relation between
visits only after checking equality of their realised frame operation,
wire correspondence and complete required domain. Different schedules
or effective controls are not presumed to supply that equality.
```

The checker needs no expected-output table for `expected:"borrow_frame"`:
the tensor-factor test itself defines the relation. This avoids pretending
that a reference program can receive local c through an interface slot.
Existing K13 before/after tuples for other relation kinds stay unchanged.
A host unable to complete the legal domain uses K75's unchecked outcome,
not a source restriction or a proof from fewer rows.

## Fat-object applicability

Incorporate [the shared scoping statement](fat-object-scope.md#citable-scoping-statement).
Append the following proposed normative paragraph to §12.3:

```text
In a mixed fat object, borrow_frame claims attach to the device-region
certificate records of the identified embedded or linked ISA object and
launched device-root closure. Source occurrences, allocation lifetimes
and invocation contexts resolve within that closure. Required frames of
the complete device closure remain covered, including default-covered
calls. Native main and unmarked host-only code require no ISA region or
borrow-proof inventory. The selected device launch determines the root
and invocation; it does not make the native binary an ISA object.
```

The shared statement lists today's assumptions and the exact remaining
container, launch, budget, handle, tape and host-memory contract changes.
This note does not assume that the device object is the entire binary.

## Seeded sampling witness

Use E=[(0,0,0),(0,0,1),(1,1,0),(1,1,1)] in (c,d,b) order.
For xoshiro256** seed `["0","288230376151711744","0","0"]`, first word
is 90. For seed `["1","72057594037927936","0","0"]`, the first two
words are 9223372036854775826 and 9223372036854781586. Either seed's
first union rank is 2. CX(c,b) maps selected (1,1,0) to (1,1,1), so the
claim rejects. A product-first reading would select product 0 and could
succeed on its B fiber; that reading is now explicitly excluded.

An overlap vector uses products {c=0,d∈{0,1}} and {c∈{0,1},d=0}, with
full one-bit B. Its deduplicated E has six tuples rather than eight.
One successful primary draw has coverage 1/6, even after replaying its
entire B fiber. The two-CX frame in the earlier union witness succeeds
on all four tuples of its complete D_F.

## Hashes, catalog and consumers

New claim fields change certificate bytes only. Source/closure identities,
ref catalog, K78 operation normalisation and schedule leaves stay fixed.
Checkers and producers must agree on lifetime/occurrence resolution and
the exact D_F, including repeated visits and inherited controls. Sharing
physical borrow storage never merges logical lifetimes or proof indices.

## Conformance vectors and verification

Positive: the two c=0 rows; adjacent `cx a c; mcx (a) c` restores the
entry constant by K78. Negative: the interleaved prefix above followed by
the CX frame (must include c=1); missing B value, narrower interface
domain, stale allocation occurrence, nonmaximal frame, phase-on-B and a
sampled label request. Include two loop visits with different entry facts
(both realisations must cover the normative union, not just their own slices),
the two-product c/d union above, two reused borrowed lifetimes, and an
inherited control in full range.
Verify the tensor relation on basis columns, which establishes it with
an external reference by linearity; basis probabilities alone do not.

The c/b truth tables and B-dependent phase counterexample are checked
exactly by [check_rev2.py](check_rev2.py). The originating note cites sched's
`.tmp/cross-repo-beads.md`, “exhaustive borrow-frame binding”. No new
checker, Rust borrow proof or sibling gate was executed for this proposal.

## Release and route

Recommend a **0.9 K claim-binding repair**, preserving K76/K78 semantics.
Keep 7s0 and the originating sched-awo implementation obligation open
until schema, domain resolver and positive/negative checker vectors land.

## Publication and validation boundary

This is a revision proposal, not council acceptance or a consumer implementation.
The default publication rule in spec/README.md requires a full version bump
for semantic changes. The recommended 0.9 K route requires explicit council
and maintainer authorization of its in-version compatibility rules; this note
does not grant it. All unchanged-hash claims assume unchanged header/source
and catalog envelopes. A full version publication recomputes affected symbol,
object, target, schedule, certificate and versioned ref-library identities and
consumer pins even when recipe bodies stay fixed. Published 0.9 bytes remain
available; older closed-schema readers require an upgrade for new fields.

The reproducible finite and document checks are in [check_rev2.py](check_rev2.py):
`timeout 120 python3 -I -B /home/tobias/Projects/bennett-workspace/bennett-isa-spec/docs/notes/2026-11-02-rev2/check_rev2.py`.
Executed for this revision: exit 0, all eight PASS result lines.
These finite checks verify witnesses and revision structure; they implement
no proposed-format checker or host shim. No full repository gate or sibling gate is claimed: scripts/pre-push
uses git/worktree writes, forbidden by this runner, and its setup changes paths
outside this job. Underlying beads remain open for publication and consumers.

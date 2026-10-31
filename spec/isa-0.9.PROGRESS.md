## 2026-10-31 Council round 1 → 2 — item 2 (K63)

COD-R1-B01 / SOL S2 addressed: source point plus integer binding key and
forward semantic-call occurrence is serialized in §11 paths and §12.1
context keys, and included in scratch chains. Fields are instance and
call_index; omitted selectors are wildcards. The existing specificity
tuple is retained, including its ties. K49 now says once per fixed row
instance per step; K52 still changes oracle level only at a call edge
which is a proper prefix of the operation chain. No dynamic lift.call
history returns. Full JSON examples distinguish qinplace G G calls 0/1
and f instances [0]/[1], even for equal values.

Rejected: restoring lift.call histories (breaks static padded rows),
using generated leaf ordinals as call occurrence (callee-local and can
be identical), hashing arguments to identify instances (equal values
must not merge positional keys), synthetic point strings (obscure source
validation). Explicit fields preserve independently checkable source
points. These are address annotations, with no further recipe changes.

## 2026-10-31 Council round 1 → 2 — item 1 (K62)

SOL09-R1-B01 addressed: canonical action adjoints apply the §5.3 source
instruction replacements before recipe instantiation. The adj adapter now
splices that body; raw inverse constructors retain forward ordinals when
reversing expanded machine rows. K28 t-then-f releases remain exact.
Rejected: making the instruction table merely extensional and imposing
leaf-inverse order on action adjoints, which would change K28's scheduled
allocation identity/order. qif retains its complete-expansion inverse.
No capability, legal binding, cleanup realisation or oracle is excluded.

The new private trace test failed on the old adapter (first allocation
width 4 rather than 1); the changed adapter passes the n=1/F=4 trace and
nondefault ordinal-0 class witness. SOL S3 / COD S04 are covered by the
same concept: K28 now labels the one-CX callees as synthetic fixtures.
Count remains 219; recipe body and source bytes change the full-library
hash from fcb2f1e6d61aced0bf9689eab5f6d82c4ae42e9fea8d1b21ae9c7ffccda6f5ce
to 5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197.
The qxor witness hash is unchanged. The H/T/S matrix test now binds the
source-table-produced SDG/TDG/H body; no engine inverse rule changes.

## 2026-10-31 INV-5 second editorial pass — dispositions and validation

Sole draft-text editor; only draft/diff/progress/beads are owned. Read
CLAUDE/AGENTS, §0/§11/§17, prior K29–K48 dispositions and rejected
alternatives, INV-5 in full, b-isa-spec-6hs and the round-18 council
report. SPEC-QUESTIONS initially ended at 32; re-read the concurrent
agent's new entry 33 (round 18) once it appeared, without editing it.
The supplied statement catalog is the dependency-ID authority; sibling
graphs/catalogs/proofs are read-only.

All **17** list items plus the bead are dispositioned below: **15 a**
(text change, with two pairs sharing their concept row), **2 NAD**
under the frozen-library boundary. There are **13 new K rows, K49–K61**,
one per changed concept. Gap 10's token encoding subpart was already
covered by K3/K13 (§12.3); §11 now cites it expressly. Gap 13's full-library
hash already existed in §11.1; the omitted-binding rule now names it.
No proof file, interpreter or conformance source is edited here.
NAD means not addressed in this pass, not a rejection of the capability.
The K55 fix closes **b-isa-spec-6hs**; separate maintainer beads retain
the two library-facing gaps.

Final clause IDs are from the scratch rebuild, not a renumbered sibling
catalog. Declared dependency IDs are the union per concept and were
checked against the supplied catalog. The old proof's ISA-11-P4 is the
accepted-0.8 Matching paragraph; its 0.9 counterpart was ISA-11-P5 and
is now split across ISA-11-P5/P6/P7.

| Gap / proof step | Disposition | K | Clauses changed/added or unchanged coverage | Declared deps (ISA- prefix) | Bead status |
|---|---|---|---|---|---|
| 1 / 1.1.3–4 | a: no lift.call element | K49 | ISA-11-P5.S1/S2 | 4.7, 4.2, 3.6, 3.7 | — |
| 2 / 1.1.5.2 | a: callee token budget | K50 | ISA-11-P5.S4 | 5.2-P3, 5.4, 10 | — |
| 3 / 1.1.6 | NAD: total non-row address/ordinal mapping needs library-facing design | — | ISA-11-P2/P5; ISA-11.1-P29 remain incomplete on this point | — no new sentence | b-isa-spec-8ka OPEN |
| 4 / 1.1.7 | a: wildcard-only absent coordinates | K51 | ISA-11-P5.S8 | 5.2-P3, 5.3, 4.7 | — |
| 5 / 1.1.8 | a: last entered oracle in entry path | K52 | ISA-11-P5.S5/S6/S7 | 4.2, 4.7, 5.2-P3, 3.6, 3.7 | — |
| 6 / 1.2.2 | a: distinct equal maxima; shadows/union cover | K53 | ISA-11-P6.S1/S2/S3/S4 | 11-P3, 4.6, 5.2-P3 | — |
| 7 / 1.2.4.1 | a: kind operation classes | K54 | ISA-11-P6.S6 | 11-P2.S1, 11-P3 | — |
| 8 / 1.2.4.2 | NAD: stable whole-cleanup carrier needs library-facing design | — | ISA-6.4; ISA-11-P6 keep both realisations without inventing a carrier | — no new sentence | b-isa-spec-23g OPEN |
| 9 / 1.4.1 | a: maximal-match validation at fixed stages | K55 | ISA-11-P7.S1–S8 | 11-P3, 9.2, 5.2-P3, 4.1, 3.7, 5.4, 10, 3.4, 4.7, 12.3, 4.6 | 6hs CLOSED |
| 10 / 1.4.3 | a: known admission / unknown first ambiguous operation; encoding subpart b, K3/K13 already cover it | K55 | ISA-11-P7.S1–S8/S10; existing ISA-12.3-P3 | 11-P3, 9.2, 5.2-P3, 4.1, 3.7, 5.4, 10, 3.4, 4.7, 12.3, 4.6 | 6hs CLOSED |
| 11 / 1.6.2 | a: default, never less-specific inheritance | K56 | ISA-11-P6.S7 | 11-P3, 17 | — |
| 12 / 1.6.3.2 | a: object ISA version | K57 | ISA-17-P8.S2/S3/S4 | 9.1, 11.1-P3, 11.1-P4, 11.1 | — |
| 13 / 1.6.3.3 | a: omitted binding names existing full-library hash | K57 | ISA-17-P8.S2/S3/S4; unchanged §11.1 hash | 9.1, 11.1-P3, 11.1-P4, 11.1 | — |
| 14 / 1.6.4.2 | a: eps-independent N, root accuracy, inclusive nested population | K58 | ISA-17-P10.S2/S3/S4/S5 | 8.1, 8.3, 11-P2, 11-P3, 8.2 | — |
| 15 / 1.6.4.3 | a: exact reduced-rational resolved share | K59 | ISA-17-P10.S2/S5/S6/S7/S8 | 8.1, 8.3, 11-P2, 11-P3, 8.2, 9.2 | — |
| 16 / 1.6.5 | a: exact decimal value | K60 | ISA-11-P7.S9 | 8.1-P1, 5.2-P1 | — |
| 17 / 1.6.7 | a: direct uncovered default, no synthetic entry | K61 | ISA-11-P6.S9/S10 | 8.3, 17, 11-P3 | — |
| b-isa-spec-6hs | a: same concept as gaps 9/10, not an extra K row | K55 | ISA-11-P7 | same K55 union above | CLOSED, note names K55 |

Exact 1.4.3 sentence (line wraps removed):

> A consumer MUST reject root-level ties and nested ties whose invocation domains are object-derivable at admission, and MUST reject a tie in a non-derivable nested domain at the first concrete operation in generated order having multiple maximal matching entries, before applying any realisation to that operation, with `E_SCHEDULE_INFEASIBLE` and the diagnostic "§11: multiple maximal matching schedule entries". [deps: ISA-11-P3, ISA-5.2-P3, ISA-5.4, ISA-10]

K55 follows the directed interpreter reading in entry 33, while fixing
the admission classification so implementation-specific inference cannot
move the failure. Literal/forwarding propagation and unresolved root
token parameters define object-derivability; a literal mint in some CC/R
caller does not fix every invocation of a Q root. No token literal or
new object/schedule field is introduced. The derivable-nested branch is
conditional; current Q token parameters leave nested domains unknown.
Root and same-machine lproc domains are known at admission. Unknown
nested domains are neither empty nor universally quantified over budgets.
Concrete rejection identifies the first ambiguous operation in generated
order, before its realisation; the already executed prefix is not claimed
to be atomic. This expressly revises theorem B's unbound-admission
premise rather than asserting the refuted theorem. Entry 33's 0.8
token-encoding gap is already resolved for this 0.9 draft by K3/K13.

Rejected alternatives / S0 cross-check:
- Keep lift.call chains: the two call sites/recursive depths select
  incompatible choices for the same static lproc row. No lproc capability
  or recursive call is removed; only oracle calls create a new step level.
- Read every selector at the innermost operation, or at the owner of a
  path's last point: those change empty-path and call-ending witnesses.
  The chain transition into a callee, including when the entry ends at
  that transition, fixes the selected level. A call row's own terminal
  point does not itself enter a callee for that operation.
- Product-order incomparability, pairwise overlap rejection, duplicate
  collapse, document-order winners: contradict the lexicographic unique
  maximum or reject fully shadowed ties. Count array positions and
  subtract all higher coverage, including union coverage.
- Root-budget clipping and symbolic any-budget rejection: respectively
  under-reject B_nested > B_root and over-reject dead B_nested=2 step 3.
  Optional eager checks using extra runtime knowledge would move rejection;
  K55 fixes the stage and diagnostic instead. No params_hash inversion.
- Store all nested tuples or multi-level selectors: unnecessary schema
  changes for these fixes; retaining direct defaults closes 1.6.7 without
  disturbing an outer step-1 entry. Token tuple hashing already exists.
- Merge omitted fields from lower entries: violates per-entry default
  equivalence. Do not rewrite hashed schedules to materialise defaults.
- Round 0.1/3 to a decimal: loses exact equal sharing and can change §8.1
  filters/buckets. Keep (1,30); (0,1) covers N=0 without division.
  N is fixed before accuracy matching, over the bound-library gates of
  one root invocation including nesting, before target scheme expansion.
  Explicit-share gates remain in N; exact-native gates are excluded.
  The achieved-error filter, cost evaluation and error charging in §8.1
  are unchanged; explicit shares/realisations remain expressible.
- Give non-row operations ad hoc pseudo-points, or choose the first leaf
  of whichever cleanup realisation is selected: neither supplies a total,
  realisation-independent address map. Non-row nested constructor paths,
  step vs row ordinal spaces, empty cleanup and reverse/recompute first
  leaves need a maintained library-facing contract. Stop on those two
  concepts under the frozen-library instruction; 8ka/23g record the exact
  work and preserve execution/charging and both unitary realisations.
- Restrict names, token budgets, recursive lprocs, oracle scope or nested
  schedules to avoid these cases: rejected under rule 10. No accepted
  0.8 text, recipe, source, ordinal, count or library hash changes.

Validation (headless, Python -I; scratch under .tmp/inv5-editorial):
- Baseline/current graphs and statement catalogs rebuilt outside the
  sibling; 49 changed/new normative-classified statements all carry
  markers (including witnesses the extractor classifies as normative).
  All declared IDs exist in the supplied catalog; zero unresolved
  declared dependencies. New K rows form one continuous K1–K61 table,
  with witnesses on every new row and beside the clauses.
- Independent finite two-level witness model: 12 operations, all 6
  shadow-cover and 24 union-cover entry orders, genuine/duplicate ties,
  partial cover, B_nested=3 vs 2, lexicographic longer-path choice, kind
  isolation, outer-step selection and uncovered nested defaults checked.
  Exact Fraction arithmetic gives 1/30, eps_decade=2, log2inv_eps=5;
  all four zero spellings agree. These are contract/witness checks,
  not executor conformance or a mechanisation of INV-5.
- Exact byte checks: accepted 0.8 SHA-256
  90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e;
  entire §11.1 SHA-256
  eab3766ce4cfd301b447230d96b01b10932d6d86ebbefd770789583c3a636d10;
  both unchanged. Library JSON bytes and 219 recipes unchanged; its
  recomputed envelope hash remains
  fcb2f1e6d61aced0bf9689eab5f6d82c4ae42e9fea8d1b21ae9c7ffccda6f5ce.
- Diff generator identified by byte-identical regeneration of the input
  artifact before editing: Python difflib.unified_diff, default 3 context
  lines, splitlines(keepends=True), stable a/spec/Bennett-ISA-Spec.md and
  b/spec/Bennett-ISA-Spec-0.9-draft.md labels. Regenerated after edits;
  raw diff context spaces intentionally retained. Source/progress
  git diff --check passes.
- Final draft SHA-256: 7d44aca7f62bdd07b4c936c10e47515fad770efd0adb56a518a2dfd47bcb1630.
  Diff SHA-256: 50c5ed98d161588bfb5002ddec7a23ca52942bd0e37afe38f0a964018d29de88.
- bd close 6hs names K55; 8ka/23g remain OPEN for the maintainer;
  bd export -o .beads/issues.jsonl performed. No push or Dolt sync.
  Concurrent interpreter edits belong to the other agent and are
  excluded from this editor's explicit-path commit.

## 2026-10-31 Proof editorial pass — final disposition and validation

All five requested beads resolved/closed; no NAD or carried blocker.
The four defect rows are cross-listed with their proof gap, not counted twice.
“60l”, etc. abbreviate b-isa-spec-60l; all bead ids in this table are closed.
a = text addition/amendment; b = already covered/no additional sentence;
the gap-5 analysis is moot under the injectivity repair, not a claim that
the old encoding was harmless. IDs below are from the final scratch graph.
The accepted/spec graphs and statements in the sibling are read-only.

| Item | Disposition | K | Final clause(s) | Declared deps (union for concept) | Closed bead(s) |
|---|---|---|---|---|---|
| INV-1/1 | b | K4 | ISA-4.5-P5 (existing) | — existing | 60l |
| INV-1/2; qif defect | a | K29 | ISA-5.3-P4/P6 | ISA-3.5, ISA-4.6-P7, ISA-5.1 | bdv; 60l |
| INV-1/3 | a | K30 | ISA-4.6-P10 | ISA-3.3-P3, ISA-4.6-P8, ISA-4.6-P9 | 60l |
| INV-1/4 | a | K31 | ISA-5.3-P7 | ISA-2-P10, ISA-3.3-P3, ISA-4.5-P5, ISA-4.7, ISA-5.3-P4, ISA-5.4, ISA-6.1-P1, ISA-6.2 | 60l |
| INV-2/1 | a | K32 | ISA-3.7-P26 | ISA-3.3-P1, ISA-3.7-P1, ISA-16.2 | 60l |
| INV-2/2 | a | K33 | ISA-3.3-P4 | ISA-3.3-P3, ISA-4.4 | 60l |
| INV-2/3 | a | K34 | ISA-3.3-P8/P20 | ISA-3.3-P7, ISA-10 | 60l |
| INV-2/4 | a | K35 | ISA-3.3-P15 | ISA-3.3-P4, ISA-4.7, ISA-4.9 | 60l |
| INV-3/1 | a | K36 | ISA-6.1-P17 | ISA-4.6-P1 | 60l |
| INV-3/2 | a | K37 | ISA-6.2-P1 | ISA-5.3, ISA-6.1-P1, ISA-14.2 | 60l |
| INV-3/3 | a | K38 | ISA-6.2-P1 | ISA-3.3-P3, ISA-6.1-P1 | 60l |
| INV-3/4 | a | K39 | ISA-6.2-P1 | ISA-2-P10, ISA-3.5, ISA-5.3-P5 | 60l |
| INV-3/5; other-value defect | a | K40 | ISA-6.2-P1 | ISA-5.3, ISA-6.1, ISA-6.4 | 0jo; 60l |
| INV-3/6 | a | K41 | ISA-6.1-P18 | ISA-3.3-P3, ISA-5.1, ISA-5.3-P1, ISA-6.1-P1, ISA-2-P10, ISA-3.5, ISA-4, ISA-14.2, ISA-5.2-P4, ISA-5.6-P1, ISA-4.4, ISA-6.3 | 0jo; 60l |
| INV-3/7 | a | K42 | ISA-5.6-P8 | ISA-2-P10, ISA-3.5, ISA-5.3-P5 | 60l |
| INV-3/8 | a | K43 | ISA-5.6-P8 | ISA-4.3-P2, ISA-4.3-P4, ISA-4.6, ISA-5.6-P2 | 60l |
| INV-3/9 | a | K44 | ISA-4.3-P5 | ISA-3.3-P3, ISA-4.3-P1, ISA-5.1, ISA-6.4 | 60l |
| INV-4/1 | a | K45 | ISA-9.2-P4 | ISA-3.7-P1, ISA-9.1-P1 | 60l |
| INV-4/2; tie-order defect | a | K46 | ISA-9.2-P2/P4 | ISA-9.1, ISA-9.1-P1, ISA-16.1 | 4it; 60l |
| INV-4/3; injectivity defect | a | K47 | ISA-9.2-P2/P4; ISA-16.3-P6/P7 | ISA-9.1, ISA-9.1-P1, ISA-16.1 | ccv; 60l |
| INV-4/4 | a | K48 | ISA-9.2-P5 (non-normative lemma) | ISA-16.2, ISA-4.8, ISA-9.1, ISA-16.1 | 60l |
| INV-4/5 | b | K47 | ISA-9.2-P4 (covered by injectivity repair; harm analysis moot) | ISA-9.1-P1, ISA-16.1 | 60l |

Validation, headless Python -I and read-only conformance clause tooling:
- Scratch baseline/current graphs and statement catalogs rebuilt outside the
  sibling. Every changed/new normative prose sentence has a declared marker;
  every new dependency id exists in the supplied baseline catalog; zero
  unresolved declared dependencies. The two changed pseudocode sentences
  also end with physical markers (the statement extractor treats code as a
  single example block); the operative prose repeats their declared edges.
  The non-normative grammar lemma also declares its reasoning dependencies.
- 20 new rows, K29–K48, in one continuous table, each with a witness.
  35 new physical dependency markers (43 total; 41 parsed outside fences).
- Independent witnesses: old dotted collision reproduced; 16 key paths
  distinct as JCS arrays; both cross-object collision and exchanged-base
  collision separated; all 24 permutations of four leaves encode identically
  and round-trip. Exact owner selection is structural.
- qif lifetime-order counterexample checked: old inverse consumes before
  produce; reversing the complete expansion is legal. Loop visit identities
  advance. Independent exact CX/phase matrices fail I_G tensor V, while
  CX/CX cancels; independent state vector gives clean d1 and dirty d0
  probability 1/2 for SQ25. These are contract/witness checks, not 0.9
  executor conformance or a completed mechanisation of the four proofs.
- Source/progress git diff --check passes. Raw diff context spaces are
  intentional and verified by exact regeneration, not whitespace stripping.
- Accepted 0.8 SHA-256: 90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e.
- Entire §11.1 SHA-256: eab3766ce4cfd301b447230d96b01b10932d6d86ebbefd770789583c3a636d10 (unchanged).
  Library JSON bytes unchanged; 219 recipes and published envelope hash
  fcb2f1e6d61aced0bf9689eab5f6d82c4ae42e9fea8d1b21ae9c7ffccda6f5ce
  unchanged. No recipe-relevant edit or §11.1 gap required a blocker.
- Final draft SHA-256: d75463a62232d574f5c5e946aec3b439e0fe3e312ca3afd95267c8cc717341bd.
- Regenerated spec/isa-0.8-to-0.9.diff SHA-256:
  2def9456deb35f9e7935d8e0900bbe50abdf3b3681a710a0c9624605249edcbf; stable a/spec/Bennett-ISA-Spec.md and
  b/spec/Bennett-ISA-Spec-0.9-draft.md labels.
- Group commits so far: 3b35aa3 (lifetimes/matching/alias rules),
  91c60dd (cleanup/borrow soundness); final envelope/audit commit contains
  this record, including S0 cross-check corrections to K31/K38.
- bd export -o .beads/issues.jsonl performed for every group. No push,
  remote sync, interpreter/conformance source edit, or worklog edit.

Final soundness cross-check: K31 binding restoration is expressly
conditional on legal §6.2 cleanup. Otherwise H;entangling reader;H could
masquerade as a restored allocation version and bypass K40 through K44.
Matching alone never removes a reader/phase obligation; opaque nested
pairs still restore bindings once their own cleanup conditions hold.

Final S0 reread: K38 names external input versions explicitly for other
values too (which do not have a perm S), and excludes pre-computation
targets; this prevents an undefined-set reading. Consolidated K29–K48
into the existing §0.2 Markdown table, retaining one row per concept.

## 2026-10-31 Proof editorial pass — group 3 (K45–K48)

INV-4/1 a K45: exact original owner keys and exact lproc global keys.
INV-4/2 + 4it a K46: emit every leaf, unique unsigned UTF-8 order of
the JCS key-array bytes, independent of values. INV-4/3 + ccv a K47:
render that array on the line. These replace dotted ordering and the
SQ31 tie break together: a tie-break cannot preserve a lost key split.
The two bases can be exchanged without losing ownership information.
Scalar paths are arrays too; the example-2 sum envelope now prints
layout ["globals","zero"] with its original leaf value. Draft example
hashes retain the existing precheck placeholders.

Dialect read-only check: §16.3 Names and layout uses ref as its reference
oracle and shape-region keys; the same table explicitly emits dotted
block names (^b<i>.s<j>, etc.), and §12 Naming uses %nz.<tag>.<r>.
This does not prove interface-region keys can never have dots, and it
does not justify restricting every legal ISA producer. Select JCS arrays
as requested; reject the NAME narrowing, whole-line/value tie breaks,
component tie breaks without encoded components, and lossy maps keyed by
dotted paths. JCS array syntax plus LF-escaped values is injective on
structured leaf/value pairs (not a claim that SHA-256 has no collisions).

INV-4/4 a K48: non-normative grammar lemma next to §9.2 establishes
recoverability without changing any envelope length framing. A function
header/body line cannot be a data/layout/device suffix or a bare hash.
Result names data/layout/device are followed by "="; in part (5a) the
next token must be NAME, (5b) an array, and (5c) a name. §4 has none of
those three opcodes. For a data symbol the known signature kind picks
one payload line, and suffixes are empty. Prefix classes distinguish
(5a–c); JSON LF is escaped. Reject unnecessary length prefixes (would
change every symbol's bytes for no grammar need).
INV-4/5 b K47: the requested harm assessment is explicitly moot after
injective structural paths; no semantic decision relies on CC/R being
unable to observe the lost distinction. The defect's encoding failure
is repaired regardless. This is not a NAD claim about 0.8.

No §16.1 NAME restriction, accepted 0.8 edit, or §11.1 edit. New rows
this pass: K29–K48 = 20 (one per changed concept; K4 covers the already
specified check timing, and K47 covers the moot analysis).

## 2026-10-31 Proof editorial pass — group 2 (K36–K44)

INV-3 gaps 1–9 all disposition a; gap 5 also resolves b-isa-spec-0jo.
Nine separate concepts/K rows preserve the provenance table, window
conditions, exact control identity, and the SQ 25 soundness correction.

K41 defines “depends on” by an exact joint residual-operator condition
M = I_G tensor V, with arbitrary G/rest/reference states, phase included.
This is the semantic meaning of no outstanding reader, rather than an
implementation-private approximation to a dependency graph. Transitive
reads through cleaned scratch are retained by composition. Checking
exactly or proving factorisation structurally/library-wise is a checker
mechanism; two conformant implementations have the same predicate.
It is applied to other values only: perm values still permit diagonal
readers and their declared phases (T5). No measurement-based cleanup,
inert non-permutation call, or basis-only test substitutes for it.
The complete computing sequence can include its reader and uncompute,
so valid larger compute/uncompute constructions remain expressible.

Witness derivation: with p=1 and t=0, after H(d0), CCX(d0,p,d1),
CX(d1,t), CCX(d0,p,d1), d1 is zero but t=d0. The middle operator on
d0/t is CX, not I(d0) tensor V(t), so K40 forbids the final H cleanup;
H;H with no middle operation is permitted. A diagonal S on d0 also
fails the factorisation, while S;SDG restores it exactly (phase included).

K37's “undone” requires matched changes and basis-preserving intervening
operations, not mere equal final bit patterns. K38 pins the produced g
version, rather than the impossible pre-computation target version.
K39 binds the computing control set/versions explicitly. K42 includes
inherited predicates in translation-lemma side conditions. K43 chooses
SQ 30 static-body maximality and never joins separate borrowed lifetimes.
K44 states the static qfree test alongside its existing dynamic clean
condition; matched inverses restore the allocation version. K36's version
zero is relative to the image, including a nonzero image.

Rejected: allowing H cleanup with an entangled interface reader; deleting
a transitive dependency when its intermediate scratch is cleaned; ignoring
diagonal/global phases; imposing the other-value rule on perm cleanup;
using current bit equality to roll back cell version counters; lexical
predicate-stack identity; maximality across dynamic loop visits.
Witnesses are on K36–K44. Entire §11.1 and the 219-recipe library remain
byte-identical; none of these clauses changes a recipe.

## 2026-10-31 Proof editorial pass — group 1 (K29–K35)

Sole implementation editor; owned draft/diff/progress/beads only. Read all
CLAUDE rules, §0, this progress file, the four structured proofs and their
format/index, both clause graphs and the statement catalog, the five task
beads, SQ 25/30/31 (also SQ 28), and dialect §16.3 Names and layout.
Impact cones queried in both graphs before changes (local evidence under
.tmp/proof-editorial-pass/impact-before.txt); the committed draft graph is
a baseline snapshot, so final clause ordinals are verified on a scratch rebuild.
No accepted 0.8 or §11.1 recipe text change is authorized.

The index/task's “18 gaps” excludes the four defect entries: the four
numbered Spec gaps lists actually contain 4+4+9+5 = 22 entries. This pass
accounts for all 22, with defect/gap duplicates recorded once.

- INV-1/1: disposition b, existing K4, ISA-4.5-P5 (also ISA-1.2-P1):
  every executing qinplace needs a covering verified inverse-pair entry
  before any effect of the enclosing CC/R call; failure faults atomically.
  The existing sentence already says the requested timing/outcome.
- INV-1/2 + bdv: disposition a, K29. §5.1 (accepted and draft) matches
  lifetimes region-wide; neither says “within each arm”. Allocation and
  release are unconditional under §3.5. The supplied program is therefore
  well-formed: forward allocates then frees d, inverse must allocate in
  inverse else before freeing in inverse then. NAD was considered and
  rejected after reading the actual §5.1 text. Retain the self-contained
  shorthand, but use the complete expansion generally; no capability lost.
- INV-1/3: a K30, per-visit names/types, including reallocated c and
  zero visits (no binding transition); only consumed locals disappear.
- INV-1/4: a K31, exact classical/token/callee/version key and set-valued
  controls; inverse pairs restore wire bindings, never roll back cell
  version counters. Same-token binding follows §5.5; opaque pairs remain
  available without inlining.
- INV-2/1: a K32, regs restricted to locations in oracles/lprocs; an
  action has no source of an unproduced wire/classical local. Empty
  regs/bounds remains supported.
- INV-2/2: a K33, denoted-wire distinctness; the read-only overlap with
  inherited controls allowed by §3.3 remains allowed.
- INV-2/3: a K34, fresh allocation (including viewed parents) and atomic
  E_ARG for overlapping launch arguments. from_id aliases are still
  references to a single ledger register, not overlapping new registers.
- INV-2/4: a K35, runtime alias check before gates or view transitions,
  CC/R FAULT_UB_TRAP; static Q interface-disjointness remains as specified.

Rejected: arm-local qif lifetime bans; keeping the ill-formed adjoint
shorthand for cross-arm lifetimes; using stale first-visit loop names;
lexical-stack matching; treating spelling inequality as disjointness;
banning qreg aliases or requiring predicate/read-only input disjointness.
Witnesses are named on each K row; these are contract edits, not claims
that the read-only 0.8 interpreter executes 0.9.

## REF-BINDING / REF-QUANTUM completion — 2026-10-30

K19/K20 extend §11.1 within A1's serialization/envelope/ordinal contract.
Typed positional wires, lexical body closures, linked closure-hash symbols,
effective-control environments, angle bit patterns and ordered wire lists
replace the previous integer-only binding gap. The qxor witness is unchanged.
Witness tests cover negative zero, inverse sign bits, width rejection and
variadic mcx operands; additional binding/quantum witnesses live in conformance.

Rejected alternatives: textual name substitution captures caller scratch;
flattening symbol calls loses invocation-local ordinals; decimal angles lose
signed zero and exact input bytes; angle arithmetic introduces independent
rounding choices; fixed-arity mcx specializations omit larger k; permutation
checking H loses amplitudes/global phase. Classical events stay canonical
bound instruction events, not private quantum opcodes. Matrix comparison for
arbitrary angles is explicitly numerical evidence, never an ε=0 proof.
Publication remains gated on a complete catalog and accepted 0.9.

# ISA 0.9 draft — progress file (resumable)

## VM certificate/estimate contract follow-up — 2026-10-30

Source: `bennett-vm` commit `849f7f7b66e11cbc8c6c56ecbefad9d879fe58ed`,
read-only `docs/PROGRESS.md` ("Implemented claims and exact obligations",
"Launch estimate and coverage behavior", and its verification limits),
and `bd show bennett-vm-hsd` / `bd show bennett-vm-clw` in that repository.
Read ISA 0.8 §8–§10 and §12, draft §0.2 and the ISA-7, Q14,
EPS-DECADE, BLOCK-IMAGE and ISA-REF-LIB additions; CLAUDE rules 1, 5,
8, 10 and 11 apply. No implementation code crosses repository boundaries.
Only the owned draft, this progress file, conformance and Beads export
are changed; the committing orchestrator owns its worklog. No interpreter
source/build, accepted 0.8 edit, ABI header edit, push or remote sync.

### K12 / LAUNCH-ESTIMATE — §10, §10.1, §12.3

Mirror **b-isa-spec-c15**, bug, label `isa-0.9`, source **bennett-vm-hsd**.
**maintainer decision required: request API, class keys and decimal policy**.

Proposed normative text adds `params_jcs, params_len` to the 0.9 estimate
request (also carrying explicit variant/inherited controls as in §11),
with every classical argument resolved in signature order and
no guessed zero/previous-run/hash default. Token arguments carry their
own tuples; the outer token fixes the request budget. Host and live
device-minted tokens are admitted, with tuples copied while valid. Scheduling uses
matching object bindings or §11 defaults, never a selected basis run in
place of the padded generated realisation. Every resource is reported:
scalar count/time and one occupancy/peak row per target class (zero rows
included), using injective UTF-8 percent-escaped keys. Rational aggregation
stays exact; display rounds upward to 34 significant digits, with a
canonical finite decimal spelling. Certificates bind the resolved request
and effective realisation, contain exact reduced rational rows, and must
recompute them and their displayed enclosures. The existing expansion
library gate remains; no end-to-end estimate implementation is claimed.

The maintainer has materially different, non-narrowing choices:

- **API/defaults:** the printed proposal changes the versioned 0.9
  signature and requires a complete explicit vector (`[]` only for no
  arguments). Alternatively keep the four-argument function and add a
  separately named bound-request function or request object carrying
  concrete parameters; the old entry must resolve a published concrete
  binding (not invert `params_hash`), or forward to the explicit entry.
  Both must support every well-typed parameter vector. Neither invents
  one basis input or drops parameterised estimates.
- **Class representation:** escaped flat keys preserve `estimate_get`'s
  pair-of-strings accessors; a class-aware accessor with resource and
  class returned separately preserves the same vector. These have
  different ABI/consumer shapes; scalar summation is not an alternative.
- **Decimal policy:** the printed proposal uses a fixed 34-digit upward
  enclosure plus exact rational certificate values. A request-selected
  precision with an explicitly bound rounding/enclosure policy is also
  sound and supports every rational cost; it produces different bytes
  and requires an extra request/certificate field. 0.8 chooses neither
  the precision nor the direction. Fixed nearest rounding would need a
  separately published error enclosure and could not serve as an upper
  bound by itself. No policy may call a finite decimal exactly 1/3.

Witness: §10.1 `LAUNCH-ESTIMATE witness` assumes independently specified
live intervals of duration 1/3: class A peak 2, class B] peak 1, idle
peak 0, occupancies 2/3, 1/3, 0; separate rows escape B] as B%5D.
The two nonterminating values round upward at the final digit only.
This is aggregation arithmetic, not a guessed padded gate count.
`G-estimate-resolved-class-decimal` records the 0.8 contract gap and is
**unavailable pending 0.9**, naming both source and mirror beads.

### K13 / CLAIM-REPLAY — §9.2, §12.1–§12.3

Mirror **b-isa-spec-9uy**, bug, label `isa-0.9`, source **bennett-vm-clw**.
**maintainer decision required: sampled default, invocation and tuple encoding**.

Proposed normative text gives sampled claims an explicit `expected`
relation (oracle, table or linked reference program) and subject; the
checker independently computes both sides and compares complete tuples,
clean workspace and permutation phase where asserted. A reference-only
claim is distinguished from a schedule-bound realised claim. Canonical
rows include every interface input/target/fault, dirty bundle and cell.
Standalone table/program/structural replay binds resolved `args` and
budget explicitly; parameter-uniform structural claims can bind
`args: "all", budget: "all"` only with every side condition checked
universally, retaining generic library/adjoint/translation proofs.
Scheduled certificates bind the root and resolve all nested contexts,
including defaults. The exact K4 inverse-pair domain /
arg_tokens binding remains authoritative and is not replaced by one
sample argument. Typed unsigned decimal bit-vectors, binary64 bit-pattern
hex and token tuples give a precise JCS params_hash preimage. Evidence
coverage counts distinct draws; sampling remains evidence at full coverage.

The maintainer has materially different, non-narrowing choices:

- **Sampled default:** the proposal requires `expected` and supplies no
  omitted-field default. Alternatively omitted `expected` can mean
  `"oracle"`, with an explicit schedule-bound subject and the same
  independent oracle comparison; explicit table/program relations remain
  available. Both retain sampling and prevent identity self-comparison;
  they differ on whether the old payload is complete without that field.
- **Invocation location/encoding:** the proposal puts a resolved invocation
  on each standalone claim (or its region) and the scheduled root in
  `bindings.invocation`, hashing the canonical args array. A shared
  invocation registry with explicit ids and validated immutable records
  would support the same invocations and avoid repeated vectors; it has
  different interchange/hash/lookup mechanics. Complete typed little-endian
  hex values are another lossless parameter encoding. Neither unresolved
  bounds nor an opaque token id can replace concrete replay bindings.
- **Tables:** the proposal uses complete positional before/after tuples
  and a row for every element of the declared domain, even for sampling;
  the independent program relation remains compact for large domains.
  Fully named complete tuples, or a keyed table limited to the distinct
  sampled inputs with mandatory total lookup for those draws, are also
  non-narrowing relations, with different size/order/coverage contracts.
  Dropping arbitrary target/fault or dirty/cell components is not sound.

Witness: §12.1 `CLAIM-REPLAY witness` uses x=1, a=2, initial target=2,
fault=3; the two-row add/retval oracle at B=2 commits payload 3, so the
final target is 1 and fault remains 3. Identity self-replay would miss
this difference. a=1 instead gives target 0; B=1 gives fault 2. One
preserved dirty bit and two preserved two-bit cells remain in the full
11-bit domain. Four singleton samples consume zero generator words,
cover one of 2048 tuples, and give evidence. §12.3's additional
`CLAIM-BINDING witness` hashes an i128 value >2^64, negative-zero f64
bits and a token tuple; +0 changes the hash, a token id cannot enter it.
`D-sampled-relation-invocation` is **unavailable pending 0.9**, naming
both beads, and invents no 0.8 replay CLI or executable certificate.

Both mirrors stay OPEN pending maintainer decisions and accepted 0.9.
These entries add capability contracts; no NorthStar certificate, @cost,
full-domain proof or padded SQLite/kernel estimate scope is removed.

Verification: the extended independent witness test failed before the
LAUNCH-ESTIMATE witness existed, then passed. Python Fraction/Decimal
checks reconstruct exact class aggregation and upward rounding (including
decade carry and a >34-digit integer), table XORs, binding-sensitive
outcomes, distinct coverage and the parameter hash without importing VM
or interpreter code. Headless offline `cargo test` passes all **25** Rust
tests; the full 0.8 harness reports **445 pass, 0 fail, 48 unavailable**
(**493** cases), exit **2**, including the two new contract-gap cases.
Exact commands and observed binary fingerprint are recorded at the top
of `conformance/PROGRESS.md`. Draft witnesses are not 0.9 executor conformance.

Accepted 0.8 SHA-256 remains
`90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e`.

## VM milestone 4 follow-up — 2026-10-30

Resumed the uncommitted implementation after the infrastructure outage.
Source context: read-only `bennett-vm/docs/PROGRESS.md`, `HANDOFF.md`, and
`bd show bennett-vm-47t bennett-vm-cxg bennett-vm-clh bennett-vm-47i` from
that repository. No VM or reference-interpreter source was edited.

Draft additions, all recorded in §0.2:

- **K9 / EPS-DECADE**, §8.1, mirror **b-isa-spec-f31** (VM cxg, open):
  zero-based min{j ≥ 0: eps·10^j ≥ 1}, exact rational comparisons,
  requested share, domain (0,1], unbound/out-of-array errors, and the
  exact-entry prohibition. The six printed boundary cases give indices
  0,1,1,2,3,4. This is a proposed 0.9 definition, not a guessed 0.8 rule.
- **K10 / BLOCK-IMAGE**, §7.3, mirror **b-isa-spec-0d0** (VM 47i, open):
  decode the named data symbol's hex in textual order into contiguous
  complete 512-byte sectors, with partial images rejected at device
  initialization before execution; empty/absent images, present zero
  sectors and absent-sector reversal are explicit. The 1024-byte witness
  distinguishes present zero, present ff and absent sectors.
- **K11 / ISA-REF-LIB**, §11.1, mirror **b-isa-spec-8r4** (VM 47t, open):
  **maintainer decision required: decomposition set and hash rule**.
  Proposed JCS serialization, ISA/library-versioned LF envelope, full-payload
  SHA-256 rule, expansion order, ordinals, scratch identities, removable
  subset and decomposition families cite their motivating 0.8 sentences.
  The single two-bit qxor recipe is only a witness: its four-line envelope
  hash is `299510fd67cc626fdcbcc10adcd40730f9af294ba596d86fb25fd31be08a5c7f`.
  No complete catalog, full-library hash, `ref/` directory or 0.8 artifact
  is published. Publication waits for the maintainer decisions and accepted
  0.9; implementation/catalog correctness and full-domain T3 remain required.

**b-isa-spec-1pl** mirrors VM clh and is closed after fixture verification.
It corrects conformance data under existing 0.8 §8.1, not the rule: canonical
target bytes have no trailing LF; the invalid exact entry now actually
references eps, and a separate constant-cost exact entry must accept every
share. The expected hash already described the no-LF bytes: the old file
hashed to `52cd3dcd6ca4634f311f4ba0b15806c52763e70504f1220ef5c2d64d10676ecb`,
while the corrected file hashes to the existing expected
`71e1973c642abe3ea4ae5b4c7211fd4e2b2b7817894ca958d138c992cc321596`.
The VM report's claim that the expected hash included LF was not reproduced.
All four mirrors are bugs labeled `isa-0.9`; only clh is closed.

Verification: the added fixture, precise-probe and witness tests failed
against a copy of committed baseline files under `conformance/.tmp/resume-red/`.
The inherited parser also accepted an empty alternative; a new regression
failed before validation was added. Green: all **25** conformance Rust tests,
including independent exact-rational/sector/witness-hash and 16 forward/reverse
qxor tuple checks. No 0.9 executor conformance is claimed. Full reference run:
**445 pass, 0 fail, 46 unavailable (491 total), exit 2**. Library-dependent
cases retain unavailable reasons naming b-isa-spec-8r4. Exact headless commands,
reference binary fingerprint, staging limits and per-suite counts are in
`conformance/PROGRESS.md`. `include/README.md` records the library gate;
no ABI declarations were changed.

Accepted 0.8 SHA-256 remains
`90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e`.
The older progress entries below are historical; the separate tracked
`spec/isa-0.8-to-0.9.diff` is outside this task's owned paths and is not regenerated.

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
| 10, through / 10, worker / 10, plus_db / 10, flip_arg | 1 each | ex 10 symbols line (Q14 continuation) |

**Cross-check values (NOT normative, not in the draft).** Computed by importing the *unmodified* seed
`reference-interpreter/isa_check.py` (functions `extract_examples`, `parse_object`, `compute_hashes`) on the
draft text before the placeholders were made distinct; examples 2 and 10 were recomputed after Q14. The seed has no CC/R closure set and no part-(5c) device
lines, so examples 4 and 8 (CC/R functions with a layout; example 8 also a device) are omitted as unreliable;
for the others the seed's model is complete (oracle, lproc, data, action, and memoryless CC/R with no devices).
The 0.9 precheck must compute its own values; these are only for comparison.

```
ex1 inc8        ed9865300b85a519c2379de98e36e635e72b5b78d8bdfca03dccf9a09aa3df5e
ex2 zero        43e032f98d7aff20f6864d6368b20ab4ef9d810938999acff8d71ed16e6b17d7
ex2 sum         498cfae2ec3e3d3322997010572345b34be5459fb7ad69edc5c9e51e10dc1e00
ex2 sum_db      443e5ab7fd4f005efbde4664972d709b0e9c7995eb8842271aebbdc78a52570f
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
ex10 plus_db    cab2fb72ce9ebea9f5942ca9956e4f1906535f095c6cf0a840f45bf4baa37c5a
ex10 flip_arg   bfa13d9a564e3f8562098c4ea39330d947efa64521310afad6a329dc594bd671
ex10 worker     ab885fe1ec4d35b98ddb1cfbcde0ad18e747343af5155bb2beff1aa618fc1563
ex10 through    fbd06a460dd2bf2cd6b19ab101f023f1ed2002eaf35117ac2e83c52dce4ac76c
```

Precheck notes: the seed's printed-table regex needs 64-hex values (the placeholders will fail it until
patched, as intended); its `--repatch` and its executable checks of inc8/sum_db need `0.9` acceptance; examples 4–10
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
  consequence filed as bd 7u6 (lprocs have no token/args source); resolved by the applied Q14 edit below.
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

Q14 — **APPLIED (2026-10-29, b-isa-spec-7u6)**. The second alternative is applied: lprocs take explicit
token/classical parameters. Source: the maintainer's acceptance recorded in
`worklog/001_2026-10-28_bootstrap.md` line 26 as "the Q14 edit (lproc token/classical params)";
the orchestrator's 2026-10-29 decision selects that alternative. The draft's §0.2 now records the applied edit
and source; the earlier pseudo-location/captured-environment alternative is not the adopted mechanism.

Sections edited: §0.1 (E17.14), §0.2 (applied/source), §2 (terminology), §3.3 (argument matching),
§3.6–§3.7 (header groups, explicit forwarding, fixed binding instances, location/stack layout and charge),
§4.2 (lifted-call operands), §4.7–§4.8 (types and opcode schema), §5.1 (roles), §5.4–§5.5 (token validity,
nested budgets and matched cleanup), §9.1–§9.2 (signature, hash and link consequences), §15 (suites C/F),
§16.2–§16.3 (grammar, example 2's empty signature arrays and new example 10).

Header: `lproc @f (params..) (result) (tokens..) (classical..) (mem..)`.
Groups are `reg[N]` parameters; one `reg[T]` result; `token` parameters; `iN ptr f64` parameters;
`region[A,W] in` parameters. Optional `token` / `classical` suffixes agree with their header group.
The full call is `w = lift.call @f (a_1..a_m) (tokens..) (classical..) (mem..)`; the original one-list form
is canonical exactly when the three new groups are empty. Tokens keep their identity and validity; classical
values remain fixed before oracle entry. Instantiating each lproc for the finite set of forwarded entry-binding
keys handles different coherent call paths and recursion without selecting classical data from wire values.
The fifth group forwards mapped regions so every lifted call available to an oracle remains available in an
lproc, including calls with memory interfaces. Bindings add no rows, captures or stack cells; ordinary
`lift.call` / `lift.ret` and lifted region-call charges remain unchanged. Nested oracles use their explicit
token's tuple, not the enclosing oracle's tuple. The required signature arrays `tokens`, `args`, `mem`
(including empty arrays) and canonical operand lists enter existing identity/hash rules.

D19 (Q14, hand derivation and independent check). Example 10 has two outer rows and six worker rows:
call, nested oracle call, const, compare, faultif, permutation call, ret, retval = 8 executed outer steps.
The worker has five locations, hence R_worker = 3 + 5 = 8 cells; token/classical/region bindings add none.
Binding key [0,1,0]; HALT = 8, P = 4. The nested oracle has six rows and computes
(x + a + (d mod 4)) mod 4, and the permutation action XORs bit 0, giving that value XOR 1. Outer B = 7
misses retval (code 1); stack_limit = 7 or depth_limit = 0 faults at the first call (code 3); nested B = 5
returns code 1, propagated by faultif at outer step 5 as code 9. The two mapped bases (16, 17) alias the same
passed region cells across invocations; neither passes a location as a classical argument.

Q14 verification (headless, only the two owned Markdown paths edited):
- The independent Python row model reads example 10's printed instruction lists: all 4,096 (x,a,d) inputs
  pass eight budget/boundary cases; all 262,144 input/target/fault tuples satisfy the XOR involution.
  This checks the numerical witness, not an implemented 0.9 executor or a full circuit-level inverse.
- Recursive binding forwarding closes on a finite, deterministically ordered key set in an independent model;
  17 complete JCS metadata/envelope lines parse and are canonical; all printed lproc signatures carry the
  three new arrays. Example 10's row counts, header and forwarding lists match the derivation.
- `python3 /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/isa_check.py`:
  PASS (two accepted 0.8 example objects, their hashes, two envelopes and stated execution).
- The unmodified seed parses all ten objects and computes their hashes as a structural cross-check;
  example 2's changed lproc/caller hashes and example 10's hashes are recorded above as non-normative values.
- Draft SHA-256: `25641147f345491a69a27208e4cbd1812011a364daf3e1821c31a8e1562233f0`.
- The 0.8→0.9 unified diff is regenerated in memory: 2103 lines, SHA-256
  `8ede5c269a6b455656f68312b9f792614246c680a69c4ce9b9ea03b386eded79`. The earlier
  job's tracked `spec/isa-0.8-to-0.9.diff` is outside this task's two owned paths and is not rewritten.
- The accepted 0.8 file's SHA-256 remains
  `90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e`.
- `git diff --check -- spec/Bennett-ISA-Spec-0.9-draft.md spec/isa-0.9.PROGRESS.md`: PASS.

Remaining lproc mechanics: none deferred by Q14. Implementation and the 0.9 precheck (including executable
conformance for example 10 and hash patching) remain pending under the draft's existing gate; this task does
not change the read-only reference interpreter. General §17 defaults (notably the unpublished `ref` expansion
bytes and optional checkpointed realisations) continue to apply to lproc rows as to other lifted rows.

## Ratified decisions applied — 2026-10-30

Binding source: the complete maintainer decision memo, Ratification section,
2026-10-30; execution brief: docs/briefs/2026-10-30-isa-09-ratified-edits.md.
This section supersedes the historical proposal/open-choice language above.
Bead b-isa-spec-2c9 tracks this sole-editor text pass. No accepted 0.8 or
interpreter files are edited; the orchestrator owns worklog updates.
Baseline: 25 Rust tests green; full 0.8 harness 445 pass / 0 fail /
48 unavailable (493 cases), exit 2.


| Memo item | Section edited | Conformance witness | Status | Commit |
|---|---|---|---|---|
| A2 / LAUNCH-ESTIMATE | §10, §10.1, §12.3 (K12) | G-estimate-resolved-class-decimal (retained) | Applied, pending accepted 0.9 | `2e3ecd2ca4d96008bedfd429fbbf2a30c8e2f8f9` |
| A3 / CLAIM-REPLAY | §9.2, §12.1–§12.3 (K13) | D-sampled-relation-invocation; CLAIM-BINDING (retained) | Applied, pending accepted 0.9 | `0e72b63adaa5ee5f446706f0ef628f1165d4e57a` |
| A1 / ISA-REF-LIB; B6 | §11.1, §17 item 7 (K11) | E-ref-09-qxor-witness; retained qxor envelope | Applied, pending accepted 0.9 | `d9922bffe31b4aa4268c59b396a1a63eb969ffb0` |
| A6 / topology | §8.2–§8.3, §11, §17 item 15 (K14/D1) | E-topology-grid-09 | Applied, pending accepted 0.9 | `679eae0b8b64dadc1b7c3105b07635058a868d85` |
| B1 / multi-block actions | §5.3 (K15) | C-multiblock-09 | Applied, pending accepted 0.9 | `23013a54b90594ba9f6acb6a951cafeee76abc09` |
| B5 / typed frame records | §9.1, §9.4, §12.3 (K16) | F-frame-records-09 | Applied, pending accepted 0.9 | `a300222603d9ca1c0ec303a3122ad896792f2db0` |
| A5 / draw ordering | §4.1 (K17) | A-measure-draw-order-09 | Applied, pending accepted 0.9 | `ed044ecb6017ecd41fb98b4f8d3271eac7c40945` |
| A4 / Q14; A5 / EPS-DECADE, BLOCK-IMAGE, ISA-7, Q1–Q13 | §0 (K9/K10/K5/K18), Q14 confirmation | F-ratified-confirmations-09; retained §16.3 examples and boundary/sector witnesses | Applied, pending accepted 0.9 | `78dd9df95ffd29abfdaa97e3c9a62185cef3811f` |
| §0 / §17 audit | §0 change list, §17 defaults; all edited clauses | All retained and added draft cases | Applied, pending accepted 0.9 | commit containing this row |

Step 1: explicit signature, complete args, escaped flat class keys, exact
reduced rationals and 34-significant-digit upward enclosure are normative.
Existing Fraction/Decimal witness checks independently reproduce every row.
Rejected alternatives (retained historically above): separate bound-request
entry, class-aware accessor, selectable precision/rounding. No alternative
remains in §10.1. No new numerical example was needed.

Step 2: expected is mandatory, standalone invocations are per claim,
scheduled roots use bindings.invocation, and complete positional tuple
arrays retain every input/target/fault/borrowed/cell component. Large
domains use the program relation. K4 quantified and universally checked
structural claims retain their scope. Existing independent checks cover
XOR outcomes, coverage, wide args, signed zero and canonical args hashes.
Rejected: omitted expected defaulting to oracle; shared invocation registry
or region-level inheritance; named or sample-only keyed tables. The
region-inheritance parenthesis was removed to match ratification literally.

Step 3: schema, envelope hash, ordinals, scratch identity, empty removable
sets and the complete deterministic decomposition families are decided.
The catalog is a separate b-isa-spec-8r4 job; no ref/ publication occurs
before acceptance. Existing independent Python checks enumerate every qxor
pair and reversal and recompute the envelope hash. New pending witness
adds one unavailable case. Rejected alternatives: a different hash rule,
optimized/CSE recipes or a different initial decomposition set; later
libraries require their own version. SPEC-QUESTIONS 21's library-related
realisation gate remains a delivery dependency, not an undecided contract.
Gate note: raw unified diffs contain space-only context lines by design;
git diff --check covers source/progress/conformance, and the diff artifact
is verified by regeneration rather than stripping its context bytes.

Step 4: optional vertices/edges with addressable classes, pairwise native
adjacency and concrete schedule placement. The memo does not enumerate
placement fields or multi-operand connectivity: §11 supplies explicit
interface/allocation vertex arrays plus deterministic defaults, and §8.3
uses pairwise adjacency for native gates with more than two operands.
These details prevent conformant consumers choosing different placements;
scheme leaves validate separately. Explicit swap sequences keep routed
computation reachable while automatic routing/cost rules wait for 0.10.
Rejected: omitting topology entirely, or deferring its presence/meaning.
Python -I JCS/hash and an independent Rust grid/byte/hash test check the
new witness. Canonical bytes have no trailing LF (b-isa-spec-1pl).

Step 5: the ratified sentence is applied. Read reference README and all
SPEC-QUESTIONS (no separate multi-block entry); action.rs flattens blocks
in text order with one shared environment. The built binary checks every
t,p pair and its adjoint against the independent expression, including
the scratch lifetime across blocks. No disagreement; no bug bead needed.
Rejected: interpreting action labels as executable CFG branch/join nodes.
The fixture's 0.8 interface hash is computed from its complete envelope
(body included), independently of the CLI; no 0.9 example hash is added.
The separate staged case is unavailable pending 0.9.

Step 6: published typed records with explicit slot/frame identities,
class/state/width rules, c-only expressions and disjoint shared lifetimes.
Independent Python and Rust checks reconstruct canonical bytes, SHA-256,
width arithmetic and half-open interval disjointness; no 0.9 executor
claim. The memo names record categories but not field spellings: §9.4
defines those plus mandatory empty arrays and identity/order rules.
Rejected: the previous interface/class-map/scheme_ancillae/retained_history
encoding, and per-class expression maps with role annotations.
SPEC-QUESTIONS 11's frame-encoding question is closed for 0.9; its 0.8
reading remains unchanged in the interpreter-owned file.

Step 7: FIFO dynamic measurement consumption, wire-first lexicographic
outcomes and strict cumulative threshold match README, --draws handling,
classical.rs VecDeque and registers.rs. The two-measurement 0.8 CLI
fixture returns bits 01 then 11 at the exact CDF ties; Python Fraction
and independent Rust integer arithmetic verify both ordering distinctions.
The fixture hashes use complete independent 0.8 envelopes; 0.9 example
hashes remain placeholders. SPEC-QUESTIONS 29 is closed for 0.9.
Rejected: reverse draw consumption and integer-order CDF enumeration.

Step 8: all confirmations marked decided. Existing operative sentences
already match ratification and stay unchanged; no new numerical values
are introduced. Q12 continues to gate draft example hashes on the 0.9
precheck. Rejected Q14 reading: a captured/readable argument environment
instead of explicit token/classical/mapped-region forwarding. Historical
Q1–Q13 choices remain applied; this pass does not reopen them. The new
staged case names the confirmation beads; independent existing witness
checks remain green.

Step 9: consolidated the K9–K18 rows and audited every ratified ISA edit
and its deferral. §17 item 7 fixes the default library and publication
dependency; item 15 fixes the routing default and preserves explicit
swap sequences. All maintainer-decision markers and rejected readings
are absent from normative text. The pre-existing §5.5 implementation
choice to retain an already identified matched frame is semantically
equivalent storage bookkeeping, not an unratified alternative reading.

Ratification scope dispositions: B2 (versioned symbolic histories), B3
(pending import obligations at linking), B4 (linked-object replay registry)
are the memo's circuits M2 proof-contract jobs, not additional ISA text
edits in this brief. Their ratification is preserved here for the circuits
implementer; no competing ISA encoding or resolver is introduced by this
text pass. B6 is satisfied by A1's decided identity contract; catalog
delivery still waits on b-isa-spec-8r4. C is a process/release decision
for the orchestrator (council round, seats, plugin/driver starts), not an
ISA semantic clause and not executed by this implementation sub-agent.

SPEC-QUESTIONS closures: 21's comparison/tuple/budget ambiguities are
closed by K13 (the old flat tuple reading is superseded only for 0.9);
11's frame format is superseded by K16; 29 closes through K17. This pass
adds no multi-block question entry, and does not edit SPEC-QUESTIONS.md.

Wording elaborations: removed K13 region-invocation inheritance to honor
per-claim binding; supplied concrete field names and identity/order rules
for the topology placements and typed frame schema; made native
multi-wire adjacency pairwise so higher-arity gates have a determinate
rule. These fill schema details the ratification leaves implicit. B1
was applied as the requested single sentence. No reference-interpreter
disagreement was found for the two behaviors checked (blocks and draws);
0.9 schema/replay/estimate implementations are not claimed.

Final observed full harness: **445 pass / 0 fail / 54 unavailable
(499 cases), exit 2**; baseline **445 / 0 / 48 (493 cases)**.
Six added cases are all unavailable pending 0.9 and name their beads.
All 28 Rust tests pass headlessly/offline; the three added Rust tests
independently check grid canonical bytes/hash/adjacency, typed-frame
bytes/hash/widths/lifetimes, and FIFO/lexicographic CDF ties. Python runs
with -I and imports no interpreter or VM implementation. Raw diff bytes
are regenerated immediately before every commit; accepted 0.8 SHA-256
remains 90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e.

K19 structured witness clarification: finite nested actual syntax may invoke
the same higher-order recipe (qloop(3,qloop(2,B)) is six B occurrences).
Rejecting any repeated recipe name on the runtime expansion stack conflates
finite source nesting with recursive declared dependencies. Captured actual
expansion starts its own dependency audit; direct and mutual cycles still
reject. Structured full-domain/fixpoint tests reproduced this error before
the expander fix. No qxor witness bytes changed.

K19 validation also enforces §3.3 controls versus diagonal/swap targets,
closed formal keys, declared dependency cycles through literal actual
syntax, and scratch LIFO/balance. The six remaining source-binding/order
gaps are afi, 1a4, 3xe, lph, 1oa and 82x; details and affected rows are
recorded in ref progress and bd. No partial-library hash is inserted at
the complete-catalog marker. The arbitrary-image and qinplace-order fixes
would exceed the explicitly scoped constructor additions, so are raised
for a separate contract decision rather than guessed.


REF completion resumed 2026-10-31: K21 binds unconditional image bytes
(afi); K22 adds indexed typed body templates with explicit frame actuals
(lph); K23 documents the integer bit extractor already used by literal
recipes. Both new language tests were observed red on unknown constructors
before implementation. Rejected: controlling image X, opaque image names,
rebinding body captures by spelling, treating template application as a
callee ordinal space, and leaving bit extraction implementation-private.
The qxor witness bytes and hash remain unchanged.

The call-flag regression was reproduced: all 192 semantic configurations
passed, then the tuple-count assertion failed (43,200 actual vs 41,760
printed). Its loop/domain and semantic assertions are retained; accounting
is corrected to 192 configurations / 43,200 tuples, plus the exact §16.3
5/10-bit counter case (131,072 tuples). This is not a relaxation of the
call predicate. Baseline harness is 445/0/65, not the older 445/0/60.

Carried contract gaps: 1a4 (arbitrary heterogeneous interface argument-list
forwarding, including token/classical types), 3xe (class and output/lifetime
wire binders), 1oa (ordered mapped-region descriptors), and 82x (ratified
reverse-allocation order versus the printed qinplace sequence). Their exact
blocking sentences remain in §11.1 and the beads; no partial library is
published. In particular changing qinplace's release order or adding a
ratified-A1 exception needs a contract decision, not an expander default.

K24 adds captured closure-bound argument vectors and exact classical/token
values, including >64-bit integers and NaN payloads. Its forwarding test
failed on the missing args-name constructor before implementation. Rejected:
reinterpreting captured actual names in the forwarder's scope, word-sized
classical integers, token ids, implicit value widening, and sharing caller
ordinals with symbol calls. 1a4 is carried until source region shape/role
metadata is encoded (the vector mechanism itself is now implemented); it
shares that remaining descriptor gap with 1oa. No flattened wire shape is
misrepresented as a complete region interface.

K21 default-image clarification: absent image is the empty string and emits
no X even on non-byte-aligned regions (one to four total wires included in
the witnesses); nonempty images retain exact byte/wire length checks. K22
lexical audit reproduced and fixed legacy integer-template wire import and
cross-type shadow leakage; formal names now shadow across all captured name
maps. Remaining activation metadata is b-isa-spec-evi; afi and lph closed,
1a4/3xe/1oa/82x carried with exact changes in bd. The completed step and
qregion recipes bring the checkpoint to 195; no complete-library hash/count
is asserted and the normative marker remains untouched.

## 2026-10-31 REF completion II — K25–K27

K25 shares ordered typed record binding between mapped byte layouts and
activation locations/captures. K26 retains source region shape, role and
origin through K24 closure vectors. K27 passes exact allocation classes and
shared declarations through recipe calls, exposing output wires in the
caller after acquisition, with explicit lifetimes and adjoint metadata.
Witnesses: conformance/tests/ref_descriptors.rs; observed red for unknown
formals/constructors before implementation. K23 already resolves 4nx; its
LSB literal witness remains unchanged.

Rejected: opaque caller memory/activation body dispatchers (outsource the
catalog algorithm); flattening gaps or region shapes (loses source binding);
using any for every class (changes resource identity); copying allocation
status at invoke (breaks returned output lifetime); host ids for tokens
(K24 already preserves value tuples). No 0.8 semantics are changed.

82x is carried: §4.5 explicitly generates `qfree t; qfree f` after
allocating t then f; §11.1 and ratified A1 require all library scratch
released in reverse allocation order. Changing §4.5 would change accepted
0.8 generated order; adding an instruction-private exception here would
change the ratified A1 rule. Neither is a constructor-language extension.
Stop on qinplace; require an explicit semantic/ratification decision.
The two calls and both private bundles remain required and in scope.

K25 checker audit in the allocation family: record wire actuals evaluate
in the captured caller environment, not in an environment overwritten by
earlier wire fields. Field width expressions instead use the receiving
recipe's earlier integers plus earlier integer fields. A regression
produced coincident operands before the fix; the two scopes are now
separate. Normative text already required these scopes.

K25 field schemas also admit the existing K21 image type, with a red/green
initializer-record witness. Keeping image metadata in the shared record
vocabulary avoids a separate ad hoc initializer-list binding. Image X
semantics stay K21; no new unconditional-gate permission is introduced.

K27 follow-up audit: declaration handles cannot bypass a declared library
scratch lifetime's LIFO/clean requirement by passing through allocation
formals. A non-LIFO scratch handle regression failed before classification
was retained on the shared identity; caller-owned output lifetimes still
permit overlapping release order. This specifically preserves 82x rather
than silently treating qinplace's private scratch as caller results.
K26 origin suffix is now the exact source NAME token; occurrence identity
is the origin binding plus the already retained cell wire identities,
which disambiguate repeated invocations without an unspecified string form.

K25 construction uses the same record vocabulary: width/initial/image and
shared storage handle; a separate indexed frame list aliases exactly the
state list's B frame handles. Source formation fixes the printed §5.2
group order and the reference §3.7 widths/initial values, with byte images
remaining K21. Rejected: an opaque caller construction body, image-only
initialization of non-byte state, bytewise name sorting across declaration
order, and flattening all heterogeneous frames into caller-private circuits.
The catalog invokes its own step algorithm; only source row/successor
templates are supplied. Record acquire retains the recipe scratch mark
through K25 scope binding, so private state cannot bypass K27 LIFO.

## 2026-10-31 REF completion — K28 instruction release ruling

Maintainer ruling on b-isa-spec-82x is applied solely to §0/§11.1 as
K28 REF-INSTRUCTION-RELEASE. The reverse-allocation rule governs recipe
scratch; an instruction-defined sequence is reproduced exactly, including
qinplace's qfree t; qfree f. The one-bit identity-pair witness records
instruction ordinals 0/1/2/3/4 and separate callee ordinal zero for both
calls. conformance/tests/ref_qinplace_contract.rs was observed red before
the normative addition, then checks the exact leaf sequence and identities.
Existing scratch-LIFO/capture regressions stay binding.

Rejected alternative: changing §4.5 to qfree f; qfree t would alter accepted
0.8 generated order. Neither accepted 0.8 nor draft §4.5 is changed. Also
rejected: relaxing release order for ordinary recipe scratch or removing
either private allocation/call. K27 declarations encode the instruction's
own lifetimes under K28, not a general scratch exception. The printed qxor
witness bytes/hash are unchanged. Catalog completion and full-library
binding follow as a separate family commit; publication requires accepted
0.9.

Rule 12 addendum: K28 and every new witness sentence carry declared
dependencies. Lookups in ISA-0.8.json and ISA-0.9-draft.json identify
qinplace as ISA-4.5-P5 and the original scratch/ordinal clause as
ISA-11.1-P21. The new rule is a separate paragraph after that clause;
unchanged sentences are not retrofitted.

Ruling gate: 109 Rust tests / 0 fail (entry 108/0); fmt --check and
clippy --offline --all-targets -D warnings green; full harness unchanged
445 pass / 0 fail / 73 unavailable (518). After the dependency-marker
addendum, draft_contracts and the exact four-tuple identity-pair witness were
rerun green, as was clippy. Accepted 0.8 fingerprint and qxor unchanged.

Completion self-review corrects the small K28 witness from n=F=1 to
n=1/F=4 under §5.4 (dependency added). The fault-width assertion was red
before fixture correction and green after; instruction/callee ordinals,
release order and four-tuple domain are unchanged. The catalog's 80 shapes
and example 7 already used F=4. No accepted text or catalog hash changed.

## 2026-10-31 REF complete draft-library binding

All 31 README dispositions are bound after qinplace completion under K28.
The single §11.1 marker now records 219 recipes and complete envelope
fcb2f1e6d61aced0bf9689eab5f6d82c4ae42e9fea8d1b21ae9c7ffccda6f5ce,
computed independently in Rust and Python -I (including original qxor check).
The new binding/publication sentences carry declared dependencies too;
only K11's non-normative bead-status annotation is updated outside that
marker in this family. No other normative K11 wording is changed.
K28 correction and release rule retain accepted §4.5 bytes and ordinals.
Final gate 111 Rust/0 fail (entry108, ruling109), fmt/clippy green,
ref_call_flag green; harness445/0/74 from445/0/73, one pending-0.9 case.
82x and parent8r4 closed; all children closed; publication still requires
accepted 0.9. Full evidence and family domains are in ref-0.9-draft.PROGRESS.md.

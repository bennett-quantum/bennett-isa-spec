# Bennett ISA reference interpreter progress

Resume here. The whole accepted ISA 0.8 remains in scope; staged deliverables
are not complete. Sole implementation sub-agent; owned interpreter and external
round-17 response only. Never push. Every cargo invocation uses the headless
prefix and TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp.

## Round-17 fixes (2026-10-31)

Baseline **390 passed / 0 failed / 5 ignored**, summed over all 33 binaries
(including 0-test main/doctests), 22.35 s runner sum. Initial witness RED:
**2 passed / 4 failed**, missing epsilon_share and pairwise nonmaximal-tie
rejection. Audit RED for sampler.count: **0 passed / 1 failed**, missing count.
All round 4–17 positive and negative tests are retained.

| Blocker | RED tests | Status | Commit |
|---|---|---|---|
| R17-B01 | metadata_schema::tests::r17_gate_payload_omission_is_literal_clause_control; metadata::r17_default_share_and_shadowed_witness_preserve_identity; cli_xam::r17_cli_default_share_and_shadowed_tie_check_hash_pins; audit r17_sampler_count_default_is_optional | GREEN | e72df8c |
| R17-B02 | metadata::r17_shadowed_tie_all_orders_and_genuine_maximum_ties | GREEN | e72df8c |

Design: TABLE gate_payload uses epsilon_share?:decimal (§17.9). The
schedule_epsilon_share consumer returns an exact equal-share recipe once the
expanded approximate-operation count is known, or preserves an explicit
assignment; exact operations remain exact. No canonical bytes are rewritten.
The class audit also found sampler.count wrongly required (§17.13); TABLE
now makes it optional and claim replay defaults to 65,536, with actual replay
and malformed-present controls. Item 7 was previously wrong and repaired in
R16; it remains covered. The independent table-driven
r17_section17_defaults_are_optional_and_applied enumerates all 14 items,
checking omitted-field admission plus consumer observations where implemented.
Nonsemantic decisions/C layout and staged target/interval choices are
explicitly identified, never counted as implemented consumers. §§8.1, 9.1,
11 and 12 audit: no additional incorrectly required default field found;
block_image and pinned_closure_hash remain optional, with empty-block-map
and unpinned-link consumers already present. Precision/seed/algorithm and
present kind payloads still have no omission default and remain required.

Schedule ambiguity uses intersection/subtraction of selector boxes within
legal invocation/iteration domains and the oracle's 1..B step range. Only a
nonempty equal-maximum remainder after subtracting the UNION of all higher
entries rejects. The resolver counts final maxima independent of order.
Shortest real chains suffice, including each call instruction's own generated
operations; empty prefixes split over all root instructions. No loop bound
or generated-ordinal sampling cap is imposed. All six witness orders admit
and resolve C; A/B and duplicate C reject, A/C admits. Union/partial coverage,
step-domain and loop bounds 0, 2, 3 and 2^40 are covered. The independent
specificity family observes **1,200 schedules + 1,200 reversed orders**,
**1,168 admitted / 32 rejected**, 54 concrete operation patterns each,
**100% oracle agreement**; every admitted resolver result is compared too.

Both read-only reviewer witnesses were copied byte-for-byte. r17_pins.py
independently reconstructs them from the R13 corpus and literal 261-byte
inc8 envelope (no interpreter/rehash). File SHA-256 8af833c2…2a5 and
9fc08dca…1aff; schedule hashes d9025ef2…4845d and e13ea989…287f;
object 8093da48…ec5d9, interface=closure 8ce5e64f…5be93 remain unchanged.
Raw schedule bytes, source text and budget [3,0,0,0] are preserved.
Null/object/negative/invalid decimal-string shares and missing realisation
reject in both library and CLI (check/hash exit 1, empty stdout, cited
§9.1/§11). Present malformed counts also reject; absent sampled count
actually replays 65,536 elements; explicit 0 and 3 remain 0 and 3.

Derived schema family: **163 productions, 9,897 positives, 1,653 unique typed
paths, 26,746 grammar negatives**; **1,992** negatives also pass through BOTH
admission validators (subset). **26 pinned objects, 2,393 parse visits,
17,173 full-parse negatives**; total negatives **43,919**. The derived family
proves enforcement of TABLE; the independent §17 table catches wrongly
required defaults. Width family and all previous controls are retained.

First full suite: **403 passed / 0 failed / 5 ignored**, 37.84 s runner sum.
Final full suite: **403 passed / 0 failed / 5 ignored**, 34.62 s runner sum,
**14 unit + 389 integration**, 0 doctests. Fmt/fmt --check and strict
all-target Clippy pass, 0 warnings. An initial Clippy finding concerned only
hex digit grouping in the unchanged RNG seed; final Clippy and full suite
reran after the spelling fix. Production and all sweep inputs/assertions
were unchanged after sweep start.

All **5 ignored sweeps passed / 0 failed**, **135.70 s** runner sum across
all 33 binaries.

| Sweep | Full domain | Observed result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 operand pairs × 4 = 262,144 executions | 1/0; 1.24 s |
| 17-wire borrow | 131,072 entries | 1/0; 5.51 s |
| checked borrow certificate | 262,144 elements, subject and expected program | 1/0; 28.56 s |
| §6 differential | 200,000 programs: 70,931 accepted (all clean), 129,069 rejected | 1/0; 96.16 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 1/0; 4.23 s |

Logs: .tmp/r17-{before,red,red-count,regressions-final,loop,schema-family,
full,final,clippy,clippy-final,sweeps}.log. Independent reconstruction and
both cmp checks pass. Owned diffs pass --check. No foreign tests, non-owned
path edits, dependencies, Beads mutation/export, push or Dolt sync. Concurrent
spec and conformance work is untouched. Historical round evidence is in
council/response-round-N.md, external council responses and git history.
Implementation commit e72df8c; this block records the final evidence.

## Resume here: outstanding work, in priority order

No user decision is required. Do not mark the job complete; no deliverable has
its full verification obligation met yet. The entire ISA remains the scope.

1. Complete §6 provenance/window analysis. Loops now apply exact repeated
   transfer, but the version-log checker still lacks complete transitive
   dependencies, entangling-read restrictions, predicate-version pins and
   precise region-cell tracking. Matched oracle_uncall rejects all intervening
   stores/calls conservatively; implement precise transitive memory effects.
2. Complete semantic schedule/certificate/coverage verification/composition,
   frame live ranges and borrowed library claims (recursive admission schemas
   are centralized and tested in metadata_schema.rs).
   Imports resolve by signature and pin, including data; linking is tested.
3. Generalize lifted reg widths and oracle bitvectors beyond128. Structured
   wire bundles and Q basis execution support256 in tests; descriptors remain
   u16. Audit JSON/JCS number corner cases and all integer spellings.
4. Complete Q verification: qinplace certificates/full domain, qif, coherent
   while dependencies, nested controlled calls, local-image and cell cleanup.
   CC qreg handles/measurement/region inverse history are now implemented;
   audit cross-call aliasing and run eligibility. Idle pools now reach lifted
   actions and designated CC registers.
5. Expand complete-D machine verification across call/ret special cases and
   every first-fault boundary. tiny_memory now covers all interface memory,
   target/fault entries and explicit commit; prior suites cover every opcode.
   Audit concrete counter widths/generalized frame sizing.
6. Audit remaining §8–§13 interpreter-facing schemas/contracts. Admission schemas
   do not implement target evaluation, schedule execution or interval verification.
7. Persist new discipline/borrow objects as canonical fixtures with hand
   expectations; cover every path/fault. Rerun ignored sweeps, strict Clippy,
   fmt, unchanged Python seed and documented CLI after substantive changes.

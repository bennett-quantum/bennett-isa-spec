# Bennett ISA reference interpreter progress

Resume here. The whole accepted ISA 0.8 remains in scope; staged deliverables
are not complete. Sole implementation sub-agent; owned interpreter and external
round-16 response only. Never push. Every cargo invocation uses the headless
prefix and TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp.

## Round-16 fixes (2026-10-31)

Baseline **384 passed / 0 failed / 5 ignored**, summed over all binaries,
20.67 s runner sum. R16 RED **2 passed / 4 failed** (all omission positives
fail with missing field expansion); the corrected existing schedule test also
fails before the fix. GREEN **6 passed / 0 failed**, 0.02 s runner sum.
All round 4–16 tests are retained, with the contradictory omission rejection
replaced by successful parse/check_hashes as §17 item 7 requires.

| Blocker | RED tests | Status | Commit |
|---|---|---|---|
| R16-B01 | metadata_schema::tests::r16_schedule_bindings_grammar_admits_omitted_expansion; metadata::r16_default_expansion_preserves_witness_and_identity; metadata::r16_schedule_consumer_resolves_default_and_preserves_explicit_binding; cli_xam::r16_cli_default_expansion_check_hash_preserves_pins; metadata::schedule_requires_typed_bindings_selectors_and_unique_maxima | GREEN | 8f4d6b7 |

Design: TABLE schedule_bindings uses expansion?:expansion with §17 item 7
alongside it; present descriptors retain the closed §11 name/version/hash
production. Object::schedule_expansion returns the effective name, version and
optional supplied hash: omission resolves to ref/0.8, with no synthetic hash.
Explicit bindings are preserved; no schedule returns None. No canonical
schedule bytes are rewritten. Admission and binding resolution are implemented;
target/schedule execution and interval verification remain staged. No spec
amendment or new narrowing reading is needed.

The copied tests/support/r16-default-expansion.bisa is byte-identical to the
read-only reviewer witness, SHA-256 8344b91c…25bfa. r16_pins.py independently
reconstructs the exact omission from the R13 record and checks the literal
261-byte LF envelope and §9.2 exclusions without interpreter/rehash. Object
8093da48…ec5d9, interface=closure 8ce5e64f…5be93 and schedule
d5ba4457…fb99d remain unchanged; budget remains [3,0,0,0]. Six present-malformed
controls (null, empty object, bad hash, missing hash, non-string version,
extra field) reject at library admission and CLI check/hash; CLI requires exit
1, empty stdout and §9.1/§11. Explicit ref/0.8 and vendor/2026.10 resolve with
their supplied hash. No execution budget/input was added to admission tests.

The retained metadata_schema::tests::r14_generated_metadata_schema_type_family
now exercises **163 productions, 9,862 generated positives, 1,653 unique typed
paths, 26,685 grammar negatives**; **1,992** negatives also run through BOTH
admission validators (a subset). **26 pinned objects, 2,393 parse path visits,
17,173 full-parse negatives** remain. Total negatives without recounting dual
assertions: **43,858**. Counts test enforcement of TABLE; the omission control
is independently clause-derived. Family/metadata/width gate: **35 passed**,
5.30 s runner sum. Width family remains **507: 159 constructed, 286 layout
rejections, 62 resource diagnostics**. The reviewer's nonblocking independent
width-outcome classification suggestion is deferred: its huge-width cases
include allocator/storage failure as well as integer representability; an
independent host-allocation model is beyond this short binding repair.
No width domain, literal boundary assertion or production behavior changed.

First full suite: **390 passed / 0 failed / 5 ignored**, 19.88 s runner sum.
Final full suite after lint-only fixes: **390 passed / 0 failed / 5 ignored**,
21.44 s runner sum (11 unit + 379 integration, 0 doctests). Fmt/fmt --check and
strict all-target Clippy pass, 0 warnings. The initial lint gate found unused
shared-helper reexports and .err().expect(); constant aliases and expect_err
fix those without changing test behavior. Final suite and Clippy reran after
those changes. The running sweeps use the same inputs/assertions and unchanged
production code; neither lint edit changes a sweep.

All **5 ignored sweeps passed / 0 failed**, **105.08 s** runner sum across
all binaries (105.07 s in the five sweep binaries).

| Sweep | Full domain | Observed result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 operand pairs × 4 = 262,144 executions | 1/0; 0.55 s |
| 17-wire borrow | 131,072 entries | 1/0; 3.29 s |
| checked borrow certificate | 262,144 elements, subject and expected program | 1/0; 16.45 s |
| §6 differential | 200,000 programs: 70,931 accepted (all clean), 129,069 rejected | 1/0; 82.21 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 1/0; 2.57 s |
Logs: interpreter .tmp/r16-{before,red,red-existing,green,family,full,final,clippy,clippy-final,sweeps}.log.
Independent pin/copy checks pass. Owned diffs pass --check. No foreign tests,
spec/conformance/include/Python-seed edits, dependencies, Beads mutation/export,
push or Dolt sync. Concurrent work is untouched. Historical round evidence is
in council/response-round-N.md, external council responses and git history.

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

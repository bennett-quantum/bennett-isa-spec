# Bennett ISA reference interpreter progress

Resume here. The whole accepted ISA 0.8 remains in scope; staged deliverables
are not complete. Sole implementation sub-agent; owned interpreter and external
round-15 response only. Never push. Every cargo invocation uses the headless
prefix and TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp.

## Round-15 fixes (2026-10-31)

Baseline **379 passed / 0 failed / 5 ignored**, summed over all binaries,
20.67 s runner sum. Combined R15 RED **2 passed / 3 failed**: direct grammar
admits frame lengths [0,1,2,3], expected [2]; library/CLI reject legal "0.0"
with §11: §5.2. The two full-admission frame regressions already pass because
of the old downstream guard. Combined GREEN **5 passed / 0 failed** (0.06 s).
All round 4–15 positives/negatives and previous family tests are retained.

| Blocker | RED tests | Status | Commit |
|---|---|---|---|
| R15-B01 | metadata_schema::tests::r15_translation_frame_is_first_and_last_point_tuple; retained-admission controls metadata::r15_translation_frame_cardinality_is_checked_at_admission, cli_xam::r15_cli_rejects_empty_translation_frame | GREEN | 9283db7 |
| R15-B02 | metadata::r15_oracle_schedule_epsilon_uses_exact_decimal_value; cli_xam::r15_cli_oracle_schedule_epsilon_is_exact_by_value | GREEN | 9283db7 |

Design: src/metadata_schema.rs::TABLE is the sole recursive structural grammar.
New translation_frame = Tuple(point, point) expresses §12.1 first/last
cardinality; V/W remain Array(point). The duplicate cardinality-only check
in certificate.rs::validate_claim is removed. Contextual claim-index checks
remain. metadata.rs::Object::validate_schedule_details checks numeric oracle
epsilon zero after decimal validation: all mantissa digits must be zero,
regardless of scale/exponent, with no binary64 conversion for the decision.
README's metadata Design paragraph reflects this structural/semantic split.
No spec reading/waiver is added; numerical/channel verification remains
explicitly Unverified at the existing staged boundary.

Two exact reviewer witnesses in tests/support/r15-witnesses/ match their
read-only originals byte-for-byte. r15_pins.py independently assembles the
literal 261-byte LF envelope and hashes it, without interpreter/rehash.
Interface=closure 8ce5e64f…5be93, object 8093da48…ec5d9 remain unchanged.
The epsilon witness SHA is 1431a1db…2df; the empty-frame file is bf88ce59…b27d.
Library/CLI positives 0, "0", "0.0", "0.000" parse/check/hash with those pins.
Nonzero "0.1", "0.000001", exact decimal-string 10^-400 and numeric 1e-300
reject §5.2. String "1e-300" is outside the existing decimal-string grammar
and rejects §9.1. CLI negatives exit 1 with empty stdout and cited diagnostics.
Only two-point frames parse/check; empty/one/three-point frames reject §12.1.

The retained metadata_schema::tests::r14_generated_metadata_schema_type_family
now exercises **163 productions, 9,806 generated positives, 1,653 unique typed
paths, 26,545 grammar negatives**; **1,992** negatives also run through BOTH
admission validators (a subset, not added again). **26 pinned objects, 2,393
parse path visits and 17,173 full-parse negatives** remain. Total negatives
without recounting dual assertions: **43,718**. Table-derived baselines now
have two frame points. The independent literal cardinality test checks the
clause separately; generated counts establish enforcement of TABLE.
The width family remains **507: 159 constructed, 286 layout rejections,
62 resource diagnostics**. Its independent outcome-classification follow-up
remains nonblocking future work, as the reviewer permits.

Observed first full suite: **384 passed / 0 failed / 5 ignored**, 20.60 s.
Final full suite: **384 passed / 0 failed / 5 ignored**, 19.65 s runner sum; 10 unit + 374 integration, 0 doctests.
Final family/width gate: **6 passed / 0 failed**, 5.19 s schema + 0.07 s widths.
Fmt and strict all-target Clippy pass (0 warnings). Independent pin script and
copy checks pass. No production/test edits after final suite or sweep start.

All **5 ignored sweeps passed / 0 failed**, **97.21 s** summed runner time.

| Sweep | Full domain | Observed result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 operand pairs × 4 = 262,144 executions | 1/0; 0.52 s |
| 17-wire borrow | 131,072 entries | 1/0; 2.74 s |
| checked borrow certificate | 262,144 elements, subject and expected program | 1/0; 14.99 s |
| §6 differential | 200,000 programs: 70,931 accepted (all clean), 129,069 rejected | 1/0; 76.54 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 1/0; 2.42 s |

Logs: interpreter .tmp/r15-{before,red,green,full,final,family,clippy,sweeps}.log.
Owned diffs pass --check. No spec/conformance edits or execution, dependency,
Beads mutation/export, push or Dolt sync. Concurrent changes are untouched.
Historical round evidence is in council/response-round-N.md, external council
responses and git history.

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

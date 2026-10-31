# Bennett ISA reference interpreter progress

Resume here. The whole accepted ISA 0.8 remains in scope; staged deliverables
are not complete. Sole implementation sub-agent; owned interpreter and external
round-14 response only. Never push. Every cargo invocation uses the headless
prefix and TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp.

## Round-14 fixes (2026-10-31)

Baseline **375 passed / 0 failed / 5 ignored**, summed over all binaries.
Combined witness RED **0 passed / 4 failed**: parse admitted both malformed
certificate types, CLI check returned exit 0/status ok. Witnesses and all
round 4–13 positives/negatives retained; the R13 type family is replaced.

| Blocker | RED tests | Status | Commit |
|---|---|---|---|
| R14-B01 | metadata::r14_y_endpoints_require_decimal_strings; cli_xam::r14_cli_rejects_numeric_y_endpoints | GREEN | pending commit |
| R14-B02 | metadata::r14_table_rows_and_components_are_typed_at_admission; cli_xam::r14_cli_rejects_malformed_table_rows_and_cells | GREEN | pending commit |

Design: src/metadata_schema.rs::TABLE is the sole recursive grammar, with
clauses on all productions. It covers every accepted JSON metadata section
and the textual data NAME/HEX projection. Admission preflight, signatures,
certificate details and validate_claim consume it; previous hand-written
field/type schemas are removed. Metadata helper checks also delegate to it.
Only contextual/range checks remain in object.rs, metadata.rs, certificate.rs
and validation.rs, with their § citations. Expected tuples use arbitrary-size
canonical integers; Y endpoints use signed finite decimal strings.
Service strings, layout extrema, integer/decimal alternatives, expression
ASTs and optional fields are retained. Numerical/channel verification stays
explicitly Unverified at the existing staged boundary.

Six exact reviewer witnesses are copied under tests/support/r14-witnesses/,
not the valid fixture inventory. tests/support/r14_pins.py independently
assembles literal LF envelopes and verifies file/interface/closure/object
pins; no interpreter or rehash defines expectations. Inc8 is 8ce5e64f…5be93,
object 8093da48…ec5d9; action is 2fcd5b3d…ab8be, object ddc5d672…666f0.
The singleton X action still verifies as Evidence { elements: 1 }. A component
above u128 remains admissible; table truth/cardinality remain verifier work.
CLI negatives exit 1 with empty stdout and §9.1/§12 diagnostics.

The derived family is metadata_schema::tests::r14_generated_metadata_schema_type_family
(src/metadata_schema/tests.rs). It generates positives and recursive mutations
from TABLE, including populated maps/arrays, every union/enum alternative,
optional fields, and integer/decimal edges. Wrong-kind expectations come from
Shape, independently of the validator's result. Both admission paths and
full parse of 26 independently pinned objects are exercised. Final counts
and sweep results will be recorded at the verification checkpoint.

Observed normal gates: 379/0/5 after replacement and 379/0/5 after final
schema/positive-family changes. Initial full run failed only because malformed
reviewer objects were placed in the valid fixture inventory; moved them to
support without weakening the fixture audit. The first pin script assumed
r14-table-bool was action-based; corrected it to read its inc8 symbol. All
supplied pins match unchanged. Clippy's redundant unit-expression finding
was corrected; final gates and ignored sweeps are in progress.

Logs: repo .tmp/r14-{before,red,green,schema-full,family,full,final,final2,
clippy,clippy-final}.log. No spec/conformance edits or execution, dependency,
Beads mutation/export, push or Dolt sync. Concurrent changes are untouched.
Historical round evidence is in council/response-round-N.md and git history.

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


# Bennett ISA reference interpreter progress

Resume here. The whole accepted ISA 0.8 remains in scope; staged deliverables
are not complete. Sole implementation sub-agent; owned interpreter and external
round-20 response only. Never push. Every cargo invocation uses the headless
prefix and TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp.

## Round-20 fixes (2026-10-31)

Clean isolated baseline **412 passed / 0 failed / 5 ignored**, **118.60 s**
runner sum across all **33** result blocks. Final **425 passed / 0 failed /
5 ignored**, **176.88 s**, **16 unit + 409 integration**, 0 doctests. All round
4–20 controls retained. The first before log observed one rebuilt R20 binary;
it is superseded by the isolated baseline, not used for before/after totals.

| Blocker | RED tests | Status | Commit |
|---|---|---|---|
| SOL-R20-B01 | r1_literals::r20_pinned_extreme_const_widths_parse_and_hash_without_panic; cli_xam::r20_cli_pinned_extreme_const_widths_check_and_hash | GREEN | af52c9c |

Exact original const_literal RED: library 0/1; max−1 CLI exits 1 with a false
§3.3 range error; max CLI exits 101 on unchecked N+1. Both reviewer files now
parse/check/hash, preserve their 269-byte envelope pins and whole-file SHA-256s.
`r20_pins.py` independently reconstructs them; reviewer cmp checks pass.
`r20_extreme_const_execution_reports_host_resource` checks both execution
engines return a cited host limit for actual unrepresentable storage.
Ordinary i8 256/-129 still reject, 255/-128 still admit.

Design: object::const_literal parses only the decimal spelling's magnitude and
compares its bit length to N; a negative N-bit magnitude must be a power of two.
No N+1 arithmetic or destination-sized admission allocation remains. Fault-code
and certificate endpoint admission share the unsigned magnitude check; errors
propagate with range/resource separation. Signature dimensions follow §9.1's
number/string boundary. Wire names, region images/cells, quantum identities,
CLI buffers and claim-element storage reserve fallibly; products/cardinalities
are checked. Representation/allocation failures cite the § and name the host
limit. No ISA width/count cap or new spec reading is introduced.

The sweep reproduced additional false large-width signature mismatches,
wire-expansion capacity panics and certificate small-endpoint allocation with
a misleading range wrapper. The arithmetic audit also repaired unchecked
region products and eager exhaustive/sample collection, and normalized cited
resource diagnostics for frames, counters, borrowed-capacity sums and CLI
storage. Bead b-isa-spec-61i records this class.

**102 fixture-qualified paths / 1,113 generated cases**, plus separate pinned
admission/execution controls. Every scalar path receives 0, 1, 2^63−1, 2^63,
2^64−2, 2^64−1 in grammar-admitted spellings. Twenty descriptor spelling
negatives include 00/+1/-0/1.0/2^64. All are under catch_unwind with preassigned
admit/range/resource expectations and § citation. See the full path/validation/
case inventory in external round-20 response.md.

| Family / consumers | Paths | Cases |
|---|---:|---:|
| r13_generated_derived_width_extremes_family: five fixtures' layout scalars + depth | 42 | 591 |
| r20_generated_text_width_count_index_extremes | 8 | 90 |
| r20_generated_descriptor_budget_storage_extremes | 15 | 110 |
| r20_generated_qreg_descriptor_arithmetic_extremes | 4 | 24 |
| r20_generated_cast_length_region_product_extremes | 5 | 30 |
| r20_generated_borrow_capacity_sum_extremes | 1 | 6 |
| r20_generated_certificate_width_bound_count_table_extremes | 10 | 60 |
| r20_region_domain_extremes (direct semantic consumer) | 5 | 30 |
| r20_exhaustive_domain_cardinality_extremes (exhaustive/sample direct consumers) | 2 | 12 |
| r20_generated_schedule_count_selector_extremes | 8 | 48 |
| r20_cli_interface_region_storage_extremes | 2 | 12 |

S1 applied: all original **507** R13 inputs retained, **84** added. Independent
§§7.1–7.2 range/non-overlap/capacity and §§3.7/5.4 width formulas assign legal,
layout rejection, deterministic representation or allocation-dependent storage
before production. Observed **167 constructed / 342 layout rejections / 82
resource diagnostics**. Allocation-dependent cases allow success or a cited
host allocation failure; this is finite boundary coverage, not an exhaustive
resource-availability or full-u64 proof.

All **5 ignored release sweeps passed / 0 failed**, **106.12 s** runner sum:

| Sweep | Full domain | Result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 pairs × 4 = 262,144 executions | 1/0; 1.09 s |
| 17-wire borrow | 131,072 entries | 1/0; 5.81 s |
| checked borrow certificate | 262,144 elements, subject and expected program | 1/0; 23.01 s |
| §6 differential | 200,000 programs: 70,931 accepted clean / 129,069 rejected | 1/0; 74.22 s |
| recursive prefix reversal | 256 inputs × every completion/first-fault prefix | 1/0; 1.99 s |

Fmt/fmt --check, strict offline all-target Clippy (0 warnings), unchanged Python
seed (2 examples / 2 envelopes), independent R20 pins and byte comparisons pass.
Production/test inputs stayed fixed throughout the final full run and sweeps.
Implementation commit **af52c9cd42965c1f7fd3fa0bcd4139673074e10c**; this block records final evidence.
Logs: interpreter .tmp/r20-{baseline-clean,exact-red,class-red,allocation-red,
cert-red,focused,widths,final,final-clippy,sweeps}.log.

No new spec gap; no user decision needed for this repair. Deliverables 8–11
remain staged/Unverified. Only owned paths committed; Beads auto-export left
for the orchestrator. No foreign changes, worklog edits, push or Dolt sync.

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
3. Continue generalized-width and host-resource verification beyond the R20
   finite boundary family. Lifted widths are u64 descriptors; concrete vectors
   and Q basis execution already exceed128. Audit remaining JSON/JCS numeric
   corner cases and spellings without an ISA width cap.
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

# Bennett ISA reference interpreter progress

Resume here. The whole accepted ISA 0.8 remains in scope; staged deliverables
are not complete. Sole implementation sub-agent; owned interpreter and external
round-18 response only. Never push. Every cargo invocation uses the headless
prefix and TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp.

## Round-18 fixes (2026-10-31)

Baseline **403 passed / 0 failed / 5 ignored**, summed over all 33 binaries
(including 0-test main/doctests), **22.51 s** runner sum. Literal witness RED:
**1 passed / 2 failed** (library admission and CLI check/hash wrongly accept;
independent outer-two/nested-three execution already passes). All round 4–18
positive and negative tests are retained, including every R17 permutation,
union/partial-coverage and zero/large-loop test and byte pin.

| Blocker | RED tests | Status | Commit |
|---|---|---|---|
| R18-B01 | metadata::r18_literal_nested_budget_ties_reject; cli_xam::r18_cli_literal_nested_budget_check_hash_reject | GREEN | pending owned-path commit |

Design: `metadata.rs::validate_schedule_details` passes a root bound only for
an oracle root. `metadata/schedule_domains.rs::InvocationSteps` tracks root,
nested-token and action domains through each chain. `chain_domain` keeps
lift.call/lift.perm_call in the same machine; lifted oracle call/uncall and
normalized oracle_call open a new invocation. params_hash supplies no token
preimage: nested domains remain symbolic over positive u64 steps, never
intersected with the root B. Known root B=0 and zero-trip ancestor loops remain
empty. No execution budget, reference body or canonical schedule is changed.
Only equal maxima with an uncovered selector-domain remainder reject; UNION
subtraction and document-order-independent resolution are unchanged.

The old shortest-chain coverage argument needed refinement too: a finite root
step domain can be fully covered while a nested invocation's larger one is
not. `has_uncovered_tie` visits call extensions until it reaches the symbolic
nested domain. Beyond the longest selector path it memoizes suffix call-graph
states, so recursive lprocs terminate without a depth/loop/step expansion cap.
Once a whole symbolic domain is covered, its extensions cannot broaden it.
The same-machine recursive sum_db control and the root-prefix union control
exercise both cases.

Literal reviewer files and probe copied verbatim into tests/support/.
Both B=2 and B=3 files reject with §11 multiple maximal matching schedule
entries; CLI check/hash exit 1 with empty stdout. Single-entry positives keep
object 8e6e852c…f815 and ignore interface=closure 9687f21a…89012.
A/B with wildcard iterations and literal fully specific C admit and resolve C
in all six orders, at generated 0, 1 and u128::MAX; CLI also checks all orders
and identity pins at both root budgets. The literal duplicate's only remaining
wildcard is generated, so a single concrete generated ordinal would NOT cover
it; the covering control deliberately broadens A/B's iterations instead.
Lproc-only root step 3 at B=2 stays empty; nested-oracle/lproc and action-root
variants reject. Outer B=2 with token B=3 still executes payload=9/code=0/steps=2.
Independent r18_pins.py checks file hashes, original nested fixture bytes,
canonical schedule bytes and the exact token preimage hash. R17 pins still pass.

`r18_nested_specificity_family_matches_brute_force` adds **4,800 schedules +
4,800 reversed orders**, **3,873 admitted / 927 rejected**, **100% oracle
agreement** for admission and every admitted resolver result. Fifteen shapes
cover call depth 1–3, oracle prefixes/lproc suffixes (all mixes valid under
§4.2), action roots, root B in {0,1,2,4} and independently varied per-token
budgets 0..4: **1,532 assignments**, **121,182 concrete operation partitions**
before deduplication. The oracle derives owning machines from O/L edges,
enumerates static dispatch-row operations at each legal concrete step and
both phases, then computes the literal specificity maximum without production
matching/domain helpers. Steps 1..3 are named selector values and 4 is the
residual; generated 0,1 and residual 2 exhaust this family's distinctions.
Admission cannot invert params_hash, so expectations use the union over token
assignments; a token-specific infeasibility is never credited as a proof of
emptiness. This finite family does not claim full schedule/target execution.
The original R17 family/distribution and all other prior assertions are intact.

First full suite: **411 passed / 0 failed / 5 ignored**, **29.82 s** runner sum.
Final full suite after the Clippy-only iterator spelling fix: **411 passed /
0 failed / 5 ignored**, **28.65 s**, **14 unit + 397 integration**, 0 doctests,
summed over all 33 binaries. Focused R18 tests: **8 passed / 0 failed**,
including the 4,800-case family. Fmt/fmt --check and strict all-target Clippy
pass with 0 warnings. Initial build-only corrections were a Rust-2021 let-chain
spelling and a test helper name shadow; initial Clippy requested enumerate()
in the new independent oracle loop. No production behavior or sweep input was
changed for those corrections. All five ignored sweeps are running; production
and test inputs/assertions are frozen from sweep start. Logs: interpreter .tmp/r18-{before,red,green,controls,family,
full,clippy,sweeps}.log. Only owned paths; no foreign tests, dependency changes,
push or Dolt sync. Full-ISA bead b-isa-spec-zq2 remains active for staged work.

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

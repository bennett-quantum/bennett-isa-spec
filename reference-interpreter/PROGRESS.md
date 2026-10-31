# Bennett ISA reference interpreter progress

Resume here. The whole accepted ISA 0.8 remains in scope; staged deliverables
are not complete. Sole implementation sub-agent; owned interpreter and external
round-18 response only. Never push. Every cargo invocation uses the headless
prefix and TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp.

## Round-18 fixes (2026-10-31)

Baseline **403 passed / 0 failed / 5 ignored**, summed over all 33 binaries
(including 0-test main/doctests), **22.51 s** runner sum. Original reviewer RED:
**1 passed / 2 failed**, root-B clipping admits the nested tie. Initial local
commit 9f7ea64 implemented the reviewer's symbolic-range remedy; superseded by
the orchestrator's INV-5 addendum because that remedy over-rejects token B=2.
Addendum RED: **0 passed / 1 failed**, literal deferred-admission test rejects
under that symbolic-range commit. Both directions are repaired below.

| Blocker | RED tests | Status | Commit |
|---|---|---|---|
| R18-B01 | metadata::r18_literal_nested_budget_ties_defer_until_run (original admission-negative witness, revised by addendum); cli_xam::r18_cli_literal_nested_budget_check_hash_defer_and_run_reject | GREEN | 42d4049 |

Design: `metadata.rs::validate_schedule_details` supplies a bound only for an
oracle root. `metadata/schedule_domains.rs::InvocationSteps` tracks root,
nested-token and action domains. lift.call/lift.perm_call stay in the same
machine; lifted oracle call/uncall and normalized oracle_call enter a new
invocation. A nested step is NEVER clipped by root B. ISA 0.8 has no object-
level token constant, only classical parameters: the derivable-budget branch
cannot fire at admission. Unknown nested domains are DEFERRED, never taken as
empty or universally quantified. Root domains still use exact union subtraction.

INV-5 steps 1.1.5, 1.4.2, 1.4.3 and Spec gaps item 10 determine the addendum:
check/hash cannot decide actual nested-budget ties from object + params_hash.
SPEC-QUESTIONS 33 records the 0.9 gap and both unsound alternatives; the
orchestrator owns bead b-isa-spec-6hs. No token encoding or 0.8 amendment is
invented. The bound Some branch is shared with admission so a future derivable
budget can use it, if 0.9 defines one.

`Object::check_schedule_matches_for_run` binds actual token arguments, forwards
them by callee parameter (including permutations), and uses each invocation's
own budget. It sends concrete uncovered selector-box witnesses to
resolve_schedule, which errors §11 multiple maximal matching schedule entries
before any realisation. Hooks precede mutation in both oracle engines and
action execution. This is a matching preflight, not full target/timing/expansion
execution. Known finite domains inspect call extensions; suffix states include
bound arguments and budgets and terminate recursive lprocs without a depth cap.
Zero-trip loops, root B=0 and zero-budget nested ancestors stay empty.

Literal reviewer files and probe copied verbatim into tests/support/. Both
root B=2 and B=3 witnesses now pass check/hash with their original identities
and bytes, and reject at run-oracle with token B=3 (exit 1, empty stdout,
§11 diagnostic). Token B=2 runs clean: payload=0/code=0/steps=2; inc8 has not
reached retval, and ignore intentionally does not propagate the nested fault.
The unscheduled token-B=3 reference still gives payload=9/code=0/steps=2.
Root step 3 is dead at B=2 and rejects a true root tie at B=3. Machine creation
and action execution reject bound ties before mutation; the action's complete
QuantumState remains equal. Single-entry positives and genuinely covering
A/B/C entries admit, preserve object/closure pins and resolve C in all six
orders. The literal duplicates are fully specific except generated="*";
covering controls broaden A/B's iterations, not one generated ordinal.

`r18_nested_specificity_family_matches_brute_force`: **7,500 schedules + 7,500
reversed orders**, **6,894 admitted / 606 root-rejected**, **319 deferred ties
materialise at binding / 6,575 bound-clean**. **100% oracle agreement** for
root-only admission, actual-budget bound matching and concrete resolver results,
including errors. Fifteen shapes cover depth 1–3 oracle prefixes/lproc suffixes,
action roots, root B in {0,1,2,4}, and every independent token assignment 0..4:
**1,532 distinct assignments**, **569,910 concrete operation partitions**.
The oracle derives owning machines from O/L edges before calling production;
steps 1..3 and residual 4, both phases, generated 0,1 and residual 2 exhaust
this finite family's selector distinctions. It models EACH actual budget map,
never their union. This does not claim a complete expansion-library executor.
The original R17 1,200-schedule family/distribution and all six literal orders,
union/partial coverage, zero/large loops, all defaults and pins remain green.
All round 4–17 assertions are retained; R18 admission expectations follow the
explicit addendum. r18_pins.py and r17_pins.py and all three reviewer cmp checks
pass; no reference body, token budget or canonical witness byte was altered.

Final full suite after the addendum: **412 passed / 0 failed / 5 ignored**,
**31.66 s** runner sum, **14 unit + 398 integration**, 0 doctests, over all
33 binaries. Focused final R18: **9 passed / 0 failed**, family 10.47 s.
Fmt/fmt --check and strict all-target Clippy pass with 0 warnings. The earlier
411-test full runs/sweeps preceded the addendum and are not final evidence.
All **5 ignored sweeps passed / 0 failed**, **96.71 s** runner sum across all
33 binaries, rerun on final bound-matching code. Production, test inputs and
assertions were unchanged from their start.

| Sweep | Full domain | Result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 operand pairs × 4 = 262,144 executions | 1/0; 0.51 s |
| 17-wire borrow | 131,072 entries | 1/0; 2.86 s |
| checked borrow certificate | 262,144 elements, subject and expected program | 1/0; 14.80 s |
| §6 differential | 200,000 programs: 70,931 accepted (all clean), 129,069 rejected | 1/0; 76.14 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 1/0; 2.40 s |

Implementation commit 42d404924e68d4a40524f6c35accdcc7c60cfc73; this block
records final evidence. No user decision is needed for this repair; the 0.9
spec decision remains tracked by b-isa-spec-6hs.

Logs: interpreter .tmp/r18-{before,red,addendum-red,addendum-green,bound-full,
bound-clippy,bound-final,bound-sweeps}.log. Only owned interpreter paths and
external response; no foreign tests, dependency changes, push or Dolt sync.
Concurrent spec/conformance/Beads changes are untouched. Full-ISA bead
b-isa-spec-zq2 remains active for staged work.

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

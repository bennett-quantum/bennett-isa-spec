# Editor response: council round 18

**R18-B01 is resolved under the orchestrator's INV-5 addendum**; none declined
or partial. Root-only admission cannot decide nested-budget ties from
params_hash. The literal witness therefore **admits at check/hash and rejects
at run-oracle with token B=3**; token B=2 runs clean. This intentionally
supersedes the reviewer's request for unconditional admission rejection.
Determining proof: INV-5 **1.1.5, 1.4.2, 1.4.3**, Spec gaps item 10.

Implementation commit: **42d404924e68d4a40524f6c35accdcc7c60cfc73**.
The initial local **9f7ea64e364a4564d09cba311c171a9eab0039ea** implemented the
reviewer's symbolic-range remedy; the addendum demonstrates its over-rejection
and the implementation commit supersedes it. Final evidence commit: **fed0d19a026ae35ea175a4c97edba2d1f2e7f68a**.
Only owned interpreter paths and this response are committed. No push.
Concurrent spec/conformance/Beads changes are untouched.

## Design

1. §11 steps belong to their invocation. `InvocationSteps` distinguishes the
   known root bound, a nested invocation with an optional resolved bound, and
   action-level operations. `lift.call` and lifted action gates stay within
   the enclosing oracle machine. Lifted oracle call/uncall and normalized
   oracle_call enter token-governed invocations. Root B never bounds them.
2. ISA 0.8 has **no object-level token constant**. Tokens are classical
   parameters, so the derivable-nested-budget branch **cannot fire at admission**.
   Whether 0.9 adds such an encoding is a spec decision; no encoding or test
   fabricating one was added. A resolved bound uses the same domain path, ready
   for a future derivable bound if the spec defines one.
3. Admission checks root-owned domains exactly, including lprocs sharing that
   root machine, and **defers** unknown nested domains. Deferred is neither a
   proof of emptiness nor universal quantification over possible budgets.
   INV-5 1.4.3 shows why both alternatives fail: root clipping admits the
   token-B=3 witness, while an "any budget" range rejects its valid B=2 run.
4. `Object::check_schedule_matches_for_run` binds the actual classical token
   arguments, follows forwarding by callee parameter, and checks each nested
   invocation against its own resolved budget. A zero-budget ancestor has no
   descendant operations. No token-budget inference from params_hash occurs.
5. The bound matching preflight constructs a concrete uncovered selector-box
   witness and passes it to `Object::resolve_schedule`. At that concrete
   ambiguity the resolver raises `§11: multiple maximal matching schedule
   entries`, before any realisation. Both oracle engines and action execution
   invoke this gate before mutation. No silent first-entry choice is possible.
6. Equal-specificity overlap is still only a candidate. Intersection followed
   by subtraction of the UNION of all strictly higher entries determines
   whether any equal-maximum remainder exists. R17 shadowed ties stay admitted
   in all six orders. The resolver's order-independent final-maxima rule stays.
7. Step/loop counts are not expanded or capped. Known finite domains also visit
   call extensions whose invocation budgets can differ; suffix states include
   bound arguments and budgets beyond the longest selector path. Recursive
   lprocs terminate without a depth cap. Empty paths split over real points;
   zero-trip loops and root B=0 stay empty.
8. Generated ordinal ranges still belong to the staged expansion library.
   This is schedule **matching** verification over selector partitions, not
   completion of target selection, timing, expansion execution or numerical
   claims. Those deliverables remain staged/Unverified. Other INV-5 readings
   are not silently imposed by this budget repair.
9. SPEC-QUESTIONS **33** records the 0.9 budget-binding/check-timing gap, both
   unsound alternatives, the absence of object token constants and the needed
   spec decision. The orchestrator owns **b-isa-spec-6hs**. Possible 0.9 choices
   are explicit deferred matching until token binding, or recoverable nested
   budgets in bindings; token JCS interchange must also be defined. No 0.8
   normative bytes or new object fields were changed.

## R18-B01 — resolved

Determining clauses: §11 Matching (ISA-11-P4), §5.2, §4.2. INV-5 1.1.5 establishes
per-invocation budgets; 1.4.2 establishes exact bound matching and harmless
shadowed ties; **1.4.3 refutes exact unbound admission** from object + params_hash.

| Blocker | RED tests | Final fix locations | Commit |
|---|---|---|---|
| R18-B01 | metadata::r18_literal_nested_budget_ties_defer_until_run (initial admission-negative witness, revised by addendum); cli_xam::r18_cli_literal_nested_budget_check_hash_defer_and_run_reject | metadata.rs::validate_schedule_details; metadata/schedule_domains.rs::{InvocationSteps,chain_domain,tie_witness,concrete_witness,check_schedule_matches_for_run}; oracle.rs::run_oracle_vectors; machine.rs::OracleVectorMachine::new; action.rs::action_internal | 42d4049 |

Original reviewer RED: **1 passed / 2 failed**. Library and CLI admission
accept the genuinely ambiguous nested witness under root-B clipping; the
independent execution derivation already passes. The first symbolic-range fix
was subsequently tested against the addendum: the revised literal test is
RED **0 passed / 1 failed**, because that remedy rejects at admission.
The final implementation repairs both the under- and over-rejection.

Final literal controls, for both supplied root budgets 2 and 3:

- `check` and `hash`: exit 0; object hash unchanged, check status ok.
- `run-oracle ... ignore --input 8 --budget B --arg token:3,0,0,0`:
  exit 1, empty stdout, §11 multiple maximal matching schedule entries.
- Same run with token:2,0,0,0: exit 0, **payload=0, code=0, steps=2**.
  inc8 has not reached its retval; ignore intentionally does not propagate
  the nested fault. This is not a three-step call clipped by the outer budget.
- The unchanged unscheduled fixture with root B=2 and token B=3 still gives
  **payload=9, code=0, steps=2**. No execution budget or body was changed.
- A true root-owned step-3 tie rejects at admission with root B=3, but is dead
  at root B=2. A same-machine lproc step 3 at B=2 stays dead; nested-oracle/lproc
  and action-root ties defer, then reject under their actual token B=3.
- Reversible-machine creation rejects the bound tie. The action-root control
  checks complete QuantumState equality before/after the error: no mutation.

Single-entry positives preserve identity and resolve uniquely. Covering-entry
controls admit and resolve C in all six orders, including CLI check/hash at
both root budgets. The literal duplicate entries are fully specific except
for generated="*"; one concrete generated ordinal cannot cover that wildcard.
The genuine covering control broadens A/B's iteration selectors to "*" and
uses literal C with [] iterations to cover ALL generated ordinals. Resolver
controls include generated 0, 1 and u128::MAX.

`r18_bound_tokens_forward_by_parameter_and_zero_ancestor_is_empty` permutes the
forwarded token arguments, uses distinct budgets at each nested invocation,
and checks both the materialised tie and the dead step-3 control. A zero-budget
ancestor prevents a larger descendant token from manufacturing operations.
`r18_recursive_lproc_keeps_root_budget_without_depth_cap` retains the same
root bound through recursive sum_db lprocs and terminates without truncation.

## Independent byte and identity evidence

Both literal .bisa files and r18-review-probe.rs were copied verbatim from the
read-only review directory into tests/support/; all three cmp checks pass.
The probe's concrete generated=0 resolution is exercised in the library
controls. `r18_pins.py` independently checks original nested fixture bytes,
canonical schedules, both file hashes and the reviewer's token-preimage hash
construction using hashlib/json, without the interpreter or rehash helper.
That construction pins this fixture; it does not declare a normative token
JCS interchange encoding (the latter is part of the recorded spec gap).

| Pin | Root B=2 | Root B=3 |
|---|---|---|
| File SHA-256 | 07c617a312f2b1f333689d95b474ccf9b7dea4a570c43ced87a92c089b66d9e7 | 0584dc9a6dbe02b138e516caca59168a4dbbf577185e9d96504f58d5a5fcf4ae |
| Schedule SHA-256 | 197452169a33d0ced5dce10dbe2949dc42d1b96bcf62a089718ce7534b98c077 | 7e66e4320a43b0d93a2302a71c926f70c9dc37407ca15605a3c898c74a44492b |

Common object hash:
`8e6e852c4717ad55763ec2162b0582896a948c073b649c40002a8aa64dd0f815`.
Ignore interface=closure:
`9687f21af62158ff87ab8a974e056774ee75b531e158cc4538a11d4631f89012`.
The supplied canonical schedules, reference bodies and token-budget pins are
unchanged. `r17_pins.py` also passes with every R17 file/schedule/identity pin.

## Nested specificity family and retained gates

`r18_nested_specificity_family_matches_brute_force` independently derives
operations from O/L edges and concrete budgets before admission or resolver
calls. Fifteen valid shapes cover depth 1–3, all oracle-prefix/lproc-suffix
mixes permitted by §4.2, and action roots. Root B is 0,1,2,4; each nested token
independently ranges 0..4. Steps 1,2,3 plus residual 4, both phases and generated
0,1 plus residual 2 exhaust the distinctions the family's selectors make.
No production overlap/domain/specificity helper computes the oracle maximum.

Observed: **7,500 schedules + 7,500 reversed orders**, **1,532 distinct concrete
budget assignments**, **569,910 concrete operation partitions**. Admission
rejects **606 root ties** and admits **6,894** schedules with any nested check
deferred. Under their actual run-time budgets, **319** deferred ties materialise
and **6,575** are bound-clean. Admission, bound matching and every concrete
resolver result (including the error on a true maximum tie) have **100% oracle
agreement**. This oracle does not take a union over possible token budgets.
It is a finite selector-domain family, not a claim to verify every expansion
library operation or full schedule execution.

All round 4–17 tests/assertions are retained; R18 admission expectations follow
the explicit addendum. All six R17 witness orders, 1,200 schedules plus reversed
orders, union/partial coverage, zero/large loops, all 14 defaults and byte pins
remain green. Earlier borrow/discipline, derived-width, schema, complete inverse
and CC/R nested-uncall tests remain. The nonblocking independent width-outcome
classification follow-up remains staged as in R17; no widths/assertions were
weakened. No foreign conformance tests were run.

## Observed verification

| Gate | Result |
|---|---|
| Baseline, summed over 33 binaries including main/doctests | 403 passed / 0 failed / 5 ignored; 22.51 s |
| Final full suite after addendum | 412 passed / 0 failed / 5 ignored; 31.66 s; 14 unit + 398 integration, 0 doctests |
| Final focused R18 | 9 passed / 0 failed; nested family 10.47 s |
| Fmt and fmt --check | pass |
| Strict all-target Clippy | pass, 0 warnings |
| Independent R17/R18 pin scripts and three reviewer cmp checks | pass |

All **5 ignored sweeps passed / 0 failed**, **96.71 s** runner sum across all
33 binaries, rerun after the addendum on final code and unchanged sweep inputs.

| Sweep | Full domain | Result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 operand pairs × 4 = 262,144 executions | 1/0; 0.51 s |
| 17-wire borrow | 131,072 entries | 1/0; 2.86 s |
| checked borrow certificate | 262,144 elements, subject and expected program | 1/0; 14.80 s |
| §6 differential | 200,000 programs: 70,931 accepted (all clean), 129,069 rejected | 1/0; 76.14 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 1/0; 2.40 s |

Exact final commands (all cargo calls headless, offline where applicable, with
absolute interpreter manifest; test opt-level 1 retains assertions/overflow
checks and complete sweep domains):

~~~sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --no-fail-fast
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --test metadata --test cli_xam --test oracle r18_ --no-fail-fast -- --nocapture
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --no-fail-fast -- --ignored --nocapture
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp cargo fmt --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp cargo fmt --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml -- --check
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp cargo clippy --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --all-targets -- -D warnings
python3 -B /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/tests/support/r18_pins.py
python3 -B /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/tests/support/r17_pins.py
cmp /home/tobias/Projects/bennett-workspace/bennett-isa-spec-review/.tmp/r18-nested-budget-2.bisa /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/tests/support/r18-nested-budget-2.bisa
cmp /home/tobias/Projects/bennett-workspace/bennett-isa-spec-review/.tmp/r18-nested-budget-3.bisa /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/tests/support/r18-nested-budget-3.bisa
cmp /home/tobias/Projects/bennett-workspace/bennett-isa-spec-review/.tmp/r18-review-probe.rs /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/tests/support/r18-review-probe.rs
~~~

Final logs: interpreter `.tmp/r18-{before,red,addendum-red,addendum-green,
bound-full,bound-clippy,bound-final,bound-sweeps}.log`. Earlier 411-test runs
and the first sweep run preceded the addendum and are not final evidence.
Production, test inputs and assertions were unchanged throughout the final
sweeps. Owned diffs pass --check. Full-ISA bead b-isa-spec-zq2 stays active for
staged work; b-isa-spec-6hs belongs to the orchestrator. No dependency, normative
spec, conformance, include, Python-seed or foreign-source changes; no push or
Dolt sync. Commits use explicit owned paths, absolute git -C and
BEADS_HOOK_TIMEOUT=20.

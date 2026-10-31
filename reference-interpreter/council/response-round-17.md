# Editor response: council round 17

Both S0 blockers, **R17-B01** and **R17-B02**, are resolved; none declined or
partial. Sole implementation sub-agent; repo instructions, binding round-7–16
briefs/responses, full round-17 report and reviewer JSON read. Implementation
commit **e72df8cf78aca2b0516e20496b2f4f631df2e504**. Final evidence commit:
**3458961098c5cc4745092b5ccd923d8c038cbc70**. Only owned interpreter paths and this response are committed.
No push. Concurrent spec/conformance work is untouched.

## Design

1. `src/metadata_schema.rs::TABLE` remains the sole recursive structural
   admission grammar, 163 named productions. `gate_payload` declares
   `epsilon_share?:decimal`, with §§11/17 item 9 cited. `realisation` remains
   required. Present malformed values are never converted to defaults.
2. `Object::schedule_epsilon_share` returns an explicit assigned share, or
   the exact recipe `region epsilon / approximate_operations`; expansion
   must supply the latter count. Exact operations return `Exact`. No binary64
   division, synthetic admission-time share or canonical-document mutation.
   The recipe represents nonterminating rational shares without rounding.
3. The requested class audit found one more incorrectly required default:
   `sampler.count` (§17 item 13). TABLE now marks it optional;
   `claims::sample_count` defaults to 65,536 in the actual replay consumer.
   Explicit counts including 0 are preserved and present values use TABLE.
4. `r17_section17_defaults_are_optional_and_applied` is a literal,
   clause-derived table over every numbered §17 item. Where an item has a
   metadata field, the test asserts the optional TABLE key AND admits the
   independent omitted form. It tests implemented consumer observations and
   records the staged/non-metadata boundaries honestly; target aggregation,
   enclosure procedures and hardware/C-layout choices are not silently
   counted as completed object-metadata consumers.
5. Specificity is the literal §11 lexicographic tuple. Equal-score overlap
   is only a candidate tie. `metadata/schedule_domains.rs` intersects both
   entries with legal invocation/iteration domains and oracle steps 1..B,
   then subtracts the UNION of every strictly higher selector of that kind.
   Ambiguity exists exactly when some selector-domain remainder survives.
6. Domains use interval boxes split only at selector boundaries. Zero-trip
   loop bodies are empty; `[]` outside loops is a singleton. Loop and step
   counts are never expanded or capped. Generated ordinals stay symbolic for
   the bound expansion library; this is admission, not target execution.
7. Shortest real invocation chains suffice: every instruction, including a
   call, has its own generated operations (§11). Longer paths cannot cover
   those operations. Covering them with prefixes covers their extensions too.
   An empty selector path is partitioned over real root instructions, never
   a fictitious empty operation chain. Existing path validation checks every
   symbol/program point/call edge and explicit iteration vector.
8. `resolve_schedule` counts maxima only after inspecting all matches, so
   shadowed ties are harmless regardless of document order, and an actual
   final tie errors rather than selecting the first entry.
9. README Design and PROGRESS document the algorithm, defaults and observed
   evidence. No ISA amendment or new narrowing reading is needed. Existing
   target/schedule-execution and rigorous interval-checking stages remain
   staged/Unverified; neither blocker requires claiming those stages finished.

## R17-B01 — resolved

Determining clauses: §17 item 9 and §8.1 define the equal-share default when
no share is assigned, including a realisation-only gate entry. §11's sketch
must not make that omission invalid.

RED tests:

- `metadata_schema::tests::r17_gate_payload_omission_is_literal_clause_control`
  calls literal `validate("gate_payload", {"realisation":"native"})`,
  independently of TABLE sample generation.
- `metadata::r17_default_share_and_shadowed_witness_preserve_identity`
- `cli_xam::r17_cli_default_share_and_shadowed_tie_check_hash_pins`

The initial combined witness run observed **2 passed / 4 failed**: the three
omission positives fail with missing field epsilon_share; the separate
shadowed-witness test fails with the pairwise §11 diagnostic. Existing
malformed-present behavior passes. The class audit's additional
`r17_sampler_count_default_is_optional` fails **0 passed / 1 failed** with
missing count before that production is repaired.

Fix locations: `metadata_schema.rs::TABLE(gate_payload, sampler)`,
`metadata.rs::Object::schedule_epsilon_share`,
`claims.rs::sample_count` and its replay call site.

GREEN: copied omission witness parses, check_hashes succeeds, CLI check/hash
both exit 0 (check status ok), and the exact supplied raw schedule/source,
object/interface/closure identities and budget [3,0,0,0] are retained.
`r17_equal_share_consumer_keeps_exact_and_explicit_assignments` checks
counts 1, 3 and 1,000, exact-operation preservation, explicit shares and
no-schedule behavior. The approximate-operation/count-zero combination is
an explicit consumer misuse error, not an admission restriction.

Five supplied malformed gate controls reject: null, object, negative numeric,
invalid decimal string, missing realisation. Library and CLI check/hash
assert §9.1/§11; CLI asserts exit 1 and empty stdout. The required
realisation production and decimal grammar remain strict. New sampler
consumer controls actually replay 65,536 samples when omitted, preserve
explicit 0 and 3, reject null/object/negative/invalid-string counts and retain
original certificate bytes.

## §17 default audit (all numbered items)

The executable table is
`src/metadata_schema/tests.rs::r17_section17_defaults_are_optional_and_applied`.
N/A means no corresponding object metadata field, not a default silently
implemented by an unrelated field. A default for an uncovered operation does
not make the payload of a present entry of that kind optional semantically.

| Item | Field / decision | Normative default | Where applied / consumer observation |
|---|---|---|---|
| 1 | Binary encoding (no metadata field) | none; §16 text | Text parse/check_hashes; binary encoding is not introduced |
| 2 | Gate-name registry (no metadata field) | §4.4 names only | Unknown opcode load rejection; existing gate dispatch unchanged |
| 3 | Device registry (no metadata field) | four §7.3 models | Existing unknown-device execution faults 8 (`FAULT_EFFECT_UNMODELLED`) |
| 4 | Mechanisation (no metadata field) | none; interpreter is oracle | Actual inc8(255), B=3 yields payload=0/code=0/steps=3 |
| 5 | Opaque C object layout (no metadata field) | opaque | Outside text metadata; no C-layout field is required |
| 6 | Uncovered cleanup; target capabilities.mbu | reverse; false | `schedule_cleanup` returns reverse; supplied cleanup payload still needs realisation; target mbu execution staged |
| 7 | schedule.bindings.expansion | ref at ISA version | Optional in TABLE; schedule_expansion resolves ref/0.8 without mutation (retained R16 repair) |
| 8 | Enclosure procedure (not precision_bits) | Taylor/Lagrange remainder < 2^-(precision_bits+2), modulo outward-enclosed 2π | Interval checker staged; numerical claim explicitly remains Unverified; precision_bits still required |
| 9 | schedule.entries[].gate.epsilon_share | region epsilon / number of ε>0 operations | **Was wrong this round**: optional TABLE field plus exact recipe consumer; exact operations stay exact |
| 10 | any-class resolution (target choice) | first JCS class with capacity | Target capacity matching staged; present schedule alloc class remains concrete |
| 11 | target.resources.*.rate.<class> | 1 | Target aggregation staged; no target production in object TABLE |
| 12 | Uncovered start times / classical timing | classical_step_time each, sequential | Start payload optional structurally; timing execution staged; no fabricated admission-time start |
| 13 | sampler.count; suite selection outside metadata | exhaustive ≤16 interface bits, 2^16 samples above | **Also wrong at audit**: optional count; explicit sampled claims default to 65,536 in replay. No claim kind/seed/algorithm is invented |
| 14 | Hardware unclean qreg.free (no metadata field) | FAILED, FAULT_UB_TRAP at instruction | Existing interpreter unclean-free fixture faults 4 and retained reversal tests pass |

Previously wrong items: **7** (fixed in R16; still covered), **9** and **13**
(fixed here). Other numbered decisions were not wrongly required TABLE fields.
Items 1–5, 8 and 14 do not imply new optional metadata keys. Items 10–12
involve staged target consumers; no completion claim is made for them.

The additional sentence audit covered §§8.1, 9.1, 11 and 12 in full. §8.1's
share rule maps to item 9. §9.1's optional pinned_closure_hash and block_image
are already optional and have unpinned-link and empty-block-map consumers.
§11's uncovered cleanup/gate/class/start defaults are distinct from required
payloads of present entries: cleanup reverse now has a consumer helper;
gate matching, capacity and timing still need staged target execution.
§12 does not default precision_bits, sampler seed/algorithm or expected/domain;
its suite/sample-size choice maps to item 13. No other incorrectly required
default field was found. Present malformed values keep rejecting.

## R17-B02 — resolved

Determining clause: §11 applies the unique maximum among matching entries,
independently per kind and independently of document order. A/B that never
attain the maximum together are not an ambiguity.

RED: `metadata::r17_shadowed_tie_all_orders_and_genuine_maximum_ties` rejects
the reviewer's three-entry witness before the fix with
`§11: §11: multiple maximal matching schedule entries`.

Fix locations: `metadata.rs::Object::validate_schedule_details`,
`metadata/schedule_domains.rs::{has_uncovered_tie, entry_domain, chain_domain,
subtract}`, `metadata.rs::Object::resolve_schedule`.

GREEN: all six A/B/C orders parse/check_hashes and resolve the literal
operation `{path:[{symbol:inc8_hash,point:"entry:0",iteration:[]}],step:1,
phase:"fwd",generated:0}` to C. A/B alone rejects; duplicated C rejects;
A/C admits. The exact copied witness also passes pinned CLI check/hash.
Raw schedule/hash and all identity pins are preserved. No run input or
budget change was added.

`r17_union_coverage_partial_coverage_and_budget_domains` admits coverage by
separate higher fwd/rev entries, rejects the partially covered variant and
checks that step 4 is outside the oracle budget 3. The loop-domain controls
admit the union covering both iterations of bound 2, reject the partial union
at bound 3 and 2^40, and admit wildcard ties inside a zero-trip body (no
matchable operation). No loop expansion is used.

`r17_specificity_lattice_family_matches_brute_force` generates **1,200**
schedules with **2–4 entries** and deterministic random wildcard/specific
path, iteration, step, phase and generated dimensions. It independently
enumerates **54 concrete patterns per schedule** (three real inc8 points,
three budget steps, both phases and two named generated values plus a
residual value). These finite partitions exhaust distinctions made by the
family's selectors. It computes the literal tuple maximum without calling
production specificity/overlap/domain helpers. Observed **1,168 admitted /
32 rejected**, **100% oracle agreement**; all **1,200 reversed orders** agree
too. Every admitted resolver result is also compared to the oracle, including
uncovered operations. The literal permutation/union/loop tests supplement
the family rather than changing its random distribution.

## Independent byte and identity evidence

`tests/support/r17_pins.py` reconstructs both witnesses from the R13 corpus,
independently JCS-serializes the modified schedule, checks exact file bytes,
and hashes the literal 261-byte LF inc8 envelope using hashlib. It does not
call the interpreter or rehash helper. Both copied files compare equal to
the read-only reviewer originals. Library/CLI assertions independently
check the literal pins.

| Pin | Omitted epsilon_share | Shadowed tie |
|---|---|---|
| File SHA-256 | 8af833c21de5d94c6684b36cf115c46f5080f528fc898ece2bdda9091b9fb2a5 | 9fc08dca3c5697d9fcad31bf82f4a4a566d0fda6959d10675859d748696c1aff |
| Schedule §9.2 SHA-256 | d9025ef24bfd8db01e4e3e0f5b28be776c4e5585b000fd5bd2333122c0d4845d | e13ea989795d205fcf983b033f24bf1945d8be197c241b3a440a67b2489c287f |

Common object hash: `8093da48bad04d6564325faf01ea1304a84b5ac5a120e685eee9951f5daec5d9`.
Common interface=closure hash:
`8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93`.

## Retained gates and observed verification

All round 4–17 tests are retained. This includes literal R16 omitted and
malformed expansion controls, exact epsilon-value/frame cardinality tests,
T3 borrow/frame/certificate negatives, nested opaque pairs and causal-reader
controls, zero-trip/carried-binding controls, literal identity pins and
fixture corrections, oracle tight budgets/first faults, complete recursive
step inverses and CC/R nested-uncall whole snapshots. Neither oracle/machine
execution nor borrow/width implementation was modified.

Schema family: **163 productions, 9,897 generated positives, 1,653 unique
typed paths, 26,746 grammar negatives**, with **1,992** negatives also through
both admission validators (subset). **26 pinned objects, 2,393 full-parse
path visits, 17,173 full-parse negatives**. Total negatives without counting
dual assertions twice: **43,919**. This proves TABLE enforcement; independent
§17 literal controls detect a wrongly required default. The retained width
family still spans 507 cases. The nonblocking independent width-outcome
classification follow-up remains deferred for the same allocator-vs-host-
representation distinction recorded in R16; no width domain or assertion
was weakened here.

| Gate | Observed result |
|---|---|
| Baseline full suite, summed over 33 binaries | 390 passed / 0 failed / 5 ignored; 22.35 s |
| First full suite | 403 passed / 0 failed / 5 ignored; 37.84 s |
| Final full suite after test-only hex-digit grouping | 403 passed / 0 failed / 5 ignored; 34.62 s |
| Clause-derived §17 test | all 14 items enumerated; omission/consumer or explicit staged/N/A observation |
| Specificity oracle | 1,200 schedules + reversed orders; 100% agreement |
| Derived schema family rerun | 1 passed; 13.61 s |
| Fmt and fmt --check | pass |
| Strict all-target Clippy | pass, 0 warnings |
| Independent pin reconstruction / witness comparisons | pass |

All **5 ignored sweeps passed / 0 failed**, **135.70 s** runner sum across
all 33 binaries.

| Sweep | Full domain | Observed result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 operand pairs × 4 = 262,144 executions | 1/0; 1.24 s |
| 17-wire borrow | 131,072 entries | 1/0; 5.51 s |
| checked borrow certificate | 262,144 elements, subject and expected program | 1/0; 28.56 s |
| §6 differential | 200,000 programs: 70,931 accepted (all clean), 129,069 rejected | 1/0; 96.16 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 1/0; 4.23 s |


Every cargo call uses the headless prefix below, absolute manifest and only
the interpreter crate. Test optimization 1 keeps assertions/overflow checks
and the complete sweep domains. Exact commands:

~~~sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --no-fail-fast
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --lib --test metadata --test cli_xam r17_ --no-fail-fast -- --nocapture
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --lib r17_sampler -- --nocapture
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --lib --test metadata --test cli_xam --test r1_certificates r17_ --no-fail-fast -- --nocapture
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --test metadata r17_loop_iteration -- --nocapture
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --lib r14_generated_metadata_schema_type_family -- --nocapture
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --no-fail-fast -- --ignored --nocapture
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp cargo fmt --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp cargo fmt --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml -- --check
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp cargo clippy --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --all-targets -- -D warnings
python3 -B /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/tests/support/r17_pins.py
cmp /home/tobias/Projects/bennett-workspace/bennett-isa-spec-review/.tmp/r17-default-epsilon-share.bisa /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/tests/support/r17-default-epsilon-share.bisa
cmp /home/tobias/Projects/bennett-workspace/bennett-isa-spec-review/.tmp/r17-shadowed-tie.bisa /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/tests/support/r17-shadowed-tie.bisa
~~~

Logs: interpreter `.tmp/r17-{before,red,red-count,regressions-final,loop,
schema-family,full,final,clippy,clippy-final,sweeps}.log`. The initial Clippy
finding was only unusual grouping in an unchanged RNG seed literal; final
Clippy and suite reran after fixing its spelling. All five sweep binaries
use unchanged implementation/inputs/assertions throughout. Owned diffs pass
--check. No spec/conformance/include/Python-seed edits, foreign test execution,
dependencies, Beads mutation/export, push or Dolt sync. Explicit owned path
commits use absolute git -C and BEADS_HOOK_TIMEOUT=20.

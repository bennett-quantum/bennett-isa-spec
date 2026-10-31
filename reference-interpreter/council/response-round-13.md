# Editor response: council round 13

All three S0 blockers are resolved: SOL-R13-B01, SOL-R13-B02 and codex
R13-B01. None declined or partial. Sole implementation sub-agent; read the
binding round-13 and round-7–12 briefs, both members' full reports/JSON and
repository instructions. Implementation commit **3242123**; generated-family
and schema-preflight commit **47961e7**; proof-addendum/final-evidence commit
**22488c8**. Only owned interpreter paths and this response are committed;
no push. The concurrent spec/conformance/Beads work is untouched.

## Design

1. `metadata_types.rs::Object::validate_metadata_types` checks every accepted
   metadata section before identity, binding or execution consumers.
2. Record/array/map fields and optional/tagged payloads are schema-typed.
3. Integer number/string spellings, decimal alternatives, expression ASTs,
   wildcard selectors and nullable verification indices retain their forms.
4. Existing semantic validators still check ranges, names and bindings.
5. Every device/model/service version is a string before model compatibility.
6. Host services retain arbitrary string versions; no model-0.8 rule is added.
7. `block_image`, when present, names defined data using the data NAME syntax.
8. Its complete-512-byte-sector invariant is checked at load for every kind.
9. The classical constructor also uses a checked image lookup for mutable Objects.
10. Derived Hc/Ac/Sc additions are checked; depth and bit-vector storage already
    use checked arithmetic. Host representation errors cite the ISA.
11. Legal layout admission and sequential B=0 semantics stay unchanged; no
    exponent cap or quiet fallback is introduced.
12. Identity still implements §9.2 as written. The cross-object dotted-path
    collision is recorded as a 0.9 spec defect, not repaired in the interpreter.

## SOL-R13-B01 — resolved

Section §9.1. The exact LF-terminated inc8 object plus
`{"models":{},"services":{"org.bennett.test":{"version":7}}}` was admitted.
The oracle does not use devices, so its envelope/interface/closure stays the
literal inc8 envelope and pin `8ce5e64f…5be93`, independent of the unused service.

RED tests:

- metadata::r13_service_versions_require_strings
- cli_xam::r13_cli_rejects_nonstrings_in_service_versions

Both failed: Object::parse returned Ok, and check returned exit 0/status ok.
GREEN tests reject 7, false, null, [] and {} at admission with §9.1. CLI
check/hash exit 1 with empty stdout. String "0.8" still checks with the
supplied object hash; "vendor.2026" also checks without repinning inc8.

Change: `validation.rs::Object::validate_metadata` reads `version.string()`
for both groups before the models branch. The generated-family step adds
`metadata_types::devices` so all fields are typed before downstream consumers.

## SOL-R13-B02 — resolved

Sections §§3.7, 5.2, 7.2. Both supplied u64-max layouts remain legal and check
successfully. Their unchanged inc8 body and signature gain exactly seven
ordered scalar lines: A_tab, H, S_cells, heap_base, heap_limit_max, null_size,
ptr_width. Empty globals/interface maps add no lines. Large scalar values are
quoted decimal strings, including in the envelope. This independently gives
both supplied pins; no interpreter or support::rehash defines expectations.

RED tests:

- r1_widths::r13_derived_counter_width_overflow_reports_host_limit
- cli_xam::r13_cli_counter_width_limit_is_a_diagnostic

Library derivation unwound on unchecked s+1; CLI reverse exited 101 instead
of 1. GREEN exercises both exact objects, checks their full literal envelopes,
interface AND closure pins, and whole-object hashes. Sequential run-oracle
at input 8 and budget 0,0,0,0 stays exactly (payload,code,steps)=(0,1,0).
DerivedWidths::derive and OracleVectorMachine::new return explicit
§3.7 host-limit errors without unwinding; CLI reverse exits 1, empty stdout,
with a cited host-limit diagnostic. Ordinary S_cells=9 recursive reversal and
all complete-tuple/prefix assertions are retained.

Change: `machine.rs::DerivedWidths::from_program` checks s+1, a+1 and h+2,
and cites §3.7 on exponent-to-usize representation errors. Adjacent paths
were audited: depth+2 is checked, P/Did are bounded bit counts, W_cell and
slots take maxima, limb count uses checked addition and storage uses
try_reserve_exact. The generated width family covers those layout paths and
both the usize/limb and depth/u128 boundaries without large allocations.

## Codex R13-B01 — resolved

Sections §§9.1, 7.3. The exact action-only object with block_image=42 has
unchanged action interface/closure `2fcd5b3d…ab8be`: actions exclude layout.
Both the malformed numeric object and undeclared-name variant originally
passed check/hash. The auxiliary zero-stored-hash R object deliberately tests
parse/new independently of check_hashes.

RED tests:

- parser::r13_block_image_schema_is_checked_without_a_layout_consumer
- parser::r13_classical_missing_block_image_never_panics
- cli_xam::r13_cli_rejects_invalid_block_image

All three failed at malformed admission. GREEN rejects 42, null, true, {},
[], empty/invalid names and undeclared data at load with §9.1, irrespective of
an envelope consumer. CLI hash/check/run-action exit 1 with empty stdout.
Absent-field and declared disk=512 zero-byte controls check and run
X|0>=|1>, while the action envelope/interface/closure stays unchanged. The disk
symbol has its own independently derived data pin, checked by check_hashes.
A short image is rejected at load with §7.3. The exact R parse path now rejects
first; a separately post-load-mutated public Object also returns a cited
constructor error without unwinding.

Change: `object.rs::Object::validate_layout` resolves the optional data NAME
and checks sectors for every function kind; `classical.rs::ClassicalMachine::construct`
uses get/ok_or_else instead of indexing. `metadata_types::layout` provides
consumer-independent field typing before identities.

## Independent witness pins

`tests/support/r13_pins.py` uses only literal body/signature/layout lines,
Python json and hashlib. It asserts every supplied pin and emits the full
LF-terminated objects/envelopes in r13_cases.json. Invalid-object hashes below
are independent byte pins, not claims of successful admission after the fix.
All supplied pins matched; no member correction was needed.

| Witness | Interface = closure | Object hash |
|---|---|---|
| service version=7 | 8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93 | a63cb1dd51929e217eb8e16e261707a4103a92ff54c20caf2bb614ba73e65734 |
| service version="0.8" | 8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93 | 3bc26d4621ba2411e29615a854bce6bb4299fe85a68df89479c325fcb0836b11 |
| S_cells=u64::MAX | dc736293ea943fd3d8f902ae1589e3e1cbc461ca42bd9229518b33ec65b724b1 | f581d1d9418da1213057fb7e6cc95c3d35619aa478f9c6fc275502a5371bd44a |
| A_tab=u64::MAX | dc2c21a6b7c2a0fea8a6881564f80b44c3742d681e71576ec6f4d66fa058f683 | cf57de219961a2cc9e02b768a8b59b984b2ba2c5aa40b48c333b1ffcbb405ef0 |
| action block_image=42 | 2fcd5b3d60b42ba6092447586a2157bf44ba3a6479ab2eb319bb67665feab8be | 3531fb9b077a90c0fef79688391192df706d99c369a21b0a9a1bc8e35ee75642 |
| action block_image="missing" | 2fcd5b3d60b42ba6092447586a2157bf44ba3a6479ab2eb319bb67665feab8be | 04e497abf45bd7012b5341da012a51c7f79a47b5efce0fcc5a7f35af1639b7de |
| action block_image absent | 2fcd5b3d60b42ba6092447586a2157bf44ba3a6479ab2eb319bb67665feab8be | ddc5d6729a98484acc2d5e8feb585c155799293bd36f31fd96cbc510f77666f0 |

## Generated families and suggestions

One table-driven test, `metadata::r13_generated_metadata_schema_type_family`,
walks **2,268 field visits** across **25 literal, correctly pinned positive
objects** and applies **12,849 negative mutations**. It covers all seven
metadata sections, all function/data/import signature forms, optional import
pins, nested globals/interface mappings, block_image, all four device models,
services, all schedule payloads/selectors, all coverage classification payloads
and certificate claim/optional productions. Each incompatible JSON kind is
substituted; integer negatives also include -1, raw 2^64−1, raw 2^64, 1.5 and
an invalid numeric string. Unsafe raw integer JSON spellings additionally
violate canonical serialization; that diagnostic cites both §16.1 and §9.1.
Canonical large strings stay accepted where the field admits them. All negative
assertions call only Object::parse; the new preflight also precedes internal
identity/binding consumers. Accepted unions come from the schema, not from
observing an interpreter result.

The initial family RED reported 1,282 failures, largely existing rejections
missing §9.1 or earlier consumer diagnostics. Six were expectation errors:
legal scratch number expressions and channel Y arrays were initially classified
too narrowly. Corrected the independent union table, preserving those forms.
The final family passes all positives/negatives without narrowing admission.

`r1_widths::r13_generated_derived_width_extremes_family` has **507 cases**:
**159 constructed, 286 cited layout rejections, 62 cited resource diagnostics**,
no panic. Every numeric layout path includes nested bases/sizes, across inc8,
heap, sum_db, tiny_memory and a 128-bit-address layout. Twelve values include
0/1, 63/64, 127/128, u64-max neighbours through 2^64; fifteen depth limits
extend to u128 d+2 overflow. Independent formula assertions cover Hc/Ac/Sc,
Dc and W_cell; fixture closures exercise P/Did/W_max and call/ret/allocation
slot sizing. Admitted layouts retain sequential B=0 (0,1,0).

All three member S1 suggestions are applied:

- SOL's typed-field mutation matrix, extended through implemented §11/§12/§13 schemas.
- Codex's test rename to interface_region_names_cannot_alias_layout_globals,
  with SPEC-QUESTIONS 32 comment; fixture and rejection assertion unchanged.
- Codex's README optional-field schema/validator/positive-negative-test index,
  starting with block_image, pinned_closure_hash and coverage payloads.

README also names the static cleanup, full-frame factorisation and runtime
return validators. Staged target/launch/schedule-execution/interval verification
is still explicitly staged, never claimed complete by these schema checks.

## Proof addendum: recorded §9.2 limitation

Read bennett-conformance/proofs/INV-4-envelope-injective.md, step **1.5.3**.
SPEC-QUESTIONS 31 now records that the original-component tie order gives
CC/R uniqueness but not cross-object injectivity: ("a","c.db") and
("a.c","db") at base 32 serialize identically. Candidate 0.9 fixes are
JCS-array leaf paths or a '.' ban in interface_regions keys; the latter needs
a NAME/dialect-mangling and rule-10 review. Neither is applied to ISA 0.8.
The orchestrator already filed its bead; this sub-agent did not touch Beads.

`identity::r13_spec_limitation_cross_object_dotted_leaf_collision` passes with
no hashing implementation change. Four independently pinned objects (two CC,
two R) have distinct structured layouts and whole-object hashes, but identical
reference-free @main envelopes within each pair. Their interface/closure pins:
CC `b16b186f89d22817eb40c0996e3f62e1a664267f2969d74c80aaa844458ed3b5`;
R `b96f8c9b770bbea38a4c5e0e15acd50a6f2dda7fe6785e5b892f7f5cb2762442`.
This pins serialization information loss, not a cryptographic SHA-256 collision
or desired behavior. r13_spec_limitation_pins.py derives complete envelopes and
stored-H object pins without the interpreter. All three R13 corpora regenerate
byte-identically: 8 witness records, 25 schema exemplars, 4 limitation records.

## Retained checks and observed verification

All round 4–13 positives and negatives remain, including literal R12
lossless-leaf/exact-owner/address pins; recursive complete-tuple reversals;
first-fault/budget/stack boundaries; per-return/frame borrow equations;
borrowed-touch restrictions; zero-trip and changing-width lifetimes;
nested/all-operand opaque inverse histories and changed input/token/control
negatives; exact-state/Born support; and R11 fixture sidecars. RNG/seeds unchanged.

| Gate | Observed result |
|---|---|
| Baseline, summed over all binaries | **365 passed / 0 failed / 5 ignored**, 67.14 s runner sum |
| Combined witness RED | **0 passed / 7 failed**, all four test targets run with --no-fail-fast |
| Combined witness GREEN | **7 passed / 0 failed** |
| Witness full suite | **372 passed / 0 failed / 5 ignored**, 48.63 s |
| Family full suite | **374 passed / 0 failed / 5 ignored**, 20.56 s |
| Both generated-family tests with --nocapture | **2 passed / 0 failed**; schema 3.93 s, widths 0.13 s |
| Final full suite including proof addendum | **375 passed / 0 failed / 5 ignored**, 18.43 s; 8 unit + 367 integration, 0 doctests |
| fmt --check | pass; 0 tests |
| strict all-target Clippy | pass; 0 warnings |
| owned diff --check and independent corpus regeneration | clean / byte-identical |

Clippy requested as_chunks::<2>() instead of chunks_exact(2) in the new
limitation test. After this style-only edit, that test passed again (1/0),
followed by clean Clippy. No production source changed after the 374-test
full gate or sweep start; the addendum adds tests/evidence/docs only.

All **5 ignored sweeps passed / 0 failed**, **122.27 s** summed runner time.

| Sweep | Full domain | Observed result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 operand pairs × 4 = 262,144 executions | 1/0; 1.25 s |
| 17-wire borrow | 131,072 entries | 1/0; 3.17 s |
| checked borrow certificate | 262,144 elements, subject and expected program | 1/0; 17.88 s |
| §6 differential | 200,000 programs: 70,931 accepted (all clean), 129,069 rejected | 1/0; 96.74 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 1/0; 3.23 s |

Every cargo invocation used the required headless prefix and absolute manifest.
Baseline/witness gates used the default profile; family/final/sweep gates use
test optimization 1, retaining assertions, overflow checks and every domain:

~~~sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --no-fail-fast
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml -- --ignored --nocapture
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp cargo fmt --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml -- --check
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp cargo clippy --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --all-targets -- -D warnings
~~~

RED/GREEN filters used the same prefix/manifest with --test metadata,
--test parser, --test r1_widths, --test cli_xam and r13_, adding --no-fail-fast
for the aggregate. The schema family used --test metadata r13_generated;
the final generated count gate used --test metadata --test r1_widths
r13_generated -- --nocapture. The post-style addendum gate used --test identity
r13_spec_limitation. Both Python generators ran headlessly with repository TMPDIR.

Logs: ISA repo .tmp/r13-{before,witness-red-all,witness-green,witness-full,
family-red,family-green,family-full,families-final,width-family,final,
final-clippy,spec-limitation,sweeps}.log. No spec/conformance/include/Python seed
edit, dependency, foreign test execution, Beads export, push or Dolt sync.
All commits use explicit owned path lists and BEADS_HOOK_TIMEOUT=20.

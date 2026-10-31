# Editor response: council round 16

The sole S0 blocker, **R16-B01**, is resolved; none declined or partial.
Sole implementation sub-agent; repository instructions, binding round-7–15
requirements and the full round-16 report/JSON read. Implementation commit
**8f4d6b7**; final lint/progress/evidence commit **e12e469**.
Only owned interpreter paths and this response are committed; no push.
Concurrent spec/conformance work is untouched.

## Design

1. `src/metadata_schema.rs::TABLE` remains the sole recursive structural
   admission grammar, **163 named productions**, each citing its clause.
2. `schedule_bindings` now declares `"expansion?":"expansion"`. The comment
   and clause annotation explicitly cite §17 item 7: omission binds ref at
   the object's ISA version, which is 0.8 for every object this parser admits.
3. A present descriptor still uses the unchanged closed §11 name/version/hash
   production. Absence is distinct from null or an incomplete descriptor.
4. `metadata.rs::Object::schedule_expansion` is the binding-consumer API:
   it returns `ScheduleExpansion { name: "ref", version: "0.8", hash: None }`
   for omission, preserves every supplied descriptor field when present,
   and returns None when there is no schedule. It uses TABLE for descriptor
   validation. There was no earlier expansion-execution consumer to repair;
   this API supplies the default for later schedule execution.
5. Resolution never rewrites object text or canonical schedule JSON and never
   invents a descriptor hash. Object/interface/closure identity and the
   supplied schedule's own §9.2 hash are preserved independently.
6. README's metadata Design discussion records this default and the new family
   counts. No new spec reading, amendment, narrowing or execution waiver is
   required. Target/schedule execution and numerical/channel interval
   verification remain staged; existing Unverified boundaries are unchanged.

## R16-B01 — resolved

Determining clause: §17 item 7 explicitly says a schedule naming no expansion
binds ref at the ISA's own version. It supplies the omitted-field case in
§11's schedule sketch. The repaired production and default consumer encode
that exception without weakening validation of present descriptors.

RED tests:

- `metadata_schema::tests::r16_schedule_bindings_grammar_admits_omitted_expansion`
  supplies literal bindings independently of TABLE samples/generation.
- `metadata::r16_default_expansion_preserves_witness_and_identity`
- `metadata::r16_schedule_consumer_resolves_default_and_preserves_explicit_binding`
- `cli_xam::r16_cli_default_expansion_check_hash_preserves_pins`

The combined RED run was **2 passed / 4 failed**, all four failures ending in
**missing field expansion** under §9.1/§11. The two malformed-present controls
already passed and are retained. Separately, the existing
`metadata::schedule_requires_typed_bindings_selectors_and_unique_maxima`
was changed from omission rejection to successful parse/check_hashes; it
failed **0 passed / 1 failed** before the production fix with the same
missing-field diagnostic. Its other negative assertions remain unchanged.

GREEN: **6 passed / 0 failed** for the new regressions; the corrected existing
schedule test passes in the family gate and both full-suite runs. CLI check
and hash admit the exact copied witness; check returns status ok, and both
retain the object pin, with interface/closure pins checked in hash output.
Library admission/check_hashes checks all three identity pins and both raw
object text and canonical schedule bytes. No synthetic descriptor, rehash,
run input or execution budget was introduced; the supplied [3,0,0,0] stays.

Six malformed present descriptors reject: null, empty object, bad hash,
missing hash, non-string version, and an extra field. Library admission and
CLI check/hash all require §9.1/§11 diagnostics; CLI negatives require exit 1
and empty stdout. Consumer controls resolve omission to ref/0.8 with no
supplied hash, preserve explicit ref/0.8 and vendor/2026.10 with their hash,
leave the canonical schedule hash unchanged, and return None for no schedule.

Fix: `metadata_schema.rs::TABLE(schedule_bindings)`,
`metadata.rs::Object::schedule_expansion`, exported `ScheduleExpansion` in
`lib.rs`. Implementation commit **8f4d6b7**. Final lint fixes use constant
aliases instead of unused shared-helper imports, and `expect_err` instead of
`.err().expect()`; neither changes a test input, assertion or behavior.

## Independent witness pins

The exact copy is `tests/support/r16-default-expansion.bisa`, outside the
valid fixture inventory. It matches the read-only reviewer original
byte-for-byte; file SHA-256:
`8344b91cdba546f8bf29db14c4c239011cd088660863e015727c76ea8df25bfa`.

`tests/support/r16_pins.py` independently reconstructs the omission from the
R13 schedule-cleanupFalse record, serializes only the schedule line canonically,
and compares the entire result to the copied file. It reuses R15's literal
261-byte LF-terminated inc8 envelope, Python json/hashlib and §9.2 section
exclusion derivation, without interpreter identity or a rehash helper. The
inc8 envelope has no references, so closure=interface. Existing R15 witness
file digests are independently rechecked too.

| Identity | Unchanged SHA-256 |
|---|---|
| inc8 interface and closure | 8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93 |
| Object, excluding schedule and its boundary LF | 8093da48bad04d6564325faf01ea1304a84b5ac5a120e685eee9951f5daec5d9 |
| Supplied canonical schedule, without line terminator | d5ba44577e812d158265a048d8e4f8843b5a472cad120f47a893c6bbc9dfb99d |

## Generated family

The retained test is
`metadata_schema::tests::r14_generated_metadata_schema_type_family`, in
`src/metadata_schema/tests.rs`; its observed regenerated counts are:

| Coverage | R15 | R16 |
|---|---:|---:|
| Declarative productions, all exercised | 163 | **163** |
| Generated positive controls/substitutions | 9,806 | **9,862** |
| Unique generated typed paths | 1,653 | **1,653** |
| Grammar-derived negative mutations | 26,545 | **26,685** |
| Those negatives through BOTH admission validators (subset) | 1,992 | **1,992** |
| Independent pinned object exemplars | 26 | **26** |
| Full-parse typed path visits | 2,393 | **2,393** |
| Full-parse negative mutations | 17,173 | **17,173** |
| Total negative cases, without recounting dual assertions | 43,718 | **43,858** |

The optional field adds generated positive omission forms. Walking the added
forms adds grammar mutation cases; required fields and present descriptors
remain strict. These counts establish TABLE enforcement; the direct literal
omission test checks §17 item 7 independently of the generation.

`r1_widths::r13_generated_derived_width_extremes_family` still reproduces
**507 cases: 159 constructed, 286 cited layout rejections, 62 cited resource
diagnostics**. The reviewer's nonblocking independent expected-classification
suggestion remains deferred. These huge-width cases include allocator/storage
failure as well as integer representability; an independent host-allocation
model is beyond this short binding repair. No width domain, literal boundary
control or production behavior changed. This is not needed for R16-B01 and
the reviewer explicitly allows it as follow-up.

## Retained checks and observed verification

All round 4–16 tests remain, replacing only the contradictory missing-expansion
rejection as requested. R15's direct two-point translation-frame and exact
numeric epsilon controls remain green, alongside R14 signed-string Y endpoints,
row/component negatives, singleton X Evidence and 2^128 admission. Prior
identity/envelope pins, arbitrary widths, exact recursive tuple/prefix reversal,
first-fault/budget boundaries, borrow-frame/return obligations, non-permutation
negatives, zero-trip/reallocated lifetimes, opaque inverse histories, changed
input/token/control negatives, Born support, sidecars and the dotted-path
limitation are retained. No RNG distribution, seed or sweep domain changed.

| Gate | Observed result |
|---|---|
| Baseline, summed over all binaries | **384 passed / 0 failed / 5 ignored**, 20.67 s runner sum |
| Combined R16 RED | **2 passed / 4 failed**, 0.02 s runner sum |
| Corrected existing test, RED | **0 passed / 1 failed**, 0.00 s |
| Combined R16 GREEN | **6 passed / 0 failed**, 0.02 s runner sum |
| Family/metadata/width gate | **35 passed / 0 failed**, 5.30 s runner sum |
| First full suite | **390 passed / 0 failed / 5 ignored**, 19.88 s runner sum |
| Final full suite after lint-only fixes | **390 passed / 0 failed / 5 ignored**, 21.44 s runner sum; 11 unit + 379 integration, 0 doctests |
| fmt and fmt --check | pass |
| Strict all-target Clippy, after lint-only fixes | pass, 0 warnings |
| Independent pin script / copied witness | pass / byte-identical copy |

All **5 ignored sweeps passed / 0 failed**, **105.08 s** runner sum across
all binaries (105.07 s in the five sweep binaries).

| Sweep | Full domain | Observed result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 operand pairs × 4 = 262,144 executions | 1/0; 0.55 s |
| 17-wire borrow | 131,072 entries | 1/0; 3.29 s |
| checked borrow certificate | 262,144 elements, subject and expected program | 1/0; 16.45 s |
| §6 differential | 200,000 programs: 70,931 accepted (all clean), 129,069 rejected | 1/0; 82.21 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 1/0; 2.57 s |

Every cargo call used the headless prefix below, the absolute manifest, and
an interpreter-only test selection. Test optimization 1 retains assertions,
overflow checks and complete sweep domains. Exact commands:

~~~sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --no-fail-fast
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --lib --test metadata --test cli_xam r16_ --no-fail-fast -- --nocapture
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --test metadata schedule_requires_typed_bindings_selectors_and_unique_maxima -- --nocapture
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --lib --test r1_widths --test metadata -- --nocapture
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --no-fail-fast -- --ignored --nocapture
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp cargo fmt --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp cargo fmt --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml -- --check
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp cargo clippy --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --all-targets -- -D warnings
python3 -B /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/tests/support/r16_pins.py
cmp /home/tobias/Projects/bennett-workspace/bennett-isa-spec-review/.tmp/r16-default-expansion.bisa /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/tests/support/r16-default-expansion.bisa
~~~

Logs: interpreter `.tmp/r16-{before,red,red-existing,green,family,full,final,clippy,clippy-final,sweeps}.log`.
Production and test behavior were unchanged after implementation; only the two
lint-only test-helper adjustments followed the first suite/sweep start.
Final full suite and Clippy were rerun after those adjustments. Owned diffs
pass --check. No spec/conformance/include/Python-seed edits or foreign test
execution, dependency, Beads mutation/export, push or Dolt sync. Commits use
explicit owned paths, absolute git -C and BEADS_HOOK_TIMEOUT=20.

# Editor response: council round 15

Both S0 blockers, **R15-B01 and R15-B02**, are resolved; none declined or
partial. Sole implementation sub-agent; binding round-7–14 briefs, repository
instructions and the full codex report/JSON read. Implementation commit
**9283db7**; final progress/evidence commit **738c67f**. Only owned interpreter paths and this response are
committed; no push. Concurrent spec/conformance work is untouched.

## Design

1. `src/metadata_schema.rs::TABLE` remains the single recursive structural
   admission grammar, now **163 named productions**, each citing its clause.
2. `translation_witness.frame` uses the new `translation_frame` production:
   `Shape::Tuple(&["point", "point"])`, implementing §12.1's first/last points.
   V and W retain `Array(point)`, including empty and longer arrays.
3. `certificate.rs::validate_claim` consumes the grammar and no longer repeats
   frame cardinality. Claim-index and other contextual checks remain there.
4. Oracle schedule exactness is a semantic value constraint (§§4.4, 5.2, 11).
   After decimal validation, zero means every mantissa digit is zero; the
   decimal scale/exponent cannot change that decision. No binary64 conversion
   is used for the zero decision, and no exponent/precision cap is added.
5. README's metadata Design paragraph records that structural/semantic split
   and the regenerated counts. The family tests enforcement of TABLE, while
   the independent literal cardinality/value controls test the cited rules.
6. Identity, execution, borrowed-frame obligations and interval verification
   are unchanged. Numerical/channel claims remain explicitly Unverified at
   the existing staged boundary; no new spec reading or waiver is needed.

## R15-B01 — resolved

Determining clause: §12.1 explicitly encodes `frame: [first point, last point]`.

RED test: `metadata_schema::tests::r15_translation_frame_is_first_and_last_point_tuple`
constructs literal all-zero-hash points independently of TABLE, samples and
baseline. It failed with accepted lengths **[0, 1, 2, 3]**, expected **[2]**.
GREEN accepts only length 2; lengths 0, 1 and 3 reject. A separate loop retains
variable-length point-array acceptance for V/W's production.

Full-admission controls:

- `metadata::r15_translation_frame_cardinality_is_checked_at_admission` checks
  all four lengths on the reviewer object. Only length 2 parses/checks and
  preserves the object pin; the other lengths reject with §12.1.
- `cli_xam::r15_cli_rejects_empty_translation_frame` replays the exact copied
  reviewer negative under check/hash: exit 1, empty stdout, §12.1 diagnostic.

These two full-admission regressions already passed before the fix because
the old semantic guard existed. They remain green after its removal. The
cardinality-only hand-written guard expresses nothing beyond Tuple, so it is
removed rather than retained as a redundant invariant. Fix: TABLE's
`translation_witness -> translation_frame`; `certificate.rs::validate_claim`.
Commit **9283db7**.

## R15-B02 — resolved

Determining clauses: §5.2 requires numeric ε=0, §4.4 requires exact oracle
gates, and §11 carries accuracy as decimal text. JCS does not canonicalize
the contents of JSON strings.

RED tests:

- `metadata::r15_oracle_schedule_epsilon_uses_exact_decimal_value`
- `cli_xam::r15_cli_oracle_schedule_epsilon_is_exact_by_value`

Both failed on **"0.0"**, with **§11: §5.2: oracle schedules must be exact**.
They use the independently pinned `schedule-cleanupFalse` record, altering
only bindings.epsilon and serializing its schedule line canonically. Its
"0.0" form equals the exact reviewer file byte-for-byte.

GREEN library parse/check and CLI check/hash admit **0, "0", "0.0", "0.000"**
with unchanged literal interface/closure/object pins. They reject nonzero
**"0.1", "0.000001", exact decimal-string 10^-400 and numeric 1e-300** with
§5.2. The string spelling **"1e-300"** is outside the existing decimal-string
grammar and rejects with §9.1; the non-exponent 10^-400 string is admitted by
the grammar and rejected as nonzero semantically. Thus underflow cannot
silently make a tiny positive value exact. Every negative check/hash call
requires exit 1 and empty stdout.

Fix: `metadata.rs::Object::validate_schedule_details` inspects validated
mantissa digits instead of comparing canonical JSON/string spellings.
Commit **9283db7**.

## Independent witness pins

Exact copies live in `tests/support/r15-witnesses/`, outside the valid fixture
inventory. `tests/support/r15_pins.py` independently constructs the literal
261-byte LF-terminated inc8 envelope with Python json/hashlib, without the
interpreter or a rehash helper. Inc8 has no references, so closure=interface.
Excluding schedule/certificate with its leading boundary LF reproduces the
object hash in both files. Both copies match the read-only originals.

| Witness | File SHA-256 |
|---|---|
| r15-frame-empty.bisa | bf88ce59f34ccca4a5e930028d51a8daa0ef9d1c5f78027a85893a19df70b27d |
| r15-epsilon-0.0.bisa | 1431a1dbbe4991d170e3848995026610ebbffce08e7279ee05a68880f06c82df |

Both use interface=closure
`8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93`
and object
`8093da48bad04d6564325faf01ea1304a84b5ac5a120e685eee9951f5daec5d9`.
No supplied pin or implementation identity changed.

## Generated family

The retained test name is
`metadata_schema::tests::r14_generated_metadata_schema_type_family`, in
`src/metadata_schema/tests.rs`; its observed R15 counts are:

| Coverage | R14 | R15 |
|---|---:|---:|
| Declarative productions, all exercised | 162 | **163** |
| Generated positive controls/substitutions | 9,754 | **9,806** |
| Unique generated typed paths | 1,632 | **1,653** |
| Grammar-derived negative mutations | 26,354 | **26,545** |
| Those negatives through BOTH admission validators (subset) | 1,992 | **1,992** |
| Independent pinned object exemplars | 26 | **26** |
| Full-parse typed path visits | 2,393 | **2,393** |
| Full-parse negative mutations | 17,173 | **17,173** |
| Total negative cases, without recounting dual assertions | 43,527 | **43,718** |

The extra Tuple gives generated baselines two points, instead of the old
one-point frame. The family asserts that TABLE is enforced, not that TABLE
is an independent oracle for the spec. The new clause-derived test catches
this cardinality defect independently of that generation.

The retained R13 width family reproduces **507 cases: 159 constructed,
286 cited layout rejections, 62 cited resource diagnostics**. The nonblocking
independent width-outcome classification follow-up remains deferred, as the
reviewer permits; this short round changes no width behavior or assertions.

## Retained checks and observed verification

All round 4–15 tests are retained. That includes R14's signed-string Y
endpoints, table-row/component negatives, singleton X Evidence control and
2^128 admission; prior identity/envelope pins, arbitrary widths, exact
recursive tuple/prefix reversal, first-fault/budget boundaries, borrowed
frame/return obligations, non-permutation negatives, zero-trip/reallocated
lifetimes, opaque inverse histories, changed input/token/control negatives,
Born support, fixture sidecars and the recorded dotted-path limitation.
No RNG distribution, seed, sweep domain or earlier assertion changed.

| Gate | Observed result |
|---|---|
| Baseline, summed over all binaries | **379 passed / 0 failed / 5 ignored**, 20.67 s runner sum |
| Combined R15 RED | **2 passed / 3 failed**; two pre-existing full-admission safeguards pass |
| Combined R15 GREEN | **5 passed / 0 failed**, 0.06 s runner sum |
| First full suite after fixes | **384 passed / 0 failed / 5 ignored**, 20.60 s runner sum |
| Final full suite | **384 passed / 0 failed / 5 ignored**, 19.65 s runner sum; 10 unit + 374 integration, 0 doctests |
| Family and width gate | **6 passed / 0 failed**; 5.19 s schema + 0.07 s widths |
| fmt --check | pass |
| Strict all-target Clippy | pass, 0 warnings |
| Independent pin script / copied witnesses | pass / two byte-identical copies |

All **5 ignored sweeps passed / 0 failed**, **97.21 s** summed runner time.

| Sweep | Full domain | Observed result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 operand pairs × 4 = 262,144 executions | 1/0; 0.52 s |
| 17-wire borrow | 131,072 entries | 1/0; 2.74 s |
| checked borrow certificate | 262,144 elements, subject and expected program | 1/0; 14.99 s |
| §6 differential | 200,000 programs: 70,931 accepted (all clean), 129,069 rejected | 1/0; 76.54 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 1/0; 2.42 s |

Every cargo call used the headless prefix below, absolute manifest and offline
mode for tests/Clippy. Test optimization 1 retains assertions, overflow checks
and the complete sweep domains:

~~~sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --no-fail-fast
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --lib --test metadata --test cli_xam r15_ --no-fail-fast -- --nocapture
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --lib --test r1_widths r1 -- --nocapture
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --no-fail-fast -- --ignored --nocapture
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp cargo fmt --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp cargo fmt --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml -- --check
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp cargo clippy --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --all-targets -- -D warnings
~~~

Logs: interpreter `.tmp/r15-{before,red,green,full,final,family,clippy,sweeps}.log`.
No production/test edits after the final suite or sweep start. Owned diffs
pass --check. No spec/conformance/include/Python-seed edits or foreign test
execution, dependency, Beads mutation/export, push or Dolt sync. Commits use
explicit owned paths, absolute git -C and BEADS_HOOK_TIMEOUT=20.

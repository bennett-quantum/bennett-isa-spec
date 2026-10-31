# Editor response: council round 14

Both S0 blockers, **R14-B01 and R14-B02**, are resolved; none declined or
partial. Sole implementation sub-agent; binding round-7–14 briefs, repository
instructions and the full codex report/JSON read. Implementation commit
**7f834d0**; final evidence/correction commit **198c08a**. Only owned interpreter paths and this response are committed;
no push. Concurrent spec/conformance work is untouched.

## Design

1. `src/metadata_schema.rs::TABLE` is the single recursive admission grammar:
   **162 named productions**, each with the determining clause.
2. It covers all seven JSON metadata sections: symbols, imports, devices,
   layout, schedule, certificate and coverage; every signature alternative;
   and the textual data section's NAME/HEX projection (§9.1/§16.2).
3. Productions cover §9.1 sections/signatures/integers, §9.2 hashes/points,
   §9.4 frame descriptors, §8.1 expressions, §11 schedules, §12.1 claims and
   interval encoding, §12.2 domains/tuple enumeration and §12.3 composition,
   as well as §13 coverage. Optional keys, closed objects, arrays, maps, tuples,
   enum alternatives and recursive expression operands are explicit.
4. `metadata_types::Object::validate_metadata_types` validates the metadata
   root before identities. `validate_certificate_details` and `validate_claim`
   consume the same table. Signature admission and data-line parsing also use
   it. Metadata hash/text/decimal helpers delegate to table productions.
5. Previous hand-written field/type schemas in object.rs, validation.rs,
   metadata.rs and certificate.rs are removed; no second schema remains.
6. Semantic checks remain where context is available: object.rs resolves
   images and layout intervals; metadata.rs resolves schedule/coverage points,
   checks payload agreement and entry overlap; certificate.rs checks domain
   names/ranges, claim indices, expression arity/environments, live ranges and
   certificate bindings; validation.rs checks device model compatibility.
   These checks cite their determining clauses.
7. Table components are arbitrary-size canonical §9.1 unsigned integers.
   Their truth, cardinality, widths and complete-result comparisons remain
   verifier obligations, not load-time execution.
8. Y entries are four signed finite decimal strings. Ordinary nonnegative
   decimals retain number/string forms; §8.1 expressions retain numeric and
   closed recursive AST forms. No ISA exponent or integer-width cap is added.
9. The derived family walks generated grammar examples and real pinned objects,
   recursively including array elements and object fields. Wrong-kind
   expectations derive from Shape independently of observed validator results.
10. Every production, enum/union alternative and optional absence is generated;
    positive substitutions exercise each accepted alternative at visited paths.
    Table rows/components are populated, including a component above u128.
11. The R13 path-specific union and family are replaced. The historical local
    round-13 response has a correction withdrawing its completeness claim.
12. Numerical/channel execution remains explicitly Unverified at the staged
    interval-verifier boundary. Scope and the recorded §9.2 dotted-path
    limitation are unchanged; no new spec reading or waiver is needed.

## R14-B01 — resolved

Determining clause: §12.1 explicitly encodes Y entries as decimal strings.
The exact reviewer object is copied byte-for-byte as
`tests/support/r14-witnesses/r14-Y-number.bisa`, full-file SHA-256
`578c913bb276d1446e2e038af0ab0a77017d9abe5ec351e3e20b69c307068e11`.

RED tests:

- `metadata::r14_y_endpoints_require_decimal_strings`
- `cli_xam::r14_cli_rejects_numeric_y_endpoints`

Both failed: parse admitted numeric endpoints; CLI check returned exit 0,
status ok and the reviewer's object hash. GREEN rejects the exact object and
each of four endpoint positions independently changed to number, bool, null,
array or object. Check and hash exit 1, empty stdout, §9.1/§12.1 diagnostic.
The otherwise identical four-string zero control still parses/checks, retains
its literal envelope/interface/closure/object pins and reports Unverified on
explicit channel verification.

Fix: TABLE's `channel -> dual -> dual_witness -> Y -> interval -> endpoint`;
`endpoint` is `SignedDecimalString`. Both admission paths consume it; the
previous Number|String hand-written check and test union are gone. Commit
**7f834d0**. No restriction is applied to ordinary decimal fields or numeric
§8.1 expressions.

## R14-B02 — resolved

Determining clauses: §§9.1, 12.1–12.2, SPEC-QUESTIONS 21's existing full-tuple
encoding. The smaller singleton X action and all adjacent reviewer negatives
are exact copies in `tests/support/r14-witnesses/`.

RED tests:

- `metadata::r14_table_rows_and_components_are_typed_at_admission`
- `cli_xam::r14_cli_rejects_malformed_table_rows_and_cells`

Both failed: malformed rows/components were admitted; check returned exit 0
and the pinned action object hash. GREEN rejects the exact witnesses and rows
changed to number/string/bool/null/object, plus bool/null/array/object,
fractional/negative integers and malformed/noncanonical integer strings as
components. Parse fails before any verification; check/hash exit 1, empty
stdout, cited §9.1/§12 diagnostic. The original inc8 table-bool witness also
rejects. The populated [[1]] singleton control still parses, checks and
verifies as `Evidence { elements: 1 }`: X maps t=0 to t=1.
A canonical decimal-string component 2^128 is admitted without host narrowing.

Fix: TABLE's `exhaustive -> expected -> table_expected -> table -> uints -> uint`;
`uint` uses uint_text, not a bounded integer. `r13_pins.py` now populates the
original table branch and adds the independently pinned singleton action.
Both admission validators use the table. Commit **7f834d0**.

## Independent witness pins

`tests/support/r14_pins.py` assembles literal LF-terminated envelopes with
Python json/hashlib, without interpreter or rehash helpers. Reference-free
singleton SCCs have closure=interface; actions exclude layout; inc8 has no
layout. Removing the certificate section, including its leading boundary LF,
gives the whole-object bytes below. All supplied pins match unchanged.
Library controls also compare complete envelopes against the independent
records. All six copied files match the read-only originals byte-for-byte.

| Witness group | Interface = closure | Object hash |
|---|---|---|
| Y-number / original inc8 table-bool | 8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93 | 8093da48bad04d6564325faf01ea1304a84b5ac5a120e685eee9951f5daec5d9 |
| Singleton action table-positive / row-bool / cell-bool / cell-object | 2fcd5b3d60b42ba6092447586a2157bf44ba3a6479ab2eb319bb67665feab8be | ddc5d6729a98484acc2d5e8feb585c155799293bd36f31fd96cbc510f77666f0 |

Three corpora regenerate byte-identically: 8 R13 witness records, 26 schema
exemplars and 6 R14 pin records. The first script draft assumed table-bool
was action-based; corrected it to use its actual inc8 symbol before asserting
the pins. No supplied pin or implementation identity changed.

## Generated family

`metadata_schema::tests::r14_generated_metadata_schema_type_family`, in
`src/metadata_schema/tests.rs`, replaces the R13 family rather than duplicating
it. Counts observed with --nocapture:

| Coverage | Count |
|---|---:|
| Declarative productions, all exercised | 162 |
| Generated positive controls/substitutions | 9,754 |
| Unique generated typed paths | 1,632 |
| Grammar-derived negative mutations | 26,354 |
| Those negatives through BOTH admission validators | 1,992 (subset) |
| Independent pinned object exemplars | 26 |
| Full-parse typed path visits | 2,393 |
| Full-parse negative mutations | 17,173 |
| Total negative cases, without recounting dual assertions | **43,527** |

Wrong JSON kinds come from the declared Shape and union alternatives;
integer negatives include -1, 1.5, unsafe raw 2^53/2^64 spellings and invalid
integer strings. Decimal negatives include nonfinite names, malformed signs,
empty/fractional spellings and disallowed string exponents. Positive examples
cover canonical large strings through 2^128, number/string decimals, every
union and enum alternative, empty/nonempty containers, optional absence and
recursive expressions. Both admission validators and direct certificate
admission reject generated malformed metadata; full parse negatives require
§9.1 before caller-side identities or execution.

The retained R13 width family reproduces **507 cases: 159 constructed,
286 cited layout rejections, 62 cited resource diagnostics**. Nonblocking S1's
stronger independently classified width-outcome assertions are deferred as the
reviewer permits; this round changes no width behavior or existing assertion.

## Retained checks and observed verification

All round 4–14 positives/negatives remain, except the expressly replaced R13
family. This includes service versions/block images, maximum-exponent layout
admission and B=0, exact recursive tuple/prefix reversal, first-fault and
budget boundaries, borrow frame/return equations, non-permutation negatives,
zero-trip/reallocated lifetimes, opaque inverse histories and changed
input/token/control negatives, exact Born support, fixture sidecars and the
recorded cross-object dotted-path limitation. RNGs/seeds are unchanged.

| Gate | Observed result |
|---|---|
| Baseline, summed over all binaries | **375 passed / 0 failed / 5 ignored**, 18.32 s runner sum |
| Combined exact witness RED | **0 passed / 4 failed** across metadata and CLI |
| Combined witness GREEN | **4 passed / 0 failed** |
| Replaced-family full suite | **379 passed / 0 failed / 5 ignored**, 21.61 s |
| Final post-style full suite | **379 passed / 0 failed / 5 ignored**, 22.84 s; 9 unit + 370 integration, 0 doctests |
| Final family/width gate | **5 passed / 0 failed** (two families + three retained width regressions); 5.21 s schema, 0.07 s widths |
| fmt --check | pass |
| Strict all-target Clippy | pass, 0 warnings |
| Independent regeneration / copied witnesses | three corpora byte-identical; six exact copies |

The initial full run failed only the valid-fixture audit after malformed
reviewer files were placed in that inventory. They were moved to support;
the audit and all original fixture assertions remain intact. Clippy found one
redundant unit expression; removed it, reran Clippy and the full suite. No
production or test changes followed the final post-style suite.

All **5 ignored sweeps passed / 0 failed**, **100.56 s** summed runner time.

| Sweep | Full domain | Observed result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 operand pairs × 4 = 262,144 executions | 1/0; 0.52 s |
| 17-wire borrow | 131,072 entries | 1/0; 2.71 s |
| checked borrow certificate | 262,144 elements, subject and expected program | 1/0; 14.81 s |
| §6 differential | 200,000 programs: 70,931 accepted (all clean), 129,069 rejected | 1/0; 80.11 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 1/0; 2.41 s |

A supplementary exact-witness RED replay builds the reviewed e3ae1af sources
using headless rustc in reference-interpreter/.tmp (not a checkout; current
sources are untouched). Two library rejection assertions on the exact
Y-number and row-bool objects fail (0/2), and all ten check/hash calls on the
five exact negative files wrongly exit 0/status ok. Replaying those same ten
CLI calls against the current binary gives exit 1, empty stdout and §9.1/§12
for every case; the singleton positive still checks with its literal pin.
This supplements the original pre-fix cargo RED rather than changing its
reported count. Logs: .tmp/r14-exact-{red-replay,cli-green}.log.


Every cargo call used this prefix, absolute manifest and offline mode;
test optimization 1 retains assertions, overflow checks and complete domains:

~~~sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --no-fail-fast
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --lib --test r1_widths r1 -- --nocapture
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml -- --ignored --nocapture
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp cargo fmt --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml -- --check
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp cargo clippy --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --all-targets -- -D warnings
~~~

RED/GREEN filters used the same prefix/manifest with --test metadata --test
cli_xam r14_ --no-fail-fast. Python generators also ran headlessly with the
repository TMPDIR. Logs: ISA repo .tmp/r14-{before,red,green,schema-full,
family,full,final,final2,families-final,clippy,clippy-final,sweeps}.log.
No spec/conformance/include/Python-seed edits or foreign test execution,
dependency, Beads mutation/export, push or Dolt sync. Commits use explicit
owned paths, absolute git -C and BEADS_HOOK_TIMEOUT=20.

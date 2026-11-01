# Current state — 2026-11-01 13:35:48 UTC

Pass XI handled the two rung-3 consumer-named beads independently.

- b-isa-spec-221: classification (c), no remaining ISA 0.9 defect. Accepted
  §0.2 K5 explicitly names 221; §4.1/§4.9 specify typed CC/R fault C, the
  cause table, exact nonzero 64-bit codes (zero maps to 4), terminality and
  intact prefix. §7.4/§10/T4 specify faulted-run reversal. Closed with those
  citations. The consumer is correct that ISA 0.8 cannot supply this contract.
  Open **b-isa-spec-fuk** retains the actual producer/executor support and
  explicit pin/ABI/envelope/hash migration prerequisite; BD-B-ISA7 is NOT
  cleared. Host execution fuel is not dialect §17.6's source counter.
- b-isa-spec-04d: classification (b), implemented and closed after green gates.
  `--tape-out FILE` and `--replay-tape FILE` on run-cc, CC/R reverse and
  run-program save/consume canonical Tape[] journals with LF. Each successful
  effect's dev/op/args/bytes matches before its step commits; missing/mismatched
  entries roll back without cursor advance, unused entries fail at run/prefix
  end, and divergence exits 1 without JSON or a tape-output write. Entropy and
  checked host replies remain explicit inputs. Valid replay is byte-identical
  including results, console and reset. Journals retain subsequently uncalled
  effects; existing output JSON still reports the active tape. README documents
  the full contract. No new normative wording, MUST, K row or council fix round.

Gates (all exit 0):

| Pass | Reference Cargo | Conformance Cargo | Draft witnesses | Catalog |
|---|---|---|---|---|
| 221 | 491 passed, 0 failed, 5 ignored | 113 passed, 0 failed, 0 ignored | PASS | PASS, 219 recipes |
| 04d | 500 passed, 0 failed, 5 unchanged ignored | 113 passed, 0 failed, 0 ignored | PASS | PASS, 219 recipes |

Every gate unset DISPLAY, WAYLAND_DISPLAY, DBUS_SESSION_BUS_ADDRESS and BROWSER,
set TMPDIR to this repo's .tmp, and used offline Cargo where applicable. Logs:
`.tmp/pass-xi-{221,04d}-{reference,conformance,witness,catalog}.log`.
Nine added regressions: eight integration tests plus the transactional rollback/
retry unit test. Red evidence: `.tmp/pass-xi-04d-red.log` (five missing-option
failures); final targeted log `.tmp/pass-xi-04d-targeted.log`. Harness fixes:
additional heap for the synthetic host-call stub, and single steps for the
rollback prefix rather than treating run(3)'s fuel diagnostic as success.
Existing fixtures/regressions retained. Owned rustfmt/whitespace and strict
lib/bins Clippy pass. No all-target Clippy claim or new ignored-test claim.

Offline release build passes. The consumer's exact executor-control.bisa passes
saved replay, complete reset and corrupted entry-1 rejection on debug/release;
stdout matches byte for byte. Evidence: `.tmp/pass-xi-consumer-*` and
`.tmp/pass-xi-release-info.json`. Local release binary:
`reference-interpreter/target/release/bennett-isa-ref`, SHA-256
`166a25c6c22481aff71d437a236cb77ce92871726594e40a725cdd88e08c2792`.
The frozen sibling release binary/pins are untouched; use an explicit consumer
--reference override/update. No full musl hello-world/native/R/reset/replay rung-3
verdict is claimed, and its North Star §8 scope is unchanged. Interpreter admission
still targets ISA 0.8; no ISA 0.9 admission claim.

Frozen artifact audit (`.tmp/pass-xi-audit.log`):
- ISA 0.8 SHA-256: 90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e.
- ISA 0.9 SHA-256: 5ea0d18965e1fa0fa694fd46ce3ab3d64ee6fa98c731533c0cccc684bdb44d99.
- Catalog bytes SHA-256: 52dbc3f8f0c91d587fd933a43080385fd5284abcfec0ce84788229fcb15c53d8.
- Catalog envelope: 5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197.
- Exact-label diff SHA-256: 55cc2aa290957296e2b85749f82a07f1a44de61ebe65d0c5903b0cf493cfb211.
All bytes equal starting HEAD 6c3c1a4; the diff matches independent diff -u with
labels a/spec/Bennett-ISA-Spec.md and b/spec/Bennett-ISA-Spec-0.9.md.
K1–K80 unchanged; no K81, inventory edit or regenerated contract needed.

Own commits: c809fc0 (221 disposition/migration docs and Beads), fdf553e
(replay implementation/regressions/docs), 666cb6b (221 green gate record).
Concurrent unrelated beads-only commit 1a7f011 is preserved. These final records
and exported closure are committed next by explicit path list; final report gives
final HEAD. No push, remote/Dolt sync or sibling repository write. The orchestrator
retains session-worklog ownership. Preserve pre-existing untracked .tmp/ and all
logs; only ?? .tmp/ is expected after the final records commit.

# ISA reference interpreter: quality batch 1, review 3

**Verdict: CLEAR — no P0/P1.** No implementation files changed.

Reviewed head: `8056ecf3834955ec8a46f091b7d90b026dc207dd` (`job/quality-isaref`).
Batch inventory: `git log --reverse main..job/quality-isaref` and
`git diff main...job/quality-isaref`; this final review focuses on
`d3a338e..8056ecf` (`15c8c3a`, `fa8f276`, `8056ecf`) and rechecks the
P0/P1 scenarios from both earlier review documents against ISA 0.9 §§4.5,
12.3 K92.

The residual **P0-1 `b-isa-spec-lfbd` is fixed**. An independent stdin Rust
probe rebuilt the round-2 `run → addya → inner → qinplace` certificate,
retargeting the root bindings to `run` with `root_invocation(2, 1)`.
With the outer pair claim retained, its standalone verdict remains `Proof`
and the report independently contains `run: Unestablished`. Removing the
outer claims and entries preserves that root obligation with zero claim
reports; removing every region record also preserves it. In all three
variants, executing `addya` at tk=3,a=1 rejects the missing inner-owner
entry before effects and leaves the quantum state unchanged.

The original round-1 findings remain fixed: missing direct inventory and
below-minimum bound tuples reject; wrong g/g library reuse rejects through
both public verification APIs; valid g/ginv library coverage executes the
hand-derived result and reverses exactly. Focused tests also cover transitive
arguments, zero-trip inventory, disabled predicates and unavailable libraries.

`CertificateReport.qinplace_applicability` retains obligations independently
of records and claims. Unsupported roots, unchecked uniform arguments,
scheduled trees and host-resource failures insert their own closure keys;
unresolved outcomes merge last so a checked concrete context cannot discharge
a uniform obligation. Static owner outcomes and checked transitive owners
are retained too. Explicit borrow-frame references remain index-validated
and checked separately; their enforcement does not rely on a nonempty report.
The standalone `verify_claim` contract explicitly disclaims executable
applicability.

The report-shape change retains all claim fields and their closure-map /
claim-index order inside `.claims`; the probe checks both linked-library
records are present and ordered. The Rust API intentionally requires callers
to select `.claims`. No production CLI command serializes or consumes
`verify_certificate`; CLI/JSON formatting code is unchanged. Existing public
CLI partition and resource-output tests pass.

Verification used headless commands, private
`.tmp/rev-quality-isaref-r3/{tmp,target}`, `ulimit -v 8388608` and `ulimit -c 0`:

```sh
timeout 600 cargo test --offline -j3 --manifest-path reference-interpreter/Cargo.toml \
  --test isa09_qinplace --test isa09_claim_replay --test isa09_certificate \
  --test isa09_token_aliases -- --test-threads=4
timeout 600 cargo test --offline -j3 --manifest-path reference-interpreter/Cargo.toml \
  --test r1_certificates --test r1_qinplace --test r21_resources \
  --test r22_coverage -- --test-threads=4
timeout 60 .tmp/rev-quality-isaref-r3/tmp/review-probe
git diff --check main...job/quality-isaref
```

All exit 0: **95 tests passed, one unchanged ignore**; independent probe passed.
The probe was compiled with `timeout 120 rustc --edition=2021 -C opt-level=1`,
including the existing fixture helpers and linking freshly built review rlibs.
Reading `git show main:reference-interpreter/src/claims.rs` and the pre-fix
`qinplace09.rs` confirms the empty-report and uniform/scheduled regressions
would fail there: main has no independent applicability report, while the
round-2 code loses unresolved outcomes on an empty owner map and skips uniform
obligations. No branch switching, full crate suite, repository gate or sibling
gate was used. No tests or expectations were weakened.

P2+ nits: **one bead `b-isa-spec-bkqj`** consolidates existing dead dispatch,
tracked litter, stale `HANDOFF.md`, missing gate ratchet, production no-op
hooks and deferred function/coverage/runtime work; no new nit in this fix.
Checked every COMMON anti-slop rule; no new semantic violation or narrowing.
Batch-2 items remain assigned to batch 2 by the implementer brief.

Coverage: read closely the complete round-3 source/test diff, both verification
APIs, K92 inventory/traversal and outcome merges, certificate invocation/index
admission, prior P0/P1 scenarios and public report consumers; skimmed unchanged
pair-theorem and CLI plumbing and continuing structural debt. Other interpreter
subsystems and full conformance were not re-reviewed.

Provenance: implementer **gpt-6.1-sol**; reviewer **gpt-6.1-sol, fresh session, D9**.
CLEAR applies to this review scope; the required critical-batch council round
remains the orchestrator's next step.

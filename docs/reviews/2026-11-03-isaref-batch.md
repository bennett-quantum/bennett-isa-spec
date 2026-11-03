# Batch review: reference-interpreter, fb4aa11..6bfba04

Date: 2026-11-03. Reviewer: opus-rev-isaref, Claude Opus 5.5 (policy rule 2, opposite-model seat for Codex work).
Range: `fb4aa11..6bfba04` (HEAD at start), `reference-interpreter/` only, 24 commits, about 11.8k lines added. Reviewed from a
detached worktree of `6bfba04`; no tracked file edited. Probes: `.tmp/opus-rev-isaref/probe/`.

## Verdict: BLOCK (1 P0, 1 P1)

| Bead | Pri | Location | Failure scenario | Run |
|---|---|---|---|---|
| b-isa-spec-opnq | P0 | `src/claims/replay09.rs:330-494` (verdict at :465), routed from `claims.rs:672`, `:875-879`; admission `certificate.rs:719-760` | A table claim whose `subject.schedule_hash` names a schedule. K13 says its left side is the realised operation, but replay runs only the reference semantics and returns `Proof`. `expected:"oracle"` in the same case returns `Unverified`. This is a silent fallback. A dangling `schedule_hash` (64 × `ab`) is also admitted and also gets `Proof`. | `pair-distinct.bisa` (accepted-k91-09) with `subject.schedule_hash` inserted into claim 0, using the real hash and then a bogus one: `check` rc=0 and `verify_certificate()` = `Ok([Proof])` in both cases (`probe/src/main.rs`) |
| b-isa-spec-7cvg | P1 | `src/claims/replay09.rs:337-342`, routed from `claims.rs:672-674`, `:875-879` | Every ISA 0.9 `expected:"inverse_pair"` claim makes `verify_claim` and `verify_certificate` return an error, even when a `verification.inverse_pairs` entry references it. So no certificate that carries a K4 inverse pair can verify. This is a narrowing of the K4/K92 checker obligation, and no test covers it. | the same fixture with claim 0 set to `expected:"inverse_pair"` plus an `inverse_pairs` entry (g = ginv = `@identity`): `check` rc=0, then `verify_certificate()` = `Err(§12.1 K4: inverse-pair claims require their entry binding)` (`probe/src/bin/p3.rs`, `ENTRY=1`) |

Nits, one bead: b-isa-spec-x6to (P3, label `nit`). Highlights:
- The crate suite is red at `6bfba04`: 639 passed, 2 failed, 5 ignored.
  - `isa09_qinplace` (published §16.3 Example 7, the known fuk.7 gap, committed red at 961dc81).
  - `r21_quantum_cardinality_and_lengths` expects `§4.3`, but `registers.rs:188` now cites `§3.3`.
- K70 borrowed-position admission. The printed witness gives D = 2^25; K70 gives 2^24. The spec decision is pending in b-isa-spec-pe1, so this is not counted against the code.

Earlier findings rechecked where cheap:
- 7fd (JCS key-array layout path) appears fixed: `published_09_layout_hashes_are_admitted_without_rehashing` passes.
- iqy (root names data) is fixed: it now returns an error, not a panic.
- iwx (`expected:oracle` without `schedule_hash`) is fixed at `certificate.rs:731-738`.
- The other earlier beads were not rechecked.

## Coverage

Read closely:
- `claims/replay09.rs`, `claims/images09.rs`, and the `claims.rs` and `certificate.rs` diffs (K13/K85 replay, coverage, inherited controls, sampling order against `enumerate`).
- `tokens.rs` (K91 aliases, token validity).
- The `classical.rs` diff and `classical/registers09.rs` (K1/K2/K3 mint and revocation, qreg preconditions, fault 2 on host allocation).
- The `machine.rs` and `oracle.rs` diffs and `oracle/program.rs` (§3.7 fixed-binding instances and index order).
- The `identity.rs`, `object.rs`, `schema.rs` and `metadata_schema.rs` diffs.
- `validation/cleanup.rs`.

Skimmed:
- `main.rs` (option plumbing).
- `metadata/schedule_domains.rs`, apart from the bound-run match.
- `ref_library.rs` and `ref_library/tokens.rs`.
- `source_constructor*`: only its tests' oracle independence was checked; they compare against `run_oracle` or closed-form values.
- `metadata/source_paths.rs`, `metadata/source_events.rs`, `validation/registers09.rs` and `action.rs`.
- The test diffs: scanned for ignores, removed asserts and rehash use. Rehash is used only to build fixtures. The published-hash lock exists.

Not read: K89 `borrow_frame` replay (not in the 0.9 `expected` schema here), K90, and the conformance crate.

Commands:
- `timeout 1500 cargo test -j4 --no-fail-fast --tests -- --test-threads=4` in the worktree crate: exit 101, with the 2 failures above.
- The probe binaries.
- `bennett-isa-ref check` on the mutated objects.

## Provenance / R14

All 24 commits are Codex runner commits (sol-ref09). Their Claude trailers do not override the Codex marker. This review is the rule-2 opposite-model seat. The verdict is BLOCK, so the rule-5 Muse seat is not due yet. It falls on the round that claims CLEAR, pending advisor confirmation. No cross-repo beads were needed.

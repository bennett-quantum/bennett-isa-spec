# ISA 0.9 main implementation review, round 1

**Verdict: BLOCK — P0-1 and P1-2 at the reviewed head.**

Range: `origin/main..be109bd`, base
`9825c42ad5c95cce9cd1419dcbcae0bfffab6e16`, reviewed head
`be109bd27889ea91e3c0ad78a30e5e4d71d3d0d3`.
This is the sol-ref09 implementation range, not `main...job/quality-isaref`.
Tests used an archive of that head under
`.tmp/rev-isaspec-main-r1/tmp/snapshot`; no checkout was switched or created.
The delivery branch started at `e9ab12a302650b41a83ac651608494174484af22`.
The council worktree was untouched. Spec prose was used as the contract only.

**P0-1 — Reversed device mints remain usable through public oracle APIs.
Bead `b-isa-spec-dnjq`.**
At `reference-interpreter/src/tokens.rs:320-323`,
`resolve_invocation_tokens` checks program membership without mint validity.
Using `program()` from `tests/isa09_budget_mint.rs`, run `mint(1,0,0,0)`,
retain the returned `DeviceToken`, then `machine.reverse()`.
For input `ins:[1]`, `run_oracle_with_token("id", …, &token)` still returns
payload 1/code 0. `OracleMachine::new_with_token` also accepts it, and one
forward step followed by `commit(0,0)` returns (1,0).
ISA 0.9 §5.4 K3 revokes the token when its mint frame reverses; an invalid
call must reject before effects. Classical launch checks do not protect these
new public entry paths. The existing mint tests pass but miss this route.
Ran `timeout 120 rustc` against the freshly built archived-head rlib, then
`timeout 60 .tmp/rev-isaspec-main-r1/tmp/token-probe`: exit 0, asserting both
incorrect successes. The probe includes the existing fixture helper and is
retained beside the archive; its arithmetic expectation is the one-row identity.

**P1-2 — Source qinplace rejects its own generated operand name.
Bead `b-isa-spec-bf57` (historical baseline; existing job repair).**
At `reference-interpreter/src/source_constructor/action/qinplace.rs:12`,
the private `$qinplace/x/<counter>` is inserted in a temporary source operand
list. `self.call` sends it through `object::list`, whose NAME grammar rejects
it, preventing the required §4.5/§11 expansion.
Ran the focused `source_qinplace_adjoints_preserve_instruction_allocation_and_release_order`
test below: exit 101, one failure at `tests/isa09_qinplace.rs:286`,
`E_SCHEDULE_INFEASIBLE: §16.2: operand lists require names separated by comma and space`.
The already-reviewed job delta changes this exact line to a legal lexical
alias. No duplicate implementation is requested: reconcile this baseline bead
with that existing repair and council result. That repair was not graded here.

P2+ nits, one bead **`b-isa-spec-81ta`**: over-300-line source-constructor/binding/expansion functions; redundant `expand_bound_local` wrapper; stale source-lowering/job header in `ref_library.rs:3-4`; unnamed K83 catalog phase indices at `ref_language.rs:1359-1361`.

All commands were headless with the brief's private TMPDIR/CARGO_TARGET_DIR,
core dumps disabled and Cargo address space capped at 8 GiB. From the archive:

```sh
timeout 600 cargo test --offline -j3 --manifest-path reference-interpreter/Cargo.toml \
  --test isa09_cleanup --test isa09_lifted_cleanup --test isa09_registers \
  --test isa09_memory_address --test isa09_pinned_free --test isa09_tape_replay \
  --test isa09_source_schedule --test isa09_budget_mint \
  --test k83_source_support --test k84_diagnostics -- --test-threads=4
timeout 300 cargo test --offline -j3 --manifest-path reference-interpreter/Cargo.toml \
  --test isa09_forwarding -- --test-threads=4 --skip source_constructor_binds \
  --skip source_forwarding_large --skip source_expansion_host
BENNETT_CONFORMANCE_BIN="$CARGO_TARGET_DIR/debug/bennett-isa-ref" \
timeout 600 cargo test --offline -j3 --manifest-path conformance/Cargo.toml \
  --test ref_language --test ref_named_interface --test ref_k83 \
  --test ref_construction --test k91_vectors -- --test-threads=4
timeout 180 cargo test --offline -j3 --manifest-path reference-interpreter/Cargo.toml \
  --test isa09_qinplace source_qinplace_adjoints_preserve_instruction_allocation_and_release_order \
  -- --test-threads=4
```

Results: 57 passed; 8 passed/3 filtered; 26 passed; one failed as P1-2.
The initial conformance attempt lacked its default binary path; explicitly
selecting the private archived-head binary made all 26 pass.
The three forwarding materialisation/resource witnesses were excluded from
this focused run, not edited or ignored in source.
Also ran each K83/K84/K91 fixture `generate.py --check --ref "$CARGO_TARGET_DIR/debug/bennett-isa-ref"`
under timeouts 90/90/120 seconds: all exit 0.
Scoped `git diff --check origin/main be109bd`: exit 0.
No full repo gate, gate tool, or sibling gate ran.

Test integrity: compared the pre-change `origin/main` validation and binding
code directly: its exact-type transfer rejects the dynamic/static-qreg test
fixtures, and its implicit wire import lacks K82's exact interface validation.
The corresponding head tests passed. Expectations in the examined tests use
published vectors, explicit state/phase results or independent BDD arithmetic;
fixture hashing is not mistaken for semantic proof. P1-2 is an existing red
test, not weakened or skipped to claim a green baseline.

Anti-slop: checked every COMMON rule. P0-1 is the confirmed missing validity
check; function size, wrapper, stale wording and phase literals are nits.
No added tracked litter, weakened tests, unrelated writes or additional
confirmed semantic bug from flags/defaults/predicate duplication was found
in this scoped pass.

Coverage: read closely `conformance/src/ref_language.rs` changes and
`named_interface.rs`, cleanup validation/replacements, tokens and source-token
binding, register runtime/validation, `oracle/program.rs`, ref-library/source
construction changes and their focused tests; read K83/K84/K91 generators.
Skimmed remaining scoped source/test diffs, CLI plumbing, hook/gate changes,
and the previously covered claims/replay/pairs/qinplace/images, certificate
admission and K91 schedule-binding interactions. The prior review's inherited
claim/library findings remain recorded in
`docs/reviews/2026-11-06-quality-isaref-r1.md`; this pass does not re-grade their
job-branch repairs or claim council clearance.

Provenance: implementer **gpt-6.1-sol**; reviewer **gpt-6.1-sol, fresh session, D9**.

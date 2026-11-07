# ISA adjoint-table review, round 2 (last)

Verdict: **BLOCK**. P0-1: **`b-isa-spec-9exx`** (oracle).

Reviewed range: `main..job/isa-adjoint-table`, base `3b1090790d82a00ac95d30b8da6b42d1d7a9f5d1`; fix diff `d31d5c9..job/isa-adjoint-table` (`a900cf9`, `80732c7`). Exact reviewed head: **`80732c7f5c0d3da3ca09a416c1f5c598443843c7`**. Read the round-1 document first; reviewed only the repair diff otherwise.

**P0-1 — oracle: reached local types still inherit parser-only validity.** At `reference-interpreter/src/claims/adjoint09.rs:258`, the new closure check delegates to `validate_function`, which copies current types without checking their widths (`object.rs:1129–1138`). Its header check admits any `Type::Reg(_)` in the local register file (`object.rs:1939`); the positive-width check exists only in `Type::parse` (`object.rs:55–59`). Consequently a currently reached lproc containing a used `reg[0]` receives **Proof**, contrary to ISA 0.9 §3.3 (reg[N] requires N ≥ 1) and K94 premise 2. Both the public claim verdict and the certificate report establish this invalid source. This is a remaining instance of the unchecked-reached-premise defect, not an unrelated malformed-input concern.

Reproduced by appending this test to the existing integration-test source **in memory**, using its `lifted_dependencies(true)` helper (action `a` calls oracle `z` twice; `z` calls lproc `worker`):

```rust
#[test]
fn review_current_reached_zero_width_register() {
    use bennett_isa_ref::{Instruction, Param, Type};
    let (mut o, root) = lifted_dependencies(true);
    assert_eq!(o.verify_claim(&root, 0).unwrap(), ClaimVerdict::Proof);
    let worker = o.functions.get_mut("worker").unwrap();
    worker.regs.push(Param {
        name: "zero".into(), ty: Type::Reg(0), role: None,
    });
    worker.blocks[0].instructions.insert(0, Instruction {
        lhs: vec!["zero".into()], op: "lift.mov".into(),
        operands: vec!["zero".into()], bodies: vec![], line: 0,
    });
    assert!(o.verify_claim(&root, 0).is_err()); // actual: Ok(Proof)
}
```

The executed probe also printed `verify_certificate()`: its sole claim has `verdict: Proof`. Independent textual control: inserting `regs { zero:reg[0] }` into that worker before parsing rejects with exactly `§3.3: bundle width must be positive`; `Type::parse("reg[0]")` also rejects. The invalid location is used by an actual reached instruction, not dead metadata.

Command run: `timeout 45s python3 -` compiled the existing test source plus the probe from stdin with `rustc --edition=2021 --test --crate-name adjoint_review_repro - --extern bennett_isa_ref=<target>/debug/deps/libbennett_isa_ref-*.rlib -L dependency=<target>/debug/deps -o <tmp>/adjoint-review-repro`, making support/fixture paths absolute and supplying Cargo's `CARGO_MANIFEST_DIR` and `CARGO_BIN_EXE_bennett-isa-ref`. Then `timeout 45s "$TMPDIR/adjoint-review-repro" review_ --nocapture --test-threads=4`: **exit 101**, zero-width regression fails with `Ok(Proof)`; original oracle/lproc witness test passes. No tracked source or test was changed. Recheck current type invariants before establishing the theorem. Round 2 is final; this remaining P0 blocks landing under the two-round policy.

**Round-1 follow-up.** Independently reproduced the original `lift.magic` mutation in reached oracle `z`, and in transitively reached lproc `worker`, after first obtaining Proof. Both now reject with an `adjoint_table source @<name>` diagnostic, and both certificates reject. Thus `b-isa-spec-krbe`'s specific opcode witnesses are fixed, but the broader source-validity repair remains incomplete as P0-1 above. Empty-current-body regressions also pass. The shared body-shape helpers preserve the original parser predicates, error bytes and call sites; their additional per-function checks repeat conditions every parsed input already passed. No accepted-input change from that extraction was found.

P2+ nits: **`b-isa-spec-x8bk`**, one aggregate carrying forward `b-isa-spec-hqlj`: positive conformance checks parse/hash admission rather than Proof; absent quality ratchet; stale tracked HANDOFF/README guidance; existing tracked pycache/log/progress litter. Rechecked these files/inventory; no new repair-diff nit.

Verification (headless; `<tmp>` = `$PWD/.tmp/rev-isa-adjoint-table-r2/tmp`, `<target>` = `$PWD/.tmp/rev-isa-adjoint-table-r2/target`, set as `TMPDIR`/`CARGO_TARGET_DIR`):

- `timeout 240s cargo test -j3 --manifest-path reference-interpreter/Cargo.toml --test isa09_adjoint_table --test isa09_token_aliases -- --test-threads=4`: **38 passed** (20 + 18), no ignored. Includes exact loader diagnostic strings and current empty oracle/lproc blocks/functions; K91 test file unchanged from main.
- `BENNETT_CONFORMANCE_BIN=<target>/debug/bennett-isa-ref timeout 180s cargo test -j3 --manifest-path conformance/Cargo.toml --test adjoint_table -- --test-threads=4`: **1 passed**, both cases exercised.
- `git diff --check d31d5c9..HEAD`: pass. Read `git show main:reference-interpreter/src/claims.rs`: its unconditional Unverified fallback makes the new positive Proof assertions fail there. No full gate, sibling gate, branch switch or worktree creation.

Coverage: read closely all three repair files, §3.3/K94, dependency traversal, type/header/function/symbol/schema/version/graph validators and all added tests; skimmed surrounding claim dispatch, layout and validation context. Checked each COMMON anti-slop rule: no additional confirmed violation hiding a bug class. Inherited quality-plan gaps remain nonblocking as above. Reviewer fixed nothing.

Provenance: implementer **gpt-6.1-sol**; reviewer **gpt-6.1-sol, fresh session (D9)**.

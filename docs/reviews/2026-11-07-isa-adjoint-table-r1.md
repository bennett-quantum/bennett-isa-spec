# ISA adjoint-table review, round 1

Verdict: **BLOCK**.

Reviewed range: `main..job/isa-adjoint-table`, base `3b1090790d82a00ac95d30b8da6b42d1d7a9f5d1`, commits `0f6e92a` and `e9c8f75`; exact reviewed head **`e9c8f75e636517eccd36847faec0449e61be800e`**.

**P0-1 — oracle: linked lifted source is not rechecked before Proof** (`b-isa-spec-krbe`). At `reference-interpreter/src/claims/adjoint09.rs:148`, `adjoint_source09` validates only action/perm_action bodies; at lines 162–164 it merely scans oracle/lproc dependencies. K94 premise 2 requires the linked source's operand/signature/version/etc. conditions to hold. The public mutable `Object` API can therefore obtain `Proof` for a source containing an invalid linked instruction. This is the same API threat exercised by the new action-mutation regression, but its lifted callees bypass the check.

Reproduced using the new test file's `attach`, `CLAIM` and `support` helpers: create perm_action `a(x mut; tk token)` whose body allocates `f[4]`, performs `oracle_call @z tk () x f () ()` twice, and frees `f`. Linked oracle `z` has empty inputs, tgt[1], fault[4], register `r:reg[1]`, and body `r = lift.const.i1 1; lift.retval r`. Attach the universal empty-subclaim claim with `token_aliases:"all"`. Initially it proves. Then:

```rust
o.functions.get_mut("z").unwrap().blocks[0].instructions[0].op = "lift.magic".into();
let verdict = o.verify_claim(&root, 0);
assert!(verdict.is_err()); // actual: Ok(Proof)
```

Command run: compiled the existing integration-test source plus this check from stdin using `rustc --edition=2021 --test --crate-name adjoint_review_repro - --extern bennett_isa_ref=<review-target>/debug/deps/libbennett_isa_ref-*.rlib -L dependency=<review-target>/debug/deps -o <review-tmp>/adjoint-review-repro`, with the existing support/fixture paths made absolute and Cargo's two compile-time path variables supplied. Then `timeout 45s .tmp/rev-isa-adjoint-table-r1/tmp/adjoint-review-repro --exact review_oracle_dependency_validation --nocapture --test-threads=4`: **exit 101**, `invalid linked oracle verdict: Ok(Proof)`. No test/source file was written or changed. Validate the current lifted definitions, including reached lprocs, before establishing the theorem.

P2+ nits: `b-isa-spec-hqlj` — positive conformance case checks parse/hashes rather than the proof verdict (`conformance/cases/F/F-adjoint-table-published/case.json:7`); pre-existing quality-plan item 9 lacks a gate ratchet, item 10 leaves stale `HANDOFF.md`/README guidance, and tracked pycache/log/progress litter remains. No other new anti-slop violation hiding a bug class was confirmed.

Verification (headless; `<review-tmp>` = `$PWD/.tmp/rev-isa-adjoint-table-r1/tmp`, `<review-target>` = `$PWD/.tmp/rev-isa-adjoint-table-r1/target`; `TMPDIR`/`CARGO_TARGET_DIR` set accordingly):

- `timeout 240s cargo test -j3 --manifest-path reference-interpreter/Cargo.toml --test isa09_adjoint_table --test isa09_token_aliases -- --test-threads=4`: **28 passed**, no ignored. K91 test file byte-identical to main.
- `BENNETT_CONFORMANCE_BIN=<review-target>/debug/bennett-isa-ref timeout 180s cargo test -j3 --manifest-path conformance/Cargo.toml --test adjoint_table -- --test-threads=4`: **1 passed**, both cases exercised.
- `timeout 30s python3 -` independently compared `diff -u --label a/spec/Bennett-ISA-Spec.md --label b/spec/Bennett-ISA-Spec-0.9.md spec/Bennett-ISA-Spec.md spec/Bennett-ISA-Spec-0.9.md` with the committed diff, and compared fixture bytes against `git show d62140e874cd3581209bccd5be4ec38f841634bd:adders/objects/arith.bij.add.w1.bisa` in bennett-arith: **pass**; positive conformance object also byte-identical. No producer code read.

Coverage: read closely all 12 changed files, §5.3/§12.1/§16.3 example 3, claim dispatch/guard cleanup and relevant source, schema, invocation, identity, borrow and certificate checks; skimmed surrounding execution paths and quality documentation. Main-side `claims.rs` always returns Unverified for these positive adjoint claims, so the new Proof assertions would fail there. Malformed claims, refuted/evidence/unverified subclaims and repeated premises were exercised; no other narrowing confirmed. Checked every COMMON anti-slop rule and plan items 9–11. No full gate or sibling gate run.

Provenance: implementer **gpt-6.1-sol**; reviewer **gpt-6.1-sol, fresh session (D9)**. Reviewer fixed nothing.

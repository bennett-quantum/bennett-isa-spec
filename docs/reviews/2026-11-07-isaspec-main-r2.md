# ISA 0.9 main implementation review, round 2

**Verdict: CLEAR — no open P0/P1 in this scope.**

Reviewed head: `5b7120852f17e56063462d50f01fcf1da708ed1f` on
`job/quality-isaref`. Scope overrides the generic job range: round-1 findings
at `be109bd27889ea91e3c0ad78a30e5e4d71d3d0d3`, token fix
`7fc37fd8321138d96661371646c9ea72be237d3a` and its oracle entry paths/tests,
plus verification of the existing qinplace repair `0d95706` at head.
This does not re-grade the full quality branch or the concurrent council.

- **P0-1, `b-isa-spec-dnjq`: verified fixed.**
  `reference-interpreter/src/tokens.rs:367-372` now checks namespace and
  live mint identity for every concrete invocation token. The weak run owner
  expires at run drop; `classical.rs:1214,1428,1445` publishes reversal,
  committed state and rollback. The unchanged round-1 probe, compiled against
  the head library, now exits 101 at its former successful-call unwrap with
  `§5.4 K3: E_ARG: revoked or expired device token`.
  The head regression separately proves live success and rejection through
  both `run_oracle_with_token` and `OracleMachine::new_with_token`.
- **P1-2, `b-isa-spec-bf57`: verified fixed.**
  `reference-interpreter/src/source_constructor/action/qinplace.rs:24-31`
  binds a collision-free lexical alias to the private constructor identity.
  `source_qinplace_adjoints_preserve_instruction_allocation_and_release_order`
  passes, asserting allocation/release widths and hand-derived forward/adjoint
  modulo-4 results over every x/y input.

Ran headless with private `.tmp/rev-isaspec-main-r2/{tmp,target}`, core dumps
disabled and Cargo address space capped at 8 GiB:

```sh
timeout 600 cargo test --offline -j3 \
  --manifest-path reference-interpreter/Cargo.toml \
  --test isa09_budget_mint --test isa09_qinplace -- --test-threads=4
```

**13 + 31 passed, zero failed/ignored.** Both probes were compiled with
`timeout 120 rustc --edition=2021`, dependency search and `--extern` pointing
at the freshly built head rlib; compile-time Cargo manifest/binary variables
were supplied. An initial standalone compile lacked those variables and was
retried. Retained probes under `.tmp/rev-isaspec-main-r2/tmp/`:

```sh
timeout 60 .tmp/rev-isaspec-main-r2/tmp/r1-token-probe-at-head
timeout 60 .tmp/rev-isaspec-main-r2/tmp/lifecycle-probe
```

First: expected rejection/exit 101 as above. Second: exit 0, independently
asserting live success, both original API rejections, vector API rejection,
revoked token arguments with a live host primary, re-mint freshness, expiry,
host-token survival and observer-rejected uncall restoring token validity.
Its initial argument call omitted the required alias binding; supplying the
explicit host primary corrected the probe. No production/test source changed.

Red-first assessment: read `git show main:reference-interpreter/src/tokens.rs`
and the main-side qinplace constructor. Main's resolver lacks mint validity,
so the new reversed/expired/observer-rollback negative assertions fail there;
its private `$qinplace/x/...` still reaches the NAME-only operand parser.
Expectations use explicit identity arithmetic and before/after state; no
weakened, skipped or output-derived expectations in the fix tests.
Scoped `git diff --check be109bd 5b71208`: clean.
No full repo gate, gate tool or sibling gate ran.

P2+ nits, **one bead `b-isa-spec-s6lr`**: pre-existing 446-line
`classical.rs:1468` dispatcher touched by the mint fix; single-use
`tokens.rs:101` helper under COMMON's delete-don't-wrap rule. Earlier
`b-isa-spec-81ta` remains outside this fix review. All COMMON anti-slop rules
checked: enum represents origins; lock failures propagate; no new silent
default, duplicated predicate, unexplained policy number, process identifier,
tracked litter or confirmed semantic bug hidden by structure found.

Coverage: read closely `tokens.rs`, oracle literal/vector entry paths,
`machine.rs` constructors, classical mint/transaction/uncall/reversal paths
and complete `step_prepared`, new mint tests and qinplace alias/regression;
skimmed inherited alias binding and remaining focused tests. Other job changes
were excluded by the brief's scope. Existing finding beads have independent
verification comments; the council checkout was untouched.

Provenance: implementer **gpt-6.1-sol**; reviewer **gpt-6.1-sol, fresh session, D9**.

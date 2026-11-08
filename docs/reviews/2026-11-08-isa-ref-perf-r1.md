# isa-ref-perf review, round 1

Reviewed `650b9b742da8f95472fc482042ddf26a0c310141..705c2af4543bcbad6aaab7fe2260fdb1001b755a`
(local `main..job/isa-ref-perf`, aggregate three-dot diff). Reviewed head:
`705c2af4543bcbad6aaab7fe2260fdb1001b755a`.

**Verdict: CLEAR. P0/P1: none.**

The oracle change preserves value semantics. `Handle` is private; class labels
are constructed once, exposed only through immutable slices, and never mutated
after sharing. Slice/concat construct fresh vectors; ownership transitions alter
only status. Snapshots clone handles; register undo reinstalls prior whole handles
and restores the register/wire cursors. `Arc` equality compares contents, so the
existing full-state reversal assertions retain their meaning.

The allocation test counts real allocations only around the clone, on its own
thread, and checks independent widths 1/64 plus snapshot independence. Reading
`git show main:reference-interpreter/src/registers.rs` confirms that the old
derived clone deep-copies every nonempty class String: this test detects that
regression. The retained red receipt reports 8/71; my head run reports 6/6.
No seconds assertion or weakened test.

The new §4.6 sentence exactly matches the existing predicate-stack comparisons
in `validation.rs:181,232`, including zero-trip bodies and the inherited controls
of while loops. It permits balanced body-local predicates. §3.5 still defines
their control meaning; no interpreter semantics or promised capability changes.

**Independent checks:** headless, private reviewer TMPDIR/target, sequential
commands, >=6 GiB available and load1<8 before each focused run. The 1800-second
timeouts are hang guards following the existing `gate.toml` crate checks.

```sh
timeout 1800 cargo test --offline -j3 --manifest-path reference-interpreter/Cargo.toml --test register_snapshot_work -- --test-threads=4 --nocapture
timeout 1800 cargo test --offline -j3 --manifest-path reference-interpreter/Cargo.toml --test registers --test isa09_registers -- --test-threads=4
timeout 1800 cargo test --offline -j3 --manifest-path reference-interpreter/Cargo.toml --test isa09_adjoint_table zero_trip_ -- --test-threads=4
```

All **31 tests passed**. Replayed saved `entry.bisa` with
`run-program <entry.bisa> top --args 0,1000000,6,1000000,1000000,1000000 --budget 10000000,0,4096,16`
on the freshly built head and both cached pins (`3b1090790d82`, `ecaaac060c44`):
exit/stdout/stderr byte-identical. Both inner-balanced and outer predication
produced x XOR c and preserved c for all four basis inputs; split/unbalanced
one-trip loops rejected with the exact §3.5 diagnostic. Independently regenerated
the stable-label `diff -u`: byte-identical to the committed diff.
`git diff --check main...job/isa-ref-perf` passed; no tracked `.tmp` files.

Evidence audit: archived full-test production sources byte-match head; receipts
sum to reference 720 passed/6 ignored and conformance 130 passed. Recounted
differential rows: 520 unique matches to 3b10907, 479 compatible matches to
ecaaac0, zero differences. Those full suites, bisect and profile were inspected,
not rerun. No gate was run, as instructed.

**P2+ nits:** `b-isa-spec-og7g` (priority 3, review): inherited missing quality
baseline/ratchet in `gate.toml` and stale `HANDOFF.md` linked by README (plan
items 9–10); both predate this range. Remaining anti-slop checks pass.

Coverage: read closely all four changed files, class construction/read/clone/undo
paths, loop validation and §3.5/§4.6; skimmed retained performance/full-suite
evidence and quality documentation.

Provenance: implementer gpt-6.1-sol; reviewer gpt-6.1-sol, fresh session (D9).

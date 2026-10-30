# Brief: reference interpreter, council round 9 → round 10 fixes (opaque pairs in the operation stacks; loop bindings)

Repository `/home/tobias/Projects/bennett-workspace/bennett-isa-spec`, crate `reference-interpreter/` (reviewed snapshot
4796970; 315 tests + 5 sweeps). You are the **sole implementer** of `reference-interpreter/` for this round. Council
round 9 (`/home/tobias/Projects/bennett-workspace/bennett/docs/council/isa-ref-2026-10-29/round-9/`, members sol and
codex; report.md, out-sol.json, out-codex.json) returned: both revise; **every round-8 blocker resolved** on its
witnesses; no regressions; mean 4.19; two distinct S0 core blockers remain. The round-7 and round-8 briefs
(`docs/briefs/2026-10-30-isa-ref-round-7-fixes.md`, `...-round-8-fixes.md`: read-first list, method, gate, owned
paths, process) still bind; this brief states the round-9 work. Read the two blockers' full text in report.md first:
each carries exact objects, independently computed interface and object hashes, and the expected CLI results.

## The blockers

- **SOL-R9-B01 ≡ codex R9-B01 (core).** Matched opaque pairs (`oracle_call @o … ; oracle_call @o …` on the same
  operands with the same token, and `qinplace @g @ginv … ; qinplace @g @ginv …`, which by §5.3 is its own inverse
  pair once the second is read as the adjoint) permanently erase the enclosing saved fact: `Facts::sequence` admits
  only Flip/Swap to `reversible()` and calls `forget_inverses` on every opaque operand, so `h c; O; O; h c; qborrow;
  cx c b; qreturn b` is rejected with "§5.6: a wire released inside a borrow frame is not clean" although §5.2 makes
  the oracle an involution and §5.3 gives `oracle_call ↦ oracle_call`, `qinplace @g @ginv ↦ qinplace @ginv @g`.
  Fix: opaque operations join the **same per-wire LIFO operation stacks** as primitive permutations. The stack key
  retains callee(s), token/classical specialisation, the actual input/memory/output wires and the effective
  controls; the inverse identity is explicit (oracle_call self-adjoint; qinplace swaps g/ginv). Push the operation
  on every read/control/output operand that has quantum history; restore the saved expressions only when the
  matching inverse is on top of **all** those operand histories. Genuinely unmatched opaque readers and lifetime
  barriers still invalidate (do not simply keep an H record across an opaque read); the borrowed-touch guard, every
  return obligation, physical frame boundaries and runtime §4.5 checks are untouched. Both members' exact witnesses
  (sol: @o `lift.retval p` with `(c) t f`; codex: the `(c) d f` variant; sol's qinplace variant with the stated H and
  object hashes) go in as red tests in `tests/r1_borrow.rs` and pinned CLI counterparts in `tests/cli_xam.rs`, with
  the library-side forward/adjoint/reverse assertions the members list (all basis inputs, a superposition, a Bell
  pair between the borrowed wire and an untouched reference, `Budget::steps(1)` and `steps(0)` for the oracle pair;
  `steps(0)` for qinplace must still reject §4.5 before mutation). Required negatives: a pair split so only the first
  call precedes the borrow (must still reject even though the whole-region operation is identity); a changed-input /
  unmatched-reader variant; and a matched pair split across `qloop 1` bodies (positive) plus `qloop 2` around the
  pair, local child-call wrappers and interleaved histories on separate operands in the deterministic family.
- **codex R9-B02 (core).** `Compiler::compile` lowers a `qloop n` body once, so a body that frees and re-allocates a
  wire (`qreturn b; qfree c; c = qalloc any 1`) names the pre-loop wire on the second visit; `Facts::repeat` then
  reads/frees the dead wire and rejects the member's exact witness (count 2 and 3 fail, count 1 and the straight-line
  expansion pass). Fix: each executed visit resolves reads/releases against **that visit's entry binding map** and
  publishes its successor bindings; reuse an alpha-renamed or binding-parameterised transfer/cycle when
  environments recur (never unroll large counts, never a trip cap); preserve zero-trip omission (SOL-R8-B02) and
  lexical qregion hoisting. Red tests `r9_mixed_loop_threads_reallocated_bindings` in `tests/r1_borrow.rs` and the
  pinned `tests/cli_xam.rs` counterpart with the member's exact function, signature and hashes; counts 0/1/2/3,
  nested loops, the alpha-renamed expansion, Bell-entangled borrowed entries, and a dirty-control variant that still
  rejects at its first invalid return.

Hand-verify every pinned hash with the member's derivation before trusting it; if a member's pin is wrong, say so
with your own derivation and the exact bytes. Every round 4–9 positive and negative is retained.

## Deliverables and process

As in the round-8 brief: red-green per witness; deterministic variant families in `tests/r1_fuzz_discipline.rs`;
rerun the five ignored sweeps; README/PROGRESS "Design" paragraph updated with the opaque-stack rule and the
per-visit binding rule; response at
`/home/tobias/Projects/bennett-workspace/bennett/docs/council/isa-ref-2026-10-29/round-9/response.md` in the
round-8 format (one entry per blocker: status, section, hand derivation, what changed, witness, test names) with a
Design section. Owned paths: `reference-interpreter/` and the response file only (`spec/` and `conformance/` belong
to a concurrent agent; `conformance/tests/ref_call_flag.rs` is intentionally red in that agent's tree, ignore it).
Commit by explicit path list after every green step with `BEADS_HOOK_TIMEOUT=20` and absolute paths; never push;
headless (unset DISPLAY, WAYLAND_DISPLAY, DBUS_SESSION_BUS_ADDRESS, BROWSER; TMPDIR under the repo's `.tmp`); no
`cd` in compound commands; rule 10 throughout. Final message: commit hashes, per-blocker table, design summary,
every observed count (tests before → after, sweeps), any ISA 0.8 gap with the exact change.

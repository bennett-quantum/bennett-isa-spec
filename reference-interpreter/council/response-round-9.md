# Editor response: council round 9

Both distinct S0 core blockers are resolved. Inputs are the round-9 brief,
its binding round-8/7 briefs, the full member blocker objects and pins in
report.md/out-sol.json/out-codex.json, and accepted ISA 0.8. Tracking:
b-isa-spec-82k. Sole interpreter implementer; only reference-interpreter/ and
this response were changed/committed. No push, spec/conformance/Python-seed,
dependency, RNG-distribution or seed change. Existing deliverables 8–11 retain
their staging; this response claims neither their completion nor conformance
harness results. The concurrent intentionally red conformance test was not run.

## Design

1. Each live wire has an ANF basis expression; allocation alone supplies zero.
2. Permutations transform expressions, diagonals/readonly controls preserve them, and non-diagonal targets become independent unknowns.
3. Causal events save all operand expressions on every operand's LIFO stack.
4. A matching adjoint restores them only when the same event tops ALL those stacks.
5. Oracle calls carry a self-adjoint key; qinplace carries explicit swapped g/ginv keys.
6. Keys include callees, classical/token specialisation, controls and actual input/memory/output wires.
7. Unmatched opaque readers block outer restoration; lifetime/unresolved-import barriers invalidate connected histories.
8. Assigning qinplace starts history even on known basis entries; its loops use causal transfer, not uninterpreted assignment squaring.
9. Source lifetime lowering consumes released names and resolves each visit against its current live binding map.
10. Equal-width, equal-role, nonoverlapping successor lifetimes alpha-rename to reusable entry slots; changed maps get distinct transfers, recurring maps share a structured cycle.
11. Zero visits publish nothing; lexical qregions remain hoisted. No count cap or sampled/truncated admission.
12. Physical frame boundaries, full borrowed-touch checks and every return's restoration/independence proof remain mandatory. Ordinary permutation maps and return obligations still square; mixed/history transfers retain the existing exact-cycle/invariant rules.

## SOL-R9-B01 ≡ codex R9-B01 — resolved

Sections: §§4.5, 4.7, 5.2, 5.3, 5.6, 6.2, 14.2.

RED before source edits: library **0 passed / 4 failed**, CLI **0 passed /
4 failed**, covering the three opaque subjects and the separate loop subject.
All four in each target rejected with `§5.6: a wire released inside a borrow
frame is not clean`; CLI exit 1 and no data output. Logs:
`.tmp/r9-red-library.log`, `.tmp/r9-red-cli.log`. Initial test setup compilation
mistakes (Argument supplied in the controls slot, then the wrong bindings field)
were corrected before these semantic RED runs and are not reproduction evidence.

Hand derivation: @o reaches lift.retval in one row. At B=1, it XORs c into
Sol's t or codex's d and leaves f=0; the identical call cancels that XOR.
At B=0, payload=0 and code=1, so two fault XORs cancel as well. Thus
H;O;O;H restores c=0 and leaves all other quantum operands unchanged.
CX(0,b)=I for every bR, and every allocated workspace wire is clean at exit.
Sol's qinplace uses a fault-free identity oracle at B=1; the explicit inverse
is normative, and the same cancellation applies to arbitrary valid g/ginv
when the second key swaps them. B=0 must still fail the runtime §4.5 guard.
No function-specific certificate or new specification reading is involved.

Changes: `borrow.rs::Node::{adjoint,instantiate}` preserve an explicit opaque
inverse identity; `Compiler::compile` supplies self-adjoint oracle and swapped
qinplace keys. `Facts::{sequence,reversible}` route opaque events through the
same all-operand stacks as primitive gates, rather than forgetting their
ancestors. Operands read by an unmatched opaque call remain on top, preventing
an outer H pop. `forget_inverses` still invalidates connected history at genuine
barriers. Assigning qinplace creates a saved event even without a preceding H.

An additional RED audit, `r9_qinplace_repeated_pairs_on_clean_workspace`, was
**0/1**, with the same §5.6 diagnostic. A pure permutation Repeat was composing
opaque assignments as g^-1(g(x)), without causal cancellation. `Node::needs_history`
now routes such loops through the stack transfer/cycle. GREEN: **8/0** across
six library regressions and the two R9 families; no runtime or count restriction.

Exact library witnesses:

- `r9_opaque_adjoint_restores_borrow_control` (Sol oracle)
- `r9_matched_opaque_oracle_restores_outer_fact` (codex oracle)
- `r9_qinplace_adjoint_restores_borrow_control` (Sol qinplace)
- `r9_qinplace_zero_budget_rejects_before_mutation`
- `r9_qinplace_repeated_pairs_on_clean_workspace`

The first three have exact `r9_cli_*` counterparts in cli_xam.rs. Each library
identity checks all **8** t/b/R basis entries, a superposition and a Bell-paired
bR entry, independently initialized forward/adjoint followed by the opposite
direction, with exact state/allocator equality and no escaping workspace.
Both oracle subjects use steps(0) and steps(1); qinplace uses steps(1).
The B=0 guard test invokes qinplace directly on a superposition in each direction
and requires §4.5 with the complete state unchanged. The exact member action's
B=0 CLI run also requires exit 1, empty stdout and §4.5.

CLI positives require hash/check with literal byte pins, t=0/1 and
0.6|0>+0.8i|1> unchanged in independent forward and adjoint runs, wires=1,
and reverse equals_entry=true for each input. Total initial exact GREEN **8/0**;
extended budget GREEN **9/0**.

`generated_r9_opaque_histories_match_independent_identity`: **13 accepted /
5 rejected**. Positives include pair loops 0/1/2/3, a matched pair split across
two qloop 1 bodies, local child-call/adjoint wrappers in both orders, nested
and interleaved disjoint histories, qinplace with distinct swapped oracle names,
and qinplace on basis workspace without an outer H. Rejections cover unmatched
readers, changed inputs, changed effective controls, different token
specialisations, and a split pair whose two intervening completed borrowings
cancel globally but violate each individual return. Rejections also run through
CLI hash/check (exit 1, empty stdout); no final whole-region identity can waive
them. Runtime §4.5 checks and the full borrowed-touch guard are untouched.

## codex R9-B02 — resolved

Sections: §§3.3, 4.3, 4.6, 5.3, 5.6, 14.2.

The exact library `r9_mixed_loop_threads_reallocated_bindings` and CLI
`r9_cli_mixed_loop_threads_reallocated_bindings` were each RED as part of the
0/4 runs above; now both are GREEN with the member's exact signature, function,
interface/closure and object pins.

Hand derivation: c=0 before the first visit. Each visit applies CX(0,b),
returns b unchanged and independent of every surviving output, frees clean c,
allocates a new zero c, and applies H²=I to t. Induction gives c=0 at every
visit and final release; the region is I_t tensor I_bR on arbitrary bR.
Zero visits leave the original c binding untouched.

Changes: `Compiler::compile` removes consumed qfree/qreturn/quncopy/qunload
bindings. `Compiler::compile_repeat` compiles against the actual current entry
map and publishes its successor. A freshly produced successor can reuse an
entry slot only at equal width/borrow role and after that old slot is absent
from every live successor binding. This renames nonoverlapping lifetimes;
it performs no quantum exchange and carries no fact across release. Allocation
and lifetime barriers still reset/check the reused slot. First-visit changes
(e.g. width 1 to 2) compile separately; repeated live maps reuse the exact cycle
of structured transfers. Frames and trace snapshots receive the same alpha
renaming. No expansion proportional to n and no trip cap. Existing zero-trip
omission and unconditional lexical qregion hoisting remain.

`generated_r9_loop_bindings_match_independent_identity`: **20 accepted /
3 rejected**, covering counts 0/1/2/3, nested loops, alpha-renamed straight-line
expansions, first-visit width changes and dirty controls that reject at their
first invalid return. Nineteen positives execute the independent I_t tensor I_bR
map on all library inputs above; the **2^40** positive checks analysis/hash only.
A fresh-wire runtime exploration at 2^40 was stopped without a completed result:
execution's existing cycle key distinguishes newly allocated wire ids. It is
not claimed as an execution pass or rejection; existing performance follow-up
b-isa-spec-436 now records it. No execution limit was added or capability removed.

## Independent literal hash derivation

All member pins reproduced exactly; none required correction. The independent
standard-library script `tests/support/r9_pins.py` contains the complete literal
functions/signatures and writes their exact LF-terminated fixture bytes. It
uses Python hashlib, not interpreter output. For each memoryless function it
hashes `isa=0.8\nptr_width=32\n`, canonical sorted-key signature JSON plus LF,
then the exact function including its definition header. In caller body lines,
@o becomes @H_o; the distinct H_o is appended once plus LF in first-reference
order. The complete object is the standard header, blank line, symbols section
with stored H values in sorted-key JSON, blank line and functions section,
functions in the specified order, final LF. The script asserts every published
H and object hash before writing; Rust additionally pins stored hashes and
object byte digests. Logs: `.tmp/r9-independent-pins.log`.

| Subject | H_a = closure | object_hash | caller envelope bytes |
|---|---|---|---|
| Sol oracle | 4cabd490b0d1a8dfe31489ec234e91867e66b54c6c4d030eb4e8d1cf6f84bb38 | ef010633dbc984abbaa9fbfb97c6233c5390fa99a17bc38e09725cddec936832 | 519 |
| Sol qinplace | bc6e5d89276a62eec7074a716d8dbce40e94a2c80c287ad8fc5b5e5c6c756fbf | 77664ac9392aec50b0415027771ae7ef49dcea7762b70025f0cd918d53a6ee1e | 596 |
| codex oracle | be1f0e087e06fecd99df7c3cbb5c3c8c186fde8f814be1f826487e858fa66296 | 49ad3ba6c411ed73bebacdebe6728ebff5995dbf8bbc03c016d1cb0c5c3e8625 | 544 |
| codex loop | bb983f1dc5a8eea46912ea64d1226a9345a05d6cba6cc8452efc9f39b4a17a89 | 80478acc73d52bbf64c395816470293b5b4b4a1ee36a5c550bf00436f06fc192 | 283 |

Sol's oracle H_o = closure is
a8f4a15663aead9ecb6618fe3847deba546cf3172f645ffc7e763f551e529e27;
codex's is cab76c914fa09a3fc8237381c6de91e83eeaefbd65cb7f52571ed4020abe8926.
All seven new fixture symbols are independently pinned in r1_fixtures.rs.

## Retained checks and observed verification

Every existing round 4–8 positive and negative remains. This includes completed
lifetime independence, later-borrow/global-identity nonwaivers, certificate
subject binding and full-domain replay, full borrowed phase/quantum/call touch
sets, unmatched primitive/opaque readers, diagonal/transitive readers, window
and cell versions, evolving controls beyond iteration three, changed-width and
zero-trip lifetimes, and lexical local regions. Certificate fallback and
physical boundaries were not weakened. No conformance command was run.

All commands unset DISPLAY, WAYLAND_DISPLAY, DBUS_SESSION_BUS_ADDRESS and BROWSER,
set TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp, use
absolute paths and --offline. Test optimization was CARGO_PROFILE_TEST_OPT_LEVEL=1,
with the same assertions, overflow checks and domains. Commands:

```sh
cargo fmt --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml -- --check
cargo clippy --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --offline --all-targets -- -D warnings
cargo test --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --offline
cargo test --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --offline -- --ignored --nocapture
python3 /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/tests/support/r9_pins.py
```

Development filters used the same manifest/environment: `--test r1_borrow r9_`,
`--test cli_xam r9_`, `--test r1_fuzz_discipline generated_r9`,
`--test r1_borrow r9_qinplace_repeated`, and the combined borrow/family targets.

| Run | Observed result |
|---|---|
| clean archived baseline | 315 passed / 0 failed / 5 ignored; 8 unit + 307 integration; 15.96 s runner sum |
| exact RED library / CLI | 0 passed / 4 failed each |
| exact GREEN library + CLI | 8 passed / 0 failed |
| first full suite | 323 passed / 0 failed / 5 ignored; 37.65 s runner sum |
| budget-extended GREEN | 9 passed / 0 failed |
| intermediate families | 2 passed / 0 failed; 28 accepted / 7 rejected (11/4 opaque, 17/3 loop) |
| final R9 families | 2 passed / 0 failed; 33 accepted / 8 rejected (13/5 opaque, 20/3 loop) |
| assigning Repeat RED | 0 passed / 1 failed |
| extended library/family GREEN | 8 passed / 0 failed (6 + 2) |
| final fmt --check | pass; 0 tests executed |
| final strict all-target Clippy | pass; 0 warnings; 0 tests executed; 1.16 s |
| final normal suite | 327 passed / 0 failed / 5 ignored; 8 unit + 319 integration; 0 doctests; 14.65 s runner sum |
| final five ignored sweeps | 5 passed / 0 failed; 93.83 s runner sum |

| Ignored sweep | Domain | Observed result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 pairs × 4 operations = 262,144 executions | 1 passed / 0 failed; 0.51 s |
| 17-wire borrow | 131,072 entries | 1 passed / 0 failed; 2.72 s |
| checked borrow certificate | 262,144 domain elements, subject and expected program | 1 passed / 0 failed; 14.85 s |
| §6 differential | 200,000 programs: 70,930 accepted (all exactly clean) / 129,070 rejected | 1 passed / 0 failed; 73.38 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 1 passed / 0 failed; 2.37 s |

Archived pre-fix CLI replay additionally ran **44 commands**: all four subjects,
hash/check and three inputs in forward/adjoint/reverse, requiring exit 1,
empty stdout and the exact §5.6 diagnostic. A final test-only pin strengthening
explicitly compares every library/CLI closure hash with the stored member
interface pin as well: targeted **10 passed / 0 failed**, followed by full suite
**327 passed / 0 failed / 5 ignored** (14.61 s runner sum), fmt pass and
strict all-target Clippy pass (0 warnings, 0 tests, 0.29 s). No semantic source
change followed the sweeps. Tracking b-isa-spec-82k is closed.

The initial live baseline overlapped newly created fixture files and failed
its fixture audit **2 passed / 2 failed**; it is not a baseline pass. The isolated
git-archive baseline lacked the spec include on its first build; a read-only
symlink supplied the unchanged accepted spec before the completed 315/0/5 run.
Initial generated family **0 passed / 2 failed** was fixture blank-line syntax;
a later changed-control fixture hit §3.3 target aliasing and was corrected.
The intermediate huge runtime exploration was stopped before its loop-family
result; no interrupted run is counted as a pass or semantic rejection.
Final logs are `.tmp/r9-final-{fmt,clippy,tests,sweeps}.log`; baseline, RED and
intermediate logs are named in PROGRESS.md.

## ISA 0.8 gaps and disposition

No new ISA gap, amendment, decline or partial blocker. The fixes implement the
existing adjoint/lifetime semantics and do not change SPEC-QUESTIONS 20/25/28/30.
No trip/wire/budget cap, frame merge/waiver or function-specific certificate
requirement was added. Existing deliverables 8–11 and performance follow-up
b-isa-spec-436 remain disclosed. Accepted spec SHA-256 remains
90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e.

Implementation commits: **334c27d**, **b07ab5b**; closure-pin/final evidence
commit **532e3de**. This response is committed separately in the council repository.
Only explicit owned paths were committed with BEADS_HOOK_TIMEOUT=20; no push.

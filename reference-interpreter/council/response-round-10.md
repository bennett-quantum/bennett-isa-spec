# Editor response: council round 10

Both distinct S0 core blockers are resolved. Inputs: the round-10 brief,
binding round-9/8/7 briefs, both members' full blocker text and exact objects
in report.md/out-sol.json/out-codex.json, accepted ISA 0.8, interpreter
README/PROGRESS/SPEC-QUESTIONS/BRIEF and round-9 response. Sole interpreter
implementer; tracking b-isa-spec-eka is closed. Implementation **4ba57f9**;
final verification evidence **a9f9261**. Only explicit interpreter paths and
this response are committed. No push. Spec/conformance/include/Python seed,
dependencies, RNG distribution and seeds are unchanged. The intentionally
red concurrent conformance test was not run. This response does not claim
completion of staged deliverables 8–11 or an independent council-wide audit.

## Design

1. The §16.2 fhead recognizer couples fn exclusively to cc/r; Q kinds are bare.
2. Function boundaries use that same recognizer; malformed later headers fail grammar before semantic validation.
3. Present bounds and explicit LHS productions require at least one element; empty operands and omitted bounds remain legal.
4. Legal header-like SSA names remain legal; exact object/function bytes are never normalized before hashing.
5. Lexical predicate stacks retain ordering, duplicates, push/pop matching and expansion-owned floors.
6. EffectiveControls resolves scalar physical wire literals, retains polarity, sorts, and deduplicates equal literals.
7. Every §6 context-bearing key uses that semantic set with complete current version-stack snapshots and dependency metadata.
8. Copy/load creation records, oracle/action/gate/memory/phase keys and single/joint changes obey the same rule.
9. Borrow compilation canonicalizes after physical resolution, including direct qpred, and canonicalizes again after callee/loop instantiation.
10. Inverse events still require the same event atop ALL operand histories; those histories retain changes to control versions.
11. ABI operands remain ordered; callee(s), token/classical specialization, polarity and explicit qinplace g/ginv inverse identities remain significant.
12. Physical frames, full borrowed-touch guards, lifetime barriers and every return's restoration/independence obligations remain mandatory. No cap or frame waiver.

## SOL-R10-B01 + SOL-R10-B02 ≡ codex R10-B01 — resolved

Sections: §§16.1–16.2, suite F. These are explicit grammar productions,
requiring no SPEC-QUESTIONS exception.

Hand derivation: fhead has exactly two prefixed alternatives (fn cc, fn r)
and four bare Q kinds. Its optional bounds suffix contains a mandatory first
INT. Its optional assignment contains a mandatory first NAME. By contrast,
operand lists can be empty. Therefore bare cc/r, all four fn-prefixed Q
kinds, bounds {}, and () = are grammar errors before ret/lift.retval executes.
Canonical fn cc @a with a single ret completes in exactly one row and returns
[]; adding an empty regs declaration without bounds has the same execution.
The one-trip action applies X once, and inc8's tuple-LHS control computes
255+1 modulo 256 = 0 in const/add/retval, three rows.

RED before production edits:
- tests/parser.rs: **0 passed / 3 failed**.
- tests/cli_xam.rs grammar subset: **1 passed / 1 failed**; legal controls pass.
- Aggregate reproduction retained all **12** header-position cases, three
  literal inc8 mutations, the empty-bounds CC and zero-result () = ret.
  The CLI test exercised **152 commands** across **19 objects**.
  Some malformed later Q headers already failed §16.2; these are retained
  negatives, not claimed new admissions. Bare headers and malformed first
  definitions either loaded or reached a later semantic error instead of
  the mandatory grammar diagnostic. The aggregate log names every case.
- All three pinned inc8 mutations were independently observed as accepted.
  Their parser rejection assertions were red, independently of hash checking.

Changes: object.rs::fhead_prefix is shared by parse_functions and is_head.
is_head distinguishes actual definition starts from legal SSA names such as
fn, r, action and oracle. Malformed later prefixes return §16.2 directly.
parse_functions rejects empty contents of a present bounds clause.
instruction rejects empty or non-NAME result lists, without changing operand
list parsing or rewriting source bytes.

Witness tests:
- parser::r10_function_header_prefix_matches_kind
- parser::r10_present_bounds_requires_an_integer
- parser::r10_exact_fhead_and_nonempty_optional_productions
- cli_xam::r10_cli_rejects_exact_grammar_witnesses_before_execution
- cli_xam::r10_cli_pinned_grammar_positive_controls

CLI negatives require exit 1, empty stdout and §16.2 for hash/check/run-cc;
run-oracle is checked too, at budgets 0/1/3 and inc8 input 255. Nothing
executes before the grammar diagnostic, even for a nonexistent entry in the
malformed object. The exact malformed pins remain byte-for-byte fixtures in
support/r10_cases.json, outside the audit of well-formed .bisa files.
Positive controls independently pin fn cc, fn r, omitted bounds, one-trip
bounds {1}, and tuple LHS, including check/hash and execution/reversal as
appropriate. All existing bare canonical Q fixtures still pass.

Deterministic grammar family: **7 accepted / 16 rejected**. Every forbidden
header occurs first and later; seven header-like NAMEs are accepted as
workspace names, preventing a new grammar restriction. The two-operand tuple
LHS in arithmetic.bisa's lift.parmov remains covered by the unchanged suite.

## codex R10-B02 — resolved

Sections: §§2, 3.5, 4.7, 5.2, 5.3, 5.6, 6.2, 14.2.

Hand derivation: conjunction is commutative and idempotent. Both oracle calls
therefore execute under p*q in the exact witness, or under p in the duplicate
variant. At B=1 each XORs that same controlled c into d with f unchanged.
At B=0 each XORs the same controlled FAULT_BUDGET_STEPS into f. Either pair is
identity. H² restores c=0; CX(0,b)=I on every borrowed b and arbitrary
entangled reference R. All workspace is clean, and the action is identity on
t,p,q,bR. The qinplace cases use the explicit swapped g/ginv adjoint at B=1;
existing B=0 runtime §4.5 rejections remain mandatory and passing.

Exact witness H_a=closure:
386dce74595f0e05ed2b7eb9218fc22fece3fafe7ad6fd206aed542ad8b7deef.
H_o=closure:
cab76c914fa09a3fc8237381c6de91e83eeaefbd65cb7f52571ed4020abe8926.
Object:
7b1153bae80cc041447870eaa9623f73e6cb6df5085365a1efcbac99789076ad.

RED: r1_borrow::r10_equivalent_control_sets_preserve_opaque_inverse was
**0/1** and explicitly collected both exact and duplicate rejections:
§6.2: released value has not been uncomputed.
cli_xam::r10_cli_equivalent_control_sets_preserve_opaque_inverse was **0/1**,
exit 1, empty stdout, the same diagnostic at hash/check.
Aggregate exact RED (all three targets): **1 passed / 6 failed**.

Changes: controls.rs::EffectiveControls is the shared semantic constructor.
validation.rs::Discipline::effective_controls resolves positive/negative
lexical predicates to physical scalar literals and pairs them with their full
current interned version stacks (collision-free IDs referencing full keys,
not hashes or truncated histories). Creation.controls and all operation,
phase, memory, oracle/action, change/joint_change context keys use it.
Phase dependency snapshots are sorted/deduplicated through read_snapshot.
Lexical push/pop and protected floors are untouched.
borrow.rs::Compiler::compile constructs canonical controls separately from
the lexical predicate stack, including direct qpred duplicates.
Node::instantiate sorts/deduplicates again after physical mapping and
combining inherited controls, so aliases introduced by calls or alpha-renaming
have the same representation. Facts::reversible retains its all-operand
LIFO comparison and complete saved expressions. Callee/token/input/output
identity and every version-changing intervening event remain significant.

GREEN: exact parser/library/CLI targets **7 passed / 0 failed**.
Both exact and duplicate objects check/hash; run-action a --mut 0 --in 1,1
--arg token:1,0,0,0 returns state {"011":[1,0]}, wires=3, unchanged at token
0,0,0,0. Independent CLI adjoints and reverse equals_entry=true pass.
CLI also checks a complex superposition across control settings.

support::r10_identity independently initializes each direction on all **32**
t,p,q,b,R basis entries, a control superposition, and a Bell-paired bR entry.
It checks exact state AND allocator equality after one execution, then the
opposite direction, with no escaping workspace. Oracle subjects use B=0/1;
qinplace uses B=1.

Deterministic tests:
- generated_r10_effective_control_sets_match_independent_identity:
  **102 accepted / 6 rejected**, all positives executed as above.
  Five permuted/duplicated/indexed scalar contexts cross each other in direct,
  doubly nested qloop 1, local action-call/adjoint, and distinct swapped
  qinplace g/ginv forms; duplicate-only [p] vs [p,p] and p[0] forms are added.
  The child uses its own p/q stack, exercising inherited-context merging.
  Rejections: absent predicate, opposite polarity, changed version, different
  token, changed ABI operand order, and different callee.
- generated_r10_context_keys_cover_all_cleanup_families:
  **9 accepted / 1 rejected**. qcopy/quncopy, qload/qunload, qstore, qexchm,
  qexch, H/H, S/Sdg and X/X under reordered/duplicated controls, plus equivalent
  negative-polarity contexts. A wrong lexical pop still rejects §3.5.
- r1_fixtures::r10_independent_effective_control_and_grammar_pins:
  independent root interface AND closure pins, object bytes, malformed digests.

All six distinct-control negatives reject parse/hash/check at §6.2; the
opposite-polarity case uses qif's else arm rather than an illegal spelling.
The changed-version case makes p mutable, changes it between predicate scopes,
and restores it afterward; no active predicate is illegally targeted.

## Independent literal hash derivation

Every supplied member pin reproduced exactly; no correction is needed.
tests/support/r10_pins.py uses only Python hashlib/json and literal function
bytes. For a memoryless function the exact envelope is:
isa=0.8 LF; ptr_width=32 LF; sorted-key canonical signature JSON LF; exact
LF-terminated function. Caller references @o are replaced with @H_o; distinct
H_o is appended once plus LF in reference order. Complete objects use the
standard header, blank-line section separators, sorted-key stored-H symbols,
functions in declared order and final LF. Rejected spellings stay rejected
spellings in both the envelope and the stored exact object.
The script asserts all supplied H/object pairs before writing the JSON corpus;
Rust checks successful interface/closure/object pins and rejected byte digests.
The 13 independently derived cases follow.

| Case | interface = closure (root) | object_hash |
|---|---|---|
| bare-cc | 2060222c2773451dd9a97e146d566bfca81c7a91c7fe04bd90d7e841163f531e | b6f4dc99c6f84d162c82c54ea4c41ceee3e9d4cc09a00c3ab8bd80d282faec3f |
| bare-r | 26aaa60527078d132feacc2ad2d7fcdc498854fda628b6131b12febf6dfa474e | ddd846be15e738a12a572592feb66b3c1a69750614033b2b89ef63a6f0c8e89a |
| cc | 218bf0ae43801ce9c291e2c3276d60df583a7a8b939b56312f88b1623c2e68cd | d2cb6bf361cd11bd9e68e31a0329b5b62bb9c2d23cf6cb7ca9d8ddf89bd208e5 |
| controls | 386dce74595f0e05ed2b7eb9218fc22fece3fafe7ad6fd206aed542ad8b7deef | 7b1153bae80cc041447870eaa9623f73e6cb6df5085365a1efcbac99789076ad |
| duplicate | 3c0aeff8beef09c65102af9b1000ba9147168f126398f1dfc9a43bc42a08189a | fab3d3bdec75198b190125a297e9c0145f25de221f8ede05beaf5949f85ce384 |
| empty-bounds | cb5e25ba34a6c84c96766f19a4b30c0999b7f262c2df63ae436296ba4a2554c8 | 8f2d76a7e18603d59a53082cd0493913725397140a6cbd162b7150fdb5358f11 |
| empty-lhs | a2f5278ca8db3abb0e407e475c1fba93c4307b97f0db8c1fe4a0bbe2b48da782 | 4ede077a7344efa101b22cd0f990feacde2c1a415dbec6667ee61290f9747480 |
| fn-oracle | 607bffa4376f76dc181141c19dc70dfbde0d8064e3d220187b13001e25cab0ca | 3868eb11e0293db6f43d3a9c5589c36feab4d88916327a34241141c1949b2dd6 |
| inc-empty-bounds | 31e7ad0837d13f0cd78e0fc2a155016c006ab18e3ecf6cc367308d58eb274437 | 6e0b6cfb1a74f8974706264ddc3b762bda7226e0abeb284f97978d5cb2f7d4f3 |
| omitted-bounds | 4b749e1b37d610bc72f3b4d5c63ea551364492a370bd2dc5054282c5b1b89e62 | 028699ab2f8bd8767b19d021e1153bac74c470d9a302636fe66928bcc0d74de5 |
| one-trip | 0b8ab098a6ba2bb42a91bb939a2d8efa08855ce6bf8500287e629b059bca35ae | 0795eacd49c0eab4b11a0640c5e52487fb7ea87b83d14f6cc5e04122deeb3011 |
| r | a0e0e58bbd950d19ef988ea34e59ae270fa891a2988e9d7ef94280b7bc5c02ee | 26db00f6a415fc59ab5df69e88b866b874ba637488efb178d6d8dccf5d641361 |
| tuple-lhs | d3d56fca12ac5d5241a321cf1095d53f605f1fd6569f83a7e1171182194f8754 | 5ae74605fdb749d2da04e9d4def8475785c84b0a602fb58d31ac8fff8efbcfc5 |

## Retained checks and observed verification

Every named round 4–9 positive and negative remains and passes: every-return
independence, later-borrow/global-identity nonwaivers, physical frame boundaries,
certificate binding/full-domain replay, full borrowed quantum/phase/call touches,
unmatched primitive/opaque readers, changed tokens/controls/inputs, full
version-stack prefixes, cell/window epochs, evolving loop bindings, zero-trip
lifetimes and lexical qregions. No new specification reading was introduced.
No test domain, seed, random distribution or runtime guard was weakened.

Commands use absolute paths; DISPLAY/WAYLAND_DISPLAY/DBUS_SESSION_BUS_ADDRESS/
BROWSER are unset and TMPDIR is
/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp.
All Rust tests use CARGO_PROFILE_TEST_OPT_LEVEL=1 (same assertions, domains,
overflow checks). The exact commands after this environment prefix are:

```sh
cargo fmt --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml -- --check
cargo clippy --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --offline --all-targets -- -D warnings
cargo test --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --offline
cargo test --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --offline -- --ignored --nocapture
python3 /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/tests/support/r10_pins.py
```

Development filters used the same manifest/environment: --test parser r10_,
--test cli_xam r10_, --test r1_borrow r10_, --test r1_fuzz_discipline generated_r10,
and --test r1_fixtures --test r1_fuzz_discipline r10. The aggregate exact
reproduction used all three exact targets, filter r10_, --no-fail-fast and
-- --nocapture so every member mutation was observed before the source repair.

| Run | Observed result |
|---|---|
| baseline | 327 passed / 0 failed / 5 ignored; 8 unit + 319 integration; 13.40 s |
| initial parser RED | 0 passed / 3 failed |
| initial grammar CLI RED | 1 passed / 1 failed |
| initial library RED | 0 passed / 1 failed |
| initial control CLI RED | 0 passed / 1 failed |
| aggregate exact RED | 1 passed / 6 failed; 152 grammar CLI commands |
| exact GREEN | 7 passed / 0 failed |
| first full suite | 334 passed / 0 failed / 5 ignored; 16.85 s |
| first deterministic families | 2 passed / 0 failed; 109 accepted / 22 rejected |
| extended families plus pin test | 4 passed / 0 failed; final families 118 accepted / 23 rejected |
| full suite with final families | 338 passed / 0 failed / 5 ignored; 15.67 s |
| extended one-trip/tuple CLI controls | 3 passed / 0 failed |
| final normal rerun | 338 passed / 0 failed / 5 ignored; 8 unit + 330 integration; 0 doctests; 17.70 s |
| fmt --check | pass, 0 tests executed |
| strict all-target Clippy | pass, 0 warnings, 0 tests executed; final rerun 1.25 s |
| five ignored sweeps | 5 passed / 0 failed; 100.42 s runner sum |
| independent pins | all supplied pins match; 13 derived case records; no correction |
| owned diff --check | clean |

| Ignored sweep | Domain | Observed result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 pairs × 4 operations = 262,144 executions | 1/0; 0.52 s |
| 17-wire borrow | 131,072 entries | 1/0; 2.91 s |
| checked borrow certificate | 262,144 domain elements, subject and expected program | 1/0; 15.23 s |
| §6 differential | 200,000 programs: 70,931 accepted, all exactly clean / 129,069 rejected | 1/0; 78.84 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 1/0; 2.92 s |

The unchanged RNG previously yielded 70,930 accepted / 129,070 rejected;
the effective-set repair admits one more independently executed clean sample.
No semantic source change followed the first 338/0/5 gate or the sweeps.
The one-trip and tuple-LHS test-only controls were strengthened after that
gate, then the full suite, fmt and Clippy were rerun successfully.
One implementation build caught an accidental edit to a lexical stack
extension; it was corrected before GREEN. An extended pin test initially
missed mod support; fixed before the 4/0 target run. Neither compiler setup
failure is counted as semantic RED or as a passing test run.

Logs live in bennett-isa-spec/.tmp/r10-*.log, specifically baseline, red-all,
green-exact, first-full, families, families-extended, positive-extended,
final-tests, final-rerun-tests, final-sweeps, final-rerun-clippy,
final-fmt and independent-pins. PROGRESS.md is current through these gates.

## ISA 0.8 gaps and disposition

No new ISA 0.8 gap, ambiguity, decline or partial blocker. **Exact spec change
needed: none.** SPEC-QUESTIONS is unchanged; the fixes implement explicit
existing grammar, conjunction, adjoint and lifetime rules. No trip/wire/budget
cap, sampled admission, frame merge/waiver or function-specific certificate
requirement was introduced. The whole ISA remains the north-star scope;
existing deliverables 8–11 and performance follow-up b-isa-spec-436 retain
their disclosed staging without claiming completion.

Accepted spec SHA-256 remains
90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e.
Commits: **4ba57f9** implementation/tests/design; **a9f9261** final evidence.
This response is committed separately in bennett. All commits use explicit
owned path lists and BEADS_HOOK_TIMEOUT=20; no push or Dolt remote sync.
Beads' automatic export is outside owned commit paths and left to the maintainer.

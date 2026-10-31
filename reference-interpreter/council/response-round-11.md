# Editor response: council round 11

All three S0 blockers, each reported by both members, are resolved; none
declined or partial. Read: round-11 and binding round-10/9/8/7 briefs, both
full member reports and exact witnesses/pins, accepted ISA 0.8, interpreter
README/PROGRESS/SPEC-QUESTIONS/BRIEF and round-10 response.
Sole interpreter implementer; tracking b-isa-spec-x02. Commits:
**d3e3c21** B01, **fc6ab68** B02, **352e8f5** B03/audit/design; **7da045f** final verification evidence.
Only explicit owned interpreter paths and this response are committed.
No push. Spec/conformance/include/Python seed, dependencies and RNG/seeds
remain unchanged. The intentionally red concurrent conformance test was
not run. Every Rust test used only the interpreter manifest.

## Design

1. Header validation enforces register-file legality before execution or hashing.
2. Oracle/lproc locations remain reg[N]; CC/R retain dynamic SSA.
3. Action/perm_action workspace requires Q producers; nonempty regs rejects §3.3.
4. Empty regs remains legal, including an action's loop-bounds clause.
5. Named rotation angles resolve specifically to f.params[3] f64 arguments (§3.1).
6. No phantom angle is defaulted; finite binary64 literals retain their values.
7. Oracle layout selection uses exact interface_regions JSON owner keys.
8. Filtering precedes flattening, even when dotted owner/region renderings collide.
9. Every non-interface layout leaf remains; own leaves retain §9.2 rendering/sorting.
10. Legal dotted names are never restricted or normalized.
11. Fixture evidence distinguishes exact support, diagnostic rounding and token preconditions.
12. Exact interference uses discrete minus-I; rounded residuals and qinplace B=0 rejection remain tested.

The borrow engine, physical frame boundaries, full effective-control/version
identities, all-operand inverse histories and every-return obligations are
unchanged. No count cap, sampled admission, frame waiver or scope reduction.

## SOL-R11-B01 ≡ codex R11-B01 — resolved

Sections §§2, 3.1, 3.3, 3.7, 5.3, 16.2.

Hand derivation: regs declares locations only for oracle/lproc. Neither
d:wire nor angle:f64 in the exact malformed objects has a structured Q
producer or entry argument binding. Thus reject at load under §3.3.
In the producer control, qalloc d starts at 0, CX(0,t)=I, qfree d is clean.
In the argument control, angle:f64 is the fourth parameter group and
Rx(0.0)=I. Empty regs creates no location and may carry loop bounds.

RED before production edits: parser **0 passed / 3 failed** (both exact
objects and the kind/type family); initial CLI **1 passed / 1 failed**
(wire hash erroneously emitted status ok, legal controls already passed).
Separate exact angle CLI RED: **0 passed / 1 failed**, hash erroneously
emitted status ok at the member's exact pins. Combined initial RED: **1/4**.

Changes: object.rs::validate_header rejects nonempty action/perm_action regs
before seeding the structured environment. validation.rs::validate_action_sequence
requires named angles to match an f64 classical argument, not the type map.

Witness tests:
- parser::r11_action_regs_do_not_produce_wires
- parser::r11_action_regs_cannot_invent_angles
- parser::r11_register_types_follow_function_kind
- cli_xam::r11_cli_action_regs_reject_before_execution
- cli_xam::r11_cli_action_angles_reject_before_execution
- cli_xam::r11_cli_producer_and_argument_controls

GREEN **6/0**. Register family: **8 accepted / 52 rejected** across six
kinds × nine nonempty types plus six empty-regs controls. Only reg[N] files
on oracle/lproc are accepted. Empty regs, one-trip bounds, named f64 positive,
undefined and non-f64 angle negatives remain covered.
CLI negatives exercise **32 commands** over both action kinds, wire/angle,
scalar/indexed forms, hash/check/run-action and t=0/1: exit 1, empty stdout,
§3.3 before execution. Both independently pinned positives check/hash and
execute forward/adjoint on both basis values, one wire and exact identity.
Full suite after this step: **344/0/5**.

## SOL-R11-B02 ≡ codex R11-B02 — resolved

Sections §§9.2, 16.1.

Hand derivation: a and a.b are distinct JSON keys although NAME admits dots.
Sol's @a has no interface regions: its envelope has only the seven
non-interface scalar layout lines. Codex's @a additionally has exactly
interface_regions.a.db {"base":16}. Moving a foreign base changes neither
@a envelope nor interface/closure hash. Moving its own base must change it.
Both complete @a envelopes are literal concat bytes in tests/identity.rs;
independent Python SHA-256 reproduces the supplied pins.

RED: identity **0/3**, both exact envelopes and dotted-name family; CLI
**3/1**, with three retained B01 passes and check's erroneous §9.2 rejection.
Sol's envelope incorrectly contained interface_regions.a.b.db; codex's
contained interface_regions.a.b.other.

Change: identity.rs::Object::envelope selects the exact JSON owner before
layout_leaves, retaining every other layout field. No dotted prefix test.

Witness tests:
- identity::r11_oracle_layout_ownership_uses_exact_names
- identity::r11_dotted_oracle_layout_ownership
- identity::r11_dotted_owner_and_region_family
- cli_xam::r11_cli_exact_oracle_layout_owners

GREEN **7/0**, including three retained B01 CLI tests.
Both member objects and sibling-base mutations check with literal pins,
interface AND closure hashes. Codex @a runs x=7, db=00, B=1 as (7,0,1).
Family: **6 accepted pinned objects** across three deeper owner/region pairs
and **3 own-base sensitivity controls**. Each pair includes the same
rendered path for owner a + region c.db versus owner a.c + region db,
catching flatten-before-filter overwrites as well as prefix absorption.
Full suite after this step: **348/0/5**.

Exact foreign-oracle pins:
- sol @a.b base 2: 9c6ed8677516f307a9957360c5328bc6df33d2263c2af75978064a7cbdc64863
- sol @a.b base 3: e3e41f7b25615cb9b97efaeff328cddff88ec8028777fd7d0745d2675eff85d5
- codex @a.b base 32: 26455a8ebeb96ca9ef90972709611ad6eed2bd2cb18f61aa368312e19edbd160
- codex @a.b base 48: 03ae1cd76644a677efd6819e42cf6ec74036c7aeeedbb431fffc89caa7aa441e

## SOL-R11-B03 ≡ codex R11-B03 — resolved

Sections §§3.1, 4.4, 4.5, 5.4, 14.2. Runtime semantics were correct.

Hand derivation 1: @o returns its input in one row. At B>=1 it is the
identity permutation: both qinplace rows are I, H² restores c=0 and
CX(0,b)=I on arbitrary bR. At B=0 every input faults with code 1, violating
qinplace's fault-free premise. Reject §4.5, never claim identity execution.
The exact council replacement is now in r9-sol-qinplace.comments and its
generator's sol-qinplace branch. Direct oracle_call fixtures retain their
valid same-token fault-XOR cancellation. All four R9 object bytes/pins remain.

Hand derivation 2: theta=6.283185307179586 is the exact binary64 rational
884279719003555/140737488355328, not symbolic 2*pi. At t=p=0 the rounded
H p; ctl p phase; H p circuit has amplitudes (1+exp(-i*theta/2))/2 on 00 and
(1-exp(-i*theta/2))/2 on 01. P(p=0)=cos(theta/4)^2 > 0, approximately
3.749399456654644e-33. Both support keys survive display rounding.
S S X S S X instead acts as X Z X Z=-I on both basis vectors. Controlled
minus-I is exactly Z on p, so H Z H takes p=0 to p=1.

RED: quantum_control **3/1**, fixtures **5/1**, aggregate **8/2**.
The stronger basis assertion failed with §14.2 (not a basis state);
qinplace's exact comment assertion displayed the false B=0 sentence.
The separate rounded support test already passed, confirming runtime behavior.

Changes: add @minus_i's six gates; controlled_phase calls it. Retain @phase
literal bytes/hash. Correct sidecars and generator; independently derive new
and dependent identity pins. Require read_basis(p)==1 and exact support
{01:(1,0)}. Add separate rounded support, both-basis forward/adjoint minus-I,
literal envelope and CLI pin tests. Correct the same misleading inline TAU
comment in quantum.rs, with two-support/non-basis assertions.

Witness tests:
- r1_fixtures::r11_qinplace_comment_matches_budget_contract
- quantum_control::qif_and_nested_control_preserve_interference
- quantum_control::r11_rounded_angle_retains_nonzero_interference_branch
- quantum_control::r11_exact_minus_i_on_both_basis_states
- r1_fixtures::r11_control_flow_pins_match_literal_envelopes
- cli_xam::r11_cli_rounded_and_exact_phase_probes
- retained r1_borrow::r9_qinplace_zero_budget_rejects_before_mutation and
  r9_qinplace_adjoint_restores_borrow_control

First GREEN **74/0**: 63 CLI + 4 quantum_control + 7 fixtures.
The later minus-I basis test and inline correction are in final **353/0/5**.
qinplace's sidecar test executes CLI B=0/B=1 at t=0/1, requiring
exit 1/empty stdout/§4.5 versus exact one-wire identity. The existing
independently initialized r9_identity helper checks all eight t,b,R basis
entries, superposition and Bell bR at B=1, both directions, reversal and
allocator/state equality with no escaping workspace.

Both codex standalone phase/probe/object triples reproduce exactly (table
below). Renaming the discrete action to minus_i changes its definition bytes:
H_minus_i=caec1fe67f4d082c9e4c9286aa43c12ffb3ba5baa75db05769bb451b8fbae443;
H_controlled_phase=9d9c349c97df952f56700426fc92bc1c1f7937d144f868f2f2287d7ec243f3cb.
The control_flow object hash is
e7e0acadaebf638a1adb34adb61efc2c79b43f0ec90b786582d2fdb6db7634d8.
All other existing control_flow symbol pins remain unchanged.

## Complete fixture-comment audit

| Sidecar | Independent hand check / executable coverage | Disposition |
|---|---|---|
| actions.comments | Copy/XOR/uncompute gives t XOR p; zero image, doubled stores/exchanges are identity; three Xs flip; H/CX Bell; lifted payload x XOR 1 in 3 rows (quantum, wide_oracle) | unchanged |
| arithmetic.comments | Arithmetic modulo 256; -7/2=-3 remainder -1; MIN/-1 fault 6; oversized shifts zero by §4.1; sext/shift/trunc 255; parallel swap; explicit branch/label and fault literals (oracle, step_machine) | unchanged |
| classical.comments | Triangular sum; inner clock segment undone once; five effect records, clock 1; absent sector restored; impure uncall, entropy, unknown-device, null/div0 faults; copied host reply truncation/full length (classical, cli_xam) | unchanged |
| control_flow.comments | Prefix flag conjunction/parity; conditional H/X amplitudes; exact six-gate XZXZ=-I; rounded θ dyadic and two nonzero interference branches; exchange, little-endian dead image, translation lemma (quantum_control, cli_xam) | corrected exact-vs-rounded claim; clarified bytes is the data symbol and dead its hex image |
| heap.comments | Base 32, 2-byte 0x1234→34 12; max(n,1) for zero alloc; pinned/free/null and capacity faults (oracle, step_machine) | unchanged |
| inc8.comments | 255+1 wraps; const/add/retval = 3 rows; B=0/2 budget faults (oracle, step_machine) | unchanged |
| mutual.comments | main has two rows, f next two, g starts at 4; main call + four g rows + retval = 6; name ordering distinct from hash ordering (identity) | unchanged |
| nested.comments | Three-row inc8; explicit fr test propagates code 9; paired XOR calls clear payload/fault even at B=0; ignore does not propagate (oracle) | unchanged |
| r9-codex-oracle.comments | Same-token self-adjoint XOR calls cancel at B=0/1; H² restores c=0; CX(0,b) identity on arbitrary bR (r1_borrow, r9_identity helper) | unchanged |
| r9-loop.comments | Each lifetime returns b under c=0 and replaces c by fresh zero; H² on t; no cancellation across invalid returns (r1_borrow) | unchanged |
| r9-sol-oracle.comments | Same direct-oracle XOR argument as codex fixture, including nonzero fault delivery (r1_borrow) | unchanged |
| r9-sol-qinplace.comments | One-row @o is identity at B>=1, but B=0 has code 1 and fails §4.5 before qinplace mutation; all t,b,R basis entries, superposition and Bell bR at B=1 (r1_fixtures, r1_borrow) | exact council replacement; matching generator branch corrected |
| registers.comments | Clean concat/views; two flips; dirty free traps; Bell 1/2 outcomes, lexicographic draws; measured segment cannot reverse (registers, cli_xam) | unchanged |
| sum_db.comments | 3+2+1=6; R_sum=3+3+1+9=16; n=16 needs 17*16=272; n=17 accesses db[16] (oracle) | unchanged |
| tiny_memory.comments | 1+8+1+4=14 interface bits; DIV0 row 4, db&1 row 8; B=9 idle; complete tuple reversal (step_machine) | unchanged |
| wide.comments | (2^100-5)(2^100+5)=2^200-25 leaves remainder 28; call/ret/retval 3 rows; 5-cell frame; high code bit 200 (wide_oracle, bitvectors, cli) | unchanged |

Generated comments: tests/support/r9_pins.py is the only Python sidecar writer.
Its sol-qinplace branch now emits the exact B>=1/B=0 distinction; direct
oracle-call cases retain valid fault-XOR cancellation and the loop branch
retains its per-lifetime derivation. r10_pins.py and r11_pins.py generate
envelopes/case records, not hand-result sidecars; both were checked for the
same class of claims. No further sidecar semantic error was found.

The same rounded-TAU-as-minus-I wording also occurred in an inline comment
in tests/quantum.rs::every_gate_and_controlled_global_phase. It now describes
the rounded residual and asserts two exact-support entries and non-basis
control, retaining the approximate diagnostic-amplitude comparison.
No runtime gate, budget, borrow or exact-state semantics changed for B03.

## Independent literal pin derivation

support/r11_pins.py uses only hashlib/json and literal function bytes:
ISA/version, ptr_width, JCS signature, LF function bytes, exact owner-selected
layout leaves; reference replacement and one dependency hash line per callee.
Objects have the canonical header, separators, stored-H symbols and functions.
It asserts every supplied member H/object pin before writing the corpus,
without interpreter calls or support::rehash. Rust compares the literal
envelopes, interface AND closure pins and object bytes. There are **16 R11
records**, including six dotted-family objects and two phase probes.
Regeneration is idempotent; all four R9 objects remain byte-identical.
No member pin correction was needed.

| Exact case | root interface = closure | object_hash |
|---|---|---|
| wire | 4a592df0f0a8651417bf00a48c5b75ad86060cf4a3b5d157a8c3bbce5443969d | c90b514421ef25e2cd27cd1b80039a39ca9a5c8d78173968ac4e5c25f7eb988a |
| angle | 544dfe0fb1b3e44d693ddb39967d5d5b043b9f92f4eb33e43b2d402a1d433ec6 | 918e9042f73b4f1f0cd72f9364e902aacc35e1cad3fcd8976ec9b0139cec2978 |
| producer | ed0795bb5a01b03715111e2fefe4bedb59a6869b3bdaab84f80fe7bc0e51fe05 | bef5f1ac1aa80ab0a4f523fe972f9227695b44f75e49c956acde273a9d7c225b |
| argument | 58e8b3a871bfc8167844e4175bbc7e6ef9cf17ccfbf1acd30b33800a49910c80 | 5d4101e0c48ea4074fd97cc9a70e869779518b2ff864ec9afbba548ebf48af45 |
| sol-layout-2 | 1b593b9027243623bfb18cc5b34c51d7df81d23526bc4987c14a5da2806b9521 | bccb8459c5f0746c49f98dad438d40a71e56d5e9533c8dc714bf32391c401877 |
| sol-layout-3 | 1b593b9027243623bfb18cc5b34c51d7df81d23526bc4987c14a5da2806b9521 | 80e8b4b4083a0cfa451b102cc9c13e40be118bc3fb6649d8572583a1768be05f |
| codex-layout-32 | 4671ca551ee57390d7f69d362e83180a1f3b29a3b8d2f45e2cd80962c6692aec | 1292bb07226f33217b884470c8e2c486bd4fb8b0f1eb4d9c829aaa75597cb4f4 |
| codex-layout-48 | 4671ca551ee57390d7f69d362e83180a1f3b29a3b8d2f45e2cd80962c6692aec | 9f818c01a67c28872cb3e28e5a4c4405616cd0c0d3510576110ab7b8d90ba0ab |
| rounded-probe | 0655c9e45f09d1748fbf0241e6d38244e7f353153a3f365f0a5f712b37a6e24d | 04baeadb96dbafb2dc94832982272b6cfeb60174de6c8bb407bd28a3e4c37294 |
| exact-probe | 0dcda336816f668d43eb75a083d0ee26574ccf854078a639de0a08a64f79009d | 0b71a29321bd566442de1d6e61774603031c3616a0b1cb4875b7719016837457 |

Rounded standalone H_phase=eb4b8c89a878e6fb8c514214417d0827f809fe0fd6b155e2f395ec19f5f31271;
discrete standalone H_phase=300340284f4927db0fc9ccf6b6285917606928b2926e62e015034eb0174e2713.

## Retained checks and observed verification

Every round 4–10 positive/negative remains in the passing normal suite:
physical frame boundaries, every-return independence, borrowed quantum touch
guards, certificate replay/binding, opaque LIFO histories, changed token/
input/control/version negatives, canonical effective controls, zero-trip
lifetimes, evolving bindings and exact-state cleanup/Born tests. No random
code, domain, seed or distribution changed.

Commands unset DISPLAY, WAYLAND_DISPLAY, DBUS_SESSION_BUS_ADDRESS, BROWSER;
TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp.
All tests set CARGO_PROFILE_TEST_OPT_LEVEL=1 (same assertions/domains/overflow
checks). After that environment prefix, exact gate commands are:

~~~sh
cargo test --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --offline
cargo test --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --offline -- --ignored --nocapture
cargo fmt --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml -- --check
cargo clippy --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --offline --all-targets -- -D warnings
python3 /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/tests/support/r11_pins.py
python3 /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/tests/support/r9_pins.py
~~~

Development filters used the same manifest/environment: parser + cli_xam
r11_ (B01); identity + cli_xam r11_ (B02), --no-fail-fast on RED;
r1_fixtures + quantum_control (B03 RED), then those plus cli_xam (GREEN).
Test-source setup errors (Rust quoting, OracleInput field name) were fixed
before semantic RED and are not counted as reproduced blockers.
A Python generation syntax error changed no files. An initial commit attempt
lacked git add for new paths; explicit add/commit lists fixed it. No foreign
path was staged/committed.

| Run | Observed result / summed runner time |
|---|---|
| baseline | 338/0/5; 8 unit + 330 integration; 15.02 s |
| B01 initial combined RED | 1 passed / 4 failed; 0.03 s |
| B01 separate exact angle CLI RED | 0 passed / 1 failed; 0.00 s |
| B01 targeted GREEN | 6/0; 0.10 s |
| B01 full suite | 344/0/5; 15.28 s |
| B02 combined RED | 3 passed / 4 failed; 0.03 s |
| B02 targeted GREEN | 7/0; 0.04 s |
| B02 full suite | 348/0/5; 15.13 s |
| B03 combined RED | 8 passed / 2 failed; 0.14 s |
| B03 first targeted GREEN | 74/0; 12.77 s |
| final normal suite | 353/0/5; 8 unit + 345 integration; 0 doctests; 15.36 s |
| fmt --check | pass; 0 tests |
| strict all-target Clippy | pass; 0 warnings, 0 tests; 1.12 s |
| independent generators | 16 R11 cases; all pins match; four R9 objects unchanged; idempotent |
| owned diff --check | clean |

All **5 ignored sweeps passed / 0 failed**, 94.46 s runner sum:

| Sweep | Full domain | Observed result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 pairs × 4 = 262,144 executions | 1/0; 0.51 s |
| 17-wire borrow | 131,072 entries | 1/0; 2.74 s |
| checked borrow certificate | 262,144 elements, subject and expected program | 1/0; 14.76 s |
| §6 differential | 200,000 programs: 70,931 accepted (all clean), 129,069 rejected | 1/0; 73.97 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 1/0; 2.48 s |

The accepted/rejected counts equal round 10 with unchanged RNG/seeds.
Fifteen new tests take the normal suite 338→344→348→353; no existing test
was removed or weakened. Tracking b-isa-spec-x02 is closed.

Logs: bennett-isa-spec/.tmp/r11-{baseline,b01-red,b01-angle-cli-red,
b01-green,b01-full,b02-red,b02-green,b02-full,b03-red,b03-green,
b03-full,pins,r9-regenerate,final-fmt,final-clippy,final-sweeps}.log.
No semantic source changed after the normal gate or sweep start.

## ISA 0.8 gaps and disposition

No new ISA 0.8 gap, ambiguity, decline or partial blocker.
**Exact spec change needed: none.** SPEC-QUESTIONS is unchanged. Existing
deliverables 8–11 and performance follow-up b-isa-spec-436 retain disclosed
staging. This response claims no completion of that staged work or an
independent council-wide audit. No north-star capability was narrowed.
Accepted spec SHA-256 remains
90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e.
Explicit owned path commits all use BEADS_HOOK_TIMEOUT=20. No push or Dolt
remote sync. Beads' automatic metadata export remains outside owned commit
paths and is left to the maintainer.

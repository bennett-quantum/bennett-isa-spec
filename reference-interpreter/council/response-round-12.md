# Editor response: council round 12

Both S0 blockers are resolved: SOL-R12-B01 ≡ codex R12-B01 (contract) and
codex R12-B02 (core). None declined or partial. Sole implementation sub-agent;
read the binding round-12 and previous-round briefs, both members' full report
and JSON, and interpreter README/PROGRESS/SPEC-QUESTIONS/BRIEF. Implementation
commits: **76714aa** (B01), **054bb7e** (B02). Final verification/evidence commit: **b76c6a9**. Only owned interpreter paths and this response are committed; no push.
Spec/conformance/include, Python seed, dependencies and RNG/seeds are untouched.
Only the crate's own tests ran; the concurrent red conformance test did not.

## Design

1. A layout leaf carries original JSON key components, rendered path and value.
2. The rendered dotted path never serves as unique structural identity.
3. CC/R retain every layout leaf, including coincident renderings.
4. Oracle selection uses the exact original owner key, preserving round 11.
5. Lprocs select exact referenced global keys and their images, once per leaf.
6. Action/data closure exclusions and image collection remain unchanged.
7. Sort by rendered path bytewise, then original components component-wise bytewise.
8. The tie order is independent of values; dots remain legal at every depth.
9. Graph construction and envelope replacement share contextual token classification.
10. Only lift.addr naming the current oracle's declared interface stays literal.
11. Calls, qregion images and actual globals retain ordinary symbol dependencies.
12. Symbol/import spelling alone imposes no interface-name ban; the separate existing
    actual-global/interface ambiguity is recorded narrowly in SPEC-QUESTIONS 32.

No quantum, budget, borrow, lifetime, causal-history or execution semantics were
changed. All round 4–12 positives and negatives are retained. No cap, frame
waiver, fallback, default address or narrowing of north-star scope was added.

## SOL-R12-B01 ≡ codex R12-B01 — resolved

Sections §§9.2, 16.1. Distinct structured paths must survive even when their
dotted renderings coincide. For codex's equal-valued object, both original
leaves produce `layout interface_regions.a.b.c {"base":16}`. These two lines
must occur, regardless of a secondary-order reading. For sol's deep-a-32 plus
@main, original owner a precedes a.c, so the base-16 line precedes base-32.
Changing a/c.db to 17 must affect CC/R while preserving the foreign oracle.

RED tests (written before implementation):
- identity::r12_cc_r_colliding_dotted_leaves
- identity::r12_cc_r_layout_paths_preserve_colliding_leaves
- identity::r12_dotted_leaf_collision_family
- cli_xam::r12_cli_checks_lossless_layout_envelopes

Combined RED: **0 passed / 4 failed**. Exact envelopes omitted a leaf;
CLI hash returned the wrong @main interface/closure pin. Tests use full
LF-terminated literal envelopes and independent stored pins, never
support::rehash. Independent support/r12_pins.py hand-assembles each oracle
and CC/R envelope with hashlib/json only and asserts every supplied pin.
All six exact objects check successfully after the fix, including both tiers
and sol's base-16/base-17 objects. Oracle pins remain independently correct.

Change: src/identity.rs::layout_leaves returns a lossless sequence of LayoutLeaf;
Object::envelope selects by original keys and serializes every selected leaf.
Lproc lookup and image collection consume that representation. No dotted-prefix
ownership inference, rendered-path deduplication or name ban remains.

The 30-object deterministic family covers three dotted owner depths, both
CC/R tiers, equal values, ascending and descending unequal values, and each
leaf changed independently. It asserts tie order, own-leaf sensitivity and
foreign-owner invariance. Round-11 exact-owner tests remain unchanged. Added
identity::r12_dotted_global_lproc_keeps_exact_closure_selection retains a
noncolliding dotted global's leaf and image without unrelated interface leaves;
actions and data still exclude layout. The added control initially moved db
into the heap; its test address was corrected to a disjoint range. That setup
failure is not counted as a reproduced blocker. Full B01 gate: **358/0/5**.

| Exact case | @main interface = closure | object_hash |
|---|---|---|
| codex CC | a298a1060aa5dcf96e3e0d1c0d5251e6ad1acd44c776cf88caa9d25b6981c7bd | d478a37dbfa4895d6ba5f9673be187a4f5b9d75d6f38a11439208af9cddd0bc6 |
| codex R | 8247f2deedc359e973521302ed25b78a21eadb8c169d01b60b3ee8d4030c48a3 | d955f72740664092cdbfcf9c5c6541c9606895cfd17b9329ebc8db82e20091e4 |
| sol CC base 16 | 84b403e0032cfdc98e4c33e628c9662043833876045378f733aaaba7d311679e | c7a1750c6e3415cb18ae1d699ba36067f0a382a93b6e409252df9bf0f1ee7f5d |
| sol CC base 17 | 69d94d8f25dae1444c6ac6b3f46fe0e9bec110393405e778a84415913ee47bff | 483ad27610edeecc63993cb5891df642ec36f035920250f3ddcadefe7e6d814e |
| sol R base 16 | 68943b7ebec4b123a9b0aed04af75e1e037545794a3dee8a3aaceb7b623d421a | 98eb9703872f2290f16b7db139167b70ef0530a9d24376ac2d0f9a83ffb843cf |
| sol R base 17 | 54af133bf40f7db2ca5da434db35f34f2157ae1808b46ed80137481e4a9efc17 | 6f3b791cb5e31eefd6f26c79248ecdaaf6c89809db659c5d12dc7d7cd592259a |

The oracle pins, full envelopes and all object hashes are checked in library
and CLI assertions. The two R object hashes not supplied by sol are derived
from canonical stored-H object bytes by the independent generator.

## Codex R12-B02 — resolved

Sections §§4.2, 9.2; rule 10. The function symbol @o cannot supply a global
address: the exact object has no globals, and its current oracle declares a
region o. Thus `p = lift.addr @o` writes 16 and stays literal in the envelope.
It contributes neither a graph edge, an @$0 replacement, a recursive :0 hash,
nor a part-(6) reference line. At B=2 retval returns 16 with code 0; at B=1
only addr runs, then budget exhaustion gives payload 0/code 1/steps 1.

RED tests:
- identity::r12_interface_address_is_not_a_function_dependency
- r1_linking::r12_interface_region_may_share_function_name
- cli_xam::r12_cli_interface_region_may_share_function_name

Initial combined RED: **0 passed / 3 failed**, all at the §9.2 namespace ban.
Deleting only the ban was independently replayed: **0/1**, exact envelope
failed with `p = lift.addr @$0`. This confirms contextual handling is necessary.

Change: src/object.rs::Object::validate_symbols removes the symbol/import
restriction. src/identity.rs::symbol_references classifies the sole lift.addr
operand in the current oracle's canonical instruction context; both identity's
graph and envelope replacement call it. Ordinary call, image and global
references retain replacement and dependency lines. Only the separate existing
actual layout.globals ambiguity rejection remains, as expressly allowed in the
member's fix; SPEC-QUESTIONS 32 records its narrow reading and proposed wording.

Both supplied objects match literal full envelopes, interface AND closure
hashes, and stored-H object hashes. support/r12_address_pins.py independently
asserts the two complete member derivations before producing the corpus:

| Case | interface = closure | object_hash |
|---|---|---|
| local o | 05827c10c37afc2feef8ccdf0d26dbe25c92204d37d732db954e8595e0d22e91 | 44158d3308861c21d1a001fdf3b8d80b5218f281580322e6406e97f22829b536 |
| local db control | cb28a9580045f28923bf1c5fc328d7b8e89038e3b0294afc1b1a9291d7e3b5e1 | 94499ff501bc706735df0b948fee0b7ed89f6f615f09d1a94934b940beb10755 |

CLI hash/check succeed. Both library and CLI return exactly
`{"code":0,"payload":16,"steps":2}` at 2,0,0,0 and
`{"code":1,"payload":0,"steps":1}` at 1,0,0,0, with empty input and
`--mem o=00` (db=00 for the control). CLI reverse at both budgets and for
both names reports REVERSED, equals_entry=true and frames_clean=true.

Additional tests:
- identity::r12_interface_symbol_kind_family: all seven symbol kinds, each
  defined and imported, 14 admitted overlaps. Even an unpinned import does
  not enter @o's interface/closure dependency set through a local address.
- identity::r12_call_operand_stays_a_dependency_beside_local_address: the
  same spelling in lift.call is replaced/listed; two callee mutations change
  root identity while leaving lift.addr literal. This guards against erasing
  real references just because a local region shares their spelling.
- identity::r12_global_and_qregion_operands_remain_symbol_references: global
  addr and qregion image dependencies remain ordinary data references; the
  non-local global case returns base 24 in two rows.
- r1_linking::r12_non_declaring_oracle_cannot_resolve_a_foreign_interface:
  @other's lift.addr @o rejects §4.2 even though @o declares that region.
- The existing interface_region_names_cannot_alias_symbols global-collision
  negative remains unchanged and passes; its former blanket-ban interpretation
  is replaced by SPEC-QUESTIONS 32's narrow actual-global ambiguity.

Initial family expansion found a missing CC/R foreign-owner layout line in
an independently generated extra symbol envelope; corrected that expectation,
not implementation. No supplied member pin correction was needed. Final
combined R12 targeted gate: **12/0**. Full crate gate: **365/0/5**.

## SPEC-QUESTIONS and ISA 0.8 gaps

Entry 31 flags the equal-rendering tie order for the 0.9 draft, with sol's
exact proposed §9.2 wording:

> Emit one line for every structured layout leaf, including leaves with coincident dotted renderings. Sort by the rendered dotted path bytewise; break ties bytewise by the sequence of original JSON key components.

Entry 32 records only the separate global/interface precedence ambiguity,
with codex's exact proposed 0.9 wording:

> An oracle interface region must not share a name with a global in layout.globals; it may share a name with a function symbol.

No accepted spec edit. No new rule-10 blocker or scope restriction; no Beads
issues created or closed, so no issue export was required. Existing staged
whole-ISA deliverables remain staged and are not claimed complete here.

## Retained checks and observed verification

Every round 4–12 positive and negative remains: complete per-return/frame
borrow equations and certificate premises, quantum borrowed-touch guards,
zero-trip and changing-width lifetimes, nested/all-operand opaque inverse
histories, changed input/token/polarity/version negatives, canonical controls,
exact-state/Born support and round-11 fixture sidecars. RNG/seeds unchanged.
The 56 independent corpus records (36 B01, 20 B02) regenerate byte-identically.
No implementation change followed the final normal gate or sweep start.

Commands run with DISPLAY, WAYLAND_DISPLAY, DBUS_SESSION_BUS_ADDRESS and BROWSER
unset and TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp.
The exact normal and sweep commands (test optimization level 1 retains
assertions, overflow checks and domains; baseline used the default profile):

~~~sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp CARGO_PROFILE_TEST_OPT_LEVEL=1 cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml -- --ignored --nocapture
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp cargo fmt --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml -- --check
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp cargo clippy --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml --all-targets -- -D warnings
~~~

Development filters used the same env/manifest: identity + cli_xam r12_
(B01); identity + r1_linking r12_interface and separate cli_xam
r12_cli_interface (B02 RED); identity r12_interface_address (guard-only RED);
identity + r1_linking + cli_xam r12_ (GREEN and families). Python generators
used the same headless prefix and repository TMPDIR.

| Run | Observed result |
|---|---|
| baseline, summed over every binary | **353 passed / 0 failed / 5 ignored** (8 unit + 345 integration) |
| B01 semantic RED | **0 passed / 4 failed**; 0.01 s runner sum |
| B01 intermediate added-control setup failure | 4 passed / 1 failed; 0.39 s; first full attempt then stopped at identity 15/1 |
| B01 corrected full GREEN | **358 passed / 0 failed / 5 ignored**; 37.76 s |
| B02 initial library RED / separate CLI RED | 0/2 / 0/1; aggregate **0/3** |
| B02 guard-only graph RED | **0/1** |
| B02 first combined GREEN | **8/0**; 0.15 s |
| initial extended family expectation error | 8/1 before correcting extra-symbol CC/R envelope |
| final R12 targeted GREEN | **12/0**; 0.17 s |
| final full normal suite | **365 passed / 0 failed / 5 ignored** (8 unit + 357 integration, 0 doctests); 36.27 s |
| fmt --check | pass; 0 tests |
| strict all-target Clippy | pass; 0 warnings, 0 tests; 2.52 s |
| independent generators | 56 records; every supplied pin matches; byte-identical regeneration |
| owned diff --check | clean |

All **5 ignored sweeps passed / 0 failed**, 168.57 s summed runner time.

| Sweep | Full domain | Observed result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 operand pairs × 4 = 262,144 executions | 1/0; 0.89 s |
| 17-wire borrow | 131,072 entries | 1/0; 4.73 s |
| checked borrow certificate | 262,144 elements, subject and expected program | 1/0; 26.83 s |
| §6 differential | 200,000 programs: 70,931 accepted (all clean), 129,069 rejected | 1/0; 131.76 s |
| recursive prefix reversal | 256 inputs × every prefix through completion/first fault | 1/0; 4.36 s |


Historical arithmetic-label suggestion applied to PROGRESS's round-1/2/3
sweep tables and council/response-round-2.md: 65,536 operand pairs × four
operations = 262,144 executions. This changes no tested domain or result.
All other historical evidence is retained.

Logs: repository .tmp/r12-{b01-red,b01-green,b01-full,b02-red,b02-cli-red,
b02-graph-red,b02-green,b02-family,b02-full,clippy,sweeps,pins,address-pins}.log.
All commits use explicit owned path lists and BEADS_HOOK_TIMEOUT=20. No push,
Dolt sync, conformance execution or foreign path edit.

# K82: binding the ref@0.9 helper recipes

2026-11-01. Stream opus-isa-helpers, for `b-isa-spec-g2q` and `bennett-sched-enz`. **Proposal only**: `spec/` is unchanged and the council has not seen this.

## Verdict

This is a spec defect. Existing text does not bind these helpers; read literally, it requires consumers to reject the published catalog. The recommended fix is a new rule, K82, that changes no catalog byte and keeps hash `5de4e499...d197`.

## What the text says

- §11.1 K19 (spec lines 2924, 2984-2990): `invoke` binds `args` positionally, consumers MUST reject an unbound name, and "There are no implicit bindings or defaults."
- The catalog: `value.add(N)` reads `a`, `b`, `t` and `bind.value.add` invokes it with only `[N]`. So a strictly lexical consumer rejects it, and the sched test `undeclared_helper_operands_are_reported_with_the_contract_gap` records exactly that.
- K77 to K81 do not help. K77 covers only the oracle token channel and says it is "not an implicit name". K78 to K81 cover entry analysis, empty addresses and cleanup.
- §17 item 7 and K57 only choose the library and its hash.
- Two passages show the intent without binding it: the ISA-REF-LIB witness treats `s`, `t` as the interface of an integer-only `qxor`, and K20 speaks of "recipe interface wires". Neither says how `invoke` supplies those names.

## What the catalog and consumers actually do

| Item | Finding | How established |
|---|---|---|
| Recipes with free names | 54 of 219, exactly those whose formals are all bare names or empty | `generate.py` for the 54; a heuristic sweep found no free name in the other 165 |
| `invoke` sites into the 54 | 20: 17 `bind.value.*`, plus `lift.sext`, `lift.trunc`, `lift.zext`; all bind each name as an identically named wire formal | `generate.py` asserts this |
| The other 34 | Never invoked; reachable only by direct instantiation (historical helpers, `qxor`, `gate.*`, and so on) | `generate.py` |
| Reference `ref_language.rs` | A recipe with all-bare formals imports all caller wires by name (`c.legacy`, `bind_seeded`); no other recipe imports anything | code read and probe |
| VM `src/ref_library.rs` | `bind_formals` starts from `capture.clone()`, so **every** recipe inherits **every** caller binding, typed ones and non-wires included | code read only; VM not run |
| Sched `src/expansion.rs` | Strictly lexical, so it rejects the catalog | code read and its own test |

Consumers therefore disagree today. The reference and VM accept the catalog by two different undocumented rules, and sched rejects it. The VM rule should give the same leaves on this catalog, since the typed recipes have no free names, but it would accept a typed recipe with a free name. That is inferred, not executed.

## Why a declared width is not part of the rule

Exact widths cannot be inferred from what a body reads: `value.trunc` reads `M` bits of an `N`-bit `a`, `value.udiv` reads 3 bits of a 4-bit `f`, and `value.zext` reads `N` bits of an `M`-bit `t`. K82 therefore range-checks reads and leaves equal-width checks to the typed formals of the invoking recipe. All 20 sites have such formals. A direct instantiation of one of the 34 has no such check beyond its explicit name map: a too-wide list is accepted and its extra wires go unread.

## Options

| Option | Catalog hash | Assessment |
|---|---|---|
| **K82, named interface (recommended)** | Unchanged | Sound on all 20 sites, closes every other recipe, and matches the leaves the reference already emits. It is an explicit rule, not a guessed fallback. |
| Rewrite 54 recipes with typed formals (the bead's proposal) | Changes | Cleaner language, same leaves, but it re-pins every consumer below and invalidates published schedule and certificate bytes in exchange for no semantic gain. |
| Keep the text, have consumers adopt the reference behaviour | Unchanged | Rejected: it leaves the contract contradicting itself. |

## Hash impact per consumer

K82 needs no re-pin anywhere. The table shows what the rejected rewrite would have to re-pin.

| Consumer | Pins | Change needed under K82 |
|---|---|---|
| ISA spec | §11.1 text (2 places), `spec/ref-0.9.json` | The diff below; regenerate `spec/isa-0.8-to-0.9.diff` |
| Reference | `reference-interpreter/src/ref_library.rs` | `b-isa-spec-b5b` (import exactly the interface) |
| Conformance in this repo | `controlled-action-09` (`schedule.json`, `controlled.bisa`, README, tool, test), `draft_witnesses.py` | None |
| bennett-vm | `SPEC_VERSIONS.toml`, `src/ref_library.rs`, `tests/ref09.rs` | Close typed recipes; import only interface wires |
| bennett-sched | `SPEC_VERSIONS.toml`, `src/schedule.rs`, `tests/canonical_cost.rs` | Implement K82; replace the test named above; then close `bennett-sched-enz` |
| bennett-targets | `docs/verification.json` | None |
| bennett-estimate | `SPEC_VERSIONS.toml`, `src/estimate.py`, `tests/test_estimate.py` | None known; its expansion was not read |
| bennett-conformance | `checker/src/cert.rs`, `checker/tests/core_witnesses.rs`, `clauses/ISA-0.9*` | Regenerate the clause files after the spec edit; K82 adds clauses |

## Evidence

Vector: `conformance/fixtures/pending-k82-09/` (see its README to reproduce).

- `generate.py --check`: OK, with 54 recipes, 20 edges, 86 instances and 16 cases; it also asserts the published envelope hash.
- Probe against the reference expander: 86/86 edge digests match an independent Python expansion that imports only the interface.
- Binding cases: 14/16 pass, and each `reject` case was confirmed to fail for the intended reason. The two failures are the real divergences in `b-isa-spec-b5b`:
  - `callee-local-is-not-captured-by-an-unrelated-caller-name`: `allocation shadows live wire z`
  - `live-declaration-binds-by-its-wire-actual-rule`: `unbound wire q`

Not done: the VM and sched were not run against the vector, the repo gate was not run because no gated file changed, and the sweep over the 165 typed recipes is a heuristic. Edge tuples cover widths 1 to 4.

## Follow-up beads for the orchestrator to file

This stream owns only this repo's `.beads`.

- **bennett-vm**: after K82, `bind_formals` must start each typed recipe from an empty environment and import only interface wires for named-interface recipes. Vector case `typed-recipe-is-closed` is expected to fail today (from reading the code).
- **bennett-sched** (under `bennett-sched-enz`): implement K82 in `invoke`, replace `undeclared_helper_operands_are_reported_with_the_contract_gap`, and pass the vector.

## To apply after acceptance

1. Apply the diff below with `git apply`.
2. Rename the fixture to `accepted-k82-09` and update its README status line and the `--manifest-path` and `k82.json` paths in it.
3. Regenerate `spec/isa-0.8-to-0.9.diff` with the command in `conformance/tests/draft_witnesses.py`.
4. Add the round 10 acceptance line, change "K1–K81" to "K1–K82", and drop "pending round 10" from the row.
5. Close `b-isa-spec-g2q`; `b-isa-spec-b5b` and the two follow-ups then proceed.

## Exact diff

```diff
--- a/spec/Bennett-ISA-Spec-0.9.md
+++ b/spec/Bennett-ISA-Spec-0.9.md
@@ -207,6 +207,7 @@
 | K79 | §4.6, §16.2 | **COHERENT-EMPTY-ADDRESS**. Coherent memory rows bind exactly A address wires; A=0 has the address-only source actual `()`, bound to the catalog's empty wire list. Ordinary wire values remain nonempty. Witness: qload/qunload/qstore/qexchm on region[0,1] select cell 0; nonempty A=0 and empty A>0 addresses reject. | consumer 7js; pass IX | S | – |
 | K80 | §11 | **CLEANUP-LEAF-OWNERSHIP**. Reverse retains written cleanup points; recompute owns one fresh caller leaf/call space at the carrier, with separate callee spaces and fresh replay identities. Witness: a six-row two-CX computation/cleanup has reverse leaves at entry:3/4/5 and recompute leaves at entry:5 ordinals 0–7. | consumer 5cd; pass X | S | – |
 | K81 | §11 | **CLEANUP-PAIR-PARTITION**. Intrinsic cleanup pairs retain their direct computing operations; release-ended terminal inverse chains coalesce before independent adjoint-call carriers are formed. No inverse occurrence has two owners. Nonzero call targets retain their exact source-derived entry history under recompute. Witness: qcopy/mix/adj-mix/quncopy has carriers at entry:2 and entry:3, independently selectable. | consumer 9hv; pass XII; pending round 9 | S | – |
+| K82 | §11.1, §15 | **NAMED-INTERFACE-RECIPES**. A recipe whose formals are all bare integer names, or none, reads its remaining wires by name from the invoking scope; every other recipe and every template is closed. No catalog byte or hash changes. Witness: bind.value.add(2,x,y,z) expands value.add over x,y,z; a typed recipe reading q, or a site with no q bound, rejects. | b-isa-spec-g2q; bennett-sched-enz; pass XIII; pending round 10 | S | – |
 
 **Decided Q14 (applied 2026-10-29, confirmed 2026-10-30; b-isa-spec-7u6).** Lprocs take explicit
 token and classical parameters (§3.7, §4.2, §4.7, §16.2), forwarded by
@@ -2987,12 +2988,45 @@
 coincident target/control, unmatched allocation/release, or cyclic
 `invoke`/symbol expansion. Static integer bounds must yield finite
 expansion; validation is repeated for every actual parameter tuple before
-its expansion can be used. There are no implicit bindings or defaults.
+its expansion can be used. There are no implicit bindings or defaults;
+K82 below is the only by-name binding, and it is explicit.
 Acyclicity concerns declared
 recipe/symbol dependencies, not repetition of a higher-order recipe name
 inside finite captured actual syntax: qloop(3, qloop(2, B)) expands to six
 copies of B. Consumers expand finite actual bodies to a constructor fixpoint.
 
+**Named-interface recipes (K82).** A recipe whose `parameters` are all
+bare names, including an empty list, is a named-interface recipe. Its
+named interface is every name its body uses as a wire before any lexical
+binding of that name, in body order, plus the named interface of each
+named-interface recipe it invokes, less the names bound at that invoke.
+Every other recipe, and every K22 template even with bare formals, is
+closed: a name it leaves unbound rejects.
+An `invoke` of a named-interface recipe evaluates each interface name as
+a wire actual in the invoke's lexical environment, under the same name,
+when it evaluates `args`. K26 and K27 apply as for any wire actual, and an
+invoke inside a captured body resolves in that capture. A name with no
+wire binding there rejects. Nothing else is imported: an integer, body or
+other non-wire name stays unbound in the callee, and any other caller name
+is invisible to it. The callee's allocations belong to its own scope, so
+they do not shadow an unrelated caller name.
+An interface name has no declared width. Each subscript must lie within
+the bound list, and a whole-name use takes the entire list under the
+gate's arity check; the bound list may be longer than the body reads.
+Equal-width checking belongs to the typed formals of the invoking recipe.
+Instantiating such a recipe other than by `invoke` requires an explicit
+map from exactly its interface names to wire lists. A body that releases
+an interface name still requires its matching live allocation.
+In `ref` version 0.9, 54 recipes have this form. Exactly 20 `invoke` sites
+reach them, each from a typed recipe whose wire formals carry the identical
+names, and none from another named-interface recipe. This rule changes no
+catalog byte, leaf, ordinal or hash.
+Witness: `bind.value.add(2,x,y,z)` binds typed formals a, b, t to x, y, z;
+its `invoke` of `value.add(2)` then reads those same three lists, emitting
+the leaves of `value.add` over x, y, z. A typed recipe reading a formal it
+does not declare rejects, as does an invoke with no `q` in scope of a recipe
+reading `q`.
+
 **Image binding (K21).** An `image` actual is a bound image formal or
 an even-length string of lowercase hexadecimal byte pairs in data-image
 byte order. The empty string denotes the absent/default-zero image,
@@ -3978,6 +4012,10 @@
 
 ## 15. Conformance suites
 
+The K82 named-interface vector is published in
+`conformance/fixtures/accepted-k82-09/`: the catalog's interfaces and invoke
+edges, expanded leaf digests for those edges and accept/reject binding cases.
+
 The K81 joint-operation / first-release vector is published in
 `conformance/fixtures/accepted-k81-09/`: both release orders, indivisible
 inverse ownership and a duplicate-maximal cleanup selector tie. Its source
```

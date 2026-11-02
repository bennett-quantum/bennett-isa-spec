# F05/F19 — budget identity belongs to the device invocation

Beads: **b-isa-spec-c1u**, **b-dialect-spec-bly**. One coordinated budget-scope amendment. Amendment proposal, never a prerequisite for rungs 3/4; F30 remains deferred until rung 3 or 4 lands.

## Exact defect and clause citations

[ISA 0.9](../../../spec/Bennett-ISA-Spec-0.9.md) §§4.7/5.4/10.1 already bind region calls, tuples, tokens and estimates to device execution. §12.3 serializes token tuples for replay, not live token identities. [Dialect 0.2.12](../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md) and [0.3](../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md) §§9.1–9.5/19.9 resolve shape tables, selection/final identities and frozen capacities but lack their mixed-container association. Applying one process-global B or equating tokens from equal tuples would break existing semantics. [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F05 is neutral; F19 needs association.

## North Star gap

No missing budget capability; scope and identity need publication for selective modules (North Star §§3.3/4.2/4.4). Native work is not a new oracle instruction unit. Device replay, helper and wrapper work must remain charged.

## Alternatives and witnesses

Native main launches P.root at `(3,0,0,0)` and Q.root at `(5,0,0,0)`. P's nested oracle can independently use `(2,0,0,0)`. A process B=5 erases P's exact invocation; resetting nested counters at a shim boundary changes faults. Tokens minted twice for the same tuple are distinct live identities, even though §12.3 intentionally gives the tuples equal params hashes. Prefer exact per-call records, with that identity/value distinction. [verify.py](verify.py) checks these equal-hash/distinct-token and nested-binding witnesses; it does not execute an estimator.

## Recommended exact ISA/dialect diff

Append to ISA §§5.4/10.1:

```text
Budgets/estimates bind selected device invocations; a native process has
no implicit global B. A mixed invocation record identifies program/local
entry, region closure, variant/control count, target/schedule/expansion,
resolved arguments and exact four-component tuple. Nested calls retain
their own resolved tuples and token arguments; action_call has no invented
primary oracle token. Program/run ownership and identity of a live token
are distinct from tuple equality and from canonical replay parameters.
```

Append to both dialect tracks §§9.2/9.5/19.9:

```text
Resolve the budget table and freeze layout capacities for the selected
complete device link, using every required shape. Mixed records retain
the source selection identity, final semantic identity, resolved static,
host-supplied or device-minted tuple, frozen capacities and table digest,
and bind them to the outer program/entry/root/build/layout association.
Capacity bounds are not rounded replacement invocation tuples. Device
snapshot/replay, helper and wrapper rows retain their published units;
crossing a native adapter does not reset an active invocation's counters.
Native-only heap/stack/libc and registration/shim overhead are reported
separately, not silently added to or subtracted from device budget units.
Equal numeric tuples in separate programs never merge live token identity.
```

Use [container-and-certificates](container-and-certificates.md)'s budget-record reference and [resource-routing](resource-routing.md)'s ledger owner; do not replace the existing resolved-table protocol with a second incompatible schema.

## Hashes, catalog and consumers

Table/build/layout/source-selection and final object identities remain distinct. Runtime token ids are not serialized proof identities. Different arguments/tuples may change params_hash/schedule/cert applicability; changed frozen layouts change objects. Adding outer references alone leaves inner bytes and ref catalog unchanged. Driver/plugin/VM/scheduler/estimator/checkers must retain exact bindings and repin only accepted publications.

## Conformance vectors and verification

Require two roots/different tuples; equal tuples/different programs and token ids; nested token tuple different from parent; host versus device mint; action calls; shape maxima distinct from actual tuple; B=0; retained replay/helper costs; params_hash with equal tuple values but distinct ids; wrong root/table/build rejection. Actual finite checks are in [INDEX](INDEX.md); no consumer gate or budget-table implementation changed.

## K entry: 0.9 versus 0.10

Scope text clarifies existing ISA semantics; outer records are the new companion plus next dialect revision. Default normative integration is **0.10**. Any in-version ISA K route needs **council + maintainer authorization of compatibility rules**, especially if a new binding field is required. No ref catalog change. Proposed decision `MIXED-INVOCATION-BUDGET-SCOPE`.

## Mixed fat object

Apply [fat-object-scope.md](../2026-11-02-rev2/fat-object-scope.md) and [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F05/F19. Marked device roots retain separate certificates/context `[]`, ledgers and tapes; native main/argv/envp/libc acquire no fake ISA budget or proof record. Container entry/layout and future memory/launch profiles supply associations without whole-binary lowering. These amendments never gate rungs 3/4; F30 stays deferred.

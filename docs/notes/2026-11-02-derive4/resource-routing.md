# F06/F13 — opaque host transport with explicit ledger ownership

Beads: **b-isa-spec-ml0**, **b-dialect-spec-jwd**. One routing/lifetime amendment. Amendment proposal, never a prerequisite for rungs 3/4; F30 remains deferred until rung 3 or 4 lands.

## Exact defect and clause citations

[ISA 0.9](../../../spec/Bennett-ISA-Spec-0.9.md) §§3.3/5.4/10 scope register ids/tokens to a program/run and retain exclusive touched-resource ownership through CLOSED. [Dialect 0.2.12](../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md) and [0.3](../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md) §§6.1/6.4/6.6 exclude source handles from ordinary host use while permitting narrow analysed launch-id transport. There is no mixed namespace/generation routing record. A pointer-width source tag is not the full 64-bit ISA id, nor is either a native data pointer. [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F06/F13.

## North Star gap

Association/lifetime mechanics, not a reason to relax North Star §3.1's device-resident handle operations. Cross-program quantum sharing remains a capability obligation; co-location in a native binary does not create a shared ledger.

## Alternatives and witnesses

Programs P and Q can both issue numeric register id 7. Sending bare 7 to Q can resolve the wrong register. Unloading P and loading a new instance can reuse a host address without reviving P's old token. On a 32-bit host, truncating a 64-bit ISA id loses information. Prefer a runtime-owned opaque routing record; communicate quantum modules now through one actually linked program, or later through a ratified ownership-transfer protocol. [verify.py](verify.py) checks wrong-owner/stale generations and equal numeric ids; it does not implement quantum state transfer.

## Recommended exact ISA/dialect diff

Append to ISA §10 and dialect §6.6:

```text
A mixed host-visible resource capability resolves through a runtime-owned
record containing loaded program instance/generation, ledger kind, full
ISA id, and validity/owner state. Native pointers and source pointer-sized
handle tags are not ISA ids. A host token is opaque: transport/storage by
the analysed adapter grants no dereference, arithmetic, gate execution,
measurement-dependent native control or unknown-reader escape permission.
BD-E-HANDLE-ESCAPE/USE and device-resident qreg tables remain in force.

Resolve and validate the owning program/generation before looking up an
id, then enforce all existing touched-run ownership through CLOSED and
token validity. Repeat entries in the same loaded program preserve ledger
identity; close does not reset the ledger. Unload invalidates all routing
records. Wrong-program, stale/unloaded, consumed/view-invalid and still-
owned-by-open-run conditions are diagnosed before mutation under the
negotiated adapter statuses. Copying/serializing a host token does not
create a second owner. No truncation to target/native pointer width is
permitted in the routing record.

Modules sharing registers may be linked into one device program and use
its existing ledger. Independent embedded programs do not share ids by
co-packaging. A future cross-program transfer must atomically detach the
source owner, validate destination classes/widths and open-run exclusions,
install one destination identity, revoke all old capabilities/aliases and
define failure rollback and reversal/journal ownership without copying or
measuring the quantum state. This protocol remains a publication and
implementation obligation; arbitrary bare-id reuse is not its substitute.
```

The transfer paragraph names exact required work, not a current rejection of the broader capability or an unversioned transfer API. Keep analysed opaque transport separate from ordinary source-level handle operations.

## Hashes, catalog and consumers

Loaded-instance/generation records are runtime identities and do not enter immutable object/closure hashes. Program/class changes invalidate old routes. Token tuple equality still differs from live identity, as in [budgets](budgets.md). No ref catalog edit. Driver/runtime/VM adapters and plugin escape analysis need matching published versioned rules; certificates remain per device root/program, not native token storage.

## Conformance vectors and verification

Require two programs/id 7; repeated closed calls preserving identity; unload/reload generation; 32-bit host transport of 64-bit ids; consumed/sliced handles; conflicting open run; device-minted token expiry; positive analysed transport and negative native operation/unknown-reader escape. Future transfer tests cover single ownership, failure rollback and correlated quantum state with no cloning. Actual checks in [INDEX](INDEX.md) cover finite routing only.

## K entry: 0.9 versus 0.10

Existing ledger/escape semantics need no semantic ISA K. Publish the adapter route as a companion and next dialect revision; new transfer mechanics default to **0.10/future versioned extension**. Any 0.9 K route requires **council + maintainer authorization of compatibility rules**, including token wire representation and diagnostics. Proposed decision `MIXED-LEDGER-ROUTING`.

## Mixed fat object

Use [fat-object-scope.md](../2026-11-02-rev2/fat-object-scope.md) and [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F06/F13. An ordinary native host retains opaque runtime ownership while only marked device closures receive ISA certificates, budgets and context `[]`. Separate embedded roots can share one linked ledger or an eventual transfer protocol; neither a host pointer nor container digest substitutes for it. Tapes remain per owning run. These amendments never gate rungs 3/4; F30 stays deferred.

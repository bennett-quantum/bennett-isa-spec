# F18 — copied outbound services and registered inbound calls

Bead: **b-dialect-spec-3ca**. Amendment proposal, never a prerequisite for rungs 3/4; F30 remains deferred until rung 3 or 4 lands.

## Exact defect and clause citations

[Dialect 0.2.12](../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md) and [0.3](../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md) §11.4 describe CC-to-H scalar/pointer-free copied service arguments and writeback, with pointer-rich callees promoted. [ISA 0.9](../../../spec/Bennett-ISA-Spec-0.9.md) §§7.4/10 retain request/reply history and validate a copied live service range. Neither is an inbound native-pointer registration or a callback/reentry ownership contract. §11.4 also lacks a complete alias policy for overlapping copied pointees. See [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F18.

## North Star gap

Direction/ownership publication, not authority to infer a live pointer from a service byte block. Native-sharing capability remains required under F04/F08 for North Star §§3.3/4.4. The existing copied CC path and R/L effect restrictions remain valid within their domain.

## Alternatives and witnesses

Device CC sends a four-byte pointer-free pointee to an H service. The host stub writes 7 to its copy; only the declared reply/writeback updates device memory. This grants no live access to the caller's arbitrary native graph. Two overlapping pointee views must share one copied backing range if the service can observe aliasing; independent copies can return the wrong result. A callback into a second run touching the first run's still-leased range conflicts even while the first is HOST_REQUEST. Prefer copied-range alias preservation plus an explicit future registered/reentry profile, rather than silently releasing ownership during host service.

## Recommended exact dialect diff

Append to §11.4:

```text
This is an outbound CC service protocol. Its pointer-free proof, explicit
extent, scalar encoding, copy ownership and reply/writeback bounds do not
register native storage or implement inbound native pointer arguments.
The service stub may use only its copied ranges for that request; request
payload lifetime is not a general host allocation lifetime. Preserve full
reply length/truncation and before-mutation range checks of ISA §10.

For pointer-free aliased arguments, normalize overlapping views of one
device allocation into a common copied backing range, preserve their
relative offsets and alias observations, and write back each covered byte
once under a specified consistent reply. The versioned service-block
profile records that range/view map; it does not infer aliasing from two
independent byte arrays. Independent/disjoint copies remain valid when
the analysis proves their observations equivalent. Native code must not
retain a service-copy pointer after reply. Pointer-rich native sharing
uses the separately published registration profile, not silent promotion
as the only eventual realization (§19.4).

HOST_REQUEST does not close the run or release its device/registered-memory
ownership. A service callback/reentry profile records parent run/request,
program/root, ordering and exact permitted leases. Reentry with disjoint
state may use an independent run under existing ownership rules. Conflicting
shared state requires a published atomic ownership delegation/handoff:
quiesce the parent, record exact ranges/generations and owner chain, prevent
parent access while delegated, restore ownership on reply/failure, and bind
all child history/results before resuming. Without that protocol the
conflict is diagnosed before mutation, not bypassed by a hidden unlock.
This names required future mechanics, not a source-level callback ban.

Retain R effect/CC reply/native event separation and source order. Recovery
uses retained replies without reissuing the service or callback. Reversing
the device buffer/history does not retract external host effects. R/L
remain closed to genuine hostcalls; registration does not change their
tier rules or permit native handle operations.
```

The new alias-range service encoding and any shared callback handoff need a versioned publication before consumers emit them. No new numeric service tag or current C API is introduced by this proposal.

## Hashes, catalog and consumers

Service declaration/adapter/body changes can change device object/closure/build hashes. Per-request copied bytes, range maps, replies and parent/child ownership bind invocation/journal records. They are not native pointer identities. No ref catalog change follows from documenting direction. Driver/plugin/runtime/VM/replay consumers must pin a published service/profile encoding rather than share implementation code.

## Conformance vectors and verification

Require pointer-free copy/writeback/truncation; aliasing overlapping views versus disjoint ranges; no retained pointer; bad range/reply before mutation; pointer-rich path using accepted registration; disjoint reentry; conflicting lease rejected before mutation; future handoff success/failure ownership restoration; exactly-once retained replies; R/L no genuine hostcalls. The finite model in [verify.py](verify.py) checks alias and lease counterexamples only; actual host-service execution is not claimed in [INDEX](INDEX.md).

## K entry: 0.9 versus 0.10

Next maintained dialect revision plus 0.3 draft; new service/registration/callback profiles default to companion/**0.10** integration. Existing ISA copied hostcall semantics are preserved. An in-version 0.9 K route needs **council + maintainer authorization of compatibility rules**, including service-block alias encoding and old consumer negotiation. Proposed decision `MIXED-HOSTCALL-DIRECTION-AND-REENTRY`.

## Mixed fat object

Apply [fat-object-scope.md](../2026-11-02-rev2/fat-object-scope.md) and [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F18. Native main/libc remain native; only marked device closures receive certificates/context `[]`, budgets and ledger ownership. Requests/journals bind their selected device run; native-memory sharing requires explicit registration, not whole-binary promotion. These amendments never gate rungs 3/4; F30 stays deferred.

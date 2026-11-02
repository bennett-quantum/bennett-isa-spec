# F07/F09 — a journal and reverse operation belong to one run

Beads: **b-isa-spec-at6**, **b-isa-spec-35m**. One run/checkpoint amendment. Amendment proposal, never a prerequisite for rungs 3/4; F30 remains deferred until rung 3 or 4 lands.

## Exact defect and clause citations

[ISA 0.9](../../../spec/Bennett-ISA-Spec-0.9.md) §§7.3–7.4/10 distinguish active R tape, retained CC replies and logical run history; saved-journal interchange explicitly requires a versioned replay profile. §10/T4 (§14.3) reverse an eligible device run, not native main. The active tape iterator returns device/op/64-bit operand fields, not console payload bytes or a native process snapshot. Mixed invocation/profile binding and host-memory commit/checkpoint rules are not published. See [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F07/F09; F09's device-only invariant is neutral.

## North Star gap

No missing device reverse capability; the missing mixed associations must preserve North Star §§4.2/4.4/5.1. Native external events already observed cannot be retracted by device reversal. Treating the entire native process as one reversible run would misstate ownership and scope.

## Alternatives and witnesses

P.root and Q.root each have a first console effect; identical `(console,write,ptr,len)` operands do not imply equal payloads or owners. Replaying P's journal against Q can corrupt unrelated state. In the two-call memory witness of [native-sharing](native-sharing.md), reversing the second run must restore its own entry `[99,23,30,40]`, not the earlier `[10,20,30,40]`. Prefer exact envelope identity plus per-run checkpoints. A global native I/O tape or a saved array of operand tuples alone cannot supply this. [verify.py](verify.py) checks wrong-root/invocation mismatch and independent checkpoint restoration; no durable journal engine is implemented.

## Recommended exact spec diff

Append to §7.3's saved-journal profile paragraph:

```text
A mixed saved-journal profile MUST bind its own schema/profile version,
source run identity, program/component/local entry, ISA object/root closure,
build/layout/mapping/runtime versions, invocation arguments/tuples,
registration generations/entry epochs, entropy input and ordered boundary
requests/replies. Runtime process-local run ids are locators, not durable
identity. Check all associations and required content before restoring or
issuing any event. Wrong-root/invocation/profile journals fail loudly.

The profile specifies its exact canonical bytes, required logical-frame
content and integrity checks separately from bennett_tape_next. R effects,
CC replies and ordinary native events keep separate owners/order. Recovery
reconstructs the same logical frames and reversal marks under §7.4 without
reissuing external events, reexecuting a Q call to recover its record or
copying quantum state. A saved active-tape projection is not a complete
recoverable journal unless that explicit profile proves it sufficient.

A profile exporting device console output includes the actual byte
sequence bound to the run/model and validates its length/content; pointer
and length operands alone do not export it. No unversioned payload or
native-process serialization is added to the current tape iterator.
```

Append to §10 States/T4's run scope:

```text
Reverse/close is per device run, including eligible FAILED prefixes. It
does not roll back native main, host libc, observed external events or
sibling runs. Under a mixed mutable-memory profile, retain the exclusive
lease and exact entry epoch through COMPLETED/FAILED/REVERSED until CLOSED.
Eligible reversal restores this run's bytes, allocation/ownership metadata,
logical active history, device models and permitted register/token state.
The entry checkpoint includes native edits made before this invocation.

Close commits the actual remaining prefix and releases its leases with
the profile's visibility synchronization. Reversal must precede close;
after ownership release a saved checkpoint is not authority to overwrite
later native changes. Interference/liveness invalidation is diagnosed
before access or reverse writes according to the negotiated profile,
without weakening measurement, unverified-free or touched-resource rules.
Failure of the current operation is atomic; earlier valid effects remain
the failed prefix until reversed or committed. Native host-service effects
remain observed even when the device reply/buffer/history is reversed.
```

The concrete wire format for a durable full journal is a separate profile publication, not invented from the existing bare arrays. This note specifies its mandatory binding/validation and semantic obligations; [native-sharing](native-sharing.md) fixes the proposed lease/checkpoint policy.

## Hashes, catalog and consumers

Journal digest, active tape contents, source-run identity and ISA object/root hashes are different identities. Changing invocation/epoch invalidates journal applicability even when source hashes match. No ref catalog edit follows. VM/runtime/driver/replay tools must publish/pin the journal profile; certificates do not become native process snapshots.

## Conformance vectors and verification

Require wrong root/run/arguments/entropy/epoch/profile, missing payload bytes, prior reply recovery without a new external request, reversed segments skipped exactly once, eligible failed prefix, measurement/unverified-free rejection, two roots with separate tapes and host edits between closed calls. Observe exact console bytes separately from active operands and never label native I/O retraction as T4. Actual checks in [INDEX](INDEX.md) are finite binding/checkpoint tests, not saved-journal integration.

## K entry: 0.9 versus 0.10

Publish the new journal companion and memory profile; integrate scope text with **0.10**. Existing §7.4 no-reissue and §10 eligibility require no relaxation. An in-version K route needs **council + maintainer authorization of compatibility rules**, including old journal admission. Suggested decision `MIXED-RUN-JOURNAL-SCOPE`; no numeric K or current tape ABI change.

## Mixed fat object

Apply [fat-object-scope.md](../2026-11-02-rev2/fat-object-scope.md) and [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F07/F09. Each marked device root has its own certificate/context `[]`, invocation budget/ledger and run-associated tape. Native main/argv/envp/host libc stay native and are not silently journalled as device regions. Explicit memory checkpoint/visibility keeps native sharing reachable. These amendments never gate rungs 3/4; F30 stays deferred.

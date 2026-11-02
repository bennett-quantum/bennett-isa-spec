# F04/F08/F23 — native calls and registered memory

Beads: **b-isa-spec-x1y**, **b-isa-spec-6ma**, **bennett-vm-8v0** (contract side only). This is an amendment proposal, never a prerequisite for rungs 3/4. F30 remains deferred until rung 3 or 4 lands. No launch ABI or backend is implemented here.

## Exact defect and clause citations

[ISA 0.9](../../../spec/Bennett-ISA-Spec-0.9.md) §10 binds INT/F64/QREG/TOKEN to device CC parameters and copies argument/entropy buffers. It has neither an ordinary `ptr` binding nor an aggregate/native calling convention. §§7.1–7.2 initialise device-owned memory and allocation metadata from `layout`; an integer holding native pointer bits supplies neither liveness nor access authority. §0.3's deferred ISA-11 cannot substitute for a published native-memory contract.

[AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F04/F08/F23 correctly identifies a mixed-case lock-out. The existing by-value launch works within its stated domain; this is not a reproduced failure of a host-sharing backend. F23's actual bead now cites [VM PLAN, proposal 0.1](../../../../bennett-vm/docs/PLAN.md), which already sketches registrations, leases and eleven future vectors. That design is useful input, not a published cross-repository interface or executed integration evidence. No duplicate F23 implementation note is needed.

## North Star gap

This is a real missing boundary capability for North Star §§2, 3.3, 4.4: native main must call marked modules while preserving ordinary pointer-rich sharing. Promoting main or copying unrelated buffers cannot be the only eventual realization. Existing rungs and device lowering remain valid and independent of this amendment.

## Alternatives and witnesses

| Construction | Witness and consequence |
|---|---|
| Treat `ptr` as INT | Native address 0x1000 can denote unmapped device storage, a different global, or stale storage after reuse. Width agreement supplies no ownership. Unsound as registration. |
| Snapshot into independent buffers | Two native pointers to one interior cell must observe the same write; two independent copies can disagree. Useful only with an explicit alias-preserving import/export protocol. |
| Promote all pointer-sharing callers | Expresses device computation but converts native main in the cyclic graph case. Preserve as an option; it does not witness the mixed capability. |
| Versioned registration and native adapter | Canonical allocation identities, alias-preserving address observations and leases support native calls and reversal. Recommended. Same-address and imported-private-oracle profiles have distinct evidence. |

Concrete acceptance witness: native bytes `[10,20,30,40]`; call A adds 3 to byte 1 and closes, yielding `[10,23,30,40]`. Native code sets byte 0 to 99. Call B XORs byte 0 with 1; reversing B **before close** restores `[99,23,30,40]`. A stale first-call snapshot would incorrectly restore 10. A second witness has two cyclic nodes and two arguments to one interior value: write 7 through one alias, read 7 through the other, preserve pointer equality and address-derived outputs. The finite model in [verify.py](verify.py) checks the bookkeeping and alias counterexample; it is not VM execution.

## Recommended exact spec diff

Append to §10 Model, and cross-reference from §§7.1–7.2:

```text
Native process entry and device CC entry are separate roles. A mixed
container MAY expose a native callable symbol whose adapter selects a
device program and CC entry. launch 0.9's existing argument kinds remain
by-value; integer bits do not register native storage. Native calls with
ordinary pointers or aggregates use an explicitly negotiated mixed-launch
and memory profile. The following requirements govern that profile.

The adapter descriptor binds container component/local-entry, linked ISA
object_hash, entry signature and closure_hash, target ABI/triple/DataLayout,
layout/mapping profile, certificate/build record, and runtime/service
versions. It specifies positional scalar and aggregate decomposition,
field offsets, padding treatment (unobservable padding is zero-filled),
endianness, integer widths and exact float bits. Data pointers, function
pointers, opaque launch ids and tokens have distinct descriptor kinds.
Pointers are capabilities plus checked offsets; raw integers grant no
native access. Null and one-past values retain their source semantics;
one-past is not a dereferenceable byte.

A registration records canonical allocation identity and generation,
owner/liveness authority, extent, permissions, target layout and mapping,
allocator metadata policy, and current entry epoch. Interior/overlapping
views resolve to that same identity. Mapping preserves aliasing and all
defined pointer observations, including comparisons, ptrtoint/address
hashing and indirect dispatch; it MUST NOT silently swizzle unidentified
integer words. A same-address profile requires matching representation and
nonconflicting ranges. Other mappings require a separately published
observation-preserving encoding/relocation protocol, not a capability cap.

Before access, validate nonwrapping ranges, generation, liveness, layout,
permissions and ownership. Device-private allocations keep §7.1 semantics.
Imported storage supplies equivalent bounds/liveness/allocation metadata;
it does not require native malloc to have executed the device bump history.
Escaping allocations require a host-compatible allocation/transfer profile
covering native free/realloc, aliases, budgets and reversible metadata.
A private device address alone cannot become a returned native pointer.

For the exclusive mutable profile, acquire a coordinated lease before the
run and retain it through CLOSED. All aliases and programs sharing that
storage participate in the same coordinator. Host/free/realloc/unregister
and conflicting runs cannot access it during the lease; compatible read
leases and disjoint storage may coexist. The liveness authority must
prevent or detect invalidation; hashing entry bytes is not an interference
protocol for arbitrary unsynchronised native writes.

Each invocation binds program/root/run, arguments, registration generations
and entry epochs. COMPLETED/FAILED retain ownership. Eligible reverse
restores this invocation's entry bytes and allocator/ownership history
before close, retaining §10's measurement/free restrictions. A failed
operation makes no partial change; earlier successful operations remain
in the failed prefix and reverse under §§7.4/10. Close commits the exact
remaining prefix and releases leases. This is synchronised visibility,
not atomic publication of an arbitrary graph. After close a later native
edit is part of the next invocation's entry state.

The synchronous adapter waits through HOST_REQUEST and suspension under
§10, distinguishes API failure from a FAILED device result/fault, extracts
declared results, closes and returns to the native caller. An asynchronous
adapter returns an opaque run owner; it does not grant early access to
leased memory. No exception crosses the C boundary implicitly. Profile
negotiation, stale/wrong-owner/unregistered pointer, range/layout mismatch,
interference and failed-operation diagnostics are distinct published
status outcomes; assign their wire encodings with the new ABI version.
Do not allocate numeric tags in launch 0.9's existing enum.

Lifted/oracle memory is a private coherent construction. Import a
consistent entry image and equivalent metadata before private allocation,
then retain the prescribed forward/reverse semantics. A quantum address
does not cause a native RAM read. Current copied launches and snapshot/
replay remain admitted constructions, not the sole eventual native-sharing
capability. Device effects, CC replies and native I/O retain separate owners.
```

Publish the descriptor and C wire layouts as one versioned companion contract before implementing it in driver/runtime/VM. The text deliberately leaves numeric encodings for that publication, rather than presenting a partial C header as ready to consume. F23 should consume this publication and the F15/F16 mapping rules; retain the VM's existing integer-address backend and by-value ABI meanwhile.

## Hashes, catalog and consumers

Runtime registrations/addresses belong to invocation records, never immutable ISA source identities. Container bytes/digest, payload bytes/digest, `object_hash`, root closure, certificate and runtime profile are separate bindings. Changed imported images/layouts or generated wrappers can change actual object/closure/schedule/certificate hashes. A memory profile alone does not change `ref@0.9` recipe bytes. Driver, runtimes, VM, plugin and conformance must pin the accepted contract independently; no consumer imports VM source as the protocol.

## Conformance vectors and actual verification

Require scalar/F64/QREG/TOKEN compatibility; aggregate padding and float-bit round trips; both concrete witnesses above; interior/overlapping/cyclic aliases; host edits between closed runs; free/reuse at the same address; null/one-past/range overflow; stale/wrong-program generations; conflicting leases; failed-store prefix and reverse; measurement ineligibility; returned host-owned allocations and native realloc; function pointers; two embedded roots; lifted private import and metadata-only invalidation. Compare the same materialised device closure and layout, preserving original pointer identity for address-sensitive baselines. Unregistered numeric CLI pointers and `--mem` hex copies are negative evidence controls. These are proposed integration vectors, not claims of an implemented backend. [INDEX](INDEX.md) records the actual bounded checks.

## K entry: 0.9 versus 0.10

Recommend a versioned companion `mixed-launch/memory` contract, normatively referenced from ISA 0.10 and the next dialect revision. A 0.9 K entry may clarify the existing by-value boundary, but cannot add pointer tags or memory semantics silently. Any in-version K route needs **council + maintainer authorization of compatibility rules**, including negotiation and status encodings. Assign numeric K ids only on acceptance. No recipe catalog change is required by this design alone.

## Mixed fat object

Apply [fat-object-scope.md](../2026-11-02-rev2/fat-object-scope.md) and [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F04/F08/F23. Native main, argv/envp and host-only libc remain native; only marked modules and their complete device closures enter ISA inventory. Each root keeps its own certificate/context `[]`, budget/ledger and run/tape association. Shared storage uses explicit coordination across those roots. This keeps the mixed fat object reachable; it neither implements it nor makes its amendments prerequisites for rungs 3/4. F30 stays deferred.

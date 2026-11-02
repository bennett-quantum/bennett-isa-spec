# F11 — define the complete device-state closure

Bead: **b-dialect-spec-hsb**. Amendment proposal, never a prerequisite for rungs 3/4; F30 remains deferred until rung 3 or 4 lands.

## Exact defect and clause citations

[Dialect 0.2.12](../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md) §19.5 says `globals: every global of the link`. §§4.3/17.1 require one semantic memory/shared globals. The [0.3 draft](../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md) §§4.3, 17.1–17.2, 19.5 likewise speaks of program globals and the complete link, while adding qtab/arena-specific ordering. Neither distinguishes host-only native globals from a selected device program's state inventory. A literal whole-native-link inventory drags native TLS/libc/unrelated state into ISA layout/certificates. See [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F11.

## North Star gap

Yes if the universal inventory is applied to mixed mode (North Star §§2, 3.3, 4.4). Device closure completeness remains mandatory. Native-only state can stay outside ISA memory only after a sound closure proof; merely naming a global “host” does not prove it unreachable from a device load or indirect call.

## Alternatives and witnesses

Native main uses native_tls and native_counter; marked root reaches shared.value through table[fp], and bounds checking also reads alloc_meta. A direct-symbol-only scan can miss shared.value or alloc_meta. Including the entire native link imports native_tls needlessly. Cloning shared.value for H and device gives two values after a native write. Prefer a conservative state/dependency fixed point with an explicit boundary binding. The finite closure witness in [verify.py](verify.py) requires shared.value, table, fp, alloc_meta and transitive callee state, while excluding native_tls/native_counter.

## Recommended exact dialect diff

Insert before §19.5's layout construction in both future tracks:

```text
A complete device link is the selected device roots, all device variants
reachable through direct calls, region candidates and every conservative
indirect candidate, their required runtime/arith bodies, and the complete
state dependency closure below. Whole-program analysis may inspect more
native code; inspection does not put that code or state into this link.

Seed the state closure with globals, initialisers and boundary-visible
shared allocations that any selected variant may read/write/address or
observe through a defined pointer operation. Close it under reachable
initialiser references, aliases, pointer-containing fields, function-pointer
targets and their variant dependencies, allocation bounds/liveness,
allocator, shadow-stack and other metadata affecting execution or faults.
Include all runtime-generated state required by the selected construction.
Unknown may-alias/indirect targets require a sound conservative dependency
set; uncertainty is not permission to drop state or reject the source.

Native-only functions, globals, TLS and libc state outside this closure
need no ISA layout/certificate record. Shared state has one semantic
identity across H and device variants; it is neither independently cloned
nor omitted. Its concrete storage and entry epoch are supplied by the
selected mixed memory/layout profile. “Native-only” is a proved absence
from the dependency closure, not a producer label overriding reachability.
```

In 0.2 §19.5 replace `globals: every global of the link at its link address with its image` with `globals: every device-state global in the complete device link, at its profile-bound address with its entry image; native-only globals remain outside this inventory`. In 0.3 apply the same scope to “every program global” and ordinary data placement, **retaining** its single range-order rule, qtab exclusion/placement, may-Co relocation, per-oracle interfaces and all nonoverlap/capacity checks. Device qtab/shadow/allocator state required by that construction remains in scope even without a direct user load.

Append to §§4.3/17.1:

```text
One classical memory means one semantic identity for genuinely shared
state. It does not require native-only state in every embedded ISA object.
Each selected device program imports/binds its complete state closure under
§19.5 and the negotiated memory profile. All device-reachable helper and
fault-metadata dependencies remain mandatory.
```

For uncertain dependencies, the sound analysis may conservatively include state while exact mapping/publication work remains an implementation obligation. It must not turn an incomplete analysis into permission to force every host function into device code.

## Hashes, catalog and consumers

Removing unrelated state changes actual layout/images and often H/A_tab, object/closure/schedule/certificate hashes and frozen capacities. Rebuild the native comparator and device output from the same selected closure; bind the new layout/profile in the container. The recipe catalog is unchanged unless its source changes independently. Producers/checkers need reproducible dependency evidence; no consumer may trust a host-only label without validating the closure against the selected module and build inputs.

## Conformance vectors and verification

Test direct/shared globals, cyclic initialisers, indirect targets, aliases, metadata-only dependency, native TLS/libc state, 0.3 qtab/arena state and two roots sharing a global under coordinated leases. Omit each required dependency in turn and require a missing binding/closure diagnostic. Mutating the shared global natively between closed calls must change the next entry image; independent clones must fail the alias observation. [INDEX](INDEX.md) reports finite graph/citation checks only, not a complete points-to analysis or backend test.

## K entry: 0.9 versus 0.10

This is a dialect state-scope amendment: next versioned maintained revision plus 0.3 draft, no automatic ISA K. ISA 0.9 layout validation remains intact. Any semantic memory extension is the companion/0.10 route in [native-sharing](native-sharing.md). An in-version ISA K route requires **council + maintainer authorization of compatibility rules**. Suggested dialect decision: `DEVICE-STATE-CLOSURE`, with the exact dependency rule above; no silent change to 0.2.12.

## Mixed fat object

Apply [fat-object-scope.md](../2026-11-02-rev2/fat-object-scope.md) and [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F11. Native main/argv/envp/libc remain outside device inventory unless their state is genuinely shared and explicitly bound. Marked roots have separate context `[]`, certificates, budget/ledger and tape associations, with complete shared device state. This addresses the inventory lock-out without shrinking any device obligation. These amendments never gate rungs 3/4; F30 remains deferred.

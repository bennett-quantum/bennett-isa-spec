# yz7/F12 — distinguish a callable module from a device process

Beads: **b-dialect-spec-yz7**, **b-dialect-spec-qbw**. One entry-role/process-context amendment. Amendment proposal, never a prerequisite for rungs 3/4; F30 remains deferred until rung 3 or 4 lands.

## Exact defect and clause citations

[Dialect 0.2.12](../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md) and [0.3](../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md) §19.2 deliberately specify whole-program R entry with `bennett.argv`, musl preparation and main-return-as-exit. §§14.7/19.9 provide exit/output/build context but no callable-module role or native/device libc state association. yz7's driver evidence is a publication request, not a reproduced driver failure in this pass. [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F12 is neutral when confined to explicit whole-device mode. The earlier [djm process-helper note](../2026-11-02-pass2/b-dialect-spec-djm.md) addresses helper preparation, not the complete callable/native distinction, so yz7 is not fully dispositioned by that note.

## North Star gap

Entry/process scope for North Star §§2/3.3/4.4, not missing argv support in the existing whole-device route. Applying main startup/exit to every marked call would prevent normal native call/return.

## Alternatives and witnesses

Native main calls marked add(4), does native work, then calls add(8). No argv service or exit handler should execute at either ordinary return. Explicit whole-device R main must still fetch its declared process context and run exit/stdio cleanup exactly once. Treating a marked i32 result as process exit conflates these cases. Prefer an entry role in the accepted adapter/build convention. The mode witness in [verify.py](verify.py) checks required actions/return classification; it is a proposed protocol model, not a generated host shim.

## Recommended exact dialect diff

Prefix §19.2 and add a §19.2 callable-entry paragraph:

```text
The following startup/argv/envp/libc/main-exit rules apply to explicitly
selected whole-device-process mode. They do not define ordinary callable
module entry. Whole-device-process retains all existing preparation,
device effects, measurement restrictions and exit/_Exit behavior.

A mixed entry descriptor has role callable-module or whole-device-process.
callable-module retains native-owned process startup, argc/argv/envp,
environ, errno/fenv/locale/stdio and exit-handler ownership unless an
explicit versioned device-context binding names a separate device instance
or shared state. Declared arguments use the accepted mixed adapter; normal
return produces function results and returns to the native caller. It
does not fetch argv, run process exit handlers, flush all stdio or reset
libc/device/tape state. Faults are device results, distinct from adapter
status and native process exit. An explicit source exit/_Exit still has
the declared §14.7 behavior under its selected context; it is not inferred
from an ordinary integer return.

Device context records identify owner/program instance, initialization
epoch, layout/image and service bindings, and lifetime. Device R/L libc
uses its tier models and history, not implicit access to host libc state.
State persists across calls according to the declared program/context
lifetime; entry adapters do not reinitialize it per call. Sharing any
native mutable context requires the memory/ownership profile, not a raw
copy of native libc internals. The mixed descriptor/build record binds
the context and process role to its selected entry and device run.
```

Add the optional explicit snapshot rule:

```text
A requested device process-context snapshot is versioned and marshalled,
not a native argv pointer cast. Record argc, argv/envp element counts and
each byte-string length excluding its trailing zero; encode bytes plus
one trailing zero and final null vector elements under the target pointer
layout. Reject embedded zero bytes in a C-string element, count/length
overflow, missing terminators and incompatible layout before mutation.
Preserve argv order and every supplied env entry; argc may be zero with
the final null element still present. Bound argc to the target C int.
The snapshot has explicit storage/liveness/address mapping, permissions,
service/run owner and release point. It does not automatically import
native TLS, auxv or libc-private structures. Any additional context needs
its own published schema and source semantics, not whole-host lowering.
```

This preserves existing whole-device preparation and makes no claim that the optional vector wire format is already implemented. The accepted profile must assign its concrete schema/ABI with the entry descriptor; current consumers keep current contracts.

## Hashes, catalog and consumers

Entry role/context/service bindings belong to outer adapter/build/invocation records. Synthesized device wrappers/images can change object/closure/certificate hashes; host-only argv does not add device records. No ref catalog change is inherent. Driver/plugin/runtime/VM must agree on persistent context and fault/exit ownership through a published revision, not a private helper convention.

## Conformance vectors and verification

Two ordinary marked calls plus native I/O; no implicit argv/exit; explicit whole-device main return and exit/_Exit; context persistence; separate native/device errno/fenv/locale/stdio; requested empty/nonempty argv/envp, invalid counts/NULs/widths/lifetime; failure before preparation mutation; program unload with open run. Keep djm's preparation vectors. [INDEX](INDEX.md) records only finite mode/marshalling checks; no driver shim or musl execution was performed.

## K entry: 0.9 versus 0.10

Next maintained dialect revision and matching 0.3 draft; no standalone ISA opcode/K. The new adapter/context profile defaults to **0.10 companion integration**. Any in-version 0.9 K route needs **council + maintainer authorization of compatibility rules**, covering service/context negotiation and old whole-device consumers. Proposed dialect decision `DEVICE-ENTRY-ROLE`.

## Mixed fat object

Use [fat-object-scope.md](../2026-11-02-rev2/fat-object-scope.md) and [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F12/F04. Native main/startup/libc retain native ownership; marked roots alone receive ISA certificates/context `[]`, budgets/ledgers and tapes. Whole-device process mode remains supported alongside callable modules; future registered-memory bindings supply any genuine context sharing. These amendments never gate rungs 3/4; F30 stays deferred.

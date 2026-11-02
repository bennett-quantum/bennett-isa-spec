# F14 — retain native pointer-sharing callers

Bead: **b-dialect-spec-yqf**. Amendment proposal, never a prerequisite for rungs 3/4; F30 remains deferred until rung 3 or 4 lands.

## Exact defect and clause citations

Both [dialect 0.2.12](../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md) and [0.3 draft](../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md) §19.4 require pointer-sharing H functions to be promoted transitively, potentially through main. §11.4 also calls for promotion of a pointer-rich outbound callee. The 0.3 §25.3 row classifies ISA-11 as an efficiency item satisfied by promotion. That preserves device computation but not a native host around marked modules. The §25.1 statement retiring five device-lowering blockers does not publish a mixed memory ABI. See [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F14.

## North Star gap

Yes, for the selective-module experience in North Star §§2, 3.3, 4.4. Promotion remains a valid construction; making it the permanent mandatory realization excludes the required native-sharing case. This is a boundary publication obligation, not a reason to reopen retired device-lowering blockers or block rungs 3/4.

## Alternatives and witnesses

Native main owns cyclic nodes A.next=B and B.next=A, passes two aliases of B.value to marked update, then uses its native-only library. Promotion through main loses that split. Copying each argument independently loses alias visibility. A versioned registration of the graph, preserving pointer observations and lifetime, lets update write 7 through either alias and main observe 7 after close. Retaining native main while silently using old device images fails when it changes B.value between calls.

Prefer explicit mixed-launch/memory profiles from [F04/F08/F23](native-sharing.md). Universal promotion is a supported fallback only when that mode is selected. A scalar-only mixed fixture is useful evidence but cannot discharge this pointer-rich obligation. The graph/epoch finite checks in [verify.py](verify.py) distinguish these alternatives; no compiler/native-sharing execution is claimed.

## Recommended exact dialect diff

Replace §19.4's paragraph in the next maintained revision and mirror in 0.3:

```text
Host placement is a mode choice with preserved source semantics. In the
promotion construction, H functions sharing pointer-rich memory with
device code have CC variants, transitively to their selected roots, as
in the existing construction. This construction remains supported.

Mixed native-sharing mode instead retains native callers and binds their
shared storage under the published mixed-launch/memory profile referenced
by the selected container. It preserves aliases, pointer observations,
liveness, allocation metadata and shared-global identity, and defines
ownership, synchronisation, writeback and reverse/close. Promotion cannot
be the only eventual realization of this mode. A missing implementation
is tracked against that contract, not reported as a source rejection or
silently satisfied by promoting native main. Until the profile is
published this is an explicit publication obligation, not an implemented
driver option or permission to improvise an ABI.
```

Replace the ISA-11 row of §25.3 with:

```text
ISA-11 | promotion realizes device computation; retaining pointer-sharing
native callers additionally requires the mixed-launch/memory contract
(ISA F04/F08; dialect F11/F15/F16). This is a boundary capability
obligation, not merely a cost optimization of mixed mode.
```

Append to §25.1's introduction:

```text
Retirement of the listed device-lowering blockers does not assert that
native-container packaging, native function adapters or live host-memory
sharing are already published. Those mixed-boundary amendments do not
change the acceptance of current device-only rungs.
```

Append to §11.4's pointer-rich promotion sentence: `This describes the promotion construction of §19.4; a future native-sharing realization uses the explicitly published profile and callback/ownership rules, never the copied service buffer alone.`

## Hashes, catalog and consumers

Placement can change device closure membership, emitted wrappers and object/certificate/build identities. A selected memory profile and runtime mapping are invocation bindings; they do not replace device closure hashes with a host hash. No `ref@0.9` catalog edit follows from this classification. Driver/plugin/runtime/VM must coordinate a published profile and independently pinned versions, including [F11](b-dialect-spec-hsb.md)'s shared-state closure.

## Conformance vectors and verification

Require native main plus a native-only dependency, cyclic/interior aliases, shared globals, host mutation between closed calls and exact writeback/reverse. Inspect both native artifact and selected ISA inventories; promotion-only output must be labelled as such. Retain the old promotion vector and all device faults/coverage. Test unsupported profile negotiation before effects, rather than introducing a program-domain rejection. Actual checks are the finite model and citation/structure checks in [INDEX](INDEX.md); a compiler or backend implementation is outside this note.

## K entry: 0.9 versus 0.10

No ISA K is needed to correct the dialect's classification. Publish a new maintained dialect revision with matching 0.3 text; preserve old pins. Its actual native-sharing ABI follows the new companion contract, default ISA 0.10 route. An ISA 0.9 K route requires **council + maintainer authorization of compatibility rules**. The proposed dialect decision is `MIXED-PLACEMENT: promotion is one realization; native-sharing publication remains required`.

## Mixed fat object

Use [fat-object-scope.md](../2026-11-02-rev2/fat-object-scope.md) and [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F14. Only marked modules and their required device closures enter ISA objects; native main/startup/argv/envp and host-only libc retain native ownership. Per-root certificates/contexts, budgets, handles and tape associations remain separate. This amendment directly removes an architectural lock-out without claiming the future ABI exists. It is never a prerequisite for rungs 3/4; F30 stays deferred.

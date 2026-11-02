# F15/F16 — native entry import and address profiles

Beads: **b-dialect-spec-5dl**, **b-dialect-spec-dce**. One memory-profile amendment covering import and observations. Amendment proposal, never a prerequisite for rungs 3/4; F30 remains deferred until rung 3 or 4 lands.

## Exact defect and clause citations

[Dialect 0.2.12](../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md) and [0.3](../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md) §8.5 reconstruct caller bytes and allocation/shadow-stack history at device bump addresses. §§4.4/17.1–17.3/19.5 tie pointer observations to a fixed comparison link and one memory identity. Native malloc graphs do not have the required bump history automatically; independent embedded programs do not become one address space. [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F15/F16 identify missing association/import mechanics, not a failure of existing replay.

## North Star gap

Boundary mechanics for North Star §§3.2–3.3/4.4, feeding the genuine native-memory gap in F08. Existing snapshot/replay remains valid; it cannot be the only eventual means of live native sharing. Pointer equality, ptrtoint/address hashing, function dispatch, liveness and allocator behavior remain part of defined observations.

## Alternatives and witnesses

Native nodes at 0x10000 and 0x18000 form a cycle with two aliases to the second node's interior value. A size-only bump replay at device heap 0x2000 produces different addresses and pointer-derived results. Blind byte copying leaves native pointer words unmapped. A registered same-address profile preserves those observations where layouts/ranges agree; a different mapping needs a published observation-preserving representation, including integer-derived pointers and dispatch. A native free/reuse can change liveness/generation without changing loaded bytes, so byte-only snapshot caching is unsound. [verify.py](verify.py) checks aliases and metadata/epoch invalidation, not an actual native-memory backend.

## Recommended exact dialect diff

Append to §19.5 in both future tracks:

```text
The existing fixed non-PIE shared-address comparison/production construction
is profile fixed-shared-address-1. Retain all range/alignment/nonoverlap
checks and common addresses for values crossing the boundary. Bind each
selected program/root to its concrete layout, build record and memory
profile; co-packaging distinct layouts does not merge them.

Other native layouts require an explicit versioned mapping/relocation
profile covering globals, data/function pointers, indirect dispatch and
defined pointer comparisons/ptrtoint/address hashes. The profile preserves
those source observations; it cannot silently relocate unidentified
integer words or require native-only code/data in ISA memory. Revalidate
the complete linked device layout and profile after linking/rebinding.
```

Append to §8.5 and cross-reference §§17.1–17.3:

```text
Device-allocation replay and native registration are different entry-state
sources. For a registered native graph, acquire a consistent entry epoch
under the memory profile before reading bytes or metadata. Compute the
complete byte and metadata dependency closure (§19.5), including aliases,
cyclic/interior pointers, allocation-validity/address observations and
logical shadow-stack dependencies even when the byte read set is empty.

Canonicalize allocation identities without merging distinct live objects;
map all pointer observations by the selected profile. Import bytes,
bounds/liveness, pinned/ownership state and required allocator/shadow-stack
metadata before any private oracle allocation/access. Do not require native
malloc to have followed the device bump history or infer live logical
frames from backing-storage containment. The private construction must
produce the same defined entry observations, allocation/fault outcomes and
addresses as the corresponding materialised native closure. Any host-
compatible allocator/escaping-storage profile is explicit, not an unchecked
conversion of a private device address to a native pointer.

Retain §8.5's private L memory semantics: writes are restored/discarded
except declared outputs, input regions remain unchanged, and no native
load at a coherent address is introduced. Device bump replay keeps its
existing allocation order, invalidation, register unpreparation and charges.
An alternative native import needs published exact metadata/address and
allocation rules; these are required mechanics, not permission to promote
all native callers or narrow their allocation graphs.

Snapshot reuse requires identical dependent bytes AND allocator/liveness/
generation/shadow-stack metadata and entry mapping epoch. Relevant native
free/realloc, logical stack changes or rebinding invalidate it even if the
byte read set is unchanged. New host edits between closed launches are
part of the next entry epoch. Export/return/close obey the memory profile;
private L state is not blindly written back as shared host mutation.
```

The exact arbitrary-layout/allocator wire realization remains a versioned publication obligation under [native-sharing](native-sharing.md); the fixed profile is an admitted first construction, not the final capability boundary.

## Hashes, catalog and consumers

Fixed source images/layouts affect object/closure/build hashes; runtime registration epochs/mappings bind invocations and private entry-state identities. Metadata changes can invalidate cached snapshots without changing source bytes. Container/root/layout identities must agree with certificates and comparators. No ref recipe edit is inherent; if the accepted import adds ordinary materialised helper/replay rows, their bodies and costs become part of the actual device/reference closure. Driver/plugin/runtime/VM/checkers pin that published realization.

## Conformance vectors and verification

Fixed-address compatibility; noncontiguous cyclic native allocations; interior aliases and address-sensitive outputs; function pointers; shared globals; incompatible pointer width/layout; colliding ranges; metadata-only invalidation; repeated host edits; private L restoration; stack backing versus logical extents; escaping allocation and native free/realloc. Compare from the same entry identity/address configuration rather than relocated independent baselines. [INDEX](INDEX.md) records finite alias/epoch checks; no pointer relocation or allocator implementation is claimed.

## K entry: 0.9 versus 0.10

Next versioned dialect revision plus 0.3 draft; default companion/**0.10** memory semantics. ISA 0.9's layout/bump model cannot silently acquire arbitrary native allocations. An in-version K route needs **council + maintainer authorization of compatibility rules**, including coherent import and allocation semantics. Suggested decision `MIXED-ENTRY-MEMORY-PROFILES`; no unversioned catalog change.

## Mixed fat object

Use [fat-object-scope.md](../2026-11-02-rev2/fat-object-scope.md) and [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F15/F16. Each marked device program/root has a concrete layout/certificate/context `[]`, budgets/ledger and run/tape, while native main/argv/envp/libc and unrelated state stay native. Explicit mappings and allocator obligations preserve live native sharing rather than replacing it with universal promotion. These amendments never gate rungs 3/4; F30 stays deferred.

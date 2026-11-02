# F01/F02/F03 — outer inventory and device-root scope

Beads: **b-isa-spec-lop**, **b-isa-spec-cob**, **b-isa-spec-8nx**. One outer-inventory amendment with link/certificate scope rules. Amendment proposal, never a prerequisite for rungs 3/4; F30 stays deferred until rung 3 or 4 lands.

## Exact defect and clause citations

[ISA 0.9](../../../spec/Bennett-ISA-Spec-0.9.md) §§9.1–9.2 define a closed ISA object and its semantic identity, not a native binary/container. §9.3's name/layout conflicts apply to one actual device link. §§9.4/12.1/12.3/13 describe device frame/proof/coverage records with root-relative contexts but do not publish a native-container association. The driver's earlier request folded into lop also needs native artifact/dependency provenance, component-local entries, images and runtime/service bindings. [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F01–F03 and F21 distinguish these obligations; F02 is neutral when scoped correctly.

## North Star gap

Packaging/association is missing for North Star §§2, 3.3, 4.4, while inner object linking is already compatible with selective modules. Applying a single flat device namespace or whole-native-binary certificate inventory would introduce a lock-out. Preserve every actual device import, owner proof and reachable region.

## Alternatives and witnesses

Two embedded programs P and Q both define private `helper`, with unequal bodies/layouts. They can be independent loaded programs in one native host. Flattening them creates a real §9.3 conflict; relaxing duplicate-name checks inside an actual link is not the solution. A native main calls both; P.root and Q.root may each call a statically shared library implementation while retaining separate root context `[]`. Substituting P's certificate for Q must fail even if the local name is `root`. A container digest alone cannot identify the correct closure.

Prefer a detached canonical inventory around unchanged inner bytes. The finite inventory/link/context checks in [verify.py](verify.py) exercise distinct program keys, tampered extraction bytes and independent root bindings. The existing interpreter checks source objects; this is not an implemented native container loader.

## Recommended exact spec diff

Add §9.5, cross-referenced from §§9.1/9.2/10/12.3/13:

```text
Mixed native packaging uses a separately versioned outer inventory. It
does not add fields/sections to object 0.9 or make native executable bytes
input to its parser. The initial inventory is detached UTF-8 JCS, no final
LF, schema "bennett-mixed-inventory-1". Its identity is SHA-256 of those
exact canonical bytes, computed externally (not a self-hash field).

The object has exactly schema, artifacts, programs, native_entries and
dependencies. artifacts is an id-sorted array of {id,kind,byte_length,
sha256}; kind is native, isa, certificate, schedule, build_record, image
or adapter. Each id resolves through the packaging profile to exact bytes;
relative paths are transport locators, not semantic identities. Hash and
length are checked before use. Native hashing covers the final native
artifact, including embedded payloads, but not this detached inventory.
Embedding the inventory itself requires a future explicit self-hash
exclusion profile; no circular hash convention is inferred.

programs is an id-sorted array. A program records its linked payload as
{artifact,offset,length,sha256}, independently computed ISA object_hash,
required full ISA/subformat versions, layout identity, ordered source
member payload/digest list and link recipe/version, build_record reference,
runtime/service versions, shared-image/mapping profile references and
entries. Extraction uses nonnegative nonwrapping in-range byte slices;
verify the slice digest and inner object_hash/version independently.
Do not canonicalize or rewrite extracted inner bytes. A program with no
link operation has one member. An actual link retains complete §9.3
resolution, layout/data conflict checks and certificate rebinding.

An entry key is (program id, local entry name); duplicate keys are invalid.
Each entry records the selected device CC entry signature/interface and
closure identities plus the region roots it can invoke. Each region root
records its closure/interface identity, certificate and optional schedule
artifact references, build/layout/version requirements and budget-selection
record reference. CC entry and quantum region root are distinct roles;
one CC entry may invoke several region roots. Runtime invocation records
bind resolved arguments/tuples to these static entry/root records.

native_entries is a symbol-sorted array associating each exported native
callable symbol and target calling-convention/adapter descriptor with its
program/local-entry key. dependencies records native library provenance,
runtime/service versions and shared-image requirements by declared artifact
digest or explicit versioned external provider identity. Missing/unknown
profile, ambiguous key, unbound required dependency or mismatched bytes,
versions, layout, signature, object/root/certificate/build association
fails validation before launch. No filesystem naming convention such as
out/main is a normative root selection rule.

All artifact references resolve to the declared kind and verified digest.
Unknown fields/duplicate ids/unsorted arrays/noncanonical encodings are
invalid in schema 1. Container identity, native-byte digest, extracted-byte
digest, ISA object_hash and device closure identities remain distinct.
```

Append to §9.3:

```text
Link conflicts are checked within one linked device program, not across
independent embedded programs in one native container. Co-packaging does
not merge namespaces, layouts, ledgers or certificates. Imports actually
crossing device components require a real resolved device link or an
explicit published cross-program protocol; packaging is not resolution.
```

Append to §12.3 and cross-reference from §13:

```text
Certificate/owner/link-proof/coverage inventory covers every record required
by the selected linked device closure. It does not inventory native main,
host-only argv/envp preparation, libc or unrelated globals. Independent
device roots have independent invocation bindings and context [] relative
to each root, even when a static callee closure is shared. All reachable
device callees and required frames/proofs remain mandatory. Outer inventory
references bind each certificate to its actual object/root/build/layout;
a matching local name alone is not a certificate association.
```

This specifies a reviewable inventory convention, not ELF packing, a host shim or a new current launch ABI. The versioned packaging profile still assigns artifact locators and producer/consumer delivery; [native-sharing](native-sharing.md) supplies the separate adapter and memory obligations.

## Hashes, catalog and consumers

Outer inventory can be added around byte-identical inner objects without changing their `object_hash`. Container/native/payload digests remain different identities even when two extracted byte strings happen to match. Real device linking, new wrappers/layouts or new ISA prefixes require recomputed closure/certificate/schedule bindings. No ref recipe edit is inherent. Driver/VM/plugin/conformance consume the published inventory; filenames stay producer-private behind the adapter.

## Conformance vectors and verification

Two payloads with colliding private names; same local root name; independent layouts; actual conflicting link negative; exact byte extraction; modified slice/length/offset/version/digest; substituted certificate/build/root; missing required callee/owner/coverage; two roots sharing one static callee with separate context trees; native-only main/libc dependency positive. Preserve strict unknown inner fields. [INDEX](INDEX.md) records finite integrity/context checks and reference source checks, not native packing/loading.

## K entry: 0.9 versus 0.10

The outer schema can be a new versioned companion around 0.9 objects without changing their format. Normative link/certificate scoping can be a council clarification; default publication alongside **0.10** avoids implicit compatibility claims. Any 0.9 K route requires **council + maintainer authorization of compatibility rules**, including which certificate assumptions remain valid. Suggested decisions `MIXED-INVENTORY-1` and `DEVICE-ROOT-INVENTORY`; numeric K ids await acceptance.

## Mixed fat object

This is the direct application of [fat-object-scope.md](../2026-11-02-rev2/fat-object-scope.md) and [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F01–F03/F21. Native binary plus marked-module ISA payloads remains reachable, with explicit entry/certificate/layout/budget/ledger/run associations. Native memory by reference and native callable ABI remain the separate versioned obligations, never assumed from embedding alone. These amendments never gate rungs 3/4; F30 stays deferred.

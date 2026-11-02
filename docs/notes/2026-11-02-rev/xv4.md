# xv4 — an exclusive proof verdict for a fixed classical image

Full revised proposal for council review, 2026-11-02; `b-isa-spec-xv4`.
Supersedes `2026-11-02-contract-xv4.md`; proposed contract text only.

## Defect classification and North Star scope

ISA 0.9 §5.2 includes every interface-region cell value in D. §12.2
K69–K74 gives `proof` only for componentwise full D; successful exhaustive
strict slices receive `evidence`. Dialect 0.3 draft §§19.9/25.2 correctly
requests that existing evidence label. Assigning it is not a checker bug.

North Star §5.1 calls exhaustive verification on a declared finite domain
a proof, while §8 rung 4(a) fixes the SQLite database classically and
varies all rows and all target values. This is a certificate terminology
and scope mismatch, not missing SQLite execution semantics. The proposal
adds a conditional proof without asserting correctness on other images.
It retains all varying input, target, fault, dirty-system and inherited
control components; neither sampling nor source edits replace the rung.

## Alternatives and complete tiny witness

| Choice | Consequence |
|---|---|
| Keep K74 | Current conformance; document the terminology mismatch. |
| Call an image slice unqualified proof | Unsoundly promotes correctness to every database image. |
| Checked image-relative proof | Recommended; the image is an explicit precondition. |
| Put the database in a local image | Existing full-external-D proof for a different specialized closure and preparation route; does not relabel the original interface-region claim. |

Let an exact, fault-free oracle at a sufficient bound have one input bit
x, one target bit t, four fault bits f and one `in` region[1,8] db, with
payload `db[x] & 1`. Its complete tuple relation is
`(x,t,f,db) -> (x,t XOR (db[x]&1),f,db)`, workspace clean.

For image `0102`, x=0 flips t and x=1 preserves it: payload `1 XOR x`.
For `0201`, payload is x. Each image slice has 64 tuples; unrestricted D
has 2^22 tuples, so coverage is exactly 1/65536. An implementation
hardcoding `0102` passes the first 64 and fails all 64 for the swapped
image. Fresh bindings and replay can prove the second image separately;
its mere difference is not malformed input.

An ordinary one-bit X action with no regions has domain t=0,1 and table
0→1, 1→0. Its full-D exhaustive check must receive only `proof`, whether
fixed_regions is absent or empty. This witness catches the first note's
vacuously true image-relative condition.

At A=16,W=8, with x width 10, t width 1 and f width 4, there are 15
varying bits and 524288 fixed image bits. Coverage against unrestricted D
is exactly 1/2^524288, never floating-point zero. Prepared cells remain
interface components even though the experiment fixes them.

## Recommended exact spec changes

Extend the §12.2 domain schema and the certificate label-request/checker
outcome enumerations; add a §0.2 K entry. Replace BOTH the introductory
"otherwise evidence" rule in §12.2 and K74's blanket strict-slice
classification with the exclusive classifier below. This is a proposed
in-version certificate extension; no instruction or domain D changes.

```text
A domain MAY contain fixed_regions:[{name,A,W,image,sha256}]. An absent
field is an empty list. name resolves by the existing component rules to
an in-role interface region, with precisely the subject's A and W.
Each resolved fixed region occurs exactly once and MUST NOT also occur
in domain.regions. A mut-role region cannot be fixed by this mechanism.

image is exactly ceil(2^A*W/8) raw bytes encoded as lowercase hexadecimal;
unused high bits in the final byte MUST be zero. Cell i consists of bits
[iW,(i+1)W) in the little-endian byte image. sha256 is SHA-256 of the raw
bytes, not the hexadecimal text. The checker hashes independently and
uses those cells on every replay. Fixed cells participate in domain
validity and tuple/output checks, but each contributes one choice to n.
The full unrestricted |D| remains unchanged. This metadata introduces no
operation for inspecting unknown coherent state.

After every §12.1 replay obligation succeeds, an exhaustive
table/program/oracle claim receives exactly one outcome, in this order:
(1) proof iff its domain equals unrestricted D component by component;
(2) otherwise proof_relative_to_image iff fixed_regions is nonempty,
    every image binding is valid, every fixed region is accounted for,
    and every remaining component occurs exactly once with its full
    range (all interface bundles/cells, borrowed positions and prepended
    inherited controls included);
(3) otherwise evidence.
Successful sampled claims always receive evidence, including when their
distinct draws cover the whole domain. Malformed domains, stale hashes
and mismatches reject, rather than receiving any successful verdict.
Omission-at-zero remains a strict slice unless the original component
is thereby genuinely at its full range under the existing rules.

Coverage remains exact n/|D| against unrestricted D, with distinct-draw
coverage for sampling. The checker report identifies closure_hash,
claims-array index, invocation/context key, and ordered resolved
(region identity,A,W,image hash) bindings. A supplied label is a request,
not a producer assertion. Requested proof rejects unless the computed
outcome is proof; requested proof_relative_to_image rejects unless the
computed outcome is proof_relative_to_image. An evidence request may
accompany a stronger independently computed outcome, as before.

K75's host-limit handling remains: unchecked (host limit) discharges no
proof obligation and does not invalidate a legal domain or shrink it.
```

This deliberately preserves full-D K74 proofs and evidence for ordinary
strict slices. The sole new successful classification is a nonempty,
valid image binding with all other components fully quantified.

```text
A proof_relative_to_image theorem is conditional on the stated image.
Using it to discharge an executable invocation additionally requires
checked preparation/binding establishing that this in-region contains
exactly those classical bytes and is factorised from other systems at
entry, and that its exit image is restored. Establish preparation from
declared bytes and logical preparation operations, never inspection of
unknown coherent data. Without that preparation evidence the theorem
remains conditional and does not discharge the invocation.

This verdict cannot discharge an unrestricted borrow-frame, inverse-pair
or other full-D obligation, and cannot be reused for another image,
invocation, target realisation or schedule by discarding its bindings.
All §12.1 exact +1 basis-phase, full-tuple, clean-workspace and
external-reference obligations remain in force. A basis table cannot
prove a non-permutation realisation; its numerical/channel obligations
are unchanged. Target matching, resource accounting and error
composition are not relaxed by conditional correctness.
```

For fixed image I, write J_I for the image-preparation isometry including
prescribed clean workspace. The permutation assertion is `V J_I = P J_I`,
also after tensoring with identity on an arbitrary external reference.
When a process claim is needed, compare
`E_real ∘ Ad_J_I` and `E_ref ∘ Ad_J_I` with the complete output interface.
Controlled-unitary comparison retains a phase-sensitive bound on
`||U J_I − V J_I||`; equal uncontrolled channels alone cannot discharge it.
Numerical/channel acceptance procedures remain those of §12.1.

## Mixed fat object (bennett-4ht)

Add to §§9/10/12: "A native binary may carry VM objects only for marked
modules. The host launch manifest selects a VM entry and records the
classical image preparation/copy into its in-region, with resolved shape
and raw-byte hash. The certificate inventory is that entry's selected VM
closure and reached contexts, not the whole native executable. Image
preparation may come from host database construction or a declared
snapshot. Host bytes are bound to device cells using the declared packing,
not host pointer identity. Unmarked main, argv/envp, libc and unrelated
code remain native. The native comparison invokes the corresponding
materialized marked entry over the same image and varying domain; it
need not treat the entire binary as the oracle. Runtime shims and build
configuration are recorded as required by the North Star."

A preparation witness for this note is a host buffer `01 02` copied to
the selected two-cell VM in-region before entry and retained unchanged
through the oracle's return. Preparing an entangled region does not meet
that classical-image precondition. This says how a mixed object uses the
contract; a physical launch shim need not be implemented by this note.

## Round-1 blocking items and resolution

| Item | Change and concrete witness |
|---|---|
| sol XV4-B01: new rule overlaps full-D proof and K74 strict-slice evidence | Replace both blanket rules with one ordered, exclusive classifier; require nonempty fixed_regions. Full-D X receives proof, the 64-tuple 0102 slice receives proof_relative_to_image with 1/65536 coverage, sampling receives evidence. |

The additional round-1 suggestions are retained: a complete oracle
relation, stale-versus-fresh image mutations, explicit preparation-isometry
comparison, and separation of requested labels from checker outcomes.

## Hashes, publication and consumers

Fixed images and requested labels change certificate bytes; assigned
verdicts and resolved bindings are checker-report data. Under an
authorized 0.9 K extension retaining the semantic envelope, §9.2 excludes
certificate sections from object_hash, so source/interface/closure/object
identities, targets, schedules and ref catalog remain unchanged. A full
version bump instead requires regenerating version-envelope identities
and dependent pins. This note authorizes neither publication choice.

Producers/checkers must publish schema and verdict vectors together;
legacy schemas must not silently treat the qualified spelling as proof.
Until acceptance retain current evidence labels and exact fractions.
The SQLite rung still requires unmodified SQLite, real SQL, quantum
addressing, all rows/targets at N≤2^10, Grover at N≤2^8 and the named-target
N=2^20 estimate. Conditional correctness does not replace those gates.
xv4 remains open for acceptance and implementation.

## Conformance vectors and verification

In addition to the three exclusive-verdict cases: swapping 0102 to 0201
while retaining the old digest, expected relation or preparation binding
rejects reuse. Updating all bindings permits a fresh check, never reuse
of the old theorem. Test hashing hex text instead of bytes, wrong A/W,
nonzero padding, duplicate region identities, omitted target/fault/dirty
ranges, partial controls, a requested unqualified proof and a mut-region
binding. Identical image strings for two distinct resolved region slots
remain separate bindings. Test outer controls 0/1, arbitrary dirty states
and external references; sampled full coverage remains evidence.

This exact finite witness checks hashes, the complete 64-tuples relations,
exclusive classification premises and coverage. It is not execution of a
new checker/schema, a SQLite run or council acceptance.

```python
from fractions import Fraction as F
import hashlib
import itertools

images = (bytes([1, 2]), bytes([2, 1]))
hashes = [hashlib.sha256(i).hexdigest() for i in images]
assert hashes == [
    'a12871fee210fb8619291eaea194581cbd2531e4b23759d225f6806923f63222',
    '25dfd29c09617dcc9852281c030e5b3037a338a4712a42a21c907f259c6412a0']
assert hashes[0] != hashlib.sha256(b'0102').hexdigest()
rows = list(itertools.product(range(2), range(2), range(16)))
assert len(rows) == 64 and F(len(rows), 2**22) == F(1, 65536)
for image in images:
    for x, t, f in rows:
        payload = image[x] & 1
        after = (x, t ^ payload, f, image)
        assert (after[0], after[1] ^ payload, after[2], after[3]) == (x, t, f, image)
assert sum((images[0][x] & 1) != (images[1][x] & 1) for x, t, f in rows) == 64
assert all((images[0][x] & 1) == (1 ^ x) for x in range(2))
assert all((images[1][x] & 1) == x for x in range(2))
# Full-D X: empty image set cannot satisfy the qualified predicate.
full, fixed, remaining_full = True, [], True
assert full and not (bool(fixed) and remaining_full)
# Fixed-image slice: not full D, but every other component is full.
full, fixed, remaining_full = False, [hashes[0]], True
assert not full and bool(fixed) and remaining_full
# Omit a target range: neither proof predicate holds.
remaining_full = False
assert not full and not (bool(fixed) and remaining_full)
assert 2**16 * 8 == 524288
assert F(2**15, 2**(15 + 524288)) == F(1, 2**524288)
print('PASS xv4: two 64-tuple image relations; 64 swapped-image failures; exclusive proof predicates; exact coverage')
```

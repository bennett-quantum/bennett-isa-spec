# xv4 — proof relative to a fixed classical region image

Proposal for council review; `b-isa-spec-xv4`.

## Defect classification and North Star scope

ISA 0.9 §5.2's D includes every value of every interface-region cell;
§12.2 K69–K74 assigns `proof` only for componentwise full D. A successful
exhaustive slice is `evidence`, even if every varying component of the
actual experiment was enumerated. Dialect 0.3 draft §§19.9/25.2 correctly
requests today's evidence label. The existing checker is not defective
for assigning it.

North Star §5.1 defines proof as exhaustive over a declared finite domain,
and §8 rung 4(a) explicitly fixes the database classically before entry
while varying all rows and all target values. There is a **certificate
scope/terminology mismatch**, not missing SQLite execution semantics.
Keeping an explicitly conditional proof restores that meaning without
claiming correctness for other databases. Shrinking the varying input,
fault, target, dirty, or inherited-control domain is not an acceptable
substitute; neither are SQLite source edits or a sampled row check.
The exact underlying machine capability is already present. Under the
rubric, judge the gap against the promised proof meaning, not as a
failure of existing K74 replay or of the rung-4 evidence gate.

## Alternatives and worked example

For a tiny oracle with one row-index bit, one target bit, four fault bits,
and one `in` region[1,8], the unrestricted D has 22 bits. Fix image bytes
`01 02`; enumerate all x,t,f values: 64 cases. That is fraction
`2^6/2^22 = 1/65536` of full D, so current K74 says evidence. All 64
cases nonetheless prove the oracle relation for this image if the
realisation is exact phase-free permutation code with full tuple checks.
An implementation that always substitutes bytes `01 02` agrees on this
slice and fails on `02 01`; the conditional proof must not certify it
for the latter image.

At the supplied rung sizes A=16,W=8, x has 10 bits, t one bit and f
four bits: 15 varying bits and 524288 fixed region bits. The fraction
is exactly `1/2^524288` when no further components are present. Do not
round it to zero. Prepared image bytes are a precondition, not absent
interface components.

| Alternative | Consequence |
|---|---|
| Keep K74 unchanged | Valid current conformance, but a finite exhaustive fixed-image experiment keeps the evidence label; document the North Star terminology gap explicitly. |
| Call the slice an unqualified proof | Unsound promotion to all database images. |
| Conditional proof with checked binding (recommended) | Proves exactly the prepared-image relation and keeps unconditional K74 proof distinct. |
| Embed the database as a local region image | Can prove that specialized closure on its full external D; changes identity and preparation route, and does not relabel the original interface-region claim. |

## Recommended exact spec diff

Extend §12.2 domain records and add a §0.2 K entry:

```text
A domain MAY additionally contain fixed_regions:[{name,A,W,image,sha256}].
name resolves to an in-role interface region by the existing component
rules. image is exactly ceil(2^A*W/8) bytes as lowercase hexadecimal;
unused high bits of the final byte, if any, MUST be zero. Cell i is bits
[iW,(i+1)W) of that little-endian byte image. sha256 is SHA-256 of these
raw bytes, not their hexadecimal spelling. A fixed region must occur
exactly once, cannot also occur in domain.regions, and must have the
subject's exact A and W. The checker independently hashes the image and
uses its cells on every replay. Mut-role regions cannot use this binding.

An exhaustive table/program/oracle claim that succeeds on every element,
has every non-fixed component at its full range, and binds all remaining
components only by fixed_regions receives proof_relative_to_image.
The checker reports the region closure hash, claim index, invocation or
context key, and ordered (region identity,A,W,image hash) bindings with
that verdict. It also reports coverage against the original unrestricted
D. This is a proof only on the stated fixed-image subspace. Sampled
claims always remain evidence, including on that subspace. Unqualified
proof retains K74's full-D condition.

Use of proof_relative_to_image for an executable invocation additionally
requires a checked preparation/binding establishing that the in-region
has exactly that classical image and is factorised from other systems
at entry, and that the region restores it at exit. Preparation is checked
from the declared classical bytes and logical preparation operations,
not by inspecting unknown coherent data. Without this precondition the
claim remains a conditional theorem and cannot discharge that invocation.
This verdict cannot discharge a full-D borrow-frame, inverse-pair or
other unrestricted obligation, nor be silently reused for another image.
All phase, full-tuple, external-reference and invocation-binding
obligations of §12.1 continue to apply.
```

Extend the checker verdict enumeration to include the spelling above;
requested `proof_relative_to_image` must be checked just like K74's
requested labels. Non-permutation lowerings retain process/channel
obligations; this is not permission to certify them from a basis table.
Where such a conditional process claim is needed, explicitly compare
the channel after the same image-preparation isometry on both sides.

## Hashes, catalog and consumers

No source closure, interface, target or catalog bytes change. The
certificate's fixed image and conditional verdict alter its own bytes;
an image change invalidates the binding even if the region hash is the
same. Plugin, checker and rung gates must retain the qualified verdict
and hash visibly. Schemas must not silently accept it as old `proof`.
Until acceptance consumers must keep current evidence labels and exact
fractions; the new spelling is only proposed here.

## Conformance vectors and verification

Use the 64-case tiny example; its raw-byte hash is computed in
[contract-verification](2026-11-02-contract-verification.md). Change one
image byte, swap cells, change W, hash the hex text, omit a target/fault
range, or request unqualified proof: reject that requested promotion.
Sampling all 64 cases remains evidence. Two identical image strings at
different resolved region identities are separate bindings. Test outer
control 0/1 and dirty systems in full; a preparation entangled with a
reference does not meet the fixed-image premise. These are proposal
vectors; neither a rung-4 gate nor a new checker test was run here.

## Release and after acceptance

Recommend a **0.9 K certificate clarification/extension** if the council
accepts a new qualified outcome; preserve every old K74 label. Add it to
schema and checker vectors together before revising rung acceptance.
The broader North Star rung remains unmodified SQLite, real SQL,
quantum-addressed memory, all rows/targets at N≤2^10, Grover at N≤2^8,
and the N=2^20 resource estimate. xv4 stays open for that decision.

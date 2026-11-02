# 2cg — distinguish exact primitive history from a minimal C-wrapper fixture

Council proposal/disposition for `b-dialect-spec-2cg` (dialect repository).

## Defect and North Star scope

Both maintained dialect texts, §17.1 “C allocation alignment”, require
ISA §7.1's exact primitive bump, requested-size record and nonreclaiming
history. C payload extents are ABI-rounded; alignment padding is explicit
and minimal. Neither clause publishes a runtime query for old requested
sizes/liveness. The bead asks whether the exact three-record
`scripts/0212_allocation.c` witness also limits every production allocator.

The actual fixture calls itself an **independent primitive/wrapper trace
fixture, not a production allocator**. Runtime ordinary-bitcode bodies
are materialised on both sides (dialect §§17.7/19.1). On that reading,
ordinary charged bookkeeping is already expressible. This is a genuine
wording/conformance-boundary ambiguity worth resolving, **not evidence
that a new ISA allocation-query instruction is necessary**, and not a
remaining unimplemented realloc in the originating runtime. Its later
source/request supersedes the earlier blocked-query interpretation.

North Star §§3.2/5/6.1 and §8 rungs 4–5 requires real libc, unmodified
SQLite and allocator-based Julia collections. A fixed record cap,
native-only allocation query, copying past the old payload or dropping
realloc would narrow scope. Ordinary dynamic metadata preserves it;
actual added budget/history costs must remain visible and reversible.

## Alternatives and worked witness

Origin read: runtimes `.tmp/sol-runtimes-queue/cross-repo-beads.md`,
“resolve realloc size/liveness bookkeeping”, its implementation update,
and “wider allocator re-pin”; `src/allocator/bump.c` was inspected
read-only. It keeps requested sizes/live records in explicit 32-byte
heap records, reuses freed metadata slots as data, and uses an explicit
zero-size primitive probe before alignment. It does not reclaim payload
addresses or access the ISA's private allocation table.

With heap base 4096, alignment 16 and an initial raw `alloc(3)`:

| Program/body | Subsequent primitive requests | Payload base | Final bump | Records including raw alloc |
|---|---|---:|---:|---:|
| Minimal fixture `cmalloc(1)` | 13 padding, 16 payload | 4112 | 4128 | 3 |
| Inspected production `malloc(1)`, empty metadata pool | 0 probe, 12 padding, 16 payload, 32 metadata | 4112 | 4160 | 5 |

The zero-size primitive advances one byte but records requested size 0
(ISA §7.1). The production body therefore needs 64 heap bytes in this
fresh case, rather than 32. With a 32-byte heap its metadata allocation
faults after the successful probe/padding/payload prefix; ordinary
function calls do not imply transaction-wide rollback. Reversal restores
the prefix's records and stores by the ordinary reversible history.

The old-size information witness is also valid: allocations 16 then 32,
or 32 then 16, have the same first pointer p and final bump p+48. A
cursor alone cannot tell whether `realloc(p,48)` preserves 16 or 32
bytes. This proves a need to retain sizes, not a need for a privileged
query: a side list written at each allocation supplies the information.

| Alternative | Assessment |
|---|---|
| Treat the minimal fixture as the only allowed production trace | Requires another published metadata mechanism to retain full libc scope. |
| Publish a new allocation-size/liveness primitive | Possible larger contract change; unnecessary for the inspected implementation. |
| Permit ordinary explicit charged bookkeeping | Keeps exact primitive semantics and native/device equality. Recommended. |

## Recommended exact spec diff

In **both** maintained maps append to §17.1 after the allocation vector:

```text
The allocation vector fixes the trace of its stated thin wrapper,
not a universal trace for every implementation of malloc/calloc/realloc.
Runtime bodies may retain requested size, liveness and alignment state
in ordinary materialised memory, including dynamically allocated metadata
and explicit zero-size address probes. Every such operation, its stores,
padding and allocation history is present identically in device and
native materialisation and charged to the actual heap/step/table budgets.
The C payload request retains the rounded extent and returned-base rule
above; padding immediately before that payload is minimal from the
actual current primitive bump. No hidden header, private ISA-table query,
native-only map or fixed metadata cap is implied or permitted by this
allowance. Reusing live metadata storage as ordinary data does not reclaim
primitive payload addresses or allocation-table/history entries.

realloc preserves min(old requested size,new size) bytes of a live C
allocation, using retained ordinary metadata or another published
mechanism. Invalid/dead/interior pointers retain the materialised
memory/fault contract. Overflow preflights precede the corresponding
allocation mutation; a later primitive failure retains the successfully
executed ordinary prefix unless explicitly reversed. Both comparison
builds execute the same body, including its failure prefix. Tests of a
production runtime bind that body's identity and exact resulting trace;
they do not impose the thin fixture's record count on a different body.
```

Keep the three-record fixture and its output exactly as published. Add a
separate tagged production-body vector to the dialect/runtime acceptance
corpus, not a replacement that weakens primitive history checks. This
proposal grants no blanket memory-check exemption to runtime functions.

## Hashes, catalog and consumers

ISA primitive semantics and ref catalog bytes stay fixed. Different
materialised runtime bodies change bitcode/native manifests, dialect
materialisation identity, generated ISA closures, resource costs and
consumer pins; exact addresses may change too. A dialect wording revision
must obey its full-version/mapping policy. Driver/plugin/conformance must
re-pin and budget the actual body, not transplant minimal fixture costs.
No sibling publication or tracker database is written here.

## Conformance vectors and verification

Check both traces above, min(16,48)=16 versus min(32,48)=32, shrink,
null/zero-size, dead/interior pointer, direct intervening primitive alloc,
pool-slot reuse, first stronger alignment and failures at each primitive.
Reverse all actual metadata/list stores and primitive records and replay
identically; never test only the returned payload. Exact hand arithmetic
is checked in [INDEX](INDEX.md). The runtime's reported LLVM18/23 gates
were read as originating evidence, not rerun here; this job runs no
full sibling gate and claims no SQLite/Julia rung acceptance.

## Release and route

Recommend a **current dialect §17.1 clarification**, in the 0.2 and 0.3
maps under their publication rules; no ISA 0.9 K or ISA 0.10 dependency.
Retain related jjr/0na and re-pin work with their owners as appropriate.
The orchestrator can attach this note to 2cg; no new cross-repo bead is
needed because the decision is already tracked there.

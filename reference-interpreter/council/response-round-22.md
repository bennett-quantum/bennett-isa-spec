# Editor response: council round 22

**CODEX-R22-B01 and SOL-R22-B01 are resolved.** The former is the deeper
retention continuation of CODEX-R21-B02. The latter is an independent false
Proof caused by coverage by name. Both seats' supplied witnesses are retained.
No ISA width/length cap, cross-role name ban or new theorem claim was introduced.

Implementation commit: 73eb62049e7624541b58730c59c417e061f112e0.
Interpreter evidence commit: 59d6114febd012a5e20eb4814209df5a8174c954.

## Blocker disposition

| Blocker | RED tests / observations | Fix location | Commit |
|---|---|---|---|
| CODEX-R22-B01 | `r22_retention::r22_b01_request_retention_and_observers`: child SIGABRT after feasible 64 MiB transfer, derived request clone; original reviewer backtrace pins history construction | `retained.rs`; `classical.rs::{step,reply,outcome,snapshot,host_transcript}`; `oracle.rs::Memory`; CLI observer/serialization consumers | 73eb62049e7624541b58730c59c417e061f112e0 |
| SOL-R22-B01 | `r22_coverage::sol_r22_b01_false_proof_and_controls`: actual Proof, required Evidence(4) | `claims.rs::{Component,Object::domain}`; `certificate.rs::domain` role-scoped duplicate checks | 73eb62049e7624541b58730c59c417e061f112e0 |

### Host request and subsequent retention

Determining clauses: §10 copies bytes into the request and retains request,
overwritten and reply bytes in the frame; §7.4 retains that frame. The initial
request transfer and history vector reserve fallibly before publication.
`HostRequest` is now a shared immutable descriptor, and request/reply payloads,
undo bytes, finished outcomes, console and tape use immutable `Retained<T>`.
It holds an `Arc<Vec<T>>` with a visible prefix length. A reserved Vec moves
into the owner; `Arc<[T]>::from(Vec<T>)` would perform another full allocation
and copy. Request/history/outcome/transcript/snapshot clones only share the
payload. Truncation for reversal changes a prefix length, without a copy.

The exact 64 MiB reviewer object now creates a valid request under the same
128 MiB address-space limit. Its five successful lengths (0/1/8/1 MiB/64 MiB)
survive outcome, pending snapshot, snapshot clone and transcript access. The
exact CLI reaches HOST_REQUEST, reports the missing reply, and exits 1 with
empty stdout. That is the expected suspended behavior when sharing makes the
request fit; a forced rejection is not claimed or required for this witness.
The file remains SHA-256
`df7f95dd72496eb446c39319d5ef1e6287ebed8fdd6b9a7f92eba31ba1431071`,
with literal interface=closure envelope
`655e72211b66a9e4e3f8dcaff74c5203da6ec1d834fc43570833f0ab1387a577`.

The later-failure regression retains that 64 MiB request and supplies an
8 MiB reply. The first reply-byte reservation succeeds; the subsequent
`retained memory bytes` reservation is denied. It must return a typed,
§10/§7.1/§7.4-cited host-resource Result, preserve the whole suspended snapshot,
leave the transcript reply empty and permit a subsequent empty reply followed
by exact reversal to entry. Console's 64 MiB first transfer likewise succeeds
but its second, cumulative console reservation fails before publication;
full prefix equality, absent tape/output/request and reversal are asserted.
The original 2^63 hostcall/console witnesses still reject at the first transfer
reservation, and all 0/1/8-byte controls still succeed.

Reply prepares reply storage, overwritten-byte history, a shadow sparse memory
and result retention before any published change. Sparse resident bytes now
use a sorted shared vector with fallible copy-on-write, so a snapshot does not
clone an unbounded tree of resident reply bytes. A successful 256 KiB nonzero
reply exercises memory reads, snapshots held through reversal, transcript
sharing, digest and undo. Memory stores and device writes prepare their sparse
capacity before the first write. This representation preserves sorted digest
bytes and sparse virtual allocations; random insertion has vector insertion
cost, a performance tradeoff rather than a semantic limit.

`host_transcript`, memory digest and snapshot delta now return Results;
`host_exchanges` permits borrowed iteration. CLI hex/UTF-8 expansion,
reply decoding/queue, trace record/delta vectors and final JSON storage reserve
fallibly. Payload Debug reports lengths rather than formatting all bytes.
Fixed 8/512-byte device operations remain fixed-size operations. Unrelated
control, register/quantum metadata and fixed-size Arc owners still use ordinary
Rust allocations: this finite transfer audit is not a proof of recovery from
arbitrary process OOM or every resident-metadata growth pattern.

### Retention and later-copy inventory

**12 tagged stages / 53 stage cases**, an extension of the retained R21
24-path/186-case and R20 102-path/1,113-case families. These are overlapping
stage observations, not a disjoint 36-path total or 53 distinct programs.

| Path / stage | Change and determining check | Cases |
|---|---|---:|
| `ClassicalMachine::step`: initial host transfer → pending/history | Fallible first buffer and frame capacity; shared request descriptor/payload; lengths 0/1/8/1 MiB/64 MiB/2^63 | 6 |
| `outcome` / `run` request observer | Shared descriptor and payload at all five feasible lengths | 5 |
| Pending `snapshot` and `ClassicalSnapshot::clone` | Share request bytes at all five feasible lengths; full equality | 5 |
| `host_transcript` request observer | Fallible entry vector; shared request bytes at all five feasible lengths | 5 |
| `reply` / `save_bytes`: reply → overwritten-byte history | Four empty-reply controls; 64 MiB request + 8 MiB reply reaches denied undo reservation, unchanged suspended prefix, recovery/reversal | 5 |
| Nonzero reply → sparse memory / snapshot / transcript / undo / digest | Fallible sparse writes and shared resident bytes; 256 KiB nonzero reply succeeds with observers held through reversal | 1 |
| `effect(console.write)`: transfer → cumulative console | Four successes; 64 MiB later rejection; 2^63 first-reservation rejection, exact rollback/reversal | 6 |
| Console/tape snapshot retention | Shared console and tape payloads, successful 0/1/8/1 MiB snapshots | 4 |
| `Retained::clone` direct payload ownership | Pointer equality and value equality, 0/1/8/1 MiB/64 MiB | 5 |
| `Retained::prepare` copy-on-write retention | Four successful appends; 64 MiB first buffer fits, required later copy denied without changing either owner | 5 |
| `Json::try_canonical` final payload serialization | Four successful sizes; feasible 64 MiB source followed by denied JSON output reserve; source unchanged | 5 |
| Exact pinned CLI witness | 64 MiB request reaches normal suspension / missing-reply error; no signal or partial JSON | 1 |
| **Total** | | **53** |

The empty-reply controls in the fifth row do not claim a nonempty undo copy;
the single 8 MiB denied-undo case and the separate nonzero successful reply
establish those stages. Mathematical extremes and pins are not substitutes for
later-copy coverage. All new memory families run in fresh Linux children with
RLIMIT_AS=128 MiB, RLIMIT_CORE=0 and `MALLOC_ARENA_MAX=1`; catch_unwind inside
children catches panics, while the parent rejects process signals. An initial
full-run unit probe exposed glibc's unrelated per-thread 64 MiB arena
reservation, which could deny the first buffer. Arena control stabilizes
headroom; no test accepts early rejection as evidence of a later stage.

### Further defects exposed and suggestions

The later-copy audit reproduced another allocator SIGABRT in legacy
`Json::canonical`: after constructing a feasible 64 MiB string, quoting cloned
that complete string infallibly. The RED subprocess log proves the first
reservation/initialization succeeded. Executed CLI payload output now calls
`Json::try_canonical`, which appends recursively into one fallibly growing
buffer, with fallible object-key ordering and no child-string clone/join.
The source remains unchanged on resource Err. The legacy canonical method is
retained for metadata/identity consumers; none of the audited CC/R output
paths uses it. Exact escaping/UTF-16 key-order controls agree with independent
literal output and the legacy canonical bytes.

Both codex suggestions are applied. The resource-stage suggestion is the
53-case family above. The previously deferred diagnostic migration is now
coordinated across the crate: `ErrorKind::HostResource`, explicit constructors
and category-preserving context distinguish host limits from diagnostic errors
without substring classification. Display strings and rule context are
preserved; ISA memory/trap/entropy faults remain outcome codes, not resource
errors. `trap` propagates resource errors rather than relabelling them as ISA
traps. Public API migrations are documented in README (`Error::new` instead of
the old tuple constructor; immutable retained values; Result-returning observer
APIs). Existing assertions were adapted without removing their expectations.

### Role-sensitive claim coverage

`Component::Bundle(group, slot)` alone discharges the corresponding bundle
obligation. Region coverage is checked over its own cells: `all`, or a complete
explicit list with every cell's full range. A borrowed component has its own
ordinal and never contributes an interface identity. This implements the
argument of [INV-6 §1.3.4](../../../../../bennett-conformance/proofs/INV-6-certificate-soundness.md),
with renaming invariance from §1.3.5 and the name-classifier refutation in
§1.5.5. It does not claim to discharge the proof's separately listed spec
readings or its open host-outcome clause.

The CNOT witness parses/checks hashes and now returns Evidence { elements: 4 },
identically to the borrowed-name-renamed control. The actual full-domain
variant retains the §12.1 mismatch at element 4. Its original file SHA-256 is
`c9cf4ac51d800f9a918deec3a14790691d0b9c2d55b395949add620425707cca`.
The separately supplied `domain-role-spoof.bisa` table variant, SHA-256
`19456db6726bdce2b10f7ae983a243c5cf2c2682b4d707aced28f81d6206e249`,
now returns Evidence { elements: 2 }; three borrowed spellings have exactly the
same verdict. An asserted written `label:"proof"` cannot upgrade that result.

A **32-case public replay family** varies coverage subsets, full/partial ranges
and four borrowed names, including collisions with real interface bundles. It
asserts both Proof iff the real bundle domain is complete and exact verdict /
count invariance under borrowed renaming. **256 direct coverage cases** add
region-name collisions, `cells:"all"`, complete explicit lists, empty lists and
partial cell ranges. This family also exposed a pre-existing admission check
that prohibited repeated spellings across roles. Duplicate checks are now
role-scoped; duplicates within a role still reject. No new cross-role name ban
was introduced. The original program/control/table files are copied byte-exact.

## Validation and retained evidence

Baseline: **436 passed / 0 failed / 5 ignored**, **112.97 s** runner sum over
**34 result blocks**. Final: **451 passed / 0 failed / 5 ignored**,
**137.93 s** over **36 blocks**, **20 unit + 431 integration**, 0 doctests.
All round 4–22 tests retained. Focused R22: **15 passed / 0 failed**.

All five ignored release sweeps: **5 passed / 0 failed**, **84.94 s**.

| Sweep | Full domain | Result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 pairs × 4 = 262,144 executions | 1/0; 0.42 s |
| 17-wire borrow | 131,072 entries | 1/0; 2.21 s |
| checked borrow certificate | 262,144 elements, subject and expected program | 1/0; 11.73 s |
| §6 differential | 200,000 programs | 1/0; 67.78 s |
| recursive prefix reversal | 256 inputs × every completion/first-fault prefix | 1/0; 2.80 s |

Fmt/check and strict offline all-target Clippy pass with no warnings. The
unchanged Python seed verifies two examples and two literal envelopes. R20 and
R21 independent pins pass; R22's independent Python pin script checks all
12 complete reviewer artifacts, including five objects, envelope, backtrace,
results and probes. All copies compare byte-for-byte with the read-only
reviewer sources. Rust also checks the exact 64 MiB file/envelope/hash pin.
Production source/test inputs stayed fixed during the final validation run.

Logs: interpreter `.tmp/r22-{baseline,red,sol-red,json-red,family,final,sweeps,
clippy,fmt}.log`. Initial harness/admission-control exploratory failures are
superseded by the final passing gates; they are not claimed as later-stage RED
evidence. The two abort RED logs and the false-Proof RED log are independent.

Every cargo invocation used the required headless prefix and project TMPDIR;
commands: offline manifest `cargo test`; release `cargo test -- --ignored
--nocapture`; `cargo fmt` and fmt/check; offline `cargo clippy --all-targets --
-D warnings`. Beads **b-isa-spec-949** and **b-isa-spec-7ld** record the defects
and follow-up discoveries. Only `reference-interpreter/` and this response are
committed; no spec/conformance/worklog/foreign-file writes, push or Dolt sync.
Beads auto-export and the session worklog are left for the orchestrator.

Deliverables 8–11 remain staged/Unverified. Full ISA scope and other-value
obligations remain. Both round-22 seats' supplied items are addressed; additional
orchestrator steering can be appended here without rewriting prior responses.

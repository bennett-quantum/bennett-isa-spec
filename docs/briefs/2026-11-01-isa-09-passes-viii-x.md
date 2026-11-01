# ISA 0.9 passes VIII–X — council round 8 brief

One reviewer covers all three passes. Core three-seat acceptance and
round-7 acceptance stand; 9eq, 7js and 5cd remain in_progress for this round.
Accepted 0.8 and the 219-recipe catalog bytes/envelope are unchanged.

| Pass / bead | Change and clauses | Evidence / review boundary |
|---|---|---|
| VIII / 9eq / K78 | §5.6 rule 2 resolves x/cx/ccx/mcx to target plus effective control literals; qxor/qexch contribute §4.5 per-bit cx/swap events. §4.4 and §5.3 support operation identity. All-operand top-event matching, K76's domain, source hashes and §11 occurrence identities remain. | `reference-interpreter/tests/frame_identity.rs`: three regressions already green against the unchanged reference before wording. Mixed-spelling positives, reader/control and polynomial-prefix negatives. This is entry analysis only, with no circuit-equivalence optimizer. |
| IX / 7js / K79 | §4.6 requires exactly A address wires for all four coherent memory rows; §16.2 spells address-only `()` when A=0, binding catalog `[]` (§11.1). `wire[0]` and empty values remain excluded (§3.3). Optional §3.3/§4.8 binop/icmp wording records the accepted integer/pointer comparison distinction. | `tests/memory_address.rs`: four A=0 positives were red before source alignment; eight final regressions include A=1/2 width negatives, confined empty actuals, local regions, inherited controls and catalog bindings. No catalog edit or new out-of-range no-op. |
| X / 5cd / K80 | §11 K65 still selects one whole cleanup before expansion. New “Cleanup leaf ownership” keeps reverse's written source points; recompute's replacement gets the carrier's coordinates and one fresh caller leaf/call space. §6.2/§6.4 fix the paired sequence and semantics; §11.1/K63 keep callee spaces separate. Raw inverse retains forward identity; K62 source adjoints and K64 support partition remain. | `tests/cleanup_ownership.rs`: three regressions green before text edit. Printed six-row witness has reverse entry:3/4/5 ordinal 0, recompute entry:5 ordinals 0–7. One-CX semantic callees give fresh call_index 0/1 then 2/3, separate from original entry:1/2 calls. Source hash/check, basis/coherent reverse identity, concrete cleanup/reversal and real selector-engine checks. |

Pass X adopts Reading A for the newly constructed recompute expansion and
rejects its optional rebinding of reverse. Literal reference action execution
(`src/action.rs`, qalloc/qfree, qcopy/quncopy and call branches) preserves
written instruction structure. `conformance/src/ref_language.rs`'s `emit`,
`invoke` and semantic `call` branches give newly instantiated caller leaves
contiguous ordinals, opaque distinct callee paths and separate callee spaces.
Neither that expander nor the reference provides a general 0.9 multirow
source-cleanup builder: the regression's finite adapter supplies source
coordinates and serialized call_index explicitly. It is evidence for the
chosen convention and semantics, not an existing implementation of K80 or
full 0.9 executor admission. No reference src change is claimed for pass X.
The ambiguity is real: K65 fixes carrier/selection, K64 partitions oracle
support, and §11.1 numbers within a row without assigning replacement rows.

Review focus: K78's normalization boundary; K79's address-only zero-width
exception; K80's ownership, fresh uncomputation call occurrences and raw
inverse numbering. Each pass adds one MUST and one inventory row only;
K1–K77 predate this round. The exact text, disposition tables and gate counts
are recorded in the corresponding top blocks of `spec/isa-0.9.PROGRESS.md`.
The orchestrator owns the council run, session worklog and downstream
statement-layer regeneration; this implementation does not edit siblings.

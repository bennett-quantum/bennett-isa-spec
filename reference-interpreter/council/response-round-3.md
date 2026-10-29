# Editor response: council round 3

Input so far: SOL out-sol.json, reviewed snapshot 2e61144. This response is
appendable for the pending astra/codex and muse second pass. Only the five
received SOL blockers are covered by this pass. No spec or Python seed edits.

## SOL-R3-B01 — resolved

`src/call_discipline.rs::validate_matched_oracle_calls` records both payload
and fault deliveries for calls and uncalls as writes. Every outstanding input
reader is invalidated; only output XOR delivery retains the commuting exception.
The inverse consumes its own pre-row identity before invalidating other readers.
Pin: `r1_rules::r3_nested_oracle_deliveries_invalidate_outstanding_inputs`:
exact inc8 witness, another uncall delivery, fault delivery, parser and CLI
hash/check rejection (§5.5, exit 1, empty stdout); no-middle positive gives
(payload,code,steps)=(0,0,4), interpreter/machine agreement and prefix reversal.
Existing two-call/two-uncall multiplicity positive remains accepted.
RED: exact witness was accepted; GREEN: all 16 r1_rules tests pass.
No SPEC-QUESTIONS entry: §§4.2 and 5.5 explicitly specify the writes.

## SOL-R3-B02 — resolved

`validation.rs::validate_action_sequence` snapshots all mutable bundles and
mutable-region cells jointly, with immutable interface/cell/control reads;
`Discipline::joint_change` generalizes exchange tracking to forward/adjoint
call pairs. Every output inherits the OLD transitive union and other provenance.
Cancellation requires the complete output group, entry versions and unchanged
read/control versions; outside readers block other-value cleanup. Mutable cell
epochs and non-permutation window closures still advance on every call.
Pins: `r1_discipline::r3_action_calls_transfer_the_old_joint_dependencies`
(exact call-wrapped swap witness and mutable-region variant reject §6.2;
doubled-CX positives are exact identities on all basis inputs and superposition,
forward/adjoint). `r1_fuzz_discipline::generated_child_call_wrapping_preserves_bell_cleanup_obligations`
generates 12 direct/wrapped, odd/even CX variants; independent Bell/parity
reasoning requires rejection or clean execution over the full small interface.
RED: exact witness accepted. GREEN: 17 discipline + 3 active generator tests.
No SPEC-QUESTIONS entry: this is the explicit §6.1 union and §6.2 identity rule.

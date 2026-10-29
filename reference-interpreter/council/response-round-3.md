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

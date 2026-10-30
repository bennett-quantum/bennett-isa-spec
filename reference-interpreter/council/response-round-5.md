# Editor response: council round 5

Input so far: SOL out-sol.json against 5d6d03e. This response remains
appendable for the pending codex/muse second pass. ISA 0.8, the Python seed,
random seeds and conformance sources are unchanged; owned paths are exclusively
reference-interpreter/.

## SOL-R5-B01 — resolved

RED: `r1_certificates::sol_r5_b01_certificates_cannot_waive_completed_lifetimes`
accepted the exact certified X witness. Both literal interface envelopes and
object digests are independently SHA-256 checked before parsing; construction
does not parse an invalid uncertified subject. X and CX subjects, with and
without correctly bound certificates, now reject at Object::parse and CLI
hash/check (exit 1, empty stdout, §5.6).

`borrow.rs` distinguishes a definite Boolean/non-permutation failure from an
unresolved opaque-function identity. Only the latter can invoke the library
route. `claims.rs::check_borrow_independence` additionally audits every actual
qreturn during full-domain replay, including inlined/nested action calls.
`quantum.rs` retains opt-in basis observations: the returned factor must equal
its entry, and every other live wire must be independent of that entry at that
boundary. Comparisons hold all other input bits fixed. Prior completed frames
have already restored their independent entries, so subsequent physical pool
reuse cannot conceal an invalid earlier frame. Audited replay visits every
iteration rather than skipping a period with unobserved internal boundaries.
Normal execution retains cycle skipping and allocates no audit records.

The existing round-4 lifetime negatives, carried-even-loop and doubled-X/CX
positives, entangled-reference tests and round-4 certificate/structural pins
remain green: targeted borrow/certificate run **24 passed, 0 failed, 2 ignored**.
No new ambiguity or blocker decline. §4.3 requires qreturn to be the matched
inverse after its frame; §§5.1/5.6 require the property per completed lifetime.
Final whole-suite/conformance verification follows after all received blockers.

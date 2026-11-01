# Current state — 2026-11-01 14:51:43 UTC

Pass XIII completes the requested round-9 acceptance/S1 follow-up.
Passes XI–XII: round 9 accepted by the codex seat, zero blocking.
K81 accepted in fix round 9; K1–K81 accepted in total. Core three-seat
acceptance stands. b-isa-spec-9hv is closed (accepted round 9), exported
with `bd export -o .beads/issues.jsonl`. b-isa-spec-fuk remains open.

Reference README defines saved-tape profile v1 as a versioned tool contract,
including the reviewer's exact journal rules, strict Tape[] encoding,
prefix/exhaustion, transactional mismatch and no-stdout/no-output-write
diagnostics. It states bennett-vm implements the same profile; no VM edit
or independent VM verification. ISA 0.9 §7.3 adds exactly:

“The active effect tape is specified here; persistence and replay of saved execution journals are tool interfaces, whose interchange format and validation behavior are defined by an explicitly versioned replay profile.”

No new K row or ISA interoperability MUST. ISA determines valid execution
and reversal for the same complete explicit inputs. Reference admission
remains 0.8; no production 0.9 source partition builder is claimed.

§15 links conformance/fixtures/accepted-k81-09/README.md and k81.json.
The two joint-swap release orders assign inverse 3 and release 4 to carrier
4; carrier 5 owns only release 5. First release owns the indivisible inverse,
including when b releases first. No independent carrier 3. A duplicate
maximal tie at 4 rejects while 5 selects independently.
`portable_k81_joint_first_release_vectors_exact_bytes` in
reference-interpreter/tests/cleanup_partition.rs derives and byte-locks
source text, exact owners, real selector diagnostics and eight basis tuples.
It executes admitted/hash-checked source forward/adjoint, checks callee
coherent/entangled behavior and exact state/allocator restoration.
Coordinates/ownership are explicitly finite-source-adapter, not production
builder output; catalog-expander leaves are null, not fabricated.
Initial red failed only because the fixture was absent; explicit regeneration
and ordinary no-write byte-lock run passed.

All four required gates exit 0, using DISPLAY, WAYLAND_DISPLAY,
DBUS_SESSION_BUS_ADDRESS and BROWSER unset, TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp,
and offline Cargo:
- Reference: 504 passed, 0 failed, 5 unchanged ignored (503 + one new test).
- Conformance: 113 passed, 0 failed, 0 ignored.
- python3 -I conformance/tests/draft_witnesses.py: PASS.
- python3 -I conformance/tools/ref_catalog.py --check: PASS, 219 recipes,
  envelope 5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197.
Cargo logs: .tmp/pass-xiii-reference-interpreter.log,
.tmp/pass-xiii-conformance.log; initial red: .tmp/pass-xiii-k81-red.log.
Owned rustfmt and whitespace checks pass. Exact-label generated diff verified:
a/spec/Bennett-ISA-Spec.md and b/spec/Bennett-ISA-Spec-0.9.md.
ISA 0.8/catalog bytes match e4eb657; all K inventory rows unchanged.

Own commits: 10e1607 (acceptance/profile), 5513d78 (K81 vector/test/§15),
293c266 (reference green). Final gate record and this handoff are committed
next; final report supplies HEAD. No push, sibling writes or remote/Dolt sync.
Orchestrator retains session-worklog and generated council attachment ownership.
Preserve pre-existing .tmp/ and gate logs; expected final status: only ?? .tmp/.

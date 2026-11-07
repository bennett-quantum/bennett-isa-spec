# ISA reference interpreter: quality batch 1, review 2

**Verdict: BLOCK — one residual P0; no P1.** No implementation files changed.

Reviewed head: `cdef3eb184e2aab9dcf97bde5fd3be1fcf6e1982` (`job/quality-isaref`).
Batch inventory: `git log --reverse main..job/quality-isaref` and
`git diff main...job/quality-isaref`; round-2 review focuses on
`b572e17..cdef3eb` (`92fbe5c`, `75cec86`, `cdef3eb`) and reproduces the
three findings in `2026-11-06-quality-isaref-r1.md`.

## Blocking finding

**P0-1 — Unresolved bound-root applicability disappears when the owner map is empty.
Bead `b-isa-spec-lfbd` (residual of `b-isa-spec-jple`).**
`reference-interpreter/src/claims/qinplace09.rs:142`–144 records an unsupported
CC/R root as unresolved, but lines 179–182 attach that outcome only to already
inventoried direct qinplace owners. If none of the supplied records directly
contains qinplace, the map is empty and the obligation silently disappears.
`reference-interpreter/src/claims.rs:923` consequently reports applicability
`None`, rather than `Unestablished`.

Repro from `tests/isa09_qinplace.rs::transitive_certificate(false)`:
the linked path is `run` (CC) → `addya` → `inner` → qinplace. The certificate
contains only the outer `addya` record and its valid inverse-pair theorem;
the required `inner` owner entry is absent. Change `bindings.closure_hash`,
`reference_program_hash` and `interface_hash` to the identities of `run`, and
set `bindings.invocation = root_invocation(2, 1)`. Parsing succeeds.
`verify_certificate()` returns one `Proof` with `qinplace_applicability: None`;
`verify_claim(addya, 0)` also returns `Proof`. Removing the outer record’s
claims and inverse-pair entries instead returns `Ok([])`, still without an
unestablished outcome. Executing the corresponding `addya` invocation at
`tk=3,a=1` rejects `no covering inverse-pair entry in @inner` before effects.

ISA 0.9 §12.3 K92 requires bound transitive coverage, and explicitly requires
an unestablished obligation when an invocation/context cannot be checked.
Keeping the standalone theorem is permitted; omitting the unresolved
applicability obligation is not. An unsupported root needs an observable
obligation even when the actual owner record or all claim reports are absent.

Ran `timeout 60 .tmp/rev-quality-isaref-r2/tmp/review-probe` and
`timeout 60 .tmp/rev-quality-isaref-r2/tmp/review-applicability-probe`: both
exit 0, asserting these observations. Probes were compiled from stdin with
`timeout 120 rustc`, included the existing qinplace fixture helpers, and
linked against the freshly built review rlib. Only certificate JSON was mutated.

## Verification and scope

The original round-1 probes now pass: missing direct owner inventory rejects;
a bound primary tuple below the entry minimum rejects through both public
checking APIs; an unbound direct owner reports explicit unestablished
applicability; the library g/g substitution rejects; a valid linked g/ginv
pair executes with the hand-derived arithmetic result and reverses exactly.
Thus `b-isa-spec-b6ds` and `b-isa-spec-0dq0` are fixed in the reviewed paths;
`b-isa-spec-jple` retains the sibling-route gap above.

All test/probe commands were headless, used private
`.tmp/rev-quality-isaref-r2/{tmp,target}` directories, and ran under
`ulimit -v 8388608; ulimit -c 0`.

```sh
timeout 600 cargo test --offline -j3 --manifest-path reference-interpreter/Cargo.toml \
  --test isa09_qinplace --test isa09_claim_replay --test isa09_certificate -- --test-threads=4
timeout 600 cargo test --offline -j3 --manifest-path reference-interpreter/Cargo.toml \
  --test r1_certificates --test r1_qinplace -- --test-threads=4
git diff --check b572e17..cdef3eb
```

Results: **58 passed, one existing ignore**, all commands exit 0. No full
reference suite, repository gate or sibling gate ran. Reading the main-side
and pre-fix claim/coverage code confirms the new inventory, library-pair
identity and executable-library regressions distinguish the old behavior;
expected arithmetic is independent of the implementation.

P2+ nits: **one bead `b-isa-spec-fg4u`** consolidates continuing dead dispatch,
tracked litter, stale `HANDOFF.md`, missing gate ratchet, production no-op
hooks, and deferred function/coverage/runtime work (also `b-isa-spec-nivd`).
No new nonblocking pattern found in the fix. Checked every COMMON anti-slop
rule; the silent obligation loss is the confirmed blocking violation. Batch-2
items remain assigned to batch 2 by the implementer brief.

Coverage: read closely the complete round-2 source/test diff, pair theorem
resolution, inventory/applicability traversal, both public checking APIs,
replay invocation decoding and certificate admission against §§4.5, 12.3
K4/K92; skimmed unchanged action/oracle call plumbing and prior structural
debt. Other interpreter subsystems and full conformance were not re-reviewed.

Provenance: implementer **gpt-6.1-sol**; reviewer **gpt-6.1-sol, fresh session, D9**.
This BLOCK review does not claim council clearance.

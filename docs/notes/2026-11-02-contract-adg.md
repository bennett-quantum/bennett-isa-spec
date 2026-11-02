# adg — loaded-program memory lifetime across launches

Proposal for council review; `b-isa-spec-adg`.

## Defect and North Star scope

ISA 0.9 §7.1 defines initial memory, allocation table and bump pointer
without saying whether initialization occurs at load or every run.
§3.2 places the register ledger at program scope; §10 gives a run
exclusive ownership only until CLOSED; §7.4 and T4 restore a run's entry
state on eligible reversal. These strongly favor program-lifetime state
but do not explicitly settle classical memory initialization.

Dialect 0.3 draft §§6.5–6.6 uses a persistent qreg table in device memory
and requires parameter/result roots to survive entry calls. North Star
§3.1's register handles and §5's REPL/library device model require
cross-entry quantum state. A fresh-every-run interpretation loses the
table while live registers remain. This is a contract ambiguity with
observable results, S0 under C1/C5; persistence is the compatible reading.
It adds no restriction to the North Star, including repeated SQLite or
kernel invocations.

## Alternatives and hand-worked witness

Layout: ptr_width=16, H=16, A_tab=2, S_cells=0, null_size=1,
heap_base=16, heap_limit_max=16; a one-byte global g at base 1 with image
00. Run A allocates one byte (returns 16) and stores 7 into g, then
completes and is CLOSED without reversal. Run B loads g and allocates
one byte. Under persistence it returns `(7,17)`; under reinitialization
it returns `(0,16)`. These are distinct visible results from the same
loaded program. Freeing 16 does not reclaim storage under §7.1.

If A is instead successfully REVERSED before CLOSED, its allocation
and store are undone, and B returns `(0,16)`. Merely closing A does not
reverse it. An eligible failed run retains its executed prefix until it
is reversed; closing a failed run does not silently roll it back.

| Alternative | Assessment |
|---|---|
| Program-lifetime memory and ledger (recommended) | Matches §10 ownership and the dialect's persistent table. |
| Reinitialize on each run | Would need a different host-held table and an explicit reset/lifetime ABI; it changes existing consumer assumptions. |
| Implicit reset when no register is live | Still changes classical allocation/global results; no clause authorizes it. |

## Recommended exact spec diff

Insert after §7.1's Initial state sentence:

```diff
+That state is established once when the program is loaded. Classical
+memory, allocation-table entries and the bump pointer belong to the
+loaded program and persist across runs. For sequential launches, a new
+run starts from the state left by the preceding CLOSED run. Closing a run does not restore its
+entry state. If the preceding run was successfully REVERSED before
+being CLOSED, its reversible state is its restored entry state (§7.4,
+§10, T4). A FAILED run otherwise leaves its executed prefix in place.
+Register-ledger entries and live register identities likewise persist
+under §3.3; ownership ends at CLOSED, not the register lifetime. Unloading
+and loading a program creates a new program instance with initial state;
+register ids of the old instance are not imported into it.
```

Append to §10's ownership paragraph:

```diff
+Persistence does not relax exclusive ownership of touched registers or
+memory while a run is not CLOSED. Separate loaded program instances have
+separate state; initialization is never repeated merely to start a run.
```

All examples here use sequential CLOSED runs. This wording does not add a
blanket prohibition on disjoint concurrent runs or invent a memory merge
rule. The existing ownership restrictions still govern overlapping uses.

## Hashes, catalog and consumers

This pins an existing reading: no source, envelope, target or catalog byte
changes and no hash re-pin. A consumer that reset memory on each run must
change its load/run lifecycle; plugin qreg-table code should not change.
Launch bindings and certificates that replay a later entry must include
its actual entry-state/preparation assumptions rather than presume a
fresh load. Domain claims do not acquire coverage of all histories merely
by showing one initialized run. Header diagnostics already include
E_INVALID_STATE; no new numeric status is proposed.

## Conformance vectors and verification

Test load→A→CLOSED→B and load→A→REVERSED→CLOSED→B with the results
above; check every byte, table live bit, bump pointer, register identity
and token lifetime. Add a failed A whose prefix stored 7: B sees 7 unless
that prefix was reversed. If B passes a register still owned by A before
A is CLOSED, it must return E_INVALID_STATE before effects. Fresh loading
restores g=0 and bump=16. A stored id can be recovered by qreg.from_id in
the next run after ownership is released. These are hand derivations;
the scalar state transitions are checked by
[contract-verification](2026-11-02-contract-verification.md), not by a
new reference execution.

## Release and after acceptance

Recommend a **0.9 K wording repair** to §§7.1/10 with lifecycle vectors.
Council should explicitly accept program scope under existing ownership, then
VM/reference owners validate their lifecycle and the dialect updates its
cross-reference. Keep adg open until the contract decision lands.

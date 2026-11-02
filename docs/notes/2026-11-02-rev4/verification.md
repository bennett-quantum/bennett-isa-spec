# Round-4 verification evidence

These are observed finite checks for the two proposals, not council
acceptance, production 0.10 implementation, or a full repository gate.

Executed successfully, exit 0:

```sh
bash /home/tobias/Projects/bennett-workspace/bennett-isa-spec/docs/notes/2026-11-02-rev4/run_checks.sh
```

The script uses an offline locked cargo build with -j4 and a 240-second
timeout, a 60-second Rust witness link, a 120-second Python witness timeout,
and a 30-second timeout for the retained round-2 finite checker. Build/link
address spaces are capped at 6 GiB and replay at 512 MiB. All subprocesses
inherit those bounds. TMPDIR, CARGO_TARGET_DIR and generated binaries/logs
are under .tmp/sol-rev4. GUI/browser environment variables are unset.
The initial isolated interpreter build completed in 34.06 seconds; the
build in run_checks.sh reused it and completed in 0.02 seconds.

| Check | Observed result |
|---|---|
| Reference interpreter + literal reversible machine | B=0/1/2 each covers 256 full x/target/fault tuples; fault counts 256/256/0. Actual commit outputs match XOR delivery for every tuple; all 12 x/budget machine executions reverse to their entire initial snapshot with clean frames. |
| CFR-3 external blobs | Supplied classifier and input-map bytes reproduce 64 undefined exclusions and zero layout-dependent exclusions. The independent B=1 recount remains 256 faults. Missing either blob, even with asserted zero exclusions, is unchecked. |
| CFR-3 rejection paths | Changed bytes in either blob, including JSON-preserving whitespace, reject before replay. Wrong count and each conflicting materialisation/program/build/entry/domain/invocation binding reject. Unsupported version and missing evidence are unchecked. |
| R2-1 launch/default path | N=3 gives 9/100000000 seconds; N=1 gives 3/100000000. Both default schedule fragments retain the complete input map and have distinct hashes. N=1 embedded versus N=3 request selects the correctly bound N=3 default; matching canonical inputs select embedded. |
| R2-1 incomplete/invalid/free input | Missing required input produces incomplete-model-objective before returning an estimate, even with an embedded schedule present. Wrong model hash/frozen distance and duplicate JSON keys reject; free N yields a symbolic outcome without a concrete estimate. Caller-buffer mutation does not change copied input. |
| Retained accepted content | Serialized surface import unchanged; d=3/d=5 arithmetic and exact units reproduce; 219 recipe bodies unchanged; ref@0.9 and ref@0.10 envelope hashes reproduce; both full notes retain the shared mixed-fat-object citation and valid local links. |
| Earlier round-2 finite checker | Exit 0, all eight PASS lines. Includes model arithmetic/hierarchy/shared reservations, operational freezing, subjects, selector/input outcomes and versioned catalog hashes. Other notes' finite checks are inherited regression evidence only. |

After tightening duplicate-key handling and canonical-map matching, reran:

```sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER \
  TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp/sol-rev4/tmp \
  prlimit --as=536870912 -- timeout 120 python3 -I -B \
  /home/tobias/Projects/bennett-workspace/bennett-isa-spec/docs/notes/2026-11-02-rev4/check_rev4.py
```

Exit 0; all four new PASS lines. This rerun includes the unchanged
interpreter/machine witness, both new contracts, identities and document checks.

Exact raw blob SHA-256 values:

```text
classifier b9378f8fa76f0401c39d4d3f04b24e98392ff5d013fe95b8a44bdc978c8e0672
input_map  5d5ff82ff001648ecd7e7f755701a17d3fab203171781d0dd3c495de5ad3c874
```

Exact **schematic schedule-fragment** SHA-256 values (not published
production schedules, targets or certificates):

```text
N=3 27a28f65b79494392b3a14431f53747c93b82b8d5a40b13d90b3cffa77bb8dd2
N=1 ee1d96e2e5d31538bd2dfef554144bf80f17b8c9ab723d16503053a25a490d88
```

The reference interpreter verifies the unchanged logical oracle and
machine reversal. The Python witnesses model the proposed external-bundle
and launch/default contracts for these finite fixtures. No native compiler,
source-UB detector, production certificate checker, 0.10 launch implementation,
vendor adapter, generic symbolic solver or mixed host shim is claimed.
Complete production multi-context and bound target/schedule/Report fixtures
remain publication work already described by the proposals. No full gates,
git commands, tracker mutations, spec/catalog/header edits or sibling writes
were performed. The runner owns commits; no commit SHA is available here.

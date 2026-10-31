# Worked controlled-X action (ISA 0.9, §17 item 7)

`controlled.bisa` calls `flip` under `ctl c`. On the complete domain of
(t,c) it maps (t,c) to (t XOR c,c), with exact +1 basis phase. There are
no allocations, releases, borrowed wires, local regions or classical
arguments. The budget is [0,0,0,0] because neither action invokes an oracle.

The complete published catalog is `spec/ref-0.9.json`, byte-identical to
`spec/ref-0.9-draft.json`: 219 recipes, four-line envelope SHA-256
`5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197`.
`controlled_action_fixture.py` uses the existing conformance JCS and §9.2
envelope tooling to derive every asset; `--check` compares exact bytes.
The Rust `controlled_action_fixture` test expands the catalog's `ctl`
and `action_call` recipes through the closure-bound flip registry, then
checks every exact matrix column in root (t,c) and callee (c,t) order.
The source registry entry is the one displayed `x t` instruction.

| File | Contents |
|---|---|
| controlled.bisa | Complete 0.9 object with embedded schedule and certificate |
| target.json | Two fast wires, exact native X with one control, duration 1 |
| leaves.json | One X leaf on t[0], effective control c[0], callee ordinal 0, full invocation path, target entry 0, time [0,1) |
| schedule.json | Explicit native selection and start 0 at the callee leaf; action step/phase selectors are both * |
| frame.json | Root mut:0 and in:0 width-one fast slots; empty scratch/history/borrow/ancillae |
| certificate.json | Root invocation, full four-element before/after relation, root and controlled-callee numerical claims with bound 0, both region frames, hashes and composition |
| compat-0.8.bisa | Same source operations under 0.8 identity/schema rules for the pinned interpreter |

The resource vector is time=1, peak_qubits=2, qubit_second=2. Both
interface wires remain live for the one-second gate. Root inherited
controls=0; the flip invocation has inherited_controls=1. The native
leaf and reference are the identical CNOT matrix: V†U-I=0, so the exact
Frobenius bound and diamond error are zero. The root table independently
lists all four complete input/output tuples. The callee frame has only
its mut slot; inherited c is caller-owned, outside its source slots.

The pinned binary is
`/home/tobias/Projects/bennett-workspace/bennett-isa-spec-review/reference-interpreter/target/debug/bennett-isa-ref`.
It accepts ISA 0.8 only (`§1: expected isa=0.8` for 0.9). The precheck runs
its `check` and `hash` on `compat-0.8.bisa`, then all four inputs in both
orientations. That file uses 0.8 frame/table encodings and default ref at
version 0.8, not the published 0.9 binding. CLI `check` validates identity
and metadata admission; it does not replay these certificate claims.
The exact conformance matrix test checks the fixture's mathematical
relation and zero error; neither test claims a pinned 0.9 executor or
interval certificate checker. The binary and interpreter trees are read-only.

Reproduction (from any directory):

```sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp python3 -I /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/tools/controlled_action_fixture.py --check
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/Cargo.toml --test controlled_action_fixture
```

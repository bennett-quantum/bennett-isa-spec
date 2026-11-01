# Accepted K78–K80 portable vectors (ISA 0.9, §15)

The three machine-readable manifests use `bennett.k78-k80.v1`. Each records
source text, expected source admission and complete basis input/output tuples.
Integers encode the listed `tuple_order` least-significant wire first. Every
output has exact phase `[1,0]`, lists all live interface/spectator/borrowed
wires, and has empty scratch. Rejected sources have no output tuples.

- `k78.json`: resolved CX/MCX identity, different controls, intervening reader,
  and polynomial-only cancellation. All 64 interface/borrowed inputs for the
  admitted case are evaluated, including arbitrary borrowed values.
- `k79.json`: all four zero-address operations, both inherited control values,
  all target/address-spectator/cell values, inverse restoration, and rejection
  of nonempty A=0 and empty A=1 addresses. `catalog_address: []` describes only
  the admitted zero-bit actual; rejected inputs have `null`.
- `k80.json`: the six-row primitive and semantic-call reverse/recompute pair,
  all 16 x/z/control/target tuples, and a one-iteration source-qloop replay
  whose release carrier lies outside the loop. Reverse retains entry:3/4/5
  with generated 0; primitive recompute uses entry:5 generated 0–7. Semantic
  recompute has fresh call_index 0/1/2/3 with callee entry:0 generated 0,
  independently of caller allocation/XOR/releases generated 0/1/2/3.
  The cross-loop original CX rows are entry:2/3 with iteration [0]; its
  replacement is at entry:6 with iteration [], never [0].

`portable_k78_vectors_exact_bytes`, `portable_k79_vectors_exact_bytes`, and
`portable_k80_vectors_exact_bytes` in the reference interpreter's existing
`frame_identity.rs`, `memory_address.rs`, and `cleanup_ownership.rs` regressions
rederive these bytes. They execute the source with `Object`/`QuantumState`,
expand the unchanged published catalog and run its cleanup leaves through
`QuantumState`, checking full amplitudes, phase, lifetimes and inverse output.
The K80 raw inverse reuses the forward stream and identities. The K80 reverse
trace selects original d leaves and the final release, with written inverse
source coordinates. Allocation names are the actual expander's stable names;
`allocation` is its debug tuple (or `None`), retained as diagnostic metadata.

Coordinates marked `finite-source-adapter` (points, iterations, semantic-call
indices and per-point generated ordinals) are adapter-assigned, including the
cross-loop original/replacement partition. Gate/wire/control/allocation data
marked `catalog-expander` comes from real expanded leaves. For calls,
`caller_point` is the invocation edge, `callee_point` the terminal row, and
`generated` the callee ordinal. There is no production 0.9 source builder
behind these addresses. The reference admits ISA 0.8 only; source inputs use
that compatibility envelope. K78/K79 admission is source parsing/validation
before hash checks (placeholder identity hashes); K80 sources are rehashed.
Scheduled recompute is constructor input, not rewritten-source admission.
This fixture neither claims full 0.9 admission nor independent source-builder
coordinate verification. Existing coherent regressions supplement the basis
tuples. Catalog and accepted 0.8 spec bytes remain unchanged.

Reproduce headlessly from the repository root (no network):

```sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR="$PWD/.tmp" cargo test --offline --manifest-path reference-interpreter/Cargo.toml --test frame_identity --test memory_address --test cleanup_ownership portable_
```

To deliberately regenerate, prefix the same command with
`BENNETT_UPDATE_ROUND8_VECTORS=1`. Normal tests compare exact bytes and never
write fixtures. Each manifest is canonical JSON plus one LF.

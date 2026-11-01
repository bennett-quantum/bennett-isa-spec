# Accepted K81 portable vector (ISA 0.9, §15)

| File | Witness | Byte-lock test |
|---|---|---|
| `k81.json` (`bennett.k81.v1`) | Joint swap inverse, first-release ownership, both release orders and duplicate-maximal selector tie | `portable_k81_joint_first_release_vectors_exact_bytes` in `reference-interpreter/tests/cleanup_partition.rs` |

Each source uses one-bit allocations at entry:0/1, a two-mutable-bit
`swap2` call at entry:2, its adjoint at entry:3, and releases at entry:4/5.
For a-then-b release, carrier 4 owns inverse occurrence 3 and release 4;
carrier 5 owns only release 5. For b-then-a release, the same points apply,
but b's release now owns the entire joint inverse. There is no independent
carrier at 3, and no split or duplicate ownership of that inverse.
Each carrier is non-emitting ordinal 0. Duplicate maximal cleanup entries
at 4 reject with the recorded diagnostic while 5 still selects reverse.

Coordinates and ownership are explicitly **finite-source-adapter** data,
not output of a production 0.9 source partition builder. There are no
catalog-expander leaves in this manifest (`null`); the catalog is unchanged.
The reference admits and hash-checks the sources using its 0.8 compatibility
envelope, then executes the full sources forward and adjoint, checking exact
state and allocator restoration. Each outer action has no interface wires,
empty scratch and phase +1. The callee is additionally executed on every
basis tuple (a is the least-significant bit), with exact phase, and on a
coherent entangled input with a spectator; its adjoint restores the input.
The real schedule selector supplies the tie diagnostic. This is a finite
witness and does not establish production 0.9 admission or pair formation.

From the repository root, reproduce without network:

```sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR="$PWD/.tmp" cargo test --offline --manifest-path reference-interpreter/Cargo.toml --test cleanup_partition portable_k81
```

To deliberately regenerate canonical JSON plus LF, prefix the command with
`BENNETT_UPDATE_K81_VECTORS=1`. Ordinary tests compare exact bytes without writes.

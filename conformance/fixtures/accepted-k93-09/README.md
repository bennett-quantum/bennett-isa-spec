# K93 MCX control bundles

These ISA 0.9 vectors derive expectations from §4.4 and §15. No execution
output supplies any expected value. `generate.py` uses ordinary integer
conjunction/XOR to enumerate the equations below. Its optional `--ref` uses
only the published `hash` command for identity metadata, never gate results.

| Vector | Independent derivation |
|---|---|
| `witness` | SWAP gives `(q,p,c)`; MCX gives `(q XOR [c=3],p,c)`. The 16 rows bind forward and adjoint: undo MCX, then SWAP. |
| `width-1/2/3` | AND of all control bits is `[c=2^width-1]`; target XORs that value. Both target entries are enumerated. |
| `mixed-sb/bs` | Controls are `[s,b[0],b[1]]` or `[b[0],b[1],s]`; both implement `t XOR (s AND [b=3])`. There are 3 explicit controls and 4 operands. Written source identities differ; K78 uses the same complete set. |
| `predicate` | Explicit controls remain `[s,b[0],b[1]]`, k=3. The disjoint active r raises effective control count to c=4. Toggle iff `r=s=1,b=3`. |
| `empty/empty-predicate` | Empty conjunction is true; X applies unconditionally or under r. Explicit k=0. |
| `wire1-target/indexed-target` | Target denotes one live bit: a bare wire[1] or t[1]. Only that bit XORs `[c=3]`. |
| suite F negatives | Repeated scalar/bundle controls, target/control overlap, multiwire target, missing target and out-of-range index violate §3.3/§4.4 before any relation is checked. |

Every basis case asserts an exact state dictionary with its sole amplitude
`[1,0]`, detecting relative or global phase and extra components. Each row is
also run with `--adjoint`, beginning from the formula-derived forward output.
The two coherent CLI cases retain the supplied binary64 amplitude on both
branches. The reference integration test additionally prepares the exact
entangled external-reference state using H(R), CX(R,c0), CX(R,c1), asserts
the entire resulting state and then exact inverse state equality.
`k93.json.reference_witness` publishes the §15 tuples in `(p,q,c,R)` order:
`[(0,0,0,0),(0,0,3,1)] -> [(0,0,0,0),(1,0,3,1)]`, amplitude `1/sqrt(2)`.

`witness.bisa` preserves the original published bytes: object
`db3d3f92a79077074901449d83532f1a0d82ef1621bd833adb7ad14d1aedd467`,
action closure
`e38d5da64f9a0ae89a66243bcbe5a321cb807a5bbabf83952b63d3465dfab346`.
An unchanged source hash does not validate a cached scalar/first-bit
execution, certificate or schedule. Rebuild source resolution, regenerate
adapter leaves and their hashes/schedules, and recheck certificates. The
published `ref@0.9` catalog remains
`5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197`.
The interpreter's borrow-call and constructor caches live only within one
fresh validation/expansion; no persistent resolution cache format changes.

Reproduce with headless environment and project-local TMPDIR:

```
python3 -I conformance/fixtures/accepted-k93-09/generate.py --ref /path/to/bennett-isa-ref --check
cargo test --offline --manifest-path reference-interpreter/Cargo.toml --test isa09_mcx_bundles
BENNETT_CONFORMANCE_BIN=/path/to/bennett-isa-ref cargo test --offline --manifest-path conformance/Cargo.toml --test k93_vectors
```

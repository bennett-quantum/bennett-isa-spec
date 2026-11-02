# K82 named-interface vector (ISA 0.9, §11.1, §15)

Status: **accepted** (council round 10, 2026-11-01; `bennett/docs/council/isa-09-k82-2026-11-01/`).
The proposal and diff are in `docs/notes/2026-11-01-ref-named-interface-k82.md`.

| File | Contents |
|---|---|
| `k82.json` (`bennett.k82.v1`) | The 54 named interfaces, the 34 uninvoked such recipes, 20 invoke edges with 86 leaf digests, 16 binding cases |
| `generate.py` | Derives `k82.json` from `spec/ref-0.9.json`, with its own expander and no interpreter imports; asserts the published envelope hash |
| `probe/` | Runs the vector against the reference's `ref_language` expander; not part of any gate |

Edge digests are SHA-256 of the compact JSON leaf list `[[gate,[wires]],...]`. Allocation identities are `#k` in
first-seen order, and alloc/release leaves carry `["#k"]`. Each case gives a recipe list, root, arguments and root
wire bundles, then either the expected leaves or `"reject"`.

Reproduce from the repository root:

```sh
python3 -I conformance/fixtures/accepted-k82-09/generate.py --check
CARGO_TARGET_DIR=.tmp/opus-isa-helpers/target cargo run -q --release --offline \
  --manifest-path conformance/fixtures/accepted-k82-09/probe/Cargo.toml -- \
  conformance/fixtures/accepted-k82-09/k82.json spec/ref-0.9.json
```

The reference now passes all 86 edge digests, all 54 interface sets and all 16 binding cases,
including unrelated caller names and live declaration arguments. The gate includes
`conformance/tests/ref_named_interface.rs`; the complete 2026-11-02 gate passed 127 conformance
tests and 564 reference tests (five preexisting ignores). The catalog bytes and envelope are unchanged.

These are finite witnesses: edge tuples use widths 1 to 4, and functional correctness of the circuits stays with the
existing catalog evidence. The vector establishes binding and leaf identity only.

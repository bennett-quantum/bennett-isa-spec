# Pending K82 named-interface vector (ISA 0.9, §11.1, §15)

Status: **proposed, not accepted**. Rename to `accepted-k82-09` when the council accepts K82; the spec diff cites that name.
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
python3 -I conformance/fixtures/pending-k82-09/generate.py --check
CARGO_TARGET_DIR=.tmp/opus-isa-helpers/target cargo run -q --release --offline \
  --manifest-path conformance/fixtures/pending-k82-09/probe/Cargo.toml -- \
  conformance/fixtures/pending-k82-09/k82.json spec/ref-0.9.json
```

Reference result on 2026-11-01: 86/86 edge digests match and 14/16 cases pass. The two failures are expected until
`b-isa-spec-b5b` lands: `callee-local-is-not-captured-by-an-unrelated-caller-name` and
`live-declaration-binds-by-its-wire-actual-rule`.

These are finite witnesses: edge tuples use widths 1 to 4, and functional correctness of the circuits stays with the
existing catalog evidence. The vector establishes binding and leaf identity only.

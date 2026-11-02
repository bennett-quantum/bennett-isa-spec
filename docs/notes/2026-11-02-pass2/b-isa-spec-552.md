# 552 — published 0.9 schema adoption, not another frame contract

Disposition for `b-isa-spec-552`.

## Defect and North Star scope

The bead's old reference rejection is real historical producer evidence,
but **not a current missing ISA contract**. ISA 0.9 §9.4 B5 specifies
slots/scratch/history/borrow/ancillae; §12.1/§12.3 K13 specifies positional
before/after tuples and invocation bindings. These are published 0.9
rules, not a schema to enable on an unchanged 0.8 object by a draft flag.
§9.1 requires version/schema validation and §9.2 prefixes semantic
envelopes with the full ISA version.

The current source has separate TABLE09 productions in
`reference-interpreter/src/metadata_schema.rs` for B5, K13 table tuples
and invocation bindings. This job built its own reference CLI and tested
the published controlled-action 0.9 fixture: **both check and hash pass**.
The old `missing field interface` boundary is not reproduced for this
published 0.9 packet. The fixture uses the newer schema directly.

North Star §§4.4/5.1 and §6.1 requires versioned substitutable interfaces
and honest checker evidence. Deliberate migration preserves that scope;
schema acceptance alone does not prove a scheduled numerical claim or
the full migration of the circuits producer.

## Alternatives and worked witness

Origin read: circuits `docs/design/2026-11-01-m5-contract-requests.md`,
“Reference draft-frame replay”, and `tests/proof_output.rs`'s check/hash
rejection assertions. Those tests allow either version mismatch or the
old frame error; they do not establish current published-0.9 rejection.

Concrete current witness:
`conformance/fixtures/controlled-action-09/controlled.bisa` contains B5
frames and a K13 four-row table for controlled X. The full tuples
`(t,c)=(0,0),(0,1),(1,0),(1,1)` map to `(0,0),(1,1),(1,0),(0,1)`.
Its object hash is
`45bbb506541e3a8bb28a18e1bb0aef16ba01ee1e7cac74d5a190deb3ad36ef5a`.
Both CLI commands return that exact object hash and status ok.

| Alternative | Assessment |
|---|---|
| Add another ISA frame schema or blanket accept-draft flag | Unnecessary and blurs the version boundary. |
| Deliberately adopt published 0.9 and regenerate bindings | Recommended producer/implementation route. |
| Keep ISA 0.8 while placing 0.9 claims in its metadata | Not a migration; incompatible contracts remain incompatible. |

## Recommended exact spec diff

```diff
# No change to Bennett-ISA-Spec-0.9.md.
# B5 and K13 already publish the requested schema.
```

The producer owner should adopt the intended published ISA/object/ABI
versions, regenerate canonical signatures and every dependent interface/
closure/object/schedule/certificate binding, then test its own packets.
Changing only the header or certificate versions is insufficient.
Metadata-only edits are excluded from object_hash as §9.2 states, but
changing the source/header/signature version changes source envelopes.
Do not backport new semantics into a 0.8 envelope by relabelling it.

Report four separate stages: version/schema admission; correct identity
bindings; claim replay; complete applicable certificate verification.
The available CLI exposes check/hash and source execution, not a new
“all certificate claims proved” command. Missing target/schedule/interval
verification must remain disclosed. The independent [0jq](b-isa-spec-0jq.md)
applicability ambiguity is not resolved merely by schema acceptance.

## Hashes, catalog and consumers

No catalog rewrite or new K is needed for this bead. An actual 0.8→0.9
producer migration changes its versioned envelopes and consumer pins;
it binds the already published ref@0.9 catalog hash. It must retain 0.8
support where promised. The current source-derived binary is a local
reproduction, not an identified released binary or a guaranteed install
path for other machines. The runner commits this note; no commit id or
reference release tag was invented.

## Conformance vectors and verification

From the repository root, with headless TMPDIR and CARGO_TARGET_DIR as
in [INDEX](INDEX.md):

```sh
timeout 180 cargo build --offline -j4 --manifest-path reference-interpreter/Cargo.toml
.tmp/sol-contract-pass2/target/debug/bennett-isa-ref check conformance/fixtures/controlled-action-09/controlled.bisa
.tmp/sol-contract-pass2/target/debug/bennett-isa-ref hash conformance/fixtures/controlled-action-09/controlled.bisa
```

All three exited 0. The INDEX verifier additionally checks the four
source-action basis outputs independently against the literal table.
Negative migration vectors remain owed by the producer: 0.8 mixed-schema
packet, bad frame identity/width, short tuple, wrong invocation and stale
closure hash; distinguish admission rejection from a false replayed row.
No full circuits gate or complete certificate proof is claimed here.

## Release and route

**Already covered by published 0.9 B5/K13; implementation/migration
follow-up, no new K and no need to wait for 0.10.** Recommend the bead
owner reassess against this reproduction, then close only after the
actual producer migration and acceptance work it tracks is completed.

# K84 checker diagnostic witnesses

`baseline.bisa` is a standalone ISA 0.9 NOT action with a complete two-row
exhaustive table certificate. There is no schedule. Each claim has its own
invocation and complete before/after tuples. `missing-expected.bisa` deletes
only the claim's `expected` field (§12.1 K13). `duplicate-component.bisa` repeats
only the `t` domain bundle (§12.2 K72); its invocation and table are unchanged.
The repeated bundle resolves to the same interface slot. This witness says
nothing about repeated borrowed labels, which K70 permits.

Suite D checks admission/rejection and the normative clause, without assigning
normative meaning to a local diagnostic spelling. The reference integration
test also checks complete mutation equality, byte hashes, both NOT inputs and
reversal. These admission vectors do not claim semantic certificate replay;
the remaining 0.9 replay migration is tracked by `b-isa-spec-fuk.7`.

For checker `bennett-isa-ref` version `0.0.0`, current local diagnostics are
`missing field expected` within the `§12.1 K13 (exhaustive)` schema path and
`§12.2: repeated domain component`. The schema path identifies
`certificate.regions[closure].claims`; the latter message identifies the
domain obligation without a more detailed location. These are implementation
messages scoped to that checker/version, not registered ISA diagnostic codes,
executor statuses or fault codes. A different checker may use different text.
K55's mandated `multiple maximal matching schedule entries` diagnostic remains
separately tested by the K83 schedule vectors.

Regenerate with `python3 -I conformance/fixtures/accepted-k84-09/generate.py
--ref /path/to/bennett-isa-ref`, or add `--check` for byte comparison. Set
`TMPDIR` to an existing job-local directory. The generator uses the published
CLI only for source identity and admission; the two-row NOT relation is derived
independently as `after = before XOR 1`.

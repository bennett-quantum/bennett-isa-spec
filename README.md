# bennett-isa-spec

The Bennett virtual ISA: spec text, reference interpreter, formal model, conformance suites A-H (as a crate), and C headers for the launch and scheduling ABIs.

Depends on: nothing.

Layout: `spec/` (normative text), `reference-interpreter/` (crate `bennett-isa-ref`), `conformance/` (crate `bennett-isa-conformance`), `include/` (C headers), `SPEC_VERSIONS.toml`.

Published specifications:

- [ISA 0.8](spec/Bennett-ISA-Spec.md): accepted round 8; current for dialect 0.2.x and the reference interpreter.
- [ISA 0.9](spec/Bennett-ISA-Spec-0.9.md): accepted 2026-10-31 by all three council seats; [ref-0.9.json](spec/ref-0.9.json) and the [controlled-action-09 fixture](conformance/fixtures/controlled-action-09/).

Reference interpreter status: it implements ISA 0.8 and is migrating to 0.9 (partial; remaining slices are beads under `b-isa-spec-fuk`). Current state and gates: [HANDOFF.md](HANDOFF.md). The 0.9 spec text is accepted; the interpreter does not yet execute all of it.

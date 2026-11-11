# bennett-isa-spec

## What this repo is

The virtual ISA contract: spec text, reference interpreter, formal model, conformance suites A-H as a crate, and C headers for the launch and scheduling ABIs. Depends on nothing; every other repo targets it.

## Where the spec is

Normative text: `spec/Bennett-ISA-Spec.md`, ISA 0.8, council-accepted 2026-10-28 in round 8, current for dialect 0.2.x and the reference interpreter; `spec/Bennett-ISA-Spec-0.9.md`, ISA 0.9, accepted by all three council seats on 2026-10-31, for new consumers (bennett-vm M5 onward). Both remain published (see `spec/README.md` for the change rules). Versions in `SPEC_VERSIONS.toml`. `reference-interpreter/isa_check.py` is the Python seed of the reference interpreter and the spec's pre-flight check.

## Rules

The one rulebook is bennett/docs/process/COMMON.md; it binds here. Gate: `scripts/pre-push`. Tasks: `bd` (`bd prime` lists its commands).

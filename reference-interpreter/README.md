# Bennett ISA reference interpreter

Rust 2021, standard library only, ISA 0.8. The implementation is in progress;
[PROGRESS.md](PROGRESS.md) records verified work and remaining obligations.
It is not yet a complete conformance reference. In particular
certificate composition, borrow proofs and comprehensive static validation
are unfinished. The numerical simulator uses `f64`; it does
not implement the rigorous certificate checker of §12.

Build and test offline from the repository root:

```sh
cargo build --manifest-path reference-interpreter/Cargo.toml --offline
cargo test --manifest-path reference-interpreter/Cargo.toml --offline
cargo test --manifest-path reference-interpreter/Cargo.toml --offline -- --ignored
```

The executable is `reference-interpreter/target/debug/bennett-isa-ref`.
All successful commands write one JSON object to stdout. Diagnostics go to
stderr with exit status 1; success returns 0. ISA diagnostics cite their rule.
Integers above the exact JSON integer range are decimal strings (§9.1).

## Commands

`check <object>` parses and validates the object, computes identities, and
verifies every stored interface hash. Output contains `status`, `object_hash`
and `symbols`, indexed by symbol name with `interface_hash` and `closure_hash`.
An unresolved dependency has a null closure hash.

`hash <object>` produces the same identities without comparing stored hashes.
This helps construct fixtures; it does not edit files or certify their hashes.

`run-oracle <object> <oracle> --input <x[,y...]> [options]` executes the §14.1
sequential interpreter. Output is `{"code":0,"payload":9,"steps":3}` for
inc8(8) at B=3. `--input ""` supplies an empty input list. Values may be
unsigned decimal or `0x` hexadecimal. The oracle name may start with `@`.

Options:

- `--budget B` changes the step limit; `--budget B,heap,stack,depth` sets all
  four limits. Default B is 1,000,000. Layout limits supply heap and stack
  defaults; depth defaults to 64 with a layout. Memoryless limits are zero.
- `--mem name=hex` supplies an interface region's bytes, in cell order.
  Unspecified regions start at zero. Each supplied region must have the exact
  declared byte count. Repeat for multiple regions.
- `--arg value` supplies the next classical argument. Unsigned integers are
  decimal/hex; `f64:0.5` supplies an angle; `token:B,heap,stack,depth` supplies
  a nested-oracle budget. Repeat in signature order. Specify `--budget` before
  abbreviated tokens to use its limits as their defaults.

`step-check <object> <oracle> --exhaustive [options]` compares the independent
sequential and reversible interpreters, then reverses all B steps and compares
every internal register, memory cell, stack cell, allocator entry and frame
against the entry tuple. It enumerates input bundles and all unspecified
interface-memory bytes. Target/fault entry values do not affect computation:
delivery XORs a fixed payload/code into them. Output reports `cases`,
`input_and_memory_bits`, `fixed_regions`, `status` and that delivery property.
Fixed regions restrict the tested domain and are explicitly listed.

The same `--budget`, `--mem` and `--arg` options apply. Default B is 256.
`--max-bits N` changes the exhaustive enumeration cap from 16 to at most 24.
Larger domains fail explicitly; the command never silently samples. Classical
arguments are fixed parameters of the check.

Examples from the repository root:

```sh
reference-interpreter/target/debug/bennett-isa-ref check reference-interpreter/tests/fixtures/inc8.bisa
reference-interpreter/target/debug/bennett-isa-ref run-oracle reference-interpreter/tests/fixtures/inc8.bisa inc8 --input 8 --budget 3
reference-interpreter/target/debug/bennett-isa-ref step-check reference-interpreter/tests/fixtures/inc8.bisa inc8 --exhaustive --budget 4
reference-interpreter/target/debug/bennett-isa-ref run-oracle reference-interpreter/tests/fixtures/sum_db.bisa sum_db --input 3 --mem db=0102030405060708090a0b0c0d0e0f10 --budget 1000,4080,512,64
```

## Library and fixtures

`Object::parse`, `identity`, `check_hashes`, `run_oracle` and `OracleMachine`
provide the corresponding library operations. `ClassicalMachine` exposes
dynamic SSA execution and retained-history reversal. A hostcall suspends `run`
with a `HostRequest`; `reply` supplies bytes, and another `run` resumes it.
The request/reply transcript is retained for reversal. `QuantumState` supports
basis permutations, small state vectors, gates, clean release and enumerated
Born measurement branches. `execute_action` executes structured Q bodies and
their adjoints. State vectors are limited to 24 live wires; basis permutations
can use larger wire stores. Numerical cleanliness uses tolerance 1e-12.

`Object::link` resolves imported declarations and pinned closures, rejects
conflicting definitions/layout/device/data entries, deduplicates definitions,
and regenerates canonical linked identities. `QuantumRegisters` implements
live/viewed/consumed handles and clean preparation. Pass it to
`ClassicalMachine::with_quantum_registers` for qreg entry arguments. Measurement
uses a reproducible xoshiro256** stream; `push_measurement_draw` supplies a
specific uniform draw for differential tests. Measured traces cannot reverse.

Canonical `.bisa` fixtures have adjacent `.comments` files with hand-derived
expected outcomes; comments are not an extension to the §16 grammar.
[SPEC-QUESTIONS.md](SPEC-QUESTIONS.md) records specification readings.

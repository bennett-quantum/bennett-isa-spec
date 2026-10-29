# Bennett ISA reference interpreter

Rust 2021, standard library only, ISA 0.8. [PROGRESS.md](PROGRESS.md) records
what is verified, what is staged and in what order; [SPEC-QUESTIONS.md](SPEC-QUESTIONS.md)
records every reading the specification leaves open;
[council/response-round-1.md](council/response-round-1.md) answers the council's
round-1 review blocker by blocker. The crate is not yet a complete conformance
reference: the launch ABI (§10), target descriptions (§8), the schedule
consumer (§11) and the rigorous interval certificate checker (§12.1) are staged
as deliverables 8–11, and claims that need them are reported as unverified,
never as verified.

Build and test offline from the repository root (headless: `DISPLAY`,
`WAYLAND_DISPLAY`, `DBUS_SESSION_BUS_ADDRESS`, `BROWSER` unset;
`TMPDIR=$PWD/reference-interpreter/.tmp`, created first):

```sh
cargo build --manifest-path reference-interpreter/Cargo.toml --offline
cargo test --manifest-path reference-interpreter/Cargo.toml --offline
cargo test --manifest-path reference-interpreter/Cargo.toml --offline -- --ignored
```

The executable is `reference-interpreter/target/debug/bennett-isa-ref`. Data
commands write one JSON object to stdout and exit 0; `help`/`--help` prints
plain usage. Diagnostics go to stderr with exit status 1, and ISA diagnostics
cite their rule. Integers above 2^53 − 1 are decimal strings (§9.1).

## Commands

`check <object>` parses and validates the object, computes identities, and
verifies every stored interface hash. Output: `object_hash`, `status`
(`"ok"`, or `"unlinked"` when imports are unresolved), `symbols` (each with
`interface_hash` and `closure_hash`, null when its closure has an unpinned
import) and `unresolved_imports`.

`hash <object>` produces the same identities without comparing stored hashes
(for building fixtures; it edits nothing).

`run-oracle <object> <oracle> --input <x[,y...]> [options]` executes the §14.1
interpreter and prints `{"code":0,"payload":9,"steps":3}` for inc8(8) at B=3.
Inputs of any width are decimal or `0x` hexadecimal; `--input ""` is the empty
list. The oracle name may start with `@`.

Options:

- `--budget B` or `--budget B,heap,stack,depth`. Default B is 1,000,000.
  Layout limits supply heap and stack defaults; depth defaults to 64 with a
  layout; memoryless limits are zero.
- `--mem name=hex` supplies an interface region's bytes in cell order;
  unspecified regions start at zero; the byte count must be exact.
- `--arg value` supplies the next classical argument: an integer,
  `f64:0.5`, or `token:B,heap,stack,depth` (after `--budget` the abbreviated
  token takes its limits as defaults).

`step-check <object> <oracle> MODE [options]` runs the independent §14.1
interpreter and the literal §3.6 machine on each domain element, compares their
F_B, then runs the B inverse steps and compares every location, cell, counter,
stack cell, allocator entry and frame with the entry tuple (T2, T6). The domain
is every input bundle and every byte of every interface region not fixed by
`--mem`; target and fault entry values do not affect the computation (XOR
delivery). MODE is one of:

- `--exhaustive [--max-bits N]`: the whole domain, refusing domains above the
  user's guard N (default 16; no fixed ceiling);
- `--suite`: the §17 item 13 default, exhaustive at ≤ 16 bits and otherwise
  2^16 xoshiro256** samples;
- `--samples N`: N samples (`--seed a,b,c,d` sets the four state words).

Output: `cases`, `input_and_memory_bits`, `mode` (`exhaustive` or `sampled`),
`fixed_regions`, `status`. Default B is 256; `--budget`, `--mem` and `--arg`
apply.

```sh
reference-interpreter/target/debug/bennett-isa-ref check reference-interpreter/tests/fixtures/inc8.bisa
reference-interpreter/target/debug/bennett-isa-ref run-oracle reference-interpreter/tests/fixtures/inc8.bisa inc8 --input 8 --budget 3
reference-interpreter/target/debug/bennett-isa-ref step-check reference-interpreter/tests/fixtures/inc8.bisa inc8 --suite --budget 4
reference-interpreter/target/debug/bennett-isa-ref run-oracle reference-interpreter/tests/fixtures/sum_db.bisa sum_db --input 3 --mem db=0102030405060708090a0b0c0d0e0f10 --budget 1000,4080,512,64
```

## Library

- `Object::parse` (grammar, schemas, §6 discipline, §5.6 borrow frames, §5.5
  matched cleanup, §5.1 call graph), `identity`, `check_hashes`, `Object::link`
  (resolution, pins, deduplication, certificate composition).
- `Object::run_oracle_vectors` is the §14.1 reference interpreter on
  `BitVector` values of any width; `run_oracle` is its checked u128 adapter.
- `OracleVectorMachine` is the literal §3.6/§3.7 machine with counters, cells
  and slots at the derived widths (`DerivedWidths`); `OracleMachine` is its
  checked u128 adapter; both expose `forward_step`, `inverse_step`, `commit`,
  `outcome` and complete snapshots.
- `ClassicalMachine` executes CC/R with dynamic SSA, retained history, the four
  §7.3 device models, tapes, hostcall request/reply, segment `uncall` and
  whole-run reversal; `QuantumRegisters` holds §3.3 qreg handles.
- `QuantumState` is the §14.2 simulator: a basis fast path and a sparse
  superposition over any number of wires, each with an exact amplitude shadow
  in Q(√2, i)[z, 1/z] (`src/exact.rs`). Cleanliness, basis reads and state
  equality are exact; binary64 amplitudes serve Born probabilities only.
  `Object::execute_action` runs structured Q bodies and their adjoints.
- `Object::verify_certificate` and `verify_claim` re-execute exhaustive and
  sampled claims (`ClaimVerdict::Proof` only for exhaustive claims over D),
  check translation-lemma and library structural claims, and report interval
  claims and `expected: "oracle"` as `Unverified` with the reason.
- `Xoshiro256` is the §12.1 sampler.

Host limits are reported as such and are not ISA rules: a width or count above
2^64 − 1, a dense amplitude vector (`amplitudes`) or a region larger than host
memory, and B frames beyond the host address space fail with a "host limit"
diagnostic.

## Fixtures

Canonical `.bisa` fixtures in `tests/fixtures/` have adjacent `.comments` files
with hand-derived expected outcomes (comments are not a §16 extension). Every
fixture's stored hashes are audited (`tests/r1_fixtures.rs`), and the inc8 and
sum_db fixtures are compared byte for byte with §16.3.

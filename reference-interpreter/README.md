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
- `--samples N`: N samples, N ≥ 1 (`--seed a,b,c,d` sets the four state words).

Output: `cases`, `input_and_memory_bits`, `mode` (`exhaustive` or `sampled`),
`fixed_regions`, `status`. Default B is 256; `--budget`, `--mem` and `--arg`
apply.

The four commands below share these conventions. A classical value is
`{"type","value"}`: `iN` and `ptr` values are unsigned decimal strings of
their bit-vector values (a signed reading at width N is the caller's), `f64`
is a JSON number, `token` is an object with `steps`, `heap`, `stack` and
`depth` (§9.1 integers), and `qreg` is the register id. Fault records are
`faults`, a list of `{"code","name"}` with the §5.4 code and name (`name` is
null outside the §5.4 table), plus `fault`, that code or null. Tape entries
are `{"args","bytes","dev","op"}`: the §7.3 tape triple plus the bytes the
operation transferred, hex. Measurements are `{"bits","reg","value"}` with
`bits` in wire order (wire 0 first). Draws select the first outcome, in
lexicographic order of the bits string (wire 0 first), whose cumulative Born
weight exceeds draw × total weight; integer values remain little-endian
(SPEC-QUESTIONS 29). Console output is `console_hex` always
and `console` when it is valid UTF-8, else null. `memory_digest` is the
SHA-256 of the complete §7.1 memory (allocation table, heap bump, every
nonzero byte).

For CC/R entries, arguments bind positionally to the entry signature (§10, `E_ARG` on a type
or count mismatch): `--args a,b`, `--input a,b`, or repeated `--arg value`,
at most one form (omission means no arguments). An `iN`/`ptr` value is decimal (negatives accepted within
the signed range) or `0x` hex, range-checked at the width; `f64:` prefixes a
binary64; `token:B[,heap,stack,depth]` is a §5.4-validated budget token
(the four-component `--budget` supplies its defaults, as for `run-oracle`);
`qreg:<bits>` allocates a fresh register and prepares those basis bits
(first character is wire 0). `--entropy hex` is the entropy tape. `--draws
d,...` queues measurement draws in [0, 1). `--replies file` is a JSON array
of `{"service","payload","reply"}` objects (payload/reply hex) consumed in
HOST_REQUEST order: each request must match the next entry's service and
payload, every request must find a reply, and every reply must be consumed,
else the run is a §10 diagnostic and no JSON is printed. `--mem global=hex`
replaces a layout global's image in the entry state, at exactly its size.
For CC/R entries `--budget B[,heap,stack,depth]`'s first component is
execution fuel (CC/R has no ISA step budget; exhaustion is a §3.4
diagnostic, default 1,000,000) and the rest are token defaults.

`run-cc <object> <entry> [options]` runs a CC or R entry on the
`ClassicalMachine` and prints the complete run. Output: `entry`, `status`
(`COMPLETED` or `FAILED`), `steps`, `budgets` (one per `token:` argument, as
for `run-program`), `results`, `fault`, `faults`, `tape`,
`console`, `console_hex`, `clock`, `effect_tape_length`, `memory_digest`,
`host_requests` (`{"address","capacity","payload","reply","service"}`),
`measurements`, and `trace`. Each trace record is one executed instruction:
`step` (counting from 0), `function`, `block`, `index` within the block,
`line` (the object text line), `op`, `lhs`, `operands`, `registers` (each
`{"activation","name","pre","post"}`, the changed dynamic instances with
their previous and new current values, null when absent) and `memory` (each
`{"address","pre","post"}`, every changed byte). A hostcall record includes
its reply's deltas. A fault or trap is a `FAILED` result with no values,
not a diagnostic. This includes runtime region-interface and control/target
aliases created through dynamic SSA: the call records fault 4 before changing
quantum state or consuming a handle. Controls may share read-only inputs.

`reverse <object> <symbol> [options]` runs forward, then reverses, and
reports whether the complete entry state is restored byte for byte:
`equals_entry` and `first_difference` (null when equal, else a description of the first
differing component). `status` is `REVERSED` when equal and `FAILED`
otherwise; a reversal the ISA forbids is a diagnostic citing its rule. The
form depends on the symbol kind. For a CC/R entry, `reverse` takes the
`run-cc` options plus `--steps N` (reverse only after N forward
instructions; without it the whole run); it is §7.4 whole-run or prefix
reversal of the retained history. It also reports `memory_equals_entry`
(the §7.1 memory component). Output: `entry`, the forward `results`, `fault`,
`faults`, `budgets`, `steps`, `tape` and `host_requests`, then the post-reversal
`console`, `console_hex`, `clock`, `effect_tape_length`, `memory_digest`
and `entry_memory_digest`. A faulted prefix reverses like any prefix
(§7.4); a run containing `measure` is a §10 `E_NOT_REVERSIBLE` diagnostic.
For an oracle, `reverse` takes the `run-oracle` options (`--input`,
`--budget`, `--mem`, `--arg`) plus `--steps k` (default B, above B is a
§5.4 diagnostic): it runs k forward steps of the literal §3.6 machine, then
k inverse steps (§14.1 prefix reversal, T2). Output: `oracle`, `budget`,
`payload` and `code` of F_k as read off the machine, `steps` (its executed
rows), `forward_steps` and `reverse_steps` (both k), `frames_clean`, and
`state_after_reversal` with `pc`, `pc_prime`, `faulted`, `counters`
(heap_used, atop, sp, fp, depth) and `locations` (`function.name` to its
bit-vector value). For an action, `reverse` takes the `run-action` options,
executes the action and then its adjoint (§10 reversal of a top-level Q
call), and prints `forward_state` and the post-reversal `state`, `mut` and
`in`.

`run-action <object> <action> [options]` executes a Q action once (its
adjoint with `--adjoint`) on a basis state or a small state vector.
Exactly one input form: `--mut v,... [--in v,...] [--mem region=hex]` (one
integer per bundle, decimal or `0x`; `in` bundles and region cells default
to zero; region hex is one little-endian value per cell in cell order, of
exactly the region's byte count), `--basis <bits>`, or `--state
<bits>=<re>[,<im>][;...]`. Classical arguments take `--args` or `--arg` in
the `run-oracle` spellings. Bit strings and the output state use the
interface order: mut bundles, then in bundles, then mem regions with cells
in order, wire 0 of each bundle or cell first (bundles little-endian,
§14.2); `--basis`/`--state` bits cover that whole order. Output: `action`,
`adjoint`, `wires` (the interface wire count), `state`, `mut`, `in` and
`mem`. `state` maps each exactly nonzero component's bit string to
`[re, im]`, the binary64 values of the exact amplitudes (a component
appears iff its exact amplitude is nonzero; `--state` amplitudes are held
exactly as the dyadic values they denote). `mut`, `in` and each `mem` cell
give the exact integer when that bundle or cell is basis-definite on every
component, else null. Borrowed wires start in |0⟩ from the borrow pool and
must be returned to |0⟩ on every component, else the run is a §5.6
diagnostic.

`run-program <object> <cc-entry> [options]` is a one-shot §10 launch:
`bennett_load` (parse and stored-hash verification), positional
`bennett_run` of a device CC entry with the `run-cc` argument, entropy,
reply and draw options, result reporting, and `bennett_run_close`;
`--reverse` applies `bennett_reverse` before closing. The entry must have
kind `cc` (§10; other kinds are `E_ARG`). Output: `entry`, `status`
(`COMPLETED`, `FAILED`, or `REVERSED` with `--reverse`), `run_state`
(`CLOSED`), `reversed`, `steps`, `results` (a FAILED run exposes no values),
`fault`, `faults`, `measurements`, `tape`, `host_requests`, `budgets` (one
`{"steps","heap","stack","depth"}` per `token:` argument, §5.4-validated at
creation), `console`, `console_hex`, `clock`, `effect_tape_length`,
`memory_digest`, `entry_memory_digest`, and the final state's
`equals_entry`, `memory_equals_entry` and `first_difference` against the
entry state. `--reverse` on a FAILED run is `E_INVALID_STATE` and on a run
that measured is `E_NOT_REVERSIBLE` (§10), both diagnostics. Two inputs are
explicit staged diagnostics, never silently ignored: `--target file`
(`bennett_load`'s target, staged deliverable 9: §8) and an object with a
`schedule` section (verbatim scheduled execution, staged deliverable 10:
§11). The rest of deliverable 8 (the full §10 C API as a library, estimates,
coverage) remains staged as PROGRESS records.

Exact output schemas follow. This structural notation describes JSON:
`T[]` is an array, `Record<string,T>` an object with arbitrary string keys,
`&` combines object fields, and `|` is a union. Every listed field is
required; no other fields are emitted. `U` is a nonnegative integer,
a JSON number through 2^53 − 1 and a decimal string above it (§9.1);
`Decimal` is always an unsigned decimal string; `Hex` is an even-length
hex string; `Digest` is a 64-character lowercase SHA-256 hex string.
`number` denotes a finite binary64 JSON number.

```text
Budget = {steps: U, heap: U, stack: U, depth: U}
Value = {type: "iN" | "ptr", value: Decimal}
      | {type: "f64", value: number}
      | {type: "token", value: Budget}
      | {type: "qreg", value: U}
Fault = {code: U, name: string | null}
Tape = {args: U[], bytes: Hex, dev: string, op: string}
Measurement = {bits: string, reg: U, value: U}
HostRequest = {address: U, capacity: U, payload: Hex, reply: Hex, service: string}
Trace = {step: U, function: string, block: string, index: U, line: U,
         op: string, lhs: string[], operands: string[],
         registers: {activation: U, name: string,
                     pre: Value | null, post: Value | null}[],
         memory: {address: U, pre: U, post: U}[]}
State = Record<string, [number, number]>
Reported = {entry: string, steps: U, budgets: Budget[], results: Value[],
            fault: U | null, faults: Fault[], tape: Tape[],
            host_requests: HostRequest[]}
Effects = {console: string | null, console_hex: Hex, clock: U,
           effect_tape_length: U, memory_digest: Digest}
Equality = {equals_entry: boolean, first_difference: string | null}
Reversal = Equality & {status: "REVERSED" | "FAILED"}
MemoryEquality = {memory_equals_entry: boolean, entry_memory_digest: Digest}

run-cc = Reported & Effects & {status: "COMPLETED" | "FAILED",
                              measurements: Measurement[], trace: Trace[]}
reverse (CC/R) = Reported & Effects & MemoryEquality & Reversal
reverse (oracle) = Reversal & {
    oracle: string, budget: Budget, payload: U, code: U, steps: U,
    forward_steps: U, reverse_steps: U, frames_clean: boolean,
    state_after_reversal: {pc: U, pc_prime: U, faulted: boolean,
                          counters: [U, U, U, U, U],
                          locations: Record<string, U>}}
reverse (action) = Reversal & {
    action: string, adjoint: boolean, wires: U, forward_state: State,
    state: State, mut: (U | null)[], in: (U | null)[]}
run-action = {action: string, adjoint: boolean, wires: U, state: State,
              mut: (U | null)[], in: (U | null)[],
              mem: Record<string, (U | null)[]>}
run-program = Reported & Effects & Equality & MemoryEquality & {
    status: "COMPLETED" | "FAILED" | "REVERSED", run_state: "CLOSED",
    reversed: boolean, measurements: Measurement[]}
```

```sh
reference-interpreter/target/debug/bennett-isa-ref check reference-interpreter/tests/fixtures/inc8.bisa
reference-interpreter/target/debug/bennett-isa-ref run-oracle reference-interpreter/tests/fixtures/inc8.bisa inc8 --input 8 --budget 3
reference-interpreter/target/debug/bennett-isa-ref step-check reference-interpreter/tests/fixtures/inc8.bisa inc8 --suite --budget 4
reference-interpreter/target/debug/bennett-isa-ref run-oracle reference-interpreter/tests/fixtures/sum_db.bisa sum_db --input 3 --mem db=0102030405060708090a0b0c0d0e0f10 --budget 1000,4080,512,64
reference-interpreter/target/debug/bennett-isa-ref run-cc reference-interpreter/tests/fixtures/classical.bisa sum --args 3
reference-interpreter/target/debug/bennett-isa-ref reverse reference-interpreter/tests/fixtures/inc8.bisa inc8 --input 8 --budget 3
reference-interpreter/target/debug/bennett-isa-ref run-action reference-interpreter/tests/fixtures/registers.bisa bell --mut 0
reference-interpreter/target/debug/bennett-isa-ref run-program reference-interpreter/tests/fixtures/registers.bisa measurement --args "" --draws 0.9
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
  whole-run reversal; `QuantumRegisters` holds §3.3 qreg handles. It also
  exposes single stepping (`step_once`, `StepPoint`), the current `outcome`,
  the `effect_tape` with transferred bytes, `memory_digest`, caller-seeded
  globals (`with_quantum_registers_and_globals`), and snapshot comparison
  (`ClassicalSnapshot::delta`, `first_difference`, `memory_equals`); the
  step machine snapshot likewise (`BasisVectorConfiguration::locations`,
  `counter_values`, `first_difference`).
- `QuantumState` is the §14.2 simulator: a basis fast path and a sparse
  superposition over any number of wires, each with an exact amplitude shadow
  in Q(√2, i)[z, 1/z] (`src/exact.rs`). Cleanliness, basis reads and state
  equality are exact. Born selection compares exact-shadow norm ratios using
  outward-rounded dyadic enclosures, refining without a precision cutoff.
  Supported outcomes survive underflow; conditional binary64 amplitudes are
  normalized with a shared extended exponent in both `amplitudes` and
  `exact_support`, including after further gates. Unprojected user vectors
  retain their supplied scale. Dyadic display conversion combines mantissa and exponent before rounding,
  including subnormals; algebraic cancellation refines exact enclosures before
  conversion when needed. Display probabilities remain
  approximations (a positive value can display as zero).
  `QuantumState::set_state_vector` installs a sparse superposition with each
  binary64 amplitude held exactly as its dyadic value.
  `Object::execute_action` runs structured Q bodies and their adjoints;
  `Object::borrow_capacity` sizes an action's §5.6 borrow pool.
- `Object::verify_certificate` and `verify_claim` re-execute exhaustive and
  sampled claims (`ClaimVerdict::Proof` only for exhaustive claims over D),
  check the complete named translation frame (canonical nested-line points,
  actual borrowed wires, exact reverse V, repeated W and side conditions) and
  library structural claims. Frame maximality inside qloop/qif is judged within
  the containing static body sequence (SPEC-QUESTIONS 30). Borrow-frame admission preserves every frame and
  completed return: definite violations cannot be waived by a region-final
  certificate. Both parsing and public certificate verification require
  permutation-only instructions touching borrowed wires, independently of tuple
  replay; unrelated quantum gates in an action remain legal. Expected programs bind bundles and region cells
  by ABI group and position, independently of local names; borrowed pools retain
  domain order and omitted partial-domain inputs start at zero. Interval
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

### Round-2 static and display semantics

Function-local `qregion` declarations are established once at function entry,
including declarations inside zero-trip loops or after their first use. Each
lexical region name is unique. `qif` arms must restore their predicate stacks;
its adjoint reverses the complete expansion, including unconditional lifetimes
(SPEC-QUESTIONS 28). `qinplace` explicit adjoints exchange the oracle pair and
retain token and effective-control versions. Exchanges retain the dependency
union and other provenance on all outputs, and their inverse histories cancel
jointly. Borrow proofs distinguish the complete classical argument and token
bindings through inlined calls. Borrow/return lifetime boundaries survive loop
composition and adjoints: each return proves restoration and independence before
repetitions compose. A borrow spanning a loop may be restored by the whole loop;
a later borrowing cannot restore an earlier completed lifetime. Loop proofs
compose per-return obligations on reachable entries, retaining known-clean
workspace invariants.

**Design (round 10).** The §16.2 parser couples `fn` exclusively to CC/R
and uses the same header recognizer at function boundaries; Q kinds are bare.
A present bounds clause and an explicit LHS each require a nonempty list;
omitted bounds and empty operand lists remain legal. Exact source bytes are
hashed without normalization. Lexical predicate stacks retain their push/pop
order, while operation identities use a separate semantic effective-control
set: resolved scalar physical wires with polarity, sorted and deduplicated.
The §6 pass retains complete current version/dependency snapshots; borrow nodes
canonicalize after physical resolution and again after callee instantiation.
ABI input/output order, callee/token identity and all-operand LIFO matching
remain significant.

**Design (fact engine).** Per-wire ANF basis facts follow every gate: diagonal
operands and read-only controls retain their values, permutations transform
their ANFs, and non-diagonal targets become independent unknowns. Per-wire
operation stacks preserve nested compute/adjoint history: an exact adjoint
restores saved expressions only when the same event tops every operand stack,
so outstanding entangling readers block restoration. Basis-only permutations
need no history unless touching an existing stack. Opaque oracle calls and
qinplace join those same causal stacks: oracle_call is self-adjoint; qinplace
swaps g/ginv. Keys retain callee(s), token/classical specialisation, actual
input/memory/output wires and effective controls. Matched pairs restore saved
facts only at the top of every operand history; unmatched opaque readers block
outer restoration, and lifetime/import barriers invalidate connected history.
Assigning qinplace starts a history even on basis entries; loops containing it
use causal transfer rather than uninterpreted ANF assignment composition.
Source lowering publishes
physical bindings, live borrows and fact nodes through one executed structured
traversal: zero-trip bodies publish none of them; static validation and lexical
qregion hoisting still apply. Each loop visit resolves against its live entry
binding map and publishes its successor map. Equal-width, equal-role successor
lifetimes reuse entry slots only after the old lifetimes end (alpha-renaming).
Distinct environments get distinct transfers; recurring maps share an exact
structured cycle, without expansion proportional to the trip count.
Local action calls reuse a memoised per-wire
structured transfer per callee/classical specialization, instantiated on actual
wires and effective controls. Known-false controls disable fact transfer;
imports leave modified outputs unknown and preserve read-only facts, while
invalidating inverse history across all opaque readers. Existing
SCC validation excludes action recursion. Every quantum gate and non-permutation
call retains its physical frame boundary and full borrowed-touch guard, even
when disabled. Mixed loops visit every actual fact entry, skip repeated full
analysis states, or prove an inductive unknown-top invariant; pure permutation
loops compose exact maps and return obligations by squaring. Every return still
proves restoration and output independence. Certificates cannot override
definite lifetime or non-permutation failures. §6 cleanup readers use complete captured
version-stack prefixes, so older readers do not block a nested exact adjoint.
Round-9 literal pins and deterministic families retain all round 4–8 negatives
and add opaque pairs, unmatched readers and carried/reallocated loop bindings,
with independent identity maps including Bell-paired borrowed references.

 Workspace widths follow executed
producing/consuming lifetimes: zero-trip bodies have no lifetime effects (lexical
qregions are still hoisted), and every inverse loop visit recreates its own
matched producer's width. Structured width plans share repeated entry maps as
cycles without a trip-count limit. Round-6 pins include nested dead bodies,
qcopy/qload/qborrow variants, changing-width qalloc/qborrow loops and
alpha-renamed unrollings. Round-5 width evidence covers its tested straight-line
and self-contained loop lifetimes; the round-6 pins extend that evidence to
loop-carried width changes.
Matched lifted oracle calls preserve all tuple
boundaries and account for every possible numeric `qjumpr` destination.

Integer comparisons require integer or pointer operands; lifted memory width
suffixes use canonical INT spelling at load. State JSON selects support exactly
and approximates rotation phases using bounded finite factors, even when their
combined symbolic angle exceeds binary64. Numerical/channel certificate claims
still report the staged rigorous interval checker as deliverable 11.

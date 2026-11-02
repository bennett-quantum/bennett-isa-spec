# Accepted K83 action region support vectors (ISA 0.9, suites F/H)

These vectors implement §11 K83, accepted in `contract-z4c-2026-11-02`
and published at spec commit `be71d03`. They keep native functions outside
of the device object and annotate device action entry/exit only.

`k83.json` (`bennett.k83.v1`) pins six canonical source objects and their
required coordinates: a zero-trip declaration, declarations in both qif
arms with nested zero-trip loops, image bytes 01 and 81, an externally
controlled call, and two calls from a caller loop. All object hashes are
checked before execution. The source and catalog are unchanged by the
annotation rule.

Each trace row is `[gate, point, ordinal, owner, bit, iteration, instance,
caller_iteration]`. Allocation owners `a0`, `a1`, ... are assigned in
first allocation order; an external wire keeps its name. `bit` is null
for allocation/release. `caller_iteration` is null at the root. Semantic
call occurrences in these vectors are 0. A trace with `complete_trace`
false lists support only, excluding ordinary qif/body gates.

`reference-interpreter/tests/k83_source_support.rs` compares actual linked
source expansion with these coordinates and executes the source, circuit
and raw inverse over every external basis value, including both control
values. A raw inverse retains forward coordinates and identities while
visiting them in reverse. This does not equate raw inversion with K62
source-adjoint formation.

Regenerate/check from the repository root, with the reference CLI built:

```sh
python3 -I conformance/fixtures/accepted-k83-09/generate.py --check
```

The eight schedule vectors in the same JSON cover valid support selectors, rejected
`$step`/`$commit` action selectors, invalid own iteration/instance vectors, duplicate
maximal entries on actual support (including a false control), and an empty support
domain that cannot witness a tie. Six `F-k83-*` and eight `H-k83-*` cases exercise
CLI admission. `conformance/tests/ref_k83.rs` independently checks the coordinate
adapter and full basis matrices.

The generator only derives canonical object identities through the
published `hash`/`check` CLI. Coordinate expectations are independently
written from K83, and its ref envelope pin is checked before generation.

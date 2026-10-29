# Job: the Bennett ISA 0.8 reference interpreter in Rust

Working directory: the git repository `bennett-isa-spec`. Build the complete reference implementation
of the ISA specification `spec/Bennett-ISA-Spec.md` (draft 0.8, accepted) as the Rust crate in
`reference-interpreter/` (crate `bennett-isa-ref`, currently an empty skeleton). Read first: `CLAUDE.md`
(fail fast and loud, red-green TDD, exhaustive verification, skepticism), `HANDOFF.md`, `spec/README.md`,
then the whole specification, then `reference-interpreter/isa_check.py` (a Python seed that parses
§16, hashes §9.2 envelopes, and executes the lifted rows used by the two §16.3 examples).

## Scope is the whole ISA. Nothing may be left out.

This interpreter is the executable model every other Bennett tool will be checked against. A previous
document limited itself to what a partial model could execute, and that was rejected: the project's
target programs (for example an unmodified SQLite query evaluated as a lifted oracle) need every lifted
row, including memory, heap allocation, calls and procedures. So implement every instruction and row of
the specification. If the text is ambiguous or seems contradictory, do not drop the feature: choose the
reading most consistent with the rest of the specification, implement it, and record the question, the
reading and the section in `reference-interpreter/SPEC-QUESTIONS.md`. Never edit `spec/`.

## Deliverables, in priority order (each with tests written and seen failing before the code)

1. **Parser and well-formedness** for the §16 canonical textual form: every construct of §16.2 and of
   the sections it references; rejection of ill-formed objects with the rule cited (suite F, §15).
2. **Identity** (§9.2): envelopes, SCC hashing, symbol-reference replacement; SHA-256 implemented in the
   crate (standard FIPS 180-4 test vectors). Reproduce the four published hashes of §16.3 exactly.
3. **§14.1 reference interpreter** for oracle reference bodies, every §4.2 row: const, binop, div,
   icmp, select, cast, mov, parmov, label, addr, boundscheck, load, store, alloc, free, faultif, call,
   ret, retval, oracle_call, oracle_uncall, qjump, qbranch, qjumpr, fault; lprocs and their closure
   (§3.7); activation records, stack accounting, `alloctab`, globals and interface regions (§7.2);
   budgets B / heap / stack / depth and every fault code (§5.4). It defines F_B and the budget unit.
4. **The predicated step machine as a reversible machine on basis configurations** (§3.6, §3.7): the
   forward step and the inverse step exactly as printed (dispatch, slots, flag, succ, commit, delivery,
   the printed inverse lists of the A+D rows), executed on concrete bit-vectors. Check, exhaustively at
   small widths, that (a) the machine's result equals the §14.1 interpreter's F_B on every input, and (b)
   running B forward steps then B inverse steps restores every register, frame and memory cell (the
   prefix-reversal property, T2 of §14.3), for success and for every fault path.
5. **Classical-control and reversible tiers** (§3.4, §4.1, §7.1, §7.3, §7.4): instructions, memory and
   allocation table, effect and entropy tapes, the run history, `call`/`ret`/`uncall` segment rules.
6. **Q-tier semantics** (§4.3–§4.7, §5, §6, §14.2): allocation, gates, copy/XOR/exchange/in-place,
   quantum memory and predication, region calls, borrowed systems; a state-vector simulator for small
   widths and a basis-configuration (permutation) fast path; the §6 uncomputation-validity checks.
7. **A command-line tool** `bennett-isa-ref` with at least: `check <object>` (parse + well-formedness +
   hashes), `run-oracle <object> <oracle> --input <x> [--budget ...]` printing F_B and the step count as
   JSON, `step-check <object> <oracle> --exhaustive` (deliverable 4), so other repositories' tools can call
   it. Document every subcommand in `reference-interpreter/README.md`.

Test fixtures: write new §16 text objects under `reference-interpreter/tests/fixtures/` that exercise
every row and every fault code, each with a hand-derived expected result written in the fixture's
comments before running it. Port every check `isa_check.py` performs. Keep `cargo test` green and fast
(long exhaustive sweeps behind `--ignored`, run them anyway and record the result).

## Constraints

- Rust standard library only, no external crates (the network is unreliable and the build must work
  offline). Edition 2021. `cargo test` must pass with no warnings you introduced.
- Write only inside `reference-interpreter/`. Leave `isa_check.py` as it is.
- Progress goes in `reference-interpreter/PROGRESS.md` (see the job protocol below): per deliverable,
  status, the verifying command and its result.
- Correctness over speed of delivery: do not mark a deliverable done without its tests and an
  independent spot check of at least one stated result by hand.

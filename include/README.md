# Published C ABI

These headers render the C declarations in the council-accepted ISA 0.8:
`bennett_launch.h` is **launch 0.8**, §10; `bennett_sched.h` is **sched 0.8**,
§11. The earlier task description's “launch 0.3” is not the accepted version.
The scheduling header includes the launch header for `bennett_status`.
Both work independently in C11 and C++17 and provide C linkage.

The declaration inventories are:

| Header | Functions | Typedefs (including enums) | Enum constants | Total named symbols |
|---|---:|---:|---:|---:|
| `bennett_launch.h` | 41 | 12 | 17 | 70 |
| `bennett_sched.h` | 15 | 3 | 3 | 21 |

Counts exclude include guards, struct tags sharing typedef names, and names
imported from the launch header. No run-state enum, version macro, fault-code
C enum, EOF code, implementation, or private VM extension is defined by the
§10/§11 C blocks. Fault values are specified separately in §5.4.

Each declaration cites the normative C declaration or enumerator. Adjacent
comments also quote the applicable prose. Object layouts remain opaque.
Section 10 transfers returned objects to the caller and specifies matching
`*_free` functions; the program has the explicitly listed `bennett_unload`.
Arguments and entropy are copied at `bennett_run`, tokens belong to the program
until unload, and accessor pointers last until their owning object is freed.
A run exclusively owns touched registers and memory until CLOSED. Closing
releases the run record while unmeasured/unfreed register ids survive.
An unknown register state is released through an in-run measurement, which
consumes the handle; prepare/free require cleanliness.

Result encodings are the §10 little-endian bytes, including binary64 F64 and
64-bit register/token ids. A short result-value buffer returns `E_ARG` without
writes. Fault codes follow §5.4 in occurrence order; FAILED runs expose no
values. Tape operands are §7.3's ordered unsigned 64-bit little-endian fields,
with pointers zero-extended. Certificate bytes use §12 JCS, coverage §13,
and schedule documents §11; §9.1 requires decimal strings for integers that
may exceed 2^53. These documents stay byte buffers, not invented C layouts.

`conformance/tests/headers.rs` reads every declaration directly from the
normative §10/§11 C blocks, checks inventory counts, enum values, handle types,
and every function signature, and compiles the probes with `-Wall -Werror`
under C11 and C++17. It checks repeated includes, both header orders and C
linkage. The two contract tests were observed failing against the stubs
before the declarations were added.

Verification for `b-isa-spec-c0k`: the headless offline conformance crate
passed all 20 tests. Scratch mutation checks independently removed each of
the 91 named symbols; every removal failed in both languages. Changed
parameter and handle widths failed too, and C++17 rejected either header
without its C linkage wrapper. This verifies the declarations; the
conformance harness's explicitly staged runtime cases remain staged.

The read-only VM candidate's 41 normative function signatures, 12 typedefs
and 17 enum constants match §10. Its six additional functions and
`bennett_snapshot_t` remain private. The VM header has no §11 declarations.
Because it redeclares the enums itself after including `bennett_launch.h`,
it will need a downstream cleanup when adopting these populated headers.

Contract gaps filed as bugs with label `isa-0.9` (the accepted spec is unchanged):

- **b-isa-spec-4w0 — host bit encoding.** §10 declares `const uint8_t* bits,
  size_t nbits` and `uint8_t* bits_out, size_t cap, size_t* nbits` without
  defining packing, bit order, padding, or measurement short-buffer behavior.
  The VM chose one byte 0/1 per wire in wire order; that is not a header promise.
- **b-isa-spec-7wk — tape exhaustion.** “`bennett_tape_next` yields each
  `effect` in tape order” supplies no EOF status or output behavior.
  The VM chose `E_INVALID_STATE`, a diagnostic and untouched outputs.
- **b-isa-spec-813 — synchronization and progress.** “`bennett_wait` blocks
  until the run leaves `RUNNING`” and exclusive run ownership do not specify
  concurrent host calls or execution progress. The VM chose cooperative runs
  and external synchronization per program.
- **b-isa-spec-5z8 — general failure/precondition contract.** The listed
  `BENNETT_E_ARG` and `BENNETT_E_NOMEM` do not define general NULL/index
  handling, output preservation, or allocation-failure coverage. The VM
  requires valid objects/out-length pointers for status-less accessors,
  accepts NULL frees, and can abort on allocator exhaustion outside recoverable
  argument-vector allocation. These choices are not added here.
- **b-isa-spec-c6w — scheduling call contracts.** §11's setters and frees do
  not state input copying, retained-buffer lifetimes, response independence,
  the scope of §10's ownership/pointer rules, required setters/defaults, or
  status-versus-outcome failures. The header adds no such promises.
- **b-isa-spec-923 — private VM interfaces.** The published run functions
  omit `bennett_run_step`, `bennett_run_snapshot`, `bennett_snapshot_jcs`,
  `bennett_snapshot_free`, `bennett_run_reverse_to` and `bennett_last_error`.
  In particular §10's “Reverse eligibility: `COMPLETED`” does not specify
  reversing a SUSPENDED/FAILED prefix. Their signatures, states, snapshot
  ownership and diagnostic lifetimes need an explicit future contract.

These gaps require specification decisions; this rendering neither adopts
VM conventions nor removes the associated capabilities from project scope.

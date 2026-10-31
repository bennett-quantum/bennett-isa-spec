# Editor response: council round 21

**CODEX-R21-B01 / SOL-R21-B01 and CODEX-R21-B02 are resolved.**
SOL-R21-B01 ≡ CODEX-R21-B01: both seats identified the same all-cells expansion,
repaired once. Admission, sparse allocation and the independently pinned
identities are preserved. No ISA width, exponent, transfer-length or sample cap
was introduced. No specification amendment or new theorem assumption is needed.

Implementation commit: f931307a002877317fb6d35813e97023f0aa2917.
Interpreter evidence commit: 0522df13e67f68f398242706b5284e2fc866dd1f.

## Blocker disposition

| Blocker | RED tests / observations | Fix location | Commit |
|---|---|---|---|
| CODEX-R21-B01 ≡ SOL-R21-B01 | `r21_resources::r21_b01_claims_public_consumers`: child SIGABRT, allocator failure during `cells:"all"` expansion | `claims.rs::Object::domain`; `lib.rs::reserve_more` | f931307a002877317fb6d35813e97023f0aa2917 |
| CODEX-R21-B02 | `r21_resources::{r21_b02_hostcall_public_consumer,r21_b02_console_public_consumer,r21_b02_cli_pinned_transfers}`: library children SIGABRT at length 2^63−1; original pinned CLI transfer exits 101 / capacity overflow | `classical.rs::{transfer_bytes,memory_bytes,effect,step}`; `lib.rs::{reserved,reserve_more}` | f931307a002877317fb6d35813e97023f0aa2917 |

### All-cells claims

Determining clauses: §§4.6, 12.1–12.2. A region has 2^A cells; `cells:"all"`
is legal, and replay consumes components in listed order. `reserve_more`
checks the cumulative component count and reserves fallibly before the first
cell push. The existing replay engine constructs the complete cell state even
for count=1, so this repair keeps the eager representation and returns a
§12.2 host-limit Result when storage is unavailable. This is the expressly
allowed checked-eager repair; symbolic replay of an enormous complete region
has not been implemented or claimed. No cells are truncated or relabelled as
an invalid mathematical range. Host resource errors are neither Proof nor
Evidence nor Unverified verdicts.

The codex A=63 witness retains interface=closure
`f30725fa31cc35b68090b82cfd046681c4342821591cd2d6dd5300ea522499af` and object_hash
`5fa6df63e007ad2bca8ab69c5a0c82e7671c585982048cf99919e59cc48a1e43`.
The sol A=63 file retains SHA-256
`dfca89aa2073705841586a1c4a76e27be92618417476e9cb3b9016c0e09ab3c4`.
Both complete witnesses, the codex A=0/1/64 files, sol's sampled A=1 control,
and sol's eight-row exhaustive identity control are copied byte-for-byte to
`tests/fixtures/r21/`. All parse/check_hashes successfully. Public verify_claim
and verify_certificate both return cited host limits for large domains; small
sampled controls remain Evidence { elements: 1 }, and the eight-row table
remains Proof. Generated variants cover both all-cells and explicit-cell lists.

The original abort was observed before the repair (`.tmp/r21-red.log`).
The copied reviewer probe source is retained beside the fixtures. Tests run a
fresh executable through Python's resource guard with RLIMIT_AS=256 MiB and
RLIMIT_CORE=0, and catch_unwind inside the child. The parent asserts normal
successful test-process exit; a signal, panic, missing error or false verdict
fails the regression. No unsafe code or dependency was added.

### Classical transfers

Determining clauses: §7.1 sparse memory and allocation checks; §10 copies the
request bytes; §7.3 validates before any change. `transfer_bytes` reserves the
entire transfer before reading bytes. The caller supplies §10 for hostcall,
§7.3 for console.write; direct memory queries cite §7.1. Console growth uses
checked cumulative length and a fallible reserve; the effect-tape entry is
reserved before device mutation. A failed transfer publishes no HOST_REQUEST,
console bytes or tape entry. Complete snapshots match the pre-instruction
prefix, and reverse restores the entry snapshot. Out-of-range intervals retain
memory faults; entropy shortage retains fault 10, without resource relabelling.

The exact CC and R witnesses still parse/check/hash with interface=closure
`23802c541101025a57a13ffb5f00ca72a7ad5ff95f78e9a4fd565c59f566752d` and
`2751a25e6e906ea512bc075da07cda7463161576c56cebf06455ba2d9dfab704` respectively.
Their literal independent envelopes are retained and compared with production
identity bytes. The library catches no panic; both CLI executions exit 1 with
empty stdout and a cited host-limit diagnostic, rather than exit 101. Lengths
0/1/8 still transfer successfully. Sparse allocation remains legal at all six
length boundaries, including 2^64−1 with a matching 64-bit heap bound.

To obtain valid isolated library RED evidence after the generated controls were
repinned, the old unreserved transfer materialization was temporarily restored
and tested, then repaired source restored in a finally block. All three B02
regressions fail (`.tmp/r21-transfer-consumer-red.log`): two SIGABRT children
and the exact CLI's capacity-overflow panic. Initial exploratory logs with
unrepinned generated controls are not used as transfer-consumer evidence.

## Cardinality / transfer-length inventory

**24 tagged paths / 186 generated cases**, plus pinned controls and reversals.
This is an extension of the retained R20 102-path / 1,113-case family; its
consumer paths overlap that inventory, so no disjoint 126-path total is claimed.
R20's scalar-width tests and certificate admission validator did not exercise
executable all-cells expansion or classical transfer materialization. Its broad
storage-coverage description is superseded by this distinction in README and
PROGRESS; the earlier response is immutable under this implementer's ownership.

Exponents: 0/1/3/62/63/64/127/128 for region counts; 1/2/3/62/63/64/127/128 for
value powers and two-component products. Lengths: 0/1/8/2^63−1/2^63/2^64−1.
Memory loads use byte-aligned bit widths 8/64/2^63−8/2^63/2^64−8, since N is a
u64 bit width, not an arbitrary byte count. Dense amplitude exponents are
1/3/62/63/64. Expectations are assigned before execution: feasible success,
range/fault, or cited host resource. Allocation-dependent byte failures are
made deterministic by the subprocess memory limit, not advertised as ISA caps.

| Path(s) | Where guarded / consumed | Paths | Cases |
|---|---|---:|---:|
| Executable `cells:"all"` and explicit-list action domains | `claims::Object::domain` cumulative reserve; `execute_element_audited` region-cell reserve; parse/check_hashes → both verify_claim and verify_certificate | 2 | 32 |
| 2^W exhaustive values; product of two component cardinalities | `claims::enumerate` checked span/product/u128/usize and fallible element/tuple reserves | 2 | 16 |
| Sample replay count | `claims::sample_elements`, per-tuple reservation before draws | 1 | 6 |
| Oracle certificate replay region | `claims::execute_element_audited` region-byte and output-cell reserves (direct semantic consumer) | 1 | 8 |
| Hostcall; console.write transfer | `classical::transfer_bytes`, request publication / console append after reservation | 2 | 12 |
| Public memory query | `ClassicalMachine::memory_bytes`, in-range check and transfer reserve; both instruction fixtures | 1 | 12 |
| Retained memory undo bytes | `classical::save_bytes`, cumulative reserve before length loop (direct consumer) | 1 | 6 |
| Entropy transfer | availability check before undo/transfer reserve and mutation; shortages stay fault 10 | 1 | 6 |
| Cumulative console bytes | `reserve_more`, checked-add + fallible reserve with eight resident bytes (direct capacity consumer) | 1 | 6 |
| Nested oracle region transfer | `oracle::region_bytes`, checked power/conversion and fallible byte reserve | 1 | 8 |
| Memory-load byte storage | `Memory::load`, fallible reserve before reading N/8 bytes | 1 | 5 |
| Allocation-table exponent | `power_of_two` / `allocation_fails`: symbolic capacity, no 2^A entry expansion | 1 | 8 |
| Sparse allocation length | `Memory::allocate`, one fallibly reserved entry; never n bytes | 1 | 6 |
| Sparse memory digest and snapshot trace consumers | `ClassicalSnapshot::memory_digest`, resident addresses/entries/nonzero bytes only, before each CC/R transfer | 1 | 12 |
| Qreg allocation, qreg slice length, dense amplitude cardinality | `QuantumState::allocate`, checked slice endpoint + fallible slice reserve, `QuantumState::amplitudes` checked power + reserve | 3 | 17 |
| CLI interface region exponent | `build_action_state`: checked cells/product, value/cell reservations before loops | 1 | 8 |
| CLI sparse digest/trace/output | `drive` reserves records incrementally, snapshots/deltas/JSON reflect three executed rows and resident entries | 1 | 6 |
| CLI output byte expansion | `hex_buffer` checked 2*len and fallible String reserve, propagated through console/tape/host output | 1 | 6 |
| CLI sample replay count | `step_check` lazy current-tuple/odometer, no sample-count output buffer | 1 | 6 |
| **Total** | | **24** | **186** |

Counts include both public claim entry points, but exclude the 13 pinned claim
checks, two exact CLI transfer checks, identity-envelope assertions, qreg concat
control, two completed small CLI sample controls and 12 transfer-prefix reversal
controls. The huge CLI sample probes stop on the first element at a downstream
frame-storage host error; they establish lazy entry, not completion of an
astronomical replay. Slice lengths above a live bundle remain range errors.

Other related consumers were inspected and guarded: host-reply byte copies and
retained memory undo, entropy buffers, qreg concat checked sum/reservation, and
CLI region cells. These accept resident payloads/handles, not independently
constructible virtual slices of huge length. Existing classical/register tests
exercise successful replies, concat and their exact reversal. Clock/block
transfers are fixed 8/512-byte operations. Trace limits and CLI odometers remain
incremental, without an ISA trip cap. Parsed array sizes, resident qreg wire
lists, sparse hash/delta traversal and already-stored tables do not expand a
virtual length. This finite family is not a proof against arbitrary host OOM,
all possible resident-object growth, or every value of u64.

### Further defect exposed

`oracle::region_bytes` still collected 2^A bytes infallibly, and claim replay
constructed whole region byte/cell vectors without a reservation. Those
materializations now reserve before iteration. Reinstalling the original
nested-region collection for the memory-limited direct-consumer regression
reproduces SIGABRT (`.tmp/r21-nested-region-red.log`); repaired source is restored
in a finally block. `Memory::load`, undo/reply/entropy, qreg slice/concat and CLI
hex/trace storage were also hardened while tracing the same class. These are
implementation resource-reporting repairs, not new spec gaps. Bead
**b-isa-spec-3va** records the class and the additional nested-transfer abort.

### Suggestions

Sol S1 is applied: derived exponent boundaries, both tagged region alternatives,
independent result expectations, isolated processes and explicit consumer rows
are added. Codex's structured Error-category S1 is deferred as nonblocking:
`Error(pub String)` is a public tuple API used throughout the interpreter and
contextual wrappers. A category migration preserving that API and wrapper
semantics needs its own coordinated change; it is not a cheap local repair.
Existing Display/CLI text and cited range/resource distinctions are preserved.
The other criterion-level test suggestions are not claimed implemented here.

## Validation and retained evidence

Baseline: **425 passed / 0 failed / 5 ignored**, **105.24 s** runner sum across
**33 result blocks**.
Final: **436 passed / 0 failed / 5 ignored**, **139.44 s** runner sum across
**34 result blocks**, **18 unit + 418 integration**, 0 doctests.
All five ignored release sweeps: **5 passed / 0 failed**, **88.31 s** runner sum.

| Sweep | Full domain | Result / runner time |
|---|---|---|
| u8 division/remainder | 65,536 pairs × 4 = 262,144 executions | 1/0; 0.39 s |
| 17-wire borrow | 131,072 entries | 1/0; 2.12 s |
| checked borrow certificate | 262,144 elements, subject and expected program | 1/0; 12.16 s |
| §6 differential | 200,000 programs; 70,931 accepted clean / 129,069 rejected | 1/0; 71.22 s |
| recursive prefix reversal | 256 inputs × all completion/first-fault prefixes | 1/0; 2.42 s |

Fmt/fmt --check and strict all-target offline Clippy pass with zero warnings.
The unchanged Python seed verifies two example objects and two printed envelopes.
`r20_pins.py` retains both original extreme literal pins. `r21_pins.py` checks
nine complete reviewer-file SHA-256s and two literal envelopes independently of
production code; all nine files also compare byte-for-byte with the read-only
reviewer originals. The focused R21 family passes **11 / 0**, runner sum 0.49 s.

All round 4–21 tests are retained. Production source and test inputs remained
fixed during final full tests, sweeps and strict Clippy. Logs are under interpreter
`.tmp/`: `r21-{baseline,red,transfer-consumer-red,nested-region-red,family,final,
sweeps,final-clippy}.log`. Initial exploratory transfer logs with invalid
controls are superseded by the correctly repinned consumer RED log above.

Each cargo invocation used the mandatory headless prefix and project TMPDIR:

```sh
ENV_PREFIX='env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp'
MANIFEST=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml
$ENV_PREFIX cargo test --offline --manifest-path "$MANIFEST"
$ENV_PREFIX cargo test --offline --manifest-path "$MANIFEST" r21_ -- --nocapture
$ENV_PREFIX cargo test --offline --release --manifest-path "$MANIFEST" -- --ignored --nocapture
$ENV_PREFIX cargo fmt --manifest-path "$MANIFEST"
$ENV_PREFIX cargo fmt --manifest-path "$MANIFEST" -- --check
$ENV_PREFIX cargo clippy --offline --manifest-path "$MANIFEST" --all-targets -- -D warnings
```

Deliverables 8–11 remain staged/Unverified. No literal other-value theorem is
newly claimed proved. No user decision is required for this repair. Only
`reference-interpreter/` and this external response are committed; no spec,
conformance, worklog, foreign file, push or Dolt sync. Beads' auto-export is left
for the orchestrator. Both seats' reported R21 blockers are addressed above;
any further orchestrator steering can be appended here.

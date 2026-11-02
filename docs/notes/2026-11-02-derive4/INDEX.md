# Contract derivations 4 — amendment queue

Proposals only. These are **amendments, never prerequisites for rungs 3/4** (advisor R10). F30 stays deferred until rung 3 or 4 lands. The brief names 29 contract beads plus F23's VM contract-side review: 30 rows. Existing issue status is not changed by writing a proposal.

| Bead | Audit/source | Verdict | Route / derivation |
|---|---|---|---|
| b-isa-spec-x1y | F04 | Boundary capability gap | [Native adapter + memory companion; default 0.10](native-sharing.md) |
| b-isa-spec-6ma | F08 | Boundary capability gap | [Same amendment](native-sharing.md) |
| bennett-vm-8v0 | F23 | Contract side derived; VM proposal exists | [Consume accepted companion, implementation remains open](native-sharing.md) |
| b-dialect-spec-yqf | F14 | Promotion-only realization loses native host | [Versioned dialect placement amendment](b-dialect-spec-yqf.md) |
| b-dialect-spec-hsb | F11 | Whole-native-link state inventory locks out mixed mode | [Versioned dialect state-closure amendment](b-dialect-spec-hsb.md) |
| b-isa-spec-8uz | Scheduler root class | Nondefault root class is not serialized | [0.10 schedule field; authorized 0.9 K alternative](b-isa-spec-8uz.md) |
| b-isa-spec-rvu | Borrow owner/idle set | Owner alias/default is not determined | [0.10 owner schema and default](b-isa-spec-rvu.md) |
| b-isa-spec-n5a | Adjoint proof binding | Cross-region proof has no checked subject | [0.10 certificate binding repair](b-isa-spec-n5a.md) |
| b-isa-spec-tea | EST-SOURCE-03 | Complete return scan exceeds delivery slot | [0.10 frame/catalog binding amendment](source-binding.md) |
| b-isa-spec-99n | EST-SOURCE-04 | Legal address widths cannot bind one P | [Same source-binding amendment](source-binding.md) |
| b-isa-spec-gck | K81 phase ownership | Fresh value cleanup loses original phase inverse | [0.10 phase/segment ownership amendment](b-isa-spec-gck.md) |
| b-isa-spec-lop | F01 + driver audit | Outer associations missing | [Versioned inventory companion; 0.10 integration](container-and-certificates.md) |
| b-isa-spec-cob | F02 | Existing link checks compatible within device program | [Same inventory/scope amendment](container-and-certificates.md) |
| b-isa-spec-8nx | F03 | Device inventory/root association needs explicit scope | [Same inventory/certificate amendment](container-and-certificates.md) |
| b-isa-spec-c1u | F05 | Existing tuples valid; mixed association missing | [Invocation budget scope companion](budgets.md) |
| b-isa-spec-ml0 | F06 | Loaded-program routing/lifetime missing | [Versioned ledger companion](resource-routing.md) |
| b-isa-spec-at6 | F07 | Journal lacks mixed invocation/profile binding | [Versioned journal/checkpoint companion](journals-and-reversal.md) |
| b-isa-spec-35m | F09 | Existing reverse is per device run | [Same run-scope/memory checkpoint amendment](journals-and-reversal.md) |
| b-dialect-spec-pou | PPC zero-low parity | Covered by exact derive3 proposal; no duplicate | [derive3 4yg.1, §13.1 repair; no ISA K](../2026-11-02-derive3/bennett-arith-4yg.1.md) |
| b-dialect-spec-q06 | Runtime instance delivery | Covered by exact derive3 proposal; no duplicate | [derive3 brt-vbi, §§12.8/19.1 protocol; no ISA K](../2026-11-02-derive3/brt-vbi.md) |
| b-dialect-spec-yz7 | Driver process convention | Callable entry/context ownership missing | [Versioned dialect entry-role amendment](process-context.md) |
| b-dialect-spec-k08 | F10 | Whole-program visibility is compatible with selective lowering | [Versioned dialect closure-scope amendment](device-closure.md) |
| b-dialect-spec-qbw | F12 | Whole-device startup must retain mode scope | [Same entry-role amendment](process-context.md) |
| b-dialect-spec-jwd | F13 | Preserve escape rules; qualify opaque transport | [Same routing + dialect amendment](resource-routing.md) |
| b-dialect-spec-5dl | F15 | Native graphs lack device bump/metadata history | [Versioned import/layout profile amendment](entry-state-and-addresses.md) |
| b-dialect-spec-dce | F16 | Fixed profile valid; general mapping remains required | [Same entry/address-profile amendment](entry-state-and-addresses.md) |
| b-dialect-spec-dl8 | F17 | Libc/effects closure is per device variant | [Same device-closure amendment](device-closure.md) |
| b-dialect-spec-3ca | F18 | Copied outbound services do not register inbound memory | [Versioned service/reentry amendment](hostcalls.md) |
| b-dialect-spec-bly | F19 | Device-link table/frozen capacities need outer binding | [Same budget + dialect amendment](budgets.md) |
| b-dialect-spec-dh8 | F20 | Device closure and native host need separate comparison subjects | [Same closure/comparison amendment](device-closure.md) |

Each note cites the shared [fat-object scope](../2026-11-02-rev2/fat-object-scope.md) and [audit](../2026-11-02-4ht-audit/AUDIT.md). Same-version ISA K routes always require council + maintainer authorization of compatibility rules; otherwise semantic changes route to 0.10. No git command or full gate is part of this job.

## Dispositions and publication boundary

Sixteen grouped notes cover 28 rows; two rows cite complete existing derivations. The actual pou record requests exactly the zero-low PPC tie repair already derived in [4yg.1](../2026-11-02-derive3/bennett-arith-4yg.1.md). The actual q06 record requests exactly the runtime all-width request/manifest/fixed-point protocol in [brt-vbi](../2026-11-02-derive3/brt-vbi.md). They need no duplicate proposal or ISA K. This pass confirms their scope against the actual issue records; it does not rerun derive3's native arithmetic or LLVM runtime-body audits. Their prior execution results remain labelled prior evidence. Both dispositions are amendments, never prerequisites for rungs 3/4, and F30 remains deferred.

F23 has an existing [VM design proposal](../../../../bennett-vm/docs/PLAN.md), not a published ABI or implemented backend. The native-sharing note derives the contract obligations it must consume. Existing 0jq/7s0/djm and shared-scope notes overlap inventory, frame proof and process preparation, but do not resolve the new owner, cross-region adjoint, callable-entry or native-memory protocols. They are cited rather than rewritten. F31's boundary-preservation inference is retained as an associated obligation in device-closure, without adding a bead to the requested 30-row index or claiming an observed inliner bug.

All source issues remain open for their publication/implementation owners. This notes-only job has completed the derivations; it does not close amendments that have not been ratified or implemented. No spec, header, catalog, interpreter or sibling source was edited. The runner owns commits; no commit SHA is asserted here.

## Evidence and bounded verification

[sources.json](sources.json) preserves all 30 actual issue descriptions/titles/notes and nine cited source snapshots with SHA-256 values and exact excerpts. ISA records were read with `bd --readonly show --json` from the main checkout. The required direct dialect/VM reads were attempted from their main checkouts and failed on read-only embedded Dolt LOCK/gate files. Copying each actual embedded database plus its metadata/config into `.tmp/sol-derive4/` permitted `bd --readonly show --json` there. No passive JSONL import, sibling tracker mutation or remote sync was used. Source snapshots are evidence pins; the verifier reports concurrent source drift separately from its frozen excerpts.

The job-built reference interpreter was compiled under a **4 GiB address-space limit**, offline with two build jobs. Environment for all commands below:

```sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER \
  TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp/sol-derive4/tmp \
  CARGO_TARGET_DIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp/sol-derive4/target
```

Build command after the shell limits and environment prefix:

```sh
ulimit -v 4194304
ulimit -c 0
# Apply the environment prefix above to this command:
timeout 240 cargo build --offline -j2 \
  --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml
```

Observed exit 0 in 22.59 seconds. No cargo test suite or full gate was run. Reference source was read-only; all build artifacts are in the job's own target directory.

The complete focused verification command is:

```sh
ulimit -v 1048576
ulimit -c 0
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER \
  TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp/sol-derive4/tmp \
  CARGO_TARGET_DIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp/sol-derive4/target \
  timeout 90 python3 -I -B -u \
  /home/tobias/Projects/bennett-workspace/bennett-isa-spec/docs/notes/2026-11-02-derive4/verify.py
```

Observed exit 0. The script additionally enforces its own 1 GiB limit and 15-second deadline on each reference subprocess. Results:

| Check | Observed result and limit |
|---|---|
| Index/notes/citations | 30 unique rows; 16 grouped notes; two prior dispositions; all required sections, links, fences, fat-object/R10/publication rules pass |
| Frozen evidence | 30 issue records and nine source snapshots pass; zero source drift on the verification run |
| Native boundary witnesses | Finite alias/epoch, metadata invalidation, wrong-ledger/generation, lease-conflict, payload-integrity, device-closure, root-context, token-value/identity and entry-role counterexamples pass; these are protocol models, not a VM/compiler implementation |
| Class/borrow/adjoint | Capacity/default-class distinction, occupancy 14 versus 10, all eight forward and reversed four-CX columns and B-dependent phase counterexample pass; no scheduler/checker C ABI probe rerun |
| qjumpr candidate | Actual published recipe transformed in memory; 1,216 bounded P/A/First/End/address/initial-successor cases match full-width predicates, XOR/fault writes and exact inverse/scratch restoration |
| Return binding | Complete scan width exceeds old N=4; W1=8 changes history 17 to 21 bits and admits every named scan bit; this is width/arithmetic evidence, not full lift.ret expansion conformance |
| Reference source objects | `check`/identity validates both read-only estimator fixtures and the new diagonal fixture |
| Reference return fixture | `run-oracle .../ret-delivery-source-gap.bisa root --input '' --budget 4,1,8,1`: payload 1, code 0, steps 4 |
| Reference jump fixture | `run-oracle .../jump-width-source-gap.bisa root --input 1,2,1 --budget 3,0,0,0`: payload 1, code 0, steps 3 |
| Reference diagonal fixture | Four basis calls preserve t/x; forward and adjoint on `00=0.6;01=0.8` preserve the complete amplitudes; both reverse calls report REVERSED, equals_entry=true, no first difference |

The candidate qjumpr recipe hashes to `bf4e2d395f642cdb645184c18650e867c614edc259ff9f5e74f29047d00acd1c` on the witness's canonical JSON domain. This is a recipe candidate fingerprint, **not** a published catalog/envelope hash. The [diagonal fixture](diagonal-source.bisa) has object_hash `cabcde923d3c5b8a66398caf34778ca64c799ea90c9e7b86d7d5be09b8a559bb` and interface/closure hash `8eb4012028bd0aa8b5ecccfd89b72f1ae46be842e4f421ec0502e04f656751b5`.

One corrected evidence detail: gck's lowercase `z` shorthand is not a §4.4 opcode. The initial reference attempt rejected it. The admitted fixture uses forward S² and inverse SDG², retaining the exact I-versus-unwanted-Z counterexample. This is a witness normalization, not an interpreter repair or a claim that the schematic spelling was admitted source.

These checks establish the stated finite witnesses and document coverage. They do not establish a general cleanup-segmentation theorem, an implemented native container/adapter, live registered-memory sharing, a production certificate schema, a complete points-to analysis or end-to-end mixed execution. Those remain explicit amendment/implementation work, with all North Star capability domains preserved. F30 remains deferred; existing rungs retain their original acceptance.

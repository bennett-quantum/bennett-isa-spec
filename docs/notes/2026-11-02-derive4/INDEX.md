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

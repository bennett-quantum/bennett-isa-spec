# Cross-repository bead text — fat-object audit

Proposed issue bodies for the orchestrator to file in the named owning repository. **No issue below has been filed by this job.** F01–F31 are audit keys, not Beads ids. Each body is one task with label `fat-object`; keep a reference to parent `bennett-4ht`. Dependencies name audit keys for conversion to the eventual owning-repo bead references. These are design/publication or focused acceptance tasks, not permission to implement an unpublished interface or to narrow source capabilities.

For F30, defer filing until rung 3 or rung 4 lands, per the runner brief. File the other boundary decisions as appropriate now; neutral findings preserve rules and add explicit scope, rather than reporting a currently observed failure. Coordinate future versioned contract publication through the owning spec process. Consumer tasks depend on those publications; existing rungs are not newly blocked by this audit.

## F01 — Publish a versioned native-container inventory around unchanged ISA objects

Owner: `bennett-isa-spec`  
Type: `task`  
Priority: `P2`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **makes harder**

Source: [ISA §§9.1–9.2][isa-object], especially 1916–1918, 2021–2025. See [AUDIT.md](AUDIT.md), row F01, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: The loader input is canonical ISA text with a closed section schema; `object_hash` is an ISA-object identity. It is not a native executable format. This is a makes harder finding for the mixed case described in F01; it does not establish a current runtime failure.

Requested change: Specify a versioned outer container inventory: native payload, each embedded ISA payload's bytes/digest and ISA `object_hash`, selected entry/root, certificate/build-record association and required versions. Native-container identity is separate. Extract inner objects for existing parsers; do not add arbitrary native fields to object 0.9.

Acceptance: A normative schema distinguishes container digest, embedded bytes/digest, ISA object_hash, selected root and certificate/build association. A two-payload example extracts byte-identical admissible inner objects, and mismatched payload or version associations are diagnosed; unknown inner fields remain errors.

Cheap now: Yes: reserve the boundary/schema now; packing and loading later.

Dependencies: None beyond the standing bennett-4ht/shared-scope obligation.

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F02 — Delimit ISA linker conflicts to each linked device program

Owner: `bennett-isa-spec`  
Type: `task`  
Priority: `P3`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **neutral**

Source: [ISA §9.3][isa-link]. See [AUDIT.md](AUDIT.md), row F02, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: An ISA link resolves imports by name/signature and rejects conflicting layouts/data or unequal definitions under one name. A flat device link has one compatible layout. This is a neutral finding for the mixed case described in F02; it does not establish a current runtime failure.

Requested change: Add: “These conflicts are checked within a linked device program, not across independent embedded programs in one native container.” The container may select separate program instances; actual device imports still require full resolution and linked certificate rebinding. Give a two-object example with colliding private names.

Acceptance: Add an independent-embedded-program example with colliding private names/layouts that remains packageable. An actual single-program link with unequal duplicate definitions or incompatible layouts still fails, and resolved imports retain signature/pin validation and certificate rebinding.

Cheap now: Yes: clarification/example; no relaxation of ISA linking.

Dependencies: F01

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F03 — Scope certificate and coverage inventory to selected device closures per root

Owner: `bennett-isa-spec`  
Type: `task`  
Priority: `P2`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **makes harder**

Source: [ISA §§12.1, 12.3][isa-context] [schema][isa-cert]; [§9.4][isa-frame]; [§13][isa-coverage]. See [AUDIT.md](AUDIT.md), row F03, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Closure-keyed regions, one root binding/schedule and root-relative contexts; frame records describe device interfaces/workspace. Native inventory is not explicitly delimited. This is a makes harder finding for the mixed case described in F03; it does not establish a current runtime failure.

Requested change: Insert the shared scope into §12.3 and cross-reference §§9.2/13: inventory all required device regions, owner/link evidence and coverage in the selected linked closure, with independent bindings and context `[]` per root. Native main, argv/envp preparation and host-only libc require no invented ISA region/frame/proof. Bind the container inventory to each inner certificate; never omit a reachable device callee.

Acceptance: A future cert publication explicitly excludes native-only main/startup/libc records while retaining every required reachable device region/frame/owner/link/coverage record. Include two selected roots sharing a static callee but having independent invocation bindings and context []; reject a missing device record or a substituted root binding.

Cheap now: Yes: wording and multi-root certificate design now.

Dependencies: F01

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F04 — Publish native-call adapters and ordinary-pointer launch binding

Owner: `bennett-isa-spec`  
Type: `task`  
Priority: `P1`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **locks out**

Source: [ISA §10 positional binding and C ABI][isa-launch]. See [AUDIT.md](AUDIT.md), row F04, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Launch starts a named device **CC** function. Argument kinds are INT/F64/QREG/TOKEN; there is no ordinary `ptr` setter/binding or general native call ABI. This is a locks out finding for the mixed case described in F04; it does not establish a current runtime failure.

Requested change: Publish a versioned host-entry-to-device-CC-entry adapter contract covering native scalar/aggregate arguments, ordinary pointers under F08, returns, faults/status, suspension and ownership/close. Explicitly distinguish native process entry from device entry. A pointer represented as integer bits alone does not register memory.

Acceptance: Specify native function ABI marshalling, device CC entry selection, scalar/aggregate and registered-pointer arguments, return/fault/status behavior, suspend/reverse/close and ownership. An ordinary host main can call and return from a marked function without becoming the CC entry; no claim treats int pointer bits as registered memory.

Cheap now: Yes for design; ABI/driver/runtime work is later.

Dependencies: F01, F08

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F05 — State device-invocation budget scope in mixed-launch and estimate records

Owner: `bennett-isa-spec`  
Type: `task`  
Priority: `P3`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **neutral**

Source: [ISA §5.4][isa-budget], [§4.7][isa-calls], [§10.1][isa-estimate]. See [AUDIT.md](AUDIT.md), row F05, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: A validated tuple and token belong to a device program/run; nested calls retain their own tuple. An action call has no invented primary oracle token. This is a neutral finding for the mixed case described in F05; it does not establish a current runtime failure.

Requested change: State: “Budgets and estimates bind selected device invocations; a native process has no implicit global B.” Include exact root/nested tuples and parameters in mixed-launch records. Account for device replay/wrappers under their existing units; report native/shim overhead separately. Keep token identity distinct from tuple equality and cache/claim binding.

Acceptance: Include a native main making two device calls with different tuples and a nested call retaining its own tuple. Records bind exact parameters/root/schedule, host work has no implicit B, replay/helper charges stay present, and token identity is not reconstructed from tuple equality.

Cheap now: Yes: scope and record design.

Dependencies: None beyond the standing bennett-4ht/shared-scope obligation.

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F06 — Specify program-ledger routing and lifetime for mixed-launch opaque ids

Owner: `bennett-isa-spec`  
Type: `task`  
Priority: `P2`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **makes harder**

Source: [ISA §3.3 ledger][isa-ledger], [§10 ownership][isa-launch]. See [AUDIT.md](AUDIT.md), row F06, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Register ids and tokens resolve in a loaded program; touched resources are exclusive until CLOSED. Separate loaded objects do not automatically share a ledger. This is a makes harder finding for the mixed case described in F06; it does not establish a current runtime failure.

Requested change: Define mixed-launch routing from an opaque host launch id to its owning program/ledger. Preserve identity across calls in that program. Cross-program use must have an explicit future ownership/transfer protocol or place the communicating device modules in one linked program; never reinterpret a bare id in another ledger or use a native pointer as the id. Specify stale/unload and wrong-owner diagnostics.

Acceptance: Specify repeated-entry identity, CLOSED ownership, stale/unloaded and wrong-program diagnostics without native dereference or pointer truncation. Show both a common linked-program construction for communicating modules and the exact future protocol needed for cross-program resource transfer; cross-program quantum sharing remains an explicit capability obligation.

Cheap now: Yes for namespace/lifetime choice; transfer mechanics later.

Dependencies: F01, F04

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F07 — Bind mixed-run effects and saved journals to their invocation/profile

Owner: `bennett-isa-spec`  
Type: `task`  
Priority: `P2`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **makes harder**

Source: [ISA §§7.3–7.4][isa-tape], [§10 tape API][isa-launch]. See [AUDIT.md](AUDIT.md), row F07, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Active R tape is device-effect order; CC host replies are retained in run frames. Saved journal interchange is explicitly a versioned tool profile, not implicit serialization of the native process. This is a makes harder finding for the mixed case described in F07; it does not establish a current runtime failure.

Requested change: Bind a mixed launch's journal to program/object/root, invocation, run and replay profile, with entropy/reply inputs and boundary ordering. Preserve separate R-effect, CC-reply and ordinary native-event ownership. Recovery must not issue external events again. Keep any saved-journal envelope outside the current bare active-tape API until versioned.

Acceptance: Define program/object/root/run/invocation and replay-profile associations plus entropy/reply and boundary ordering. Wrong-root or wrong-invocation saved journals fail before unrelated effects; R effects, CC replies and native host events keep separate owners, and recovery never reissues external events.

Cheap now: Yes: associations and replay design.

Dependencies: F01, F04

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F08 — Publish host-memory-by-reference mapping and reversible ownership contract

Owner: `bennett-isa-spec`  
Type: `task`  
Priority: `P1`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **locks out**

Source: [ISA §§7.1–7.2][isa-memory], [§10 copied buffers][isa-launch], [§0.3 ISA-11/G-12][isa-deferred]. See [AUDIT.md](AUDIT.md), row F08, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Device memory starts from layout images and bump-allocation metadata. Copying launch arguments and hostcall bytes provides no general live host-memory-by-reference binding. This is a locks out finding for the mixed case described in F08; it does not establish a current runtime failure.

Requested change: Publish memory registration/mapping: range and target layout, pointer/address mapping, liveness and allocation metadata, alias preservation, ownership/synchronisation, read/write sets, writeback/visibility, reverse eligibility and release. Include pointer-rich graphs and host changes between calls. Retain snapshot/replay as one construction; promotion/copying must not be the sole eventual realization of native sharing.

Acceptance: Cover pointer-free ranges and pointer-rich aliased/cyclic graphs, target layout/address mapping, liveness/bounds/allocator metadata, ownership/synchronisation, read/write/writeback and reverse/release. A native host mutation between launches reaches device reads without compulsory host promotion; host interference and invalid lifetime are diagnosed under explicit rules. Snapshot/replay remains supported, not the only eventual realization.

Cheap now: Yes to name the exact contract now; runtime implementation is substantial.

Dependencies: F01

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F09 — Delimit mixed-run reversal from native process rollback

Owner: `bennett-isa-spec`  
Type: `task`  
Priority: `P3`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **neutral**

Source: [ISA §10 states][isa-launch], [§14.3 T4][isa-roundtrip]. See [AUDIT.md](AUDIT.md), row F09, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Round trip restores the eligible **device run**, its active history and owned state; observed external events are not retracted, and measurement can make a run irreversible. This is a neutral finding for the mixed case described in F09; it does not establish a current runtime failure.

Requested change: Add mixed-launch wording: reverse/close is per run, not rollback of native main, host libc, or all sibling device runs. Define the F08 memory writeback checkpoint and host-interference invalidation without weakening current ownership or measurement rules.

Acceptance: Document eligible completed and failed device-run reversal, non-retraction of observed host events and per-run state ownership. Specify shared-memory commit/checkpoint/interference semantics without weakening measurement, unverified-free or touched-resource eligibility rules; no claim says native main or its libc is automatically reversed.

Cheap now: Yes: scope plus boundary protocol.

Dependencies: F08

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F10 — Clarify whole-program analysis versus selective device lowering

Owner: `bennett-dialect-spec`  
Type: `task`  
Priority: `P3`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **neutral**

Source: [0.2.12 §§4.1–4.5][d2-tiers], [§19.1][d2-driver]; [0.3 same clauses][d3-tiers] [§19.1][d3-driver]. See [AUDIT.md](AUDIT.md), row F10, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: One merged LTO module is the analysis input; H originals and CC/R/L clones coexist. Calls without bitcode trap **in device variants** (§4.5). This is a neutral finding for the mixed case described in F10; it does not establish a current runtime failure.

Requested change: Add: “Whole-program visibility computes device closures; it does not lower every input function. H-only functions and native-only dependencies outside those closures remain native and are not trapped merely for lacking device bitcode.” Preserve device taint, indirect candidate completeness, missing-plugin failure and polyvariance.

Acceptance: Amend the future dialect with a marked-module plus H-only no-bitcode library example. H definitions and that native import stay native; all device-reachable candidates still undergo analysis, normalization and missing-bitcode coverage, and missing plugin/marker failures remain loud.

Cheap now: Yes: wording and a native-only dependency fixture.

Dependencies: None beyond the standing bennett-4ht/shared-scope obligation.

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F11 — Exclude host-only globals from complete device-link inventory

Owner: `bennett-dialect-spec`  
Type: `task`  
Priority: `P1`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **locks out**

Source: [0.2.12 §19.5][d2-layout], especially `globals: every global of the link`; [0.3 §§4.3, 17.1–17.2][d3-tiers] [memory][d3-memory] [§19.5][d3-layout]. See [AUDIT.md](AUDIT.md), row F11, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Every link global can become device layout state; shared globals and address identity are stated as one memory. Read literally, unrelated native globals/TLS/libc state join the device inventory. This is a locks out finding for the mixed case described in F11; it does not establish a current runtime failure.

Requested change: In a future revision define “complete device link” and its globals as device-reachable state plus boundary-visible shared state and required metadata. H-only state stays outside ISA layout/certificate inventory. Preserve one semantic identity for truly shared globals; do not clone or omit them. Prove closure selection sound for loads, indirect calls, allocation-validity and aliases.

Acceptance: Define a sound device/global closure including indirect and allocation-metadata dependencies; native-only globals/TLS/libc are excluded from ISA layout and certificates. A shared mutable global retains one semantic identity across H/device variants, and omission of a device-reachable global is detected. Do not silently rewrite the published 0.2.12 compatibility unit.

Cheap now: Yes: scope definition now; closure/state analysis work later.

Dependencies: F08, F10

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F12 — Scope device main argv/envp and exit handling to explicit whole-device mode

Owner: `bennett-dialect-spec`  
Type: `task`  
Priority: `P3`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **neutral**

Source: [0.2.12 §19.2][d2-mode]; [0.3 §19.2][d3-mode], including argc/argv/envp, musl process context and exit handling. See [AUDIT.md](AUDIT.md), row F12, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: The **explicit whole-program reversible mode** makes main the device program and creates a CC argv stub; normal main return follows device exit/stdio cleanup. This is a neutral finding for the mixed case described in F12; it does not establish a current runtime failure.

Requested change: Prefix the rule with its mode scope. Ordinary mixed mode retains native process startup, argv/envp, environ, libc context and exit handlers. Device functions receive declared arguments; an optional device process-context snapshot must be explicitly requested and marshalled. Returning from a marked function must return to native main, without running process-exit cleanup or fetching argv again.

Acceptance: Document native mixed startup and ordinary marked-function return alongside unchanged whole-program R main startup/exit. Native argv/envp/libc need no ISA records; no implicit bennett.argv or exit-handler/stdio flush occurs for a scalar marked call. A requested device process-context snapshot has explicit marshalled lifetime/ownership.

Cheap now: Yes: exact mode discriminator and call/return design.

Dependencies: F04

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F13 — Preserve source-handle exclusion while naming opaque launch-id transport

Owner: `bennett-dialect-spec`  
Type: `task`  
Priority: `P3`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **neutral**

Source: [0.2.12 §§6.1, 6.4, 6.6][d2-handles] [boundary][d2-boundary]; [0.3 counterparts][d3-handles] [boundary][d3-boundary]; [North Star §3.1][north-handles]. See [AUDIT.md](AUDIT.md), row F13, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Source handle operations and measurement-dependent device work cannot survive as native host execution. Existing boundary text narrowly permits analysed transport of opaque launch ids. This is a neutral finding for the mixed case described in F13; it does not establish a current runtime failure.

Requested change: Preserve BD-E-HANDLE-ESCAPE/USE and the device-only qreg table. Clarify that launch-runtime opaque ids are not dereferenceable source pointers or permission to execute handle operations/measurement-derived control natively. Bind transport to F06, including target pointer width versus 64-bit ISA id and stale generations. Do not solve mixed execution by weakening escape checks.

Acceptance: Keep fail-loud prohibited source-handle operations/unknown-reader escape and device-resident table semantics. Specify the narrow analyzed opaque-id transport distinction, pointer-width tags versus full ISA ids, persistent ownership and stale generations; negative native-handle-use tests must still reject.

Cheap now: Yes: terminology and positive/negative boundary tests.

Dependencies: F06

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F14 — Replace mandatory pointer-sharing host promotion with an explicit future native-sharing route

Owner: `bennett-dialect-spec`  
Type: `task`  
Priority: `P1`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **locks out**

Source: [0.2.12 §19.4][d2-placement]; [0.3 §19.4][d3-placement], [§25.3 ISA-11][d3-efficiency]; [ISA §0.3][isa-deferred]. See [AUDIT.md](AUDIT.md), row F14, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Pointer-rich sharing **requires** H-to-CC promotion transitively to roots, potentially main. ISA-11 is described as only an efficiency/placement item. This is a locks out finding for the mixed case described in F14; it does not establish a current runtime failure.

Requested change: Retain promotion as a valid current construction; add the F08 versioned native-sharing route and make promotion a selected realization, not a permanent prerequisite. State explicitly that promotion alone does not preserve the promised native host for this case. Record a mixed-boundary publication obligation distinct from the retired 0.9 lowering blockers; do not turn “no blocker active” into “mixed ABI complete.”

Acceptance: Describe promotion as one current construction and the exact alternative native mapping contract as required follow-up. A mixed pointer-rich fixture must retain native main; promotion-only success is not evidence of this capability. Classify the mixed-boundary publication obligation separately from the device lowering blockers retired in draft 0.3.

Cheap now: Yes: correct the architectural classification now; implementation deferred.

Dependencies: F08, F11

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F15 — Define native host entry-state import alongside device snapshot replay

Owner: `bennett-dialect-spec`  
Type: `task`  
Priority: `P2`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **makes harder**

Source: [0.2.12 §8.5][d2-snapshot]; [0.3 §8.5][d3-snapshot], [§17.1][d3-memory]. See [AUDIT.md](AUDIT.md), row F15, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Snapshot/replay reproduces caller allocations/bytes at device bump addresses; metadata and shadow-stack history are semantic dependencies, not just byte loads. Native malloc graphs do not automatically have that history/address model. This is a makes harder finding for the mixed case described in F15; it does not establish a current runtime failure.

Requested change: Specify how F08 supplies equivalent bytes, pointer values/aliases, bounds/liveness, allocator and shadow-stack state before private allocations. Distinguish host-pointer registration from device-allocation replay; preserve private L entry-state semantics, snapshot invalidation and restoration. No blanket bit-copy of native pointers, silent relocation or mandatory promotion.

Acceptance: Specify how arbitrary native allocation graphs become the required private L entry state with preserved aliases/pointer observations, bytes, bounds/liveness and allocator/shadow-stack metadata. Include host changes between calls and metadata-only invalidation; preserve existing device replay order and budget charges without requiring native malloc to have the device bump history.

Cheap now: Yes: boundary algorithm/invariants now; implementation later.

Dependencies: F08, F11, F14

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F16 — Name fixed shared-address layout as a mixed profile and reserve mappings

Owner: `bennett-dialect-spec`  
Type: `task`  
Priority: `P2`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **makes harder**

Source: [0.2.12 §§4.4, 19.5][d2-tiers] [layout][d2-layout]; [0.3 §§4.4, 19.5][d3-tiers] [layout][d3-layout]. See [AUDIT.md](AUDIT.md), row F16, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Comparison-link addresses are common across tiers; fixed non-PIE placement also constrains production host functions/globals that cross the boundary. Independent device layouts are not automatically one address space. This is a makes harder finding for the mixed case described in F16; it does not establish a current runtime failure.

Requested change: Keep fixed shared-address layout as an admitted first profile; name it as such. In the mixed ABI publish container/root-to-layout association and mapping/relocation rules for other native layouts, including function-pointer dispatch and shared globals. Revalidate after device linking; do not change pointer comparisons/hashes or require native-only data/code in ISA memory.

Acceptance: Publish profile and root/layout identity for the existing fixed-address construction. State mapping/relocation obligations for other native layouts, including function pointers and shared globals, with no silent address change and no requirement to include native-only code/data in ISA memory; device-linked ranges remain validated.

Cheap now: Yes: profile and identities now; general relocation later.

Dependencies: F08, F11

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F17 — Scope libc and effect closure to device variants in mixed programs

Owner: `bennett-dialect-spec`  
Type: `task`  
Priority: `P3`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **neutral**

Source: [0.2.12 §§4.5, 11.1–11.3][d2-tiers] [effects][d2-effects]; [0.3 counterparts][d3-tiers] [effects][d3-effects]. See [AUDIT.md](AUDIT.md), row F17, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Device-reachable libc/libm/shims need bitcode and tier-specific effects; H calls its own libc. R has model effects; L is closed and virtualised. This is a neutral finding for the mixed case described in F17; it does not establish a current runtime failure.

Requested change: State that libc closure is per device closure/variant, including all reachable helpers; host-only libc needs no device bitcode/certificate. Preserve fail-loud trapped device calls, R tape and L model state. Give a mixed example where native printf remains host libc and a marked R printf uses the reversible console model.

Acceptance: Add a native printf plus marked R printf example: H uses host libc and R uses complete runtime bitcode plus console tape. Retain L closed device state, trapped missing device callees, coverage completeness and no real host effect from R/L; host-only libc has no ISA certificate inventory.

Cheap now: Yes: scope/example; no new hostcalls in R/L.

Dependencies: F10, F12

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F18 — Distinguish CC copied hostcalls from native shared-memory launch and callbacks

Owner: `bennett-dialect-spec`  
Type: `task`  
Priority: `P2`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **makes harder**

Source: [0.2.12 §11.4][d2-hostcalls]; [0.3 §11.4][d3-hostcalls]. See [AUDIT.md](AUDIT.md), row F18, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: CC-to-host calls use scalar and proved pointer-free copied pointees; pointer-rich arguments trigger promotion. This is an outbound service protocol, not a general inbound native-pointer launch binding. This is a makes harder finding for the mixed case described in F18; it does not establish a current runtime failure.

Requested change: Document direction and alias/lifetime/copy/writeback limits. Add F08 as the future native-sharing path without changing current copies. Publish callback/re-entry rules, declaring when an existing run owns shared state and retaining genuine effect/reply ordering. Never infer host memory registration from a hostcall request buffer.

Acceptance: Document direction, pointer-free proof, copy/alias limits and writeback of the current service block. Specify the future registered-memory and callback/re-entry ownership protocol, preserving effect/reply order and reverse eligibility; no consumer infers registration from a copied hostcall buffer.

Cheap now: Yes: protocol boundaries now; shared callbacks later.

Dependencies: F04, F08

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F19 — Scope budget resolution and frozen capacities to selected device links

Owner: `bennett-dialect-spec`  
Type: `task`  
Priority: `P2`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **makes harder**

Source: [0.2.12 §9][d2-budgets]; [0.3 §§9.1–9.5][d3-budgets], especially staged capacities and launch tokens. See [AUDIT.md](AUDIT.md), row F19, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Budget table/layout maxima cover shapes of a complete device link; tokens are minted per reference object/tuple, and replay/helper rows count. This is a makes harder finding for the mixed case described in F19; it does not establish a current runtime failure.

Requested change: Scope table resolution and frozen capacities to the selected device link/program, retain per-root selection/final identities and per-call resolved tuples in container launch records. Host-only heap/stack/libc work has no device budget. Do not reset counters at shim boundaries, round tuples, or share a token solely because two objects have equal numeric budgets.

Acceptance: Mixed launch records retain selected reference object/root, selection/final identity and exact static, host-supplied or device-minted tuple. Preserve per-shape frozen capacities and wrapper/replay/helper units; native heap/stack work is not device charged, and equal tuples in different programs do not merge token identity.

Cheap now: Yes: record/schema scoping.

Dependencies: F05, F21

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F20 — Separate materialised-closure comparison from native host integration

Owner: `bennett-dialect-spec`  
Type: `task`  
Priority: `P3`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **neutral**

Source: [0.2.12 §17.7][d2-comparison], [§19.7][d2-closed]; [0.3 counterparts][d3-comparison] [closed][d3-closed]. See [AUDIT.md](AUDIT.md), row F20, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Native comparison code-generates materialised **device variants**, with matching runtime/layout and classifiers. Closed-unit mode checks closure and falls back to whole-program analysis. This is a neutral finding for the mixed case described in F20; it does not establish a current runtime failure.

Requested change: State that the comparison subject is the selected materialised closure, not all of native main. Add a separate host integration comparison for mixed mode. Preserve original-input definedness evidence and device comparison support isolation. Closed-unit failure must fall back to analysis/linking, not force whole-binary lowering or reject the mixed host.

Acceptance: Publish two independently named comparison subjects and retain materialised module/runtime/layout/classifier identity for the device side. Closed-unit failure returns to whole-program analysis without forcing whole-host lowering. Original-input classifier coverage, device comparison support isolation and declared outcome fractions remain explicit.

Cheap now: Yes: scope and regression specification.

Dependencies: F10, F16

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F21 — Publish driver fat-object entry-to-payload and certificate adapter manifest

Owner: `bennett-driver`  
Type: `task`  
Priority: `P2`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **makes harder**

Source: [0.2.12 §19.9 outputs][d2-output]; [0.3 §19.9][d3-output]; [North Star §4.4 ownership][north-ownership]. See [AUDIT.md](AUDIT.md), row F21, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Outputs include native host module/stubs, ISA objects and sidecars, but no published container-to-entry-to-certificate inventory or full native call/return protocol. These are contract obligations, not a verified description of driver code. This is a makes harder finding for the mixed case described in F21; it does not establish a current runtime failure.

Requested change: Publish the driver producer/consumer edge under the ISA-owned mixed-launch/container version: entry and ABI symbol, ISA object/linked closure/root, certificate/build/layout and invocation associations, versions, runtime ownership and load/unload. Keep plugin-defined filenames behind the adapter; do not canonise rung directory names as the ISA container format.

Acceptance: Define the versioned producer/consumer edge and examples mapping native ABI symbols to selected program/object/root, layout, certificate/build/invocation and runtime load/unload ownership. Native-only symbols need no device records. Adapter filenames remain configurable; out/main and launches.json are not silently promoted into a normative ISA format. This task is contract-derived; driver source was not audited.

Cheap now: Yes: schema/design now; packaging/shim later.

Dependencies: F01, F03, F04, F06, F07, F16

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F22 — Design VM container selection over existing ISA source loading/linking

Owner: `bennett-vm`  
Type: `task`  
Priority: `P2`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **makes harder**

Source: [src/launch.rs:245–304][vm-load]; [src/link.rs:43–80, 158–178][vm-link]. See [AUDIT.md](AUDIT.md), row F22, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: C loader parses one UTF-8 ISA object; source linker merges compatible device metadata and rejects unresolved imports. There is no native executable/container selector in this interface. This is a makes harder finding for the mixed case described in F22; it does not establish a current runtime failure.

Requested change: Consume F01/F21 through a container adapter that extracts, verifies and selects linked ISA programs before calling the existing loader. Do not send ELF/native bytes to the ISA parser or merge all embedded objects merely because they share a host. Preserve exact import, conflict and certificate-composition checks.

Acceptance: Document container extraction and digest/version/root checks before the existing UTF-8 ISA loader. Independent payloads keep independent programs; actual imports use validated linked graphs and certificate composition. Native bytes or unrelated colliding layouts are never passed as a forced single ISA link.

Cheap now: Yes: interface split now; adapter later.

Dependencies: F01, F02, F21

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F23 — Add a versioned VM native-pointer and host-memory binding design

Owner: `bennett-vm`  
Type: `task`  
Priority: `P1`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **locks out**

Source: [src/launch.rs:94–104, 458–518][vm-args] [run][vm-run]; [src/cc.rs:238–277, 1842–1853][vm-data] [resources][vm-resources]. See [AUDIT.md](AUDIT.md), row F23, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: C launch matches INT/F64/QREG/TOKEN, not ordinary Ptr; resources own device memory initialized from layout. A numeric CLI ptr or `--mem` copy is not a borrowed live native allocation. This is a locks out finding for the mixed case described in F23; it does not establish a current runtime failure.

Requested change: After F04/F08 publish a versioned binding/backend for registered host memory and native argument/result marshalling; preserve allocation validation, aliasing, ownership, writeback and reversal. Retain existing by-value launches. Add bounded integration cases for host mutation between launches and a pointer-rich alias graph; no native pointer dereference inside the current integer-address memory backend.

Acceptance: Publish the binding/backend design before implementation and retain current by-value ABI. Design bounded host mutation, pointer-rich alias, liveness, interference and reverse/writeback integration vectors. Distinguish sparse device memory/CLI numeric ptr/global hex copies from registered live host storage; no implicit native dereference or unregistered pointer success.

Cheap now: Design is cheap now; backend and ABI are later.

Dependencies: F04, F08, F15, F16

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F24 — Route mixed VM launches and journals through program/run ownership

Owner: `bennett-vm`  
Type: `task`  
Priority: `P2`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **makes harder**

Source: [src/launch.rs:120–130, 179–240][vm-program] [locks][vm-locks]; [src/cc.rs:112–180, 1935–1952][vm-journal] [sync][vm-sync]. See [AUDIT.md](AUDIT.md), row F24, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Resources/locks/tokens are program-owned; journal/reply/entropy cursors are run-local. Replay input is a bare JSON array of effects, not proof of container/root identity. No built-in argv service is evidenced here. This is a makes harder finding for the mixed case described in F24; it does not establish a current runtime failure.

Requested change: Add mixed-adapter routing to exact program/root/run and profile-bound journals; retain separate run cursors and resource locks across repeated native calls. Dispatch argv/envp through declared host service or explicit context only in the selected entry profile. Test wrong program/id, wrong journal/root, overlapping ownership and unload; no process-global journal or implicit native-main bootstrap.

Acceptance: Specify explicit program/root/run routing, repeated calls, journal associations, allowed argv/context service and unload/close ownership. Include wrong owner/id, wrong root journal and overlapping-resource negatives with unchanged pre-error state; retain per-run entropy/reply/tape cursors and program locks rather than one process-global journal.

Cheap now: Yes: lifecycle design; runtime tests when adapter exists.

Dependencies: F06, F07, F12, F21

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F25 — Add plugin mixed-host publication obligations and real-stub acceptance

Owner: `bennett-plugin`  
Type: `task`  
Priority: `P2`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **makes harder**

Source: [docs/PLAN.md:14–27, 39–45][plugin-plan]; [docs/BUILD.md:29–40, 96–107][plugin-build]; [docs/PROGRESS.md:63–81][plugin-progress]. See [AUDIT.md](AUDIT.md), row F25, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Plan retains H originals and promises host stubs; scalar/development reference paths and reported objects are not a native-container execution witness. Historical fixture restrictions on entry memory are not source-language legality. This is a makes harder finding for the mixed case described in F25; it does not establish a current runtime failure.

Requested change: Add the shared-scope reference and mixed acceptance obligations to the M3/M4 publication boundary: retain ordinary H functions/native imports, emit launch bindings for marked roots only, report actual device closure inventory and distinguish reference-only/fixture outputs from production. Test the generated host path under the published ABI; do not claim completion from device CLI runs or freeze a fixture-only memory restriction.

Acceptance: Link the shared scope from PLAN/BUILD and distinguish historical development/reference-only limits from production legality. Specify a generated-host regression with a native-only library and marked roots, exact device inventories and real published launch stubs. Preserve native H originals and handle restrictions; direct device CLI and native mock runs cannot pass host integration.

Cheap now: Yes: design/docs and fixture specification now.

Dependencies: F10, F11, F12, F13, F21, F31

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F26 — Keep comparison-link libc restrictions local to the materialised comparator

Owner: `bennett-conformance`  
Type: `task`  
Priority: `P3`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **neutral**

Source: [rung-1 gate.py:264–307][r1-comparison]; [rung-3 gate.py:188–225][r3-comparison]; [rung-4 gate.py:523–540][r4-comparison]; [rung-4 DESIGN §§3/10][r4-design]. See [AUDIT.md](AUDIT.md), row F26, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Materialised closure comparisons forbid host libc and require fixed/pinned link inputs; rung 1's `_start` is a comparison harness. These constraints are correct for those comparisons. This is a neutral finding for the mixed case described in F26; it does not establish a current runtime failure.

Requested change: Scope these exact restrictions to the materialised comparison executable. Add a separate ordinary-native-host baseline and execution row for mixed mode, while keeping the existing closure comparison, classifiers and no-host-libc checks intact. Do not apply `-nostdlib`, no NEEDED, or the comparison input allowlist to the production native host.

Acceptance: Annotate rung-1/3/4 comparison helpers as device-closure comparators. Preserve existing pinned input, no-host-libc and fixed-address checks there, while the separate mixed host baseline/production executable may use ordinary host libc. A weaker comparator or host-libc-only baseline cannot satisfy the original rungs.

Cheap now: Yes: design wording; extra run when mixed ABI lands.

Dependencies: F20

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F27 — Require execution of generated native host in an additive mixed integration row

Owner: `bennett-conformance`  
Type: `task`  
Priority: `P2`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **makes harder**

Source: [rung-1 gate.py:248–263, 321–350][r1-launch] [adapter][r1-adapter]; [rung-2 gate.py:159–175, 197–225][r2-artefacts] [run][r2-run]; [rung-2 README][r2-readme]. See [AUDIT.md](AUDIT.md), row F27, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Rung 1 selects standalone oracle objects; rung 2 verifies host/device outputs then runs `launch.bennett` directly with run-program. Neither executes the generated native host/stub/container. This is a makes harder finding for the mixed case described in F27; it does not establish a current runtime failure.

Requested change: Preserve these acceptance domains and add the F30 host integration row: execute original native main around marked calls, prove host-only code remains native and bind the invoked embedded roots. Presence/LLVM verification of host.bc and native mock stubs cannot satisfy that row.

Acceptance: Retain rung-1 full-D and rung-2 hashtable/reversal acceptance. Add or share a 3M helper that actually runs native main and production generated stubs, verifies retained native-only work and selected payloads, and fails if only host.bc existence/verification or direct device CLI output is supplied.

Cheap now: Yes: test design now; no full-D rerun required for this audit.

Dependencies: F21, F25, F30

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F28 — Name rung-3 whole-device main argv and tape helper scope

Owner: `bennett-conformance`  
Type: `task`  
Priority: `P3`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **neutral**

Source: [rung-3 README required pipeline][r3-readme]; [gate.py:151–178, 238–278][r3-mode] [argv/run][r3-argv]. See [AUDIT.md](AUDIT.md), row F28, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Rung 3 intentionally forces reversible main, `out/main`, a bennett.argv reply, i32 status, no other host requests and a console-only tape. This is a neutral finding for the mixed case described in F28; it does not establish a current runtime failure.

Requested change: Keep this exact hello-world whole-device test. Label its helpers/profile accordingly; a mixed rung has native main plus named device entries and boundary-specific service expectations. Do not require main/argv or forbid ordinary native I/O in the mixed host; device effect/replay expectations remain exact per run.

Acceptance: Keep hello.c, reversible main, argv reply, i32 status and console tape checks unchanged. Document them as the whole-device entry profile; the future mixed helper selects named marked entries with its own boundary service expectations and does not require native main to be an ISA entry or put native I/O on the R tape.

Cheap now: Yes: helper/profile scope and additive design.

Dependencies: F12

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F29 — Limit rung-4 closure scans and shape rules to the actual SQLite device fixture

Owner: `bennett-conformance`  
Type: `task`  
Priority: `P2`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **makes harder**

Source: [rung-4 DESIGN §§1/3/4][r4-design]; [gate.py:486–520][r4-closure]; [gate.py:174–216][r4-checks]. See [AUDIT.md](AUDIT.md), row F29, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Undefined-symbol scan covers the merged runtime-linked fixture, per-entry coverage must have no trapped line, SQLite oracle needs ≥500 lprocs, and tuple launches need no host request. Its host image builder is a separate native program, not the generated mixed host. This is a makes harder finding for the mixed case described in F29; it does not establish a current runtime failure.

Requested change: Keep all these SQLite fixture checks. For mixed tests compute closure/coverage checks per selected device variant and distinguish legitimate native unresolved-at-bitcode-stage imports from unresolved device callees. Add a host-only native dependency as a positive control and a reachable missing device dependency as a negative control. Do not reuse SQLite's size/shape floor or no-host-request rule as universal container admission.

Acceptance: Retain SQLite source/shape/size/full-domain and no-trapped-device coverage requirements. Specify mixed helper closure checks using selected device inventories, with a positive H-only native dependency and a negative reached missing device helper. Native unresolved-at-bitcode-stage calls must not become device faults merely because they share a container.

Cheap now: Yes: fixture scope/test design; closure inventory plumbing later.

Dependencies: F10, F11, F17, F21

Filing: Ready for orchestrator routing in the owning repo; retain this exact scope and convert audit dependency keys to real Beads references.

## F30 — Add rung 3M for a real native binary with selective marked modules

Owner: `bennett-conformance`  
Type: `task`  
Priority: `P2`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **makes harder**

Source: [rung-4 gate.py:605–637][r4-run]; [gate.sh wrappers 1][r1-shell] [2][r2-shell] [3][r3-shell] [4][r4-shell]; [rungs 5–8 READMEs][r5] [6][r6] [7][r7] [8][r8]; [North Star §8][north-ladder]. See [AUDIT.md](AUDIT.md), row F30, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Rung 4 still invokes device CC entries directly; later READMEs specify whole-image/application demonstrations. No required row proves a native main survives around selective device modules. Gate shell wrappers only invoke their Python gates. This is a makes harder finding for the mixed case described in F30; it does not establish a current runtime failure.

Requested change: When rung 3 **or** 4 lands, file the additive mixed-binary rung described in [MIXED-BINARY-RUNG.md](MIXED-BINARY-RUNG.md). Keep every existing rung and its acceptance domain. Require a real native host with two marked roots, native-only dependency, exact associations, native/closure comparisons and per-run reversal/tape evidence. Track general host-memory and persistent-handle extensions explicitly; a scalar first fixture is not the completed mixed capability.

Acceptance: Adopt MIXED-BINARY-RUNG.md as the first acceptance proposal when rung 3 or 4 lands. Execute native main with a no-bitcode host library and A/B/A marked launches on both executors; verify full declared-domain results, payload/root/certificate associations, per-run budgets/reversal/replay and corruption controls. Keep current rungs unchanged and explicitly retain later pointer-rich memory, general layouts and persistent-resource obligations.

Cheap now: Yes: preserve slot/acceptance text now; filing trigger is deferred, implementation not on today's critical path.

Dependencies: F01, F03, F04, F06, F07, F10, F12, F17, F20, F21, F22, F24, F25, F26, F28, F29, F31

Filing: **Defer** until rung 3 or rung 4 lands; record that landing evidence. The implementation may remain scheduled after the mixed boundary contracts. No prerequisite is added to the current rung-3/4 acceptance.

## F31 — Preserve marked native-to-device entry boundaries across post-link optimisation

Owner: `bennett-dialect-spec`  
Type: `task`  
Priority: `P2`  
Labels: `fat-object`  
Parent context: `bennett-4ht`  
Audit severity: **makes harder**

Source: [0.2.12 §§4.2, 7.6, 19.1][d2-tiers] [outlining][d2-outlining] [pipeline][d2-driver]; [0.3 counterparts][d3-tiers] [outlining][d3-outlining] [pipeline][d3-driver]. See [AUDIT.md](AUDIT.md), row F31, and the [shared scope](../2026-11-02-rev2/fat-object-scope.md).

Determining sentence: Tier assignment sees functions after post-link optimisation; a function containing markers is CC. Region bodies are outlined, but a surrounding native-to-device entry wrapper has no stated mixed-boundary preservation rule. Inference: legal wrapper inlining could pull markers into main and expand the device partition. This is a design inference; no inliner runtime failure was reproduced.

Requested change: Specify how mixed mode preserves or reconstructs the native/device entry boundary across LTO: a supported standard LLVM noinline/build boundary or sound post-optimisation re-outlining, with declared ABI and identities. Keep the allowed LLVM pipeline and no-pre-link-plugin rule. Add a wrapper-small-enough-to-inline fixture proving main stays native; do not claim an observed compiler failure from this text-only audit.

Acceptance: Specify a supported standard-LLVM boundary-preservation construction or sound post-optimisation re-outlining, preserving the native ABI, marked root identities and declared pipeline. Include a wrapper small enough to inline under ordinary LTO and assert that native main and its native-only dependency stay H in the mixed build. Keep marker legality and no-pre-link-plugin rules; no dependence on an unverified source module boundary.

Cheap now: Yes: define boundary/pipeline contract now; compiler realization later.

Dependencies: F04, F10

Filing: Ready for orchestrator routing in the owning repo; this is a publication/design task, not a reproduced production bug.

[ruling]: ../../../../bennett/.tmp/orchestrator/advisor-notes.md#L58-L68
[north-architecture]: ../../../../bennett/docs/Bennett-NorthStar.md#L107-L130
[north-compilation]: ../../../../bennett/docs/Bennett-NorthStar.md#L276-L310
[north-handles]: ../../../../bennett/docs/Bennett-NorthStar.md#L144-L154
[north-ownership]: ../../../../bennett/docs/Bennett-NorthStar.md#L520-L545
[north-ladder]: ../../../../bennett/docs/Bennett-NorthStar.md#L795-L809
[isa-deferred]: ../../../spec/Bennett-ISA-Spec-0.9.md#L245-L273
[isa-ledger]: ../../../spec/Bennett-ISA-Spec-0.9.md#L450-L525
[isa-calls]: ../../../spec/Bennett-ISA-Spec-0.9.md#L1037-L1061
[isa-budget]: ../../../spec/Bennett-ISA-Spec-0.9.md#L1262-L1328
[isa-memory]: ../../../spec/Bennett-ISA-Spec-0.9.md#L1583-L1624
[isa-tape]: ../../../spec/Bennett-ISA-Spec-0.9.md#L1625-L1750
[isa-object]: ../../../spec/Bennett-ISA-Spec-0.9.md#L1912-L2067
[isa-link]: ../../../spec/Bennett-ISA-Spec-0.9.md#L2068-L2076
[isa-frame]: ../../../spec/Bennett-ISA-Spec-0.9.md#L2077-L2153
[isa-launch]: ../../../spec/Bennett-ISA-Spec-0.9.md#L2154-L2292
[isa-estimate]: ../../../spec/Bennett-ISA-Spec-0.9.md#L2293-L2408
[isa-support]: ../../../spec/Bennett-ISA-Spec-0.9.md#L2711-L2719
[isa-context]: ../../../spec/Bennett-ISA-Spec-0.9.md#L3611-L3632
[isa-cert]: ../../../spec/Bennett-ISA-Spec-0.9.md#L3899-L3993
[isa-coverage]: ../../../spec/Bennett-ISA-Spec-0.9.md#L3994-L3999
[isa-roundtrip]: ../../../spec/Bennett-ISA-Spec-0.9.md#L4034-L4060
[d2-tiers]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md#L862-L959
[d2-handles]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md#L1674-L1750
[d2-boundary]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md#L2026-L2039
[d2-snapshot]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md#L3229-L3299
[d2-outlining]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md#L2626-L2641
[d2-budgets]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md#L3929-L4175
[d2-effects]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md#L4593-L4654
[d2-hostcalls]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md#L4655-L4666
[d2-comparison]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md#L10207-L10227
[d2-driver]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md#L11167-L11185
[d2-mode]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md#L11186-L11203
[d2-placement]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md#L11215-L11223
[d2-layout]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md#L11224-L11251
[d2-closed]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md#L11259-L11271
[d2-output]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md#L11283-L11323
[d3-tiers]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md#L858-L955
[d3-handles]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md#L1699-L1775
[d3-boundary]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md#L2062-L2074
[d3-snapshot]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md#L3264-L3334
[d3-outlining]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md#L2658-L2673
[d3-budgets]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md#L3963-L4204
[d3-effects]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md#L4629-L4690
[d3-hostcalls]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md#L4691-L4702
[d3-memory]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md#L10026-L10062
[d3-comparison]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md#L10222-L10241
[d3-driver]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md#L11181-L11199
[d3-mode]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md#L11200-L11219
[d3-placement]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md#L11231-L11239
[d3-layout]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md#L11240-L11372
[d3-closed]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md#L11380-L11392
[d3-output]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md#L11404-L11449
[d3-blockers]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md#L11907-L11928
[d3-efficiency]: ../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md#L11945-L11961
[vm-load]: ../../../../bennett-vm/src/launch.rs#L245-L304
[vm-link]: ../../../../bennett-vm/src/link.rs#L43-L178
[vm-args]: ../../../../bennett-vm/src/launch.rs#L94-L104
[vm-run]: ../../../../bennett-vm/src/launch.rs#L458-L518
[vm-data]: ../../../../bennett-vm/src/cc.rs#L238-L277
[vm-resources]: ../../../../bennett-vm/src/cc.rs#L1842-L1884
[vm-program]: ../../../../bennett-vm/src/launch.rs#L120-L130
[vm-locks]: ../../../../bennett-vm/src/launch.rs#L179-L240
[vm-journal]: ../../../../bennett-vm/src/cc.rs#L112-L180
[vm-sync]: ../../../../bennett-vm/src/cc.rs#L1935-L1952
[plugin-plan]: ../../../../bennett-plugin/docs/PLAN.md#L14-L45
[plugin-build]: ../../../../bennett-plugin/docs/BUILD.md#L29-L107
[plugin-progress]: ../../../../bennett-plugin/docs/PROGRESS.md#L63-L81
[plugin-resume]: ../../../../bennett-plugin/docs/PROGRESS.md#L5-L36
[r1-comparison]: ../../../../bennett-conformance/ladder/rung-1/gate.py#L264-L307
[r1-launch]: ../../../../bennett-conformance/ladder/rung-1/gate.py#L248-L263
[r1-adapter]: ../../../../bennett-conformance/ladder/rung-1/gate.py#L321-L350
[r2-artefacts]: ../../../../bennett-conformance/ladder/rung-2/gate.py#L159-L175
[r2-run]: ../../../../bennett-conformance/ladder/rung-2/gate.py#L197-L225
[r2-readme]: ../../../../bennett-conformance/ladder/rung-2/README.md#L44-L51
[r3-readme]: ../../../../bennett-conformance/ladder/rung-3/README.md#L76-L104
[r3-comparison]: ../../../../bennett-conformance/ladder/rung-3/gate.py#L188-L225
[r3-mode]: ../../../../bennett-conformance/ladder/rung-3/gate.py#L151-L178
[r3-argv]: ../../../../bennett-conformance/ladder/rung-3/gate.py#L238-L278
[r4-design]: ../../../../bennett-conformance/ladder/rung-4/DESIGN.md#L1-L206
[r4-comparison]: ../../../../bennett-conformance/ladder/rung-4/gate.py#L523-L540
[r4-closure]: ../../../../bennett-conformance/ladder/rung-4/gate.py#L486-L520
[r4-checks]: ../../../../bennett-conformance/ladder/rung-4/gate.py#L174-L216
[r4-run]: ../../../../bennett-conformance/ladder/rung-4/gate.py#L605-L637
[r1-shell]: ../../../../bennett-conformance/ladder/rung-1/gate.sh#L1-L7
[r2-shell]: ../../../../bennett-conformance/ladder/rung-2/gate.sh#L1-L7
[r3-shell]: ../../../../bennett-conformance/ladder/rung-3/gate.sh#L1-L7
[r4-shell]: ../../../../bennett-conformance/ladder/rung-4/gate.sh#L1-L7
[r5]: ../../../../bennett-conformance/ladder/rung-5/README.md#L1-L5
[r6]: ../../../../bennett-conformance/ladder/rung-6/README.md#L1-L5
[r7]: ../../../../bennett-conformance/ladder/rung-7/README.md#L1-L5
[r8]: ../../../../bennett-conformance/ladder/rung-8/README.md#L1-L5

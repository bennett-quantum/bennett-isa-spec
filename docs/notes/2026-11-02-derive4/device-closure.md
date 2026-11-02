# F10/F17/F20 — device closure and two comparison subjects

Beads: **b-dialect-spec-k08**, **b-dialect-spec-dl8**, **b-dialect-spec-dh8**. One closure-scope amendment across analysis, libc and comparison. Amendment proposal, never a prerequisite for rungs 3/4; F30 remains deferred until rung 3 or 4 lands.

## Exact defect and clause citations

[Dialect 0.2.12](../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md) and [0.3](../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md) §§4.1–4.5/19.1 require whole-program visibility but retain H originals/device clones. §§11.1–11.3 define tier-specific effects; §§17.7/19.7 compare materialised variants and provide closed-unit fallback. These rules are compatible with mixed modules, but universal wording about missing bitcode/comparison can be misapplied to native-only dependencies. [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F10/F17/F20 are neutral scope findings, not observed compiler failures.

## North Star gap

No new machine capability gap. Preserve North Star §§3.2–3.3/4.2/5.1: whole-program visibility supports complete selected device closures; it does not require whole-native-binary lowering. Device callees cannot disappear behind a “host-only” label.

## Alternatives and witnesses

Native main calls a native-only `native_log` without bitcode and a marked R function that prints through the reversible console model. Only the R closure needs musl/shim device bitcode and tape proof. If the R function itself reaches a missing-bitcode callee, the existing trapped coverage rule still applies. Comparing all of native main to one materialised region confuses process behavior with device semantics. Prefer separate materialised-closure and host-integration comparisons. The finite closure/mode witness in [verify.py](verify.py) checks which dependencies enter each set; it does not compile a mixed host.

## Recommended exact dialect diff

Append to §§4.1/19.1:

```text
Whole-program visibility computes complete selected device closures; it
does not lower every input function. H-only functions/dependencies outside
those closures remain native and need no device bitcode or trapped line.
Every conservative device indirect/region candidate and reachable helper
still undergoes the existing analysis, normalization, coverage and missing-
bitcode rules. A missing plugin/marker implementation remains a loud
production link failure. Host-only is a reachability result, not an escape
from those rules for a reachable device callee.
```

Append to §§11.1–11.3:

```text
Libc/libm/runtime closure is per selected device variant, including all
reachable implementation bodies. Native-only printf uses host libc; R
printf uses complete device runtime bitcode and its console effect tape;
L printf uses the closed private model state required by §11.3. R/L never
gain genuine host calls from mixed packaging. Host-only libc requires no
ISA certificate inventory; shared-state identity uses §19.5 and the
negotiated memory/context profile.
```

Append to §§17.7/19.7/19.9:

```text
The materialised-closure comparison subject is the selected device variant
and its complete linked helper closure, target layout, runtime/arith,
normalizer/pipeline and input bindings. Keep original-input definedness,
layout-dependence/undefined classifications and outcome fractions explicit.
Comparison-only support remains isolated from production device inputs.

Mixed native-host integration is a separate comparison subject: the
generated native host/adapter/container calling selected device roots,
with separately declared host observations and per-device-run results,
faults, tape and reverse checks. Passing either subject alone does not
establish the other. Certificates do not inventory native main or host-only
libc as device regions. Closed-unit failure falls back to whole-program
analysis/linking with the same semantics; it does not force whole-host
lowering or reject an otherwise valid mixed source program.
```

Additionally preserve the entry boundary across post-link optimization: use the supported standard LLVM build/noinline boundary or sound post-optimization re-outlining, bind the resulting native adapter/device entry identities, and test a small wrapper eligible for inlining. [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F31 records this as an inferred preservation obligation, not an observed compiler bug or an extra prerequisite for this queue. Do not introduce a pre-link Bennett pass or restrict the permitted envelope pipeline to hide it.

## Hashes, catalog and consumers

Closure-comparison identity includes the actual materialised module/runtime/layout/input configuration; host-integration identity includes native artifact/adapter/outer inventory. They cannot replace each other. Scoping alone needs no ref catalog change; changing actual linked device bodies/layouts changes objects/proofs. Driver/plugin/conformance maintain separate evidence labels and published pins.

## Conformance vectors and verification

Native-only no-bitcode dependency positive; reachable missing device callee negative; same source function H and L variants with shared globals; native printf plus R console effect; L private output; small wrapper across LTO; missing plugin; closed-unit pass and fallback; original-input classifier coverage and comparison-only support isolation. [INDEX](INDEX.md) reports finite dependency checks only, not an executed compiler comparison or rung.

## K entry: 0.9 versus 0.10

Next versioned dialect revision plus 0.3 draft; no ISA semantic K required for these scope clarifications. Outer ABI follows the companion/0.10 route. Any in-version ISA 0.9 K route needs **council + maintainer authorization of compatibility rules**. Proposed dialect decision `SELECTED-DEVICE-CLOSURE`; all existing device traps/coverage remain exact.

## Mixed fat object

Apply [fat-object-scope.md](../2026-11-02-rev2/fat-object-scope.md) and [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F10/F17/F20/F31. Native main/argv/envp/libc stay native; only marked modules and complete required device closures enter objects/certificates, with separate root context `[]`, budgets, ledgers and tapes. Explicit adapter/memory mappings remain required. These amendments never gate rungs 3/4; F30 stays deferred.

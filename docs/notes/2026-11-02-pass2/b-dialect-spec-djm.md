# djm — process preparation has a state contract; the helper is optional ABI work

Council disposition/proposal for `b-dialect-spec-djm` (dialect repository).

## Defect and North Star scope

Both maintained dialect texts §19.2 already require a static single-thread
musl process context with supplied argc/argv/envp, null-terminated vectors,
environ, program name, initial fenv and required libc/stdio state. §19.1
links runtime bitcode before materialisation. §14.7/§19.2 already define
return-from-main as exit, handlers/flush exactly once and _Exit bypass.
Thus the absence of the name `__bennett_rt_prepare_process` is **not a
missing process capability or an established semantic defect**. It is a
low-priority cross-implementation convenience ABI question (C5/C8).

Origin read: runtimes `.tmp/sol-runtimes-queue/cross-repo-beads.md`,
“static process preparation”, `src/musl/context.c` and `src/native/start.c`.
The runtime provides and uses that ordinary helper; the dialect does not
currently mandate it. No private musl layout is imported into this proposal.

North Star §§3.2/4.2/5 and §8 rungs 3,5,6 requires full program startup,
devices and reversible execution. Keeping the state requirement and an
ordinary linked preparation route preserves it. Requiring Linux TLS/auxv,
omitting environment/program state or bypassing normal exit would narrow it.

## Alternatives and worked witness

With argc=1, argv=["/bin/demo",null], envp=["K=V",null], required public
state includes environ pointing at the supplied envp, full program name
"/bin/demo", short name "demo", initial errno/fenv, and a usable main
thread/C locale. Leaving a static empty environment yields getenv("K")
absent instead of "V". Merely passing argv to main is insufficient.

For argc=0 with nonnull argv=[null], the helper uses an empty program
name. For argv[0]="/bin/", basename is empty; preserving a stale previous
name is wrong. These are hand-worked byte/state cases, not a production
process run in this job. Preparation must not reset a descriptor-bound
device model or erase its effect tape to obtain a fresh libc state.

| Alternative | Assessment |
|---|---|
| Keep only the existing public-state requirement | Semantically sufficient; package-specific adapters remain ordinary linked code. |
| Standardise an optional helper with exact preconditions | Useful independent-driver interface without exposing private layout. Recommended if an ABI is desired. |
| Require drivers to write private libc structures | Couples implementations through unpublished layouts; reject. |
| Require this symbol unconditionally from every runtime | New compatibility restriction; not justified by the current gap. |

## Recommended exact spec diff

No repair to the required §19.2 state semantics is necessary. If council
wants the convenience ABI, append the following in **both** maps:

```text
An implementation may provide the ordinary linked bitcode convenience
entry void __bennett_rt_prepare_process(i32 argc, ptr argv, ptr envp),
with the target's C calling convention and pointer width. A runtime
advertising this entry must provide that exact signature. A driver may
instead establish the same required process state through its pinned
runtime's documented preparation path; the state obligation is unchanged.

The entry is called for a fresh declared process instance after its
static images and supplied vector/string allocations are installed.
argc is nonnegative. argv and envp are nonnull even when empty; argv
contains argc nonnull NUL-terminated strings followed by null, envp a
null-terminated vector of NUL-terminated strings, all in live comparison-
registered/device allocations. Vector/string reads use ordinary
materialised memory checks. Invalid state is a named materialised fault,
not a host crash or permission to read arbitrary host process state.

On success it installs the supplied environment, full program name
(argv[0], or empty for argc=0), short name (suffix after the last slash),
and the initial errno, fenv, main-thread/C-locale and libc state required
by the linked program. Private state layout belongs to that runtime.
It does not reset device-model state, effect/entropy tapes, allocation
history or live quantum registers, nor run main or flush/exit handlers.
The driver roots any selected helper and its static-image dependencies
before materialisation; both builds execute the same bound preparation.
Preparation inside an R trace has ordinary charged, reversible stores;
preparation supplied as an entry image is recorded in that entry-state
binding. It is not an unrecorded reset between region calls.
```

This optional entry does not promise arbitrary reinitialisation of an
already running process, idempotence after program effects, or an implicit
reset on every launch. Those would require separate lifecycle semantics.
Keep the existing main/exit/_Exit and failed-run behavior unchanged.

## Hashes, catalog and consumers

No ISA catalog or instruction change is required. Selecting different
ordinary preparation bodies/static images changes materialised closure
and manifest identities, including native comparison pins; startup must
not be linked only into the native path. A dialect ABI clarification
obeys its own version policy, not an ISA K numbering scheme. Driver,
runtime and plugin must include the chosen helper before outlining and
lower its stores through the published dialect semantics.

## Conformance vectors and verification

Test K=V, empty environment, argc=0, full/short names including trailing
slash and empty argv[0], initial errno/fenv, C-locale/main-thread state,
live-bound checks, normal return/exit exactly-once handlers/flush, _Exit
bypass, failed-run no flush, and preservation of descriptor model/tape.
Compare both builds from the same registered entry state and reverse the
R startup prefix exactly. The simple name/environment byte examples are
checked in [INDEX](INDEX.md). No process gate or whole-program ISA run
is claimed; originating runtime tests remain reported evidence only.

## Release and route

**No semantic defect and no ISA 0.9 K/0.10 blocker.** Optional current
dialect helper-ABI clarification is enough if council wants a standard
entry. Otherwise disposition djm with §19.2's existing state contract and
retain driver startup/re-pin implementation work with its existing owners.

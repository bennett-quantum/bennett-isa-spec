# brt-vbi — delivery of arbitrary-width runtime helper bodies

Council proposal, not a specification or runtime implementation. Origin:
`brt-vbi` (blocked decision), blocking `brt-zct` in bennett-runtimes.
References: [dialect 0.2.12](../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec.md)
§§12.1, 12.7–12.8, 13.1, 19.1, and the corresponding
[0.3 draft](../../../../bennett-dialect-spec/spec/Bennett-Dialect-Spec-0.3-draft.md).

## Exact defect and North Star scope

§12.7 names width-specific ordinary runtime bodies for wide multiply,
divide, remainder and shifts. §12.8 similarly names integer intrinsic
helpers, including slot-based wide forms. §19.1 requires matching-major
runtime bitcode before materialisation. §13.1 publishes an offline
instance-request protocol for arith, but none of these clauses publishes
a runtime-integer request/response contract or a collection procedure for
helpers introduced during normalisation. A consumer cannot depend on
the runtime producer's private generator interface.

This is a **dialect delivery/interoperability gap**, alongside an actual
missing-implementation obligation. It is not a new ISA arithmetic
instruction or launch ABI gap. Naming an ordinary helper already makes
its body required; a finite default library failing to contain i257
does not make that LLVM width unsupported. The gap matters to North Star
§§3.2–3.3, 5.1's whole materialised LLVM and reproducible comparison.
It does not justify narrowing widths, trapping valid arithmetic, omitting
libc/Julia/SQLite closure, or declaring the broader brt-zct work complete.

## Alternatives and concrete witness

Read-only source witness:
`../bennett-runtimes/.tmp/sol-runtimes-queue/tmp/runtime-width-request.ll`.
It loads two i257 values, multiplies, calls `llvm.ctpop.i257`, stores i257.
Both LLVM 18 and 23 `opt -passes=verify -disable-output` accept it.
Both live `dist/llvm<M>/bennett-runtimes.bc` lack definitions for
`__bennett_rt_i257_mul` and `__bennett_rt_i257_ctpop`, confirmed with
matching-major `llvm-nm --defined-only`. These are current observations,
not an inference from the producer's progress report.

For a concrete numerical vector, a=2^256+3, b=2. Modulo 2^257 the product
is 6 and its population count is 2. A five-limb input slot has widths
[64,64,64,64,1]; ctpop writes [2,0,0,0,0]. Dropping the top input limb
can be caught separately by ctpop(a)=3. This requires actual ordinary
bodies, not a declaration that merely passes LLVM verification.

| Alternative | Assessment |
|---|---|
| Publish an offline request/manifest protocol in the dialect | Producer-independent, analogous to arith; recommended. |
| Prebuild every admitted width and operation | A possible complete distribution, but costly; a common-width subset is only a cache. |
| Publish a separate runtime ABI note | Viable only if versioned and normatively incorporated by the dialect; a producer README alone cannot bind competitors. |
| Add ISA primitives or a private driver-generator API | Unnecessary machine extension or unpublished coupling. |

## Recommended exact dialect/ABI text

Insert the following titled subsection at the end of §12.8 before the
examples, and cross-reference it from §19.1. Mirror it in both tracks.
The executable name below is a proposed public interface, not a claim
that today's runtimes supplies it.

```text
Runtime integer instance delivery v1.

The runtimes distribution MUST supply an offline executable interface:
  bennett-runtimes-instantiate --request REQUEST --bitcode OUTPUT_BC
                             --manifest OUTPUT_JSON
The driver may configure the executable path. Exit zero means both
outputs are complete; nonzero means no output is usable. Generation
must not require network access, a host runtime service or private
consumer knowledge. The distribution MUST identify its version.

REQUEST is UTF-8 JSON with exactly schema, llvm_major, target_triple,
data_layout and entries. schema is "bennett-runtimes-instantiations-1";
llvm_major is the link's integer LLVM major; target_triple and
data_layout are the link's effective LLVM strings. Each entries element
has exactly op (string) and width (integer, not boolean).

The op set is mul, udiv, sdiv, urem, srem, shl, lshr, ashr from §12.7;
and bswap, bitreverse, ctpop, ctlz, cttz, fshl, fshr, sshl_sat,
ushl_sat, smul_fix, umul_fix, smul_fix_sat, umul_fix_sat, sdiv_fix,
udiv_fix, sdiv_fix_sat, udiv_fix_sat, pdep, pext, clmul,
smul_with_overflow and umul_with_overflow from §12.8. Dots in an
intrinsic name become underscores. width is the original integer
operand width W, not a limb width or result aggregate size. The
operation and width must be admitted by the link's LLVM major and the
corresponding published row; §12.7 requests require W>128, and the
overflow-helper requests require W>64. Unknown fields/operations,
nonintegral/out-of-range widths and target/major/layout mismatches are
errors. Every valid request remains an implementation obligation.

Duplicates denote one instance. Canonical request bytes are UTF-8 JCS,
without a trailing LF, with unique entries sorted by op's unsigned
UTF-8 bytes then ascending numeric width. request_sha256 is the
lowercase hexadecimal SHA-256 of those bytes. A supplied width list
or prebuilt cache MUST NOT become a capability limit.

OUTPUT_BC is ordinary matching-major bitcode whose triple and
DataLayout equal the request. For each requested (op,W), provide the
exact public definition, type, scalar flags, result convention and
fault behavior of §§12.7–12.8. In particular, wide mul is
void @__bennett_rt_i<W>_mul(ptr dst,ptr a,ptr b), and wide ctpop is
void @__bennett_rt_i<W>_ctpop(ptr dst,ptr a). The existing shadow-stack
slot layout, partial top limb and push/store/call/load/pop order are
unchanged. All reachable implementation bodies must be supplied or
resolved through the published matching-major base runtime/arith
distributions and their published instance protocols. No unresolved
instance placeholder, trap stub or hidden host implementation counts
as delivery.

OUTPUT_JSON is JCS with exactly schema, request_sha256,
bitcode_sha256 and exports. schema is "bennett-runtimes-instances-1";
bitcode_sha256 hashes the exact OUTPUT_BC bytes. exports lists every
externally visible definition in that module, sorted by unsigned UTF-8
symbol bytes, each with exactly symbol and function_type; function_type
is LLVM's printed function type for the requested major, without the
function name or attributes. Every requested public symbol must occur
exactly once and have its published type. Generation with identical
canonical request and producer version must reproduce both output
byte strings regardless of output paths. The driver verifies hashes,
triple, layout, definitions and types before using the output.

Collection is a link-preparation fixed point. Starting from the selected
device entries and their conservatively reachable bitcode closure,
include calls already naming these helpers and all helpers that the
published normalisation rules will introduce, including vector
scalarisation, N27 and the N22 applications after N27/N32. Collect
arith instances by §13.1 as well. Resolve runtime/arith requests and
their dependencies until no new (provider,instance) is required and
every required body is linked. Include all possible device call
targets; do not select just one runtime input path. Declarations alone
do not satisfy the fixed point. A provider cycle must resolve to actual
linked definitions; repeated unresolved requests are a loud failure.

This discovery is a non-emitting analysis using the published rules;
it does not rerun or change §12.1's normative rule order. Link the
complete collected bodies before final materialisation, identically
in the lifted and native comparison inputs. If later optimisation or
materialisation reveals a new required helper, restart preparation
with the enlarged request and rebuild both outputs; never patch only
one build or silently leave it unresolved. Failure to generate a
valid instance is an implementation blocker, not an unsupported-width
verdict, narrower substitute or permitted fault of the device program.
```

Add to §19.1 after the bitcode-source paragraph:

```text
Before final materialisation the driver MUST complete the runtime
integer instance-delivery fixed point of §12.8 and the arith requests
of §13.1, using the same resolved closure in both builds. Whole-program
device mode uses its entire device closure; selective-module mode
collects the union required by its marked device roots. Neither mode
requires host-only code to become a device region.
```

## Hashes, catalog and consumers

Request and output hashes are different identities; neither is an ISA
`object_hash` or the arith/ref catalog hash. Provider-version changes
invalidate the generation cache even when the request is identical.
Generated bitcode changes materialised closures, costs, stack use and
the resulting object/schedule/certificate bindings. Drivers, plugins,+runtimes, numerical providers and conformance owners must coordinate
publication and repin. No ISA `ref@0.9` recipe change is necessary.

## Conformance vectors and actual verification

The bounded command in [INDEX](INDEX.md) ran both major-specific LLVM
verifiers and symbol audits: exit 0, both missing-definition assertions
passed. This establishes the delivery witness, not helper implementation
or whole-program execution. No sibling gate was run.

After publication require: i129/i192/i257 partial-top-limb values;
mul and ctpop together; large admitted widths with a resource-bounded
generation test; narrow intrinsic helpers; scalar flags and overflow
aggregate types; all helper families with published fault behavior.
Check reordered/duplicate requests canonicalise identically, unknown
op/invalid width/major/layout reject loudly, manifests cannot omit or
mis-type a requested definition, and changed request/bitcode hashes fail.
Test a helper introduced only by scalarisation, one after N32/repeated
N22, a transitive dependency and a discovery restart. Verify byte
reproduction in relocated directories and link/execute both materialised
builds. Merely passing opt verification is insufficient.

## K entry and publication route

**No ISA 0.9/0.10 K entry:** this is a dialect/tool-distribution contract.
Proposed dialect decision row: `RUNTIME-INSTANCES-1 | §§12.8,19.1 |
offline request/manifest and complete device-closure fixed point |
brt-vbi | additive versioned delivery protocol`. Recommend the next
published 0.2 patch and the corresponding 0.3 draft text. A separate
ABI note is acceptable only with a normative versioned dialect reference.
Leave brt-vbi/brt-zct open for ratification and implementation; publication
of a proposal does not unblock a consumer's unpublished interface.

## Mixed fat object

[fat-object-scope.md](../2026-11-02-rev2/fat-object-scope.md) binds this
proposal (North Star §§2, 3.3, 4.4). Collect helpers for marked modules and
their linked device closures. Multiple roots may share generated bodies;
each device root retains its own certificate/context association. The
ordinary native host binary, host entry/argv/envp and host-only libc stay
outside that inventory. Native comparison binds the matching materialised
device closure. Request manifests are build inputs, not a new launch,
budget, handle or effect-tape protocol. Container/root association and
host-memory mapping still need the exact mixed-launch additions named in
the shared scope note; this delivery decision keeps them reachable.

# tea/99n — make source-to-recipe widths total

Beads: **b-isa-spec-tea** (EST-SOURCE-03), **b-isa-spec-99n** (EST-SOURCE-04). One source-binding/catalog amendment with two independent corrections. Amendment proposal, never a prerequisite for rungs 3/4; F30 remains deferred until rung 3 or 4 lands.

## Exact defect and clause citations

[ISA 0.9](../../../spec/Bennett-ISA-Spec-0.9.md) §3.6 sizes history slots from listed write sets; §3.7/K25 requires `lift.ret` to scan every Globals record. The published [ref catalog](../../../spec/ref-0.9.json)'s lift.ret R4 scans `width` bits of each location through `delivery:wire[N]`. An unused wider global register can exceed N even when its selection control is false: recipe wire typing still sees the out-of-range index.

§3.3's Lifted width rules allow a qjumpr address with any width A satisfying 2^A>n. §3.6 uses one P for the complete pc/successor. The catalog's qjumpr binds both `a` and `succ` to P, so two legal source address widths cannot both instantiate it. The source domain, not a producer's assumed common operand width, is binding.

## North Star gap

These are total-expansion defects for already admitted programs (North Star §§3.2/5.1), not missing source capabilities. Do not remove unused wide locations, force equal jump widths or truncate high address bits to hide them. The exact expansion and all resource charges must be publishable for every legal source instance.

## Alternatives and witnesses

The read-only [return fixture](../../../../bennett-estimate/tests/fixtures/ret-delivery-source-gap.bisa) has result/global r width 1, unused ghost width 8, S_cells=3 and depth limit 1. Sc=4 and Dc=2; call slots `(4,4,2)` and return slots `(1,4,2)` give W=`(4,4,2)`. With n=4 and P=3 the history width is `3+4+4+2+1+3=17`. R4 still names delivery[4..7]. Widening W1 to 8 gives 21 bits and makes every controlled scan well typed. The job-built reference actually returned `{payload:1,code:0,steps:4}` at tuple `(4,1,8,1)`.

The read-only [jump fixture](../../../../bennett-estimate/tests/fixtures/jump-width-source-gap.bisa) has n=3 and address widths 3 and 4, both legal. No common P equals both widths. The reference returned `{payload:1,code:0,steps:3}` at input `(1,2,1)`, tuple `(3,0,0,0)`. These are direct semantic executions, not a successful recipe expansion or estimator run.

| Alternative | Assessment |
|---|---|
| Bound each return scan by min(width,N) | Can avoid the invalid index but changes the published scan/ordinals; needs its own complete delivery/zero-extension argument. |
| Widen first delivery slot to complete W_max | Smallest normative frame-binding change retaining the exact R4 scan. Recommended. |
| Resize source qjumpr operands to P | Changes the legal source representation; truncation can turn out-of-range high bits into a valid jump. |
| Separate address width A and pc width P | Preserves source operands, full-width interval checks and one pc. Recommended. |

## Recommended exact spec/catalog diff

Append to §3.6 Frame layout:

```text
If the instantiated lproc closure contains lift.ret, W_1 is additionally
at least W_max (§3.7), including locations absent from row write sets.
The ref lift.ret formal N is this complete W_1-bit disp_1 slot. Every
history frame retains that width. Other slots remain the maxima of their
listed captures/assignments. R4's complete Globals delivery scan is not
pruned by the chosen destination or a basis-dependent control.
```

Add to §11.1 source binding, and update lift.ret's catalog `source` annotation to cite the new W1 rule (its gate body is unchanged):

```text
Bind lift.ret N=W_1 and V to its result width. Bind the complete Globals
and Locations lists under K25, without dropping unused or wider locations.
```

For qjumpr, apply this exact transformation to its recipe only:

```text
parameters: after integer P insert integer A, integer CopyWidth.
a.width: P -> A. succ.width stays P.
body: replace each scalar occurrence of "P" by "A", EXCEPT the bound
      of the sole a[bit]-to-succ[bit] XOR loop, which becomes CopyWidth.
```

Add its binding rule:

```text
P is the containing oracle's pc/successor width; A is the source address
operand's declared width and satisfies the existing 2^A>n rule. Bind
CopyWidth=min(P,A), First to this function instance's first canonical
row, End=First+its row count. Validate these values rather than accepting
arbitrary producer integers. First/End comparisons and constants use A
bits. XOR only CopyWidth low address bits into the clean P-bit succ;
when P>A its higher bits remain zero. Preserve the existing flag, cause
and flagged successor semantics; an invalid full-A-bit address faults
even when its low P bits denote a valid row. Do not resize source wires.
```

The source check 2^A>n makes End representable. On valid targets a<n<2^P, so dropping bits above P in the copy loses no valid address information; the separate full-width predicate protects invalid values. [verify.py](verify.py) constructs the transformed recipe without editing the publication and executes its actual gate DSL for bounded P/A/interval cases, including inverse cleanup of scratch.

## Hashes, catalog and consumers

These corrections change exact expansion widths/costs and qjumpr catalog bytes. Publish a new catalog/version/envelope hash together with §3.6; never keep the old `ref@0.9` hash for changed bytes. Same source bytes under the same ISA prefix retain their source identities only under an explicitly authorized in-version repair; semantic costs/schedules/certificates still require regeneration. A 0.10 prefix changes object/interface/closure envelopes as usual. Estimator, scheduler, VM, circuits, checker and dialect references must pin the matching catalog and binding rule. B=0 generates no step-row expansion but does not waive source validation.

## Conformance vectors and actual verification

Require unused widest globals, result at W_max, multiple lproc bindings/recursion, different destination widths, faulted depth-zero return, and forward/inverse R4 scans. Test P<A, P=A, P>A, multiple jump widths in one object, nonzero First, End boundaries, invalid high bits, all flag/fault writes, scratch restoration and B=0. Actual reference fixture results above and the finite recipe checks are reproduced by the bounded command in [INDEX](INDEX.md). This is not a full catalog conformance gate or evidence that consumers already implement the amendment.

## K entry: 0.9 versus 0.10

Recommend **0.10 source-binding/catalog amendment**, proposed decisions `RET-DELIVERY-WIDTH` and `QJUMPR-ADDRESS-WIDTH`. An in-version K route needs **council + maintainer authorization of compatibility rules**, explicitly covering changed ref identity, frame/cost discontinuity and old certificates. No private catalog patch or source restriction is authorized by these notes.

## Mixed fat object

Apply [fat-object-scope.md](../2026-11-02-rev2/fat-object-scope.md) and [AUDIT.md](../2026-11-02-4ht-audit/AUDIT.md) F03/F05/F15. W_max and row indices belong to the selected device oracle/lproc closure, not native main or all host globals. Bind regenerated costs/catalog/certificates separately per root/context `[]`; retain native startup/libc and the future registered-memory adapter. These amendments never gate rungs 3/4; F30 remains deferred.

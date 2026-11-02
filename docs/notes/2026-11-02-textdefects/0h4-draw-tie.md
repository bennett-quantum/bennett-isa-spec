# 0h4 — measurement draws, exact ties and the existing K17 rule

Council derivation for `b-isa-spec-0h4`, related `b-isa-spec-bas`.
Proposal and reproduced evidence only; no spec, catalog or interpreter edit.

## Finding: the checked-in 0.9 already decides this

The advisor's 2026-11-02T07:55Z witness is reproducible: H on a fresh wire,
then `--draws 0.5`, returns **1** in the reference interpreter. The supplied
report of the pre-fix VM returning 0 is historical evidence; this job did
not run either VM revision or reproduce the reported 24/32 harness tally.

The claim that **neither** published text states the tie rule is stale for
this checkout. [0.8 §4.1](../../../spec/Bennett-ISA-Spec.md#41-classical-control-cc-r)
only names the Born rule. [0.9 §4.1](../../../spec/Bennett-ISA-Spec-0.9.md#41-classical-control-cc-r)
already says “strictly greater”, explicitly sends a CDF tie to the next
nonzero-weight outcome, specifies FIFO including deterministic measurements,
and includes two exact ties. Its §0.2 K17 and “Decided confirmations” record
ratification; `bd show b-isa-spec-bas` also reports the applied rule. This
is evidence against reintroducing an absent-text defect in current 0.9,
not a claim that the VM has always implemented K17. A consumer pinned to
another snapshot should compare the actual §4.1 bytes before diagnosis.

Recommendation: retain K17, optionally apply the editorial strengthening
below, add the single-H boundary vectors to the harness, and disposition
0h4 after council reviews this evidence. Keep the issue open here: notes
alone do not accept a council change. Do not reopen accepted semantics or
change the catalog to compensate for a consumer comparator.

## Derivation from the reference

Read-only implementation evidence:

| Source | Determining behavior |
|---|---|
| [quantum.rs](../../../reference-interpreter/src/quantum.rs), `measurement_branches` | Projects jointly on the requested wire list. `BTreeMap<Vec<bool>,...>` orders bits lexicographically, `false < true`. Sums squared exact amplitudes within each projection; retains the conditional state of other wires. |
| [registers.rs](../../../reference-interpreter/src/registers.rs), `measure` | Sums branch weights into T, forms `-u*T`, and selects only when the exact sign of cumulative weight minus uT is `Greater`. Consumes the register after selection. |
| [exact/measurement.rs](../../../reference-interpreter/src/exact/measurement.rs), `real_sign` | Decides exact zero before refining outward enclosures; equality is not resolved with a tolerance or binary64 display probability. |
| [classical.rs](../../../reference-interpreter/src/classical.rs), `measurement_draw`, `transaction`, measure row | Pops the front of the queue; otherwise advances its measurement PRNG. Failed transactional steps restore queue and PRNG as well as state. Every successful measurement obtains a draw, even for a basis state. |
| [main.rs](../../../reference-interpreter/src/main.rs), options and `measurements_json` | Parses each `--draws` item as binary64, validates through `push_measurement_draw`, and prints wire-first bits and the little-endian integer. |
| [SPEC-QUESTIONS 29](../../../reference-interpreter/SPEC-QUESTIONS.md), [README](../../../reference-interpreter/README.md) | Documents this CLI convention for the interpreter's **0.8** execution surface. It is not evidence that this binary admits 0.9 objects. |

Let the measured register's wires be b0,...,b(n−1) in register order. For
each outcome s, let P_s project on those bits while leaving every other
live wire alone, and let w_s = ||P_s ψ||². Enumerate all bit strings s in
lexicographic order with b0 printed first, 0 before 1. Set T = Σ_s w_s > 0,
C_j = Σ_(i≤j) w_(s_i), C_−1 = 0. For a supplied real draw 0 ≤ u < 1:

```text
selected index = min { j : C_j > u*T }
selected outcome s_j iff C_(j-1)/T <= u < C_j/T
returned value = sum_(i=0..n-1) s_j[i] * 2^i
conditional residual state = P_(s_j) ψ / sqrt(w_(s_j)), with measured wires removed
```

Zero-weight outcomes have empty intervals and are never selected. At any
equality `u*T=C_j`, skip that outcome and any following zeros, selecting
the next positive weight. At u=0 choose the first supported outcome; u=1
is outside the domain. For n=2 the strings `00,01,10,11` have integer
values **0,2,1,3**, not 0,1,2,3. Sorting by the returned integer gives a
different coupling to supplied draws. A multi-qubit measurement consumes
**one** draw for its joint outcome, not one draw per qubit.

After a measurement, compute subsequent weights from the conditional
state. The exact shadow may be unnormalized, so compare against uT rather
than presuming T=1. An independent two-wire |++> followed by separate
measurements at draws 0.25,0.5 returns 0,1 even when the first projection
leaves the exact shadow with total weight 1/2. Entangled wires retain the
correct conditional correlations; a Bell pair measured separately with
0.5,0 returns 1,1.

The interval length is w_s/T, exactly the Born probability. Choosing a
half-open endpoint convention changes only the deterministic coupling to
a supplied draw; it preserves the Born law in §§4.1/14.2. It does not
change gates, coherent control, oracle faults, register ownership, the
measurement ban inside unitary regions, or source measurement's reversal
bar. No North Star capability is narrowed: classical feedback, repeated
measurements, arbitrary-width registers and all demo rungs remain in scope.

## Draw streams, seeds and CLI boundaries

Draws are consumed in **dynamic execution order**, across calls, branches
and loops, one per successfully executed measure. Untaken rows consume
none; deterministic successful measurements consume one; a failed
precondition consumes none. Multiple qubits in one measure still consume
one. There is no independent queue per register or function. Validation of
a supplied draw list is before execution, including any surplus entries.

The CLI's decimal token is parsed to binary64 first. Its exact dyadic
value is the u used by the exact comparison. Thus
`0.49999999999999999` rounds to 0.5 and returns 1 for H, whereas the
round-trip spelling `0.49999999999999994` denotes the predecessor of 0.5
and returns 0. A decimal-looking near tie is not necessarily a different
binary64 draw. Both signs of zero denote the endpoint 0. Nonfinite values,
negative values other than negative zero, and 1 or greater reject. Repeated
`--draws` options reject; a comma-separated list is one option.

When the queue is exhausted the current reference uses xoshiro256** state
`[1,2,3,4]`, taking `(next_u64 >> 11)/2^53`. Supplied draws do not advance
that generator. Surplus valid draws are unused, without a leftover-list
diagnostic. These are reproduced/source-derived **CLI** conventions, not
new universal ISA PRNG/seed requirements. An exhausted list still gets a
Born draw under 0.9; do not mandate an exhaustion fault or a zero fallback.
`--entropy` belongs to the effect device and does not seed measurements.
`--seed` is accepted by `step-check` for §12.1 sampling, not by `run-cc`
or `run-program`. Do not equate that sampler with measurement randomness.
A future execution API may expose a seed; replay portability requires it
to state generator, state and number conversion, or supply an explicit
common draw stream. K17 fixes mapping/consumption, not a cross-executor
unsupplied-random-bit identity. No new seeded CLI is proposed.

## Exact proposed 0.9 diff

An empty semantic diff is sufficient today. If the council wants the
single-H witness made explicit, insert the following directly after
“Born distribution (§14.2).” and before `**DRAW-ORDER witness**`; append
the E entry immediately after K82 in §0.2. This is an editorial reading
already required by K17; the label `K-next` is allocated at application
time and must not collide with concurrent proposals.

```diff
--- a/spec/Bennett-ISA-Spec-0.9.md
+++ b/spec/Bennett-ISA-Spec-0.9.md
@@ -210,6 +210,7 @@
 | K80 | §11 | **CLEANUP-LEAF-OWNERSHIP**. Reverse retains written cleanup points; recompute owns one fresh caller leaf/call space at the carrier, with separate callee spaces and fresh replay identities. Witness: a six-row two-CX computation/cleanup has reverse leaves at entry:3/4/5 and recompute leaves at entry:5 ordinals 0–7. | consumer 5cd; pass X | S | – |
 | K81 | §11 | **CLEANUP-PAIR-PARTITION**. Intrinsic cleanup pairs retain their direct computing operations; release-ended terminal inverse chains coalesce before independent adjoint-call carriers are formed. No inverse occurrence has two owners. Nonzero call targets retain their exact source-derived entry history under recompute. Witness: qcopy/mix/adj-mix/quncopy has carriers at entry:2 and entry:3, independently selectable. | consumer 9hv; pass XII; pending round 9 | S | – |
 | K82 | §11.1, §15 | **NAMED-INTERFACE-RECIPES**. A recipe whose formals are all bare integer names, or none, reads its remaining wires by name from the invoking scope; every other recipe and every template is closed. No catalog byte or hash changes. Witness: bind.value.add(2,x,y,z) expands value.add over x,y,z; a typed recipe reading q, or a site with no q bound, rejects. | b-isa-spec-g2q; bennett-sched-enz; pass XIII; round 10 | S | – |
+| K-next | §4.1, §15 | **DRAW-TIE-WITNESS**. Makes K17's existing half-open CDF intervals and single-H tie explicit; no sampling semantics changes. | b-isa-spec-0h4; b-isa-spec-bas | E | – |
 
 **Decided Q14 (applied 2026-10-29, confirmed 2026-10-30; b-isa-spec-7u6).** Lprocs take explicit
 token and classical parameters (§3.7, §4.2, §4.7, §16.2), forwarded by
@@ -864,6 +865,15 @@
 outcome. Returned integers remain little-endian (wire 0 is bit 0).
 This fixes the reference CLI convention; unsupplied draws follow the
 Born distribution (§14.2).
+
+For ordered outcomes s_j with Born weights w_j, let T = sum_j w_j > 0
+and C_j = sum_(i<=j) w_i, with C_-1 = 0. Outcome s_j is selected exactly
+when C_(j-1) <= draw*T < C_j; an outcome of weight zero is never selected.
+These comparisons use the Born weights, not rounded display probabilities
+or a tie tolerance. One joint n-wire measurement consumes one draw,
+and every later measurement uses the conditional state left by earlier
+measurements. Witness: H on |0> followed by a draw of exactly 0.5 returns
+bit 1; a draw strictly below 0.5 returns 0 and one above it returns 1.
 
 **DRAW-ORDER witness**
 
@@ -4013,6 +4023,11 @@
 ### 14.4 Mechanisation *(deferred)*
 
 ## 15. Conformance suites
+
+K17's measurement vectors include H on a fresh wire with draws immediately
+below, exactly at and immediately above 0.5; the exact tie returns 1.
+Multi-wire vectors retain wire-first lexicographic ordering and one draw
+per joint measurement, including deterministic-state consumption.
 
 The K82 named-interface vector is published in
 `conformance/fixtures/accepted-k82-09/`: the catalog's interfaces and invoke
```

The patch includes this §0.2 insertion, expressed as an exact added table row:

```text
| K-next | §4.1, §15 | **DRAW-TIE-WITNESS**. Makes K17's existing half-open CDF intervals and single-H tie explicit; no sampling semantics changes. | b-isa-spec-0h4; b-isa-spec-bas | E | – |
```

The patch includes this §15 addition immediately before `The K82 named-interface vector`:

```text
K17's measurement vectors include H on a fresh wire with draws immediately
below, exactly at and immediately above 0.5; the exact tie returns 1.
Multi-wire vectors retain wire-first lexicographic ordering and one draw
per joint measurement, including deterministic-state consumption.
```

No 0.10 bump is justified for restating this already-published 0.9 rule.
An actually different tie convention would be semantic and would require
a version decision under [spec/README](../../../spec/README.md); this note
recommends none. The optional wording changes neither canonical source,
semantic envelope, catalog bytes/hash, leaf ordering, nor any object,
interface, closure, target or schedule hash. Regenerate the published
0.8-to-0.9 diff and checker clause indexes after any accepted text edit;
this job edits neither. The K82 note is the precedent for distinguishing
text fixes from catalog changes, not evidence that all semantics may be
changed without a version bump.

## Conformance vectors and consumer impact

All draws in this table have exact binary64 representations except the
explicit parsing vector. Decimal spellings use round-trip values.

| State / measurements | Draw(s) | Expected wire-first bits; integer(s) |
|---|---|---|
| H\|0> | 0 | `0`; 0 |
| H\|0> | 0.49999999999999994 | `0`; 0 |
| H\|0> | 0.5 | `1`; 1 |
| H\|0> | 0.5000000000000001 | `1`; 1 |
| H\|0> | 0.9999999999999999 | `1`; 1 |
| H⊗H\|00> | 0.24999999999999997 / 0.25 / 0.25000000000000006 | `00`/`01`/`01`; 0/2/2 |
| H⊗H\|00> | 0.49999999999999994 / 0.5 / 0.5000000000000001 | `01`/`10`/`10`; 2/1/1 |
| H⊗H\|00> | 0.7499999999999999 / 0.75 / 0.7500000000000001 | `10`/`11`/`11`; 1/3/3 |
| Two fresh \|++> registers, joint measures | 0.25,0.75 | `01`,`11`; 2,3 (published K17 carrier) |
| Fresh deterministic 0, then fresh H\|0> | 0.75,0.25 | `0`,`0`; 0,0 |
| Independent \|++>, measured one wire at a time | 0.25,0.5 | `0`,`1`; 0,1 (conditional normalization) |
| Bell pair, joint | 0.5 | `11`; 3 (unsupported outcomes skipped) |
| Bell pair, separate wires | 0.5,0 | `1`,`1`; 1,1 |
| H\|0>, decimal parsing witness | token 0.49999999999999999 | `1`; 1 after binary64 conversion |

Further acceptance vectors: a repeated call/loop with asymmetric draws;
a branch that skips a measurement; a width/view precondition failure
followed by retry through an API that permits retry, verifying no queue or
PRNG advancement; 128-bit/high-wire values and run-time-width 0.9 measures;
and reversal rejection after any successful source measure. These are
**proposed**, not executions of 0.9 or fault-injection coverage here.

For bennett-vm use strict `CDF > u*T`, wire-first outcome ordering and
conditional weights; retain the stated VM fix if it meets those rules.
Floating roundoff with an epsilon tie test is not this rule. For the
conformance harness serialize explicit common draws, compare bits and
little-endian values, and test FIFO separately from threshold strictness.
Do not mask the H disagreement by dropping exact ties or use two executors'
seed defaults as a portable oracle. 0.8 draws are an explicitly documented
reference/harness profile; 0.9 draws are normative K17. Existing broad
harness claims require their owner's rerun; these finite vectors do not
establish all 32 cases.

## Verification and reproduction

Every shell command below was run with this prefix (and the absolute repo
path as the tool's working directory):

```sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER \
  TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp/sol-isa-textdefects/tmp \
  CARGO_TARGET_DIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp/sol-isa-textdefects/target
```

Exact build and focused existing-test commands:

```sh
timeout 240 cargo build --offline --locked --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml -j4
timeout 180 cargo test --offline --locked --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml -j4 --test registers measurement -- --test-threads=4
timeout 120 python3 -I -B /home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp/sol-isa-textdefects/draw_probe.py
```

Results: build exited 0; both existing measurement tests passed (10 other
register tests filtered); probe passed **28 draw vectors**, the published
K17 carrier and **7 CLI rejection vectors**. Fixtures are generated and
hash-checked only under the job directory. The probe below is exactly the
script run; save it as the named scratch file after making the job's tmp
directory. It checks actual reference CLI results, not a second copy of
the sampler. Its use of the reference hash command to prepare executable
fixtures is not an independent identity proof.

```python
from pathlib import Path
import json, math, os, subprocess
ROOT = Path('/home/tobias/Projects/bennett-workspace/bennett-isa-spec')
JOB = ROOT / '.tmp/sol-isa-textdefects'
BIN = JOB / 'target/debug/bennett-isa-ref'
def cli(*args, good=True):
    p = subprocess.run([str(BIN), *map(str, args)], capture_output=True, text=True, timeout=15)
    if good:
        assert p.returncode == 0, (args, p.stderr)
        return json.loads(p.stdout)
    assert p.returncode == 1 and not p.stdout and p.stderr, (args, p)
    return p.stderr

def fixture(name, actions, cc, results):
    symbols, bodies = {}, []
    for i, (n, widths, body) in enumerate(actions):
        sig = dict(abi='0.8',args=[],bounds=[],**{'in':[]},kind='action',mem=[],mut=widths,tier='q')
        params = ', '.join(f'q{j}:wire[{w}] mut' for j,w in enumerate(widths))
        symbols[str(i)] = dict(interface_hash='0'*64,kind='action',name=n,signature=sig)
        bodies.append(f'action @{n} ({params}) () () () ()\n^entry():\n{body}\n')
    sig = dict(abi='0.8',kind='cc',params=[],results=[f'i{w}' for w in results],tier='cc')
    symbols[str(len(symbols))] = dict(interface_hash='0'*64,kind='cc',name='main',signature=sig)
    outs = ', '.join(f'r{i}:i{w}' for i,w in enumerate(results))
    bodies.append(f'fn cc @main () ({outs}) () () ()\n^entry():\n{cc}\nret '+ ' '.join(f'r{i}' for i in range(len(results)))+'\n')
    def text():
        return ('bennett-object isa=0.8 object=0.8 launch=0.8 sched=0.8 cert=0.8 coverage=0.8 target=0.8 ptr_width=32\n\nsection symbols\n'+json.dumps(symbols,sort_keys=True,separators=(',',':'))+'\n\nsection functions\n'+''.join(bodies))
    path = JOB / (name+'.bisa')
    path.write_text(text())
    hashes = cli('hash', path)['symbols']
    for sym in symbols.values():
        sym['interface_hash'] = hashes[sym['name']]['interface_hash']
    path.write_text(text())
    assert cli('check', path)['status'] == 'ok'
    return path
h1 = [('prep',[1],'h q0[0]')]
one = fixture('one-h', h1, 'q = qreg.new any 1\naction_call @prep (q) () () ()\nr0 = measure q',[1])
two = fixture('two-h', [('prep',[2],'h q0[0]\nh q0[1]')], 'q = qreg.new any 2\naction_call @prep (q) () () ()\nr0 = measure q',[2])
bell = fixture('bell', [('prep',[2],'h q0[0]\ncx q0[0] q0[1]')], 'q = qreg.new any 2\naction_call @prep (q) () () ()\nr0 = measure q',[2])
det = fixture('deterministic', [], 'q = qreg.new any 1\nr0 = measure q',[1])
fifo = fixture('det-then-h', h1, 'a = qreg.new any 1\nr0 = measure a\nb = qreg.new any 1\naction_call @prep (b) () () ()\nr1 = measure b',[1,1])
conditional = fixture('conditional', [('prep',[1,1],'h q0[0]\nh q1[0]')], 'a = qreg.new any 1\nb = qreg.new any 1\naction_call @prep (a, b) () () ()\nr0 = measure a\nr1 = measure b',[1,1])
entangled = fixture('entangled', [('prep',[1,1],'h q0[0]\ncx q0[0] q1[0]')], 'a = qreg.new any 1\nb = qreg.new any 1\naction_call @prep (a, b) () () ()\nr0 = measure a\nr1 = measure b',[1,1])
checks = []
def check(path, draws, bits, values, command='run-cc'):
    actual=cli(command,path,'main','--draws',draws)
    m=actual['measurements']
    assert [x['bits'] for x in m] == bits, (path.name,draws,m,bits)
    assert [int(x['value']) for x in m] == values
    checks.append(dict(fixture=path.name,draws=draws,bits=bits,values=values,command=command))
    return actual
for u,want in [(0.,0),(math.nextafter(.5,0),0),(.5,1),(math.nextafter(.5,1),1),(math.nextafter(1.,0),1)]:
    check(one,repr(u),[str(want)],[want])
for j,u in enumerate([0.,.25,.5,.75]):
    for d,k in [(u,j),(math.nextafter(u,1),j)]+([] if j==0 else [(math.nextafter(u,0),j-1)]):
        bits=['00','01','10','11'][k]
        check(two,repr(d),[bits],[[0,2,1,3][k]])
check(two,repr(math.nextafter(1.,0)),['11'],[3])
check(bell,'0.5',['11'],[3])
check(det,'0.9999999999999999',['0'],[0])
check(fifo,'0.75,0.25',['0','0'],[0,0])
check(conditional,'0.25,0.5',['0','1'],[0,1])
check(entangled,'0.5,0',['1','1'],[1,1])
check(one,'0.49999999999999999',['1'],[1]) # binary64 parsing rounds to 1/2
check(one,'-0',['0'],[0])
check(one,'0.5,0.1',['1'],[1]) # surplus draws do not reject
check(one,'0.5',['1'],[1],command='run-program')
# Default PRNG starts at [1,2,3,4]; these first draws are below 1/2.
check(one,'',['0'],[0])
check(fifo,'0.75',['0','0'],[0,0])
# Published K17's executable carrier is a 0.8 witness, not 0.9 admission.
carrier=ROOT/'conformance/cases/A/A-measure-draw-order-09/two-measures.bisa'
m=cli('run-program',carrier,'twice','--draws','0.25,0.75')['measurements']
assert [(x['bits'],int(x['value'])) for x in m] == [('01',2),('11',3)]
for invalid in ['-0.1','1','NaN','inf','0.5,NaN']:
    error=cli('run-cc',one,'main','--draws',invalid,good=False)
    assert '[0,1)' in error, error
assert 'seed' in cli('run-cc',one,'main','--seed','1,2,3,4',good=False)
assert 'twice' in cli('run-cc',one,'main','--draws','0.5','--draws','0.5',good=False)
(JOB/'draw-results.json').write_text(json.dumps(checks,indent=2)+'\n')
print(f'PASS: {len(checks)} draw vectors; published K17 carrier; 7 CLI rejection vectors')
```

The companion [fd0 note](fd0-import-text.md#verification) contains the exact
combined document-check command and publication checks. `scripts/pre-push`
was read, not run: it invokes git and creates sibling worktrees, prohibited
by this job. No full repository gate, sibling gate, 0.9 interpreter
admission or council acceptance is claimed.

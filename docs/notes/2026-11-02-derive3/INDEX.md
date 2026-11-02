# Contract derivations 3 — council decision index

Five proposals, not published spec changes or consumer acceptance. Read
COMMON, repository CLAUDE/AGENTS, the Beads skill/prime, two first-pass
notes and two pass2 notes before derivation. All five full issue records
were read with bd. Direct sibling reads, including --readonly, failed
at the read-only embedded Dolt LOCK. Copies of the actual embedded
databases under `.tmp/sol-derive3/` then permitted `bd --readonly show`
for 4yg.1 and brt-vbi. No passive JSONL was imported or substituted for
the database. Source issues remain open for council/implementation work.

| Issue | Determination | Recommended publication |
|---|---|---|
| [bennett-arith-4yg.1](bennett-arith-4yg.1.md) | Valid FMA and binary128-conversion midpoints have no even candidate under the printed PPC rule; flags separately violate an existing rule | Dialect §13.1 parity repair in next 0.2 patch and 0.3 draft; no ISA K |
| [brt-vbi](brt-vbi.md) | Published all-width helper requirements lack a public delivery/collection protocol; actual i257 bodies also missing | Dialect §§12.8/19.1 offline interface and fixed point; no new ISA opcode/K |
| [b-isa-spec-pe1](b-isa-spec-pe1.md) | K13 example invents a borrowed lifetime that K70 rejects | 0.9 K13 example erratum: empty borrowed arrays, 24-bit D |
| [b-isa-spec-2qc](b-isa-spec-2qc.md) | Executor preflight and proof owner exist; certificate checker coverage duty is ambiguous | Explicit owner/site/context rule, coordinated with 0jq; council-authorized 0.9 K repair or 0.10 |
| [b-isa-spec-sqr](b-isa-spec-sqr.md) | Bundle entries in an mcx list have no explicit operand rule; current implementations disagree | Ordered bundle expansion, full wire arity/distinctness; council-authorized 0.9 K repair or 0.10 |

None is an excuse to reduce a format/width domain, omit an inverse
obligation, or declare a North Star capability inert. K13 is an example
defect rather than a new machine capability gap. The remaining findings
concern deterministic contract interpretation or delivery. All existing
SQLite, Julia, libc, reversible execution and demo-ladder obligations
remain. Existing beads and cross-repo requests already track the decisions;
no duplicate task or sibling tracker mutation is needed for this pass.

## Publication and combined decisions

[spec/README.md](../../../spec/README.md) requires a full version change
for semantic changes and council review. “0.9 K repair” in these notes
means a proposed use of the existing council repair process, **not
authority to silently amend a published contract**. Without explicit
in-version authorization, the 2qc/sqr semantic additions go to 0.10.
New version prefixes change affected object/interface/closure envelopes,
example identities and consumer pins. A same-version repair with unchanged
source bytes preserves their source identities but still invalidates
incompatible execution/schedule/proof assumptions. `ref@0.9` changes only
if its own catalog bytes/version change; none of these proposals requires
such a rewrite. Assign numeric K entries only at acceptance.

PPC parity and runtime delivery belong to the dialect because they define
LLVM numerical/helper lowering and build inputs. ISA 0.9/0.10 need no
new K for either. Publish the exact proposed dialect text in the next
available maintained patch and mirror it in 0.3; the proposal is not a
private runtime/driver agreement. The new helper protocol is itself
versioned with schema 1, independent of ISA subformat numbers.

For 2qc use the existing concrete invocation-context vocabulary, owner
closure selection and owner-local claim indices. Coordinate with
[pass2 0jq](../2026-11-02-pass2/b-isa-spec-0jq.md); do not create a second
inverse proof owner or replace tuple applicability by root budget equality.
For sqr retain the written source identity and resolve complete wire
operands before K78 primitive matching and scheduling. A parenthesized
indexed list is not currently a legal substitute under §16.2.

## Mixed fat object

Every note cites and applies
[fat-object-scope.md](../2026-11-02-rev2/fat-object-scope.md), binding under
bennett-4ht and advisor 08:45Z (North Star §§2, 3.3, 4.4). A native host
binary remains native; only marked device modules and their required
linked closures become ISA objects. Helper generation, comparison,
certificate inventory and invocation coverage bind those device roots,
with a separate root-relative context [] per root. They do not inventory
native main, host-only argv/envp setup or host-only libc. The shared note
names the exact remaining container/root/certificate association, launch,
budget/handle, tape/run and host-memory mapping changes. This pass neither
implements those mechanisms nor locks them out.

## Bounded verification and observed results

Commands run headless; no git command or full gate was run. Durable
changes are only these six Markdown files; job build/output/snapshots and
the single Resume block live under `.tmp/sol-derive3/`. Sources in the
reference interpreter and sibling repositories were read-only.

The environment prefix, from any working directory, is:

```sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER \
  TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp/sol-derive3/tmp \
  CARGO_TARGET_DIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp/sol-derive3/target
```

For the build, used `ulimit -v 4194304; ulimit -c 0` (4 GiB address space),
`date -u`, `free -g`, `uptime`, then `timeout 240 cargo build --offline
-j2 --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml`.
Exit 0 in 72 seconds. The job's own target directory was used. No cargo
test suite or full repository/sibling gate was run.

Every following execution used `ulimit -v 1048576; ulimit -c 0`
(1 GiB address space), the environment above, and `date -u` in the same
shell command. The script below also imposes subprocess deadlines.

| Command or focused check | Actual result |
|---|---|
| `bd prime`, `bd show` for the three ISA issues, snapshot `bd --readonly show` for the two sibling issues | Exit 0; complete records read, no status changes |
| `timeout 30 python3 -I -B -u /home/tobias/Projects/bennett-workspace/bennett-arith/tests/ppc_zero_low_witness.py` | Exit 0; exact FMA/conversion midpoint proof; native LLVM18/23 FMA flags 0 and conversion flags 24 reproduced, both wrong by final-rounding semantics |
| `timeout 90 python3 -B -m unittest -v tests.test_replay.K13Witness tests.test_replay.QinplaceCoverage` from sibling checker-replay | Exit 0; 12 tests passed (4 K13, 8 coverage) |
| Matching-major LLVM18/23 `opt -passes=verify -disable-output` on runtime-width-request.ll, plus `llvm-nm --defined-only` on each live runtime bitcode | Exit 0; valid source, both required i257 definitions absent on both majors |
| Current replay CLI on the mcx-bundle-control certificate/object, `--max-elements 16` | Exit 0; relation claim proof, 16 elements, fraction 1; unrelated metadata remains outside checked scope |
| Job-built reference `check` on original mcx bundle witness | Exit 1, `§3.3: c must name one wire`; an expected existing implementation boundary |
| Scratch ccx equivalent, reference `check`, `hash`, then 16 forward and 16 adjoint basis calls | All pass; the two object/closure identities are listed in sqr |
| Extracted Python verification block below, `timeout 120 python3 -I -B -u -` under the 1 GiB bound | Exit 0; five-note/index structure, exact witnesses, 12 replay tests, both LLVM audits and all 32 reference executions passed |

The runtime modules actually audited had SHA-256
`cc8f318f2caf9d989ea64932cd177c3bf32e6c65c156b9949d3b49e4b94c999f`
(LLVM18) and
`964ea16513db1d54adbaa5a4c8124b1ac731fee8f487f34dc6e9e12bb9df5642`
(LLVM23). These are observed evidence pins, not a request to re-pin
consumers to missing helper bodies. The proposed canonical request
example in the finite block hashes to
`74e7a75be203bc350a18a84980258fa09a3e9936837e8aa67fb01532064cdd81`;
it is a wire-format example, not an implemented generator result.

The PPC script loads the producer's existing native test libraries. This
pass does not freshly compile them from the publication or infer full
arith conformance from them; its exact rational defect proof is independent
of their answers. Stored older checker verdicts are identified as stored
evidence, distinct from the newly built reference result. The focused
replay tests do not implement the proposed nested-context checker rule.

## Reproducible finite checks

Run the following shell block after the environment prefix. It extracts
the Python below from this index and verifies the actual notes plus
concrete witnesses; it creates only a scratch ccx object. The outer memory
bound is inherited by all child commands.

```sh
date -u
ulimit -v 1048576
ulimit -c 0
timeout 120 python3 -I -B -u - <<'PY'
from pathlib import Path
p = Path('/home/tobias/Projects/bennett-workspace/bennett-isa-spec/docs/notes/2026-11-02-derive3/INDEX.md')
code = p.read_text().split('```python\n', 1)[1].split('\n```', 1)[0]
exec(compile(code, str(p), 'exec'))
PY
```

```python
from pathlib import Path
from fractions import Fraction as F
from itertools import product
import hashlib, json, os, re, subprocess, sys

ROOT = Path('/home/tobias/Projects/bennett-workspace/bennett-isa-spec')
WS = ROOT.parent
OUT = ROOT / 'docs/notes/2026-11-02-derive3'
JOB = ROOT / '.tmp/sol-derive3'
REF = JOB / 'target/debug/bennett-isa-ref'
REPLAY = WS / 'bennett-conformance/checker-replay'
WITNESS = REPLAY / 'witnesses/mcx-bundle-control'
assert os.environ['TMPDIR'] == str(JOB / 'tmp')
assert os.environ['CARGO_TARGET_DIR'] == str(JOB / 'target')
assert not any(k in os.environ for k in
               ('DISPLAY', 'WAYLAND_DISPLAY', 'DBUS_SESSION_BUS_ADDRESS', 'BROWSER'))
ids = ['bennett-arith-4yg.1', 'brt-vbi', 'b-isa-spec-pe1',
       'b-isa-spec-2qc', 'b-isa-spec-sqr']
assert {p.name for p in OUT.iterdir()} == {i+'.md' for i in ids} | {'INDEX.md'}
index = (OUT / 'INDEX.md').read_text()
for ident in ids:
    s = (OUT / (ident+'.md')).read_text()
    assert ident in s and not re.search(r'\b(?:TODO|TBD)\b', s)
    for heading in ('defect', 'north star', 'alternatives', 'recommended exact',
                    'hashes', 'conformance', 'k entry', 'mixed fat object'):
        assert heading in s.lower(), (ident, heading)
    assert 'fat-object-scope.md' in s
    assert index.count('| ['+ident+']') == 1
for p in OUT.glob('*.md'):
    s = p.read_text()
    assert sum(line.startswith('```') for line in s.splitlines()) % 2 == 0, p
    prose = re.sub(r'```[^\n]*\n.*?\n```', '', s, flags=re.S)
    for link in re.findall(r'\[[^\]]+\]\(([^)]+)\)', prose):
        if not link.startswith(('http://', 'https://', '#')):
            assert (p.parent / link.split('#')[0]).exists(), (p, link)
print('PASS: five proposals and index; sections, links, fences, fat-object citations')

def pow2(n):
    return F(2**n) if n >= 0 else F(1, 2**(-n))
def decode(bits, fraction_bits, exponent_bits, bias):
    sign = -1 if bits >> (fraction_bits+exponent_bits) else 1
    exp = (bits >> fraction_bits) & ((1 << exponent_bits)-1)
    frac = bits & ((1 << fraction_bits)-1)
    assert exp != (1 << exponent_bits)-1
    sig = frac if exp == 0 else (1 << fraction_bits) | frac
    return sign * sig * pow2((1-bias if exp == 0 else exp-bias)-fraction_bits)
def f64(bits):
    return decode(bits, 52, 11, 1023)
u = pow2(-1074)
h = f64(0x3ff0000000000001)
assert h == 1+pow2(-52)
exact = (h+u)*F(1,2) + f64(0x3fe0000000000001)
assert exact == h+u/2
H = f64(0x0030000000000001)
V = decode(0x3c030000000000001200000000000000, 112, 15, 16383)
assert V == H+u/2 and H > pow2(-1022)
for hi, v in ((h, exact), (H, V)):
    assert (hi/u).denominator == 1
    assert v-hi == hi+u-v == u/2
    assert hi+u-hi == u
# The two high raw significands and the nonzero low significand are odd.
assert (0x3ff0000000000001 & 1) == (0x0030000000000001 & 1) == 1
assert (0 & 1, 1 & 1) == (0, 1)  # proposed common low comparison selects zero
# Both-low-zero subnormal neighbours still use high parity.
assert f64(2)-f64(1) == u and (1 & 1, 2 & 1) == (1, 0)
print('PASS: independent exact PPC FMA/conversion midpoints and parity distinction')

spec = (ROOT / 'spec/Bennett-ISA-Spec-0.9.md').read_text()
block = spec.split('**CLAIM-REPLAY witness**', 1)[1].split('```json\n', 1)[1].split('\n```', 1)[0]
assert block.count('"borrowed":["1"]') == 6
fixed = block.replace('"borrowed":["1"]', '"borrowed":[]')
fixed = fixed.replace('"borrowed":[1]', '"borrowed":[]')
fixed = fixed.replace('1/33554432', '1/16777216')
k13 = json.loads(fixed)
tuples = [k13['row']['before'], k13['row']['after'],
          k13['wrong_row']['before'], k13['wrong_row']['after'],
          k13['changed_arg_after'], k13['short_budget_after']]
assert all(t['borrowed'] == [] and t['regions'] == [['2','1']] for t in tuples)
assert k13['widths']['borrowed'] == []
bits = sum(k13['widths']['bundles']) + sum(sum(r) for r in k13['widths']['regions'])
assert bits == 24 and str(F(1, 1 << bits)) == k13['covered_fraction']
assert 2 ^ ((1+2) % 4) == 1 and 2 ^ ((1+1) % 4) == 0
assert (2 ^ 0, 3 ^ 1) == (2, 2)
assert k13['sample_count'] == 4 and k13['distinct_samples'] == 1
print('PASS: all six K13 tuple repairs; 24-bit domain and exact singleton coverage')

# A finite coverage counterexample, not an implementation of the checker.
minimum = (3,0,0,0)
def covers(tk, arg, domain=range(4), token_arg=None):
    return (all(a <= b for a,b in zip(minimum,tk)) and arg in domain
            and (token_arg is None or token_arg == (3,0,0,0)))
assert covers((3,0,0,0),1) and covers((9,0,0,0),1)
assert not covers((2,0,0,0),1)
assert not covers((3,0,0,0),3,range(3))
assert not covers((9,0,0,0),1,token_arg=(4,0,0,0))
contexts = [(3,0,0,0), (2,0,0,0)]
assert [covers(t,1) for t in contexts] == [True,False]
assert {('G','H'), ('H','G')} == {('H','G'), ('G','H')}
print('PASS: qinplace minimum, exact-token, args-domain and two-context counterexamples')

# Proposed wire-request example; arithmetic is independent of any runtime body.
mask = (1 << 257)-1
a, b = (1 << 256)+3, 2
assert ((a*b) & mask) == 6
assert a.bit_count() == 3 and (((a*b) & mask).bit_count()) == 2
assert [min(64,257-64*k) for k in range(5)] == [64,64,64,64,1]
entries = [{'op':'mul','width':257}, {'op':'ctpop','width':257}, {'op':'mul','width':257}]
canonical_entries = [{'op':op,'width':w} for op,w in sorted({(e['op'],e['width']) for e in entries})]
assert canonical_entries == [{'op':'ctpop','width':257},{'op':'mul','width':257}]
request = {'schema':'bennett-runtimes-instantiations-1', 'llvm_major':18,
           'target_triple':'x86_64-unknown-linux-musl', 'data_layout':'e-p:64:64',
           'entries':canonical_entries}
canonical = json.dumps(request,sort_keys=True,separators=(',',':')).encode()
print('Proposed request example SHA-256:',hashlib.sha256(canonical).hexdigest())
print('PASS: i257 top-limb arithmetic and canonical request example')

def run(argv, cwd=ROOT, timeout=20, expected=0):
    p = subprocess.run([str(x) for x in argv], cwd=cwd, text=True,
                       capture_output=True, timeout=timeout)
    assert p.returncode == expected, (argv,p.returncode,p.stdout,p.stderr)
    return p
native = run([sys.executable,'-I','-B','-u',WS/'bennett-arith/tests/ppc_zero_low_witness.py'])
assert native.stdout.count('flags=0;') == 2
assert native.stdout.count('flags=24;') == 2
print(native.stdout, end='')
tests = run([sys.executable,'-B','-m','unittest','-v',
             'tests.test_replay.K13Witness','tests.test_replay.QinplaceCoverage'],cwd=REPLAY)
assert 'Ran 12 tests' in tests.stderr and tests.stderr.rstrip().endswith('OK')
print('PASS: independent replay focused tests, 12/12')
runtime = WS/'bennett-runtimes'
ll = runtime/'.tmp/sol-runtimes-queue/tmp/runtime-width-request.ll'
for major in (18,23):
    tools = Path(f'/usr/lib/llvm-{major}/bin')
    run([tools/'opt','-passes=verify','-disable-output',ll])
    bc = runtime/f'dist/llvm{major}/bennett-runtimes.bc'
    nm = run([tools/'llvm-nm','--defined-only',bc]).stdout
    defs = {line.split()[-1] for line in nm.splitlines() if line.split()}
    assert not {'__bennett_rt_i257_mul','__bennett_rt_i257_ctpop'} & defs
    print('PASS: LLVM',major,'i257 source verifies; both definitions absent;',
          'runtime SHA-256',hashlib.sha256(bc.read_bytes()).hexdigest())

replay = json.loads(run([sys.executable,'-B','-m','checker_replay',
                        WITNESS/'certificate.json','--object',WITNESS/'object.bisa',
                        '--max-elements','16'],cwd=REPLAY).stdout)
claim = next(iter(replay['regions'].values()))['claims'][0]
assert (claim['outcome'],claim['elements'],claim['covered_fraction']) == ('proof',16,'1')
original = run([REF,'check',WITNESS/'object.bisa'],expected=1)
assert 'c must name one wire' in original.stderr
scratch = JOB/'tmp/mcx-ccx.bisa'
source = (WITNESS/'object.bisa').read_text()
old = 'e38d5da64f9a0ae89a66243bcbe5a321cb807a5bbabf83952b63d3465dfab346'
new = 'f53238499e07824b6c62385c33426d4bc56bfaf900df61264ec65e0589061f39'
assert source.count('mcx (c) p') == 1 and source.count(old) == 1
scratch.write_text(source.replace('mcx (c) p','ccx c[0] c[1] p').replace(old,new))
for command in ('check','hash'):
    result = json.loads(run([REF,command,scratch]).stdout)
    assert result['object_hash'] == 'c785ff3f310f968e2a26f68834e99ab892cf0d1bffe816cf26b17d45c33a65af'
    assert result['symbols']['a']['interface_hash'] == new
    assert result['symbols']['a']['closure_hash'] == new
for p,q,c in product(range(2),range(2),range(4)):
    result = json.loads(run([REF,'run-action',scratch,'a','--mut',f'{p},{q}','--in',str(c)]).stdout)
    assert result['mut'] == [q ^ int(c==3),p] and result['in'] == [c]
    back = json.loads(run([REF,'run-action',scratch,'a','--mut',
                          ','.join(map(str,result['mut'])),'--in',str(c),'--adjoint']).stdout)
    assert back['mut'] == [p,q] and back['in'] == [c]
print('PASS: mcx replay proof; original reference rejection; ccx check/hash and 16+16 executions')
print('PASS: derive3 verification complete; proposals remain unaccepted/unimplemented')
```

## Remaining decisions and ownership

Council/editor owners must ratify and publish these changes; the existing
five issues remain their tracking records. Arith still owes exact incoming
conversion/FMA mechanics and flags after the parity decision. Runtimes
and driver/plugin still owe the helper protocol and bodies. Checker and
executor owners must add the proposed context/operand vectors, and the
K13 editor must publish a complete corrected fixture. None of that
downstream implementation is claimed complete by this notes-only job.

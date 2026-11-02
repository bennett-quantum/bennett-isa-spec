# Contract pass 2 — council decision index and verification

These are proposals and dispositions, not spec edits or implementation
acceptance. All 13 requested beads were read with `bd show`. ISA records
were read in their originating checkout. Dialect `bd show` could not open
the sibling's read-only Dolt LOCK, including with `--readonly`; copying its
live embedded database to job scratch allowed both `bd --readonly show`
commands to succeed without writing the source repository. The passive
JSONL export was not imported or used as the authoritative database.

The binding COMMON/CLAUDE rules, all first-pass contract notes and the
historical K82 note were read before derivation. Citations below refer to
ISA `spec/Bennett-ISA-Spec-0.9.md`, both maintained dialect specifications
under `../bennett-dialect-spec/spec/`, North Star
`../bennett/docs/Bennett-NorthStar.md`, and the council's ISA rubric.
Originating requests, source fixtures and selected implementation boundaries
were inspected read-only as each note identifies.

| Bead | Verdict | Recommended route |
|---|---|---|
| [b-isa-spec-0jq](b-isa-spec-0jq.md) | Structural-only applicability and action reference hash are gaps; qinplace owner already specified | 0.9 K; null/no-census semantics and linked owner-record checks |
| [b-isa-spec-4ta](b-isa-spec-4ta.md) | One legal mixed-class slot has no B5 descriptor | 0.9 K/B5 class fragments; preserve slot identity |
| [b-isa-spec-v2v](b-isa-spec-v2v.md) | One closure's frames can differ across equal variant/control contexts | 0.9 K; exact context-indexed frames with uniform shorthand |
| [b-isa-spec-7s0](b-isa-spec-7s0.md) | Existing restricted D_F proof has no frame/workspace claim binding | 0.9 K; selected-frame tensor relation, exact K76/K78 entry domain |
| [b-isa-spec-95w](b-isa-spec-95w.md) | Already covered native-model gap; add exact-entry/context/digest qualifications | First-pass 0.9 K plus versioned model; if 0.10, retain explicit current blocker |
| [b-isa-spec-0n5](b-isa-spec-0n5.md) | No demonstrated semantic defect; local diagnostic/report API improvement | First-pass optional editorial/report guidance; no new K requirement |
| [b-isa-spec-jw7](b-isa-spec-jw7.md) | Already covered missing fault census invocation | First-pass 0.9 K; coordinate action applicability and common context set |
| [b-isa-spec-cwq](b-isa-spec-cwq.md) | Already covered repeated-context resource ambiguity | First-pass 0.9 K; align frames/class rows and prevent overlapping-view double counting |
| [b-isa-spec-ysw](b-isa-spec-ysw.md) | Tuple-only bindings lose observable token equality | 0.9 K plus additive/versioned ABI; preserve every consistent alias partition |
| [b-isa-spec-552](b-isa-spec-552.md) | Current published-0.9 fixture admits B5/K13; old boundary is migration/tool evidence | Existing 0.9 adoption and producer/reference follow-up, no new spec mechanism |
| [b-isa-spec-ojy](b-isa-spec-ojy.md) | 31/54 fixture interface arrays contradict K82 body order | Repair fixture under existing K82; no catalog rewrite/new K |
| [b-dialect-spec-2cg](b-dialect-spec-2cg.md) | Minimal-wrapper versus production-history ambiguity; ordinary metadata mechanism exists | Current dialect §17.1 clarification in both maps; retain exact primitive trace |
| [b-dialect-spec-djm](b-dialect-spec-djm.md) | Process state already required; helper name is optional ABI convenience | Optional current dialect §19.2 helper contract; no ISA blocker |

No capability in the North Star thesis, §§3–5 or §8 ladder is removed.
In particular all unmodified SQLite, Julia, operating-system, Grover,
kernel-fuzzing and Monte Carlo requirements remain. An unchecked report
or a mechanics deferral records remaining work; it does not complete or
reduce that scope. No original issue is closed: council decisions and
consumer implementations remain with the listed beads. These findings
are already tracked; no duplicate or cross-repo tracker mutation is needed.

## Publication and combined schema decisions

“0.9 K repair” means the existing council-reviewed repair track, not
permission to alter a published semantic contract silently. `spec/README.md`
normally requires a full-version bump for semantic changes. Council must
choose that publication route and explicit subformat/ABI compatibility
rules. Every note's unchanged-source-hash statement assumes an authorized
in-version repair with unchanged source envelope bytes. A version bump
changes all affected version-prefixed envelopes and consumer pins. The
ref catalog changes only if its own versioned bytes change; none of these
proposals needs a silent rewrite of `ref@0.9`.

Use one context vocabulary across v2v/cwq/jw7: `context_key` is exactly
§12.1's concrete invocation chain in the bound root realisation, including
instance, call_index, loop iterations and applicable oracle step/phase.
The root key is []; nested default-covered calls are included. Resolve
variant/controls/arguments/budget/placements from that tree. Order arrays
by the unsigned UTF-8 bytes of each key's JCS encoding. Frame/resource/
fault arrays validate the same context identities where applicable;
variant/control-count alone is never an invocation identifier. K12 exact
rationals and outward resource display remain unchanged. ysw separately
adds the missing token equality relation; it does not change params_hash
or serialize process ids.

Recommended acceptance groups are applicability/owner binding (0jq),
frames and reports (4ta/v2v/cwq/jw7), borrow proofs (7s0), token context
(ysw), and native models (95w). The fixture and migration dispositions
should proceed with their existing owners. K numbers are assigned only
at acceptance, avoiding collisions with concurrent council work.

## Commands and actual results

All execution used this headless environment, with repository root as
the working directory; no git command was run:

```sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER \
  TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp/sol-contract-pass2/tmp \
  CARGO_TARGET_DIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp/sol-contract-pass2/target
```

The environment above is a command prefix. Scratch and the required
single Resume block are under `.tmp/sol-contract-pass2/`; durable
deliverables are only this index and the 13 sibling notes.

| Command (after that prefix) | Observed result |
|---|---|
| `bd prime` and `bd show <each ISA id>` | Exit 0; no issue status mutation |
| `bd --readonly show b-dialect-spec-2cg` and `... djm` in the isolated database snapshot | Exit 0; both full records read; direct sibling access had failed at LOCK |
| `timeout 180 cargo build --offline -j4 --manifest-path reference-interpreter/Cargo.toml` | Exit 0; own target directory, source unchanged |
| `.tmp/sol-contract-pass2/target/debug/bennett-isa-ref check conformance/fixtures/controlled-action-09/controlled.bisa` | Exit 0, status ok, object hash `45bbb506541e3a8bb28a18e1bb0aef16ba01ee1e7cac74d5a190deb3ad36ef5a` |
| Same binary `hash` on the same fixture | Exit 0, same object/interface/closure identities |
| `timeout 120 python3 -I -B conformance/fixtures/accepted-k82-09/generate.py --check` | Exit 0; 54 named recipes, 20 edges, 86 instances, 16 cases; generator self-consistency only |
| `timeout 120 python3 -I -B conformance/tools/ref_catalog.py --check` | Exit 0; 219 recipes; envelope `5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197` |
| `timeout 120 cargo run -q --offline -j4 --manifest-path conformance/fixtures/accepted-k82-09/probe/Cargo.toml -- conformance/fixtures/accepted-k82-09/k82.json spec/ref-0.9.json` | Exit 0; 86/86 edge digests, 16/16 binding cases, original fixture |
| Extracted Python block below, run with `timeout 300 python3 -I -B -u -` | Exit 0; 13-note structure/links, exact finite witnesses, published CLI identities and 8 action executions; K82 original/corrected probes each 86/86 and 16/16 |
| First-pass verification block extracted from `docs/notes/2026-11-02-contract-verification.md`, run with `timeout 30 python3 -I -B -` | Exit 0; 12 proposals and its exact ring/channel/domain/resource witnesses remain consistent |

The script additionally checks an in-memory body-order correction, writes
only its resulting fixture to job scratch, and compares both documents
after removing interface-enumeration differences (31 interfaces and 10
edge-interface arrays). Every other field,
including all 86 leaf digests and 16 cases, is identical. It independently
counts the 31 differing interfaces. The corrected fixture probe passes;
this does not edit or claim acceptance of the published fixture repair.
An initial combined run timed out at a 45-second per-probe debug limit
after the document/arithmetic/CLI checks passed. The final command uses
120 seconds per probe and 300 seconds overall; no assertion was removed.
Final document/arithmetic additions were rechecked with the same block's
prefix through the line before `def run(argv):`; the unchanged reference
and probe checks above were already complete.
The catalog/document-construction portion through the line before
`for path in (original_path, corrected_path):` was also re-executed:
31 differing interfaces, 10 differing edge arrays, and both artifact
digests below passed. These final checks used `timeout 30 python3 -I -B
-u -`, with the same extraction and `compile`/`exec` method and a distinct
`validation_source` variable so the executed document code could not
overwrite the extraction buffer.

For artifact comparison, SHA-256 of the original `k82.json` file (including
its trailing LF) is
`38be6788bc416385a060f3e4dcb7dff9a57f0fec48ad92b74acd7b22ae9f7efe`;
the corrected scratch fixture is
`2bd6a1512e1762b616fc8194d31a7a7838ede32cb4f4620a33214331d628a90b`.
Neither digest is the separately pinned ref catalog envelope hash.

`scripts/pre-push` was read and not run: it invokes git and creates/removes
sibling worktrees, contrary to this runner's explicit no-git/owned-path
rules. No source implementation changed, no Rust full test gate or sibling
gate is claimed, and no complete numerical certificate proof, new schema
implementation, council acceptance or full demo-rung run is claimed.

## Reproducible finite checks

From the repository root under the environment prefix:

```sh
timeout 300 python3 -I -B -u - <<'PY'
from pathlib import Path
p = Path('docs/notes/2026-11-02-pass2/INDEX.md')
s = p.read_text().split('```python\n', 1)[1].split('\n```', 1)[0]
exec(compile(s, str(p), 'exec'))
PY
```

This code checks exact small mathematical witnesses and actual published
reference interfaces. The arithmetic is not an implementation of any new
certificate verifier or a substitute for its required conformance tests.

```python
from pathlib import Path
from itertools import product
from fractions import Fraction as F
import copy
import hashlib
import json
import os
import re
import subprocess

ROOT = Path('/home/tobias/Projects/bennett-workspace/bennett-isa-spec')
OUT = ROOT / 'docs/notes/2026-11-02-pass2'
SCRATCH = ROOT / '.tmp/sol-contract-pass2'
REF = SCRATCH / 'target/debug/bennett-isa-ref'
PROBE = SCRATCH / 'target/debug/k82-probe'
assert Path.cwd() == ROOT
assert os.environ['TMPDIR'] == str(SCRATCH / 'tmp')
assert os.environ['CARGO_TARGET_DIR'] == str(SCRATCH / 'target')
assert not any(k in os.environ for k in
               ('DISPLAY', 'WAYLAND_DISPLAY', 'DBUS_SESSION_BUS_ADDRESS', 'BROWSER'))

isa = ['0jq', '4ta', 'v2v', '7s0', '95w', '0n5', 'jw7', 'cwq',
       'ysw', '552', 'ojy']
ids = ['b-isa-spec-' + x for x in isa] + [
       'b-dialect-spec-2cg', 'b-dialect-spec-djm']
assert {p.name for p in OUT.iterdir()} == {x + '.md' for x in ids} | {'INDEX.md'}
for ident in ids:
    p = OUT / (ident + '.md')
    s = p.read_text()
    for title in ('Defect', 'North Star', 'Alternatives',
                  'Recommended exact spec diff', 'Hashes', 'Conformance', 'Release'):
        assert title.lower() in s.lower(), (ident, title)
    assert ident in s and not re.search(r'\b(?:TODO|TBD)\b', s)
for p in OUT.glob('*.md'):
    s = p.read_text()
    assert sum(line.startswith('```') for line in s.splitlines()) % 2 == 0, p
    # Remove fenced examples before treating Markdown syntax as links.
    prose = re.sub(r'```[^\n]*\n.*?\n```', '', s, flags=re.S)
    for link in re.findall(r'\[[^\]]+\]\(([^)]+)\)', prose):
        if not link.startswith(('http://', 'https://', '#')):
            assert (p.parent / link.split('#')[0]).exists(), (p, link)
index = (OUT / 'INDEX.md').read_text()
for ident in ids:
    assert index.count('| [' + ident + ']') == 1, ident
print('PASS: 13 per-bead notes and index; required sections, fences and links')

# 0jq: H = M/sqrt(2), so M*M=2I exactly; each H column has two nonzeros.
M = ((1, 1), (1, -1))
assert [[sum(M[i][k]*M[k][j] for k in range(2)) for j in range(2)]
        for i in range(2)] == [[2, 0], [0, 2]]
assert all(sum(M[i][j] != 0 for i in range(2)) == 2 for j in range(2))
# 95w: Bell +/- have norm squared 2 before division by sqrt(2), overlap zero.
plus, minus = [1, 0, 0, 1], [1, 0, 0, -1]
assert sum(a*b for a, b in zip(plus, minus)) == 0
assert sum(a*a for a in plus) == sum(a*a for a in minus) == 2
assert F(1)-F(1) == 0

# 4ta, v2v, cwq: exact basis maps and separate root/child footprints.
for t in range(4):
    assert (t ^ 3) ^ 3 == t
for t, u in product(range(2), repeat=2):
    assert ((t ^ 1) ^ 1, (u ^ 1) ^ 1) == (t, u)
assert (2, 1*2, 1*2, 1, 1) == (2, 2, 2, 1, 1)  # split time, A/B occupancy/peak
child_footprints = [(1, 1, 1), (1, 2, 2)]  # time, occupancy, peak
root = (2, 2*2, 2)
assert root == (2, 4, 2)
assert sum(c[2] for c in child_footprints) != root[2]
assert root[1] + sum(c[1] for c in child_footprints) != root[1]
assert F(1) + F(1,4) == F(5,4) and F(2) + F(1,4) == F(9,4)

# 7s0: a finite model of K76's all-operand stack rule for these CX-only prefixes.
def entry_c(sequence):
    values = {'a': None, 'x': None, 'c': 0}
    stacks = {w: [] for w in values}
    events = []
    for control, target in sequence:  # cx and mcx(1) already normalized to one kind
        identity = (control, target)
        operands = (control, target)
        top = stacks[control][-1] if stacks[control] else None
        if (top is not None and all(stacks[w] and stacks[w][-1] == top
                                   for w in operands)
                and events[top][0] == identity):
            before = events[top][1]
            for w in operands:
                stacks[w].pop()
                values[w] = before[w]
        elif values[control] == 0:
            continue  # disabled: no causal event
        else:
            event = len(events)
            events.append((identity, {w: values[w] for w in operands}))
            for w in operands:
                stacks[w].append(event)
            values[target] = (values[target] ^ values[control]
                              if all(values[w] is not None for w in operands)
                              else None)
    return values['c']
assert entry_c([]) == 0
assert entry_c([('a','c'), ('a','c')]) == 0
assert entry_c([('a','c'), ('x','c'), ('a','c'), ('x','c')]) is None
def frame_ok(c_values):
    return all((c, b ^ c) == (c, b) for c in c_values for b in range(2))
assert frame_ok([0]) and not frame_ok([0, 1])
assert not frame_ok(sorted(set([0] + [1])))  # repeated-visit D_F union cannot be narrowed
entry_products = [(0,0), (1,1)]
assert all(b ^ c ^ d == b for c,d in entry_products for b in range(2))
assert not all(b ^ c ^ d == b for c,d,b in product(range(2), repeat=3))
assert sum(a*b for a, b in zip((1,1), (1,-1))) == 0  # phase on B changes |+>

# 0n5: legal state size versus sample count, without allocating it.
assert (1 << 63) // 8 == (1 << 60)
# jw7: all input/target/fault tuples; F_B's code, not final XOR word.
for budget, expected_faults in [(1, 256), (2, 0)]:
    faults = 0
    for x, target, fault in product(range(4), range(4), range(16)):
        payload, code = (0, 1) if budget == 1 else ((x+1) % 4, 0)
        after = (x, target ^ payload, fault ^ code)
        assert 0 <= after[1] < 4 and 0 <= after[2] < 16
        faults += code != 0
    assert faults == expected_faults
assert 1 ^ 1 == 0 and 1 ^ 0 == 1

# ysw: identity partitions are independent of canonical tuple values.
def partition(handles):
    seen, result = {}, []
    for h in handles:
        if h not in seen:
            seen[h] = len(seen)
        result.append(seen[h])
    return result
def valid_labels(labels):
    return (all(type(x) is int and x >= 0 for x in labels)
            and labels == partition(labels))
assert partition([10,20,20]) == [0,1,1]
assert partition([10,20,30]) == [0,1,2]
assert partition([20,20]) == [0,0] and partition([]) == []
assert all(not valid_labels(x) for x in ([1,1], [0,2], [0,-1]))
args = [['1','0','0','0'], ['1','0','0','0']]
def jcs(x):
    return json.dumps(x, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
params = hashlib.sha256(jcs(args)).hexdigest()
assert params == hashlib.sha256(jcs(copy.deepcopy(args))).hexdigest()
assert jcs({'args':args, 'token_aliases':[0,0]}) != jcs(
           {'args':args, 'token_aliases':[0,1]})
assert (4 if partition([20,20]) == [0,0] else 3) == 4
assert (4 if partition([20,30]) == [0,0] else 3) == 3

# 2cg: exact primitive request/bump history; production metadata stores are not simulated.
def allocation_trace(production, limit):
    bump, records = 4096, []
    def alloc(n):
        nonlocal bump
        if bump + max(n, 1) > 4096 + limit:
            raise OverflowError
        p = bump
        records.append((p, n))
        bump += max(n, 1)
        return p
    payload, code = None, 0
    try:
        alloc(3)
        if production:
            alloc(0)
        padding = (-bump) % 16
        if padding:
            alloc(padding)
        payload = alloc(16)
        if production:
            alloc(32)
    except OverflowError:
        code = 2
    return payload, bump, records, code
minimal = allocation_trace(False, 32)
production = allocation_trace(True, 64)
assert minimal == (4112, 4128, [(4096,3),(4099,13),(4112,16)], 0)
assert production == (4112, 4160,
                      [(4096,3),(4099,0),(4100,12),(4112,16),(4128,32)], 0)
failed = allocation_trace(True, 32)
assert failed == (4112, 4128, production[2][:-1], 2)
assert 16+32 == 32+16 == 48 and min(16,48) != min(32,48)
# djm: required public name/environment examples, not a musl process simulation.
assert '/bin/demo'.rsplit('/',1)[-1] == 'demo'
assert '/bin/'.rsplit('/',1)[-1] == ''
assert dict(s.split('=',1) for s in ['K=V'])['K'] == 'V'
print('PASS: structural, frame/resource, borrow, census, token, allocation and process witnesses')

def run(argv):
    p = subprocess.run([str(x) for x in argv], cwd=ROOT, text=True,
                       capture_output=True, timeout=120)
    assert p.returncode == 0, (argv, p.returncode, p.stdout, p.stderr)
    return p.stdout
fixture = ROOT / 'conformance/fixtures/controlled-action-09/controlled.bisa'
pin = '45bbb506541e3a8bb28a18e1bb0aef16ba01ee1e7cac74d5a190deb3ad36ef5a'
for command in ('check', 'hash'):
    result = json.loads(run([REF, command, fixture]))
    assert result['status'] == 'ok' and result['object_hash'] == pin
for t, c in product(range(2), repeat=2):
    for initial, adjoint, expected in [(t, False, t ^ c), (t ^ c, True, t)]:
        cmd = [REF, 'run-action', fixture, 'controlled', '--mut', str(initial), '--in', str(c)]
        if adjoint:
            cmd += ['--adjoint']
        result = json.loads(run(cmd))
        assert result['mut'] == [expected] and result['in'] == [c]
        assert result['state'] == {str(expected)+str(c): [1,0]}
print('PASS: published B5/K13 check/hash identity and 4 forward/4 adjoint action cases')

# ojy: independent written-order sweep of the current 54 bare-formal recipes.
catalog_path = ROOT / 'spec/ref-0.9.json'
raw = catalog_path.read_bytes()
recipes = {r['kind']:r for r in json.loads(raw)['recipes']}
pin_ref = '5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197'
assert hashlib.sha256(b'isa=0.9\nexpansion=ref\nversion=0.9\n'+raw+b'\n').hexdigest() == pin_ref
body_interfaces = {}
for kind, recipe in recipes.items():
    if not all(isinstance(p,str) for p in recipe['parameters']):
        continue
    used = []
    def walk(body, bound):
        for node in body:
            if 'gate' in node:
                for wire in node['wires']:
                    name = wire.split('[',1)[0]
                    if name not in bound and name not in used:
                        used.append(name)
            elif 'alloc' in node:
                bound.add(node['alloc'])
            elif 'for' in node:
                walk(node['body'], bound | {node['for']})
            elif 'inverse' in node:
                walk(node['body'], bound)
            elif 'release' in node:
                assert node['release'] in bound
            else:
                raise AssertionError((kind, node))  # no unexamined constructors
    walk(recipe['body'], set(recipe['parameters']))
    body_interfaces[kind] = used
assert len(body_interfaces) == 54
changed = [k for k,v in body_interfaces.items() if v != sorted(v)]
assert len(changed) == 31
assert body_interfaces['value.select'] == ['p','b','a','t']

# Execute the existing generator only through document construction, before its file-write branch.
# The one-line correction is in memory; the published generator/fixture are never written.
generator = ROOT / 'conformance/fixtures/accepted-k82-09/generate.py'
source = generator.read_text().split('target = HERE / "k82.json"',1)[0]
assert source.count('return sorted(used)') == 1
scope = {'__file__':str(generator), '__name__':'pass2_in_memory_generator'}
exec(compile(source.replace('return sorted(used)', 'return used'), str(generator), 'exec'), scope)
corrected = scope['doc']
original_path = generator.parent / 'k82.json'
original = json.loads(original_path.read_text())
assert corrected['interfaces'] == body_interfaces
assert sum(original['interfaces'][k] != corrected['interfaces'][k]
           for k in body_interfaces) == 31
changed_edges = sum(a['interface'] != b['interface']
                    for a,b in zip(original['edges'], corrected['edges']))
assert len(original['edges']) == len(corrected['edges']) == 20
assert changed_edges == 10
without_enumeration = copy.deepcopy(corrected)
without_enumeration['interfaces'] = original['interfaces']
for old, new in zip(original['edges'], without_enumeration['edges']):
    new['interface'] = old['interface']
assert without_enumeration == original
assert sum(len(e['instances']) for e in corrected['edges']) == 86
assert len(corrected['cases']) == 16
corrected_path = SCRATCH / 'k82-body-order.json'
corrected_path.write_bytes(jcs(corrected) + b'\n')
for path in (original_path, corrected_path):
    output = run([PROBE, path, catalog_path])
    assert 'edges: 86 match, 0 differ' in output
    assert sum(': PASS' in line for line in output.splitlines()) == 16, output
print('PASS: K82 31/54 ordering mismatches;', changed_edges,
      'edge interface arrays differ; all other fields identical; both probes 86/86 and 16/16')
print('K82 original fixture SHA-256:', hashlib.sha256(original_path.read_bytes()).hexdigest())
print('K82 corrected scratch fixture SHA-256:', hashlib.sha256(corrected_path.read_bytes()).hexdigest())
print('PASS: pass2 verification complete; proposed schemas remain unimplemented/unaccepted')
```

## Remaining acceptance work

Council must decide and publish the exact repairs above before consumers
invent fields. Implementation owners then add the proposed positive and
negative vectors to suites C/F/H and relevant checker/producer tests.
552 needs actual producer adoption; ojy needs the fixture-owner repair;
the two dialect dispositions belong to that contract's council. Every
bead stays open for those decisions/work. This notes-only pass is complete
when the listed commands and finite checks pass; it does not close the
underlying contract or implementation obligations.

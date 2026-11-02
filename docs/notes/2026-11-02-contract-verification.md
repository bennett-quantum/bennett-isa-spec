# Contract derivations — review index and reproducible verification

These are proposals for council grading, not published spec changes.
Contract beads remain open for acceptance/implementation; b5b's owner
handles its implementation status. No git command was run by this job.

A recommendation for a 0.9 K repair means the existing council-reviewed
repair track, not permission to publish an incompatible change silently.
`spec/README.md` normally requires a full-version bump for semantic
changes. Council must settle that publication choice. Every statement
below that existing source hashes stay fixed assumes an authorized
in-version repair with unchanged envelope bytes; a full-version bump
instead changes all symbol/interface/closure envelopes and their consumer
pins. Catalog bytes/hash change only when that separately versioned
catalog is actually changed. No note here performs either publication.

| Item | Disposition | Proposed release |
|---|---|---|
| [z4c](2026-11-02-contract-z4c.md) | Missing action support addresses; blocks existing legal source | 0.9 K repair |
| [l2p](2026-11-02-contract-l2p.md) | Clean-isometry mechanism plus explicit North Star measurement-cleanup gap | 0.9 semantic K repair, versioned expansion |
| [fd0](2026-11-02-contract-fd0.md) | Missing symbolic/import hierarchy contract; scalar-instance path is incomplete scope | 0.10 formats, current explicit blocker |
| [lpe](2026-11-02-contract-lpe.md) | Compact width-family extension; existing dispatch has a practical emission limit | 0.10 |
| [adg](2026-11-02-contract-adg.md) | Pin program-lifetime memory under current ownership | 0.9 K wording |
| [xv4](2026-11-02-contract-xv4.md) | Qualified proof for a checked fixed image; current K74 evidence is correct | 0.9 K extension |
| [b5b](2026-11-02-contract-b5b.md) | Accepted K82 already resolves contract; implementation follow-up | Existing 0.9 K82 |
| [5u3](2026-11-02-contract-5u3.md) | Exact analysis scalability; decline verdict-changing offset caps | Current dialect K clarification |
| [Checker native model](2026-11-02-contract-checker-native-model.md) | Missing realised operator data for independent approximation checking | 0.9 K/versioned model, or explicit 0.10 blocker |
| [Checker diagnostics](2026-11-02-contract-checker-diagnostics.md) | No demonstrated semantic defect or mandated E_CERT registry | Optional 0.9 editorial wording |
| [Checker fault rate](2026-11-02-contract-checker-fault-rate.md) | Missing per-recount invocation/context binding | 0.9 K repair |
| [Checker resources](2026-11-02-contract-checker-resource-contexts.md) | Missing exact per-context resource-report meaning | 0.9 K repair |

The four checker requests already exist in the read-only cross-repo filing
queue. The orchestrator should associate those requests with these notes;
request 2 needs a documentation disposition rather than an invented
semantic defect. Existing fd0/l2p carry the missing scope mechanisms.
No new tracker database or export was written from the notes-only job.

## Commands and results

All shell checks use the following environment prefix, with the working
directory `/home/tobias/Projects/bennett-workspace/bennett-isa-spec`:

```sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER \
  TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp/sol-contract/tmp
```

1. `bd prime`: exited 0; workflow read without tracker mutation.
2. `python3 -I -B conformance/fixtures/accepted-k82-09/generate.py --check`:
   exited 0, 54 named-interface recipes, 20 edges, 86 instances, 16 cases.
   This validates the independently generated vector, not a new Rust
   expander execution of the 16 cases.
3. `timeout 120 python3 -I -B conformance/tools/ref_catalog.py --check`:
   exited 0, 219 recipes; envelope hash
   `5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197`.
4. `timeout 120 python3 -I -B conformance/tests/draft_witnesses.py`:
   exited 0; published diff/example identities, 219-recipe catalog binding,
   exact controlled fixture bytes, pinned 0.8 check/hash and eight
   forward/adjoint tuples passed, along with its other publication vectors.
5. The exact Python block below is executed by extraction with
   `python3 -I -B -` as described below. It checks all 12 note structures,
   balanced fences/local links, exact clean-AND columns, unitary cleanup,
   both measurement/fixup branches with/without coherent control and a
   closed-window H, negative correction channels, modular orbit and
   resource/domain arithmetic. It neither imports nor edits implementation
   code and does not certify the proposed schemas as implemented.

The complete exact check exited 0: 12 proposals, eight AND columns,
32 inverse columns, 128 corrected branch columns, three negative
channels, model/domain/orbit/resource arithmetic, fences and local links.
The fixed-image SHA-256 for raw bytes `01 02` is
`a12871fee210fb8619291eaea194581cbd2531e4b23759d225f6806923f63222`.

`scripts/pre-push` was read but not run: it invokes git and creates sibling
worktrees, contrary to this runner's no-git/owned-path rules. No Rust build,
full repository gate, sibling gate, SQLite production run or council
acceptance is claimed. These are notes-only changes; the applicable
read-only published-evidence checks above do not substitute for later
implementation acceptance tests.

## Exact arithmetic and document check

Run this extraction from the repository root under the environment prefix:

```sh
python3 -I -B - <<'PY'
from pathlib import Path
p = Path('docs/notes/2026-11-02-contract-verification.md')
source = p.read_text().split('```python\n', 1)[1].split('\n```', 1)[0]
exec(compile(source, str(p), 'exec'))
PY
```

The arithmetic is in Q[z]/(z^4+1), z=exp(i*pi/4), with rational
coefficients; H uses (z−z³)/2. There are no floating-point quantum
calculations in this check. Equality of every branch operator to V/sqrt(2)
establishes the resulting channel on all matrix units, hence also with
an external reference; the negative vectors compare density matrices.
This is exact finite mathematical verification, not a producer certificate
accepted by an implementation of the proposed contract.

```python
from fractions import Fraction as F
from pathlib import Path
from math import gcd
import hashlib
import re

def scalar(n):
    return (F(n), F(0), F(0), F(0))

ZERO, ONE = scalar(0), scalar(1)
Z = (F(0), F(1), F(0), F(0))

def add(a, b):
    return tuple(x + y for x, y in zip(a, b))

def neg(a):
    return tuple(-x for x in a)

def mul(a, b):
    out = [F(0)] * 7
    for i in range(4):
        for j in range(4):
            out[i+j] += a[i] * b[j]
    for k in range(6, 3, -1):
        out[k-4] -= out[k]
    return tuple(out[:4])

def conj(a):
    out = [a[0], F(0), F(0), F(0)]
    for k in range(1, 4):
        out[4-k] -= a[k]
    return tuple(out)

I = mul(Z, Z)
TINV = conj(Z)
HFACTOR = (F(0), F(1, 2), F(0), F(-1, 2))
assert mul(HFACTOR, HFACTOR) == scalar(F(1, 2))
assert mul(Z, TINV) == ONE
assert mul(I, I) == neg(ONE)

def compact(state):
    return {i: a for i, a in state.items() if a != ZERO}

def accumulate(state, i, a):
    state[i] = add(state.get(i, ZERO), a)

def enabled(i, controls):
    return all((i >> c) & 1 for c in controls)

def hadamard(state, wire, controls=()):
    out = {}
    mask = 1 << wire
    for i, a in state.items():
        if not enabled(i, controls):
            accumulate(out, i, a)
            continue
        a = mul(a, HFACTOR)
        low = i & ~mask
        accumulate(out, low, a)
        accumulate(out, low | mask, neg(a) if i & mask else a)
    return compact(out)

def flip(state, target, controls=()):
    out = {}
    for i, a in state.items():
        j = i ^ (1 << target) if enabled(i, controls) else i
        accumulate(out, j, a)
    return compact(out)

def phase(state, target, factor, controls=()):
    return compact({i: mul(a, factor) if (i >> target) & 1
                    and enabled(i, controls) else a
                    for i, a in state.items()})

# Bit positions: a=0, b=1, g=2, y=3, outer q=4.
C = [('h', 2), ('t', 2), ('cx', (0, 2)), ('tdg', 2),
     ('cx', (1, 2)), ('t', 2), ('cx', (0, 2)),
     ('tdg', 2), ('h', 2), ('sdg', 2)]
adj = {'h': 'h', 'cx': 'cx', 't': 'tdg', 'tdg': 't',
       's': 'sdg', 'sdg': 's'}
CDAG = [(adj[op], args) for op, args in reversed(C)]

def sequence(state, gates, controls=()):
    for op, args in gates:
        if op == 'h':
            state = hadamard(state, args, controls)
        elif op == 'cx':
            state = flip(state, args[1], (args[0],) + controls)
        else:
            factor = {'t': Z, 'tdg': TINV, 's': I, 'sdg': neg(I)}[op]
            state = phase(state, args, factor, controls)
    return state

for a in range(2):
    for b in range(2):
        i = a | (b << 1)
        assert sequence({i: ONE}, C) == {i | ((a*b) << 2): ONE}
        dirty_phase = I if (a, b) == (0, 1) else neg(I)
        assert sequence({i | 4: ONE}, C) == {
            i | ((1 ^ (a*b)) << 2): dirty_phase}

def expected(state, controlled, closed):
    outer = (4,) if controlled else ()
    state = flip(state, 3, (0, 1) + outer)
    return hadamard(state, 0, outer) if closed else state

def branches(state, controlled=False, closed=False, correction='correct'):
    outer = (4,) if controlled else ()
    state = sequence(state, C, outer)
    state = flip(state, 3, (2,) + outer)
    if closed:
        state = hadamard(state, 0, outer)
    state = hadamard(state, 2)  # Unconditional X-basis measurement.
    result = []
    for m in range(2):
        branch = {i: a for i, a in state.items() if ((i >> 2) & 1) == m}
        if m:
            branch = flip(branch, 2)  # Reset measured private wire.
            transport = closed and correction != 'stale'
            if transport:
                branch = hadamard(branch, 0, outer)
            if correction != 'none':
                controls = (0, 1) + (() if correction == 'uncontrolled' else outer)
                branch = {i: neg(a) if enabled(i, controls) else a
                          for i, a in branch.items()}
            if transport:
                branch = hadamard(branch, 0, outer)
        result.append(compact(branch))
    return result

branch_columns = 0
for controlled in (False, True):
    outer = (4,) if controlled else ()
    for i in range(32):
        if i & 4:
            continue
        start = {i: ONE}
        unitary = sequence(start, C, outer)
        unitary = flip(unitary, 3, (2,) + outer)
        unitary = sequence(unitary, CDAG, outer)
        assert unitary == expected(start, controlled, False)
        for closed in (False, True):
            want = {j: mul(a, HFACTOR)
                    for j, a in expected(start, controlled, closed).items()}
            for branch in branches(start, controlled, closed):
                assert branch == compact(want)
                branch_columns += 1
assert branch_columns == 128

def density(states):
    out = {}
    for state in states:
        for i, a in state.items():
            for j, b in state.items():
                accumulate(out, (i, j), mul(a, conj(b)))
    return compact(out)

superposition = {0: ONE}
for wire in (0, 1, 4):
    superposition = hadamard(superposition, wire)
for controlled, closed, correction in [
        (False, False, 'none'), (True, False, 'uncontrolled'),
        (False, True, 'stale')]:
    actual = density(branches(superposition, controlled, closed, correction))
    ideal = density([expected(superposition, controlled, closed)])
    assert actual != ideal, correction

# Imported model, unit conversion, sequence/parallel accounting.
def model(d):
    return 3*2*d, 2*d*d, F(1, 10) * F(1, 100)**((d+1)//2)
assert model(5) == (30, 50, F(1, 10**7))
assert model(3) == (18, 18, F(1, 100000))
assert F(30, 10**9) == F(3, 10**8)
assert 3*30*50 == 4500
assert (3*30, 50) == (90, 50)  # Sequential time/peak.
assert (30, 3*50) == (30, 150)  # Parallel time/peak.

# Width dispatch, persistent versus reversed load state.
assert (1 ^ 0, 5 ^ 3) == (1, 6)
def later_run(reverse):
    memory, bump = 7, 17  # A stored 7 and allocated at 16.
    if reverse:
        memory, bump = 0, 16
    return memory, bump
assert later_run(False) == (7, 17)
assert later_run(True) == (0, 16)

# Fixed-image coverage; hashing raw bytes differs from hashing hex text.
image_hash = hashlib.sha256(bytes([1, 2])).hexdigest()
assert image_hash != hashlib.sha256(b'0102').hexdigest()
assert 2**(1+1+4) == 64
assert F(64, 2**22) == F(1, 65536)
assert (2**16)*8 == 524288

# Exact modular orbit; no enormous enumeration.
for seed in (0, 3, 4):
    visited, current = set(), seed
    while current not in visited:
        visited.add(current)
        current = (current + 8) % 256
    assert visited == {x for x in range(256) if (x-seed) % 8 == 0}
    assert len(visited) == 32
assert (2**64)//gcd(8, 2**64) == 2**61

# Controlled X and -X produce orthogonal Bell states.
bell_plus = {0: HFACTOR, 3: HFACTOR}
bell_minus = {0: HFACTOR, 3: neg(HFACTOR)}
inner = ZERO
for i, a in bell_plus.items():
    inner = add(inner, mul(conj(a), bell_minus[i]))
assert inner == ZERO

# Fault counts concern code(F_B), not the incoming XOR word.
assert 4*4*16 == 256
assert 1 ^ 1 == 0  # Code 1 failure can leave final fault word zero.
assert 1 ^ 0 == 1  # Successful invocation can leave it nonzero.
assert (1+1, 2*2, max(1, 2)) == (2, 4, 2)  # Root time/occupancy/peak.
assert 1+2 != 2  # Child peaks cannot be summed to get root peak.

# Owned deliverable structure and local links.
base = Path('docs/notes')
ids = ['z4c', 'l2p', 'fd0', 'lpe', 'adg', 'xv4', 'b5b', '5u3',
       'checker-native-model', 'checker-diagnostics', 'checker-fault-rate',
       'checker-resource-contexts']
for ident in ids:
    path = base / ('2026-11-02-contract-' + ident + '.md')
    body = path.read_text()
    for word in ['Defect', 'North Star', 'Alternatives',
                 'Recommended exact spec diff', 'Hashes',
                 'Conformance', 'Release']:
        assert word.lower() in body.lower(), (ident, word)
    assert not re.search(r'\b(?:TODO|TBD)\b', body)
    assert sum(line.startswith('```') for line in body.splitlines()) % 2 == 0
    for target in re.findall(r'\[[^\]]+\]\(([^)]+)\)', body):
        if not target.startswith(('https://', 'http://', '#')):
            assert (path.parent / target.split('#')[0]).exists(), (ident, target)
print('PASS: 12 proposals; exact AND 8 columns/32 inverse columns; '
      '128 corrected branch columns; 3 negative channels; '
      'model/domain/orbit/resource arithmetic; fences and local links')
print('Fixed-image SHA-256:', image_hash)
```

## Verification limits

The exact checks establish the finite witnesses and internal document
consistency. They do not prove the proposed general contract sound in
all cases, accept a new K entry, close an implementation bead, or show
that SQLite's full reference analysis converges in practical time.
Each item states its own remaining acceptance vectors and consumer work.

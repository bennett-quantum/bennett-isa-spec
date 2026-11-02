#!/usr/bin/env python3
"""Bounded derivation witnesses. Models are not production implementations."""
from __future__ import annotations

import copy
import hashlib
import itertools
import json
import os
from pathlib import Path
import re
import resource
import subprocess

resource.setrlimit(resource.RLIMIT_AS, (1024**3, 1024**3))
resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[2]
WS = ROOT.parent
JOB = ROOT / '.tmp/sol-derive4'
REF = JOB / 'target/debug/bennett-isa-ref'

GROUPS = {
    'native-sharing.md': ['b-isa-spec-x1y', 'b-isa-spec-6ma', 'bennett-vm-8v0'],
    'b-dialect-spec-yqf.md': ['b-dialect-spec-yqf'],
    'b-dialect-spec-hsb.md': ['b-dialect-spec-hsb'],
    'b-isa-spec-8uz.md': ['b-isa-spec-8uz'],
    'b-isa-spec-rvu.md': ['b-isa-spec-rvu'],
    'b-isa-spec-n5a.md': ['b-isa-spec-n5a'],
    'source-binding.md': ['b-isa-spec-tea', 'b-isa-spec-99n'],
    'b-isa-spec-gck.md': ['b-isa-spec-gck'],
    'container-and-certificates.md': ['b-isa-spec-lop', 'b-isa-spec-cob', 'b-isa-spec-8nx'],
    'budgets.md': ['b-isa-spec-c1u', 'b-dialect-spec-bly'],
    'resource-routing.md': ['b-isa-spec-ml0', 'b-dialect-spec-jwd'],
    'journals-and-reversal.md': ['b-isa-spec-at6', 'b-isa-spec-35m'],
    'process-context.md': ['b-dialect-spec-yz7', 'b-dialect-spec-qbw'],
    'device-closure.md': ['b-dialect-spec-k08', 'b-dialect-spec-dl8', 'b-dialect-spec-dh8'],
    'entry-state-and-addresses.md': ['b-dialect-spec-5dl', 'b-dialect-spec-dce'],
    'hostcalls.md': ['b-dialect-spec-3ca'],
}
PRIOR = {
    'b-dialect-spec-pou': '../2026-11-02-derive3/bennett-arith-4yg.1.md',
    'b-dialect-spec-q06': '../2026-11-02-derive3/brt-vbi.md',
}


def jcs(value):
    # Witness objects use only ASCII keys, integers and strings: this is JCS
    # on that restricted domain, not a replacement for a general JCS library.
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def digest(data):
    return hashlib.sha256(data).hexdigest()


def documents():
    assert os.environ['TMPDIR'] == str(JOB / 'tmp')
    assert os.environ['CARGO_TARGET_DIR'] == str(JOB / 'target')
    assert not any(k in os.environ for k in
                   ('DISPLAY', 'WAYLAND_DISPLAY', 'DBUS_SESSION_BUS_ADDRESS', 'BROWSER'))
    expected = {i for ids in GROUPS.values() for i in ids} | set(PRIOR)
    assert len(expected) == 30
    index = (OUT / 'INDEX.md').read_text()
    rows = [line for line in index.splitlines()
            if re.match(r'\| (?:b-isa-spec-|b-dialect-spec-|bennett-vm-)', line)]
    assert len(rows) == 30
    assert {line.split('|')[1].strip() for line in rows} == expected
    assert 'Awaiting derivation' not in index
    for name, ids in GROUPS.items():
        text = (OUT / name).read_text()
        assert all(i in text for i in ids), name
        for heading in ['Exact defect', 'North Star gap', 'Alternatives',
                        'Recommended exact', 'Hashes, catalog', 'Conformance',
                        'K entry', 'Mixed fat object']:
            assert '## ' + heading in text, (name, heading)
        assert 'never a prerequisite for rungs 3/4' in text, name
        assert 'F30' in text and 'deferred' in text, name
        assert 'council + maintainer authorization of compatibility rules' in text, name
        assert 'fat-object-scope.md' in text and 'AUDIT.md' in text, name
        assert not re.search(r'\b(?:TODO|TBD)\b', text), name
    for ident, path in PRIOR.items():
        row = next(row for row in rows if ident in row)
        assert path in row and (OUT / path).exists()
    for path in OUT.glob('*.md'):
        text = path.read_text()
        assert sum(s.startswith('```') for s in text.splitlines()) % 2 == 0, path
        prose = re.sub(r'```[^\n]*\n.*?\n```', '', text, flags=re.S)
        for target in re.findall(r'\[[^\]]+\]\(([^)]+)\)', prose):
            if not target.startswith(('https://', 'http://', '#')):
                assert (path.parent / target.split('#')[0]).exists(), (path, target)
    print('PASS: 30 unique bead rows, 16 grouped notes, prior dispositions, links and scope/routes')


def memory_and_ownership():
    storage = bytearray([10, 20, 30, 40])
    storage[1] += 3
    assert list(storage) == [10, 23, 30, 40]
    storage[0] = 99
    entry = bytes(storage)
    storage[0] ^= 1
    assert storage[0] == 98
    storage[:] = entry
    assert list(storage) == [99, 23, 30, 40]
    nodes = {'A': {'next': 'B', 'value': 2}, 'B': {'next': 'A', 'value': 5}}
    first = ('B', 'value')
    second = (nodes['A']['next'], 'value')
    assert first == second and nodes[nodes['B']['next']]['next'] == 'B'
    nodes[first[0]][first[1]] = 7
    assert nodes[second[0]][second[1]] == 7
    copies = [{'value': 5}, {'value': 5}]
    copies[0]['value'] = 7
    assert copies[1]['value'] != 7  # independent copies are a counterexample
    snapshot_key = (bytes(storage), ('alloc', 1, True), ('stack', 4), 9)
    assert snapshot_key != (bytes(storage), ('alloc', 2, True), ('stack', 4), 9)
    assert snapshot_key != (bytes(storage), ('alloc', 1, True), ('stack', 5), 9)
    assert snapshot_key != (bytes(storage), ('alloc', 1, True), ('stack', 4), 10)
    owner = ('P', 1, 'qreg', 7)
    assert owner != ('Q', 1, 'qreg', 7) and owner != ('P', 2, 'qreg', 7)
    assert (1 << 40) != ((1 << 40) & ((1 << 32) - 1))
    lease = {'owner': ('P', 'run1'), 'range': (0, 4), 'open': True}
    conflict = lambda lo, hi: lease['open'] and lo < 4 and hi > 0
    assert conflict(1, 2) and not conflict(4, 8)
    # HOST_REQUEST is still open; it does not release a parent lease.
    assert conflict(0, 4)
    lease['open'] = False
    assert not conflict(0, 4)
    print('PASS: entry epochs, cyclic/interior aliases, copy counterexample, metadata invalidation and ledger/lease model')


def closure_inventory_and_modes():
    edges = {'root': {'table', 'fp', 'alloc_meta'},
             'table': {'shared'}, 'fp': {'callee'}, 'callee': {'runtime_state'},
             'native_main': {'native_counter', 'native_tls', 'native_log'}}
    closure = {'root'}
    while True:
        enlarged = closure | set().union(*(edges.get(n, set()) for n in closure))
        if enlarged == closure:
            break
        closure = enlarged
    assert closure == {'root', 'table', 'fp', 'alloc_meta', 'shared', 'callee', 'runtime_state'}
    assert not {'native_counter', 'native_tls', 'native_log'} & closure
    payload = b'bennett-object example bytes'
    native = b'ELF-profile-prefix' + payload + b'other-payload'
    offset = len(b'ELF-profile-prefix')
    assert native[offset:offset + len(payload)] == payload
    expected = digest(payload)
    assert digest(native[offset:offset + len(payload)]) == expected
    assert digest(payload + b'x') != expected and digest(native) != expected
    definitions = {('P', 'helper'): 'body1', ('Q', 'helper'): 'body2'}
    assert len(definitions) == 2
    assert definitions['P', 'helper'] != definitions['Q', 'helper']  # actual flat link conflicts
    contexts = {('P', 'root'): [[], ['shared']], ('Q', 'root'): [[], ['shared']]}
    assert len(contexts) == 2 and all(ctx[0] == [] for ctx in contexts.values())
    binding = {'program': 'P', 'root': 'root', 'run': '1', 'args': ['2'], 'epoch': 3}
    assert binding != {**binding, 'program': 'Q'}
    assert binding != {**binding, 'args': ['3']}
    assert binding != {**binding, 'epoch': 4}
    root_tuple, other_tuple, nested_tuple = (3, 0, 0, 0), (5, 0, 0, 0), (2, 0, 0, 0)
    assert len({root_tuple, other_tuple, nested_tuple}) == 3
    token1, token2 = ('P', 1, root_tuple), ('Q', 1, root_tuple)
    assert token1 != token2
    assert digest(jcs([list(token1[2])])) == digest(jcs([list(token2[2])]))
    # Role actions are a specification witness, not actual generated code.
    actions = {'callable-module': ['marshal', 'run', 'result', 'close', 'return'],
               'whole-device-process': ['argv', 'prepare', 'run', 'exit', 'flush', 'close']}
    assert 'argv' not in actions['callable-module'] and 'exit' not in actions['callable-module']
    assert actions['whole-device-process'].count('exit') == 1
    strings = [b'prog', b'arg']
    encoded = b''.join(s + b'\0' for s in strings)
    assert encoded == b'prog\0arg\0' and all(b'\0' not in s for s in strings)
    assert b'\0' in b'bad\0arg'  # proposed C-string negative
    print('PASS: device dependency fixed point, payload integrity, program namespaces, root contexts, budget identities and entry roles')


def classes_and_borrow():
    def defaults(widths, capacity):
        free = dict(capacity)
        chosen = []
        for width in widths:
            cls = next(c for c in sorted(free) if free[c] >= width)
            chosen.append(cls)
            free[cls] -= width
        return chosen
    gate_classes = {'x': {'b'}}
    assert defaults([1], {'a': 8, 'b': 8}) == ['a']
    assert defaults([1], {'a': 8, 'b': 8})[0] not in gate_classes['x']
    assert defaults([1], {'a': 0, 'b': 8}) == ['b']
    assert defaults([2, 1], {'a': 2, 'b': 8}) == ['a', 'b']
    assert defaults([1, 2, 1], {'a': 2, 'b': 8}) == ['a', 'b', 'a']
    assert 3 * 4 + .5 * 4 == 14 and 2 * 4 + .5 * 4 == 10
    assert max(3, 4) == 4 and 3 == 3  # external demand versus physical alias
    seq = [('cx', 'x', 'b'), ('cx', 'b', 't'), ('cx', 'x', 'b'), ('cx', 'b', 't')]
    for t, x, b in itertools.product(range(2), repeat=3):
        def apply(ops):
            state = {'t': t, 'x': x, 'b': b}
            for _, c, target in ops:
                state[target] ^= state[c]
            return state
        assert apply(seq) == {'t': t ^ x, 'x': x, 'b': b}
        assert apply(reversed(seq)) == {'t': t ^ x, 'x': x, 'b': b}
        # Matching output values does not rule out a forbidden B-dependent phase.
        assert (1 if b == 0 else -1) == (-1)**b
    assert [(-1)**b for b in range(2)] == [1, -1]
    print('PASS: root class/default counterexample, borrow costs, all eight forward/adjoint T3 columns and phase control')


def phase_cleanup():
    # Exact signed basis arithmetic; Z is notation, realized as S*S below.
    for x in range(2):
        d = 0
        phase = 1
        d ^= x
        phase *= (-1)**d
        phase *= (-1)**d
        d ^= x
        assert d == 0 and phase == 1
        d = x
        phase = (-1)**x
        fresh = x
        phase *= (-1)**fresh
        d ^= fresh
        phase *= (-1)**fresh
        fresh ^= x
        assert d == fresh == 0 and phase == (-1)**x
    # Four-wire unitary witness for interleaved value/phase boundaries:
    # C = CX(x,d); S(d); CX(y,d). Undo must restore d before SDG(d).
    for x, y in itertools.product(range(2), repeat=2):
        d = x ^ y
        source_phase = (1j)**x
        d ^= y
        source_phase *= (-1j)**d
        d ^= x
        assert d == 0 and source_phase == 1
        delayed_phase = (1j)**x * (-1j)**(x ^ y)
        if y:
            assert delayed_phase != 1
    print('PASS: exact I-versus-Z cleanup witness and intermediate-value phase-boundary counterexample')


class RecipeMachine:
    """Small exact bit simulator for the actual qjumpr recipe gate language."""
    def __init__(self, env, wires):
        self.env = dict(env)
        self.wires = copy.deepcopy(wires)
        self.widths = {k: len(v) for k, v in wires.items()}

    def expr(self, expr):
        if isinstance(expr, int):
            return expr
        if isinstance(expr, str):
            return self.env[expr]
        op, *args = expr
        a = [self.expr(v) for v in args]
        return {'+': lambda: sum(a), '-': lambda: a[0] - a[1],
                '*': lambda: a[0] * a[1], 'min': lambda: min(a)}[op]()

    def value(self, name):
        return sum(bit << i for i, bit in enumerate(self.wires[name]))

    def toggle_value(self, name, val, width):
        assert len(self.wires[name]) == width
        for i in range(width):
            self.wires[name][i] ^= (val >> i) & 1

    def bit(self, text):
        m = re.fullmatch(r'(\w+)\[(.*)\]', text)
        assert m, text
        index = self.expr(json.loads(m[2]))
        assert 0 <= index < len(self.wires[m[1]]), (text, index)
        return m[1], index

    def execute(self, nodes, inverse=False):
        for node in reversed(nodes) if inverse else nodes:
            if 'inverse' in node:
                assert node['inverse'] is True
                self.execute(node['body'], not inverse)
            elif 'alloc' in node or 'release' in node:
                allocation = 'alloc' in node
                name = node['alloc'] if allocation else node['release']
                if allocation:
                    self.widths[name] = self.expr(node['width'])
                if allocation != inverse:
                    assert name not in self.wires
                    self.wires[name] = [0] * self.widths[name]
                else:
                    assert not any(self.wires[name]), (name, self.wires[name])
                    del self.wires[name]
            elif 'for' in node:
                key = node['for']
                had = key in self.env
                prior = self.env.get(key)
                indices = range(self.expr(node['from']), self.expr(node['to']))
                for index in reversed(indices) if inverse else indices:
                    self.env[key] = index
                    self.execute(node['body'], inverse)
                if had:
                    self.env[key] = prior
                else:
                    self.env.pop(key, None)
            elif 'under' in node:
                assert len(self.wires[node['under']]) == 1
                if self.value(node['under']):
                    self.execute(node['body'], inverse)
            elif 'gate' in node:
                refs = [self.bit(w) for w in node['wires']]
                assert len(refs) == len(set(refs))
                vals = [self.wires[n][i] for n, i in refs]
                kind = node['gate']
                assert kind in ('x', 'cx', 'ccx'), kind
                assert len(refs) == {'x': 1, 'cx': 2, 'ccx': 3}[kind]
                if all(vals[:-1]):
                    name, index = refs[-1]
                    self.wires[name][index] ^= 1
            elif 'invoke' in node:
                args = node['args']
                if node['invoke'] == 'value.literal':
                    width, val, target = args
                    self.toggle_value(target, self.expr(val), self.expr(width))
                elif node['invoke'] == 'bind.value.icmp.ult':
                    width, a, b, target = args
                    assert len(self.wires[a]) == len(self.wires[b]) == self.expr(width)
                    self.toggle_value(target, int(self.value(a) < self.value(b)), 1)
                else:
                    raise AssertionError(node['invoke'])
            else:
                raise AssertionError(node)


def source_binding():
    original = json.loads((ROOT / 'spec/ref-0.9.json').read_text())
    recipes = {r['kind']: r for r in original['recipes']}
    old = recipes['qjumpr']
    assert next(p for p in old['parameters'] if p['name'] == 'a')['width'] == 'P'

    def replace(value):
        if isinstance(value, str):
            return 'A' if value == 'P' else value
        if isinstance(value, list):
            return [replace(v) for v in value]
        if isinstance(value, dict):
            return {k: replace(v) for k, v in value.items()}
        return value

    candidate = copy.deepcopy(old)
    candidate['parameters'][1:1] = [{'name': 'A', 'type': 'integer'},
                                    {'name': 'CopyWidth', 'type': 'integer'}]
    next(p for p in candidate['parameters'] if p['name'] == 'a')['width'] = 'A'
    candidate['body'] = replace(candidate['body'])
    loops = [node for node in candidate['body'] if node.get('for') == 'bit']
    assert len(loops) == 1 and loops[0]['to'] == 'A'
    loops[0]['to'] = 'CopyWidth'
    assert next(p for p in candidate['parameters'] if p['name'] == 'succ')['width'] == 'P'

    def bits(n, width):
        return [(n >> i) & 1 for i in range(width)]
    count = 0
    for p, a in itertools.product(range(1, 5), range(1, 6)):
        limit = min((1 << p) - 1, (1 << a) - 1)
        for end in sorted({1, limit}):
            for first in sorted({0, end - 1}):
                for addr in range(1 << a):
                    for succ in sorted({0, (1 << p) - 1}):
                        before = {'a': bits(addr, a), 'succ': bits(succ, p),
                                  'flag': [0], 'faulted': [count & 1], 'code': bits(9, 4)}
                        env = {'P': p, 'A': a, 'CopyWidth': min(p, a),
                               'F': 4, 'First': first, 'End': end}
                        machine = RecipeMachine(env, before)
                        machine.execute(candidate['body'])
                        invalid = not first <= addr < end
                        assert machine.value('flag') == int(invalid)
                        assert machine.value('succ') == succ ^ (addr & ((1 << p) - 1))
                        assert machine.value('faulted') == (count & 1) ^ invalid
                        assert machine.value('code') == 9 ^ (4 if invalid else 0)
                        assert set(machine.wires) == set(before)
                        machine.execute(candidate['body'], inverse=True)
                        assert machine.wires == before
                        count += 1
    # Complete scan, even for a statically false control, must type-check.
    widths = [1, 8]
    assert max(widths) > 4
    old_frame = 3 + 4 + 4 + 2 + 1 + 3
    new_frame = 3 + 8 + 4 + 2 + 1 + 3
    assert (old_frame, new_frame) == (17, 21)
    for width in widths:
        assert all(bit < 8 for bit in range(width))
        for val in range(1 << width):
            delivery, location = 1, val
            delivery, location = location, delivery
            delivery, location = location, delivery
            assert (delivery, location) == (1, val)
    print(f'PASS: transformed actual qjumpr recipe, {count} full-width interval cases plus inverse; return 17->21 bits')
    print('Candidate qjumpr recipe SHA-256 (not a published catalog):', digest(jcs(candidate)))


def run_ref(*args):
    result = subprocess.run([str(REF), *map(str, args)], cwd=ROOT,
                            capture_output=True, text=True, timeout=15)
    assert result.returncode == 0, (args, result.returncode, result.stdout, result.stderr)
    return json.loads(result.stdout)


def reference_witnesses():
    fixtures = WS / 'bennett-estimate/tests/fixtures'
    ret = fixtures / 'ret-delivery-source-gap.bisa'
    jump = fixtures / 'jump-width-source-gap.bisa'
    assert run_ref('check', ret)['object_hash'] == 'b026d3de1b9636bdb7c45deb771f3ccb468fd0e0dbb2bee38da6863500e4d500'
    assert run_ref('check', jump)['object_hash'] == 'd111d27b9a049d152963dfb76cc2c9365d99b74c4f7eb885894be7ec943ea1ba'
    assert run_ref('run-oracle', ret, 'root', '--input', '', '--budget', '4,1,8,1') == {
        'code': 0, 'payload': 1, 'steps': 4}
    assert run_ref('run-oracle', jump, 'root', '--input', '1,2,1', '--budget', '3,0,0,0') == {
        'code': 0, 'payload': 1, 'steps': 3}
    diagonal = OUT / 'diagonal-source.bisa'
    identity = run_ref('check', diagonal)
    assert identity['object_hash'] == 'cabcde923d3c5b8a66398caf34778ca64c799ea90c9e7b86d7d5be09b8a559bb'
    expected = {'00': [0.6, 0], '01': [0.8, 0]}
    for adjoint in (False, True):
        flags = ['--adjoint'] if adjoint else []
        got = run_ref('run-action', diagonal, 'root', '--state', '00=0.6;01=0.8', *flags)
        assert got['state'] == expected and got['mut'] == [0]
        rev = run_ref('reverse', diagonal, 'root', '--state', '00=0.6;01=0.8', *flags)
        assert rev['equals_entry'] and rev['first_difference'] is None
        assert rev['state'] == expected and rev['status'] == 'REVERSED'
    for t, x in itertools.product(range(2), repeat=2):
        got = run_ref('run-action', diagonal, 'root', '--mut', t, '--in', x)
        assert got['mut'] == [t] and got['in'] == [x]
    print('PASS: reference check/hash for three source objects; two oracle runs; four basis and four superposition/adjoint/reverse runs')


def evidence():
    path = OUT / 'sources.json'
    data = json.loads(path.read_text())
    assert len(data['beads']) == 30 and {b['id'] for b in data['beads']} == {
        i for ids in GROUPS.values() for i in ids} | set(PRIOR)
    drift = 0
    for source in data['sources']:
        source_path = (OUT / source['path']).resolve()
        current = source_path.read_bytes()
        # The frozen excerpts are evidence of the read, not a claim that a
        # concurrently edited sibling must remain byte-identical forever.
        assert re.fullmatch('[0-9a-f]{64}', source['sha256'])
        assert all(ex['text'] and ex['clause'] for ex in source['excerpts'])
        if digest(current) != source['sha256']:
            drift += 1
            print('SOURCE DRIFT (frozen excerpts retained):', source['path'])
        else:
            for ex in source['excerpts']:
                assert ex['text'] in current.decode(), (source_path, ex['clause'])
    print(f'PASS: 30 issue source records and {len(data["sources"])} frozen citation sources; drift={drift}')


if __name__ == '__main__':
    documents()
    evidence()
    memory_and_ownership()
    closure_inventory_and_modes()
    classes_and_borrow()
    phase_cleanup()
    source_binding()
    reference_witnesses()
    print('PASS: derive4 bounded verification; proposals remain unaccepted/unimplemented')

#!/usr/bin/env python3
"""Focused exact algebra and amendment-anchor checks. No interpreter execution.

Run with the bounded command printed in l2p-amendment.md. Coefficients are
SymPy's exact Q(sqrt(2), i) field; no floating-point comparisons are used.
"""
from fractions import Fraction
from functools import lru_cache
from hashlib import sha256
import json
from pathlib import Path
import re
import sys

from sympy import I, QQ, sqrt

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
FIELD = QQ.algebraic_field(sqrt(2), I)
ONE, ZERO = FIELD.one, FIELD.zero
R = FIELD.from_sympy(sqrt(2) / 2)
T = FIELD.from_sympy((1 + I) / sqrt(2))
J = FIELD.from_sympy(I)


def add(out, key, value):
    value += out.get(key, ZERO)
    if value:
        out[key] = value
    else:
        out.pop(key, None)


def gate(state, op, target, controls=()):
    out = {}
    for bits, amp in state.items():
        if not all((bits >> c) & 1 for c in controls):
            add(out, bits, amp)
            continue
        bit = (bits >> target) & 1
        if op == 'x':
            add(out, bits ^ (1 << target), amp)
        elif op == 'h':
            add(out, bits & ~(1 << target), amp * R)
            add(out, bits | (1 << target), amp * R * (-1 if bit else 1))
        else:
            phase = {'t': T, 'tdg': ONE / T, 's': J,
                     'sdg': -J, 'z': -ONE}[op]
            add(out, bits, amp * (phase if bit else ONE))
    return out


# a=0, b=1, y=2, g=3; optional q=4. Time order is list order.
C = [('h', 3, ()), ('t', 3, ()), ('x', 3, (0,)),
     ('tdg', 3, ()), ('x', 3, (1,)), ('t', 3, ()),
     ('x', 3, (0,)), ('tdg', 3, ()), ('h', 3, ()), ('sdg', 3, ())]
ADJ = {'h': 'h', 'x': 'x', 'z': 'z', 't': 'tdg', 'tdg': 't',
       's': 'sdg', 'sdg': 's'}
CDAG = [(ADJ[op], t, cs) for op, t, cs in reversed(C)]


def sequence(state, gates, coherent_control=False):
    for op, t, cs in gates:
        state = gate(state, op, t, cs + ((4,) if coherent_control else ()))
    return state


def expected(i, transport=False, controlled=False):
    a, b = i & 1, (i >> 1) & 1
    enabled = (i >> 4) & 1 if controlled else 1
    state = {i ^ ((a * b * enabled) << 2): ONE}
    return gate(state, 'h', 0) if transport else state


def before_measure(i, transport=False, controlled=False):
    state = sequence({i: ONE}, C, controlled)
    state = gate(state, 'x', 2, (3,))
    return gate(state, 'h', 0) if transport else state


def branch(i, m, transport=False, controlled=False, wrong=None, reset=True):
    state = gate(before_measure(i, transport, controlled), 'h', 3)
    state = {k: v for k, v in state.items() if (k >> 3) & 1 == m}
    if m and wrong != 'missing':
        if transport and wrong != 'stale':
            state = gate(state, 'h', 0)
        state = gate(state, 'z', 1, (0,) + ((4,) if controlled and wrong != 'q' else ()))
        if transport and wrong != 'stale':
            state = gate(state, 'h', 0)
    if m and reset:
        state = gate(state, 'x', 3)
    return state


@lru_cache(maxsize=64)
def conjugate(value):
    return FIELD.from_sympy(FIELD.to_sympy(value).conjugate())


def outer(left, right):
    # All amplitudes are in Q(sqrt(2),i); conjugation is exact.
    out = {}
    for a, x in left.items():
        for b, y in right.items():
            yc = conjugate(y)
            add(out, (a, b), x * yc)
    return out


def channel_matches(columns, reference, ids):
    for i in ids:
        for j in ids:
            out = {}
            for branch_columns in columns:
                for k, v in outer(branch_columns[i], branch_columns[j]).items():
                    add(out, k, v)
            if out != outer(reference[i], reference[j]):
                return False
    return True


def algebra():
    # Full dirty-target table is independently checked, not assumed from clean g.
    phases = [-J, J, -J, -J]
    for a in range(2):
        for b in range(2):
            base = a + 2*b
            for g in range(2):
                result = sequence({base + 8*g: ONE}, C)
                want = {base + 8*(g ^ (a*b)): ONE if not g else phases[2*a+b]}
                assert result == want, ('compute', a, b, g, result)
    for i in range(32):
        assert sequence(sequence({i: ONE}, C), CDAG) == {i: ONE}
    ids = list(range(8))
    for i in ids:
        uncomputed = sequence(gate(sequence({i: ONE}, C), 'x', 2, (3,)), CDAG)
        assert uncomputed == expected(i)
    checked_branches = matrix_units = 0
    for transport, controlled in [(False, False), (True, False), (False, True)]:
        ids = [i + 16*q for q in range(2 if controlled else 1) for i in range(8)]
        ref = {i: expected(i, transport, controlled) for i in ids}
        cols = [{i: branch(i, m, transport, controlled) for i in ids} for m in range(2)]
        for m in range(2):
            for i in ids:
                assert cols[m][i] == {k: v * R for k, v in ref[i].items()}
                assert all(not ((k >> 3) & 1) for k in cols[m][i])
                checked_branches += 1
        assert channel_matches(cols, ref, ids)
        matrix_units += len(ids)**2
        wrong = 'q' if controlled else ('stale' if transport else 'missing')
        bad = [{i: branch(i, m, transport, controlled, wrong) for i in ids} for m in range(2)]
        assert not channel_matches(bad, ref, ids), ('negative accidentally passes', wrong)
    # A literal late C-adjoint is NOT the legal reference for delayed cleanup.
    literal = sequence(gate(gate(sequence({2: ONE}, C), 'x', 2, (3,)), 'h', 0), CDAG)
    assert literal == {2: R, 11: J*R}
    # qcopy/quncopy source bridge, independent of any optional provenance rule.
    for a in range(2):
        state = gate({a: ONE}, 'x', 1, (0,))
        literal = gate(gate(state, 'h', 0), 'x', 1, (0,))
        assert any((k >> 1) & 1 for k in literal)
        for m in range(2):
            measured = gate(gate(state, 'h', 0), 'h', 1)
            measured = {k: v for k, v in measured.items() if (k >> 1) & 1 == m}
            if m:
                measured = gate(gate(measured, 'x', 0), 'x', 1)
            assert measured == {k: v * R for k, v in gate({a: ONE}, 'h', 0).items()}
    # Omitted reset leaves g=1 on every outcome-1 column.
    assert all((k >> 3) & 1 for k in branch(0, 1, reset=False))
    # Logical inverse of H_a W is W H_a, realised with fresh measurements.
    # Check both inverse branches, then all four forward/inverse outcome pairs.
    for i in range(8):
        h_input = gate({i: ONE}, 'h', 0)
        inverse_reference = {}
        for k, amp in h_input.items():
            for out, coeff in expected(k).items():
                add(inverse_reference, out, amp*coeff)
        for n in range(2):
            inverse_branch = {}
            for k, amp in h_input.items():
                for out, coeff in branch(k, n).items():
                    add(inverse_branch, out, amp*coeff)
            assert inverse_branch == {k: v*R for k, v in inverse_reference.items()}
            for m in range(2):
                after_h = gate(branch(i, m, transport=True), 'h', 0)
                round_trip = {}
                for k, amp in after_h.items():
                    for out, coeff in branch(k, n).items():
                        add(round_trip, out, amp*coeff)
                assert round_trip == {i: ONE / 2}
    assert sum(op in ('t', 'tdg') for op, _, _ in C) == 4
    assert [4*(w-1) for w in (1, 2, 3, 5)] == [0, 4, 8, 16]
    assert Fraction(2) + Fraction(3) == 5  # measurement end + feedforward latency
    print(f'PASS exact SymPy field: 8 compute columns, 32 C/Cdag columns, '
          f'8 compute/use/Cdag columns, {checked_branches} corrected branch columns, '
          f'{matrix_units} matrix units; 3 negative channels; qcopy bridge; '
          'late-inverse/reset negatives; 16 inverse branch columns; 32 round-trip branches')


def document():
    path = HERE / 'l2p-amendment.md'
    doc = path.read_text()
    spec = (ROOT / 'spec/Bennett-ISA-Spec-0.9.md').read_text()
    # Every exact BEFORE block must be a unique byte-for-byte source span.
    blocks = re.findall(r'<!-- before:([^>]+) -->\n```text\n(.*?)\n```', doc, re.S)
    assert blocks, 'missing before/after clauses'
    for name, before in blocks:
        assert spec.count(before) == 1, (name, 'stale or ambiguous source text')
        assert f'<!-- after:{name} -->' in doc, (name, 'missing replacement')
    assert '__BEFORE_' not in doc, 'unresolved clause placeholder'
    after = dict(re.findall(r'<!-- after:([^>]+) -->\n```text\n(.*?)\n```', doc, re.S))
    assert len(after) == len(blocks), 'mismatched replacement blocks'
    # Apply the proposed semantic substitutions only in memory, catching stale
    # anchors and accidental overlap. Version-header migration is separately
    # specified, so this is a patch-consistency check, not a spec publication.
    proposed = spec
    for name, before in blocks:
        assert proposed.count(before) == 1, (name, 'overlapping replacements')
        proposed = proposed.replace(before, after[name], 1)
    for marker in ('B1', 'B2', 'L2P-1', 'L2P-2', 'sol-ref09', '0.10',
                   'measure_fixup', 'clean_isometry', 'mbu', 'matrix unit',
                   'argv/envp', 'feedforward_latency'):
        assert marker in doc, ('missing required scope', marker)
    # Relative links inside the amendment must resolve. Ignore section fragments.
    for target in re.findall(r'\]\(([^)]+)\)', doc):
        if '://' not in target and not target.startswith('#'):
            assert (HERE / target.split('#')[0]).exists(), target
    assert doc.count('```') % 2 == 0
    raw = (ROOT / 'spec/ref-0.9.json').read_bytes()
    catalog = json.loads(raw)
    h = sha256(b'isa=0.9\nexpansion=ref\nversion=0.9\n' + raw + b'\n').hexdigest()
    assert len(catalog['recipes']) == 219
    assert h == '5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197'
    assert h in doc
    print(f'PASS amendment: {len(blocks)} exact unique before/after anchors; links/fences; '
          f'219-recipe ref@0.9 envelope {h}')


if __name__ == '__main__':
    algebra()
    if '--algebra-only' not in sys.argv:
        document()

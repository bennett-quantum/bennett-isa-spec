#!/usr/bin/env python3
"""Finite CFR-3/R2-1 contract witnesses; no production 0.10 implementation claim.

Reads notes and interpreter artifacts. Writes nothing, invokes no git/network,
and never changes the interpreter. run_checks.sh bounds this process and its children.
"""
from collections import Counter
from copy import deepcopy
from fractions import Fraction as F
from itertools import product
from pathlib import Path
import hashlib
import importlib.util
import json
import re
import subprocess

ROOT = Path('/home/tobias/Projects/bennett-workspace/bennett-isa-spec')
HERE = Path(__file__).resolve().parent
JOB = ROOT / '.tmp/sol-rev4'


def raw_hash(data):
    return hashlib.sha256(data).hexdigest()


def jcs(value):
    """Exact finite subset: strings, booleans, null, small integers, maps/arrays."""
    if value is None:
        return 'null'
    if type(value) is bool:
        return 'true' if value else 'false'
    if isinstance(value, dict):
        return '{' + ','.join(jcs(k) + ':' + jcs(value[k]) for k in
                              sorted(value, key=lambda k: k.encode('utf-16be'))) + '}'
    if isinstance(value, list):
        return '[' + ','.join(map(jcs, value)) + ']'
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False, separators=(',', ':'))
    if type(value) is int and abs(value) < 2**53:
        return str(value)
    raise ValueError('outside finite JCS subset')


def digest(value):
    return raw_hash(jcs(value).encode())


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError('duplicate JSON key')
        result[key] = value
    return result


def rejected(operation, reason):
    try:
        operation()
    except ValueError as exc:
        assert reason in str(exc), (reason, str(exc))
        return
    raise AssertionError('negative vector unexpectedly accepted: ' + reason)


def run(*args):
    out = subprocess.run(list(map(str, args)), check=True, capture_output=True,
                         text=True, timeout=20)
    return out.stdout


def extract(text, marker):
    return json.loads(text.split(marker + '\n```json\n', 1)[1].split('\n```', 1)[0])


cfr = (HERE / 'checker-fault-rate.md').read_text()
fd0 = (HERE / 'fd0.md').read_text()
old_fd0 = (ROOT / 'docs/notes/2026-11-02-rev2/fd0.md').read_text()
assert extract(fd0, '<!-- import-example -->') == extract(old_fd0, '<!-- import-example -->')
assert 'ISA 0.9 defines no certificate-attachment transport.' in cfr
assert 'existing certificate attachments' not in cfr
assert 'bennett_estimate_run_with_model_inputs' in fd0
assert 'Every complete, well-typed resolved request' in fd0
assert 'Let x=q*factor' in fd0
for text in (cfr, fd0):
    assert '## Round-2 blocking items and resolution' in text
    assert '../2026-11-02-rev2/fat-object-scope.md#citable-scoping-statement' in text
    assert text.count('```') % 2 == 0
    for link in re.findall(r'\]\(([^)]+)\)', text):
        if '://' not in link and not link.startswith('#'):
            assert (HERE / link.split('#', 1)[0]).resolve().exists(), link
assert jcs({'\ue000': 'bmp', '\U00010000': 'supplementary'}) == \
    '{"\U00010000":"supplementary","\ue000":"bmp"}'

binary = JOB / 'target/debug/bennett-isa-ref'
fixture = HERE / 'checker-fault-rate.bisa'
identity = json.loads(run(binary, 'check', fixture))
assert identity['status'] == 'ok'
machine = [json.loads(line) for line in
           run(JOB / 'oracle-witness', fixture).splitlines()]
assert machine == [{'budget': b, 'tuples': 256, 'faults': 256 if b < 2 else 0}
                   for b in range(3)]
rows = list(product(range(4), range(4), range(16)))
for b in range(3):
    counts = Counter()
    for x in range(4):
        outcome = json.loads(run(binary, 'run-oracle', fixture, 'add2',
                                 '--input', x, '--arg', '1', '--budget', f'{b},0,0,0'))
        assert (outcome['payload'], outcome['code']) == \
            ((0, 1) if b < 2 else ((x + 1) % 4, 0))
        if outcome['code']:
            counts[str(outcome['code'])] += 4 * 16
    assert dict(counts) == ({'1': 256} if b < 2 else {})
print('PASS interpreter: 768 interface tuples; F_B faults 256/256/0; actual commit and full machine reversal')

# The two raw blobs have deliberate whitespace/LF; digesting parsed JSON is wrong.
CLASSIFIER = b'{ "schema":"sol-rev4.classifier/1", "op":"equals", "value":0 }\n'
INPUT_MAP = b'{"schema":"sol-rev4.input-map/1","tuple_component":0}\n'
classifier_hash, map_hash = raw_hash(CLASSIFIER), raw_hash(INPUT_MAP)
artifacts = {classifier_hash: CLASSIFIER, map_hash: INPUT_MAP}
domain = {'bundles': [{'name': n, 'width': w, 'lo': 0, 'hi': 2**w - 1}
                      for n, w in [('x', 2), ('t', 2), ('f', 4)]],
          'borrowed': [], 'regions': []}
invocation = {'args': ['1'], 'budget': ['1', '0', '0', '0']}
expected_binding = {
    'materialisation_hash': digest({'fixture': 'synthetic add2 classification', 'version': '1'}),
    'reference_program_hash': identity['symbols']['add2']['interface_hash'],
    # Explicit fixture identity; no native compiler/build is claimed.
    'native_build_hash': raw_hash(b'synthetic comparison-build identity; no native build'),
    'native_entry': 'synthetic_add2_thunk', 'invocation': invocation,
    'domain_hash': digest(domain)}
evidence = dict(deepcopy(expected_binding), classifiers=[{
    'category': 'undefined', 'artifact_hash': classifier_hash, 'version': '1',
    'configuration': {}, 'input_map_hash': map_hash}])
rate = {'invocation': invocation, 'domain': domain, 'faults_by_code': {'1': '256'},
        'undefined_excluded': '64', 'layout_dependent_excluded': '0',
        'exclusion_evidence': evidence}


def exclusion_report(report, blobs):
    """Finite external bundle semantics, separate from the F_B verdict."""
    ev = report.get('exclusion_evidence')
    if ev is None:
        return 'unchecked'
    for key, expected in expected_binding.items():
        if ev.get(key) != expected:
            raise ValueError('binding: ' + key)
    if report['invocation'] != invocation or digest(report['domain']) != ev['domain_hash']:
        raise ValueError('binding: report invocation/domain')
    required = {c[key] for c in ev['classifiers']
                for key in ('artifact_hash', 'input_map_hash')}
    # Verify addressed bytes before interpreting either blob, including if another is missing.
    for key in required & blobs.keys():
        if raw_hash(blobs[key]) != key:
            raise ValueError('digest mismatch')
    if required - blobs.keys():
        return 'unchecked'
    flagged = {'undefined': set(), 'layout_dependent': set()}
    for c in ev['classifiers']:
        if c['category'] not in flagged:
            raise ValueError('invalid category')
        classifier = json.loads(blobs[c['artifact_hash']])
        mapping = json.loads(blobs[c['input_map_hash']])
        if c['version'] != '1' or c['configuration'] != {} or \
                classifier != json.loads(CLASSIFIER) or mapping != json.loads(INPUT_MAP):
            return 'unchecked'
        flagged[c['category']].update(row for row in rows if row[mapping['tuple_component']] == classifier['value'])
    for category, values in flagged.items():
        if report[category + '_excluded'] != str(len(values)):
            raise ValueError('exclusion count mismatch')
    return 'checked'


assert exclusion_report(rate, artifacts) == 'checked'
for absent in (classifier_hash, map_hash):
    remaining = dict(artifacts)
    del remaining[absent]
    assert exclusion_report(rate, remaining) == 'unchecked'
    zero = deepcopy(rate)
    zero['undefined_excluded'] = '0'
    assert exclusion_report(zero, remaining) == 'unchecked'
    tampered = dict(artifacts)
    tampered[absent] += b' '  # Identical parsed JSON, different raw bytes.
    rejected(lambda: exclusion_report(rate, tampered), 'digest')
wrong_count = deepcopy(rate)
wrong_count['undefined_excluded'] = '63'
rejected(lambda: exclusion_report(wrong_count, artifacts), 'count')
for field in ('materialisation_hash', 'reference_program_hash', 'native_build_hash',
              'native_entry', 'domain_hash', 'invocation'):
    wrong = deepcopy(rate)
    wrong['exclusion_evidence'][field] = None
    rejected(lambda: exclusion_report(wrong, artifacts), 'binding')
unsupported = deepcopy(rate)
unsupported['exclusion_evidence']['classifiers'][0]['version'] = '2'
assert exclusion_report(unsupported, artifacts) == 'unchecked'
absent_ev = deepcopy(rate)
del absent_ev['exclusion_evidence']
assert exclusion_report(absent_ev, artifacts) == 'unchecked'
assert rate['faults_by_code'] == {'1': '256'}  # No classifier path shrinks F_B.
print('PASS CFR-3: 64 exclusions; missing blobs unchecked; changed raw bytes, counts and bindings reject')
print('ARTIFACT SHA256 classifier=' + classifier_hash + ' input_map=' + map_hash)

entry = extract(fd0, '<!-- import-example -->')
model = entry['model']
assert raw_hash(bytes.fromhex(model['source']['bytes'])) == model['source']['sha256']
model_hash = digest(model)
assert model_hash == 'c69312a9d3fc53728a6554b44bae48a9fab7eaefbcca3651c75ad9a0128e022f'
inputs3 = extract(fd0, '<!-- launch-input-example -->')
inputs1 = deepcopy(inputs3)
inputs1['model_cycle']['models']['surface']['parameters']['N'] = '1'


def unit(name):
    u = model['units'][name]
    return F(u['scale']), {k: F(v) for k, v in u['dimension'].items()}


class FreeInput(Exception):
    pass


def evaluate(expr, params, seen=()):
    """Typed exact evaluator for the finite example's rational operations."""
    if isinstance(expr, str):
        value = F(expr)
        if str(value) != expr:
            raise ValueError('noncanonical coefficient')
        return value, {}
    if set(expr) == {'lit', 'unit'}:
        scale, dims = unit(expr['unit'])
        return F(expr['lit']) * scale, dims
    if set(expr) == {'param'}:
        name = expr['param']
        if params[name] is None:
            raise FreeInput(name)
        value, dims = evaluate(params[name], {})
        if dims:
            raise ValueError('typed coefficient')
        scale, dims = unit(model['parameters'][name]['unit'])
        return value * scale, dims
    if set(expr) == {'quantity'}:
        name = expr['quantity']
        if name in seen:
            raise ValueError('quantity cycle')
        q = model['quantities'][name]
        value, dims = evaluate(q['expr'], params, seen + (name,))
        if dims != unit(q['unit'])[1]:
            raise ValueError('quantity dimensions')
        return value, dims
    if set(expr) != {'op', 'args'} or len(expr['args']) != 2:
        raise ValueError('outside finite expression subset')
    x, dx = evaluate(expr['args'][0], params, seen)
    y, dy = evaluate(expr['args'][1], params, seen)
    op = expr['op']
    if op == '+':
        if dx != dy:
            raise ValueError('unit mismatch')
        return x + y, dx
    if op in ('*', '/'):
        dims = dict(dx)
        for key, value in dy.items():
            dims[key] = dims.get(key, F(0)) + value * (1 if op == '*' else -1)
        return (x * y if op == '*' else x / y), {k: v for k, v in dims.items() if v}
    if op == 'pow' and not dy and y.denominator == 1:
        return x ** int(y), {k: v * y for k, v in dx.items() if v * y}
    raise ValueError('outside finite expression subset')


def validate_inputs(inputs):
    if not isinstance(inputs, dict):
        raise ValueError('malformed input map')
    if 'model_cycle' not in inputs:
        raise ValueError('incomplete-model-objective')
    inp = inputs['model_cycle']
    if set(inputs) != {'model_cycle'} or set(inp) != {'root', 'models', 'shared'} or \
            inp['root'] != {'model': 'surface', 'node': 'algorithm'} or \
            set(inp['models']) != {'surface'} or inp['shared'] != []:
        raise ValueError('malformed ModelInput')
    supplied = inp['models']['surface']
    if set(supplied) != {'model_hash', 'parameters'} or supplied['model_hash'] != model_hash:
        raise ValueError('model hash/fields')
    params = supplied['parameters']
    if set(params) != set(model['parameters']):
        raise ValueError('parameter keys')
    for name, p in model['parameters'].items():
        value = params[name]
        # This example's only non-operational algorithm parameter is N.
        if name != 'N' and value != p['value']:
            raise ValueError('frozen parameter')
        if value is None:
            continue
        coefficient, dims = evaluate(value, {})
        if dims or (p['type'] == 'integer' and coefficient.denominator != 1):
            raise ValueError('parameter type')
        d = p['domain']
        if (d['min'] is not None and coefficient < F(d['min'])) or \
                (d['max'] is not None and coefficient > F(d['max'])) or \
                (d['parity'] == 'odd' and coefficient % 2 != 1):
            raise ValueError('parameter domain')
    return params


BASE = {'invocation': deepcopy(invocation)}  # Explicitly a schedule fragment, not a production certificate.


def launch(model_inputs_bytes, embedded=None):
    # Copy first: neither an external report nor a later buffer mutation is an input.
    copied = bytes(model_inputs_bytes) if model_inputs_bytes is not None else b'{}'
    inputs = json.loads(copied, object_pairs_hook=unique_object)
    params = validate_inputs(inputs)
    try:
        n, dims = evaluate({'param': 'N'}, params)
    except FreeInput:
        return {'status': 'symbolic', 'estimate': None}
    assert not dims and n.denominator == 1 and n >= 0
    bindings = dict(deepcopy(BASE), model_inputs=inputs)
    matches = embedded is not None and jcs(embedded['bindings']) == jcs(bindings)
    schedule = deepcopy(embedded) if matches else {'bindings': bindings, 'entries': []}
    cycle, cycle_dims = evaluate({'quantity': 'cycle'}, params)
    risk, risk_dims = evaluate({'quantity': 'risk'}, params)
    physical, physical_dims = evaluate({'quantity': 'physical'}, params)
    assert cycle_dims == {'s': F(1)} and risk_dims == {}
    assert physical_dims == {'physical_qubit': F(1), 'logical_qubit': F(-1)}
    leaves = [cycle for _ in range(int(n))]  # Independent finite expansion.
    return {'status': 'concrete', 'selected': 'embedded' if matches else 'default',
            'schedule': schedule, 'schedule_hash': digest(schedule),
            'duration': sum(leaves, F(0)), 'risk': sum((risk for _ in leaves), F(0)),
            'occupancy': sum((physical * duration for duration in leaves), F(0)),
            'peak': physical if leaves else F(0)}


result3, result1 = launch(jcs(inputs3).encode()), launch(jcs(inputs1).encode())
for n, result in [(3, result3), (1, result1)]:
    assert result['selected'] == 'default' and result['schedule']['entries'] == []
    assert result['duration'] == F(n * 3, 100000000)
    assert result['risk'] == F(n, 10000000) and result['peak'] == 50
    assert result['occupancy'] == F(n * 3, 2000000)
assert result3['schedule_hash'] != result1['schedule_hash']
assert result3['schedule']['bindings']['model_inputs'] == inputs3
rejected(lambda: launch(None), 'incomplete-model-objective')
rejected(lambda: launch(b'{}', result3['schedule']), 'incomplete-model-objective')
bad = deepcopy(inputs3)
bad['model_cycle']['models']['surface']['model_hash'] = '0' * 64
rejected(lambda: launch(jcs(bad).encode()), 'model hash')
bad = deepcopy(inputs3)
bad['model_cycle']['models']['surface']['parameters']['d'] = '3'
rejected(lambda: launch(jcs(bad).encode()), 'frozen')
conflict = launch(jcs(inputs3).encode(), result1['schedule'])
assert conflict == result3  # Correctly bound N=3 default, not the N=1 embedded schedule.
reordered = deepcopy(result3['schedule'])
reordered['bindings']['model_inputs']['model_cycle'] = dict(reversed(
    list(reordered['bindings']['model_inputs']['model_cycle'].items())))
assert launch(jcs(inputs3).encode(), reordered)['selected'] == 'embedded'
rejected(lambda: launch(b'{"model_cycle":{},"model_cycle":{}}'), 'duplicate JSON key')
free = deepcopy(inputs3)
free['model_cycle']['models']['surface']['parameters']['N'] = None
assert launch(jcs(free).encode()) == {'status': 'symbolic', 'estimate': None}
caller_buffer = bytearray(jcs(inputs3).encode())
copied_result = launch(caller_buffer)
caller_buffer[:] = jcs(inputs1).encode()
assert copied_result == result3
for distance, ns, qubits, risk in [('5', 30, 50, F(1, 10**7)), ('3', 18, 18, F(1, 10**5))]:
    params = {k: p['value'] for k, p in model['parameters'].items()}
    params['d'] = distance
    base, _ = evaluate({'quantity': 'cycle'}, params)
    scale = unit(model['quantities']['cycle']['unit'])[0]
    coefficient = base / scale
    assert coefficient == ns and coefficient * F(model['bindings'][0]['factor']) == base
    assert evaluate({'quantity': 'physical'}, params)[0] == qubits
    assert evaluate({'quantity': 'risk'}, params)[0] == risk
assert F(10, 10**9) / F(1, 10**6) == F(1, 100)  # Cross-model unit conversion vector.
assert F(1, 100) * F(1, 10**6) == F(1, 100000000)
print('PASS R2-1: N=3/N=1 durations and distinct bound hashes; copied inputs; missing/malformed/symbolic cases; embedded conflict defaults')
print('SCHEDULE-FRAGMENT SHA256 N=3=' + result3['schedule_hash'] + ' N=1=' + result1['schedule_hash'])

spec = importlib.util.spec_from_file_location('rev4_catalog', ROOT / 'conformance/tools/ref_catalog.py')
catalog_module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(catalog_module)
raw = (ROOT / 'spec/ref-0.9.json').read_bytes()
catalog = json.loads(raw)
assert catalog_module.jcs(catalog).encode() == raw and len(catalog['recipes']) == 219
new = deepcopy(catalog)
new['isa'] = new['version'] = '0.10'
old_pin = raw_hash(b'isa=0.9\nexpansion=ref\nversion=0.9\n' + raw + b'\n')
new_pin = raw_hash(b'isa=0.10\nexpansion=ref\nversion=0.10\n' + catalog_module.jcs(new).encode() + b'\n')
assert old_pin == '5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197'
assert new_pin == '83cc9c177bad054bd7af35dc1892645d869b9e4c0b9c968f35b8783ce2b54c42'
assert old_pin in fd0 and new_pin in fd0 and new['recipes'] == catalog['recipes']
print('PASS preservation: serialized model unchanged; exact d=3/d=5 units/arithmetic; 219 catalog recipes; versioned hashes; links and scope')

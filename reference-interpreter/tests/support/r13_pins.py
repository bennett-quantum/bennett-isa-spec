"""Independent §9.2 derivations; no interpreter or rehash helper is used."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
def sha(s):
    return hashlib.sha256(s.encode()).hexdigest()
def j(v):
    return json.dumps(v, sort_keys=True, separators=(',', ':'))
inc = (ROOT / 'fixtures/inc8.bisa').read_text()
sig = json.loads(inc.split('section symbols\n')[1].splitlines()[0])['0']['signature']
body = inc.split('section functions\n')[1]
env = 'isa=0.8\nptr_width=32\n' + j(sig) + '\n' + body
assert sha(env) == '8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93'
cases = []
def record(name, obj, envelope, pin=None, object_pin=None):
    h = sha(envelope)
    if pin:
        assert h == pin, (name, h)
    oh = sha(obj)
    if object_pin:
        assert oh == object_pin, (name, oh)
    cases.append(dict(name=name, object=obj, envelope=envelope, interface_hash=h, object_hash=oh))
for version, pin in [(7, 'a63cb1dd51929e217eb8e16e261707a4103a92ff54c20caf2bb614ba73e65734'), ('0.8', '3bc26d4621ba2411e29615a854bce6bb4299fe85a68df89479c325fcb0836b11')]:
    record('service-' + str(version), inc + '\nsection devices\n' + j(dict(models={}, services={'org.bennett.test': dict(version=version)})) + '\n', env, object_pin=pin)
layout = dict(A_tab=0, H=6, S_cells=0, globals={}, heap_base=48, heap_limit_max=0, interface_regions={}, null_size=1, ptr_width=32)
for key, pin, oh in [('S_cells', 'dc736293ea943fd3d8f902ae1589e3e1cbc461ca42bd9229518b33ec65b724b1', 'f581d1d9418da1213057fb7e6cc95c3d35619aa478f9c6fc275502a5371bd44a'), ('A_tab', 'dc2c21a6b7c2a0fea8a6881564f80b44c3742d681e71576ec6f4d66fa058f683', 'cf57de219961a2cc9e02b768a8b59b984b2ba2c5aa40b48c333b1ffcbb405ef0')]:
    l = dict(layout, **{key: str(2**64 - 1)})
    # Empty maps emit no leaves. These are exactly the seven scalar lines.
    leaves = ''.join('layout ' + k + ' ' + j(v) + '\n' for k, v in sorted(l.items()) if not isinstance(v, dict))
    record(key, inc.replace(sha(env), pin) + '\nsection layout\n' + j(l) + '\n', env + leaves, pin, oh)
asig = dict(abi='0.8', args=[], bounds=[], **{'in': []}, kind='action', mem=[], mut=[1], tier='q')
abody = 'action @a (t:wire mut) () () () ()\n^entry():\nx t\n'
aenv = 'isa=0.8\nptr_width=32\n' + j(asig) + '\n' + abody
ah = sha(aenv)
assert ah == '2fcd5b3d60b42ba6092447586a2157bf44ba3a6479ab2eb319bb67665feab8be'
header = inc.split('\n\nsection symbols')[0]
def action(l, disk=False):
    syms = {'0': dict(interface_hash=ah, kind='action', name='a', signature=asig)}
    if disk:
        dsig = dict(abi='0.8', kind='data', size=512, tier='-')
        denv = 'isa=0.8\nptr_width=32\n' + j(dsig) + '\ndisk ' + '00'*512 + '\n'
        syms['1'] = dict(interface_hash=sha(denv), kind='data', name='disk', signature=dsig)
    text = header + '\n\nsection symbols\n' + j(syms) + '\n\nsection functions\n' + abody
    if disk:
        text += '\nsection data\ndisk ' + '00'*512 + '\n'
    return text + '\nsection layout\n' + j(l) + '\n'
record('block-number', action(dict(layout, block_image=42)), aenv, object_pin='3531fb9b077a90c0fef79688391192df706d99c369a21b0a9a1bc8e35ee75642')
record('block-missing', action(dict(layout, block_image='missing')), aenv, object_pin='04e497abf45bd7012b5341da012a51c7f79a47b5efce0fcc5a7f35af1639b7de')
record('block-absent', action(layout), aenv, object_pin='ddc5d6729a98484acc2d5e8feb585c155799293bd36f31fd96cbc510f77666f0')
record('block-disk', action(dict(layout, block_image='disk'), True), aenv)
(ROOT / 'support/r13_cases.json').write_text(json.dumps(cases, indent=2) + '\n')
print(f'{len(cases)} records; all supplied interface and object pins match')

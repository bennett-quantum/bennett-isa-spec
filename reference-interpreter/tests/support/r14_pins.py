"""Hand-derived R14 envelopes and exact reviewer witness bytes; no interpreter."""
import hashlib
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
W = ROOT / 'support/r14-witnesses'
def sha(text):
    return hashlib.sha256(text.encode()).hexdigest()
def j(v):
    return json.dumps(v, sort_keys=True, separators=(',', ':'))
ah = '2fcd5b3d60b42ba6092447586a2157bf44ba3a6479ab2eb319bb67665feab8be'
ih = '8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93'
# Reference-free singleton SCC: its closure is exactly its interface hash.
# Actions exclude layout. Inc8 has no layout or dependencies. LF on every line.
aenv = 'isa=0.8\nptr_width=32\n' + j(dict(abi='0.8',args=[],bounds=[],**{'in':[]},kind='action',mem=[],mut=[1],tier='q')) + '\naction @a (t:wire mut) () () () ()\n^entry():\nx t\n'
ienv = 'isa=0.8\nptr_width=32\n' + j(dict(abi='0.8',args=[],fault=4,**{'in':[8]},kind='oracle',mem=[],tgt=8,tier='q')) + '\noracle @inc8 (x:wire[8] in) (t:wire[8]) (f:wire[4]) () ()\nregs { one:reg[8], r:reg[8] }\n^entry():\none = lift.const.i8 1\nr = lift.add x one\nlift.retval r\n'
assert sha(aenv) == ah
assert sha(ienv) == ih
out=[]
for p in sorted(W.glob('*.bisa')):
    text=p.read_text()
    base, payload=text.split('\nsection certificate\n')
    cert=json.loads(payload)
    symbol=json.loads(text.split('section symbols\n')[1].splitlines()[0])['0']
    is_y=symbol['kind']=='oracle'
    h=ih if is_y else ah
    env=ienv if is_y else aenv
    object_pin='8093da48bad04d6564325faf01ea1304a84b5ac5a120e685eee9951f5daec5d9' if is_y else 'ddc5d6729a98484acc2d5e8feb585c155799293bd36f31fd96cbc510f77666f0'
    # §9.2 excludes certificate and uses its own leading section-boundary LF.
    assert sha(base) == object_pin, (p.name,sha(base))
    symbol=json.loads(text.split('section symbols\n')[1].splitlines()[0])['0']
    assert symbol['interface_hash']==h
    assert cert['bindings']['object_hash']==object_pin
    assert cert['bindings']['interface_hash']==cert['bindings']['closure_hash']==h
    if p.name=='r14-Y-number.bisa':
        assert sha(text)=='578c913bb276d1446e2e038af0ab0a77017d9abe5ec351e3e20b69c307068e11'
    out.append(dict(name=p.stem,envelope=env,interface_hash=h,closure_hash=h,object_hash=object_pin,file_sha256=sha(text)))
(ROOT/'support/r14_pins.json').write_text(json.dumps(out,indent=2)+'\n')
print(f'{len(out)} exact reviewer witnesses: literal envelopes, interface/closure/object pins match')

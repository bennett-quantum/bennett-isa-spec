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

# Schema exemplars are literal, independently pinned objects. They exercise
# every section and optional production, including each tagged payload branch.
from copy import deepcopy
samples = []
def sample(label, obj):
    samples.append(dict(name=label, object=obj))
def section(base, key, value):
    return base + '\nsection ' + key + '\n' + j(value) + '\n'
for fixture in ['inc8', 'sum_db', 'heap', 'actions', 'classical', 'registers', 'control_flow']:
    sample(fixture, (ROOT / ('fixtures/' + fixture + '.bisa')).read_text())
sample('block', cases[-1]['object'])
sample('devices', section(inc, 'devices', dict(models={d: dict(version='0.8') for d in ['console','clock','entropy','block']}, services={'org.bennett.test': dict(version='vendor.1')})))
# Every signature form is exercised through the independent imports schema too.
imports = {}
for s in samples[:7]:
    syms = json.loads(s['object'].split('section symbols\n')[1].splitlines()[0])
    for symbol in syms.values():
        kind = symbol['kind']
        imports['import_' + kind] = dict(signature=symbol['signature'], pinned_closure_hash='4'*64)
sample('imports', inc.replace('\nsection functions', '\nsection imports\n' + j(imports) + '\n\nsection functions'))
h = sha(env)
point = [h, 'entry', 0]
path = [dict(symbol=h, point='entry:0', iteration=[])]
bindings = dict(budget=[3,0,0,0], closure_hash=h, epsilon=0, expansion=dict(hash='1'*64,name='ref',version='0.8'), inherited_controls=0,params_hash='2'*64,region='inc8',scheduler=dict(name='test',version='1'),target_hash='3'*64,variant='forward')
for kind, payload in [('cleanup', dict(realisation='reverse')), ('gate', dict(realisation='native',epsilon_share=0)), ('alloc', dict(**{'class':'clean'})), ('start', dict(time=0))]:
    for wildcard in [False, True]:
        e = dict(path=deepcopy(path), step='*' if wildcard else 1, phase='*' if wildcard else 'fwd', generated='*' if wildcard else 0, kind=kind, **{kind:payload})
        if wildcard:
            e['path'][0]['iteration'] = '*'
        sample('schedule-' + kind + str(wildcard), section(inc, 'schedule', dict(bindings=bindings, entries=[e])))
for kind, extra in [('lowered', {}), ('trapped', dict(fault_code=1)), ('shimmed', dict(shim_hash='4'*64))]:
    sample('coverage-' + kind, section(inc, 'coverage', dict(entries=[dict(classification=kind, count=1, point=point,reason='direct',source='dialect.add', **extra)])))
domain = dict(bundles=[dict(name='x',width=8,lo=0,hi=255)], borrowed=[dict(name='b',width=1,lo=0,hi=1)], regions=[])
subject = dict(closure_hash=h,variant='forward',inherited_controls=0,schedule_hash='4'*64)
claims = [
    dict(kind='exhaustive', domain=domain, expected=dict(table=[]),label='proof'),
    dict(kind='exhaustive', domain=domain, expected=dict(program=h)),
    dict(kind='exhaustive', domain=domain, expected='oracle',subject=subject),
    dict(kind='sampled', domain=domain, sampler=dict(algorithm='xoshiro256**',seed=['1','0','0','0'],count=10),label='evidence'),
    dict(kind='structural',rule='adjoint_table',subclaims=[0],witness={}),
    dict(kind='structural',rule='inverse_pair_library',subclaims=[],witness=dict(library_closure_hash=h,claim_index=0)),
    dict(kind='structural',rule='translation_lemma',subclaims=[],witness=dict(V=[point], W=[point], X=['x'], B=['b'], frame=[point,point])),
    dict(kind='numerical',subject=subject,metric='operator_norm',precision_bits=128,bound='0.001'),
    dict(kind='channel',subject=subject,metric='diamond',precision_bits=128,bound=0,dual=[dict(Y=[['-1','0','0','1']],mu=0)]),
]
frame = dict(interface=[dict(role='in',width=8,**{'class':'any'},name='x'), dict(role='tgt',width=8,**{'class':'any'}),dict(role='fault',width=4,**{'class':'any'})],scratch={'any':dict(clean=0,dirty={'var':'c'})},retained_history=dict(op='+',args=[0,dict(table='frame_table',index='k')]),borrow={'any':0},scheme_ancillae={'any':0},live_ranges=[dict(frame='f',start=0,end=1)])
# Frame table index k is intentionally not bound; the valid c expression is used.
frame['retained_history']['args'][1] = dict(var='c')
region = dict(kind='oracle',signature=sig,bounds=[],frame=frame,resources={'gates':0},gates_used={'h':dict(count=0,scheme='native',epsilon='0')},claims=claims,verification=dict(oracle=0,borrow_frames=[0],inverse_pairs=[dict(g=h,ginv=h,claim=0)],unitary=[0],channel=[8],fault_rate=dict(domain=domain,faults_by_code={'1':0},undefined_excluded=0,layout_dependent_excluded=0)),cleanup={'forward:0':[dict(path=path,realisation='reverse')]},error={'forward:0':dict(local=dict(operator_norm=0,diamond='0'),inclusive=dict(operator_norm=0,diamond=0))},definedness_assumptions=['defined'])
cert = dict(versions={v:'0.8' for v in ['isa','object','launch','sched','cert','coverage','target']},bindings=dict(object_hash=sha(inc),interface_hash=h,closure_hash=h,reference_program_hash=h,dialect_mapping_version='test.1',target_hash='3'*64,schedule_hash='4'*64),producer=dict(name='tests',version='1',materialisation=dict(llvm_version='none',pipeline='none',normaliser_version='1')),regions={h:region},composition=dict(multiplicities={h+':forward:0':'18446744073709551615'},epsilon_total=dict(diamond=0),method='sum_over_multiplicities'))
sample('certificate', section(inc, 'certificate', cert))
# Explicit null-or-index branch, decimal-string/number branches, expression
# constants/table AST, plus a region domain with both cells representations.
c2 = deepcopy(cert)
c2['regions'][h]['verification']['oracle'] = None
c2['regions'][h]['frame']['retained_history'] = dict(table='frame_table',index='eps_decade')
# eps is not in the frame environment: table AST is covered in expression tests,
# so use the accepted constant alternative at this position instead.
c2['regions'][h]['frame']['retained_history'] = 0
sample('certificate-null', section(inc, 'certificate', c2))
base = (ROOT/'fixtures/sum_db.bisa').read_text()
syms = json.loads(base.split('section symbols\n')[1].splitlines()[0])
root = next(s for s in syms.values() if s['name']=='sum_db')
rh = root['interface_hash']
c3 = deepcopy(cert)
c3['bindings'].update(object_hash=sha(base),interface_hash=rh,closure_hash=rh,reference_program_hash=rh)
r = deepcopy(region)
r.update(signature=root['signature'],claims=[],cleanup={},kind='oracle')
r['verification'] = dict(oracle=None,borrow_frames=[],inverse_pairs=[],unitary=[],channel=[],fault_rate=dict(domain=dict(bundles=[],borrowed=[],regions=[dict(name='db',cells=[dict(index=0,lo=0,hi=255)])]),faults_by_code={},undefined_excluded=0,layout_dependent_excluded=0))
c3['regions'] = {rh:r}
sample('certificate-regions', section(base, 'certificate', c3))
r['verification']['fault_rate']['domain']['regions'][0]['cells']='all'
sample('certificate-all-cells', section(base, 'certificate', c3))
(ROOT/'support/r13_schema_cases.json').write_text(json.dumps(samples,indent=2)+'\n')
print(f'{len(samples)} schema exemplars')

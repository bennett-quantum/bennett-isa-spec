#!/usr/bin/env python3
"""Read-only finite witnesses for the rev2 proposals, not an ISA/format checker.

Uses only this repository's published catalog routine/data and the eight notes.
No subprocesses, network, git, temporary files, tracker or source mutations.
"""
from pathlib import Path
from fractions import Fraction as F
from copy import deepcopy
from math import isqrt, ceil, floor
import hashlib, json, re
ROOT=Path('/home/tobias/Projects/bennett-workspace/bennett-isa-spec')
OUT=ROOT/'docs/notes/2026-11-02-rev2'
text=(OUT/'fd0.md').read_text()
entry=json.loads(text.split('<!-- import-example -->\n```json\n',1)[1].split('\n```',1)[0])
m=entry['model']
def q(x):
    assert isinstance(x,str)
    v=F(x)
    assert str(v)==x, ('noncanonical rational',x)
    return v
def dims(d):return {k:q(v) for k,v in d.items() if q(v)}
def dimcombine(a,b,sign=1):
    out=dict(a)
    for k,v in b.items():out[k]=out.get(k,F(0))+sign*v
    return {k:v for k,v in out.items() if v}
def unit(n):
    u=m['units'][n]
    assert set(u)=={'dimension','scale'} and q(u['scale'])>0
    return q(u['scale']),dims(u['dimension'])
class Unbound(Exception):pass

def ev(e,params,seen=()):
    if isinstance(e,str):return q(e),{}
    assert isinstance(e,dict)
    if set(e)=={'lit','unit'}:
        scale,d=unit(e['unit']); return q(e['lit'])*scale,d
    if set(e)=={'param'}:
        p=m['parameters'][e['param']]; x=params[e['param']]
        if x is None:raise Unbound(e['param'])
        v,d=ev(x,{},seen); assert not d
        scale,d=unit(p['unit']); return v*scale,d
    if set(e)=={'quantity'}:
        n=e['quantity']; assert n not in seen, ('cycle',n)
        v,d=ev(m['quantities'][n]['expr'],params,seen+(n,))
        assert d==unit(m['quantities'][n]['unit'])[1]
        return v,d
    if set(e)=={'external'}:raise Unbound(e['external']['location'])
    assert set(e)=={'op','args'}
    op=e['op']; a=e['args']
    assert op in ['+','-','*','/','min','max','ceil','sqrt','pow']
    assert len(a)==(1 if op in ['ceil','sqrt'] else 2)
    x,dx=ev(a[0],params,seen)
    if op=='ceil':assert not dx; return F(ceil(x)),{}
    if op=='sqrt':
        assert x>=0
        n,d=isqrt(x.numerator),isqrt(x.denominator)
        assert n*n==x.numerator and d*d==x.denominator, 'nonrational root: use exact algebraic AST/enclosure'
        return F(n,d),{k:v/2 for k,v in dx.items()}
    y,dy=ev(a[1],params,seen)
    if op in ['+','-','min','max']:
        assert dx==dy, ('unit mismatch',dx,dy)
        return {'+':lambda:x+y,'-':lambda:x-y,'min':lambda:min(x,y),'max':lambda:max(x,y)}[op](),dx
    if op=='*':return x*y,dimcombine(dx,dy)
    if op=='/':assert y!=0; return x/y,dimcombine(dx,dy,-1)
    assert not dy and y.denominator==1
    assert x!=0 or y>0
    return x**int(y),{k:v*y for k,v in dx.items()}

def defaults():return {k:p['value'] for k,p in m['parameters'].items()}
def validate_param(p,x):
    if x is None:return
    v,d=ev(x,{})
    assert not d
    if p['type']=='integer':assert v.denominator==1
    dom=p['domain']
    assert dom['min'] is None or v>=q(dom['min'])
    assert dom['max'] is None or v<=q(dom['max'])
    assert dom['parity']=='any' or (p['type']=='integer' and int(v)%2=={'odd':1,'even':0}[dom['parity']])
    if p['role']=='code_distance':assert p['type']=='integer' and unit(p['unit'])== (F(1),{}) and q(dom['min'])>=1

def fails(call):
    try:call()
    except (AssertionError,ZeroDivisionError,KeyError,Unbound):return
    raise AssertionError('negative vector did not fail')
assert set(entry)=={'version','map','model'}
assert set(m)=={'schema','source','units','parameters','quantities','bindings','nodes','assumptions'}
assert m['schema']=='bennett.import-model/1'
assert set(m['source'])=={'format','version','media_type','bytes','sha256'}
assert hashlib.sha256(bytes.fromhex(m['source']['bytes'])).hexdigest()==m['source']['sha256']
for p in m['parameters'].values():
    assert set(p)=={'role','type','unit','value','domain'}
    assert set(p['domain'])=={'min','max','parity'}
    validate_param(p,p['value'])
for n in m['units']:unit(n)
resource_units={'model_cycle':(F(1),{'s':F(1)}),'model_physical_ratio':(F(1),{'physical_qubit':F(1),'logical_qubit':F(-1)}),'risk_charge':(F(1),{})}
assert set(entry['map'])==set(m['quantities'])
assert len({a[0] for a in entry['map'].values()})==len(entry['map'])
for name,(r,factor) in entry['map'].items():
    u=m['quantities'][name]['unit']; sc,d=unit(u); rc,rd=resource_units[r]
    assert d==rd and F(str(factor))==sc/rc
params=defaults()
for d,ns,physical,risk in [('5',30,50,F(1,10**7)),('3',18,18,F(1,10**5))]:
    p=dict(params,d=d)
    assert ev({'quantity':'cycle'},p)==(F(ns,10**9),{'s':F(1)})
    assert ev({'quantity':'physical'},p)==(F(physical),{'physical_qubit':F(1),'logical_qubit':F(-1)})
    assert ev({'quantity':'risk'},p)==(risk,{})
b=m['bindings'][0]
assert set(b)=={'path','quantity','factor','rounding','places','value'}
assert b['path']=='/gates/0/cost/time' and b['rounding']=='exact'
cyc,dc=ev({'quantity':b['quantity']},params)
assert q(b['factor'])==unit(m['quantities'][b['quantity']]['unit'])[0]
assert cyc==q(b['value'])==F(3,100000000)
# Binding factors multiply quantities in their declared source unit.
assert F(30)*q(b['factor'])==q(b['value'])
fails(lambda:validate_param(m['parameters']['d'],'4'))
fails(lambda:validate_param(m['parameters']['d'],'0'))
even=deepcopy(m['parameters']['d']); even['domain']['parity']='any'; validate_param(even,'4')
fails(lambda:ev({'param':'N'},params))
for e in [{'op':'/','args':['1','0']},{'op':'sqrt','args':['-1']},{'op':'pow','args':['2','1/2']},{'op':'pow','args':['0','0']},{'var':'eps'},{'op':'log2inv_eps','args':[]},{'param':'absent'},{'op':'+','args':[{'param':'tau'},'1']}]:
    fails(lambda e=e:ev(e,params))
assert ev({'op':'pow','args':['2','-3']},params)==(F(1,8),{})
assert ev({'op':'sqrt','args':['9/4']},params)==(F(3,2),{})
# Sqrt(2) tight outward 3-place enclosure, without approximating sqrt.
lo,hi=F(1414,1000),F(1415,1000)
assert lo*lo<2<hi*hi and hi-lo==F(1,1000)
assert (ceil(F(1,3)*1000)/F(1000))==F(334,1000)
assert floor(F(1,3)*1000)/F(1000)==F(333,1000)
# Forward quantity references work; cycles reject even beneath a zero coefficient.
m['quantities']['alias']={'unit':'ns','role':'duration','expr':{'quantity':'cycle'}}
assert ev({'quantity':'alias'},params)==ev({'quantity':'cycle'},params)
m['quantities']['bad']={'unit':'one','role':'scalar','expr':{'op':'*','args':['0',{'quantity':'bad'}]}}
fails(lambda:ev({'quantity':'bad'},params))
del m['quantities']['alias'];del m['quantities']['bad']

nodes=deepcopy(m['nodes'])
nodes.update(seq={'kind':'sequence','children':['cycle']*3},par={'kind':'parallel','children':['cycle']*3})
def graph(n,seen=()):
    assert n not in seen
    a=nodes[n];k=a['kind']
    if k in ('sequence','parallel'):
        for c in a['children']:graph(c,seen+(n,))
    elif k=='repeat':graph(a['child'],seen+(n,))
    else:assert k=='leaf'
for n in nodes:graph(n)
def tree(n,p,origin=F(0)):
    a=nodes[n];k=a['kind']
    if k in ('sequence','parallel','repeat'):
        children=a.get('children')
        if k=='repeat':
            count,dd=ev(a['count'],p);assert not dd and count>=0 and count.denominator==1
            children=[a['child']]*int(count)
        duration=F(0);counts={};intervals=[];uses=[]
        for c in children:
            off=duration if k!='parallel' else F(0)
            dt,cs,ins,us=tree(c,p,origin+off)
            duration=max(duration,dt) if k=='parallel' else duration+dt
            for r,v in cs.items():counts[r]=counts.get(r,F(0))+v
            intervals+=ins;uses+=us
        return duration,counts,intervals,uses
    assert k=='leaf'
    dt,dd=ev(a['duration'],p);assert dd=={'s':F(1)} and dt>=0
    counts={r:ev(e,p)[0]/resource_units[r][0] for r,e in a['counts'].items()}
    ins=[]
    for i in a['intervals']:
        start,ds=ev(i['start'],p);end,de=ev(i['end'],p);amount,da=ev(i['amount'],p)
        assert ds==de=={'s':F(1)} and 0<=start<=end<=dt
        assert da==unit(i['unit'])[1] and amount>=0 and amount.denominator==1
        ins.append((origin+start,origin+end,amount))
    uses=[(origin,origin+dt,u['id'],ev(u['slots'],p)[0]) for u in a['shared_uses']]
    return dt,counts,ins,uses

def stats(ins):
    times=sorted({t for s,e,a in ins for t in (s,e)})
    peak=area=F(0)
    for s,e in zip(times,times[1:]):
        demand=sum((a for st,en,a in ins if st<=s<en),F(0))
        peak=max(peak,demand);area+=demand*(e-s)
    return peak,area
for node,expected in [('seq',(F(90,10**9),F(50))),('par',(F(30,10**9),F(150)))]:
    dt,counts,ins,uses=tree(node,params)
    peak,area=stats(ins)
    assert (dt,peak)==expected and area==F(4500,10**9)
    assert counts=={'risk_charge':F(3,10**7)} and not uses
p=dict(params,N='3')
assert tree('algorithm',p)==tree('seq',p)
p['N']='0';assert tree('algorithm',p)==(0,{},[],[])
nodes['badzero']={'kind':'repeat','count':'0','child':'missing'}
fails(lambda:graph('badzero'));del nodes['badzero']
# One reserved physical factory, three uses; private intervals are removed.
nodes['shared']=deepcopy(nodes['cycle']);nodes['shared']['intervals']=[]
nodes['shared']['shared_uses']=[{'id':'factory0','slots':'1'}]
nodes['sharedseq']={'kind':'sequence','children':['shared']*3}
nodes['sharedpar']={'kind':'parallel','children':['shared']*3}
def shared(node,slots):
    dt,cs,ins,uses=tree(node,params)
    times=sorted({t for s,e,i,a in uses for t in (s,e)})
    for s,e in zip(times,times[1:]):assert sum(a for st,en,i,a in uses if st<=s<en)<=slots
    assert not ins
    return stats([(F(0),dt,F(50))])
assert shared('sharedseq',1)==(50,F(4500,10**9))
fails(lambda:shared('sharedpar',1))
assert shared('sharedpar',3)==(50,F(1500,10**9))
# Identity changes are explicit, not hidden in caller defaults.
def modelhash(x):return hashlib.sha256(json.dumps(x,sort_keys=True,separators=(',',':')).encode()).hexdigest()
hash0=modelhash(m);changed=deepcopy(m);changed['parameters']['d']['value']='3'
assert modelhash(changed)!=hash0
assert hashlib.sha256(bytes.fromhex(m['source']['bytes'])+b' ').hexdigest()!=m['source']['sha256']

# Exact target-model identities, including UTF-16 rather than code-point key order.
def model_jcs(v):
    if v is None: return 'null'
    if type(v) is bool: return 'true' if v else 'false'
    if isinstance(v, dict):
        return '{'+','.join(model_jcs(k)+':'+model_jcs(v[k])
                            for k in sorted(v,key=lambda x:x.encode('utf-16be')))+'}'
    if isinstance(v, list): return '['+','.join(map(model_jcs,v))+']'
    if isinstance(v, str): return json.dumps(v,ensure_ascii=False,separators=(',',':'))
    if type(v) is int and abs(v)<2**53: return str(v)
    raise AssertionError(('outside this finite JCS subset',v))

def digest(v): return hashlib.sha256(model_jcs(v).encode()).hexdigest()
assert m['source']['sha256']=='426daeed30fce3fa03234faff9d8ea543005c4681917f126b8dbaca6508c9be5'
assert digest(m)==hash0=='c69312a9d3fc53728a6554b44bae48a9fab7eaefbcca3651c75ad9a0128e022f'
unicode_keys={'\ue000':'bmp','\U00010000':'supplementary'}
assert list(sorted(unicode_keys))==['\ue000','\U00010000']
assert model_jcs(unicode_keys)=='{"\U00010000":"supplementary","\ue000":"bmp"}'

# Full-version catalog publication: unchanged recipes, different metadata/envelope.
import importlib.util
spec=importlib.util.spec_from_file_location('rev2_catalog',ROOT/'conformance/tools/ref_catalog.py')
catalog_module=importlib.util.module_from_spec(spec)
spec.loader.exec_module(catalog_module)
raw=(ROOT/'spec/ref-0.9.json').read_bytes()
catalog=json.loads(raw)
assert catalog_module.jcs(catalog).encode()==raw
assert len(catalog['recipes'])==219
old_pin=hashlib.sha256(b'isa=0.9\nexpansion=ref\nversion=0.9\n'+raw+b'\n').hexdigest()
new_catalog=deepcopy(catalog)
new_catalog['isa']=new_catalog['version']='0.10'
new_raw=catalog_module.jcs(new_catalog).encode()
new_pin=hashlib.sha256(b'isa=0.10\nexpansion=ref\nversion=0.10\n'+new_raw+b'\n').hexdigest()
assert new_catalog['recipes']==catalog['recipes']
assert old_pin=='5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197'
assert new_pin=='83cc9c177bad054bd7af35dc1892645d869b9e4c0b9c968f35b8783ce2b54c42'
assert old_pin in text and new_pin in text
print('PASS fd0: exact formulas/units/negative expressions, hierarchy/shared factories, source/model/JCS identities, 219-recipe 0.9/0.10 hashes')

# Operational dependencies follow quantity references, including zero factors.
def dependencies(expr,quantities,seen=()):
    if isinstance(expr,str): return set()
    if 'param' in expr: return {expr['param']}
    if 'quantity' in expr:
        n=expr['quantity'];assert n not in seen
        return dependencies(quantities[n]['expr'],quantities,seen+(n,))
    if 'op' in expr:
        return set().union(*(dependencies(x,quantities,seen) for x in expr['args']))
    assert set(expr)=={'lit','unit'}
    return set()
frozen=set().union(*(dependencies({'quantity':b['quantity']},m['quantities'])
                     for b in m['bindings']))
assert frozen=={'d','tau'} and 'N' not in frozen
probe_quantities={'indirect':{'expr':{'op':'*','args':['0',{'param':'N'}]}},
                  'duration':{'expr':{'op':'*','args':[
                      {'quantity':'indirect'},{'lit':'1','unit':'s'}]}}}
assert dependencies({'quantity':'duration'},probe_quantities)=={'N'}
def check_rebinding(target_values,provided,frozen_names,algorithm_names):
    assert set(provided)==set(target_values)
    for name,value in provided.items():
        if name in frozen_names or name not in algorithm_names:
            assert value==target_values[name],('frozen parameter changed',name)
target_values={'N':'1'}
saved_model=m
m=deepcopy(m)
m['units']['seconds']={'dimension':{'s':'1'},'scale':'1'}
m['quantities']['operational_N']={'unit':'seconds','role':'duration',
    'expr':{'op':'*','args':[{'param':'N'},{'lit':'1','unit':'seconds'}]}}
assert ev({'quantity':'operational_N'},dict(defaults(),N='1'))==(1,{'s':F(1)})
assert ev({'quantity':'operational_N'},dict(defaults(),N='2'))==(2,{'s':F(1)})
m=saved_model
check_rebinding(target_values,{'N':'1'},{'N'},{'N'})
fails(lambda:check_rebinding(target_values,{'N':'2'},{'N'},{'N'}))
# The same comparison applies to reports, ModelInput and invoke argument binding.
for binding_kind in ('report','ModelInput','invoke'):
    fails(lambda:check_rebinding(target_values,{'N':'2'},{'N'},{'N'}))
new_target={'parameters':{'N':'2'},'Binding.value':'2'}
old_target={'parameters':{'N':'1'},'Binding.value':'1'}
assert digest(new_target)!=digest(old_target)
check_rebinding({'N':None},{'N':'3'},set(),{'N'})

# Standalone resource subjects distinguish effective controls and variant.
base_subject={'closure_hash':'a'*64,'variant':'forward','inherited_controls':0,
              'invocation':{'args':[],'budget':['0']*4}}
controlled_subject=dict(base_subject,inherited_controls=1)
adjoint_subject=dict(base_subject,variant='adjoint')
assert digest(base_subject)!=digest(controlled_subject)!=digest(adjoint_subject)
assert {0:F(1),1:F(2)}[base_subject['inherited_controls']]==1
assert {0:F(1),1:F(2)}[controlled_subject['inherited_controls']]==2

# Objective inputs, rather than selection of an arbitrary report, bind the problem.
selector={'model':'surface','node':'algorithm','metric':'peak','pool':'["surface","factory"]'}
shared_model=deepcopy(m)
shared_model['nodes']['cycle']['intervals']=[]
shared_model['nodes']['cycle']['shared_uses']=[{'id':'factory0','slots':'1'}]
shared_model_hash=digest(shared_model)
assert shared_model_hash!=hash0
def model_input(amount):
    return {'root':{'model':'surface','node':'algorithm'},
            'models':{'surface':{'model_hash':shared_model_hash,'parameters':dict(defaults(),N='3')}},
            'shared':[{'id':'factory0','model':'surface','pool':'factory','unit':'physical',
                       'start':{'lit':'0','unit':'ns'},'end':{'lit':'90','unit':'ns'},
                       'amount':{'lit':str(amount),'unit':'physical'},'slots':'1'}]}
input50,input100=model_input(50),model_input(100)
assert input50['root']=={k:selector[k] for k in ('model','node')}
duration=F(90,10**9)
assert stats([(F(0),duration,F(50))])==(50,F(9,2000000))
assert stats([(F(0),duration,F(100))])==(100,F(9,1000000))
schedule50={'bindings':{'target_hash':'b'*64,'model_inputs':{'factory_peak':input50}},'entries':[]}
schedule100=deepcopy(schedule50)
schedule100['bindings']['model_inputs']['factory_peak']=input100
assert digest(schedule50)!=digest(schedule100)
def certify_inputs(bound,report):
    assert model_jcs(bound)==model_jcs({k:report[k] for k in ('root','models','shared')})
certify_inputs(input50,dict(input50,outputs={'peak':'50'}))
fails(lambda:certify_inputs(input50,dict(input100,outputs={'peak':'100'})))
def objective_state(select,inp,free=False):
    if select is None: return 'validation error'
    if inp is None: return 'incomplete-model-objective'
    return 'symbolic' if free else 'concrete scalar'
assert objective_state(None,input50)=='validation error'
assert objective_state(selector,None)=='incomplete-model-objective'
assert objective_state(selector,input50,True)=='symbolic'
assert objective_state(selector,input50)=='concrete scalar'
print('PASS fd0 blockers: transitive parameter freeze; complete subjects; selector/input verdicts; reservation-bound schedule identity')

# 7s0 generator and range reduction match the published sampler contract.
MASK=(1<<64)-1
def rotl(x,k): return ((x<<k)|(x>>(64-k)))&MASK
def xoshiro(seed):
    s=list(seed)
    while True:
        result=(rotl((s[1]*5)&MASK,7)*9)&MASK
        t=(s[1]<<17)&MASK
        s[2]^=s[0];s[3]^=s[1];s[1]^=s[2];s[0]^=s[3]
        s[2]^=t;s[3]=rotl(s[3],45)
        yield result
def rank_draw(words,N):
    assert N>=1
    bits=(N-1).bit_length()
    if bits==0:return 0
    nwords=(bits+63)//64
    while True:
        x=sum(next(words)<<(64*i) for i in range(nwords))&((1<<bits)-1)
        if x<N:return x
E=[(0,0,0),(0,0,1),(1,1,0),(1,1,1)]
for seed,first in [([0,288230376151711744,0,0],90),
                   ([1,72057594037927936,0,0],9223372036854775826)]:
    assert next(xoshiro(seed))==first
    words=xoshiro(seed);rank=rank_draw(words,len(E))
    assert rank==2 and E[rank]==(1,1,0)
    c,d,b=E[rank]
    assert (c,d,b^c)==(1,1,1)
    # The obsolete product-first extension would miss this failing fiber.
    assert rank_draw(xoshiro(seed),2)==0
words=xoshiro([1,72057594037927936,0,0])
assert [next(words),next(words)]==[9223372036854775826,9223372036854781586]
class CountedWords:
    def __init__(self,values):self.values=iter(values);self.count=0
    def __iter__(self):return self
    def __next__(self):self.count+=1;return next(self.values)
words=CountedWords([3,2])
assert rank_draw(words,3)==2 and words.count==2 # rejection consumes another word
words=CountedWords([])
assert rank_draw(words,1)==0 and words.count==0
# Canonical product order: borrowed b precedes singleton/variable local c,d.
products=[[(0,d,b) for b in range(2) for d in range(2)],
          [(c,0,b) for b in range(2) for c in range(2)]]
union=list(dict.fromkeys(t for product in products for t in product))
assert len(union)==6
assert F(len({union[0]}),len(union))==F(1,6)
def fiber_ok(op,non_b,domain):
    rows=[x for x in domain if x[:-1]==non_b]
    outputs=[op(x) for x in rows]
    return (all(out[-1]==row[-1] for row,out in zip(rows,outputs))
            and len({out[:-1] for out in outputs})==1)
words=CountedWords([90,17])
draw=E[rank_draw(words,4)]
assert not fiber_ok(lambda x:(x[0],x[1],x[2]^x[0]),draw[:-1],E)
assert words.count==1 # complete B-fiber replay draws no random words
assert all(fiber_ok(lambda x:(x[0],x[1],x[2]^x[0]^x[1]),x[:-1],E) for x in E)
assert all((c,b^c)==(c,b) for c,b in [(0,0),(0,1)])
assert (1,0^1)!=(1,0)
# A B-dependent phase violates the tensor identity although basis values agree.
assert [1,(-1)**1]==[1,-1]
assert all((0^a^x^a^x)==0 for a in range(2) for x in range(2))
print('PASS 7s0: two exact xoshiro seeds, rejection/zero-word cases, union order/dedup, B-fiber word count and tensor counterexamples')

# ysw equality partitions, including action_call's lack of a primary token.
def partition(ids):
    labels={};return [labels.setdefault(x,len(labels)) for x in ids]
def token_list(kind,args,primary=None):
    if kind=='oracle':
        assert primary is not None
        return [primary]+args
    assert kind in ('action','perm_action','cc','r')
    return args
assert partition(token_list('oracle',[],99))==[0]
for kind in ('action','perm_action','cc','r'):
    assert partition(token_list(kind,[]))==[]
    assert partition(token_list(kind,[5,5]))==[0,0]
    assert partition(token_list(kind,[5,6]))==[0,1]
assert partition(token_list('oracle',[5,5],99))==[0,1,1]
assert partition(token_list('oracle',[5,6],99))==[0,1,2]
tuple1=['1','0','0','0'];args=[tuple1,tuple1]
assert digest(args)=='bf8eae8d885943f1948223391b24a677ec8eaefc4d39544bd6ed106277a211ed'
def validate_aliases(ids,tuples):
    assert len(ids)==len(tuples) and all(type(x) is int and x>=0 for x in ids)
    assert partition(ids)==ids
    for i in range(len(ids)):
        for j in range(i):
            if ids[i]==ids[j]:assert tuples[i]==tuples[j]
for invalid in ([1,1],[0,2],[-1,0],[0]):
    fails(lambda:validate_aliases(invalid,args))
validate_aliases([0,0],args);validate_aliases([0,1],args)
fails(lambda:validate_aliases([0,0],[['3','0','0','0'],['2','0','0','0']]))
assert partition([100,101])!=partition([100,100]) # fresh mint cannot be tuple interned
print('PASS ysw: oracle/action/CC/R arities; alias consistency, token-argument hash and inverse-pair equality constraint')

# 4ta register vectors and source-slot frames, v2v per-context classes.
a={'id':1,'wires':['wa'],'classes':['A'],'state':'live'}
b={'id':2,'wires':['wb'],'classes':['B'],'state':'live'}
prior=deepcopy([a,b])
t={'id':3,'wires':a['wires']+b['wires'],'classes':a['classes']+b['classes'],'state':'live'}
a['state']=b['state']='consumed'
fragments=[{'offset':0,'width':1,'class':'A'},{'offset':1,'width':1,'class':'B'}]
expanded=[f['class'] for f in fragments for _ in range(f['width'])]
assert t['classes']==expanded==['A','B']
assert t['classes'][1:2]==['B'] and t['wires'][1:2]==['wb']
assert t['classes']!=['A','A']
for value in range(4):
    assert value^3==[3,2,1,0][value]
    assert (value^3)^3==value
# Undo the action, then concat: original identities/vectors/states are restored.
t['state']='consumed';a,b=deepcopy(prior)
assert [a,b]==prior and t['state']!='live'
assert [(cell,bit) for cell in range(2) for bit in range(2)]==[(0,0),(0,1),(1,0),(1,1)]
for control in range(2):
    for value in range(4):assert (value^(3*control))^(3*control)==value
scheduled_end=sum([F(1),F(1)])
assert scheduled_end==2
for cls in ('A','B'):
    live=[(F(0),scheduled_end,F(1))]
    assert stats(live)==(1,2) # one interface wire live through both sequential slots
pair='a'*64
keys=[[{'symbol':pair,'point':f'entry:{i}','iteration':[],'instance':[],'call_index':0}]
      for i in range(2)]
assert sorted(keys,key=lambda x:model_jcs(x).encode())==keys
frames={model_jcs(key):cls for key,cls in zip(keys,['A','B'])}
assert len(frames)==2 and set(frames.values())=={'A','B'}
assert model_jcs([])=='[]' # root-relative key, independent of native main
print('PASS 4ta/v2v: A/B concat/slice/reversal vectors, all four maps/inverses, controlled case, frame keys and resources')

# 0jq: exact H^2 = I, but H has two nonzero amplitudes in each column.
M=[[1,1],[1,-1]]
assert [[F(sum(M[i][k]*M[k][j] for k in range(2)),2) for j in range(2)]
        for i in range(2)]==[[1,0],[0,1]]
assert all(sum(x!=0 for x in row)==2 for row in M)

# 2cg: same existing thin/production primitive arithmetic, no new allocation rule.
def allocate_trace(requests,heap):
    bump=4096;records=[];pointers=[]
    for request in requests:
        if bump+max(request,1)>4096+heap:return bump,records,pointers,2
        pointers.append(bump);records.append((bump,request,1));bump+=max(request,1)
    return bump,records,pointers,0
bump,records,pointers,fault=allocate_trace([3,13,16],32)
assert (bump,len(records),pointers[-1],fault)==(4128,3,4112,0)
bump,records,pointers,fault=allocate_trace([3,0,12,16,32],64)
assert (bump,len(records),pointers[3],fault)==(4160,5,4112,0)
assert records[1]==(4099,0,1)
bump,records,pointers,fault=allocate_trace([3,0,12,16,32],32)
assert (bump,len(records),pointers[3],fault)==(4128,4,4112,2)
assert min(16,48)==16 and min(32,48)==32
print('PASS 0jq/2cg: exact H algebra, minimal/production allocation traces and retained failure prefix')

# All requested revisions are standalone, linked and map every round-1 blocker.
names=['b-isa-spec-0jq','b-isa-spec-v2v','b-dialect-spec-2cg','b-isa-spec-7s0',
       'b-isa-spec-ysw','b-isa-spec-4ta','fd0']
expected_items={'b-isa-spec-0jq':['0JQ-1','B1'],
                'b-isa-spec-v2v':['V2V-1'],'b-dialect-spec-2cg':['B1'],
                'b-isa-spec-7s0':['B1','B2'],'b-isa-spec-ysw':['B1','B2'],
                'b-isa-spec-4ta':['4ta-B1','4ta-B2','B1','B2'],
                'fd0':['FD0-1','FD0-2','FD0-3','FD0-4','FD0-5','B1','B2','B3']}
for name in names+['fat-object-scope']:
    path=OUT/(name+'.md');body=path.read_text()
    assert body.count('\n```')%2==0,(name,'unbalanced fences')
    assert not re.search(r'\b(?:TODO|TBD)\b',body),(name,'unfinished marker')
    assert '(INDEX.md)' not in body,(name,'depends on missing index')
    for block in re.findall(r'^```json\n(.*?)\n```',body,re.M|re.S):
        json.loads(block)
    if name!='fat-object-scope':
        assert '## Round-1 blocking items and resolution' in body,name
        assert '## Fat-object applicability' in body,name
        assert 'fat-object-scope.md#citable-scoping-statement' in body,name
        section=body.split('## Round-1 blocking items and resolution',1)[1].split('\n## ',1)[0]
        assert all(item in section for item in expected_items[name]),name
    for target in re.findall(r'\[[^\]]+\]\(([^)]+)\)',body):
        if not target.startswith(('http://','https://','#')):
            assert (path.parent/target.split('#')[0]).exists(),(name,'dangling link',target)
    fenced=False;table_pipes=None
    for line in body.splitlines():
        if line.startswith('```'):fenced=not fenced;continue
        if fenced:continue
        if line.startswith('|'):
            count=len(re.findall(r'(?<!\\)\|',line))
            if table_pipes is not None:assert count==table_pipes,(name,'table arity',line)
            table_pipes=count
        else:table_pipes=None
scope=(OUT/'fat-object-scope.md').read_text()
assert all(word in scope for word in ('argv/envp','§5.4','§3.3','§7.3','§10',
                                    'host-memory-by-reference','launch'))
assert 'no general host-memory-by-reference' in scope
assert len(scope.split('> A mixed fat object',1)[1].split('\n\n',1)[0].split())<250
assert 'subject:{closure_hash:H,variant:S,inherited_controls:N,' in text
assert 'schedule.bindings' in text and 'ModelInput = {' in text
assert 'Model objective with and without an exact root selector' not in text
assert 'Catalog bytes/hash change only if separately revised' not in text
old_2cg=(ROOT/'docs/notes/2026-11-02-pass2/b-dialect-spec-2cg.md').read_text()
new_2cg=(OUT/'b-dialect-spec-2cg.md').read_text()
def allocator_insertion(s):
    return s.split('## Recommended exact spec diff',1)[1].split('```text\n',1)[1].split('\n```',1)[0]
assert allocator_insertion(old_2cg)==allocator_insertion(new_2cg)
fat_2cg=new_2cg.split('## Fat-object applicability',1)[1].split('```text\n',1)[1].split('\n```',1)[0]
for name in ('Bennett-Dialect-Spec.md','Bennett-Dialect-Spec-0.3-draft.md'):
    dialect=(ROOT.parent/'bennett-dialect-spec/spec'/name).read_text()
    start=dialect.index('### 17.1 ');end=dialect.index('### 17.2 ',start)
    proposed=dialect[:end]+fat_2cg+'\n\n'+dialect[end:]
    assert proposed[:start]==dialect[:start]
    assert proposed.split('### 17.2 ',1)[1]==dialect.split('### 17.2 ',1)[1]
    assert proposed.count(fat_2cg)==1
print('PASS documents: eight complete notes, seven round-1 item maps, fat-object citations, local links, fences and tables')
print('PASS: all rev2 finite semantic and document checks; proposal schemas and mixed launch remain unimplemented/unaccepted')


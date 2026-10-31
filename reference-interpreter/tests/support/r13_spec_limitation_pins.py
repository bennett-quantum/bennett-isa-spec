"""INV-4 step 1.5.3: independently pin the current §9.2 spec limitation.

No interpreter/rehash dependency. These are equal envelope bytes, not a
cryptographic SHA-256 collision. A future spec fix should retire this witness.
"""
import hashlib
import json
from pathlib import Path

def j(v):
    return json.dumps(v, sort_keys=True, separators=(',', ':'))
def sha(s):
    return hashlib.sha256(s.encode()).hexdigest()
header = 'bennett-object isa=0.8 object=0.8 launch=0.8 sched=0.8 cert=0.8 coverage=0.8 target=0.8 ptr_width=32\n'
scalars = dict(A_tab=0,H=6,S_cells=0,heap_base=48,heap_limit_max=0,null_size=1,ptr_width=32)
scalar_lines = {k:'layout '+k+' '+j(v)+'\n' for k,v in scalars.items()}
leaf = 'layout interface_regions.a.c.db {"base":32}\n'
lines = ''.join(v for k,v in sorted({**scalar_lines, 'interface_regions.a.c.db':leaf}.items()))
osig = dict(abi='0.8',args=[],fault=4,**{'in':[8]},kind='oracle',mem=[dict(A=0,W=8,role='in')],tgt=8,tier='q')
cases = []
for tier in ['cc','r']:
    sig = dict(abi='0.8',kind=tier,params=[],results=[],tier=tier)
    body = f'fn {tier} @main () () () () ()\n^entry():\nret\n'
    main_env = 'isa=0.8\nptr_width=32\n'+j(sig)+'\n'+body+lines
    for owner,region in [('a','c.db'),('a.c','db')]:
        obody = f'oracle @{owner} (x:wire[8] in) (t:wire[8]) (f:wire[4]) ({region}:region[0,8] in) ()\n^entry():\nlift.retval x\n'
        oracle_env = 'isa=0.8\nptr_width=32\n'+j(osig)+'\n'+obody+lines
        symbols = {'0':dict(name='main',kind=tier,signature=sig,interface_hash=sha(main_env)), '1':dict(name=owner,kind='oracle',signature=osig,interface_hash=sha(oracle_env))}
        layout = dict(scalars,globals={},interface_regions={owner:{region:dict(base=32)}})
        text = header+'\nsection symbols\n'+j(symbols)+'\n\nsection functions\n'+body+obody+'\nsection layout\n'+j(layout)+'\n'
        cases.append(dict(name=tier+'-'+owner,object=text,envelope=main_env,interface_hash=sha(main_env),object_hash=sha(text)))
    assert cases[-1]['envelope'] == cases[-2]['envelope']
    assert cases[-1]['object_hash'] != cases[-2]['object_hash']
    print(tier, 'equal interface/closure', sha(main_env), 'distinct object pins', cases[-2]['object_hash'], cases[-1]['object_hash'])
Path(__file__).with_name('r13_spec_limitation_cases.json').write_text(json.dumps(cases,indent=2)+'\n')

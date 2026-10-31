"""Generate resumable constructor checkpoints. Python -I, stdlib only.
No interpreter imports. Does not publish or update the normative hash marker.
"""
import argparse
import hashlib
import json
import runpy
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
def jcs(v):
    # This catalog uses integer numbers below 2**53; no floats. UTF-16 key order.
    if type(v) is bool: return "true" if v else "false"
    if isinstance(v, dict):
        return '{' + ','.join(jcs(k)+':'+jcs(v[k]) for k in sorted(v, key=lambda k:k.encode('utf-16be'))) + '}'
    if isinstance(v, list):
        return '[' + ','.join(map(jcs,v)) + ']'
    if isinstance(v, str):
        return json.dumps(v, ensure_ascii=False, separators=(',', ':'))
    if type(v) is int and abs(v) < 2**53:
        return str(v)
    raise ValueError(f'outside catalog JCS subset: {v!r}')
def plus(a,b): return ['+',a,b]
def minus(a,b): return ['-',a,b]
def times(a,b): return ['*',a,b]
def wire(name, bit=0): return name+'['+jcs(bit)+']'
def gate(g,*w): return {'gate':g,'wires':list(w)}
def alloc(n,w): return {'alloc':n,'width':w,'class':'any','state':'clean'}
def release(n): return {'release':n}
def loop(i,n,b): return {'for':i,'from':0,'to':n,'body':b}
def inv(b): return {'inverse':True,'body':b}
class Circuit:
    def __init__(self): self.body=[]; self.scratch=[]; self.serial=0
    def temp(self,width=1):
        n=f'z{self.serial}'; self.serial+=1
        self.scratch.append(n); self.body.append(alloc(n,width)); return n
    def bit(self): return wire(self.temp())
    def boolean(self,e,out):
        """Postorder tree, fresh scratch per node, no CSE; root XOR out."""
        computes=[]; names=[]
        def visit(e):
            if isinstance(e,str): return e
            op,*args=e
            if op=='or': return visit(('xor',('xor',*args),('and',*args)))
            ws=[visit(a) for a in args]
            n=self.temp(); names.append(n); d=wire(n)
            if op=='not': ops=[gate('x',d),gate('cx',ws[0],d)]
            elif op=='xor': ops=[gate('cx',s,d) for s in ws]
            elif op=='and': ops=[gate('cx',ws[0],d)] if ws[0]==ws[1] else [gate('ccx',*ws,d)]
            else: raise ValueError(op)
            self.body.extend(ops); computes.extend(ops); return d
        root=visit(e); self.body.append(gate('cx',root,out)); self.body.append(inv(computes))
        self.body.extend(release(n) for n in reversed(names))
    def ripple(self,a,b,n,sub=False):
        p=self.temp(n); s=self.temp(n); c=self.temp(plus(n,1))
        body_start=len(self.body)
        if sub: self.body.append(gate('x',wire(c)))
        start=len(self.body)
        aa=a('i'); bb=b('i')
        self.boolean(('xor',aa,('not',bb) if sub else bb),wire(p,'i'))
        self.boolean(('xor',wire(p,'i'),wire(c,'i')),wire(s,'i'))
        self.boolean(('xor',('and',aa,('not',bb) if sub else bb),('and',wire(p,'i'),wire(c,'i'))),wire(c,plus('i',1)))
        bitbody=self.body[start:]; self.body[start:]=[loop('i',n,bitbody)]
        return s,c,body_start
    def finish(self,outops,start=0):
        compute=self.body[start:]; self.body=self.body[:start]+compute+outops+[inv(compute)]
    def recipe(self,kind,params,source):
        return {'kind':kind,'parameters':params,'body':self.body,'scratch':self.scratch,'removable':[], 'source':source}
def recipe(kind,params,body,source,scratch=()):
    return {'kind':kind,'parameters':params,'body':body,'scratch':list(scratch),'removable':[],'source':source}
def copy_family():
    out=[]
    for k in ['qxor','qcopy','quncopy','qexch']:
        op=gate('swap',wire('a','i'),wire('b','i')) if k=='qexch' else gate('cx',wire('s','i'),wire('t','i'))
        body=[loop('i','N',[op])]
        if k=='qcopy': body.insert(0,alloc('t','N'))
        if k=='quncopy': body += [inv([alloc('t','N')])]
        out.append(recipe(k,['N'],body,'0.8 §4.5: '+('swap a[i] b[i]' if k=='qexch' else 'cx s[i] t[i]')))
    for k,ws in [('x',['a']),('cx',['a','b']),('ccx',['a','b','t']),('swap',['a','b'])]:
        out.append(recipe('gate.'+k,[],[gate(k,*map(wire,ws))],'0.8 §4.4: '+k+' is exact, including global phase.'))
    return out
def arithmetic_family():
    out=[]
    for op in ['add','sub']:
        c=Circuit(); s,carry,_=c.ripple(lambda i:wire('a',i),lambda i:wire('b',i),'N',op=='sub')
        compute=c.body[:]
        c.body += [loop('j','N',[gate('cx',wire(s,'j'),wire('t','j'))]),inv(compute)]
        out.append(c.recipe('value.'+op,['N'],'0.8 §4.1: add sub wrap; §17 item 7: ripple-carry adders and comparators.'))
    for op in ['eq','ne','ult','ule','slt','sle']:
        c=Circuit();carry=None;eq=None
        if op in ['ult','ule','slt','sle']: _,carry,_=c.ripple(lambda i:wire('a',i),lambda i:wire('b',i),'N',True)
        if op in ['eq','ne','ule','sle']:
            eq=c.temp(plus('N',1)); c.body.append(gate('x',wire(eq)))
            start=len(c.body)
            c.boolean(('and',wire(eq,'i'),('not',('xor',wire('a','i'),wire('b','i')))),wire(eq,plus('i',1)))
            c.body[start:]=[loop('i','N',c.body[start:])]
        u=('not',wire(carry,'N')) if carry else None; e=wire(eq,'N') if eq else None
        v={'eq':e,'ne':('not',e),'ult':u,'ule':('or',u,e)}.get(op)
        if v is None:
            sa=wire('a',minus('N',1)); sb=wire('b',minus('N',1)); diff=('xor',sa,sb)
            signed=('xor',('and',diff,sa),('and',('not',diff),u))
            v=signed if op=='slt' else ('or',signed,e)
        dest=c.temp(); c.boolean(v,wire(dest)); compute=c.body[:]
        c.body += [gate('cx',wire(dest),wire('t')),inv(compute)]
        out.append(c.recipe('value.icmp.'+op,['N'],'0.8 §4.1: icmp.{eq,ne,ult,ule,slt,sle}; §17 item 7: ripple-carry adders and comparators.'))
    return out

def build(families):
    recipes=copy_family()
    if families>=2: recipes+=arithmetic_family()
    if families>=3: recipes+=runpy.run_path(str(ROOT/'conformance/tools/ref_arithmetic.py'))['family'](globals())
    if families>=4: recipes+=runpy.run_path(str(ROOT/'conformance/tools/ref_flags.py'))['family'](globals())
    if families>=5: recipes+=runpy.run_path(str(ROOT/'conformance/tools/ref_dispatch.py'))['family'](globals())
    if families>=6: recipes+=runpy.run_path(str(ROOT/'conformance/tools/ref_bound.py'))['classical'](globals())
    if families>=7: recipes+=runpy.run_path(str(ROOT/'conformance/tools/ref_bound.py'))['rows'](globals())
    if families>=8: recipes+=runpy.run_path(str(ROOT/'conformance/tools/ref_memory.py'))['family'](globals())
    if families>=9: recipes+=runpy.run_path(str(ROOT/'conformance/tools/ref_structure.py'))['structure'](globals())
    if families>=10:
        recipes+=runpy.run_path(str(ROOT/'conformance/tools/ref_structure.py'))['native'](globals())
        recipes+=runpy.run_path(str(ROOT/'conformance/tools/ref_memory.py'))['call_flag'](globals())
    if families>=11: recipes+=runpy.run_path(str(ROOT/'conformance/tools/ref_step.py'))['family'](globals())
    if families>=12: recipes+=runpy.run_path(str(ROOT/'conformance/tools/ref_layout.py'))['family'](globals())
    if families>=13: recipes+=runpy.run_path(str(ROOT/'conformance/tools/ref_allocations.py'))['family'](globals())
    if families>=14: recipes+=runpy.run_path(str(ROOT/'conformance/tools/ref_calls.py'))['family'](globals())
    if families>=15: recipes+=runpy.run_path(str(ROOT/'conformance/tools/ref_activation.py'))['family'](globals())
    recipes.sort(key=lambda r:r['kind'].encode())
    return {'isa':'0.9','name':'ref','version':'0.9','recipes':recipes}
def witness():
    text=(ROOT/'spec/Bennett-ISA-Spec-0.9-draft.md').read_text()
    w=json.loads(text.split('**ISA-REF-LIB witness**')[1].split('```json\n')[1].split('\n```')[0])
    assert hashlib.sha256(('isa=0.9\nexpansion=ref\nversion=0.9\n'+jcs(w)+'\n').encode()).hexdigest()=='299510fd67cc626fdcbcc10adcd40730f9af294ba596d86fb25fd31be08a5c7f'
if __name__=='__main__':
    p=argparse.ArgumentParser(); p.add_argument('--families',type=int,choices=range(1,16),default=15); p.add_argument('--check',action='store_true'); a=p.parse_args()
    witness(); raw=jcs(build(a.families)).encode(); path=ROOT/'spec/ref-0.9-draft.json'
    if a.check: assert path.read_bytes()==raw
    else: path.write_bytes(raw)
    print('checkpoint only:',len(build(a.families)['recipes']),'recipes; Python envelope SHA-256',hashlib.sha256(b'isa=0.9\nexpansion=ref\nversion=0.9\n'+raw+b'\n').hexdigest())

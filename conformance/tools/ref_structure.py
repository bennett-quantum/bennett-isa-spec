"""Literal structured-body sequences and native matrix leaves."""
def _all(api):
    recipe,wire,gate,loop,inv,plus,minus,alloc,release=[api[k] for k in ['recipe','wire','gate','loop','inv','plus','minus','alloc','release']]
    def F(n,t='integer',w=None):
        r={'name':n,'type':t}
        if w is not None:r['width']=w
        return r
    def under(w,b):return {'under':w,'body':b}
    splice={'splice':'B'}
    out=[]
    out.append(recipe('qif',[F('p','wire',1),F('Then','body'),F('Else','body')],[under('p',[{'splice':'Then'}]),gate('x',wire('p')),under('p',[{'splice':'Else'}]),gate('x',wire('p'))],'§4.6: qpred w; then; qunpred w; x w; qpred w; else; qunpred w; x w.'))
    out.append(recipe('qloop',[F('N'),F('B','body')],[loop('iteration','N',[splice])],'§4.6: body n times, n static and nonnegative.'))
    nonempty=minus(1,['div',1,plus('N',1)])
    initial=loop('first',nonempty,[gate('cx',wire('cond'),wire('flags'))])
    advance=loop('advance',minus(1,['div',1,minus('N','i')]),[gate('ccx',wire('flags','i'),wire('cond',plus('i',1)),wire('flags',plus('i',1)))])
    flags=[initial,{'for':'i','from':1,'to':minus(plus('N',1),nonempty),'body':[gate('ccx',wire('flags',minus('i',1)),wire('cond','i'),wire('flags','i'))]}]
    body=[alloc('flags','N'),initial,loop('i','N',[under(wire('flags','i'),[splice]),advance]),inv(flags),release('flags')]
    out.append(recipe('qloop.while',[F('N'),F('cond','wire','N'),F('B','body')],body,'§4.6 qloop while: flags c1..cn clean; initialize from cond[0], iterate under ci, advance by CCX, uncompute reverse and free. N=0 emits no flag gates.',['flags']))
    out.append(recipe('qpred.scope',[F('p','wire',1),F('B','body')],[under('p',[splice])],'§3.5/§4.6: matching qpred/qunpred pair is one lexical effective-control scope.'))
    out.append(recipe('ctl',[F('p','wire',1),F('B','body')],[under('p',[splice])],'§4.7: ctl adds the control to semantic gates of the bound call body; allocation is unconditional.'))
    out.append(recipe('adj',[F('B','body')],[splice],'§4.7/§5.3: B is the source-adjointed body formed by the instruction replacements before recipe instantiation; splice it with its own instruction ordinal spaces.'))
    for g,ports in [('x',['a']),('cx',['a','b']),('ccx',['a','b','t']),('swap',['a','b'])]:
        out.append(recipe('bind.gate.'+g,[F(n,'wire',1) for n in ports]+[F('E','environment')],[gate(g,*ports)],'§4.4 native '+g+' with explicit positional wire/environment binding.'))
    for g in ['h','s','sdg','t','tdg','cz','rx','ry','rz','mcx']:
        params=[]
        if g=='mcx':params=[F('K'),F('controls','wire','K'),F('a','wire',1)];body=[gate(g,'controls','a')]
        else:
            if g in ['rx','ry','rz']:params.append(F('theta','angle'))
            params.append(F('a','wire',1));body=[gate(g,'a')]
            if g=='cz':params.append(F('b','wire',1));body=[gate(g,'a','b')]
            if g in ['rx','ry','rz']:body[0]['params']=['theta']
        params.append(F('E','environment'))
        out.append(recipe('gate.'+g,params,body,'§4.4 exact native '+g+' leaf; §14.2 full-basis matrix semantics; target schemes expand later.'))
    return out

def structure(api):return [r for r in _all(api) if not (r["kind"].startswith("gate.") or r["kind"].startswith("bind.gate."))]
def native(api):return [r for r in _all(api) if r["kind"].startswith("gate.") or r["kind"].startswith("bind.gate.")]

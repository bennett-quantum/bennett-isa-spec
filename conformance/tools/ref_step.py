"""The §3.6 dispatcher algorithm; row bodies and successors are typed templates."""
def family(api):
    recipe,wire,gate,loop,inv,plus,times,alloc,release=[api[k] for k in ['recipe','wire','gate','loop','inv','plus','times','alloc','release']]
    def I(n):return {'name':n,'type':'integer'}
    def W(n,w):return {'name':n,'type':'wire','width':w}
    def T(n):return {'name':n,'type':'templates','length':'N'}
    def call(k,*args):return {'invoke':k,'args':list(args)}
    def under(w,b):return {'under':w,'body':b}
    def scan(source):
        literal=call('value.literal','P','k','counter')
        return [alloc('counter','P'),loop('k',plus('N',1),[literal,call('bind.value.icmp.eq','P',source,'counter',wire('at','k')),inv([literal])]),release('counter')]
    body=[alloc('at',plus('N',1))]+scan('pc')
    body += [loop('k','N',[under(wire('at','k'),[{'apply':'Rows','index':'k','args':['frame']}])])]
    body += [loop('k','N',[under(wire('at','k'),[{'apply':'Next','index':'k','args':['frame','next']}])]),under(wire('at','N'),[call('value.literal','P','N','next')])]
    body += [loop('bit','P',[gate('swap',wire('pc','bit'),wire('next','bit'))]),loop('bit','P',[gate('swap',wire('next','bit'),wire('frame','bit'))])]
    oldpc={'range':'frame','from':0,'to':'P'}
    body += [inv(scan(oldpc)),release('at')]
    params=[I('P'),I('N'),I('Q'),W('pc','P'),W('next','P'),W('frame','Q'),T('Rows'),T('Next')]
    # P satisfies 2^P>N; Q>=P; first P frame bits are old_pc, remaining
    # fields are selected explicitly by each typed source row template.
    out = [recipe('step',params,body,'§3.6: dispatch k=0..N inclusive; execute rows k<N in index order; successors k<N then HALT; exchange pc/next and next/frame.old_pc; uncompute dispatch from old_pc. P satisfies 2^P>N; Q>=P; Rows[k] takes frame[Q], Next[k] takes frame[Q], successor[P].', ['at','counter'])]

    cells=times('C','W')
    body=[alloc('cells',cells),{'image':'I','wires':'cells'},{'apply':'B','index':0,'args':['cells']},inv([{'image':'I','wires':'cells'}]),release('cells')]
    params=[I('C'),I('W'),{'name':'I','type':'image'},{'name':'B','type':'templates','length':1},{'name':'E','type':'environment'}]
    out.append(recipe('qregion',params,body,'§4.6 qregion: complete function-local scope; C=2^A cells of W bits; allocate at entry, initialize from bound data image unconditionally, apply function body to cells[C*W], reverse image and release clean at exit.',['cells']))
    return out

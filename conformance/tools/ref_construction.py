"""§5.2 construction contains allocation, steps, commit and reversal."""
def family(api):
    recipe,wire,gate,loop,inv,plus,times,alloc,release=[api[k] for k in ['recipe','wire','gate','loop','inv','plus','times','alloc','release']]
    def I(n):return {'name':n,'type':'integer'}
    def W(n,w):return {'name':n,'type':'wire','width':w}
    def A(n):return {'name':n,'type':'allocation'}
    def call(k,*args):return {'invoke':k,'args':list(args)}
    def under(w,b):return {'under':w,'body':b}
    def visit(n,b):return {'visit':n,'body':b}
    state={'name':'State','type':'records','fields':['width','initial',{'name':'I','type':'image'},A('storage')],'order':[]}
    frames={'name':'Frames','type':'records','fields':['index',A('frame')],'order':['index']}
    params=[I(n) for n in ['P','N','Q','B','F','T']]+[A(n) for n in ['pc','next','faulted','code','result']]+[W('tgt','T'),W('fr','F'),state,frames,{'name':'Rows','type':'templates','length':'N'},{'name':'Next','type':'templates','length':'N'}]
    initialize=[{'acquire':'storage'},call('value.literal','width','initial','storage'),{'image':'I','wires':'storage'}]
    step=call('step','P','N','Q','pc','next','frame','Rows','Next')
    body=[visit('State',initialize),visit('Frames',[step])]
    literal=call('value.literal','P','N','halt');eq=call('bind.value.icmp.eq','P','pc','halt','at')
    body+=[alloc('halt','P'),literal,alloc('at',1),eq]
    body+=[gate('x',wire('faulted')),under(['at','faulted'],[loop('bit','T',[gate('cx',wire('result','bit'),wire('tgt','bit'))])]),gate('x',wire('faulted'))]
    body+=[under('faulted',[loop('bit','F',[gate('cx',wire('code','bit'),wire('fr','bit'))])]),gate('x',wire('at')),under('at',[call('value.literal','F',1,'fr')]),gate('x',wire('at'))]
    body+=[inv([eq]),release('at'),inv([literal]),release('halt'),inv([visit('Frames',[step])]),inv([visit('State',initialize)])]
    return [recipe('oracle.construct',params,body,'§5.2: source-formed State descriptors allocate exact step state, declaration-order locations, image regions, table and B frames; Frames has exactly B indices 0..B-1 and matching State identities. Run catalog step for each frame, commit success payload / first fault / step exhaustion, reverse frames then initialization and allocations. P satisfies 2^P>N; Q is exact source frame width; F>=4. Explicit row/successor templates contain source rows, never an opaque dispatcher.',['halt','at','storage'])]

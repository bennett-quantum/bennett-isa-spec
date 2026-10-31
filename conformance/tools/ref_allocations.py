"""Allocation-producing source rows with K27 shared declarations."""
def family(api):
    recipe,wire,gate,loop=[api[k] for k in ['recipe','wire','gate','loop']]
    def I(n):return {'name':n,'type':'integer'}
    def W(n,w):return {'name':n,'type':'wire','width':w}
    def A():return {'name':'D','type':'allocation'}
    def call(k,*args):return {'invoke':k,'args':list(args)}
    out=[]
    for k in ['qalloc','qborrow','qfree','qreturn']:
        acquire=k in ['qalloc','qborrow']
        out.append(recipe(k,[A()],[{'acquire' if acquire else 'relinquish':'D'}],'§4.3 '+k+': declaration D is bound to the exact source class/width; state must be '+('borrowed' if k in ['qborrow','qreturn'] else 'clean')+'. Unconditional allocation/release; the caller declaration owns the lifetime.'))
    for k in ['qcopy.bound','quncopy.bound']:
        body=[loop('bit','N',[gate('cx',wire('s','bit'),wire('D','bit'))])]
        body=([{'acquire':'D'}]+body) if k=='qcopy.bound' else (body+[{'relinquish':'D'}])
        out.append(recipe(k,[I('N'),W('s','N'),A()],body,'§4.5 '+k.split('.')[0]+': shared clean declaration D has width N and exact source class; allocate then XOR, or XOR then clean release.'))
    for role in ['in','mut']:
        for k in ['qload','qunload']:
            params=[I('A'),I('C'),I('W'),W('a','A'),{'name':'R','type':'region','address':'A','cell':'W','role':role},A()]
            body=[call('dispatch.copy','A','C','W','a','R','D')]
            body=([{'acquire':'D'}]+body) if k=='qload' else (body+[{'relinquish':'D'}])
            out.append(recipe(k+'.'+role,params,body,'§4.6 '+k+': C=2^A; region role '+role+'; D is clean width W; increasing-address equality dispatch into allocation result, or identical XOR then release.'))
    return out

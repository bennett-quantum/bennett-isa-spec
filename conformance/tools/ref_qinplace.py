"""§4.5 qinplace literal sequence, with K22 source-call binding adapters."""
def family(api):
    recipe,wire,gate,loop = [api[k] for k in ['recipe','wire','gate','loop']]
    params=[{'name':n,'type':'integer'} for n in ['N','F']]
    params += [{'name':'x','type':'wire','width':'N'}]
    params += [{'name':n,'type':'templates','length':1} for n in ['G','H']]
    params += [{'name':'E','type':'environment'}]
    body=[{'acquire':'t'},{'acquire':'f'},
          {'apply':'G','index':0,'args':['x','t','f']},
          loop('bit','N',[gate('swap',wire('x','bit'),wire('t','bit'))]),
          {'apply':'H','index':0,'args':['x','t','f']},
          {'relinquish':'t'},{'relinquish':'f'}]
    for name,width in [('f','F'),('t','N')]:
        body=[{'declare':name,'width':width,'class':{'class':'any'},'state':'clean','body':body}]
    return [recipe('qinplace',params,body,'§4.5 qinplace: t=qalloc any N; f=qalloc any F; oracle_call G; qexch x t; oracle_call H; qfree t; qfree f, exactly in this order (K28). G/H are one-element K22 source-call adapters with wire[N],wire[N],wire[F] formals for x,t,f: each body is the corresponding linked oracle call, capturing the exact ins, mem, args and token actuals (K24/K26), never a supplied decomposition. N/F and both interfaces match §4.5; a covering inverse-pair certificate is required before effects. All call/exchange gates inherit E; allocations/releases are unconditional. The private instruction allocations are not recipe scratch. The adjoint swaps G/H with all captures retained.')]

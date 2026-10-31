"""§4.2 byte memory: descriptor iteration contains the complete dispatcher."""
def family(api):
    recipe,wire,gate,loop,inv,plus,minus,times,alloc,release=[api[k] for k in ['recipe','wire','gate','loop','inv','plus','minus','times','alloc','release']]
    def I(n):return {'name':n,'type':'integer'}
    def W(n,w):return {'name':n,'type':'wire','width':w}
    def call(k,*args):return {'invoke':k,'args':list(args)}
    def under(w,b):return {'under':w,'body':b}
    out=[]
    for op in ['load','store']:
        params=[I(n) for n in ['P','L','E','C','T','F','N','B']]+[W(n,w) for n,w in [('a','P'),('base',times('C','P')),('size',times('C','P')),('live','C'),('pinned','C'),('atop','T'),('flag',1),('faulted',1),('code','F'),('v','N'),('disp','N')]]
        params += [{'name':'Layout','type':'records','fields':['base','extent',W('cells',times(8,'extent'))],'order':['base']}]
        # length literal, inherent no-wrap range check and effective fault writes
        body=[alloc('len','L'),call('value.literal','L','B','len'),call('lift.boundscheck','P','L','E','C','T','F','a','len','base','size','live','pinned','atop','flag','faulted','code'),inv([call('value.literal','L','B','len')]),release('len')]
        # All address temporaries vanish before load exchanges its destination.
        setup=[alloc('ax','E'),loop('bit','P',[gate('cx',wire('a','bit'),wire('ax','bit'))])]
        byte_setup=[alloc('offset','E'),call('value.literal','E','byte','offset'),alloc('address','E'),call('bind.value.add','E','ax','offset','address')]
        literal=call('value.literal','E',plus('base','cell'),'constant')
        equal=call('bind.value.icmp.eq','E','address','constant','equal')
        cellbit=wire('cells',plus(times(8,'cell'),'bit')); dbit=wire('disp',plus(times(8,'byte'),'bit'))
        effect=gate('cx',cellbit,dbit) if op=='load' else gate('swap',cellbit,dbit)
        scan=[{'visit':'Layout','body':[loop('cell','extent',[alloc('constant','E'),literal,alloc('equal',1),equal,under('equal',[loop('bit',8,[effect])]),inv([equal]),release('equal'),inv([literal]),release('constant')])]}]
        scan_body=byte_setup+scan+[inv(byte_setup)]
        success=setup[:]
        if op=='store':success+=[loop('bit','N',[gate('cx',wire('v','bit'),wire('disp','bit'))])]
        success+=[loop('byte','B',scan_body),inv(setup)]
        if op=='load':success+=[loop('bit','N',[gate('swap',wire('v','bit'),wire('disp','bit'))])]
        body+=[gate('x',wire('flag')),under('flag',success),gate('x',wire('flag'))]
        out.append(recipe('lift.'+op,params,body,'§4.2 lift.'+op+': inherent inrange(a,B) first, effective fault 7; N=8*B with B=1..16, L=bit_length(B), E=max(P,L)+1, F>=4. Layout is K25 byte partition (base is base), complete [0,2^H), H<=P. Scan byte then layout address then cell LSB. Load computes whole slot before exchanging destination; store captures whole v before all memory exchanges. Frame disp/flag initially zero; v may alias a on load.',['len','ax','offset','address','constant','equal']))
    return out

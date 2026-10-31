"""§3.7 C1-C5/R1-R5 algorithms over shared K25 descriptors."""
def family(api):
    recipe,wire,gate,loop,inv,plus,times,alloc,release=[api[k] for k in ['recipe','wire','gate','loop','inv','plus','times','alloc','release']]
    def I(n):return {'name':n,'type':'integer'}
    def W(n,w):return {'name':n,'type':'wire','width':w}
    def call(k,*args):return {'invoke':k,'args':list(args)}
    def under(w,b):return {'under':w,'body':b}
    def visit(n,b):return {'visit':n,'body':b}
    def copy(n,a,b):return loop('bit',n,[gate('cx',wire(a,'bit'),wire(b,'bit'))])
    def exchange(n,a,b):return loop('bit',n,[gate('swap',wire(a,'bit'),wire(b,'bit'))])
    out=[]
    for op in ['copy','store','exchange']:
        lit=call('value.literal','S','cell','constant');eq=call('bind.value.icmp.eq','S','address','constant','equal')
        sb=wire('stack',plus(times('cell','K'),'bit'));v=wire('v','bit')
        effect=gate('swap',sb,v) if op=='exchange' else gate('cx',sb,v) if op=='copy' else gate('cx',v,sb)
        body=[loop('cell','C',[alloc('constant','S'),lit,alloc('equal',1),eq,under('equal',[loop('bit','N',[effect])]),inv([eq]),release('equal'),inv([lit]),release('constant')])]
        params=[I(n) for n in ['S','C','K','N']]+[W('address','S'),W('stack',times('C','K')),W('v','N')]
        out.append(recipe('stack.'+op,params,body,'§3.7: C physical stack cells of K bits, N<=K; one-hot address dispatch touches only low N cell bits.',['constant','equal']))
    locations={'name':'Locations','type':'records','fields':['id','width','slot',W('location','width')],'order':['id']}
    globals=dict(locations,name='Globals')
    captures={'name':'Captures','type':'records','fields':['position','width',W('source','width'),W('parameter','width'),W('capture','width')],'order':['position']}
    def address(base,offset,op,n,v):
        setup=[alloc('offset','S'),call('value.literal','S',offset,'offset'),alloc('address','S'),call('bind.value.add','S',base,'offset','address')]
        return setup+[call('stack.'+op,'S','C','K',n,'address','stack',v),inv(setup)]
    common=[I(n) for n in ['D','S','E','R','C','K','P','Did','F','DepthLimit','StackLimit','Ret','Dest']]+[W(n,w) for n,w in [('depth','D'),('sp','S'),('fp','S'),('stack',times('C','K')),('dfp','S'),('dsp','S'),('ddepth','D'),('flag',1),('faulted',1),('code','F')]]
    body=[call('value.call.flag','D','S','E','R','DepthLimit','StackLimit','depth','sp','flag')]
    success=[visit('Captures',[copy('width','source','capture')])]
    success+=[visit('Locations',address('sp',plus(3,'slot'),'exchange','width','location'))]
    for offset,n,literal,v in [(0,'P','Ret','header'),(1,'S',None,'fp'),(2,'Did','Dest','header')]:
        if literal is not None:
            h='header'+str(offset)
            success+=[alloc(h,n),call('value.literal',n,literal,h)]+address('sp',offset,'store',n,h)+[inv([call('value.literal',n,literal,h)]),release(h)]
        else:success+=address('sp',offset,'store',n,v)
    success+=[visit('Captures',[copy('width','capture','parameter')]),copy('S','sp','dfp')]
    for n,amount,source,target in [('S','R','sp','dsp'),('D',1,'depth','ddepth')]:
        inc='increment'+n
        success+=[alloc(inc,n),call('value.literal',n,amount,inc),call('bind.value.add',n,source,inc,target),inv([call('value.literal',n,amount,inc)]),release(inc)]
    success+=[exchange('S','dfp','fp'),exchange('S','dsp','sp'),exchange('D','ddepth','depth')]
    body+=[gate('x',wire('flag')),under('flag',success),gate('x',wire('flag')),gate('cx',wire('flag'),wire('faulted')),under('flag',[call('value.literal','F',3,'code')])]
    scratch=['offset','address','header0','header2','incrementS','incrementD']
    out.append(recipe('lift.call',common+[locations,captures],body,'§3.7 C1 captures; C2 declaration-order stack push; C3 ret/fp/destination headers; C4 parameter bind; C5 compute all counters then fp/sp/depth exchanges; effective fault 3. R=3+len(Locations), C=2^S_cells, K=W_cell, E=max(S,bit_length(R))+1; F>=4; Ret=k+1; Dest=id(w). Capture/assignment slots and flag initially zero. Stack record is clean when success; source descriptors include recursive aliases.',scratch))
    # Ret has a full-width delivery slot: narrower candidate locations use
    # its low width; only the header-selected, correctly typed id can fire.
    params=[I(n) for n in ['D','S','C','K','P','Did','F','N','V']]+[W(n,w) for n,w in [('depth','D'),('fp','S'),('stack',times('C','K')),('dfp','S'),('ddepth','D'),('flag',1),('faulted',1),('code','F'),('v','V'),('delivery','N'),('succ','P')]]+[locations,globals]
    flag=[alloc('zero','D'),call('bind.value.icmp.eq','D','depth','zero','flag'),release('zero')]
    success=address('fp',0,'copy','P','succ')+[copy('V','v','delivery')]
    success+=[visit('Locations',address('fp',plus(3,'slot'),'exchange','width','location'))]
    success+=[alloc('destination','Did')]+address('fp',2,'copy','Did','destination')
    lit=call('value.literal','Did','id','id_constant');eq=call('bind.value.icmp.eq','Did','destination','id_constant','equal')
    success+=[visit('Globals',[alloc('id_constant','Did'),lit,alloc('equal',1),eq,under('equal',[exchange('width','delivery','location')]),inv([eq]),release('equal'),inv([lit]),release('id_constant')])]
    success+=[inv(address('fp',2,'copy','Did','destination')),release('destination')]
    success+=address('fp',1,'copy','S','dfp')
    success+=[alloc('one','D'),call('value.literal','D',1,'one'),call('bind.value.sub','D','depth','one','ddepth'),inv([call('value.literal','D',1,'one')]),release('one'),exchange('S','dfp','fp'),exchange('D','ddepth','depth')]
    body=flag+[gate('x',wire('flag')),under('flag',success),gate('x',wire('flag')),gate('cx',wire('flag'),wire('faulted')),under('flag',[call('value.literal','F',4,'code')])]
    out.append(recipe('lift.ret',params,body,'§3.7 R1 stack return index; R2 result capture; R3 pop; R4 complete global-id delivery; R5 compute fp and depth-1 before exchanges. Inverse restores fp before reading any stack header or delivery condition. N is widest frame delivery slot, V is result width; selected destination has width V. Stack header/location history valid for nonzero depth; flag/slots clean. F>=4.',scratch+['zero','destination','id_constant','equal','one']))
    return out

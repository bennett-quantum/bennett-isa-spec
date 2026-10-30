"""Generic ordered scans and byte-independent memory wire operations."""
def family(api):
    recipe,wire,gate,loop,inv,plus,minus,times,alloc,release,Circuit=[api[k] for k in ['recipe','wire','gate','loop','inv','plus','minus','times','alloc','release','Circuit']]
    def I(n):return {'name':n,'type':'integer'}
    def W(n,w):return {'name':n,'type':'wire','width':w}
    def call(k,*args):return {'invoke':k,'args':list(args)}
    def inc(name,n):return loop('incbit',n,[gate('mcx',{'range':name,'from':0,'to':minus(minus(n,1),'incbit')},wire(name,minus(minus(n,1),'incbit')))])
    def rg(n,fr,to):return {'range':n,'from':fr,'to':to}
    out=[]
    for op in ['copy','store','exchange']:
        eq=call('bind.value.icmp.eq','A','a','counter','eq')
        cell=wire('cells',plus(times('address','W'),'bit'));v=wire('v','bit')
        effect={'under':'eq','body':[loop('bit','W',[gate('swap',cell,v) if op=='exchange' else gate('cx',cell,v) if op=='copy' else gate('cx',v,cell)])]}
        body=[alloc('counter','A'),alloc('eq',1),loop('address','C',[eq,effect,eq,inc('counter','A')]),inv([loop('address','C',[inc('counter','A')])]),release('eq'),release('counter')]
        params=[I('A'),I('C'),I('W'),W('a','A'),W('cells',times('C','W')),W('v','W')]
        out.append(recipe('dispatch.'+op,params,body,'§11.1 one-hot address dispatch: C=2^A cells in increasing address order, equality uncomputed before the next address.',['counter','eq']))
        if op!='copy':out.append(recipe('qstore' if op=='store' else 'qexchm',params,[call('dispatch.'+op,'A','C','W','a','cells','v')],'§4.6: for every cell i under a==i, '+('qxor v cell_i' if op=='store' else 'qexch v cell_i')+'.'))
    # A table entry has P-bit base and size. Length has L bits. E is exactly
    # max(P,L)+1, T is atop's declared counter width, C table capacity.
    for op in ['inrange','free.flag']:
        params=[I('P'),I('L'),I('E'),I('C'),I('T'),W('p','P'),W('n','L'),W('base',times('C','P')),W('size',times('C','P')),W('live','C'),W('pinned','C'),W('atop','T'),W('t',1)]
        c=Circuit();fold=c.temp(plus('C',1));counter=c.temp('T');start=len(c.body)
        active=c.temp();c.body.append(call('bind.value.icmp.ult','T',counter,'atop',active))
        base=rg('base',times('entry','P'),times(plus('entry',1),'P'))
        if op=='inrange':
            px=c.temp('E');nx=c.temp('E');bx=c.temp('E');sx=c.temp('E');endp=c.temp('E');endb=c.temp('E');gebase=c.temp();geend=c.temp()
            c.body.extend([loop('bit','P',[gate('cx',wire('p','bit'),wire(px,'bit')),gate('cx',wire('base',plus(times('entry','P'),'bit')),wire(bx,'bit')),gate('cx',wire('size',plus(times('entry','P'),'bit')),wire(sx,'bit'))]),loop('bit','L',[gate('cx',wire('n','bit'),wire(nx,'bit'))]),call('bind.value.add','E',px,nx,endp),call('bind.value.add','E',bx,sx,endb),call('bind.value.icmp.ule','E',bx,px,gebase),call('bind.value.icmp.ule','E',endp,endb,geend)])
            pred=('and',('and',('and',wire(active),wire('live','entry')),wire(gebase)),wire(geend))
        else:
            eq=c.temp();c.body.append(call('bind.value.icmp.eq','P','p',base,eq))
            pred=('and',('and',('and',wire(active),wire(eq)),wire('live','entry')),('not',wire('pinned','entry')))
        entry=c.temp();c.boolean(pred,wire(entry));compute=c.body[start:]
        c.boolean(('or',wire(fold,'entry'),wire(entry)),wire(fold,plus('entry',1)))
        c.body.extend([inv(compute),inc(counter,'T')]);c.body[start:]=[loop('entry','C',c.body[start:])]
        compute=c.body[:]
        c.boolean(wire(fold,'C') if op=='inrange' else ('not',wire(fold,'C')),wire('t'));c.body.append(inv(compute))
        out.append(c.recipe('scan.'+op,params,'§4.2: ordered linear table scan; entry i<atop, live, widened no-wrap range/free predicate; §11.1 ordered OR fold with immediate entry scratch uncomputation.'))
    # Row effects on existing frame/state wires. Generic allocation results,
    # mapped byte layout interfaces and image initialization remain separate.
    flagparams=[I('P'),I('L'),I('E'),I('C'),I('T'),W('p','P'),W('n','L'),W('base',times('C','P')),W('size',times('C','P')),W('live','C'),W('pinned','C'),W('atop','T'),W('flag',1),W('faulted',1),W('code',4)]
    flagargs=['P','L','E','C','T','p','n','base','size','live','pinned','atop','flag']
    out.append(recipe('lift.boundscheck',flagparams,[gate('x',wire('flag')),call('scan.inrange',*flagargs),gate('cx',wire('flag'),wire('faulted')),{'under':'flag','body':[call('value.literal',4,7,'code')]}],'§4.2 lift.boundscheck: flag=not inrange(p,n), then code ^= flag*7 and faulted ^= flag.'))
    c=Circuit();counter=c.temp('T');active=c.temp();eq=c.temp();hit=c.temp();start=len(c.body)
    base=rg('base',times('entry','P'),times(plus('entry',1),'P'))
    c.body.extend([call('bind.value.icmp.ult','T',counter,'atop',active),call('bind.value.icmp.eq','P','p',base,eq)])
    c.boolean(('and',wire(active),wire(eq)),wire(hit));match=c.body[start:]
    c.body.extend([gate('ccx',wire('flag'),wire(hit),wire('live','entry')),inv(match),inc(counter,'T')]);c.body[start:]=[loop('entry','C',c.body[start:])]
    c.body.extend([inv([loop('entry','C',[inc(counter,'T')])]),release(hit),release(eq),release(active),release(counter)])
    body=[call('scan.free.flag',*flagargs),gate('x',wire('flag'))]+c.body+[gate('x',wire('flag')),gate('cx',wire('flag'),wire('faulted')),{'under':'flag','body':[call('value.literal',4,7,'code')]}]
    out.append(recipe('lift.free',flagparams,body,'§4.2 lift.free: old-state free flag first; under not flag toggle every live_i whose i<atop and base_i=p; uncompute match scratch and apply fault writes.',c.scratch))
    return out


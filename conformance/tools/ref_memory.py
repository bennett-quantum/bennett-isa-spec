"""Generic ordered scans and byte-independent memory wire operations."""
def family(api):
    recipe,wire,gate,loop,inv,plus,minus,times,alloc,release,Circuit=[api[k] for k in ['recipe','wire','gate','loop','inv','plus','minus','times','alloc','release','Circuit']]
    def I(n):return {'name':n,'type':'integer'}
    def W(n,w):return {'name':n,'type':'wire','width':w}
    def call(k,*args):return {'invoke':k,'args':list(args)}
    def rg(n,fr,to):return {'range':n,'from':fr,'to':to}
    out=[]
    for op in ['copy','store','exchange']:
        eq=call('bind.value.icmp.eq','A','a','counter','eq')
        cell=wire('cells',plus(times('address','W'),'bit'));v=wire('v','bit')
        effect={'under':'eq','body':[loop('bit','W',[gate('swap',cell,v) if op=='exchange' else gate('cx',cell,v) if op=='copy' else gate('cx',v,cell)])]}
        literal=call('value.literal','A','address','counter')
        body=[loop('address','C',[alloc('counter','A'),literal,alloc('eq',1),eq,effect,inv([eq]),release('eq'),inv([literal]),release('counter')])]
        params=[I('A'),I('C'),I('W'),W('a','A'),W('cells',times('C','W')),W('v','W')]
        out.append(recipe('dispatch.'+op,params,body,'§11.1 one-hot address dispatch: C=2^A cells in increasing address order, equality uncomputed before the next address.',['counter','eq']))
        if op!='copy':out.append(recipe('qstore' if op=='store' else 'qexchm',params,[call('dispatch.'+op,'A','C','W','a','cells','v')],'§4.6: for every cell i under a==i, '+('qxor v cell_i' if op=='store' else 'qexch v cell_i')+'.'))
    # A table entry has P-bit base and size. Length has L bits. E is exactly
    # max(P,L)+1, T is atop's declared counter width, C table capacity.
    for op in ['inrange','free.flag']:
        params=[I('P'),I('L'),I('E'),I('C'),I('T'),W('p','P'),W('n','L'),W('base',times('C','P')),W('size',times('C','P')),W('live','C'),W('pinned','C'),W('atop','T'),W('t',1)]
        c=Circuit();fold=c.temp(plus('C',1));start=len(c.body);counter=c.temp('T');c.body.append(call('value.literal','T','entry',counter))
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
        c.body.append(inv(compute));c.body[start:]=[loop('entry','C',c.body[start:])]
        compute=c.body[:]
        c.boolean(wire(fold,'C') if op=='inrange' else ('not',wire(fold,'C')),wire('t'));c.body.append(inv(compute))
        out.append(c.recipe('scan.'+op,params,'§4.2: ordered linear table scan; entry i<atop, live, widened no-wrap range/free predicate; §11.1 ordered OR fold with immediate entry scratch uncomputation.'))
    # Row effects on existing frame/state wires. Generic allocation results,
    # mapped byte layout interfaces and image initialization remain separate.
    flagparams=[I('P'),I('L'),I('E'),I('C'),I('T'),I('F'),W('p','P'),W('n','L'),W('base',times('C','P')),W('size',times('C','P')),W('live','C'),W('pinned','C'),W('atop','T'),W('flag',1),W('faulted',1),W('code','F')]
    flagargs=['P','L','E','C','T','p','n','base','size','live','pinned','atop','flag']
    out.append(recipe('lift.boundscheck',flagparams,[gate('x',wire('flag')),call('scan.inrange',*flagargs),gate('cx',wire('flag'),wire('faulted')),{'under':'flag','body':[call('value.literal','F',7,'code')]}],'§4.2 lift.boundscheck: flag=not inrange(p,n), then code ^= flag*7 and faulted ^= flag.'))
    c=Circuit();counter=c.temp('T');active=c.temp();eq=c.temp();hit=c.temp()
    base=rg('base',times('entry','P'),times(plus('entry',1),'P'))
    c.body.extend([call('value.literal','T','entry',counter),call('bind.value.icmp.ult','T',counter,'atop',active),call('bind.value.icmp.eq','P','p',base,eq)])
    c.boolean(('and',('and',wire(active),wire(eq)),('not',wire('pinned','entry'))),wire(hit));match=c.body[:]
    c.body.extend([gate('ccx',wire('flag'),wire(hit),wire('live','entry')),inv(match)]);c.body=[loop('entry','C',c.body)]
    body=[call('scan.free.flag',*flagargs),gate('x',wire('flag'))]+c.body+[gate('x',wire('flag')),gate('cx',wire('flag'),wire('faulted')),{'under':'flag','body':[call('value.literal','F',7,'code')]}]
    out.append(recipe('lift.free',flagparams,body,'§4.2 lift.free: old-state free flag first; under not flag toggle every live_i whose i<atop, base_i=p and not pinned_i; uncompute match scratch and apply fault writes.',c.scratch))
    # All four assignment values are computed from old counters before any
    # exchange. Record cells pack base[P], size[P], live, pinned, in that order.
    params=[I(n) for n in ['P','N','H','T','E','K','J','C','Base','Limit','F']]+[W('n','N'),W('used','H'),W('atop','T'),W('table',times('C',plus(times(2,'P'),2))),W('w','P'),W('dp','P'),W('de',plus(times(2,'P'),2)),W('du','H'),W('da','T'),W('flag',1),W('faulted',1),W('code','F')]
    def prospective(c):
        used=c.temp('E');mx=c.temp('E');zero=c.temp('N');iszero=c.temp();total=c.temp('E')
        c.body.extend([loop('bit','H',[gate('cx',wire('used','bit'),wire(used,'bit'))]),loop('bit','N',[gate('cx',wire('n','bit'),wire(mx,'bit'))]),call('bind.value.icmp.eq','N','n',zero,iszero),gate('cx',wire(iszero),wire(mx)),call('bind.value.add','E',used,mx,total)])
        return total
    c=Circuit();total=prospective(c);limit=c.temp('E');capacity=c.temp('T');heapok=c.temp();tableok=c.temp();c.body.extend([call('value.literal','E','Limit',limit),call('value.literal','T','C',capacity),call('bind.value.icmp.ule','E',total,limit,heapok),call('bind.value.icmp.ult','T','atop',capacity,tableok)]);flagcompute=c.body[:]
    c.boolean(('or',('not',wire(heapok)),('not',wire(tableok))),wire('flag'));c.body.append(inv(flagcompute))
    d=Circuit();d.serial=c.serial;total=prospective(d);base=d.temp('P');used=d.temp('P');one=d.temp('T');d.body.extend([call('value.literal','P','Base',base),loop('bit','K',[gate('cx',wire('used','bit'),wire(used,'bit'))]),call('value.literal','T',1,one)]);setup=d.body[:]
    d.body.extend([call('bind.value.add','P',base,used,'dp'),call('bind.value.add','P',base,used,rg('de',0,'P')),loop('bit','J',[gate('cx',wire('n','bit'),wire('de',plus('P','bit')))]),gate('x',wire('de',times(2,'P'))),loop('bit','H',[gate('cx',wire(total,'bit'),wire('du','bit'))]),call('bind.value.add','T','atop',one,'da'),inv(setup)])
    def exchange(a,b,n):return loop('bit',n,[gate('swap',wire(a,'bit'),wire(b,'bit'))])
    d.body.extend([exchange('dp','w','P'),call('dispatch.exchange','T','C',plus(times(2,'P'),2),'atop','table','de'),exchange('du','used','H'),exchange('da','atop','T')])
    body=c.body+[gate('x',wire('flag')),{'under':'flag','body':d.body},gate('x',wire('flag')),gate('cx',wire('flag'),wire('faulted')),{'under':'flag','body':[call('value.literal','F',2,'code')]}]
    out.append(recipe('lift.alloc',params,body,'§4.2 lift.alloc: flag from widened prospective heap use and table capacity; compute pointer, packed entry, heap_used and atop slots before exchanges in precisely that order. E=max(H,N)+1, K=min(P,H), J=min(P,N); Base/Limit are resolved layout/budget integers. Inverse restores atop before addressing table[atop].',c.scratch+d.scratch))
    return out



def call_flag(api):
    Circuit,wire,gate,loop,inv,plus,recipe=[api[k] for k in ['Circuit','wire','gate','loop','inv','plus','recipe']]
    c=Circuit();flags=[]
    def call(k,*args):return {'invoke':k,'args':list(args)}
    for source,width,total_width,amount,limit in [('depth','D',plus('D',1),1,'DepthLimit'),('sp','S','E','R','StackLimit')]:
        src=c.temp(total_width);constant=c.temp(total_width);total=c.temp(total_width);lim=c.temp(total_width);ok=c.temp()
        c.body.extend([loop('bit',width,[gate('cx',wire(source,'bit'),wire(src,'bit'))]),call('value.literal',total_width,amount,constant),call('bind.value.add',total_width,src,constant,total),call('value.literal',total_width,limit,lim),call('bind.value.icmp.ule',total_width,total,lim,ok)])
        flags.append(('not',wire(ok)))
    result=c.temp();c.boolean(('or',*flags),wire(result));compute=c.body[:];c.body.extend([gate('cx',wire(result),wire('t')),inv(compute)])
    params=[{'name':n,'type':'integer'} for n in ['D','S','E','R','DepthLimit','StackLimit']]+[{'name':n,'type':'wire','width':w} for n,w in [('depth','D'),('sp','S'),('t',1)]]
    return [c.recipe('value.call.flag',params,'§3.7: prospective depth+1 and sp+R_f use no wrap, then flag is the OR of their budget excesses. E=max(S,bit_length(R))+1, R=3+location_count; budget limits are resolved classical integers.')]

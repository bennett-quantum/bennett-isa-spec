"""Typed row and structured constructor catalog. No private runtime leaves."""
def formal(name,ty='integer',width=None):
    f={'name':name,'type':ty}
    if width is not None:f['width']=width
    return f
CLASSICAL=['const','add','sub','mul','and','or','xor','shl','lshr','ashr','udiv','urem','sdiv','srem','icmp.eq','icmp.ne','icmp.ult','icmp.ule','icmp.slt','icmp.sle','select','zext','sext','trunc','ptrtoint','inttoptr','load','store','alloc','free','br','call','ret','uncall','trap','fault','qreg.new','qreg.free','qreg.slice','qreg.concat','qreg.width','qreg.id','qreg.from_id','measure','measure.iN','budget.mint','effect','hostcall']
def classical(api):
    recipe=api['recipe']
    return [recipe('cc.'+op,[formal('E','event')],[{'classical':op,'point':'E'}], '0.9 §4.1: '+op+'; classical event at the bound §8.2 point, retaining §7.4 history.') for op in CLASSICAL]

def rows(api):
    recipe,wire,gate,loop,inv,plus,minus,times=[api[k] for k in ['recipe','wire','gate','loop','inv','plus','minus','times']]
    alloc,release=api['alloc'],api['release']
    I=lambda n:formal(n)
    W=lambda n,w:formal(n,'wire',w)
    invoke=lambda k,args:{'invoke':k,'args':args}
    under=lambda w,b:{'under':w,'body':b}
    swap=lambda a,b,n:loop('bit',n,[gate('swap',wire(a,'bit'),wire(b,'bit'))])
    out=[]
    # Explicit typed adapters bind the legacy helper declaration names.
    for op in ['add','sub','mul','and','or','xor','icmp.eq','icmp.ne','icmp.ult','icmp.ule','icmp.slt','icmp.sle']:
        out.append(recipe('bind.value.'+op,[I('N'),W('a','N'),W('b','N'),W('t',1 if op.startswith('icmp.') else 'N')],[invoke('value.'+op,['N'])],'§11.1 typed binding of the §4.1 value circuit.'))
    # Width-uniform shifts: scan j with an N-bit constant counter. All >=N
    # cases contribute zero (including arithmetic shift, per §4.1).
    inc=[loop('p','N',[gate('mcx',{'range':'counter','from':0,'to':minus(minus('N',1),'p')},wire('counter',minus(minus('N',1),'p')))])]
    for op in ['shl','lshr','ashr']:
        eq=invoke('bind.value.icmp.eq',['N','b','counter','eq'])
        if op=='shl':body=[{'for':'bit','from':'j','to':'N','body':[gate('ccx',wire('eq'),wire('a',minus('bit','j')),wire('t','bit'))]}]
        else:
            body=[loop('bit',minus('N','j'),[gate('ccx',wire('eq'),wire('a',plus('bit','j')),wire('t','bit'))])]
            if op=='ashr':body += [{'for':'bit','from':minus('N','j'),'to':'N','body':[gate('ccx',wire('eq'),wire('a',minus('N',1)),wire('t','bit'))]}]
        out.append(recipe('bind.value.'+op,[I('N'),W('a','N'),W('b','N'),W('t','N')],[alloc('counter','N'),alloc('eq',1),loop('j','N',[eq]+body+[eq]+inc),inv([loop('j','N',inc)]),release('eq'),release('counter')],'§4.1: shift >= N gives zero; §11.1 increasing shift-amount scan.', ['counter','eq']))
    for op in ['add','sub','mul','and','or','xor','shl','lshr','ashr','icmp.eq','icmp.ne','icmp.ult','icmp.ule','icmp.slt','icmp.sle']:
        width=1 if op.startswith('icmp.') else 'N'
        out.append(recipe('lift.'+op,[I('N'),W('a','N'),W('b','N'),W('t',width),W('w',width)],[invoke('bind.value.'+op,['N','a','b','t']),swap('t','w',width)],'§4.2: compute every value into its assignment slot before exchanging with the destination.'))
    for op in ['mov','retval']:
        body=[loop('bit','N',[gate('cx',wire('a','bit'),wire('t','bit'))])]
        params=[I('N'),W('a','N'),W('t','N')]
        if op=='mov':params+=[W('w','N')];body+=[swap('t','w','N')]
        out.append(recipe('lift.'+op,params,body,'§4.2: '+('w = lift.mov a; slot retains old w.' if op=='mov' else 'result_val ^= v; successor HALT.')))
    for op in ['zext','sext','trunc']:
        out.append(recipe('lift.'+op,[I('N'),I('M'),W('a','N'),W('t','M'),W('w','M')],[invoke('value.'+op,['N','M']),swap('t','w','M')],'§4.2 lift.cast: out-of-place extension/truncation then assignment exchange.'))
    increment=[loop('bit','N',[gate('mcx',{'range':'literal','from':0,'to':minus(minus('N',1),'bit')},wire('literal',minus(minus('N',1),'bit')))])]
    compute=[loop('count','C',increment)]
    out.append(recipe('value.literal',[I('N'),I('C'),W('t','N')],[alloc('literal','N')]+compute+[loop('bit','N',[gate('cx',wire('literal','bit'),wire('t','bit'))]),inv(compute),release('literal')],'§4.1 const: XOR the literal bit vector into the slot; integer C is its canonical unsigned value.',['literal']))
    for op in ['const','label','addr']:
        out.append(recipe('lift.'+op,[I('N'),I('C'),W('t','N'),W('w','N')],[invoke('value.literal',['N','C','t']),swap('t','w','N')],'§4.2 lift.'+op+': resolved literal/canonical label/layout address computed into its slot then exchanged.'))
    out.append(recipe('lift.parmov',[I('N'),W('a','N'),W('t','N'),W('w','N')],[loop('bit','N',[gate('cx',wire('a','bit'),wire('t','bit'))]),swap('t','w','N')],'§4.2: concatenate source, slot and destination bundles in listed slot order, total N bits; all computes precede every exchange, including aliases.'))
    out.append(recipe('lift.select',[I('N'),W('a','N'),W('b','N'),W('p',1),W('t','N'),W('w','N')],[invoke('value.select',['N']),swap('t','w','N')],'§4.2 lift.select: compute from old operands then exchange the assignment slot.'))
    for op in ['udiv','urem','sdiv','srem']:
        out.append(recipe('bind.value.'+op,[I('N'),W('a','N'),W('b','N'),W('t','N'),W('f',4)],[invoke('value.'+op,['N'])],'§4.1 division: typed adapter of the fault-aware value helper.'))
        c=api['Circuit']();c.boolean(('or',('or',wire('f'),wire('f',1)),('or',wire('f',2),wire('f',3))),wire('flag'))
        body=[alloc('f',4),alloc('unused','N'),invoke('bind.value.'+op,['N','a','b','t','f'])]+c.body+[gate('x',wire('flag')),under('flag',[swap('t','w','N')]),gate('x',wire('flag')),gate('cx',wire('flag'),wire('faulted')),loop('bit',4,[gate('cx',wire('f','bit'),wire('code','bit'))]),under('flag',[invoke('bind.value.'+op,['N','a','b','unused','f'])]),release('unused'),release('f')]
        out.append(recipe('lift.'+op,[I('N'),W('a','N'),W('b','N'),W('t','N'),W('w','N'),W('flag',1),W('faulted',1),W('code',4)],body,'§4.2 lift.div: compute flag from old inputs; exchange only on success; XOR code 5/6 and faulted after effects. Fault-only recomputation clears code scratch from unchanged faulting operands.', ['f','unused']+c.scratch))
    out.append(recipe('lift.faultif',[I('C'),W('p',1),W('flag',1),W('faulted',1),W('code',4)],[gate('cx',wire('p'),wire('flag')),gate('cx',wire('flag'),wire('faulted')),under('flag',[invoke('value.literal',[4,'C','code'])])],'§4.2 lift.faultif: flag=p, faulted XOR flag, code XOR flag*C.'))
    out.append(recipe('fault',[I('C'),W('faulted',1),W('code',4)],[gate('x',wire('faulted')),invoke('value.literal',[4,'C','code'])],'§4.2 fault C: code ^= C; faulted ^= 1; successor HALT.'))
    return out


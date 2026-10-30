"""Typed row and structured constructor catalog. No private runtime leaves."""
def formal(name,ty='integer',width=None):
    f={'name':name,'type':ty}
    if width is not None:f['width']=width
    return f
CLASSICAL=['const','add','sub','mul','and','or','xor','shl','lshr','ashr','udiv','urem','sdiv','srem','icmp.eq','icmp.ne','icmp.ult','icmp.ule','icmp.slt','icmp.sle','select','zext','sext','trunc','ptrtoint','inttoptr','load','store','alloc','free','br','call','ret','uncall','trap','fault','qreg.new','qreg.free','qreg.slice','qreg.concat','qreg.width','qreg.id','qreg.from_id','measure','measure.iN','budget.mint','effect','hostcall']
def classical(api):
    recipe=api['recipe']
    return [recipe('cc.'+op,[formal('E','event')],[{'classical':'measure' if op=='measure.iN' else op,'point':'E'}], '0.9 §4.1: '+op+'; classical event at the bound §8.2 point, retaining §7.4 history.') for op in CLASSICAL]

def rows(api):
    recipe,wire,gate,loop,inv,plus,minus,times=[api[k] for k in ['recipe','wire','gate','loop','inv','plus','minus','times']]
    alloc,release=api['alloc'],api['release']
    I=lambda n:formal(n)
    W=lambda n,w:formal(n,'wire',w)
    invoke=lambda k,args:{'invoke':k,'args':args}
    under=lambda w,b:{'under':w,'body':b}
    swap=lambda a,b,n:loop('bit',n,[gate('swap',wire(a,'bit'),wire(b,'bit'))])
    out=[]
    def captured_binary(op):
        copies=[loop('bit','N',[gate('cx',wire('a','bit'),wire('source0','bit')),gate('cx',wire('b','bit'),wire('source1','bit'))])]
        return [alloc('source0','N'),alloc('source1','N')]+copies+[invoke('bind.value.'+op,['N','source0','source1','t'])]+[inv(copies),release('source1'),release('source0')]
    for kind,ports,g in [('qxor',['s','t'],'cx'),('qexch',['a','b'],'swap')]:
        out.append(recipe('bind.'+kind,[I('N')]+[W(name,'N') for name in ports],[loop('bit','N',[gate(g,*[wire(name,'bit') for name in ports])])],'§4.5: positional wire binding of '+kind+'; operands remain borrowed.'))
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
        out.append(recipe('lift.'+op,[I('N'),W('a','N'),W('b','N'),W('t',width),W('w',width)],captured_binary(op)+[swap('t','w',width)],'§4.2: capture both source operands into distinct clean snapshots, compute every value into its assignment slot, clear snapshots, then exchange with the destination.', ['source0','source1']))
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
    out.append(recipe('bind.value.select',[I('N'),W('a','N'),W('b','N'),W('p',1),W('t','N')],[invoke('value.select',['N'])],'§11.1 typed binding of select inputs and value destination.'))
    out.append(recipe('lift.select',[I('N'),W('a','N'),W('b','N'),W('p',1),W('t','N'),W('w','N')],[alloc('source0','N'),alloc('source1','N'),alloc('sourcep',1),loop('bit','N',[gate('cx',wire('a','bit'),wire('source0','bit')),gate('cx',wire('b','bit'),wire('source1','bit'))]),gate('cx',wire('p'),wire('sourcep')),invoke('bind.value.select',['N','source0','source1','sourcep','t']),gate('cx',wire('p'),wire('sourcep')),loop('bit','N',[gate('cx',wire('a','bit'),wire('source0','bit')),gate('cx',wire('b','bit'),wire('source1','bit'))]),release('sourcep'),release('source1'),release('source0'),swap('t','w','N')],'§4.2 lift.select: capture old operands (including aliases), compute, clear captures, then exchange the assignment slot.', ['source0','source1','sourcep']))
    def zero(c,name,n):
        chain=c.temp(plus(n,1));c.body.append(gate('x',wire(chain)))
        start=len(c.body);c.boolean(('and',wire(chain,'i'),('not',wire(name,'i'))),wire(chain,plus('i',1)));c.body[start:]=[loop('i',n,c.body[start:])]
        return wire(chain,n)
    for op in ['udiv','urem','sdiv','srem']:
        out.append(recipe('bind.value.'+op,[I('N'),W('a','N'),W('b','N'),W('t','N'),W('f',4)],[invoke('value.'+op,['N'])],'§4.1 division: typed adapter of the fault-aware value helper.'))
        c=api['Circuit']();dz=zero(c,'b','N');pred=dz
        if op.startswith('s'):
            lowzero=zero(c,'a',minus('N',1));allones=c.temp(plus('N',1));c.body.append(gate('x',wire(allones)));start=len(c.body)
            c.boolean(('and',wire(allones,'i'),wire('b','i')),wire(allones,plus('i',1)));c.body[start:]=[loop('i','N',c.body[start:])]
            pred=('or',dz,('and',('and',lowzero,wire('a',minus('N',1))),wire(allones,'N')))
        compute=c.body[:];c.boolean(pred,wire('flag'));c.body.append(inv(compute))
        d=api['Circuit']();d.serial=c.serial;dz=zero(d,'b','N');faultcompute=d.body[:]
        d.body.extend([gate('cx',dz,wire('code')),gate('x',dz),gate('cx',dz,wire('code',1)),gate('x',dz),gate('x',wire('code',2)),inv(faultcompute)])
        body=c.body+[alloc('f',4),gate('x',wire('flag')),under('flag',[alloc('source0','N'),alloc('source1','N'),loop('bit','N',[gate('cx',wire('a','bit'),wire('source0','bit')),gate('cx',wire('b','bit'),wire('source1','bit'))]),invoke('bind.value.'+op,['N','source0','source1','t','f']),loop('bit','N',[gate('cx',wire('a','bit'),wire('source0','bit')),gate('cx',wire('b','bit'),wire('source1','bit'))]),release('source1'),release('source0'),swap('t','w','N')]),gate('x',wire('flag')),release('f'),gate('cx',wire('flag'),wire('faulted')),under('flag',d.body)]
        out.append(recipe('lift.'+op,[I('N'),I('F'),W('a','N'),W('b','N'),W('t','N'),W('w','N'),W('flag',1),W('faulted',1),W('code','F')],body,'§4.2/§3.6: compute the old-state flag first, then compute/exchange the value only under not flag, then fault writes. On a fault the operands are unchanged: recompute b==0 to select cause5 versus6 and clean that scratch.', ['f','source0','source1']+c.scratch+d.scratch))
    out.append(recipe('lift.faultif',[I('F'),I('C'),W('p',1),W('flag',1),W('faulted',1),W('code','F')],[gate('cx',wire('p'),wire('flag')),gate('cx',wire('flag'),wire('faulted')),under('flag',[invoke('value.literal',['F',plus('C',times(4,['div',1,plus('C',1)])),'code'])])],'§4.2 lift.faultif: flag=p, faulted XOR flag, code XOR flag*C-prime; C-prime=4 for C=0 else C (§5.4).'))
    out.append(recipe('fault',[I('F'),I('C'),W('faulted',1),W('code','F')],[gate('x',wire('faulted')),invoke('value.literal',['F',plus('C',times(4,['div',1,plus('C',1)])),'code'])],'§4.2 fault C: code ^= effective cause(C); faulted ^= 1; successor HALT.'))
    out.append(recipe('qjump',[I('C')],[],'§4.2 qjump: no row state effect; successor C is emitted by successor.literal in §3.6 phase 3.'))
    out.append(recipe('successor.literal',[I('P'),I('C'),W('next','P')],[invoke('value.literal',['P','C','next'])],'§3.6 phase 3: XOR the static canonical successor into pc-prime.'))
    out.append(recipe('qbranch',[I('P'),I('Then'),I('Else'),W('p',1),W('succ','P')],[under('p',[invoke('value.literal',['P','Then','succ'])]),gate('x',wire('p')),under('p',[invoke('value.literal',['P','Else','succ'])]),gate('x',wire('p'))],'§3.6/§4.2 qbranch: compute the data-dependent successor from the old predicate before effects.'))
    out.append(recipe('successor.flagged',[I('P'),I('Next'),I('HALT'),W('flag',1),W('next','P')],[under('flag',[invoke('value.literal',['P','HALT','next'])]),gate('x',wire('flag')),under('flag',[invoke('value.literal',['P','Next','next'])]),gate('x',wire('flag'))],'§3.6 phase 3: flag chooses HALT, otherwise the static successor.'))
    out.append(recipe('successor.dynamic',[I('P'),I('HALT'),W('flag',1),W('succ','P'),W('next','P')],[under('flag',[invoke('value.literal',['P','HALT','next'])]),gate('x',wire('flag')),under('flag',[loop('bit','P',[gate('cx',wire('succ','bit'),wire('next','bit'))])]),gate('x',wire('flag'))],'§3.6 phase 3: flag chooses HALT, otherwise frame.succ.'))
    c=api['Circuit']();lo=c.temp('P');hi=c.temp('P');below=c.temp();lessend=c.temp();start=len(c.body)
    c.body.extend([invoke('value.literal',['P','First',lo]),invoke('value.literal',['P','End',hi]),invoke('bind.value.icmp.ult',['P','a',lo,below]),invoke('bind.value.icmp.ult',['P','a',hi,lessend])]);compute=c.body[start:]
    c.boolean(('or',wire(below),('not',wire(lessend))),wire('flag'));c.body.append(inv(compute));c.body.extend(release(n) for n in [lessend,below,hi,lo]);c.body.extend([loop('bit','P',[gate('cx',wire('a','bit'),wire('succ','bit'))]),gate('cx',wire('flag'),wire('faulted')),under('flag',[invoke('value.literal',['F',4,'code'])])])
    out.append(c.recipe('qjumpr',[I('P'),I('F'),I('First'),I('End'),W('a','P'),W('succ','P'),W('flag',1),W('faulted',1),W('code','F')],'§4.2 qjumpr: flag=(a<first_f) or (a>=first_f+n_f); frame.succ XOR a; cause4 after effects. End=First+n_f within the canonical function interval.'))
    return out



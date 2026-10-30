"""Catalog constructor generation only; correctness oracle lives separately in Rust."""
def family(api):
    Circuit,wire,gate,loop,inv,plus,minus,times,recipe=[api[k] for k in ['Circuit','wire','gate','loop','inv','plus','minus','times','recipe']]
    out=[]
    def finish(c,k,params=['N']):
        out.append(c.recipe('value.'+k,params,'0.8 §4.1: '+k+'; §4.2: w (under not flag); §17 item 7: clean scratch released in reverse order.'))
    def value(c,e,k):
        start=len(c.body);c.boolean(e,wire('t','i'));c.body[start:]=[loop('i','N',c.body[start:])];finish(c,k)
    for k in ['and','xor','or']:
        c=Circuit();value(c,(k,wire('a','i'),wire('b','i')),k)
    c=Circuit();value(c,('xor',('and',('not',wire('p')),wire('b','i')),('and',wire('p'),wire('a','i'))),'select')
    for k in ['zext','trunc','sext']:
        # Legal parameter domains: zext/sext M>=N; trunc M<=N.
        body=[loop('i','M' if k=='trunc' else 'N',[gate('cx',wire('a','i'),wire('t','i'))])]
        if k=='sext': body.append({'for':'i','from':'N','to':'M','body':[gate('cx',wire('a',minus('N',1)),wire('t','i'))]})
        out.append(recipe('value.'+k,['N','M'],body,'0.8 §4.1: '+k+'; §3.3 equal operand widths.'))
    c=Circuit();acc=c.temp(times(plus('N',1),'N'));part=c.temp(times('N','N'))
    start=len(c.body)
    c.boolean(('and',wire('a','i'),wire('b','j')),wire(part,plus(times('j','N'),plus('i','j'))))
    c.body[start:]=[loop('j','N',[loop('i',minus('N','j'),c.body[start:])])]
    start=len(c.body)
    s,_,_=c.ripple(lambda i:wire(acc,plus(times('j','N'),i)),lambda i:wire(part,plus(times('j','N'),i)),'N')
    comp=c.body[start:];c.body+= [loop('i','N',[gate('cx',wire(s,'i'),wire(acc,plus(times(plus('j',1),'N'),'i')))]),inv(comp)]
    c.body[start:]=[loop('j','N',c.body[start:])]
    comp=c.body[:];c.body += [loop('i','N',[gate('cx',wire(acc,plus(times('N','N'),'i')),wire('t','i'))]),inv(comp)]
    finish(c,'mul')
    def equals(c,a,n,negate=False):
        q=c.temp(plus(n,1));c.body.append(gate('x',wire(q)))
        start=len(c.body);c.boolean(('and',wire(q,'e'),('not',a('e')) if negate else a('e')),wire(q,plus('e',1)))
        c.body[start:]=[loop('e',n,c.body[start:])];return wire(q,n)
    def conditional_negate(c,a,sign,n):
        zero=c.temp(n);s,_,_=c.ripple(lambda i:wire(zero,i),a,n,True)
        v=c.temp(n);start=len(c.body)
        c.boolean(('xor',('and',sign,wire(s,'i')),('and',('not',sign),a('i'))),wire(v,'i'))
        c.body[start:]=[loop('i',n,c.body[start:])];return v
    for k in ['udiv','urem','sdiv','srem']:
        c=Circuit();signed=k.startswith('s')
        dz=equals(c,lambda i:wire('b',i),'N',True)
        overflow=c.temp()
        if signed:
            minlow=equals(c,lambda i:wire('a',i),minus('N',1),True)
            bm=equals(c,lambda i:wire('b',i),'N')
            c.boolean(('and',('and',minlow,wire('a',minus('N',1))),bm),wire(overflow))
        flag=c.temp();c.boolean(('or',dz,wire(overflow)),wire(flag))
        aa=conditional_negate(c,lambda i:wire('a',i),wire('a',minus('N',1)),'N') if signed else 'a'
        bb=conditional_negate(c,lambda i:wire('b',i),wire('b',minus('N',1)),'N') if signed else 'b'
        rem=c.temp(times(plus('N',1),plus('N',1)));quot=c.temp('N')
        start=len(c.body)
        shifted=c.temp(plus('N',1));shstart=len(c.body)
        c.body.append(gate('cx',wire(aa,minus(minus('N',1),'j')),wire(shifted)))
        c.body.append(loop('i','N',[gate('cx',wire(rem,plus(times('j',plus('N',1)),'i')),wire(shifted,plus('i',1)))]))
        zero=c.temp();ext=c.temp(plus('N',1));extstart=len(c.body)
        c.body.append(loop('i','N',[gate('cx',wire(bb,'i'),wire(ext,'i'))]))
        substart=len(c.body);s,carry,_=c.ripple(lambda i:wire(shifted,i),lambda i:wire(ext,i),plus('N',1),True)
        subcomp=c.body[substart:]
        c.body.append(gate('cx',wire(carry,plus('N',1)),wire(quot,minus(minus('N',1),'j'))))
        bitstart=len(c.body)
        pred=wire(carry,plus('N',1))
        c.boolean(('xor',('and',pred,wire(s,'i')),('and',('not',pred),wire(shifted,'i'))),wire(rem,plus(times(plus('j',1),plus('N',1)),'i')))
        c.body[bitstart:]=[loop('i',plus('N',1),c.body[bitstart:])]
        # Clear only the per-stage temporary computations, preserving next rem and q.
        c.body += [inv(subcomp),inv(c.body[extstart:extstart+1]),{'release':ext},{'release':zero},inv(c.body[shstart:shstart+2]),{'release':shifted}]
        c.body[start:]=[loop('j','N',c.body[start:])]
        if k.endswith('rem'):
            result=lambda i:wire(rem,plus(times('N',plus('N',1)),i))
        else: result=lambda i:wire(quot,i)
        if signed:
            sign=wire('a',minus('N',1)) if k=='srem' else ('xor',wire('a',minus('N',1)),wire('b',minus('N',1)))
            sg=c.temp();c.boolean(sign,wire(sg));v=conditional_negate(c,result,wire(sg),'N');result=lambda i:wire(v,i)
        compute=c.body[:]
        start=len(c.body);c.boolean(('and',('not',wire(flag)),result('i')),wire('t','i'));outs=[loop('i','N',c.body[start:])]
        c.body=compute[:]
        # Full-domain helpers XOR fault code 5 or 6; no data output on fault.
        outs += [gate('cx',dz,wire('f',0)),gate('cx',wire(overflow),wire('f',1)),gate('cx',dz,wire('f',2)),gate('cx',wire(overflow),wire('f',2))]
        c.body=compute+outs+[inv(compute)];finish(c,k)
    # Width-specialized shifts avoid inventing an exponent/bit-extraction scalar
    # operator. General-width count-bit constants remain checkpoint work.
    for n in range(1,5):
        for k in ['shl','lshr','ashr']:
            c=Circuit()
            for j in range(n):
                e=tuple(['not',wire('b',0)]) if not j&1 else wire('b',0)
                for bit in range(1,n):e=('and',e,wire('b',bit) if j>>bit&1 else ('not',wire('b',bit)))
                p=c.temp();compstart=len(c.body);c.boolean(e,wire(p));comp=c.body[compstart:]
                for i in range(n):
                    src=i-j if k=='shl' else i+j
                    if src<0:continue
                    if src>=n:
                        if k!='ashr':continue
                        src=n-1
                    c.body.append(gate('ccx',wire(p),wire('a',src),wire('t',i)))
                c.body += [inv(comp),{'release':p}]
            finish(c,k+'.w'+str(n),[])
    return out

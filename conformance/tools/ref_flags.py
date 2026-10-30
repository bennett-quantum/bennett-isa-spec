"""Constructor checkpoints for widened linear scans; no runtime semantic leaves."""
def family(api):
    Circuit,wire,gate,loop,inv,plus,times=[api[k] for k in ['Circuit','wire','gate','loop','inv','plus','times']]
    out=[]
    for entries,n,tag in [(1,'N','e1'),(2,1,'e2.w1'),(2,2,'e2.w2')]:
        def active(i):
            if entries==1:return wire('atop')
            return ('or',wire('atop'),wire('atop',1)) if i==0 else wire('atop',1)
        for op in ['inrange','free.flag']:
            c=Circuit();fold=c.temp()
            if op=='inrange':
                px=c.temp(plus(n,1));nx=c.temp(plus(n,1))
                c.body += [loop('i',n,[gate('cx',wire('p','i'),wire(px,'i')),gate('cx',wire('n','i'),wire(nx,'i'))])]
                endp,_,_=c.ripple(lambda i:wire(px,i),lambda i:wire(nx,i),plus(n,1))
            for entry in range(entries):
                base=lambda i:wire('base',plus(times(entry,n),i))
                if op=='inrange':
                    bx=c.temp(plus(n,1));sx=c.temp(plus(n,1))
                    c.body.append(loop('i',n,[gate('cx',base('i'),wire(bx,'i')),gate('cx',wire('size',plus(times(entry,n),'i')),wire(sx,'i'))]))
                    endb,_,_=c.ripple(lambda i:wire(bx,i),lambda i:wire(sx,i),plus(n,1))
                    _,gebase,_=c.ripple(lambda i:wire(px,i),lambda i:wire(bx,i),plus(n,1),True)
                    _,geend,_=c.ripple(lambda i:wire(endb,i),lambda i:wire(endp,i),plus(n,1),True)
                    pred=('and',('and',('and',active(entry),wire('live',entry)),wire(gebase,plus(n,1))),wire(geend,plus(n,1)))
                else:
                    eq=c.temp(plus(n,1));c.body.append(gate('x',wire(eq)))
                    start=len(c.body);c.boolean(('and',wire(eq,'i'),('not',('xor',wire('p','i'),base('i')))),wire(eq,plus('i',1)))
                    c.body[start:]=[loop('i',n,c.body[start:])]
                    match=('and',('and',active(entry),wire(eq,n)),('not',wire('pinned',entry)))
                    pred=('and',match,wire('live',entry))
                nxt=c.temp();c.boolean(('or',wire(fold),pred),wire(nxt));fold=nxt
            result=c.temp();c.boolean(wire(fold) if op=='inrange' else ('not',wire(fold)),wire(result))
            compute=c.body[:];c.body += [gate('cx',wire(result),wire('t')),inv(compute)]
            out.append(c.recipe('value.'+op+'.'+tag,['N'] if n=='N' else [],'0.8 §4.2: inrange(p,n) = exists i < atop: live_i and base_i <= p and p+n <= base_i+size_i, no wrap; free match_i includes not pinned_i; §17 item 7: linear inrange scans.'))
    # Allocator prospective addition never wraps; A names the table exponent.
    c=Circuit();nz=c.temp(plus('N',1));c.body.append(gate('x',wire(nz)))
    start=len(c.body);c.boolean(('and',wire(nz,'i'),('not',wire('n','i'))),wire(nz,plus('i',1)))
    c.body[start:]=[loop('i','N',c.body[start:])]
    mx=c.temp(plus('N',1));used=c.temp(plus('N',1))
    c.body.append(loop('i','N',[gate('cx',wire('n','i'),wire(mx,'i')),gate('cx',wire('used','i'),wire(used,'i'))]))
    c.body.append(gate('cx',wire(nz,'N'),wire(mx)))
    total,_,_=c.ripple(lambda i:wire(used,i),lambda i:wire(mx,i),plus('N',1))
    _,ok,_=c.ripple(lambda i:wire('limit',i),lambda i:wire(total,i),plus('N',1),True)
    result=c.temp();c.boolean(('or',('not',wire(ok,plus('N',1))),wire('atop','A')),wire(result))
    compute=c.body[:];c.body += [gate('cx',wire(result),wire('t')),inv(compute)]
    out.append(c.recipe('value.alloc.flag',['N','A'],'0.8 §4.2: flag = (heap_used + max(n,1) > heap_limit) or (atop >= 2^A_tab); §3.7 prospective sums use no wrap.'))
    for n in range(1,5):
        c=Circuit();preds=[]
        for name,limit,amount in [('depth','depth_limit',1),('sp','stack_limit',4)]:
            w=max(n,amount.bit_length())+1
            src=c.temp(w);const=c.temp(w)
            c.body.append(loop('i',n,[gate('cx',wire(name,'i'),wire(src,'i'))]))
            for bit in range(w):
                if amount>>bit&1:c.body.append(gate('x',wire(const,bit)))
            total,_,_=c.ripple(lambda i:wire(src,i),lambda i:wire(const,i),w)
            _,ok,_=c.ripple(lambda i:wire(limit,i),lambda i:wire(total,i),w,True)
            preds.append(('not',wire(ok,w)))
        result=c.temp();c.boolean(('or',*preds),wire(result));compute=c.body[:]
        c.body += [gate('cx',wire(result),wire('t')),inv(compute)]
        out.append(c.recipe('value.call.flag.rf4.w'+str(n),[],'0.8 §3.7: flag := (depth + 1 > depth_limit) or (sp + R_f > stack_limit); R_f=3+m-prime, here m-prime=1 and R_f=4; all prospective sums use no wrap.'))
    return out

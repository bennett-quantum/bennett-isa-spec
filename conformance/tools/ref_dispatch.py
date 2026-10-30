"""One-hot two-cell dispatch constructors; region/ABI binding stays blocked."""
def family(api):
    Circuit,wire,gate,loop,inv,plus,times=[api[k] for k in ['Circuit','wire','gate','loop','inv','plus','times']]
    out=[]
    for op in ['copy','store','exchange']:
        c=Circuit()
        for address in range(2):
            pred=c.temp();start=len(c.body)
            c.boolean(wire('a') if address else ('not',wire('a')),wire(pred))
            compute=c.body[start:]
            cell=wire('cells',plus(times(address,'W'),'i'));value=wire('v','i')
            if op=='copy':gates=[gate('ccx',wire(pred),cell,value)]
            elif op=='store':gates=[gate('ccx',wire(pred),value,cell)]
            else:gates=[gate('cx',value,cell),gate('ccx',wire(pred),cell,value),gate('cx',value,cell)]
            c.body += [loop('i','W',gates),inv(compute),{'release':pred}]
        out.append(c.recipe('dispatch.'+op+'.a1',['W'],'0.8 §4.6: for each cell i: under a == i, qxor cell_i d / qxor v cell_i / qexch v cell_i; §17 item 7: one-hot address dispatch.'))
    return out

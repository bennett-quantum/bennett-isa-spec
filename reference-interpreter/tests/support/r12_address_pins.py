"""Hand derivation of codex R12-B02: lift.addr is local, no dependency/SCC.
Two rows: addr writes base 16, retval returns it. B=1 stops before retval.
The db control changes exactly header region, addr operand and layout key.
All supplied pins are asserted before writing; hashlib/json only.
"""
import hashlib
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
def j(v):
    return json.dumps(v,sort_keys=True,separators=(",",":"))
def sha(s):
    return hashlib.sha256(s.encode()).hexdigest()
HEADER = "bennett-object isa=0.8 object=0.8 launch=0.8 sched=0.8 cert=0.8 coverage=0.8 target=0.8 ptr_width=32\n"
cases = {}
for r,h,op in [
    ("o","05827c10c37afc2feef8ccdf0d26dbe25c92204d37d732db954e8595e0d22e91","44158d3308861c21d1a001fdf3b8d80b5218f281580322e6406e97f22829b536"),
    ("db","cb28a9580045f28923bf1c5fc328d7b8e89038e3b0294afc1b1a9291d7e3b5e1","94499ff501bc706735df0b948fee0b7ed89f6f615f09d1a94934b940beb10755")]:
    sig = dict(abi="0.8",args=[],fault=4,**{"in":[]},kind="oracle",mem=[dict(A=0,W=8,role="in")],tgt=32,tier="q")
    body = f"oracle @o () (t:wire[32]) (f:wire[4]) ({r}:region[0,8] in) ()\nregs {{ p:reg[32] }}\n^entry():\np = lift.addr @{r}\nlift.retval p\n"
    env = "isa=0.8\nptr_width=32\n"+j(sig)+"\n"+body+"layout A_tab 0\nlayout H 6\nlayout S_cells 0\nlayout heap_base 48\nlayout heap_limit_max 0\n"+f"layout interface_regions.o.{r} {{\"base\":16}}\n"+"layout null_size 1\nlayout ptr_width 32\n"
    layout = dict(A_tab=0,H=6,S_cells=0,globals={},heap_base=48,heap_limit_max=0,interface_regions={"o":{r:dict(base=16)}},null_size=1,ptr_width=32)
    symbols = {"0":dict(interface_hash=h,kind="oracle",name="o",signature=sig)}
    obj = HEADER+"\nsection symbols\n"+j(symbols)+"\n\nsection functions\n"+body+"\nsection layout\n"+j(layout)+"\n"
    assert sha(env)==h, (r,sha(env))
    assert sha(obj)==op, (r,sha(obj))
    cases[r]=dict(text=obj,envelope=env,hash=h,object_hash=op)
    print(r,h,op)
# Every symbol kind, defined and imported. A data symbol is not a global
# without a layout.globals entry. None of these spellings is an address
# dependency inside @o; even an unpinned import leaves @o's closure known.
base = cases["db"]
for kind in ["cc","r","oracle","lproc","action","perm_action","data"]:
    if kind in ("cc","r"):
        sig = dict(abi="0.8",kind=kind,params=[],results=[],tier=kind)
        body = f"fn {kind} @alias () () () () ()\n^entry():\nret\n"
    elif kind in ("action","perm_action"):
        sig = dict(abi="0.8",args=[],bounds=[],**{"in":[]},kind=kind,mem=[],mut=[1],tier="q")
        body = f"{kind} @alias (t:wire mut) () () () ()\n^entry():\nx t\n"
    elif kind == "oracle":
        sig = dict(abi="0.8",args=[],fault=4,**{"in":[32]},kind=kind,mem=[],tgt=32,tier="q")
        body = "oracle @alias (x:wire[32] in) (t:wire[32]) (f:wire[4]) () ()\n^entry():\nlift.retval x\n"
    elif kind == "lproc":
        sig = dict(abi="0.8",kind=kind,params=[],result=32,tier="q")
        body = "lproc @alias () (r:reg[32]) () () ()\n^entry():\nr = lift.const.i32 0\nlift.ret r\n"
    else:
        sig = dict(abi="0.8",kind=kind,size=1,tier="-")
        body = "alias 00\n"
    for imported in [False,True]:
        root = base["text"].replace("db","alias")
        root_sig = json.loads(root.splitlines()[3])["0"]["signature"]
        env = base["envelope"].replace("db","alias")
        symbols = {"0":dict(name="o",kind="oracle",signature=root_sig,interface_hash=sha(env))}
        other_env = "isa=0.8\nptr_width=32\n"+j(sig)+"\n"+body
        if kind in ("cc","r","oracle"):
            other_env += "layout A_tab 0\nlayout H 6\nlayout S_cells 0\nlayout heap_base 48\nlayout heap_limit_max 0\nlayout null_size 1\nlayout ptr_width 32\n"
        if kind in ("cc","r"):
            other_env = other_env.replace("layout null_size", 'layout interface_regions.o.alias {"base":16}\nlayout null_size')
        if not imported:
            symbols["1"]=dict(name="alias",kind=kind,signature=sig,interface_hash=sha(other_env))
        obj = root.replace(root.splitlines()[3],j(symbols))
        if imported:
            obj = obj.replace("\nsection functions\n","\nsection imports\n"+j({"alias":dict(signature=sig)})+"\n\nsection functions\n")
        elif kind == "data":
            obj = obj.replace("\nsection layout\n","\nsection data\n"+body+"\nsection layout\n")
        else:
            obj = obj.replace("\nsection layout\n",body+"\nsection layout\n")
        tag = f"family-{kind}-{imported}"
        cases[tag]=dict(text=obj,envelope=env,hash=sha(env),object_hash=sha(obj))
        # Same spelling in a call remains a reference, alongside local addr.
        if kind == "lproc" and not imported:
            for value in [0,1]:
                b = body.replace("i32 0","i32 "+str(value))
                dep = sha("isa=0.8\nptr_width=32\n"+j(sig)+"\n"+b)
                call = "r = lift.call @alias ()\n"
                call_env = env.replace("p:reg[32] }","p:reg[32], r:reg[32] }").replace("lift.retval p",call.replace("@alias","@"+dep)+"lift.retval r")+dep+"\n"
                call_obj = obj.replace(body,b).replace("p:reg[32] }","p:reg[32], r:reg[32] }").replace("lift.retval p",call+"lift.retval r")
                syms = json.loads(call_obj.splitlines()[3])
                syms["0"]["interface_hash"]=sha(call_env)
                syms["1"]["interface_hash"]=dep
                call_obj = call_obj.replace(call_obj.splitlines()[3],j(syms))
                cases[f"call-{value}"]=dict(text=call_obj,envelope=call_env,hash=sha(call_env),object_hash=sha(call_obj),dependency=dep)
# A non-declaring address with an actual global must stay a data reference.
g = cases["family-data-False"]
obj = g["text"].replace("(alias:region", "(db:region").replace('"o":{"alias":', '"o":{"db":')
old_layout = obj.splitlines()[-1]
layout = json.loads(old_layout)
layout["A_tab"]=1
layout["globals"]={"alias":dict(base=24,size=1,image="alias")}
obj = obj.replace(old_layout,j(layout))
dep_env = "isa=0.8\nptr_width=32\n"+j(dict(abi="0.8",kind="data",size=1,tier="-"))+"\nalias 00\n"
dep=sha(dep_env)
env = g["envelope"].replace("(alias:region","(db:region").replace("interface_regions.o.alias", "interface_regions.o.db").replace("lift.addr @alias", "lift.addr @"+dep).replace("layout A_tab 0","data alias 00\nlayout A_tab 1").replace("layout heap_base",'layout globals.alias {"base":24,"image":"alias","size":1}\nlayout heap_base')+dep+"\n"
syms = json.loads(obj.splitlines()[3])
syms["0"]["interface_hash"]=sha(env)
obj = obj.replace(obj.splitlines()[3],j(syms))
cases["global"]=dict(text=obj,envelope=env,hash=sha(env),object_hash=sha(obj),dependency=dep)

# The same @alias image in an action remains an ordinary data dependency.
g = cases["family-data-False"]
sig = dict(abi="0.8",args=[],bounds=[],**{"in":[]},kind="action",mem=[],mut=[1],tier="q")
body = "action @a (t:wire mut) () () () ()\n^entry():\nqregion cells 0 8 @alias\nx t\n"
env = "isa=0.8\nptr_width=32\n"+j(sig)+"\n"+body.replace("@alias", "@"+dep)+dep+"\n"
obj = g["text"].replace("\nsection data\n",body+"\nsection data\n")
syms = json.loads(obj.splitlines()[3])
syms["2"]=dict(name="a",kind="action",signature=sig,interface_hash=sha(env))
obj = obj.replace(obj.splitlines()[3],j(syms))
cases["image"]=dict(text=obj,envelope=env,hash=sha(env),object_hash=sha(obj),dependency=dep)
print("address corpus",len(cases),"cases")
(ROOT/"support/r12_address_cases.json").write_text(j(cases)+"\n")

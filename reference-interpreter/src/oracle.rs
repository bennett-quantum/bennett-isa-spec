//! §14.1 sequential interpreter. It uses independent dynamic activations rather
//! than the reversible machine's global location file and frame-slot exchanges.
use crate::{err,object::{integer,is_binop,is_div,list,mask,parse_bits},Error,Function,Instruction,Object,Result,Type};
use std::collections::{BTreeMap,BTreeSet};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Budget {pub steps:u64,pub heap:u128,pub stack:u128,pub depth:u128}
impl Budget {pub fn steps(steps:u64)->Self{Self{steps,heap:0,stack:0,depth:0}}}
#[derive(Clone,Debug,PartialEq)]
pub enum Argument{Bits(u128),Float(f64),Token(Budget)}
#[derive(Clone,Debug,Default,PartialEq)]
pub struct OracleInput{pub ins:Vec<u128>,pub args:Vec<Argument>,pub regions:BTreeMap<String,Vec<u8>>}
impl OracleInput{pub fn bits(ins:&[u128])->Self{Self{ins:ins.into(),..Self::default()}}}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct OracleOutcome{pub payload:u128,pub code:u128,pub steps:u64}

#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct Allocation{pub base:u128,pub size:u128,pub live:bool,pub pinned:bool}
#[derive(Clone,Debug,PartialEq,Eq,Default)]
pub(crate) struct Memory{pub bytes:BTreeMap<u128,u8>,pub allocations:Vec<Allocation>,pub heap_base:u128,pub heap_used:u128,pub table_capacity:u128,pub addresses:BTreeMap<String,u128>}
impl Memory{
    pub fn initial(object:&Object,oracle:&Function,input:&OracleInput)->Result<Self>{
        let Some(layout)=object.section("layout")else{return Ok(Self::default());};
        let mut m=Self{heap_base:layout.get("heap_base")?.uint()?,table_capacity:1u128<<(layout.get("A_tab")?.uint()? as u32),..Self::default()};
        for(n,g)in layout.get("globals")?.object()?{
            let base=g.get("base")?.uint()?;let size=g.get("size")?.uint()?;
            m.addresses.insert(n.clone(),base);m.allocations.push(Allocation{base,size,live:true,pinned:true});
            for(i,b)in object.data[n].iter().enumerate(){m.set_byte(base+i as u128,*b);}
        }
        if !oracle.params[3].is_empty(){let regions=layout.get("interface_regions")?.get(&oracle.name)?;
            // Allocation pre-entries follow layout order (§3.7), not signature order.
            for(n,r)in regions.object()?{
                let p=oracle.params[3].iter().find(|p|p.name==*n).expect("validated region mapping");let Type::Region(a,_) = p.ty else{unreachable!("validated region type")};
                let base=r.get("base")?.uint()?;let size=1u128<<a;
                let bytes=input.regions.get(n).ok_or_else(||Error(format!("§5.1: missing interface region {n}")))?;
                if bytes.len() as u128!=size{return err("§5.1",format!("interface region {n} has wrong byte count"));}
                m.addresses.insert(n.clone(),base);m.allocations.push(Allocation{base,size,live:true,pinned:true});
                for(i,b)in bytes.iter().enumerate(){m.set_byte(base+i as u128,*b);}
            }
        }Ok(m)
    }
    pub fn valid(&self,p:u128,n:u128)->bool{p.checked_add(n).is_some_and(|end|self.allocations.iter().any(|e|e.live&&e.base<=p&&e.base.checked_add(e.size).is_some_and(|limit|end<=limit)))}
    pub fn byte(&self,p:u128)->u8{self.bytes.get(&p).copied().unwrap_or(0)}
    pub fn set_byte(&mut self,p:u128,b:u8){if b==0{self.bytes.remove(&p);}else{self.bytes.insert(p,b);}}
    pub fn load(&self,p:u128,width:u16)->u128{(0..width/8).map(|i|(self.byte(p+i as u128)as u128)<<(8*i)).fold(0,|a,b|a|b)}
    pub fn store(&mut self,p:u128,width:u16,v:u128){for i in 0..width/8{self.set_byte(p+i as u128,(v>>(8*i)) as u8);}}
    pub fn allocation_fails(&self,n:u128,budget:Budget)->bool{self.heap_used.checked_add(n.max(1)).is_none_or(|total|total>budget.heap)||self.allocations.len()as u128>=self.table_capacity}
    pub fn allocate(&mut self,n:u128)->u128{let base=self.heap_base+self.heap_used;self.allocations.push(Allocation{base,size:n,live:true,pinned:false});self.heap_used+=n.max(1);base}
    pub fn free(&mut self,p:u128)->bool{if let Some(e)=self.allocations.iter_mut().find(|e|e.base==p&&e.live&&!e.pinned){e.live=false;true}else{false}}
}

pub(crate) struct Program<'a>{pub rows:Vec<(&'a Function,&'a Instruction)>,pub labels:BTreeMap<(String,String),usize>,pub ranges:BTreeMap<String,(usize,usize)>,pub halt:usize}
impl<'a> Program<'a>{
    pub fn new(object:&'a Object,root:&str)->Result<Self>{
        let oracle=object.functions.get(root).filter(|f|f.kind=="oracle").ok_or_else(||Error(format!("§5.2: unknown oracle @{root}")))?;
        let mut reachable=BTreeSet::new();let mut pending=vec![root.to_string()];
        while let Some(n)=pending.pop(){for i in object.functions[&n].instructions().filter(|i|i.op=="lift.call"){
            let n=i.operands[0].trim_start_matches('@').to_string();if reachable.insert(n.clone()){pending.push(n);}
        }}
        let mut closure:Vec<_>=reachable.into_iter().collect();
        if !closure.is_empty(){let identity=object.identity()?;closure.sort_by(|a,b|identity.closure_hashes[a].cmp(&identity.closure_hashes[b]).then(a.cmp(b)));}
        let order=std::iter::once(&oracle.name).chain(closure.iter());let mut rows=Vec::new();let mut labels=BTreeMap::new();let mut ranges=BTreeMap::new();
        for n in order{let f=&object.functions[n];let start=rows.len();for b in &f.blocks{labels.insert((n.clone(),b.name.clone()),rows.len());for i in &b.instructions{rows.push((f,i));}}ranges.insert(n.clone(),(start,rows.len()));}
        let halt=rows.len();Ok(Self{rows,labels,ranges,halt})
    }
    pub fn label(&self,f:&str,label:&str)->Result<usize>{self.labels.get(&(f.into(),label.trim_start_matches('^').into())).copied().ok_or_else(||Error(format!("§3.6: unknown label {label} in @{f}")))}
}

#[derive(Clone)]
struct Activation{function:String,values:BTreeMap<String,u128>,ret_pc:usize,destination:String}
impl Activation{
    fn new(f:&Function)->Self{Self{function:f.name.clone(),values:f.locations().iter().map(|p|(p.name.clone(),0)).collect(),ret_pc:0,destination:String::new()}}
    fn read(&self,n:&str)->Result<u128>{self.values.get(n).copied().ok_or_else(||Error(format!("§3.7: unreadable operand {n} in @{}",self.function)))}
    fn write(&mut self,f:&Function,n:&str,v:u128,ptr:u16)->Result<()>{let p=f.locations().into_iter().find(|p|p.name==n).ok_or_else(||Error(format!("§3.7: unwritable location {n}")))?;self.values.insert(n.into(),v&mask(p.ty.width(ptr)?));Ok(())}
}

impl Object{
    pub fn validate_budget(&self,b:Budget)->Result<()>{
        if let Some(l)=self.section("layout"){
            if b.heap>l.get("heap_limit_max")?.uint()?||b.stack>1u128<<(l.get("S_cells")?.uint()? as u32)||b.depth==u128::MAX{return err("§5.4","budget limit does not fit layout/counter");}
        }else if b.heap!=0||b.stack!=0||b.depth!=0{return err("§5.4","memoryless budgets require zero heap, stack and depth");}Ok(())
    }
    pub fn default_budget(&self,steps:u64)->Result<Budget>{if let Some(l)=self.section("layout"){Ok(Budget{steps,heap:l.get("heap_limit_max")?.uint()?,stack:1u128<<(l.get("S_cells")?.uint()? as u32),depth:64})}else{Ok(Budget::steps(steps))}}
    pub fn run_oracle(&self,name:&str,input:&OracleInput,budget:Budget)->Result<OracleOutcome>{
        self.validate_budget(budget)?;
        let program=Program::new(self,name)?;let root=&self.functions[name];
        if input.ins.len()!=root.params[0].len()||input.args.len()!=root.params[4].len()||input.regions.len()!=root.params[3].len(){return err("§5.1","oracle interface argument count mismatch");}
        let mut activation=Activation::new(root);let mut arguments=BTreeMap::new();
        for(p,v)in root.params[0].iter().zip(&input.ins){if *v&!mask(p.ty.width(self.ptr_width)?)!=0{return err("§3.3","input value exceeds declared width");}activation.values.insert(p.name.clone(),*v);}
        for(p,v)in root.params[4].iter().zip(&input.args){match(&p.ty,v){
            (Type::Int(w),Argument::Bits(v)) if *v&!mask(*w)==0=>{activation.values.insert(p.name.clone(),*v);},
            (Type::Ptr,Argument::Bits(v))if *v&!mask(self.ptr_width)==0=>{activation.values.insert(p.name.clone(),*v);},
            (Type::F64,Argument::Float(v))if v.is_finite()=>{},(Type::Token,Argument::Token(b))=>{self.validate_budget(*b)?;},
            _=>return err("§3.3","classical argument type/width mismatch"),
        }arguments.insert(p.name.clone(),v.clone());}
        let mut memory=Memory::initial(self,root,input)?;let mut stack:Vec<Activation>=Vec::new();let mut sp=0u128;
        let mut pc=0;let mut steps=0;let mut result=0;
        macro_rules! fault{($code:expr)=>{return Ok(OracleOutcome{payload:0,code:$code,steps})};}
        while steps<budget.steps{
            if pc==program.halt{return Ok(OracleOutcome{payload:result,code:0,steps});}
            let(f,i)=program.rows[pc];
            if activation.function!=f.name{return err("§3.7","control escaped its active function");}
            steps+=1;let mut next=pc+1;let op=i.op.as_str();let a=&i.operands;
            let rd=|n:&str|activation.read(n);
            let width=|n:&str|->Result<u16>{f.locations().into_iter().chain(if f.kind=="oracle"{f.params[0].iter().chain(&f.params[4]).collect::<Vec<_>>()}else{Vec::new()}).find(|p|p.name==n).ok_or_else(||Error(format!("§3.7: unknown width for {n}")))?.ty.width(self.ptr_width)};
            let mut output=None;
            if let Some(n)=op.strip_prefix("lift.const.i"){output=Some(parse_bits(&a[0],n.parse().expect("validated constant width"))?);}
            else if let Some(bin)=op.strip_prefix("lift.").filter(|s|is_binop(s)){output=Some(binop(bin,rd(&a[0])?,rd(&a[1])?,width(&a[0])?));}
            else if let Some(div)=op.strip_prefix("lift.").filter(|s|is_div(s)){match divide(div,rd(&a[0])?,rd(&a[1])?,width(&a[0])?){Ok(v)=>output=Some(v),Err(c)=>fault!(c)}}
            else if let Some(pred)=op.strip_prefix("lift.icmp."){output=Some(compare(pred,rd(&a[0])?,rd(&a[1])?,width(&a[0])?) as u128);}
            else if let Some((cast,_))=op.strip_prefix("lift.").and_then(|s|s.split_once(".i")).filter(|(c,_)|["zext","sext","trunc"].contains(c)){let v=rd(&a[0])?;output=Some(if cast=="sext"{signed(v,width(&a[0])?) as u128}else{v});}
            else if let Some(n)=op.strip_prefix("lift.load.i"){
                let w=n.parse::<u16>().expect("validated load width");let p=rd(&a[0])?;if !memory.valid(p,(w/8)as u128){fault!(7);}output=Some(memory.load(p,w));
            }else if let Some(n)=op.strip_prefix("lift.store.i"){
                let w=n.parse::<u16>().expect("validated store width");let p=rd(&a[0])?;let v=rd(&a[1])?;if !memory.valid(p,(w/8)as u128){fault!(7);}memory.store(p,w,v);
            }else{match op{
                "lift.mov"=>output=Some(rd(&a[0])?),
                "lift.select"=>output=Some(rd(if rd(&a[0])?!=0{&a[1]}else{&a[2]})?),
                "lift.parmov"=>{let values=list(&a[0])?.iter().map(|s|rd(s)).collect::<Result<Vec<_>>>()?;for(d,v)in i.lhs.iter().zip(values){activation.write(f,d,v,self.ptr_width)?;}}
                "lift.label"=>output=Some(program.label(&f.name,&a[0])? as u128),
                "lift.addr"=>output=Some(*memory.addresses.get(a[0].trim_start_matches('@')).ok_or_else(||Error("§4.2: address not mapped".into()))?),
                "lift.boundscheck"=>{let p=rd(&a[0])?;let n=if integer(&a[1]){a[1].parse::<u128>().expect("validated length")}else{rd(&a[1])?};if !memory.valid(p,n){fault!(7);}}
                "lift.alloc"=>{let n=if integer(&a[0]){a[0].parse::<u128>().expect("validated length")}else{rd(&a[0])?};if memory.allocation_fails(n,budget){fault!(2);}output=Some(memory.allocate(n));}
                "lift.free"=>{let p=rd(&a[0])?;if !memory.free(p){fault!(7);}}
                "lift.faultif"=>{if rd(&a[0])?!=0{fault!(a[1].parse::<u128>().expect("validated fault code"));}}
                "fault"=>fault!(a[0].parse::<u128>().expect("validated fault code")),
                "qjump"=>next=program.label(&f.name,&a[0])?,
                "qbranch"=>next=program.label(&f.name,if rd(&a[0])?!=0{&a[1]}else{&a[2]})?,
                "qjumpr"=>{let address=rd(&a[0])?;let(start,end)=program.ranges[&f.name];if address<(start as u128)||address>=end as u128{fault!(4);}next=address as usize;}
                "lift.call"=>{
                    let callee=&self.functions[a[0].trim_start_matches('@')];let cells=callee.frame_cells()as u128;
                    if stack.len()as u128+1>budget.depth||sp.checked_add(cells).is_none_or(|p|p>budget.stack){fault!(3);}
                    let values=list(&a[1])?.iter().map(|s|rd(s)).collect::<Result<Vec<_>>>()?;
                    let mut called=Activation::new(callee);called.ret_pc=pc+1;called.destination=i.lhs[0].clone();for(p,v)in callee.params[0].iter().zip(values){called.values.insert(p.name.clone(),v);}
                    sp+=cells;stack.push(activation);activation=called;next=program.ranges[&callee.name].0;
                }
                "lift.ret"=>{
                    if stack.is_empty(){fault!(4);}let v=rd(&a[0])?;let ret=activation.ret_pc;let dest=activation.destination.clone();activation=stack.pop().expect("nonempty activation stack");let caller=&self.functions[&activation.function];activation.write(caller,&dest,v,self.ptr_width)?;next=ret;
                }
                "lift.retval"=>{result^=rd(&a[0])?;return Ok(OracleOutcome{payload:result,code:0,steps});}
                "lift.oracle_call"|"lift.oracle_uncall"=>{
                    let callee=&self.functions[a[0].trim_start_matches('@')];
                    let token=arguments.get(&a[1]).and_then(|a|if let Argument::Token(b)=a{Some(*b)}else{None}).ok_or_else(||Error("§4.2: nested oracle call needs a budget token".into()))?;
                    let mut child=OracleInput::bits(&list(&a[2])?.iter().map(|s|rd(s)).collect::<Result<Vec<_>>>()?);
                    let region_args=list(&a[5])?;for(p,n)in callee.params[3].iter().zip(region_args){let base=memory.addresses[&n];let Type::Region(aw,_)=p.ty else{unreachable!("validated region")};let len=usize::try_from(1u128<<aw).map_err(|_|Error("§7.2: region exceeds host addressable storage".into()))?;child.regions.insert(p.name.clone(),(0..len).map(|i|memory.byte(base+i as u128)).collect());}
                    for arg in list(&a[6])?{child.args.push(match arguments.get(&arg){Some(v)=>v.clone(),None=>Argument::Bits(rd(&arg)?)});}
                    let r=self.run_oracle(&callee.name,&child,token)?;let tgt=rd(&a[3])?^r.payload;let code=rd(&a[4])?^r.code;
                    activation.write(f,&a[3],tgt,self.ptr_width)?;activation.write(f,&a[4],code,self.ptr_width)?;
                }
                "lift.perm_call"=>return err("§4.2","permutation-action execution awaits the Q engine"),
                _=>return err("§4.8",format!("unimplemented sequential row {op}")),
            }}
            if let Some(v)=output{activation.write(f,&i.lhs[0],v,self.ptr_width)?;}
            pc=next;
        }
        Ok(OracleOutcome{payload:0,code:1,steps})
    }
}
pub(crate) fn signed(v:u128,width:u16)->i128{let shift=128-width;(v<<shift)as i128>>shift}
pub(crate) fn binop(op:&str,a:u128,b:u128,w:u16)->u128{let v=match op{
    "add"=>a.wrapping_add(b),"sub"=>a.wrapping_sub(b),"mul"=>a.wrapping_mul(b),"and"=>a&b,"or"=>a|b,"xor"=>a^b,
    "shl"=>if b>=w as u128{0}else{a<<(b as u32)},"lshr"=>if b>=w as u128{0}else{a>>(b as u32)},"ashr"=>if b>=w as u128{0}else{(signed(a,w)>>(b as u32))as u128},_=>unreachable!("validated binop"),
};v&mask(w)}
pub(crate) fn divide(op:&str,a:u128,b:u128,w:u16)->std::result::Result<u128,u128>{
    if b==0{return Err(5);}if op.starts_with('s')&&a==1u128<<(w-1)&&b==mask(w){return Err(6);}
    Ok(match op{"udiv"=>a/b,"urem"=>a%b,"sdiv"=>(signed(a,w)/signed(b,w))as u128,"srem"=>(signed(a,w)%signed(b,w))as u128,_=>unreachable!("validated division")} & mask(w))
}
pub(crate) fn compare(pred:&str,a:u128,b:u128,w:u16)->bool{match pred{"eq"=>a==b,"ne"=>a!=b,"ult"=>a<b,"ule"=>a<=b,"slt"=>signed(a,w)<signed(b,w),"sle"=>signed(a,w)<=signed(b,w),_=>unreachable!("validated comparison")}}

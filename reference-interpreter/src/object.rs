use crate::{err, json::Json, Error, Result};
use std::collections::{BTreeMap, BTreeSet};

pub const SECTIONS: [&str; 9] = ["symbols","imports","functions","data","devices","layout","schedule","certificate","coverage"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type { Int(u16), Ptr, F64, Token, Qreg(u64), Wire(u16), Reg(u16), Region(u16,u16) }
impl Type {
    pub fn parse(s: &str) -> Result<Self> {
        let width=|s: &str| -> Result<u16> { let n=s.parse::<u16>().map_err(|_| Error(format!("§3.3: invalid width {s}")))?; if !(1..=128).contains(&n) { return err("§3.3", "width outside 1..128"); } Ok(n) };
        Ok(match s {
            "ptr"=>Self::Ptr, "f64"=>Self::F64,"token"=>Self::Token,"wire"=>Self::Wire(1),
            _ if s.starts_with("i")=>Self::Int(width(&s[1..])?),
            _ if s.starts_with("wire[") && s.ends_with(']')=>Self::Wire(width(&s[5..s.len()-1])?),
            _ if s.starts_with("reg[") && s.ends_with(']')=>Self::Reg(width(&s[4..s.len()-1])?),
            _ if s.starts_with("qreg[") && s.ends_with(']')=>Self::Qreg(s[5..s.len()-1].parse().map_err(|_|Error("§3.3: invalid qreg size".into()))?),
            _ if s.starts_with("region[") && s.ends_with(']')=> {
                let (a,w)=s[7..s.len()-1].split_once(',').ok_or_else(||Error("§3.3: malformed region type".into()))?;
                let a=a.parse().map_err(|_|Error("§3.3: invalid address width".into()))?;
                if a>127 { return err("§3.3","region address width exceeds 127"); }
                Self::Region(a,width(w)?)
            }
            _=>return err("§3.3",format!("unknown type {s}")),
        })
    }
    pub fn width(&self, ptr: u16) -> Result<u16> {
        match self { Self::Int(n)|Self::Wire(n)|Self::Reg(n)=>Ok(*n),Self::Ptr=>Ok(ptr),_=>err("§3.3","expected bit-vector type") }
    }
    pub fn text(&self) -> String { match self {
        Self::Int(n)=>format!("i{n}"),Self::Wire(n)=>format!("wire[{n}]"),Self::Reg(n)=>format!("reg[{n}]"),Self::Region(a,w)=>format!("region[{a},{w}]"),Self::Qreg(n)=>format!("qreg[{n}]"),Self::Ptr=>"ptr".into(),Self::F64=>"f64".into(),Self::Token=>"token".into(),
    } }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Param { pub name: String, pub ty: Type, pub role: Option<String> }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instruction {
    pub lhs: Vec<String>, pub op: String, pub operands: Vec<String>,
    pub bodies: Vec<Vec<Instruction>>, pub line: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block { pub name: String, pub params: Vec<Param>, pub instructions: Vec<Instruction> }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
    pub kind: String, pub name: String, pub params: [Vec<Param>;5],
    pub regs: Vec<Param>, pub bounds: Vec<u64>, pub blocks: Vec<Block>, pub lines: Vec<String>,
}
impl Function {
    pub fn locations(&self) -> Vec<&Param> {
        let mut out=Vec::new(); if self.kind=="lproc" { out.extend(&self.params[0]); out.extend(&self.params[1]); } out.extend(&self.regs); out
    }
    pub fn frame_cells(&self) -> usize { 3+self.locations().len() }
    pub fn instructions(&self) -> impl Iterator<Item=&Instruction> { self.blocks.iter().flat_map(|b|&b.instructions) }
    pub fn signature(&self) -> Json {
        let widths=|p: &[Param]| Json::Array(p.iter().map(|p|Json::Number(match &p.ty { Type::Wire(n)|Type::Reg(n)=>n.to_string(),_=>"0".into() })).collect());
        let types=|p: &[Param]| Json::Array(p.iter().map(|p|Json::String(p.ty.text())).collect());
        let mem=|p: &[Param]| Json::Array(p.iter().map(|p| {
            let (a,w)=match p.ty { Type::Region(a,w)=>(a,w),_=>(0,0) };
            Json::Object([("A".into(),Json::Number(a.to_string())),("W".into(),Json::Number(w.to_string())),("role".into(),Json::String(p.role.clone().unwrap_or_default()))].into())
        }).collect());
        let mut m=BTreeMap::from([("abi".into(),Json::String("0.8".into())),("kind".into(),Json::String(self.kind.clone())),("tier".into(),Json::String(if matches!(self.kind.as_str(),"cc"|"r") { self.kind.clone() } else { "q".into() }))]);
        match self.kind.as_str() {
            "oracle"=> { m.insert("in".into(),widths(&self.params[0])); m.insert("tgt".into(),widths(&self.params[1]).array().expect("array")[0].clone()); m.insert("fault".into(),widths(&self.params[2]).array().expect("array")[0].clone()); m.insert("mem".into(),mem(&self.params[3])); m.insert("args".into(),types(&self.params[4])); }
            "lproc"=> { m.insert("params".into(),widths(&self.params[0])); m.insert("result".into(),widths(&self.params[1]).array().expect("array")[0].clone()); }
            "action"|"perm_action"=> { m.insert("mut".into(),widths(&self.params[0])); m.insert("in".into(),widths(&self.params[1])); m.insert("mem".into(),mem(&self.params[2])); m.insert("args".into(),types(&self.params[3])); m.insert("bounds".into(),Json::Array(self.bounds.iter().map(|n|Json::Number(n.to_string())).collect())); }
            _=> { m.insert("params".into(),types(&self.params[0])); m.insert("results".into(),types(&self.params[1])); }
        }
        Json::Object(m)
    }
}

#[derive(Clone, Debug)]
pub struct Object {
    pub text: String, pub ptr_width: u16, pub metadata: BTreeMap<String,Json>,
    pub functions: BTreeMap<String,Function>, pub function_order: Vec<String>, pub data: BTreeMap<String,Vec<u8>>,
}

pub(crate) fn name(s: &str) -> bool {
    let mut b=s.bytes(); matches!(b.next(),Some(b'A'..=b'Z'|b'a'..=b'z'|b'_')) && b.all(|b| b.is_ascii_alphanumeric()||b==b'_'||b==b'.')
}
pub(crate) fn integer(s: &str) -> bool {
    let s=s.strip_prefix('-').unwrap_or(s); s=="0" || (!s.starts_with('0') && !s.is_empty() && s.bytes().all(|b|b.is_ascii_digit()))
}
pub(crate) fn list(s: &str) -> Result<Vec<String>> {
    let s=s.strip_prefix('(').and_then(|s|s.strip_suffix(')')).ok_or_else(||Error(format!("§16.2: expected operand list {s}")))?;
    if s.is_empty() { return Ok(Vec::new()); }
    let v:Vec<_>=s.split(", ").map(str::to_string).collect();
    if v.iter().any(|s|!name(s)) { return err("§16.2","operand lists require names separated by comma and space"); } Ok(v)
}
fn params(s: &str) -> Result<Vec<Param>> {
    if s.is_empty() { return Ok(Vec::new()); }
    let mut seen=BTreeSet::new(); let mut out=Vec::new();
    for p in s.split(", ") {
        let (n,rest)=p.split_once(':').ok_or_else(||Error(format!("§16.2: invalid parameter {p}")))?;
        if !name(n) || !seen.insert(n) { return err("§16.2","invalid or duplicate parameter name"); }
        let (t,role)=match rest.split_once(' ') { Some((t,r)) if matches!(r,"in"|"mut")=>(t,Some(r.to_string())),Some(_)=>return err("§16.2","invalid parameter role"),None=>(rest,None) };
        out.push(Param { name:n.into(),ty:Type::parse(t)?,role });
    } Ok(out)
}

/// Split instruction operands without splitting tuple or string operands.
pub(crate) fn tokens(s: &str) -> Result<Vec<String>> {
    let mut out=Vec::new(); let mut start=0; let mut nesting=0; let mut quoted=false; let mut escaped=false;
    for (i,c) in s.char_indices() {
        if quoted { if escaped { escaped=false; } else if c=='\\' { escaped=true; } else if c=='"' { quoted=false; } continue; }
        match c { '"'=>quoted=true,'('=>nesting+=1,')'=> { nesting-=1; if nesting<0 { return err("§16.2","unmatched close parenthesis"); } }, ' ' if nesting==0=> { if start==i { return err("§16.1","multiple spaces between tokens"); } out.push(s[start..i].into()); start=i+1; },_=>{} }
    }
    if quoted || nesting!=0 { return err("§16.2","unterminated operand"); }
    if start<s.len() { out.push(s[start..].into()); } else if !s.is_empty() { return err("§16.1","trailing space"); }
    Ok(out)
}
fn instruction(lines: &[(usize,&str)], pos: &mut usize) -> Result<Instruction> {
    let (line,raw)=lines[*pos]; *pos+=1;
    let (lhs,rhs)=if let Some((a,b))=raw.split_once(" = ") {
        let lhs=if a.starts_with('(') { list(a)? } else if name(a) { vec![a.into()] } else { return err("§16.2","invalid result list"); }; (lhs,b)
    } else { (Vec::new(),raw) };
    let ts=tokens(rhs)?; if ts.is_empty() { return err("§16.2","empty instruction"); }
    let mut ins=Instruction { lhs,op:ts[0].clone(),operands:ts[1..].to_vec(),bodies:Vec::new(),line };
    if ins.operands.last().is_some_and(|s|s=="{") {
        ins.operands.pop();
        loop {
            let mut body=Vec::new();
            while *pos<lines.len() && !lines[*pos].1.starts_with('}') { body.push(instruction(lines,pos)?); }
            if body.is_empty() || *pos==lines.len() { return err("§16.2","empty or unterminated structured body"); }
            let close=lines[*pos].1; *pos+=1; ins.bodies.push(body);
            match close { "}"=>break,"} {" if ins.bodies.len()==1=>{},_=>return err("§16.2","invalid structured body close") }
        }
    }
    Ok(ins)
}

impl Object {
    pub fn parse(text: &str) -> Result<Self> {
        if !text.ends_with('\n') || text.contains('\r') || text.contains('\t') || text.lines().any(|s|s.ends_with(' ')) { return err("§16.1","object requires LF lines, no tabs or trailing spaces, and final LF"); }
        let lines:Vec<_>=text.lines().collect();
        let header=tokens(lines.first().copied().unwrap_or(""))?;
        if header.len()!=9 || header[0]!="bennett-object" { return err("§16.2","malformed object header"); }
        for (i,k) in ["isa","object","launch","sched","cert","coverage","target"].iter().enumerate() {
            if header[i+1]!=format!("{k}=0.8") { return err("§1",format!("expected {k}=0.8")); }
        }
        let ptr_width=header[8].strip_prefix("ptr_width=").and_then(|s|s.parse::<u16>().ok()).ok_or_else(||Error("§16.2: invalid ptr_width".into()))?;
        if !(1..=128).contains(&ptr_width) { return err("§3.3","pointer width outside 1..128"); }
        let mut obj=Object { text:text.into(),ptr_width,metadata:BTreeMap::new(),functions:BTreeMap::new(),function_order:Vec::new(),data:BTreeMap::new() };
        let mut pos=1; let mut previous=None;
        while pos<lines.len() {
            if lines[pos]!="" || pos+1>=lines.len() || !lines[pos+1].starts_with("section ") { return err("§16.1",format!("exactly one blank required before section at line {}",pos+1)); }
            pos+=1; let section=&lines[pos][8..]; pos+=1;
            let order=SECTIONS.iter().position(|s|*s==section).ok_or_else(||Error(format!("§9.1: unknown section {section}")))?;
            if previous.is_some_and(|n|n>=order) { return err("§9.1","duplicate or out-of-order section"); } previous=Some(order);
            let start=pos;
            while pos<lines.len() && !lines[pos].is_empty() {
                if lines[pos].starts_with("section ") { return err("§16.1","missing blank line before section"); } pos+=1;
            }
            if pos<lines.len() && (pos+1==lines.len() || !lines[pos+1].starts_with("section ")) { return err("§16.1","blank line outside section boundary"); }
            match section {
                "functions"=>obj.parse_functions(&lines[start..pos],start+1)?,
                "data"=>for line in &lines[start..pos] {
                    let t=tokens(line)?;
                    // Empty data images have an empty HEX token, represented by NAME SP.
                    if t.len()!=2 || !name(&t[0]) || t[1].len()%2!=0 || !t[1].bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)) { return err("§16.2","invalid data line"); }
                    let bytes=(0..t[1].len()).step_by(2).map(|i|u8::from_str_radix(&t[1][i..i+2],16).expect("hex checked")).collect();
                    if obj.data.insert(t[0].clone(),bytes).is_some() { return err("§9.1","duplicate data name"); }
                },
                _=> {
                    if pos-start!=1 { return err("§9.1",format!("section {section} requires one JSON object line")); }
                    let v=Json::parse(lines[start])?; v.object()?;
                    if v.canonical()!=lines[start] { return err("§16.1",format!("section {section} is not RFC 8785 canonical")); }
                    obj.metadata.insert(section.into(),v);
                }
            }
        }
        obj.validate()?; Ok(obj)
    }
    fn parse_functions(&mut self, lines: &[&str], base: usize) -> Result<()> {
        let mut pos=0;
        while pos<lines.len() {
            let start=pos; let ts=tokens(lines[pos])?; pos+=1;
            let (kind,offset)=if ts.first().is_some_and(|s|s=="fn") { (ts.get(1).map(String::as_str).unwrap_or(""),2) } else { (ts.first().map(String::as_str).unwrap_or(""),1) };
            if !matches!(kind,"cc"|"r"|"oracle"|"action"|"perm_action"|"lproc") || ts.len()!=offset+6 { return err("§16.2","invalid function header or number of parameter lists"); }
            let n=ts[offset].strip_prefix('@').filter(|s|name(s)).ok_or_else(||Error("§16.2: invalid function name".into()))?;
            let mut ps:[Vec<Param>;5]=Default::default();
            for (i,s) in ts[offset+1..].iter().enumerate() { let inner=s.strip_prefix('(').and_then(|s|s.strip_suffix(')')).ok_or_else(||Error("§16.2: invalid parameter list".into()))?; ps[i]=params(inner)?; }
            let mut f=Function { kind:kind.into(),name:n.into(),params:ps,regs:Vec::new(),bounds:Vec::new(),blocks:Vec::new(),lines:Vec::new() };
            if pos<lines.len() && lines[pos].starts_with("regs ") {
                let body=lines[pos].strip_prefix("regs { ").ok_or_else(||Error("§16.2: malformed regs".into()))?;
                let (rs,rest)=body.split_once(" }").ok_or_else(||Error("§16.2: malformed regs close".into()))?;
                f.regs=params(rs)?;
                if !rest.is_empty() {
                    let bs=rest.strip_prefix(" bounds {").and_then(|s|s.strip_suffix('}')).ok_or_else(||Error("§16.2: malformed bounds".into()))?;
                    if !bs.is_empty() { f.bounds=bs.split(", ").map(|s|s.parse::<u64>().map_err(|_|Error("§16.2: invalid loop bound".into()))).collect::<Result<_>>()?; }
                }
                pos+=1;
            }
            while pos<lines.len() && lines[pos].starts_with('^') {
                let b=lines[pos]; pos+=1; let (bn,ps)=b[1..].split_once('(').ok_or_else(||Error("§16.2: malformed block".into()))?;
                if !name(bn) { return err("§16.2","invalid block name"); }
                let ps=ps.strip_suffix("):").ok_or_else(||Error("§16.2: malformed block parameter list".into()))?;
                let mut block=Block { name:bn.into(),params:params(ps)?,instructions:Vec::new() };
                let end=(pos..lines.len()).find(|i|lines[*i].starts_with('^')||is_head(lines[*i])).unwrap_or(lines.len());
                let numbered:Vec<_>=lines[pos..end].iter().enumerate().map(|(i,s)|(base+pos+i,*s)).collect(); let mut ip=0;
                while ip<numbered.len() { block.instructions.push(instruction(&numbered,&mut ip)?); }
                if block.instructions.is_empty() { return err("§16.2","empty block"); }
                f.blocks.push(block); pos=end;
            }
            if f.blocks.is_empty() { return err("§16.2","function requires blocks"); }
            f.lines=lines[start..pos].iter().map(|s|s.to_string()).collect();
            self.function_order.push(n.into());
            if self.functions.insert(n.into(),f).is_some() { return err("§9.1","duplicate function name"); }
        }
        Ok(())
    }
    pub fn section(&self, key: &str) -> Option<&Json> { self.metadata.get(key) }
    pub fn symbols(&self) -> Result<&BTreeMap<String,Json>> { self.metadata.get("symbols").ok_or_else(||Error("§9.1: missing symbols section".into()))?.object() }
    fn validate(&self) -> Result<()> {
        self.validate_symbols()?;
        for f in self.functions.values() { self.validate_function(f)?; }
        self.validate_layout()?;
        self.validate_metadata()?;
        self.validate_call_graph()?;
        Ok(())
    }
    fn validate_symbols(&self) -> Result<()> {
        let mut names=BTreeSet::new();
        for (index,s) in self.symbols()? {
            if !integer(index) || index.starts_with('-') { return err("§9.1","invalid local symbol index"); }
            s.fields(&["name","kind","signature","interface_hash"],&[])?;
            let n=s.get("name")?.string()?; let k=s.get("kind")?.string()?;
            if !name(n) || !names.insert(n) { return err("§9.1","invalid or duplicate symbol name"); }
            let h=s.get("interface_hash")?.string()?;
            if h.len()!=64 || !h.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)) { return err("§9.2","interface_hash must be 64 lowercase hex digits"); }
            let sig=s.get("signature")?;
            if k=="data" {
                sig.fields(&["abi","kind","size","tier"],&[])?;
                if sig.get("kind")?.string()?!="data" || sig.get("tier")?.string()?!="-" || sig.get("abi")?.string()?!="0.8" { return err("§9.1","invalid data signature"); }
                let bytes=self.data.get(n).ok_or_else(||Error(format!("§9.1: missing data definition {n}")))?;
                if sig.get("size")?.uint()?!=bytes.len() as u128 { return err("§7.2","data size differs from signature"); }
            } else {
                let f=self.functions.get(n).ok_or_else(||Error(format!("§9.1: missing function definition {n}")))?;
                validate_header(f)?;
                if k!=f.kind || sig!=&f.signature() { return err("§9.1",format!("signature mismatch for {n}")); }
            }
        }
        if self.functions.keys().chain(self.data.keys()).any(|n|!names.contains(n.as_str())) { return err("§9.1","definition without symbol"); }
        if let Some(imports)=self.section("imports") { for (n,im) in imports.object()? {
            if !name(n)||names.contains(n.as_str()) { return err("§9.3","invalid import name or import conflicts with definition"); }
            im.fields(&["signature"],&["pinned_closure_hash"])?;
            if let Some(p)=im.object()?.get("pinned_closure_hash") { let h=p.string()?; if h.len()!=64||!h.bytes().all(|b|b.is_ascii_hexdigit()&&!b.is_ascii_uppercase()) { return err("§9.3","invalid pinned hash"); } }
        } }
        Ok(())
    }
    fn validate_function(&self,f:&Function)->Result<()> {
        let lifted=matches!(f.kind.as_str(),"oracle"|"lproc");
        let mut types=BTreeMap::new();
        for p in f.params.iter().flatten().chain(&f.regs) {
            if types.insert(p.name.clone(),p.ty.clone()).is_some() { return err("§3.7","duplicate function location or parameter"); }
        }
        let mut labels=BTreeSet::new();
        for b in &f.blocks {
            if !labels.insert(&b.name) { return err("§3.6","duplicate block label"); }
            if lifted && !b.params.is_empty() { return err("§3.7","lifted blocks have no parameters"); }
        }
        if lifted {
            for b in &f.blocks {
                for (i,ins) in b.instructions.iter().enumerate() {
                    let term=matches!(ins.op.as_str(),"qjump"|"qbranch"|"qjumpr"|"lift.ret"|"lift.retval"|"fault");
                    if term!=(i+1==b.instructions.len()) { return err("§3.6",format!("block {} must end in exactly one terminator",b.name)); }
                    self.validate_row(f,ins,&types,&labels)?;
                }
            }
        } else {
            self.validate_structured(f,&mut types,&labels)?;
        }
        Ok(())
    }
    fn validate_row(&self,f:&Function,i:&Instruction,types:&BTreeMap<String,Type>,labels:&BTreeSet<&String>)->Result<()> {
        let op=i.op.as_str(); let args=&i.operands;
        if !i.bodies.is_empty() { return err("§4.8","structured instructions are not lifted rows"); }
        let rd=|n:&str|->Result<u16> {
            let p=if f.kind=="oracle" { f.regs.iter().chain(&f.params[0]).chain(&f.params[4]).find(|p|p.name==n) } else { f.params[0].iter().chain(&f.params[1]).chain(&f.regs).find(|p|p.name==n) };
            p.ok_or_else(||Error(format!("§3.7: unreadable location {n}")))?.ty.width(self.ptr_width)
        };
        let dest=|n:&str|->Result<u16> { if !f.locations().iter().any(|p|p.name==n) { return err("§3.7",format!("not a writable location: {n}")); } types[n].width(self.ptr_width) };
        let arity=|n:usize,lhs:usize|->Result<()> { if args.len()!=n||i.lhs.len()!=lhs { err("§4.8",format!("{} expects {lhs} results and {n} operands at line {}",op,i.line)) } else { Ok(()) } };
        let eq=|a:u16,b:u16|->Result<()> { if a!=b { err("§3.3",format!("width mismatch in {op}: {a} != {b}")) } else { Ok(()) } };
        let label=|s:&str|->Result<()> { if s.strip_prefix('^').is_none_or(|s|!labels.iter().any(|x|x.as_str()==s)) { err("§3.6",format!("unknown label {s}")) } else { Ok(()) } };
        let addr=|s:&str|->Result<()> { eq(rd(s)?,self.ptr_width) };
        let length=|s:&str|->Result<()> { if integer(s) { let n=s.parse::<u128>().map_err(|_|Error("§3.3: invalid length".into()))?; if self.ptr_width<128 && n>=1u128<<self.ptr_width { return err("§3.3","length does not fit pointer"); } Ok(()) } else { rd(s).map(|_|()) } };
        let code=|s:&str|->Result<()> {
            let n=s.parse::<u128>().map_err(|_|Error("§3.3: fault code requires nonnegative literal".into()))?;
            let fw=if f.kind=="oracle" { f.params[2][0].ty.width(self.ptr_width)? } else { 4 };
            if fw<128 && n>=1u128<<fw { return err("§3.3","fault code does not fit fault register"); } Ok(())
        };
        if let Some(n)=op.strip_prefix("lift.const.i") {
            arity(1,1)?; let w=Type::parse(&format!("i{n}"))?.width(self.ptr_width)?; eq(dest(&i.lhs[0])?,w)?;
            if !integer(&args[0]) || parse_bits(&args[0],w).is_err() { return err("§16.1","invalid integer literal"); }
        } else if let Some(base)=op.strip_prefix("lift.").filter(|s|is_binop(s)||is_div(s)) {
            let _=base; arity(2,1)?; eq(rd(&args[0])?,rd(&args[1])?)?; eq(dest(&i.lhs[0])?,rd(&args[0])?)?;
        } else if let Some(pred)=op.strip_prefix("lift.icmp.") {
            arity(2,1)?; if !["eq","ne","ult","ule","slt","sle"].contains(&pred) { return err("§4.8","unknown comparison predicate"); }
            eq(rd(&args[0])?,rd(&args[1])?)?; eq(dest(&i.lhs[0])?,1)?;
        } else if op=="lift.select" {
            arity(3,1)?; eq(rd(&args[0])?,1)?; eq(rd(&args[1])?,rd(&args[2])?)?; eq(dest(&i.lhs[0])?,rd(&args[1])?)?;
        } else if let Some((cast,n))=op.strip_prefix("lift.").and_then(|s|s.split_once(".i")).filter(|(c,_)|["zext","sext","trunc"].contains(c)) {
            arity(1,1)?; let out=Type::parse(&format!("i{n}"))?.width(self.ptr_width)?; eq(dest(&i.lhs[0])?,out)?; let input=rd(&args[0])?;
            if if cast=="trunc" { out>=input } else { out<=input } { return err("§3.3","cast has wrong width direction"); }
        } else { match op {
            "lift.mov"=> { arity(1,1)?; eq(dest(&i.lhs[0])?,rd(&args[0])?)?; }
            "lift.parmov"=> {
                if args.len()!=1 { return err("§4.8","parmov needs one source tuple"); } let src=list(&args[0])?;
                if src.len()!=i.lhs.len()||src.is_empty() { return err("§4.8","parmov source/destination arity"); }
                let mut seen=BTreeSet::new(); for (d,s) in i.lhs.iter().zip(src) { if !seen.insert(d) { return err("§3.3","parmov destinations alias"); } eq(dest(d)?,rd(&s)?)?; }
            }
            "lift.label"=> { arity(1,1)?; label(&args[0])?; let w=dest(&i.lhs[0])?; if w<128 && (1u128<<w)<=self.functions.values().map(|f|f.instructions().count() as u128).sum() { return err("§3.3","label location cannot represent index space"); } }
            "lift.addr"=> {
                arity(1,1)?; eq(dest(&i.lhs[0])?,self.ptr_width)?;
                let n=args[0].strip_prefix('@').ok_or_else(||Error("§4.8: addr requires @name".into()))?;
                let global=self.section("layout").and_then(|l|l.get("globals").ok()).and_then(|g|g.object().ok()).is_some_and(|g|g.contains_key(n));
                if !global && !(f.kind=="oracle"&&f.params[3].iter().any(|p|p.name==n)) { return err("§4.2",format!("addr cannot name {n} in {}",f.name)); }
            }
            "lift.boundscheck"=> { arity(2,0)?; addr(&args[0])?; length(&args[1])?; }
            "lift.alloc"=> { arity(1,1)?; length(&args[0])?; eq(dest(&i.lhs[0])?,self.ptr_width)?; }
            "lift.free"=> { arity(1,0)?; addr(&args[0])?; }
            "lift.faultif"=> { arity(2,0)?; eq(rd(&args[0])?,1)?; code(&args[1])?; }
            "lift.call"=> {
                arity(2,1)?; let callee=args[0].strip_prefix('@').and_then(|n|self.functions.get(n)).ok_or_else(||Error("§3.7: unknown lproc".into()))?;
                if callee.kind!="lproc" { return err("§3.7","lift.call requires lproc"); }
                let src=list(&args[1])?; if src.len()!=callee.params[0].len() { return err("§3.3","lproc argument count"); }
                for (s,p) in src.iter().zip(&callee.params[0]) { eq(rd(s)?,p.ty.width(self.ptr_width)?)?; } eq(dest(&i.lhs[0])?,callee.params[1][0].ty.width(self.ptr_width)?)?;
            }
            "lift.ret"|"lift.retval"=> {
                arity(1,0)?;
                if (op=="lift.ret")!=(f.kind=="lproc") { return err("§4.2","ret/retval in wrong function kind"); }
                eq(rd(&args[0])?,f.params[1][0].ty.width(self.ptr_width)?)?;
            }
            "qjump"=> { arity(1,0)?; label(&args[0])?; }
            "qbranch"=> { arity(3,0)?; eq(rd(&args[0])?,1)?; label(&args[1])?; label(&args[2])?; }
            "qjumpr"=> { arity(1,0)?; let w=rd(&args[0])?; if w<128 && (1u128<<w)<=self.functions.values().map(|f|f.instructions().count() as u128).sum() { return err("§3.3","qjumpr width cannot represent index space"); } }
            "fault"=> { arity(1,0)?; code(&args[0])?; }
            "lift.oracle_call"|"lift.oracle_uncall"|"lift.perm_call"=>self.validate_region_call(f,i,types)?,
            _ if op.starts_with("lift.load.i")||op.starts_with("lift.store.i")=> {
                let load=op.starts_with("lift.load"); arity(if load {1}else{2},if load {1}else{0})?;
                let n=op.rsplit_once(".i").expect("suffix").1.parse::<u16>().map_err(|_|Error("§4.8: malformed memory width".into()))?;
                if n==0||n>128||n%8!=0 { return err("§3.3","load/store width must be a byte multiple up to 128"); }
                addr(&args[0])?; eq(if load {dest(&i.lhs[0])?} else {rd(&args[1])?},n)?;
            }
            _=>return err("§4.8",format!("unknown or invalid lifted opcode {op}")),
        } }
        Ok(())
    }
    fn validate_layout(&self)->Result<()> {
        let Some(l)=self.section("layout") else {
            for f in self.functions.values() {
                if f.kind=="lproc" || f.params.iter().flatten().any(|p|matches!(p.ty,Type::Region(..))) || f.instructions().any(|i|matches!(i.op.as_str(),"lift.addr"|"lift.boundscheck"|"lift.alloc"|"lift.free"|"lift.call"|"lift.ret"|"alloc"|"free")||i.op.starts_with("lift.load.")||i.op.starts_with("lift.store.")||i.op.starts_with("load.")||i.op.starts_with("store.")) { return err("§7.2","memory or lprocs require layout"); }
            } return Ok(());
        };
        l.fields(&["ptr_width","H","A_tab","S_cells","null_size","heap_base","heap_limit_max","globals","interface_regions"],&["block_image"])?;
        if l.get("ptr_width")?.uint()?!=self.ptr_width as u128 { return err("§7.2","layout pointer width mismatch"); }
        let h=l.get("H")?.uint()?; let a=l.get("A_tab")?.uint()?; let s=l.get("S_cells")?.uint()?;
        if h>self.ptr_width as u128 || h>127 || a>127 || s>127 { return err("§7.2","invalid layout size exponent"); }
        let capacity=1u128<<(h as u32); let entries=1u128<<(a as u32);
        let globals=l.get("globals")?.object()?;
        let null=l.get("null_size")?.uint()?; if null==0 { return err("§7.1","null region must be nonempty"); }
        let mut shared=vec![(0,null,"null".to_string()),(l.get("heap_base")?.uint()?,l.get("heap_limit_max")?.uint()?,"heap".into())];
        for (n,g) in globals {
            g.fields(&["base","size","image"],&[])?;
            let image=g.get("image")?.string()?;
            if image!=n || self.data.get(n).is_none_or(|b|b.len() as u128!=g.get("size").and_then(Json::uint).unwrap_or(u128::MAX)) { return err("§7.2","global image must be same-name data of declared size"); }
            shared.push((g.get("base")?.uint()?,g.get("size")?.uint()?,n.clone()));
        }
        let interfaces=l.get("interface_regions")?.object()?;
        for (n,regions) in interfaces {
            let f=self.functions.get(n).filter(|f|f.kind=="oracle").ok_or_else(||Error("§7.2: interface layout must name an oracle".into()))?;
            let regions=regions.object()?;
            if regions.len()!=f.params[3].len() { return err("§7.2","interface mapping count mismatch"); }
            let mut ranges=shared.clone();
            for p in &f.params[3] {
                let (a,w)=match p.ty {Type::Region(a,w)=>(a,w),_=>return err("§5.1","mem slot requires region")};
                if w!=8 { return err("§7.2","lifted interface regions require W=8"); }
                let r=regions.get(&p.name).ok_or_else(||Error("§7.2: missing interface mapping".into()))?; r.fields(&["base"],&[])?;
                ranges.push((r.get("base")?.uint()?,1u128<<a,p.name.clone()));
            }
            if globals.len() as u128+regions.len() as u128>entries { return err("§7.2","too many allocation pre-entries"); }
            validate_ranges(&ranges,capacity)?;
        }
        for f in self.functions.values().filter(|f|f.kind=="oracle"&&!f.params[3].is_empty()) { if !interfaces.contains_key(&f.name) { return err("§7.2","missing oracle interface layout"); } }
        if globals.len() as u128>entries { return err("§7.2","too many globals"); }
        validate_ranges(&shared,capacity)
    }
}

fn is_head(s:&str)->bool { ["fn cc @","fn r @","oracle @","lproc @","action @","perm_action @"].iter().any(|p|s.starts_with(p)) }
pub(crate) fn is_binop(op:&str)->bool { ["add","sub","mul","and","or","xor","shl","lshr","ashr"].contains(&op) }
pub(crate) fn is_div(op:&str)->bool { ["udiv","urem","sdiv","srem"].contains(&op) }
pub(crate) fn parse_bits(s:&str,width:u16)->Result<u128> {
    let negative=s.starts_with('-'); let magnitude=s.trim_start_matches('-').parse::<u128>().map_err(|_|Error("§16.1: integer exceeds 128 bits".into()))?;
    let n=if negative {0u128.wrapping_sub(magnitude)} else {magnitude}; Ok(n & mask(width))
}
pub(crate) fn mask(width:u16)->u128 { if width==128 {u128::MAX} else {(1u128<<width)-1} }
fn validate_header(f:&Function)->Result<()> {
    let empty=|indices:&[usize]|->Result<()> { if indices.iter().any(|i|!f.params[*i].is_empty()) {err("§16.2","nonempty reserved parameter list")}else{Ok(())} };
    let all=|ps:&[Param],kind:fn(&Type)->bool|->Result<()> { if ps.iter().any(|p|!kind(&p.ty)) {err("§3.3","invalid interface operand type")}else{Ok(())} };
    match f.kind.as_str() {
        "oracle"=> {
            if f.params[1].len()!=1||f.params[2].len()!=1 { return err("§5.1","oracle needs one target and one fault slot"); }
            for j in 0..3 { all(&f.params[j],|t|matches!(t,Type::Wire(_)))?; }
            if f.params[2][0].ty.width(1)?<4 {return err("§5.4","fault width must be at least 4");}
            all(&f.params[3],|t|matches!(t,Type::Region(..)))?;
            if f.params[0].iter().chain(&f.params[3]).any(|p|p.role.as_deref()!=Some("in")) {return err("§5.1","oracle inputs and regions require in role");}
        }
        "lproc"=> {empty(&[2,3,4])?; if f.params[1].len()!=1 {return err("§3.7","lproc needs one result location");} all(&f.params[0],|t|matches!(t,Type::Reg(_)))?;all(&f.params[1],|t|matches!(t,Type::Reg(_)))?;}
        "action"|"perm_action"=> {empty(&[4])?; for j in 0..2 {all(&f.params[j],|t|matches!(t,Type::Wire(_)))?;} all(&f.params[2],|t|matches!(t,Type::Region(..)))?;}
        _=>empty(&[2,3,4])?,
    }
    if matches!(f.kind.as_str(),"oracle"|"lproc") {all(&f.regs,|t|matches!(t,Type::Reg(_)))?;if !f.bounds.is_empty(){return err("§5.3","bounds only on actions");}}
    Ok(())
}
fn validate_ranges(ranges:&[(u128,u128,String)],capacity:u128)->Result<()> {
    let mut sorted=Vec::new();
    for (base,size,n) in ranges { let end=base.checked_add(*size).ok_or_else(||Error("§7.2: range overflow".into()))?; if end>capacity {return err("§7.2",format!("{n} exceeds address space"));} if *size!=0{sorted.push((*base,end,n));} }
    sorted.sort();
    for p in sorted.windows(2) { if p[0].1>p[1].0 {return err("§7.2",format!("overlapping layout ranges {} and {}",p[0].2,p[1].2));} } Ok(())
}

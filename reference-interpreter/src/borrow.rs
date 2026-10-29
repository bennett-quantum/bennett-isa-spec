//! §5.6 translation lemma and exact small-frame permutation verification.
use crate::{err,object::list,validation::{gate,gate_operands},Error,Function,Instruction,Object,Result,Type};
use std::collections::{BTreeMap,BTreeSet};

pub fn validate_translation_lemma(v:&[Instruction],w:&[Instruction],x:&BTreeSet<String>,borrowed:&BTreeSet<String>)->Result<()>{
    if !x.is_disjoint(borrowed){return err("§5.6","translation lemma requires X disjoint from B");}
    let flips=|sequence:&[Instruction]|->Result<Vec<(Vec<String>,String)>>{sequence.iter().map(|i|{if !["x","cx","ccx","mcx"].contains(&i.op.as_str()){return err("§5.6","translation lemma contains a non-X-family gate");}let(c,t,_,_)=gate_operands(i)?;Ok((c,t[0].clone()))}).collect()};
    let v=flips(v)?;let w=flips(w)?;let controls:BTreeSet<_>=w.iter().flat_map(|(c,_)|c.iter().cloned()).collect();
    for(c,t)in v{if !borrowed.contains(&t)||c.iter().any(|c|!x.contains(c)){return err("§5.6","V must target only B with controls only in X");}}
    for(c,t)in w{if x.contains(&t)||borrowed.contains(&t)||controls.contains(&t){return err("§5.6","W target overlaps X, B, or W's controls");}if c.iter().filter(|c|borrowed.contains(*c)).count()>1{return err("§5.6","W gate has more than one borrowed control");}}
    Ok(())
}
#[derive(Clone,Debug,PartialEq,Eq)]
struct Operation{op:String,controls:Vec<(String,bool)>,targets:Vec<String>,permutation:bool,opaque:bool}
impl Operation{
    fn wires(&self)->impl Iterator<Item=&String>{self.controls.iter().map(|(w,_)|w).chain(&self.targets)}
    fn instruction(&self)->Instruction{let controls=self.controls.iter().map(|(w,_)|w.clone()).collect::<Vec<_>>();let(op,operands)=match controls.len(){0=>("x".into(),self.targets.clone()),1=>("cx".into(),[controls,self.targets.clone()].concat()),2=>("ccx".into(),[controls,self.targets.clone()].concat()),_ =>("mcx".into(),vec![format!("({})",controls.join(", ")),self.targets[0].clone()])};Instruction{lhs:Vec::new(),op,operands,bodies:Vec::new(),line:0}}
}
struct Compiler{bundles:BTreeMap<String,Vec<String>>,borrowed:BTreeSet<String>,operations:Vec<Operation>,generation:usize}
impl Compiler{
    fn bundle(&self,name:&str)->Result<Vec<String>>{if let Some((base,index))=name.split_once('['){let index=index.trim_end_matches(']').parse::<usize>().map_err(|_|Error("§3.3: indexed borrow operand".into()))?;return self.bundles.get(base).and_then(|w|w.get(index)).map(|w|vec![w.clone()]).ok_or_else(||Error("§3.3: unknown indexed borrow operand".into()));}self.bundles.get(name).cloned().ok_or_else(||Error(format!("§3.3: unknown borrow operand {name}")))}
    fn produce(&mut self,name:&str,width:usize)->Vec<String>{let generation=self.generation;self.generation+=1;let wires=(0..width).map(|bit|format!("v{generation}_{bit}")).collect::<Vec<_>>();self.bundles.insert(name.into(),wires.clone());wires}
    fn flip(&mut self,source:String,target:String,context:&[(String,bool)]){let mut controls=context.to_vec();if !controls.contains(&(source.clone(),true)){controls.push((source,true));}self.operations.push(Operation{op:"x".into(),controls,targets:vec![target],permutation:true,opaque:false});}
    fn compile(&mut self,seq:&[Instruction],context:&[(String,bool)])->Result<()>{let mut predicate=context.to_vec();for i in seq{let a=&i.operands;let op=i.op.as_str();match op{
        "qalloc"|"qborrow"=>{let wires=self.produce(&i.lhs[0],a[1].parse().map_err(|_|Error("§4.3: allocation width".into()))?);if op=="qborrow"{self.borrowed.extend(wires);}},
        "qfree"|"qreturn"=>{self.bundles.remove(&a[0]);},
        "qpred"=>{predicate.push((self.bundle(&a[0])?[0].clone(),true));},"qunpred"=>{predicate.pop().ok_or_else(||Error("§3.5: borrow predicate underflow".into()))?;},
        "qcopy"|"quncopy"=>{let source=self.bundle(&a[0])?;let target=if op=="qcopy"{self.produce(&i.lhs[0],source.len())}else{self.bundle(&a[1])?};for(s,t)in source.into_iter().zip(target){self.flip(s,t,&predicate);}if op=="quncopy"{self.bundles.remove(&a[1]);}},
        "qxor"=>{for(s,t)in self.bundle(&a[0])?.into_iter().zip(self.bundle(&a[1])?){self.flip(s,t,&predicate);}},
        "qexch"=>{for(s,t)in self.bundle(&a[0])?.into_iter().zip(self.bundle(&a[1])?){self.operations.push(Operation{op:"swap".into(),controls:predicate.clone(),targets:vec![s,t],permutation:true,opaque:false});}},
        "qif"=>{let selector=self.bundle(&a[0])?[0].clone();let mut yes=predicate.clone();yes.push((selector.clone(),true));self.compile(&i.bodies[0],&yes)?;let mut no=predicate.clone();no.push((selector,false));self.compile(&i.bodies[1],&no)?;},
        "qloop"=>{let count=a[0].parse::<usize>().map_err(|_|Error("§4.6: loop size".into()))?;if count>1_000_000{return err("§5.6","borrow circuit expansion exceeds reference verifier storage; supply a structural/library claim");}for iteration in 0..count{let mut c=predicate.clone();if a.len()==3{let selectors=self.bundle(&a[2])?;for selector in &selectors[..=iteration]{c.push((selector.clone(),true));}}self.compile(&i.bodies[0],&c)?;}},
        _ if gate(op)=>{let(controls,targets,_,_)=gate_operands(i)?;let mut context=predicate.clone();for control in controls{let wire=self.bundle(&control)?[0].clone();if !context.contains(&(wire.clone(),true)){context.push((wire,true));}}let targets=targets.iter().map(|w|self.bundle(w).map(|w|w[0].clone())).collect::<Result<Vec<_>>>()?;self.operations.push(Operation{op:if op=="swap"{"swap".into()}else{op.into()},controls:context,targets,permutation:["x","cx","ccx","mcx","swap"].contains(&op),opaque:false});},
        _=>{let mut operands=Vec::new();for operand in a{if operand.starts_with('('){operands.extend(list(operand)?);}else{operands.push(operand.clone());}}let touched=operands.iter().filter_map(|n|self.bundles.get(n)).flatten().cloned().collect();self.operations.push(Operation{op:op.into(),controls:predicate.clone(),targets:touched,permutation:true,opaque:true});},
    }}Ok(())}
}
impl Object{
    pub(crate) fn validate_borrow_frames(&self,f:&Function)->Result<()>{
        fn has_borrow(seq:&[Instruction])->bool{seq.iter().any(|i|i.op=="qborrow"||i.bodies.iter().any(|b|has_borrow(b)))}
        if !f.blocks.iter().any(|b|has_borrow(&b.instructions)){return Ok(());}
        let mut compiler=Compiler{bundles:BTreeMap::new(),borrowed:BTreeSet::new(),operations:Vec::new(),generation:0};
        for p in f.params.iter().flatten(){if let Type::Wire(width)=p.ty{compiler.produce(&p.name,width as usize);}}
        for block in &f.blocks{compiler.compile(&block.instructions,&[])?;}
        let mut start=0;while start<compiler.operations.len(){let op=&compiler.operations[start];if !op.permutation{if op.wires().any(|w|compiler.borrowed.contains(w)){return err("§5.6","borrowed wire touched by a non-permutation instruction");}start+=1;continue;}
            let end=(start..compiler.operations.len()).find(|i|!compiler.operations[*i].permutation).unwrap_or(compiler.operations.len());
            let touches=|op:&Operation|op.wires().any(|w|compiler.borrowed.contains(w));let first=(start..end).find(|i|touches(&compiler.operations[*i]));let last=(start..end).rfind(|i|touches(&compiler.operations[*i]));
            if let(Some(first),Some(last))=(first,last){verify_frame(&compiler.operations[first..=last],&compiler.borrowed)?;}start=end;
        }Ok(())
    }
}
fn verify_frame(frame:&[Operation],borrowed:&BTreeSet<String>)->Result<()>{
    if frame.iter().any(|o|o.opaque){return err("§5.6","borrow frame requires a checked full-domain library claim for its opaque operation");}
    let only_flips=frame.iter().all(|op|op.op!="swap"&&op.controls.iter().all(|(_,v)|*v));
    if only_flips&&frame.len().is_multiple_of(2){let half=frame.len()/2;for vlen in 1..half{let wlen=half-vlen;let v=&frame[..vlen];let w=&frame[vlen..half];if frame[half..half+vlen].iter().eq(v.iter().rev())&&frame[half+vlen..]==*w{
        let x=v.iter().flat_map(|o|o.controls.iter().map(|(w,_)|w.clone())).collect();if validate_translation_lemma(&v.iter().map(Operation::instruction).collect::<Vec<_>>(),&w.iter().map(Operation::instruction).collect::<Vec<_>>(),&x,borrowed).is_ok(){return Ok(());}let _=wlen;
    }}}
    let wires:BTreeSet<_>=frame.iter().flat_map(Operation::wires).cloned().collect();if wires.len()>16{return err("§5.6","borrow frame exceeds 16 exhaustive wires and has no valid structural proof");}let positions:BTreeMap<_,_>=wires.into_iter().enumerate().map(|(i,w)|(w,i)).collect();let borrowed_indices:Vec<_>=positions.iter().filter_map(|(w,i)|borrowed.contains(w).then_some(*i)).collect();let mut outputs=BTreeMap::new();
    for input in 0..1usize<<positions.len(){let mut output=input;for operation in frame{if operation.controls.iter().all(|(w,value)|(output&(1usize<<positions[w])!=0)==*value){if operation.op=="swap"{let a=1usize<<positions[&operation.targets[0]];let b=1usize<<positions[&operation.targets[1]];if(output&a!=0)!=(output&b!=0){output^=a|b;}}else{output^=1usize<<positions[&operation.targets[0]];}}}
        let borrowed_mask=borrowed_indices.iter().fold(0,|m,i|m|(1usize<<i));if input&borrowed_mask!=output&borrowed_mask{return err("§5.6","borrow frame changes a borrowed wire");}let key=input&!borrowed_mask;let value=output&!borrowed_mask;if outputs.get(&key).is_some_and(|prior|*prior!=value){return err("§5.6","borrow frame output depends on borrowed entry values");}outputs.insert(key,value);
    }Ok(())
}

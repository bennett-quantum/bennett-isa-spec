//! Unbounded-width §14.1 bit-vector execution, with independent dynamic activations.
use crate::{err,object::{integer,is_binop,is_div,list,mask},oracle::{Memory,Program},Argument,BitVector,Budget,Error,Function,Object,OracleInput,Result,Type};
use std::collections::BTreeMap;
#[derive(Clone,Debug,Default,PartialEq)]
pub struct OracleVectorInput {pub ins:Vec<BitVector>,pub args:Vec<Argument>,pub regions:BTreeMap<String,Vec<u8>>}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct OracleVectorOutcome {pub payload:BitVector,pub code:BitVector,pub steps:u64}
struct Activation {function:String,values:BTreeMap<String,BitVector>,ret_pc:usize,destination:String}
impl Activation {
    fn new(f:&Function,ptr:u16)->Result<Self>{Ok(Self{function:f.name.clone(),values:f.locations().iter().map(|p|Ok((p.name.clone(),BitVector::zero(p.ty.width(ptr)? as usize)?))).collect::<Result<_>>()?,ret_pc:0,destination:String::new()})}
    fn read(&self,n:&str)->Result<BitVector>{self.values.get(n).cloned().ok_or_else(||Error(format!("§3.7: unreadable operand {n}")))}
    fn write(&mut self,f:&Function,n:&str,value:BitVector,ptr:u16)->Result<()>{let p=f.locations().into_iter().find(|p|p.name==n).ok_or_else(||Error("§3.7: unwritable location".into()))?;self.values.insert(n.into(),value.resize(p.ty.width(ptr)? as usize,false)?);Ok(())}
}
impl Object {
    pub fn run_oracle_vectors(
        &self,
        name: &str,
        input: &OracleVectorInput,
        budget: Budget,
    ) -> Result<OracleVectorOutcome> {
        self.validate_budget(budget)?;
        let program = Program::new(self, name)?;
        let root = &self.functions[name];
        if input.ins.len() != root.params[0].len()
            || input.args.len() != root.params[4].len()
            || input.regions.len() != root.params[3].len()
        {
            return err("§5.1", "oracle interface argument count mismatch");
        }
        let mut activation = Activation::new(root, self.ptr_width)?;
        let mut arguments = BTreeMap::new();
        for (p, v) in root.params[0].iter().zip(&input.ins) {
            if v.width() != p.ty.width(self.ptr_width)? as usize {
                return err("§3.3", "input value exceeds declared width");
            }
            activation.values.insert(p.name.clone(), v.clone());
        }
        for (p, v) in root.params[4].iter().zip(&input.args) {
            match (&p.ty, v) {
                (Type::Int(w), Argument::Bits(v)) if *v & !mask(*w) == 0 => {
                    activation.values.insert(p.name.clone(), BitVector::from_u128(p.ty.width(self.ptr_width)? as usize,*v)?);
                }
                (Type::Ptr, Argument::Bits(v)) if *v & !mask(self.ptr_width) == 0 => {
                    activation.values.insert(p.name.clone(), BitVector::from_u128(p.ty.width(self.ptr_width)? as usize,*v)?);
                }
                (Type::F64, Argument::Float(v)) if v.is_finite() => {}
                (Type::Token, Argument::Token(b)) => {
                    self.validate_budget(*b)?;
                }
                _ => return err("§3.3", "classical argument type/width mismatch"),
            }
            arguments.insert(p.name.clone(), v.clone());
        }
        let mut memory = Memory::initial(self, root, &OracleInput{regions:input.regions.clone(),..Default::default()})?;
        let mut stack: Vec<Activation> = Vec::new();
        let mut sp = 0u128;
        let mut pc = 0;
        let mut steps = 0;
        let target_width=root.params[1][0].ty.width(self.ptr_width)? as usize;
        let fault_width=root.params[2][0].ty.width(self.ptr_width)? as usize;
        let mut result = BitVector::zero(target_width)?;
        macro_rules! fault {
            ($code:expr) => {
                return Ok(OracleVectorOutcome {
                    payload: BitVector::zero(target_width)?,
                    code: BitVector::from_u128(fault_width,$code)?,
                    steps,
                })
            };
        }
        while steps < budget.steps {
            if pc == program.halt {
                return Ok(OracleVectorOutcome {
                    payload: result,
                    code: BitVector::zero(fault_width)?,
                    steps,
                });
            }
            let (f, i) = program.rows[pc];
            if activation.function != f.name {
                return err("§3.7", "control escaped its active function");
            }
            steps += 1;
            let mut next = pc + 1;
            let op = i.op.as_str();
            let a = &i.operands;
            let rd = |n: &str| activation.read(n);
            let width = |n: &str| -> Result<u16> {
                f.locations()
                    .into_iter()
                    .chain(if f.kind == "oracle" {
                        f.params[0].iter().chain(&f.params[4]).collect::<Vec<_>>()
                    } else {
                        Vec::new()
                    })
                    .find(|p| p.name == n)
                    .ok_or_else(|| Error(format!("§3.7: unknown width for {n}")))?
                    .ty
                    .width(self.ptr_width)
            };
            let mut output = None;
            if let Some(n) = op.strip_prefix("lift.const.i") {
                output = Some(BitVector::parse_decimal(n.parse().expect("validated constant width"), &a[0])?);
            } else if let Some(bin) = op.strip_prefix("lift.").filter(|s| is_binop(s)) {
                output = Some(rd(&a[0])?.binary(bin,&rd(&a[1])?)?);
            } else if let Some(div) = op.strip_prefix("lift.").filter(|s| is_div(s)) {
                match rd(&a[0])?.binary(div,&rd(&a[1])?) {
                    Ok(v) => output = Some(v),
                    Err(e) if e.0.contains("FAULT_UB_DIV0") => fault!(5),
                    Err(e) if e.0.contains("FAULT_UB_SDIV_OVERFLOW") => fault!(6),
                    Err(e) => return Err(e),
                }
            } else if let Some(pred) = op.strip_prefix("lift.icmp.") {
                let x=rd(&a[0])?;let y=rd(&a[1])?;let order=if pred.starts_with('s'){x.signed_cmp(&y)?}else{x.unsigned_cmp(&y)?};
                let value=match pred{"eq"=>order.is_eq(),"ne"=>!order.is_eq(),"ult"|"slt"=>order.is_lt(),"ule"|"sle"=>!order.is_gt(),_=>return err("§4.8","unknown comparison")};
                output=Some(BitVector::from_u128(1,u128::from(value))?);
            } else if let Some((cast, target)) = op
                .strip_prefix("lift.")
                .and_then(|s| s.split_once(".i"))
                .filter(|(c, _)| ["zext", "sext", "trunc"].contains(c))
            {
                let v = rd(&a[0])?;
                output=Some(v.resize(target.parse().expect("cast width"),cast=="sext")?);
            } else if let Some(n) = op.strip_prefix("lift.load.i") {
                let w = n.parse::<u16>().expect("validated load width");
                let p = rd(&a[0])?.to_u128()?;
                if !memory.valid(p, (w / 8) as u128) {
                    fault!(7);
                }
                output = Some(BitVector::from_u128(w as usize,memory.load(p,w))?);
            } else if let Some(n) = op.strip_prefix("lift.store.i") {
                let w = n.parse::<u16>().expect("validated store width");
                let p = rd(&a[0])?.to_u128()?;
                let v = rd(&a[1])?;
                if !memory.valid(p, (w / 8) as u128) {
                    fault!(7);
                }
                memory.store(p,w,v.to_u128()?);
            } else {
                match op {
                    "lift.mov" => output = Some(rd(&a[0])?),
                    "lift.select" => {
                        output = Some(rd(if !rd(&a[0])?.is_zero() { &a[1] } else { &a[2] })?)
                    }
                    "lift.parmov" => {
                        let values = list(&a[0])?
                            .iter()
                            .map(|s| rd(s))
                            .collect::<Result<Vec<_>>>()?;
                        for (d, v) in i.lhs.iter().zip(values) {
                            activation.write(f, d, v, self.ptr_width)?;
                        }
                    }
                    "lift.label" => output = Some(BitVector::from_u128(width(&i.lhs[0])? as usize,program.label(&f.name,&a[0])? as u128)?),
                    "lift.addr" => {
                        output = Some(BitVector::from_u128(self.ptr_width as usize,*memory.addresses.get(a[0].trim_start_matches('@')).ok_or_else(||Error("§4.2: address not mapped".into()))?)?);
                    }
                    "lift.boundscheck" => {
                        let p = rd(&a[0])?.to_u128()?;
                        let n = if integer(&a[1]) {
                            a[1].parse::<u128>().expect("validated length")
                        } else {
                            match rd(&a[1])?.to_u128(){Ok(n)=>n,Err(_)=>fault!(7)}
                        };
                        if !memory.valid(p, n) {
                            fault!(7);
                        }
                    }
                    "lift.alloc" => {
                        let n = if integer(&a[0]) {
                            a[0].parse::<u128>().expect("validated length")
                        } else {
                            match rd(&a[0])?.to_u128(){Ok(n)=>n,Err(_)=>fault!(2)}
                        };
                        if memory.allocation_fails(n, budget) {
                            fault!(2);
                        }
                        output = Some(BitVector::from_u128(self.ptr_width as usize,memory.allocate(n))?);
                    }
                    "lift.free" => {
                        let p = rd(&a[0])?.to_u128()?;
                        if !memory.free(p) {
                            fault!(7);
                        }
                    }
                    "lift.faultif" => {
                        if !rd(&a[0])?.is_zero() {
                            return Ok(OracleVectorOutcome{payload:BitVector::zero(target_width)?,code:BitVector::parse_decimal(fault_width,&a[1])?,steps});
                        }
                    }
                    "fault" => return Ok(OracleVectorOutcome{payload:BitVector::zero(target_width)?,code:BitVector::parse_decimal(fault_width,&a[0])?,steps}),
                    "qjump" => next = program.label(&f.name, &a[0])?,
                    "qbranch" => {
                        next =
                            program.label(&f.name, if !rd(&a[0])?.is_zero() { &a[1] } else { &a[2] })?
                    }
                    "qjumpr" => {
                        let address = match rd(&a[0])?.to_u128(){Ok(n)=>n,Err(_)=>fault!(4)};
                        let (start, end) = program.ranges[&f.name];
                        if address < (start as u128) || address >= end as u128 {
                            fault!(4);
                        }
                        next = address as usize;
                    }
                    "lift.call" => {
                        let callee = &self.functions[a[0].trim_start_matches('@')];
                        let cells = callee.frame_cells() as u128;
                        if stack.len() as u128 + 1 > budget.depth
                            || sp.checked_add(cells).is_none_or(|p| p > budget.stack)
                        {
                            fault!(3);
                        }
                        let values = list(&a[1])?
                            .iter()
                            .map(|s| rd(s))
                            .collect::<Result<Vec<_>>>()?;
                        let mut called = Activation::new(callee, self.ptr_width)?;
                        called.ret_pc = pc + 1;
                        called.destination = i.lhs[0].clone();
                        for (p, v) in callee.params[0].iter().zip(values) {
                            called.values.insert(p.name.clone(), v);
                        }
                        sp += cells;
                        stack.push(activation);
                        activation = called;
                        next = program.ranges[&callee.name].0;
                    }
                    "lift.ret" => {
                        if stack.is_empty() {
                            fault!(4);
                        }
                        let v = rd(&a[0])?;
                        let ret = activation.ret_pc;
                        let dest = activation.destination.clone();
                        activation = stack.pop().expect("nonempty activation stack");
                        let caller = &self.functions[&activation.function];
                        activation.write(caller, &dest, v, self.ptr_width)?;
                        next = ret;
                    }
                    "lift.retval" => {
                        result = result.binary("xor",&rd(&a[0])?)?;
                        return Ok(OracleVectorOutcome {
                            payload: result,
                            code: BitVector::zero(fault_width)?,
                            steps,
                        });
                    }
                    "lift.oracle_call" | "lift.oracle_uncall" => {
                        let callee = &self.functions[a[0].trim_start_matches('@')];
                        let token = arguments
                            .get(&a[1])
                            .and_then(|a| {
                                if let Argument::Token(b) = a {
                                    Some(*b)
                                } else {
                                    None
                                }
                            })
                            .ok_or_else(|| {
                                Error("§4.2: nested oracle call needs a budget token".into())
                            })?;
                        let mut child=OracleVectorInput{ins:list(&a[2])?.iter().map(|s|rd(s)).collect::<Result<Vec<_>>>()?,..Default::default()};
                        let region_args = list(&a[5])?;
                        for (p, n) in callee.params[3].iter().zip(region_args) {
                            let base = memory.addresses[&n];
                            let Type::Region(aw, _) = p.ty else {
                                unreachable!("validated region")
                            };
                            let len = usize::try_from(1u128 << aw).map_err(|_| {
                                Error("§7.2: region exceeds host addressable storage".into())
                            })?;
                            child.regions.insert(
                                p.name.clone(),
                                (0..len).map(|i| memory.byte(base + i as u128)).collect(),
                            );
                        }
                        for arg in list(&a[6])? {
                            child.args.push(match arguments.get(&arg) {
                                Some(v) => v.clone(),
                                None => Argument::Bits(rd(&arg)?.to_u128()?),
                            });
                        }
                        let r = self.run_oracle_vectors(&callee.name, &child, token)?;
                        let tgt = rd(&a[3])?.binary("xor",&r.payload)?;
                        let code = rd(&a[4])?.binary("xor",&r.code)?;
                        activation.write(f, &a[3], tgt, self.ptr_width)?;
                        activation.write(f, &a[4], code, self.ptr_width)?;
                    }
                    "lift.perm_call" => {
                        let callee = &self.functions[a[0].trim_start_matches('@')];
                        let destinations = list(&a[1])?;
                        let mutable = destinations
                            .iter()
                            .map(|n| rd(n))
                            .collect::<Result<Vec<_>>>()?;
                        let inputs = list(&a[2])?
                            .iter()
                            .map(|n| rd(n))
                            .collect::<Result<Vec<_>>>()?;
                        let mut regions = BTreeMap::new();
                        for (p, n) in callee.params[2].iter().zip(list(&a[3])?) {
                            let base = memory.addresses[&n];
                            let Type::Region(aw, _) = p.ty else {
                                unreachable!("region")
                            };
                            let len = usize::try_from(1u128 << aw)
                                .map_err(|_| Error("§7.2: action region too large".into()))?;
                            regions.insert(
                                p.name.clone(),
                                (0..len).map(|i| memory.byte(base + i as u128)).collect(),
                            );
                        }
                        let args = list(&a[4])?
                            .iter()
                            .map(|n| match arguments.get(n) {
                                Some(v) => Ok(v.clone()),
                                None => Ok(Argument::Bits(rd(n)?.to_u128()?)),
                            })
                            .collect::<Result<Vec<_>>>()?;
                        let values = self.permutation_action_vectors(
                            &callee.name,
                            &mutable,
                            &inputs,
                            &regions,
                            &args,
                            false,
                        )?;
                        for (n, v) in destinations.iter().zip(values) {
                            activation.write(f, n, v, self.ptr_width)?;
                        }
                    }
                    _ => return err("§4.8", format!("unimplemented sequential row {op}")),
                }
            }
            if let Some(v) = output {
                activation.write(f, &i.lhs[0], v, self.ptr_width)?;
            }
            pc = next;
        }
        Ok(OracleVectorOutcome {
            payload: BitVector::zero(target_width)?,
            code: BitVector::from_u128(fault_width,1)?,
            steps,
        })
    }
}

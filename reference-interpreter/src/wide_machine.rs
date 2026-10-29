//! Literal §3.6/§3.7 basis machine. Frames contain displaced/captured values;
//! reversal uses inverse operations, never a saved pre-step state.
use crate::{
    err,
    object::{integer, is_binop, is_div, list},
    oracle::{Allocation, Memory, Program},
    Argument, BitVector, Budget, Error, Function, Instruction, Object, OracleVectorInput,
    OracleVectorOutcome, Result, Type,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
struct Word(BitVector);
impl Default for Word {
    fn default() -> Self {
        Self(BitVector::zero(128).expect("word"))
    }
}
impl From<u128> for Word {
    fn from(n: u128) -> Self {
        Self(BitVector::from_u128(128, n).expect("word"))
    }
}
impl From<i32> for Word {
    fn from(n: i32) -> Self {
        assert!(n >= 0);
        Self::from(n as u128)
    }
}
impl PartialEq for Word {
    fn eq(&self, other: &Self) -> bool {
        let width = self.0.width().max(other.0.width());
        self.0.resize(width, false).expect("word") == other.0.resize(width, false).expect("word")
    }
}
impl Eq for Word {}
impl PartialEq<u128> for Word {
    fn eq(&self, other: &u128) -> bool {
        self.0.to_u128() == Ok(*other)
    }
}
impl std::ops::BitXorAssign<Word> for Word {
    fn bitxor_assign(&mut self, other: Word) {
        let width = self.0.width().max(other.0.width());
        self.0 = self
            .0
            .resize(width, false)
            .expect("word")
            .binary("xor", &other.0.resize(width, false).expect("word"))
            .expect("word xor");
    }
}
impl std::ops::BitXorAssign<u128> for Word {
    fn bitxor_assign(&mut self, other: u128) {
        *self ^= Word::from(other);
    }
}
impl Word {
    fn number(&self) -> Result<u128> {
        self.0.to_u128()
    }
    fn exchange_counter(&mut self, counter: &mut u128) -> Result<()> {
        let incoming = self.number()?;
        *self = Word::from(*counter);
        *counter = incoming;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
struct Slot {
    word: Word,
    size: u128,
    live: bool,
    pinned: bool,
}
impl Slot {
    fn word(word: impl Into<Word>) -> Self {
        Self {
            word: word.into(),
            ..Self::default()
        }
    }
    fn entry(e: Allocation) -> Self {
        Self {
            word: e.base.into(),
            size: e.size,
            live: e.live,
            pinned: e.pinned,
        }
    }
    fn allocation(&self) -> Allocation {
        Allocation {
            base: self.word.number().expect("allocation base"),
            size: self.size,
            live: self.live,
            pinned: self.pinned,
        }
    }
    fn xor(&mut self, other: &Self) {
        self.word ^= other.word.clone();
        self.size ^= other.size;
        self.live ^= other.live;
        self.pinned ^= other.pinned;
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Default)]
struct Frame {
    old_pc: usize,
    slots: Vec<Slot>,
    flag: bool,
    succ: usize,
}

/// The complete mutable basis tuple, including every retained frame and cell.
/// Immutable program text, input bindings and budget are held by OracleVectorMachine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BasisVectorConfiguration {
    locations: BTreeMap<(String, String), Word>,
    stack: BTreeMap<u128, Word>,
    pub pc: usize,
    pub pc_prime: usize,
    pub faulted: bool,
    code: Word,
    result_val: Word,
    pub sp: u128,
    pub fp: u128,
    pub depth: u128,
    pub atop: u128,
    memory: Memory,
    frames: Vec<Frame>,
}
pub struct OracleVectorMachine<'a> {
    object: &'a Object,
    root: String,
    input: OracleVectorInput,
    budget: Budget,
    program: Program<'a>,
    state: BasisVectorConfiguration,
    used: usize,
    ids: Vec<(String, String)>,
}
#[derive(Clone)]
enum Destination {
    Location(String),
    Memory(String, u16),
    EntryAtop,
    Heap,
    Atop,
}

impl<'a> OracleVectorMachine<'a> {
    pub fn new(
        object: &'a Object,
        name: &str,
        input: &OracleVectorInput,
        budget: Budget,
    ) -> Result<Self> {
        object.validate_budget(budget)?;
        let program = Program::new(object, name)?;
        let root = &object.functions[name];
        if root.params[0].len() != input.ins.len()
            || root.params[4].len() != input.args.len()
            || root.params[3].len() != input.regions.len()
        {
            return err("§5.1", "machine interface argument count mismatch");
        }
        let mut locations = BTreeMap::new();
        let mut ids = Vec::new();
        for (f, _) in &program.rows {
            for p in f.locations() {
                let key = (f.name.clone(), p.name.clone());
                if let std::collections::btree_map::Entry::Vacant(entry) = locations.entry(key) {
                    ids.push(entry.key().clone());
                    entry.insert(Word(BitVector::zero(
                        p.ty.width(object.ptr_width)? as usize
                    )?));
                }
            }
        }
        for (p, v) in root.params[0].iter().zip(&input.ins) {
            if v.width() != p.ty.width(object.ptr_width)? as usize {
                return err("§3.3", "machine input exceeds width");
            }
            locations.insert((name.into(), p.name.clone()), Word(v.clone()));
        }
        for (p, v) in root.params[4].iter().zip(&input.args) {
            match (&p.ty, v) {
                (Type::Int(w), Argument::Bits(v)) if (*w == 128 || *v < 1u128 << w) => {
                    locations.insert(
                        (name.into(), p.name.clone()),
                        Word(BitVector::from_u128(
                            p.ty.width(object.ptr_width)? as usize,
                            *v,
                        )?),
                    );
                }
                (Type::Ptr, Argument::Bits(v))
                    if (object.ptr_width == 128 || *v < 1u128 << object.ptr_width) =>
                {
                    locations.insert(
                        (name.into(), p.name.clone()),
                        Word(BitVector::from_u128(
                            p.ty.width(object.ptr_width)? as usize,
                            *v,
                        )?),
                    );
                }
                (Type::Token, Argument::Token(b)) => object.validate_budget(*b)?,
                (Type::F64, Argument::Float(x)) if x.is_finite() => {}
                _ => return err("§3.3", "machine classical argument type mismatch"),
            }
        }
        let memory = Memory::initial(
            object,
            root,
            &crate::OracleInput {
                regions: input.regions.clone(),
                ..Default::default()
            },
        )?;
        let atop = memory.allocations.len() as u128;
        let slots = program
            .rows
            .iter()
            .map(|(_, i)| match i.op.as_str() {
                "lift.call" => list(&i.operands[1]).map(|a| a.len() + 3).unwrap_or(0),
                "lift.ret" => 3,
                "lift.alloc" => 4,
                "lift.parmov" => i.lhs.len(),
                _ => usize::from(!i.lhs.is_empty() || i.op.starts_with("lift.store.")),
            })
            .max()
            .unwrap_or(0);
        let count = usize::try_from(budget.steps)
            .map_err(|_| Error("§3.6: frame count exceeds host address space".into()))?;
        let frame = Frame {
            slots: vec![Slot::default(); slots],
            ..Frame::default()
        };
        let mut frames = Vec::new();
        frames
            .try_reserve_exact(count)
            .map_err(|e| Error(format!("§3.6: cannot allocate step frames: {e}")))?;
        frames.resize(count, frame);
        Ok(Self {
            object,
            root: name.into(),
            input: input.clone(),
            budget,
            program,
            state: BasisVectorConfiguration {
                locations,
                stack: BTreeMap::new(),
                pc: 0,
                pc_prime: 0,
                faulted: false,
                code: Word::default(),
                result_val: Word::default(),
                sp: 0,
                fp: 0,
                depth: 0,
                atop,
                memory,
                frames,
            },
            used: 0,
            ids,
        })
    }
    pub fn snapshot(&self) -> BasisVectorConfiguration {
        self.state.clone()
    }
    pub fn frames_clean(&self) -> bool {
        self.state.frames.iter().all(|f| {
            f.old_pc == 0 && !f.flag && f.succ == 0 && f.slots.iter().all(|s| s == &Slot::default())
        })
    }
    /// §5.2 commit, with caller-owned target/fault entry values. These writes
    /// persist while all internal state is subsequently prefix-reversed.
    pub fn commit(&self, target: &BitVector, fault: &BitVector) -> Result<(BitVector, BitVector)> {
        let tw = self.object.functions[&self.root].params[1][0]
            .ty
            .width(self.object.ptr_width)? as usize;
        if target.width() != tw || fault.width() != self.fault_width() {
            return err("§3.3", "commit width mismatch");
        }
        let mut target = Word(target.clone());
        let mut fault = Word(fault.clone());
        let at_halt = self.state.pc == self.program.halt;
        if at_halt && !self.state.faulted {
            target ^= self.state.result_val.clone();
        }
        if self.state.faulted {
            fault ^= self.state.code.clone();
        }
        if !at_halt {
            fault ^= 1u128;
        }
        Ok((
            target.0.resize(tw, false)?,
            fault.0.resize(self.fault_width(), false)?,
        ))
    }

    pub fn outcome(&self) -> OracleVectorOutcome {
        let tw = self.object.functions[&self.root].params[1][0]
            .ty
            .width(self.object.ptr_width)
            .expect("target width") as usize;
        let success = self.state.pc == self.program.halt && !self.state.faulted;
        OracleVectorOutcome {
            payload: if success {
                self.state.result_val.0.resize(tw, false).expect("target")
            } else {
                BitVector::zero(tw).expect("target")
            },
            code: if self.state.faulted {
                self.state
                    .code
                    .0
                    .resize(self.fault_width(), false)
                    .expect("code")
            } else {
                BitVector::from_u128(self.fault_width(), u128::from(!success)).expect("code")
            },
            steps: self.state.frames[..self.used]
                .iter()
                .filter(|f| f.old_pc != self.program.halt)
                .count() as u64,
        }
    }
    fn fault_width(&self) -> usize {
        self.object.functions[&self.root].params[2][0]
            .ty
            .width(self.object.ptr_width)
            .expect("fault width") as usize
    }

    pub fn forward_step(&mut self) -> Result<()> {
        if self.used == self.state.frames.len() {
            return err("§3.6", "forward step exceeds budget");
        }
        let mut frame = self.state.frames[self.used].clone();
        if frame
            != (Frame {
                slots: vec![Slot::default(); frame.slots.len()],
                ..Frame::default()
            })
        {
            return err("§3.6", "forward frame is not clean");
        }
        // Dispatch is read once before any row effects. On a basis tuple exactly
        // one comparator is true, so inactive controlled rows are identities.
        let old = self.state.pc;
        let successor = if old == self.program.halt {
            self.program.halt
        } else {
            let (f, i) = self.program.rows[old];
            let (flag, code) = self.flag(f, i)?;
            frame.flag ^= flag;
            if i.op == "lift.ret" {
                self.forward_ret(f, i, &mut frame)?;
            } else if i.op == "lift.call" {
                self.forward_call(f, i, old, &mut frame)?;
            } else {
                if matches!(i.op.as_str(), "qbranch" | "qjumpr") {
                    frame.succ ^= self.dynamic_successor(f, i)?;
                }
                if !flag {
                    let destinations = self.destinations(i)?;
                    let values = self.values(f, i)?;
                    for (j, v) in values.iter().enumerate() {
                        frame.slots[j].xor(v);
                    }
                    for (j, d) in destinations.iter().enumerate() {
                        self.exchange(f, d, &mut frame.slots[j])?;
                    }
                    self.direct(f, i, false)?;
                }
            }
            if flag {
                self.state.faulted ^= true;
                self.state.code ^= code;
            }
            self.successor(f, i, old, &frame)?
        };
        self.state.pc_prime ^= successor;
        std::mem::swap(&mut self.state.pc, &mut self.state.pc_prime);
        std::mem::swap(&mut self.state.pc_prime, &mut frame.old_pc);
        // Uncomputing dispatch from old_pc leaves no retained comparator bits.
        if self.state.pc_prime != 0 {
            return err("§3.6", "forward pc scratch did not clear");
        }
        self.state.frames[self.used] = frame;
        self.used += 1;
        Ok(())
    }
    pub fn inverse_step(&mut self) -> Result<()> {
        if self.used == 0 {
            return err("§3.6", "no forward step to invert");
        }
        let index = self.used - 1;
        let mut frame = self.state.frames[index].clone();
        let old = frame.old_pc;
        std::mem::swap(&mut self.state.pc_prime, &mut frame.old_pc);
        std::mem::swap(&mut self.state.pc, &mut self.state.pc_prime);
        let successor = if old == self.program.halt {
            self.program.halt
        } else {
            let (f, i) = self.program.rows[old];
            self.successor(f, i, old, &frame)?
        };
        self.state.pc_prime ^= successor;
        if self.state.pc_prime != 0 {
            return err("§3.6", "inverse successor scratch did not clear");
        }
        if old != self.program.halt {
            let (f, i) = self.program.rows[old];
            let code = self.fault_code(f, i)?;
            if frame.flag {
                self.state.faulted ^= true;
                self.state.code ^= code;
            }
            if i.op == "lift.call" {
                self.inverse_call(f, i, old, &mut frame)?;
            } else if i.op == "lift.ret" {
                self.inverse_ret(f, i, &mut frame)?;
            } else {
                if !frame.flag {
                    self.direct(f, i, true)?;
                    let destinations = self.destinations(i)?;
                    for (j, d) in destinations.iter().enumerate().rev() {
                        self.exchange(f, d, &mut frame.slots[j])?;
                    }
                    let values = self.values(f, i)?;
                    for (j, v) in values.iter().enumerate() {
                        frame.slots[j].xor(v);
                    }
                }
                if matches!(i.op.as_str(), "qbranch" | "qjumpr") {
                    frame.succ ^= self.dynamic_successor(f, i)?;
                }
            }
            frame.flag ^= self.flag(f, i)?.0;
        }
        if frame.old_pc != 0
            || frame.flag
            || frame.succ != 0
            || frame.slots.iter().any(|s| s != &Slot::default())
        {
            return err(
                "§3.6",
                format!("inverse frame is not clean at row {old}: {frame:?}"),
            );
        }
        self.state.frames[index] = frame;
        self.used -= 1;
        Ok(())
    }
    fn read(&self, f: &Function, n: &str) -> Result<Word> {
        let value = self
            .state
            .locations
            .get(&(f.name.clone(), n.into()))
            .cloned()
            .ok_or_else(|| Error(format!("§3.7: missing machine location {n} in @{}", f.name)))?;
        Ok(Word(value.0.resize(self.width(f, n)? as usize, false)?))
    }
    fn width(&self, f: &Function, n: &str) -> Result<u16> {
        f.locations()
            .into_iter()
            .chain(f.params[0].iter())
            .chain(f.params[4].iter())
            .find(|p| p.name == n)
            .ok_or_else(|| Error("§3.7: unknown machine operand width".into()))?
            .ty
            .width(self.object.ptr_width)
    }
    fn length(&self, f: &Function, n: &str) -> Result<u128> {
        self.prospective_length(f,n)?.ok_or_else(||Error("§3.6: unguarded length exceeds address domain".into()))
    }
    fn prospective_length(&self,f:&Function,n:&str)->Result<Option<u128>>{
        if integer(n) {
            n.parse().map(Some)
                .map_err(|_| Error("§3.3: invalid machine length".into()))
        } else {
            let value=self.read(f,n)?;
            Ok(value.number().ok())
        }
    }
    fn stack_read(&self, p: u128) -> Word {
        self.state.stack.get(&p).cloned().unwrap_or_default()
    }
    fn stack_set(&mut self, p: u128, v: Word) {
        if v == 0 {
            self.state.stack.remove(&p);
        } else {
            self.state.stack.insert(p, v);
        }
    }
    fn stack_xor(&mut self, p: u128, v: u128) {
        let mut value = self.stack_read(p);
        value ^= v;
        self.stack_set(p, value);
    }
    fn exchange_location(&mut self, f: &Function, n: &str, value: &mut Word) -> Result<()> {
        let width = self.width(f, n)? as usize;
        let value_width = value.0.width().max(width);
        value.0 = value.0.resize(value_width, false)?;
        let loc = self
            .state
            .locations
            .get_mut(&(f.name.clone(), n.into()))
            .ok_or_else(|| Error("§3.7: missing exchange location".into()))?;
        let mut incoming = BitVector::zero(width)?;
        for bit in 0..width {
            incoming.set(bit, value.0.bit(bit)?)?;
            value.0.set(bit, loc.0.bit(bit)?)?;
        }
        loc.0 = incoming;
        Ok(())
    }

    fn fault_code(&self, f: &Function, i: &Instruction) -> Result<Word> {
        if i.op == "lift.faultif" {
            return Ok(Word(BitVector::parse_unsigned_exact(
                self.fault_width(),
                &i.operands[1],
            )?));
        }
        let code = match i.op.as_str() {
            "lift.call" => 3,
            "lift.ret" | "qjumpr" => 4,
            "lift.alloc" => 2,
            "lift.free" | "lift.boundscheck" => 7,
            op if op.starts_with("lift.load.") || op.starts_with("lift.store.") => 7,
            op if op.strip_prefix("lift.").is_some_and(is_div) => {
                let a = self.read(f, &i.operands[0])?;
                let b = self.read(f, &i.operands[1])?;
                match a.0.binary(op.trim_start_matches("lift."), &b.0) {
                    Err(e) if e.0.contains("FAULT_UB_SDIV_OVERFLOW") => 6,
                    _ => 5,
                }
            }
            _ => 0,
        };
        Ok(Word::from(code))
    }

    fn flag(&self, f: &Function, i: &Instruction) -> Result<(bool, Word)> {
        let a = &i.operands;
        let flag = match i.op.as_str() {
            "lift.call" => {
                let c = &self.object.functions[a[0].trim_start_matches('@')];
                self.state.depth + 1 > self.budget.depth
                    || self
                        .state
                        .sp
                        .checked_add(c.frame_cells() as u128)
                        .is_none_or(|s| s > self.budget.stack)
            }
            "lift.ret" => self.state.depth == 0,
            "lift.alloc" => match self.prospective_length(f, &a[0])? {
                Some(n) => self.state.memory.allocation_fails(n, self.budget),
                None => true,
            },
            "lift.free" => {
                let p = self.read(f, &a[0])?.number()?;
                !self
                    .state
                    .memory
                    .allocations
                    .iter()
                    .any(|e| e.base == p && e.live && !e.pinned)
            }
            "lift.boundscheck" => match self.prospective_length(f, &a[1])? {
                Some(n) => !self.state.memory.valid(self.read(f, &a[0])?.number()?, n),
                None => true,
            },
            "lift.faultif" => self.read(f, &a[0])? != 0,
            "qjumpr" => {
                let (start, end) = self.program.ranges[&f.name];
                !self
                    .read(f, &a[0])?
                    .number()
                    .is_ok_and(|n| n >= start as u128 && n < end as u128)
            }
            op if op.starts_with("lift.load.i") || op.starts_with("lift.store.i") => {
                !self.state.memory.valid(
                    self.read(f, &a[0])?.number()?,
                    op.rsplit_once(".i")
                        .expect("suffix")
                        .1
                        .parse::<u128>()
                        .expect("width")
                        / 8,
                )
            }
            op if op.strip_prefix("lift.").is_some_and(is_div) => self
                .read(f, &a[0])?
                .0
                .binary(op.trim_start_matches("lift."), &self.read(f, &a[1])?.0)
                .is_err(),
            _ => false,
        };
        Ok((flag, self.fault_code(f, i)?))
    }

    fn dynamic_successor(&self, f: &Function, i: &Instruction) -> Result<usize> {
        if i.op == "qbranch" {
            self.program.label(
                &f.name,
                if self.read(f, &i.operands[0])? != 0 {
                    &i.operands[1]
                } else {
                    &i.operands[2]
                },
            )
        } else {
            let width = (usize::BITS - self.program.halt.leading_zeros()).max(1) as usize;
            Ok(self
                .read(f, &i.operands[0])?
                .0
                .resize(width, false)?
                .to_u128()? as usize)
        }
    }

    fn successor(&self, f: &Function, i: &Instruction, k: usize, frame: &Frame) -> Result<usize> {
        if frame.flag {
            return Ok(self.program.halt);
        }
        Ok(match i.op.as_str() {
            "lift.retval" | "fault" => self.program.halt,
            "qjump" => self.program.label(&f.name, &i.operands[0])?,
            "qbranch" | "qjumpr" | "lift.ret" => frame.succ,
            "lift.call" => self.program.ranges[i.operands[0].trim_start_matches('@')].0,
            _ => k + 1,
        })
    }
    fn destinations(&self, i: &Instruction) -> Result<Vec<Destination>> {
        Ok(match i.op.as_str() {
            "lift.alloc" => vec![
                Destination::Location(i.lhs[0].clone()),
                Destination::EntryAtop,
                Destination::Heap,
                Destination::Atop,
            ],
            op if op.starts_with("lift.store.i") => vec![Destination::Memory(
                i.operands[0].clone(),
                op.rsplit_once(".i")
                    .expect("suffix")
                    .1
                    .parse()
                    .expect("width"),
            )],
            _ => i
                .lhs
                .iter()
                .filter(|_| !matches!(i.op.as_str(), "lift.call" | "lift.ret"))
                .map(|n| Destination::Location(n.clone()))
                .collect(),
        })
    }
    fn values(&self, f: &Function, i: &Instruction) -> Result<Vec<Slot>> {
        let op = i.op.as_str();
        let a = &i.operands;
        let rd = |n: &str| self.read(f, n);
        let width = |n: &str| self.width(f, n);
        let values: Vec<Word> = if let Some(w) = op.strip_prefix("lift.const.i") {
            vec![Word(BitVector::parse_decimal(
                w.parse().expect("width"),
                &a[0],
            )?)]
        } else if let Some(binary) = op
            .strip_prefix("lift.")
            .filter(|s| is_binop(s) || is_div(s))
        {
            vec![Word(rd(&a[0])?.0.binary(binary, &rd(&a[1])?.0)?)]
        } else if let Some(pred) = op.strip_prefix("lift.icmp.") {
            let x = rd(&a[0])?;
            let y = rd(&a[1])?;
            let order = if pred.starts_with('s') {
                x.0.signed_cmp(&y.0)?
            } else {
                x.0.unsigned_cmp(&y.0)?
            };
            let bit = match pred {
                "eq" => order.is_eq(),
                "ne" => !order.is_eq(),
                "ult" | "slt" => order.is_lt(),
                "ule" | "sle" => !order.is_gt(),
                _ => return err("§4.8", "invalid comparison"),
            };
            vec![Word::from(u128::from(bit))]
        } else if let Some((cast, target)) = op
            .strip_prefix("lift.")
            .and_then(|s| s.split_once(".i"))
            .filter(|(c, _)| ["zext", "sext", "trunc"].contains(c))
        {
            vec![Word(
                rd(&a[0])?
                    .0
                    .resize(target.parse().expect("width"), cast == "sext")?,
            )]
        } else if let Some(w) = op.strip_prefix("lift.load.i") {
            vec![Word::from(
                self.state
                    .memory
                    .load(rd(&a[0])?.number()?, w.parse().expect("width")),
            )]
        } else if op.starts_with("lift.store.i") {
            vec![rd(&a[1])?]
        } else {
            match op {
                "lift.mov" => vec![rd(&a[0])?],
                "lift.select" => vec![rd(if rd(&a[0])? != 0 { &a[1] } else { &a[2] })?],
                "lift.parmov" => list(&a[0])?.iter().map(|s| rd(s)).collect::<Result<_>>()?,
                "lift.label" => vec![Word::from(self.program.label(&f.name, &a[0])? as u128)],
                "lift.addr" => vec![Word::from(
                    self.state.memory.addresses[a[0].trim_start_matches('@')],
                )],
                "lift.alloc" => {
                    let n = self.length(f, &a[0])?;
                    let base = self.state.memory.heap_base + self.state.memory.heap_used;
                    return Ok(vec![
                        Slot::word(base),
                        Slot::entry(Allocation {
                            base,
                            size: n,
                            live: true,
                            pinned: false,
                        }),
                        Slot::word(self.state.memory.heap_used + n.max(1)),
                        Slot::word(self.state.atop + 1),
                    ]);
                }
                _ => Vec::new(),
            }
        };
        values
            .into_iter()
            .enumerate()
            .map(|(j, v)| {
                Ok(Slot::word(if j < i.lhs.len() {
                    Word(v.0.resize(width(&i.lhs[j])? as usize, false)?)
                } else {
                    v
                }))
            })
            .collect()
    }

    fn exchange(&mut self, f: &Function, d: &Destination, s: &mut Slot) -> Result<()> {
        match d {
            Destination::Location(n) => self.exchange_location(f, n, &mut s.word)?,
            Destination::Memory(n, w) => {
                let p = self.read(f, n)?.number()?;
                let old = self.state.memory.load(p, *w);
                self.state.memory.store(p, *w, s.word.number()?);
                s.word = old.into();
            }
            Destination::Heap => s.word.exchange_counter(&mut self.state.memory.heap_used)?,
            Destination::Atop => s.word.exchange_counter(&mut self.state.atop)?,
            Destination::EntryAtop => {
                let index = usize::try_from(self.state.atop).map_err(|_| {
                    Error("§3.6: allocation index exceeds host address space".into())
                })?;
                let zero = Allocation {
                    base: 0,
                    size: 0,
                    live: false,
                    pinned: false,
                };
                let old = self
                    .state
                    .memory
                    .allocations
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| zero.clone());
                let new = s.allocation();
                if index == self.state.memory.allocations.len() {
                    self.state.memory.allocations.push(new);
                } else {
                    self.state.memory.allocations[index] = new;
                }
                while self.state.memory.allocations.last() == Some(&zero) {
                    self.state.memory.allocations.pop();
                }
                *s = Slot::entry(old);
            }
        }
        Ok(())
    }
    fn direct(&mut self, f: &Function, i: &Instruction, inverse: bool) -> Result<()> {
        match i.op.as_str() {
            "lift.free" => {
                let p = self.read(f, &i.operands[0])?.number()?;
                for e in &mut self.state.memory.allocations {
                    if e.base == p {
                        e.live ^= true;
                    }
                }
            }
            "lift.retval" => self.state.result_val ^= self.read(f, &i.operands[0])?,
            "fault" => {
                self.state.code ^= Word(BitVector::parse_decimal(
                    self.fault_width(),
                    &i.operands[0],
                )?);
                self.state.faulted ^= true;
            }
            "lift.oracle_call" | "lift.oracle_uncall" => self.nested_oracle(f, i)?,
            "lift.perm_call" => self.permutation_call(f, i, inverse)?,
            _ => {}
        }
        Ok(())
    }
    fn push_pop_locations(&mut self, f: &Function, base: u128) -> Result<()> {
        for (j, p) in f.locations().iter().enumerate() {
            let addr = base + 3 + j as u128;
            let mut cell = self.stack_read(addr);
            self.exchange_location(f, &p.name, &mut cell)?;
            self.stack_set(addr, cell);
        }
        Ok(())
    }
    fn forward_call(
        &mut self,
        f: &Function,
        i: &Instruction,
        k: usize,
        frame: &mut Frame,
    ) -> Result<()> {
        if frame.flag {
            return Ok(());
        }
        let callee = &self.object.functions[i.operands[0].trim_start_matches('@')];
        let args = list(&i.operands[1])?;
        let m = args.len();
        // C1 capture before moving ANY callee locations (including recursive args).
        for (j, a) in args.iter().enumerate() {
            frame.slots[j].word ^= self.read(f, a)?;
        }
        self.push_pop_locations(callee, self.state.sp)?; // C2
        self.stack_xor(self.state.sp, (k + 1) as u128);
        self.stack_xor(self.state.sp + 1, self.state.fp);
        let dest = (f.name.clone(), i.lhs[0].clone());
        let id = self
            .ids
            .iter()
            .position(|n| *n == dest)
            .ok_or_else(|| Error("§3.7: missing destination id".into()))?;
        self.stack_xor(self.state.sp + 2, id as u128); // C3
        for (j, p) in callee.params[0].iter().enumerate() {
            *self
                .state
                .locations
                .get_mut(&(callee.name.clone(), p.name.clone()))
                .expect("callee param") ^= frame.slots[j].word.clone();
        } // C4
        frame.slots[m].word ^= self.state.sp;
        frame.slots[m + 1].word ^= self.state.sp + callee.frame_cells() as u128;
        frame.slots[m + 2].word ^= self.state.depth + 1;
        frame.slots[m].word.exchange_counter(&mut self.state.fp)?;
        frame.slots[m + 1]
            .word
            .exchange_counter(&mut self.state.sp)?;
        frame.slots[m + 2]
            .word
            .exchange_counter(&mut self.state.depth)?;
        Ok(())
    }
    fn inverse_call(
        &mut self,
        f: &Function,
        i: &Instruction,
        k: usize,
        frame: &mut Frame,
    ) -> Result<()> {
        if frame.flag {
            return Ok(());
        }
        let callee = &self.object.functions[i.operands[0].trim_start_matches('@')];
        let args = list(&i.operands[1])?;
        let m = args.len();
        frame.slots[m + 2]
            .word
            .exchange_counter(&mut self.state.depth)?;
        frame.slots[m + 1]
            .word
            .exchange_counter(&mut self.state.sp)?;
        frame.slots[m].word.exchange_counter(&mut self.state.fp)?;
        frame.slots[m + 2].word ^= self.state.depth + 1;
        frame.slots[m + 1].word ^= self.state.sp + callee.frame_cells() as u128;
        frame.slots[m].word ^= self.state.sp;
        for (j, p) in callee.params[0].iter().enumerate() {
            *self
                .state
                .locations
                .get_mut(&(callee.name.clone(), p.name.clone()))
                .expect("callee param") ^= frame.slots[j].word.clone();
        }
        let id = self
            .ids
            .iter()
            .position(|n| *n == (f.name.clone(), i.lhs[0].clone()))
            .expect("destination id");
        self.stack_xor(self.state.sp + 2, id as u128);
        self.stack_xor(self.state.sp + 1, self.state.fp);
        self.stack_xor(self.state.sp, (k + 1) as u128);
        self.push_pop_locations(callee, self.state.sp)?;
        for (j, a) in args.iter().enumerate() {
            frame.slots[j].word ^= self.read(f, a)?;
        }
        Ok(())
    }
    fn deliver(&mut self, slot: &mut Slot) -> Result<()> {
        let id = self.stack_read(self.state.fp + 2).number()?;
        let (f, n) = self
            .ids
            .get(usize::try_from(id).map_err(|_| Error("§3.7: invalid delivery id".into()))?)
            .ok_or_else(|| Error("§3.7: invalid delivery location".into()))?
            .clone();
        self.exchange_location(&self.object.functions[&f], &n, &mut slot.word)
    }
    fn forward_ret(&mut self, f: &Function, i: &Instruction, frame: &mut Frame) -> Result<()> {
        if frame.flag {
            return Ok(());
        }
        frame.succ ^= self.stack_read(self.state.fp).number()? as usize;
        frame.slots[0].word ^= self.read(f, &i.operands[0])?;
        self.push_pop_locations(f, self.state.fp)?;
        self.deliver(&mut frame.slots[0])?;
        frame.slots[1].word ^= self.stack_read(self.state.fp + 1);
        frame.slots[2].word ^= self.state.depth - 1;
        frame.slots[1].word.exchange_counter(&mut self.state.fp)?;
        frame.slots[2]
            .word
            .exchange_counter(&mut self.state.depth)?;
        Ok(())
    }
    fn inverse_ret(&mut self, f: &Function, i: &Instruction, frame: &mut Frame) -> Result<()> {
        if frame.flag {
            return Ok(());
        }
        frame.slots[2]
            .word
            .exchange_counter(&mut self.state.depth)?;
        frame.slots[1].word.exchange_counter(&mut self.state.fp)?;
        frame.slots[2].word ^= self.state.depth - 1;
        frame.slots[1].word ^= self.stack_read(self.state.fp + 1);
        self.deliver(&mut frame.slots[0])?;
        self.push_pop_locations(f, self.state.fp)?;
        frame.slots[0].word ^= self.read(f, &i.operands[0])?;
        frame.succ ^= self.stack_read(self.state.fp).number()? as usize;
        Ok(())
    }
    fn nested_oracle(&mut self, f: &Function, i: &Instruction) -> Result<()> {
        let a = &i.operands;
        let callee = &self.object.functions[a[0].trim_start_matches('@')];
        let root = &self.object.functions[&self.root];
        let arguments: BTreeMap<_, _> = root.params[4]
            .iter()
            .zip(&self.input.args)
            .map(|(p, a)| (p.name.clone(), a))
            .collect();
        let Some(Argument::Token(budget)) = arguments.get(&a[1]).copied() else {
            return err("§4.2", "nested machine oracle needs token");
        };
        let mut input = OracleVectorInput {
            ins: list(&a[2])?
                .iter()
                .map(|n| self.read(f, n).map(|v| v.0))
                .collect::<Result<_>>()?,
            ..Default::default()
        };
        for (p, n) in callee.params[3].iter().zip(list(&a[5])?) {
            let base = self.state.memory.addresses[&n];
            let Type::Region(aw, _) = p.ty else {
                unreachable!("region")
            };
            let len = usize::try_from(1u128 << aw)
                .map_err(|_| Error("§7.2: nested region too large".into()))?;
            input.regions.insert(
                p.name.clone(),
                (0..len)
                    .map(|j| self.state.memory.byte(base + j as u128))
                    .collect(),
            );
        }
        for n in list(&a[6])? {
            input.args.push(match arguments.get(&n) {
                Some(v) => (*v).clone(),
                None => Argument::Bits(self.read(f, &n)?.number()?),
            });
        }
        // A nested invocation is itself a complete compute/commit/reverse
        // permutation. Use the reversible executor recursively as well.
        let mut child = Self::new(self.object, &callee.name, &input, *budget)?;
        for _ in 0..budget.steps {
            child.forward_step()?;
        }
        let outcome = child.outcome();
        for _ in 0..budget.steps {
            child.inverse_step()?;
        }
        if !child.frames_clean() {
            return err("§5.2", "nested oracle left dirty frames");
        }
        *self
            .state
            .locations
            .get_mut(&(f.name.clone(), a[3].clone()))
            .expect("target location") ^= Word(outcome.payload);
        *self
            .state
            .locations
            .get_mut(&(f.name.clone(), a[4].clone()))
            .expect("fault location") ^= Word(outcome.code);
        Ok(())
    }
    fn permutation_call(&mut self, f: &Function, i: &Instruction, inverse: bool) -> Result<()> {
        let a = &i.operands;
        let callee = &self.object.functions[a[0].trim_start_matches('@')];
        let names = list(&a[1])?;
        let mutable = names
            .iter()
            .map(|n| self.read(f, n).map(|v| v.0))
            .collect::<Result<Vec<_>>>()?;
        let inputs = list(&a[2])?
            .iter()
            .map(|n| self.read(f, n).map(|v| v.0))
            .collect::<Result<Vec<_>>>()?;
        let mut regions = BTreeMap::new();
        for (p, n) in callee.params[2].iter().zip(list(&a[3])?) {
            let base = self.state.memory.addresses[&n];
            let Type::Region(aw, _) = p.ty else {
                unreachable!("region")
            };
            let count = usize::try_from(1u128 << aw)
                .map_err(|_| Error("§7.2: action region too large".into()))?;
            regions.insert(
                p.name.clone(),
                (0..count)
                    .map(|j| self.state.memory.byte(base + j as u128))
                    .collect(),
            );
        }
        let root = &self.object.functions[&self.root];
        let arguments: BTreeMap<_, _> = root.params[4]
            .iter()
            .zip(&self.input.args)
            .map(|(p, a)| (p.name.clone(), a))
            .collect();
        let args = list(&a[4])?
            .iter()
            .map(|n| match arguments.get(n) {
                Some(v) => Ok((*v).clone()),
                None => Ok(Argument::Bits(self.read(f, n)?.number()?)),
            })
            .collect::<Result<Vec<_>>>()?;
        let values = self.object.permutation_action_vectors(
            &callee.name,
            &mutable,
            &inputs,
            &regions,
            &args,
            inverse,
        )?;
        for (n, v) in names.iter().zip(values) {
            *self
                .state
                .locations
                .get_mut(&(f.name.clone(), n.clone()))
                .expect("action output") = Word(v);
        }
        Ok(())
    }
}

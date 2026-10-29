//! Literal §3.6/§3.7 basis machine. Frames contain displaced/captured values;
//! reversal uses inverse operations, never a saved pre-step state.
use crate::{
    err,
    object::{integer, is_binop, is_div, list, mask, parse_bits},
    oracle::{binop, compare, divide, signed, Allocation, Memory, Program},
    Argument, Budget, Error, Function, Instruction, Object, OracleInput, OracleOutcome, Result,
    Type,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
struct Slot {
    word: u128,
    size: u128,
    live: bool,
    pinned: bool,
}
impl Slot {
    fn word(word: u128) -> Self {
        Self {
            word,
            ..Self::default()
        }
    }
    fn entry(e: Allocation) -> Self {
        Self {
            word: e.base,
            size: e.size,
            live: e.live,
            pinned: e.pinned,
        }
    }
    fn allocation(&self) -> Allocation {
        Allocation {
            base: self.word,
            size: self.size,
            live: self.live,
            pinned: self.pinned,
        }
    }
    fn xor(&mut self, other: &Self) {
        self.word ^= other.word;
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
/// Immutable program text, input bindings and budget are held by OracleMachine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BasisConfiguration {
    pub locations: BTreeMap<(String, String), u128>,
    pub stack: BTreeMap<u128, u128>,
    pub pc: usize,
    pub pc_prime: usize,
    pub faulted: bool,
    pub code: u128,
    pub result_val: u128,
    pub sp: u128,
    pub fp: u128,
    pub depth: u128,
    pub atop: u128,
    memory: Memory,
    frames: Vec<Frame>,
}
pub struct OracleMachine<'a> {
    object: &'a Object,
    root: String,
    input: OracleInput,
    budget: Budget,
    program: Program<'a>,
    state: BasisConfiguration,
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

impl<'a> OracleMachine<'a> {
    pub fn new(
        object: &'a Object,
        name: &str,
        input: &OracleInput,
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
                    entry.insert(0);
                }
            }
        }
        for (p, v) in root.params[0].iter().zip(&input.ins) {
            if *v & !mask(p.ty.width(object.ptr_width)?) != 0 {
                return err("§3.3", "machine input exceeds width");
            }
            locations.insert((name.into(), p.name.clone()), *v);
        }
        for (p, v) in root.params[4].iter().zip(&input.args) {
            match (&p.ty, v) {
                (Type::Int(w), Argument::Bits(v)) if *v & !mask(*w) == 0 => {
                    locations.insert((name.into(), p.name.clone()), *v);
                }
                (Type::Ptr, Argument::Bits(v)) if *v & !mask(object.ptr_width) == 0 => {
                    locations.insert((name.into(), p.name.clone()), *v);
                }
                (Type::Token, Argument::Token(b)) => object.validate_budget(*b)?,
                (Type::F64, Argument::Float(x)) if x.is_finite() => {}
                _ => return err("§3.3", "machine classical argument type mismatch"),
            }
        }
        let memory = Memory::initial(object, root, input)?;
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
            state: BasisConfiguration {
                locations,
                stack: BTreeMap::new(),
                pc: 0,
                pc_prime: 0,
                faulted: false,
                code: 0,
                result_val: 0,
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
    pub fn snapshot(&self) -> BasisConfiguration {
        self.state.clone()
    }
    pub fn frames_clean(&self) -> bool {
        self.state.frames.iter().all(|f| {
            f.old_pc == 0 && !f.flag && f.succ == 0 && f.slots.iter().all(|s| s == &Slot::default())
        })
    }
    pub fn outcome(&self) -> OracleOutcome {
        let code = if self.state.faulted {
            self.state.code
        } else if self.state.pc != self.program.halt {
            1
        } else {
            0
        };
        OracleOutcome {
            payload: if self.state.pc == self.program.halt && !self.state.faulted {
                self.state.result_val
            } else {
                0
            },
            code,
            steps: self.state.frames[..self.used]
                .iter()
                .filter(|f| f.old_pc != self.program.halt)
                .count() as u64,
        }
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
    fn read(&self, f: &Function, n: &str) -> Result<u128> {
        self.state
            .locations
            .get(&(f.name.clone(), n.into()))
            .copied()
            .ok_or_else(|| Error(format!("§3.7: missing machine location {n} in @{}", f.name)))
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
        if integer(n) {
            n.parse()
                .map_err(|_| Error("§3.3: invalid machine length".into()))
        } else {
            self.read(f, n)
        }
    }
    fn stack_read(&self, p: u128) -> u128 {
        self.state.stack.get(&p).copied().unwrap_or(0)
    }
    fn stack_set(&mut self, p: u128, v: u128) {
        if v == 0 {
            self.state.stack.remove(&p);
        } else {
            self.state.stack.insert(p, v);
        }
    }
    fn stack_xor(&mut self, p: u128, v: u128) {
        let value = self.stack_read(p) ^ v;
        self.stack_set(p, value);
    }
    fn exchange_location(&mut self, f: &Function, n: &str, value: &mut u128) -> Result<()> {
        let w = self.width(f, n)?;
        let mask = mask(w);
        let loc = self
            .state
            .locations
            .get_mut(&(f.name.clone(), n.into()))
            .ok_or_else(|| Error("§3.7: missing exchange location".into()))?;
        let low = *value & mask;
        *value = (*value & !mask) | *loc;
        *loc = low;
        Ok(())
    }
    fn fault_code(&self, f: &Function, i: &Instruction) -> Result<u128> {
        Ok(match i.op.as_str() {
            "lift.call" => 3,
            "lift.ret" | "qjumpr" => 4,
            "lift.alloc" => 2,
            "lift.free" | "lift.boundscheck" => 7,
            "lift.faultif" => i.operands[1].parse().expect("validated fault code"),
            op if op.starts_with("lift.load.") || op.starts_with("lift.store.") => 7,
            op if op.strip_prefix("lift.").is_some_and(is_div) => {
                let a = self.read(f, &i.operands[0])?;
                let b = self.read(f, &i.operands[1])?;
                if b == 0 {
                    5
                } else if op.starts_with("lift.s")
                    && a == 1u128 << (self.width(f, &i.operands[0])? - 1)
                    && b == mask(self.width(f, &i.operands[0])?)
                {
                    6
                } else {
                    5
                }
            }
            _ => 0,
        })
    }
    fn flag(&self, f: &Function, i: &Instruction) -> Result<(bool, u128)> {
        let a = &i.operands;
        let flag =
            match i.op.as_str() {
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
                "lift.alloc" => self
                    .state
                    .memory
                    .allocation_fails(self.length(f, &a[0])?, self.budget),
                "lift.free" => !self.state.memory.allocations.iter().any(|e| {
                    e.base == self.read(f, &a[0]).unwrap_or(u128::MAX) && e.live && !e.pinned
                }),
                "lift.boundscheck" => !self
                    .state
                    .memory
                    .valid(self.read(f, &a[0])?, self.length(f, &a[1])?),
                "lift.faultif" => self.read(f, &a[0])? != 0,
                "qjumpr" => {
                    let v = self.read(f, &a[0])?;
                    let (start, end) = self.program.ranges[&f.name];
                    v < start as u128 || v >= end as u128
                }
                op if op.starts_with("lift.load.i") || op.starts_with("lift.store.i") => {
                    !self.state.memory.valid(
                        self.read(f, &a[0])?,
                        op.rsplit_once(".i")
                            .expect("memory suffix")
                            .1
                            .parse::<u128>()
                            .expect("validated width")
                            / 8,
                    )
                }
                op if op.strip_prefix("lift.").is_some_and(is_div) => divide(
                    op.trim_start_matches("lift."),
                    self.read(f, &a[0])?,
                    self.read(f, &a[1])?,
                    self.width(f, &a[0])?,
                )
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
            let w = (usize::BITS - self.program.halt.leading_zeros()).max(1);
            Ok((self.read(f, &i.operands[0])? & mask(w as u16)) as usize)
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
        let values = if let Some(w) = op.strip_prefix("lift.const.i") {
            vec![parse_bits(&a[0], w.parse().expect("width"))?]
        } else if let Some(bin) = op.strip_prefix("lift.").filter(|s| is_binop(s)) {
            vec![binop(bin, rd(&a[0])?, rd(&a[1])?, width(&a[0])?)]
        } else if let Some(div) = op.strip_prefix("lift.").filter(|s| is_div(s)) {
            vec![divide(div, rd(&a[0])?, rd(&a[1])?, width(&a[0])?)
                .map_err(|c| Error(format!("§3.6: unguarded division fault {c}")))?]
        } else if let Some(pred) = op.strip_prefix("lift.icmp.") {
            vec![compare(pred, rd(&a[0])?, rd(&a[1])?, width(&a[0])?) as u128]
        } else if let Some((cast, _)) = op
            .strip_prefix("lift.")
            .and_then(|s| s.split_once(".i"))
            .filter(|(c, _)| ["zext", "sext", "trunc"].contains(c))
        {
            vec![if cast == "sext" {
                signed(rd(&a[0])?, width(&a[0])?) as u128
            } else {
                rd(&a[0])?
            }]
        } else if let Some(w) = op.strip_prefix("lift.load.i") {
            vec![self
                .state
                .memory
                .load(rd(&a[0])?, w.parse().expect("width"))]
        } else if op.starts_with("lift.store.i") {
            vec![rd(&a[1])?]
        } else {
            match op {
                "lift.mov" => vec![rd(&a[0])?],
                "lift.select" => vec![rd(if rd(&a[0])? != 0 { &a[1] } else { &a[2] })?],
                "lift.parmov" => list(&a[0])?.iter().map(|s| rd(s)).collect::<Result<_>>()?,
                "lift.label" => vec![self.program.label(&f.name, &a[0])? as u128],
                "lift.addr" => vec![self.state.memory.addresses[a[0].trim_start_matches('@')]],
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
        Ok(values
            .into_iter()
            .enumerate()
            .map(|(j, v)| {
                Slot::word(if j < i.lhs.len() {
                    v & mask(width(&i.lhs[j]).expect("validated assignment width"))
                } else {
                    v
                })
            })
            .collect())
    }
    fn exchange(&mut self, f: &Function, d: &Destination, s: &mut Slot) -> Result<()> {
        match d {
            Destination::Location(n) => self.exchange_location(f, n, &mut s.word)?,
            Destination::Memory(n, w) => {
                let p = self.read(f, n)?;
                let old = self.state.memory.load(p, *w);
                self.state.memory.store(p, *w, s.word);
                s.word = old;
            }
            Destination::Heap => std::mem::swap(&mut self.state.memory.heap_used, &mut s.word),
            Destination::Atop => std::mem::swap(&mut self.state.atop, &mut s.word),
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
                let p = self.read(f, &i.operands[0])?;
                for e in &mut self.state.memory.allocations {
                    if e.base == p {
                        e.live ^= true;
                    }
                }
            }
            "lift.retval" => self.state.result_val ^= self.read(f, &i.operands[0])?,
            "fault" => {
                self.state.code ^= i.operands[0].parse::<u128>().expect("fault code");
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
                .expect("callee param") ^= frame.slots[j].word;
        } // C4
        frame.slots[m].word ^= self.state.sp;
        frame.slots[m + 1].word ^= self.state.sp + callee.frame_cells() as u128;
        frame.slots[m + 2].word ^= self.state.depth + 1;
        std::mem::swap(&mut frame.slots[m].word, &mut self.state.fp);
        std::mem::swap(&mut frame.slots[m + 1].word, &mut self.state.sp);
        std::mem::swap(&mut frame.slots[m + 2].word, &mut self.state.depth);
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
        std::mem::swap(&mut frame.slots[m + 2].word, &mut self.state.depth);
        std::mem::swap(&mut frame.slots[m + 1].word, &mut self.state.sp);
        std::mem::swap(&mut frame.slots[m].word, &mut self.state.fp);
        frame.slots[m + 2].word ^= self.state.depth + 1;
        frame.slots[m + 1].word ^= self.state.sp + callee.frame_cells() as u128;
        frame.slots[m].word ^= self.state.sp;
        for (j, p) in callee.params[0].iter().enumerate() {
            *self
                .state
                .locations
                .get_mut(&(callee.name.clone(), p.name.clone()))
                .expect("callee param") ^= frame.slots[j].word;
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
        let id = self.stack_read(self.state.fp + 2);
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
        frame.succ ^= self.stack_read(self.state.fp) as usize;
        frame.slots[0].word ^= self.read(f, &i.operands[0])?;
        self.push_pop_locations(f, self.state.fp)?;
        self.deliver(&mut frame.slots[0])?;
        frame.slots[1].word ^= self.stack_read(self.state.fp + 1);
        frame.slots[2].word ^= self.state.depth - 1;
        std::mem::swap(&mut frame.slots[1].word, &mut self.state.fp);
        std::mem::swap(&mut frame.slots[2].word, &mut self.state.depth);
        Ok(())
    }
    fn inverse_ret(&mut self, f: &Function, i: &Instruction, frame: &mut Frame) -> Result<()> {
        if frame.flag {
            return Ok(());
        }
        std::mem::swap(&mut frame.slots[2].word, &mut self.state.depth);
        std::mem::swap(&mut frame.slots[1].word, &mut self.state.fp);
        frame.slots[2].word ^= self.state.depth - 1;
        frame.slots[1].word ^= self.stack_read(self.state.fp + 1);
        self.deliver(&mut frame.slots[0])?;
        self.push_pop_locations(f, self.state.fp)?;
        frame.slots[0].word ^= self.read(f, &i.operands[0])?;
        frame.succ ^= self.stack_read(self.state.fp) as usize;
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
        let mut input = OracleInput::bits(
            &list(&a[2])?
                .iter()
                .map(|n| self.read(f, n))
                .collect::<Result<Vec<_>>>()?,
        );
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
                None => Argument::Bits(self.read(f, &n)?),
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
            .expect("target location") ^= outcome.payload;
        *self
            .state
            .locations
            .get_mut(&(f.name.clone(), a[4].clone()))
            .expect("fault location") ^= outcome.code;
        Ok(())
    }
    fn permutation_call(&mut self, f: &Function, i: &Instruction, inverse: bool) -> Result<()> {
        let a = &i.operands;
        let callee = &self.object.functions[a[0].trim_start_matches('@')];
        let names = list(&a[1])?;
        let mutable = names
            .iter()
            .map(|n| self.read(f, n))
            .collect::<Result<Vec<_>>>()?;
        let inputs = list(&a[2])?
            .iter()
            .map(|n| self.read(f, n))
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
                None => Ok(Argument::Bits(self.read(f, n)?)),
            })
            .collect::<Result<Vec<_>>>()?;
        let values = self.object.permutation_action_values(
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
                .expect("action output") = v;
        }
        Ok(())
    }
}

//! The literal §3.6/§3.7 predicated step machine on basis configurations.
//!
//! Frames hold displaced and captured values; the inverse step uses inverse
//! operations only, never a saved pre-step state (snapshots exist solely as an
//! assertion surface for tests). Every location, cell, slot and counter is a
//! bit-vector of its §3.7 derived width: `heap_used : wire[Hc]` with Hc = H + 2,
//! `atop : wire[Ac]` with Ac = A_tab + 1, `sp, fp : wire[Sc]` with
//! Sc = S_cells + 1, `depth : wire[Dc]` with Dc = max(1, ceil(log2(depth_limit + 2))),
//! stack cells of W_cell = max(P, Sc, Did, W_max) bits and slot j of the widest
//! value any row writes into it (SPEC-QUESTIONS 4).
//!
//! Dispatch is executed extensionally: on a basis tuple exactly one `at_k`
//! holds, so the rows controlled on the other comparators are identities and
//! the machine runs only `I[pc]` (permitted by §3.6, "executors MAY re-index
//! dispatch extensionally").
use crate::{
    err,
    object::{integer, is_binop, is_div, list},
    oracle::{
        literal_u128, operand_width, power_of_two, region_bytes, Allocation, Memory, Program,
    },
    Argument, BitVector, Budget, Error, Function, Instruction, Object, OracleInput, OracleOutcome,
    OracleVectorInput, OracleVectorOutcome, Result, Type,
};
use std::collections::BTreeMap;

/// The derived widths of §3.6/§3.7 for one oracle invocation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DerivedWidths {
    /// 2^P > n (the index-space size).
    pub p: usize,
    pub hc: usize,
    pub ac: usize,
    pub sc: usize,
    pub dc: usize,
    /// Bits of a global location id.
    pub did: usize,
    /// Widest location width in the closure.
    pub w_max: usize,
    pub w_cell: usize,
    /// Width of each frame slot disp_1..disp_{k_max}.
    pub slots: Vec<usize>,
}
fn bits_for(n: u128) -> usize {
    (128 - n.leading_zeros() as usize).max(1)
}
impl DerivedWidths {
    pub fn derive(object: &Object, root: &str, budget: Budget) -> Result<Self> {
        let program = Program::new(object, root)?;
        Self::from_program(object, &program, budget)
    }
    fn from_program(object: &Object, program: &Program, budget: Budget) -> Result<Self> {
        let layout = object.section("layout");
        let exponent = |key: &str| -> Result<usize> {
            match layout {
                Some(l) => usize::try_from(l.get(key)?.uint()?)
                    .map_err(|_| Error(format!("§3.7: host limit: layout {key} exceeds usize"))),
                None => Ok(0),
            }
        };
        let (h, a, s) = (exponent("H")?, exponent("A_tab")?, exponent("S_cells")?);
        let depth_plus_two = budget
            .depth
            .checked_add(2)
            .ok_or_else(|| Error("§5.4: depth limit exceeds the counter range".into()))?;
        // ceil(log2(d + 2)) = bits(d + 1).
        let dc = bits_for(depth_plus_two - 1);
        let p = bits_for(program.halt as u128);
        let mut seen = std::collections::BTreeSet::new();
        let mut ids = 0u128;
        let mut w_max = 1usize;
        for (f, _) in &program.rows {
            if seen.insert(f.name.clone()) {
                for loc in f.locations() {
                    ids += 1;
                    w_max = w_max.max(loc.ty.bits(object.ptr_width)?);
                }
            }
        }
        let did = bits_for(ids.saturating_sub(1));
        // Legal exponents need not have host-representable counter widths.
        // Report a representation limit here, never reject the object schema.
        let add_width = |n: usize, extra: usize, label: &str| {
            n.checked_add(extra).ok_or_else(|| {
                Error(format!(
                    "§3.7: host limit: derived {label} width exceeds usize"
                ))
            })
        };
        let sc = add_width(s, 1, "Sc = S_cells + 1")?;
        let mut widths = Self {
            p,
            hc: add_width(h, 2, "Hc = H + 2")?,
            ac: add_width(a, 1, "Ac = A_tab + 1")?,
            sc,
            dc,
            did,
            w_max,
            w_cell: p.max(sc).max(did).max(w_max),
            slots: Vec::new(),
        };
        for (f, i) in &program.rows {
            let row = widths.row_slots(object, f, i)?;
            for (j, w) in row.into_iter().enumerate() {
                if j == widths.slots.len() {
                    widths.slots.push(w);
                } else {
                    widths.slots[j] = widths.slots[j].max(w);
                }
            }
        }
        Ok(widths)
    }
    /// §4.2 slot use of one row, in slot order.
    fn row_slots(&self, object: &Object, f: &Function, i: &Instruction) -> Result<Vec<usize>> {
        let ptr = object.ptr_width as usize;
        let width = |n: &str| -> Result<usize> { Ok(operand_width(object, f, n)? as usize) };
        Ok(match i.op.as_str() {
            "lift.call" => {
                let callee = &object.functions[i.operands[0].trim_start_matches('@')];
                let mut v = callee.params[0]
                    .iter()
                    .map(|p| p.ty.bits(object.ptr_width))
                    .collect::<Result<Vec<_>>>()?;
                v.extend([self.sc, self.sc, self.dc]);
                v
            }
            "lift.ret" => vec![width(&i.operands[0])?, self.sc, self.dc],
            "lift.alloc" => vec![ptr, ptr, self.hc, self.ac],
            op if op.starts_with("lift.store.i") => vec![width(&i.operands[1])?],
            "lift.parmov" => i.lhs.iter().map(|d| width(d)).collect::<Result<_>>()?,
            op if !i.lhs.is_empty() && op.starts_with("lift.") => vec![width(&i.lhs[0])?],
            _ => Vec::new(),
        })
    }
}

/// One frame slot: a `wire[W_j]` word, plus the (size, live, pinned) fields an
/// `alloctab` entry occupies when the slot carries one (lift.alloc's slot 2).
#[derive(Clone, Debug, PartialEq, Eq)]
struct Slot {
    word: BitVector,
    size: u128,
    live: bool,
    pinned: bool,
}
impl Slot {
    fn clean(width: usize) -> Result<Self> {
        Ok(Self {
            word: BitVector::zero(width)?,
            size: 0,
            live: false,
            pinned: false,
        })
    }
    fn is_clean(&self) -> bool {
        self.word.is_zero() && self.size == 0 && !self.live && !self.pinned
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Frame {
    old_pc: usize,
    slots: Vec<Slot>,
    flag: bool,
    succ: usize,
}
impl Frame {
    fn is_clean(&self) -> bool {
        self.old_pc == 0 && !self.flag && self.succ == 0 && self.slots.iter().all(Slot::is_clean)
    }
}

/// The complete mutable basis tuple: every location, cell, counter, frame and
/// the step state of §3.2. Program text, inputs and budget are held by the machine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BasisVectorConfiguration {
    locations: BTreeMap<(String, String), BitVector>,
    stack: BTreeMap<u128, BitVector>,
    pub pc: usize,
    pub pc_prime: usize,
    pub faulted: bool,
    code: BitVector,
    result_val: BitVector,
    heap_used: BitVector,
    atop: BitVector,
    sp: BitVector,
    fp: BitVector,
    depth: BitVector,
    memory: Memory,
    frames: Vec<Frame>,
}
pub type BasisConfiguration = BasisVectorConfiguration;
impl BasisVectorConfiguration {
    /// The five §3.7 counters (heap_used, atop, sp, fp, depth) at their
    /// derived widths, without a u128 narrowing.
    pub fn counter_values(&self) -> [BitVector; 5] {
        [
            self.heap_used.clone(),
            self.atop.clone(),
            self.sp.clone(),
            self.fp.clone(),
            self.depth.clone(),
        ]
    }
    /// Every location of the tuple, keyed (function, name), in sorted order.
    pub fn locations(&self) -> Vec<((String, String), BitVector)> {
        self.locations
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
    /// A description of the first component in which `other` differs from
    /// `self` (locations, stack, step state, counters, memory, frames), or
    /// None when the complete §3.2 tuples are equal.
    pub fn first_difference(&self, other: &Self) -> Option<String> {
        let keys: std::collections::BTreeSet<&(String, String)> = self
            .locations
            .keys()
            .chain(other.locations.keys())
            .collect();
        for key in keys {
            if self.locations.get(key) != other.locations.get(key) {
                return Some(format!(
                    "location {}.{}: {} differs from {}",
                    key.0,
                    key.1,
                    self.locations
                        .get(key)
                        .map(BitVector::to_decimal)
                        .unwrap_or_else(|| "<absent>".into()),
                    other
                        .locations
                        .get(key)
                        .map(BitVector::to_decimal)
                        .unwrap_or_else(|| "<absent>".into())
                ));
            }
        }
        let cells: std::collections::BTreeSet<u128> = self
            .stack
            .keys()
            .chain(other.stack.keys())
            .copied()
            .collect();
        for cell in cells {
            if self.stack.get(&cell) != other.stack.get(&cell) {
                return Some(format!("stack cell {cell} differs"));
            }
        }
        if self.pc != other.pc || self.pc_prime != other.pc_prime || self.faulted != other.faulted {
            return Some(format!(
                "step state: pc {} / pc' {} / faulted {} differs from pc {} / pc' {} / faulted {}",
                self.pc, self.pc_prime, self.faulted, other.pc, other.pc_prime, other.faulted
            ));
        }
        for (name, a, b) in [
            ("code", &self.code, &other.code),
            ("result_val", &self.result_val, &other.result_val),
            ("heap_used", &self.heap_used, &other.heap_used),
            ("atop", &self.atop, &other.atop),
            ("sp", &self.sp, &other.sp),
            ("fp", &self.fp, &other.fp),
            ("depth", &self.depth, &other.depth),
        ] {
            if a != b {
                return Some(format!(
                    "counter {name}: {} differs from {}",
                    a.to_decimal(),
                    b.to_decimal()
                ));
            }
        }
        if self.memory != other.memory {
            let addresses: std::collections::BTreeSet<u128> = self
                .memory
                .bytes
                .keys()
                .chain(other.memory.bytes.keys())
                .copied()
                .collect();
            for address in addresses {
                if self.memory.byte(address) != other.memory.byte(address) {
                    return Some(format!(
                        "memory byte at address {address}: {} differs from {}",
                        self.memory.byte(address),
                        other.memory.byte(address)
                    ));
                }
            }
            return Some("memory allocation state differs".to_string());
        }
        for (i, (a, b)) in self.frames.iter().zip(&other.frames).enumerate() {
            if a != b {
                return Some(format!(
                    "frame {i}: old_pc {} / flag {} / succ {} differs from old_pc {} / flag {} / succ {}",
                    a.old_pc, a.flag, a.succ, b.old_pc, b.flag, b.succ
                ));
            }
        }
        if self.frames.len() != other.frames.len() {
            return Some(format!(
                "frame count: {} differs from {}",
                self.frames.len(),
                other.frames.len()
            ));
        }
        None
    }
    pub fn counters(&self) -> Result<[u128; 5]> {
        Ok([
            self.heap_used.to_u128()?,
            self.atop.to_u128()?,
            self.sp.to_u128()?,
            self.fp.to_u128()?,
            self.depth.to_u128()?,
        ])
    }
}

pub struct OracleVectorMachine<'a> {
    object: &'a Object,
    arguments: BTreeMap<String, Argument>,
    budget: Budget,
    program: Program<'a>,
    widths: DerivedWidths,
    state: BasisVectorConfiguration,
    used: usize,
    ids: Vec<(String, String)>,
}
#[derive(Clone)]
enum Destination {
    Location(String),
    Memory(String, u64),
    EntryAtop,
    Heap,
    Atop,
}

fn counter(width: usize, value: u128) -> Result<BitVector> {
    BitVector::from_u128_exact(width, value)
}
fn number(v: &BitVector) -> Result<u128> {
    v.to_u128()
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
        let widths = DerivedWidths::from_program(object, &program, budget)?;
        let root = &object.functions[name];
        if root.params[0].len() != input.ins.len() || root.params[4].len() != input.args.len() {
            return err("§5.1", "machine interface argument count mismatch");
        }
        let mut locations = BTreeMap::new();
        let mut ids = Vec::new();
        for (f, _) in &program.rows {
            for p in f.locations() {
                let key = (f.name.clone(), p.name.clone());
                if let std::collections::btree_map::Entry::Vacant(entry) = locations.entry(key) {
                    ids.push(entry.key().clone());
                    entry.insert(BitVector::zero(p.ty.bits(object.ptr_width)?)?);
                }
            }
        }
        for (p, v) in root.params[0].iter().zip(&input.ins) {
            if v.width() != p.ty.bits(object.ptr_width)? {
                return err("§3.3", "machine input width differs from its bundle");
            }
            locations.insert((name.into(), p.name.clone()), v.clone());
        }
        let arguments = object.bind_arguments(root, &input.args)?;
        for (p, v) in root.params[4].iter().zip(&input.args) {
            if let (Type::Int(_) | Type::Ptr, Argument::Bits(v)) = (&p.ty, v) {
                locations.insert(
                    (name.into(), p.name.clone()),
                    BitVector::from_u128_exact(p.ty.bits(object.ptr_width)?, *v)?,
                );
            }
        }
        let memory = Memory::initial(object, root, &input.regions)?;
        let atop = memory.allocations.len() as u128;
        let count = usize::try_from(budget.steps)
            .map_err(|_| Error("host limit: B frames exceed host address space".into()))?;
        let frame = Frame {
            old_pc: 0,
            slots: widths
                .slots
                .iter()
                .map(|w| Slot::clean(*w))
                .collect::<Result<_>>()?,
            flag: false,
            succ: 0,
        };
        let mut frames = Vec::new();
        frames
            .try_reserve_exact(count)
            .map_err(|e| Error(format!("host limit: cannot allocate B frames: {e}")))?;
        frames.resize(count, frame);
        let target = root.params[1][0].ty.bits(object.ptr_width)?;
        let fault = root.params[2][0].ty.bits(object.ptr_width)?;
        let state = BasisVectorConfiguration {
            locations,
            stack: BTreeMap::new(),
            pc: 0,
            pc_prime: 0,
            faulted: false,
            code: BitVector::zero(fault)?,
            result_val: BitVector::zero(target)?,
            heap_used: counter(widths.hc, 0)?,
            atop: counter(widths.ac, atop)?,
            sp: counter(widths.sc, 0)?,
            fp: counter(widths.sc, 0)?,
            depth: counter(widths.dc, 0)?,
            memory,
            frames,
        };
        Ok(Self {
            object,
            arguments,
            budget,
            program,
            widths,
            state,
            used: 0,
            ids,
        })
    }
    pub fn widths(&self) -> &DerivedWidths {
        &self.widths
    }
    pub fn snapshot(&self) -> BasisVectorConfiguration {
        self.state.clone()
    }
    pub fn frames_clean(&self) -> bool {
        self.state.frames.iter().all(Frame::is_clean)
    }
    /// §5.2 commit, with caller-owned target/fault entry values. These writes
    /// persist while all internal state is subsequently prefix-reversed.
    pub fn commit(&self, target: &BitVector, fault: &BitVector) -> Result<(BitVector, BitVector)> {
        if target.width() != self.state.result_val.width()
            || fault.width() != self.state.code.width()
        {
            return err("§3.3", "commit width mismatch");
        }
        let mut target = target.clone();
        let mut fault = fault.clone();
        let at_halt = self.state.pc == self.program.halt;
        if at_halt && !self.state.faulted {
            target = target.binary("xor", &self.state.result_val)?;
        }
        if self.state.faulted {
            fault = fault.binary("xor", &self.state.code)?;
        }
        if !at_halt {
            fault = fault.binary("xor", &BitVector::from_u128(fault.width(), 1)?)?;
        }
        Ok((target, fault))
    }
    /// F_B as read off the committed step state (§5.2).
    pub fn outcome(&self) -> Result<OracleVectorOutcome> {
        let tw = self.state.result_val.width();
        let fw = self.state.code.width();
        let (payload, code) = self.commit(&BitVector::zero(tw)?, &BitVector::zero(fw)?)?;
        Ok(OracleVectorOutcome {
            payload,
            code,
            steps: self.state.frames[..self.used]
                .iter()
                .filter(|f| f.old_pc != self.program.halt)
                .count() as u64,
        })
    }

    pub fn forward_step(&mut self) -> Result<()> {
        if self.used == self.state.frames.len() {
            return err("§3.6", "forward step exceeds budget");
        }
        let mut frame = self.state.frames[self.used].clone();
        if !frame.is_clean() {
            return err("§3.6", "forward frame is not clean");
        }
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
                        xor_slot(&mut frame.slots[j], v)?;
                    }
                    for (j, d) in destinations.iter().enumerate() {
                        self.exchange(f, d, &mut frame.slots[j])?;
                    }
                    self.direct(f, i, false)?;
                }
            }
            if flag {
                self.state.faulted ^= true;
                self.state.code.xor_low(&code)?;
            }
            self.successor(f, i, old, &frame)?
        };
        self.state.pc_prime ^= successor;
        std::mem::swap(&mut self.state.pc, &mut self.state.pc_prime);
        std::mem::swap(&mut self.state.pc_prime, &mut frame.old_pc);
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
            if frame.flag {
                let code = self.fault_code(f, i)?;
                self.state.faulted ^= true;
                self.state.code.xor_low(&code)?;
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
                        xor_slot(&mut frame.slots[j], v)?;
                    }
                }
                if matches!(i.op.as_str(), "qbranch" | "qjumpr") {
                    frame.succ ^= self.dynamic_successor(f, i)?;
                }
            }
            frame.flag ^= self.flag(f, i)?.0;
        }
        if !frame.is_clean() {
            return err(
                "§3.6",
                format!("inverse frame is not clean at row {old}: {frame:?}"),
            );
        }
        self.state.frames[index] = frame;
        self.used -= 1;
        Ok(())
    }
    fn read(&self, f: &Function, n: &str) -> Result<BitVector> {
        self.state
            .locations
            .get(&(f.name.clone(), n.into()))
            .cloned()
            .ok_or_else(|| Error(format!("§3.7: missing machine location {n} in @{}", f.name)))
    }
    fn location_mut(&mut self, f: &str, n: &str) -> Result<&mut BitVector> {
        self.state
            .locations
            .get_mut(&(f.into(), n.into()))
            .ok_or_else(|| Error(format!("§3.7: missing machine location {n} in @{f}")))
    }
    /// A length operand as an exact integer; None when it is 2^128 or more.
    fn length(&self, f: &Function, n: &str) -> Result<Option<u128>> {
        if integer(n) {
            literal_u128(n).map(Some)
        } else {
            Ok(self.read(f, n)?.to_u128().ok())
        }
    }
    fn stack_read(&self, p: u128) -> Result<BitVector> {
        match self.state.stack.get(&p) {
            Some(v) => Ok(v.clone()),
            None => BitVector::zero(self.widths.w_cell),
        }
    }
    fn stack_set(&mut self, p: u128, v: BitVector) {
        if v.is_zero() {
            self.state.stack.remove(&p);
        } else {
            self.state.stack.insert(p, v);
        }
    }
    fn stack_xor(&mut self, p: u128, v: &BitVector) -> Result<()> {
        let mut cell = self.stack_read(p)?;
        cell.xor_low(v)?;
        self.stack_set(p, cell);
        Ok(())
    }
    fn exchange_location(&mut self, f: &str, n: &str, value: &mut BitVector) -> Result<()> {
        let loc = self.location_mut(f, n)?;
        let w = loc.width();
        if !value.high_clear(w) {
            return err(
                "§3.7",
                "exchange partner has dirty bits above the location width",
            );
        }
        loc.swap_low(value, w)
    }
    fn heap_limit_flag(&self, n: Option<u128>) -> Result<bool> {
        let Some(n) = n else { return Ok(true) };
        let used = number(&self.state.heap_used)?;
        let atop = number(&self.state.atop)?;
        let table = self
            .object
            .section("layout")
            .map(|l| l.get("A_tab").and_then(|a| a.uint()))
            .transpose()?
            .unwrap_or(0);
        Ok(used
            .checked_add(n.max(1))
            .is_none_or(|t| t > self.budget.heap)
            || power_of_two(table).is_some_and(|cap| atop >= cap))
    }
    fn live_entries(&self) -> Result<&[Allocation]> {
        let atop = usize::try_from(number(&self.state.atop)?)
            .map_err(|_| Error("host limit: atop exceeds host address space".into()))?;
        let entries = &self.state.memory.allocations;
        Ok(&entries[..atop.min(entries.len())])
    }
    fn inrange(&self, p: u128, n: Option<u128>) -> Result<bool> {
        let Some(n) = n else { return Ok(false) };
        Ok(self
            .live_entries()?
            .iter()
            .any(|e| e.live && e.base <= p && n <= e.size && p - e.base <= e.size - n))
    }
    fn fault_code(&self, f: &Function, i: &Instruction) -> Result<BitVector> {
        let fw = self.state.code.width();
        let code = match i.op.as_str() {
            "lift.faultif" => return BitVector::parse_unsigned_exact(fw, &i.operands[1]),
            "lift.call" => 3,
            "lift.ret" | "qjumpr" => 4,
            "lift.alloc" => 2,
            "lift.free" | "lift.boundscheck" => 7,
            op if op.starts_with("lift.load.") || op.starts_with("lift.store.") => 7,
            op if op.strip_prefix("lift.").is_some_and(is_div) => {
                let a = self.read(f, &i.operands[0])?;
                let b = self.read(f, &i.operands[1])?;
                a.division_fault(&b, op.starts_with("lift.s"))?.unwrap_or(5)
            }
            _ => 0,
        };
        BitVector::from_u128_exact(fw, code)
    }
    /// The row's flag, computed from the old state before any effect.
    fn flag(&self, f: &Function, i: &Instruction) -> Result<(bool, BitVector)> {
        let a = &i.operands;
        let flag = match i.op.as_str() {
            "lift.call" => {
                let c = &self.object.functions[a[0].trim_start_matches('@')];
                number(&self.state.depth)? + 1 > self.budget.depth
                    || number(&self.state.sp)?
                        .checked_add(c.frame_cells() as u128)
                        .is_none_or(|s| s > self.budget.stack)
            }
            "lift.ret" => self.state.depth.is_zero(),
            "lift.alloc" => self.heap_limit_flag(self.length(f, &a[0])?)?,
            "lift.free" => {
                let p = number(&self.read(f, &a[0])?)?;
                !self
                    .live_entries()?
                    .iter()
                    .any(|e| e.base == p && e.live && !e.pinned)
            }
            "lift.boundscheck" => {
                !self.inrange(number(&self.read(f, &a[0])?)?, self.length(f, &a[1])?)?
            }
            "lift.faultif" => !self.read(f, &a[0])?.is_zero(),
            "qjumpr" => {
                let (start, end) = self.program.ranges[&f.name];
                !self
                    .read(f, &a[0])?
                    .to_u128()
                    .is_ok_and(|n| n >= start as u128 && n < end as u128)
            }
            op if op.starts_with("lift.load.i") || op.starts_with("lift.store.i") => {
                let w = op.rsplit_once(".i").map(|(_, w)| w).unwrap_or("");
                let w = crate::object::host_count(w, "memory width")? as u128;
                !self.inrange(number(&self.read(f, &a[0])?)?, Some(w / 8))?
            }
            op if op.strip_prefix("lift.").is_some_and(is_div) => self
                .read(f, &a[0])?
                .division_fault(&self.read(f, &a[1])?, op.starts_with("lift.s"))?
                .is_some(),
            _ => false,
        };
        Ok((flag, self.fault_code(f, i)?))
    }
    /// The data-dependent successor of qbranch / qjumpr (`succ <- a`, low P bits).
    fn dynamic_successor(&self, f: &Function, i: &Instruction) -> Result<usize> {
        if i.op == "qbranch" {
            self.program.label(
                &f.name,
                if !self.read(f, &i.operands[0])?.is_zero() {
                    &i.operands[1]
                } else {
                    &i.operands[2]
                },
            )
        } else {
            Ok(self
                .read(f, &i.operands[0])?
                .resize(self.widths.p, false)?
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
            op if op.starts_with("lift.store.i") => {
                let w = op.rsplit_once(".i").map(|(_, w)| w).unwrap_or("");
                vec![Destination::Memory(
                    i.operands[0].clone(),
                    crate::object::host_count(w, "store width")?,
                )]
            }
            _ => i
                .lhs
                .iter()
                .map(|n| Destination::Location(n.clone()))
                .collect(),
        })
    }
    /// The A-row values, computed out of place from the current state.
    fn values(&self, f: &Function, i: &Instruction) -> Result<Vec<Slot>> {
        let op = i.op.as_str();
        let a = &i.operands;
        let rd = |n: &str| self.read(f, n);
        let word = |v: BitVector| Slot {
            word: v,
            size: 0,
            live: false,
            pinned: false,
        };
        let values: Vec<BitVector> = if let Some(w) = op.strip_prefix("lift.const.i") {
            vec![BitVector::parse_decimal(
                crate::object::host_count(w, "const width")? as usize,
                &a[0],
            )?]
        } else if let Some(binary) = op
            .strip_prefix("lift.")
            .filter(|s| is_binop(s) || is_div(s))
        {
            vec![rd(&a[0])?.binary(binary, &rd(&a[1])?)?]
        } else if let Some(pred) = op.strip_prefix("lift.icmp.") {
            vec![BitVector::from_u128(
                1,
                u128::from(rd(&a[0])?.compare(pred, &rd(&a[1])?)?),
            )?]
        } else if let Some((cast, target)) = op
            .strip_prefix("lift.")
            .and_then(|s| s.split_once(".i"))
            .filter(|(c, _)| ["zext", "sext", "trunc"].contains(c))
        {
            vec![rd(&a[0])?.resize(
                crate::object::host_count(target, "cast width")? as usize,
                cast == "sext",
            )?]
        } else if let Some(w) = op.strip_prefix("lift.load.i") {
            vec![self.state.memory.load(
                number(&rd(&a[0])?)?,
                crate::object::host_count(w, "load width")?,
            )?]
        } else if op.starts_with("lift.store.i") {
            vec![rd(&a[1])?]
        } else {
            match op {
                "lift.mov" => vec![rd(&a[0])?],
                "lift.select" => vec![rd(if !rd(&a[0])?.is_zero() { &a[1] } else { &a[2] })?],
                "lift.parmov" => list(&a[0])?.iter().map(|s| rd(s)).collect::<Result<_>>()?,
                "lift.label" => vec![BitVector::from_u128_exact(
                    operand_width(self.object, f, &i.lhs[0])? as usize,
                    self.program.label(&f.name, &a[0])? as u128,
                )?],
                "lift.addr" => vec![BitVector::from_u128_exact(
                    self.object.ptr_width as usize,
                    *self
                        .state
                        .memory
                        .addresses
                        .get(a[0].trim_start_matches('@'))
                        .ok_or_else(|| Error("§4.2: address not mapped".into()))?,
                )?],
                "lift.alloc" => {
                    let n = self
                        .length(f, &a[0])?
                        .ok_or_else(|| Error("§3.6: unguarded allocation length".into()))?;
                    let used = number(&self.state.heap_used)?;
                    let base = self
                        .state
                        .memory
                        .heap_base
                        .checked_add(used)
                        .ok_or_else(|| Error("§7.1: heap bump exceeds 2^128".into()))?;
                    let ptr = self.object.ptr_width as usize;
                    return Ok(vec![
                        word(BitVector::from_u128_exact(ptr, base)?),
                        Slot {
                            word: BitVector::from_u128_exact(ptr, base)?,
                            size: n,
                            live: true,
                            pinned: false,
                        },
                        word(counter(self.widths.hc, used + n.max(1))?),
                        word(counter(self.widths.ac, number(&self.state.atop)? + 1)?),
                    ]);
                }
                _ => Vec::new(),
            }
        };
        Ok(values.into_iter().map(word).collect())
    }
    fn exchange(&mut self, f: &Function, d: &Destination, s: &mut Slot) -> Result<()> {
        match d {
            Destination::Location(n) => self.exchange_location(&f.name, n, &mut s.word)?,
            Destination::Memory(n, width) => {
                // The N/8 cells at a exchange with the low N bits of the slot.
                let p = number(&self.read(f, n)?)?;
                let w = *width as usize;
                if !s.word.high_clear(w) {
                    return err("§3.6", "store slot has dirty bits above N");
                }
                let old = self.state.memory.load(p, *width)?;
                self.state.memory.store(p, &s.word.resize(w, false)?)?;
                s.word = old.resize(s.word.width(), false)?;
            }
            Destination::Heap => {
                let width = self.state.heap_used.width();
                if !s.word.high_clear(width) {
                    return err("§3.7", "heap_used slot exceeds Hc");
                }
                self.state.heap_used.swap_low(&mut s.word, width)?;
            }
            Destination::Atop => {
                let width = self.state.atop.width();
                if !s.word.high_clear(width) {
                    return err("§3.7", "atop slot exceeds Ac");
                }
                self.state.atop.swap_low(&mut s.word, width)?;
            }
            Destination::EntryAtop => {
                // alloctab[atop], addressed with the counter value current now (§3.6).
                let index = usize::try_from(number(&self.state.atop)?)
                    .map_err(|_| Error("host limit: alloctab index exceeds usize".into()))?;
                let entries = &mut self.state.memory.allocations;
                let clean = Allocation {
                    base: 0,
                    size: 0,
                    live: false,
                    pinned: false,
                };
                while entries.len() <= index {
                    entries.push(clean.clone());
                }
                let old = entries[index].clone();
                entries[index] = Allocation {
                    base: number(&s.word)?,
                    size: s.size,
                    live: s.live,
                    pinned: s.pinned,
                };
                while entries.last() == Some(&clean) {
                    entries.pop();
                }
                *s = Slot {
                    word: BitVector::from_u128_exact(s.word.width(), old.base)?,
                    size: old.size,
                    live: old.live,
                    pinned: old.pinned,
                };
            }
        }
        Ok(())
    }
    /// D rows (and the direct parts of lifted region calls).
    fn direct(&mut self, f: &Function, i: &Instruction, inverse: bool) -> Result<()> {
        match i.op.as_str() {
            "lift.free" => {
                // live_i ^= match_i for every i < atop with base_i = p (§4.2).
                let p = number(&self.read(f, &i.operands[0])?)?;
                let atop = self.live_entries()?.len();
                for e in &mut self.state.memory.allocations[..atop] {
                    if e.base == p {
                        e.live ^= true;
                    }
                }
            }
            "lift.retval" => {
                let v = self.read(f, &i.operands[0])?;
                self.state.result_val = self.state.result_val.binary("xor", &v)?;
            }
            "fault" => {
                let code =
                    BitVector::parse_unsigned_exact(self.state.code.width(), &i.operands[0])?;
                self.state.code = self.state.code.binary("xor", &code)?;
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
            let addr = base
                .checked_add(3 + j as u128)
                .ok_or_else(|| Error("§3.7: stack address exceeds 2^128".into()))?;
            let mut cell = self.stack_read(addr)?;
            self.exchange_location(&f.name, &p.name, &mut cell)?;
            self.stack_set(addr, cell);
        }
        Ok(())
    }
    fn word(&self, v: u128) -> Result<BitVector> {
        BitVector::from_u128_exact(self.widths.w_cell, v)
    }
    fn exchange_counter(slot: &mut Slot, counter: &mut BitVector) -> Result<()> {
        let w = counter.width();
        if !slot.word.high_clear(w) {
            return err("§3.7", "counter slot exceeds the counter width");
        }
        counter.swap_low(&mut slot.word, w)
    }
    fn destination_id(&self, f: &Function, n: &str) -> Result<u128> {
        self.ids
            .iter()
            .position(|id| id.0 == f.name && id.1 == n)
            .map(|p| p as u128)
            .ok_or_else(|| Error("§3.7: missing destination id".into()))
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
        // C1 capture before moving ANY callee location (recursive arguments too).
        for (j, a) in args.iter().enumerate() {
            frame.slots[j].word.xor_low(&self.read(f, a)?)?;
        }
        let sp = number(&self.state.sp)?;
        self.push_pop_locations(callee, sp)?; // C2
        let fp = number(&self.state.fp)?;
        let id = self.destination_id(f, &i.lhs[0])?;
        self.stack_xor(sp, &self.word(k as u128 + 1)?)?; // C3
        self.stack_xor(sp + 1, &self.word(fp)?)?;
        self.stack_xor(sp + 2, &self.word(id)?)?;
        for (j, p) in callee.params[0].iter().enumerate() {
            let value = frame.slots[j]
                .word
                .resize(p.ty.bits(self.object.ptr_width)?, false)?;
            self.location_mut(&callee.name, &p.name)?.xor_low(&value)?; // C4
        }
        let cells = callee.frame_cells() as u128;
        let depth = number(&self.state.depth)?;
        frame.slots[m].word.xor_low(&counter(self.widths.sc, sp)?)?; // C5
        frame.slots[m + 1]
            .word
            .xor_low(&counter(self.widths.sc, sp + cells)?)?;
        frame.slots[m + 2]
            .word
            .xor_low(&counter(self.widths.dc, depth + 1)?)?;
        Self::exchange_counter(&mut frame.slots[m], &mut self.state.fp)?;
        Self::exchange_counter(&mut frame.slots[m + 1], &mut self.state.sp)?;
        Self::exchange_counter(&mut frame.slots[m + 2], &mut self.state.depth)?;
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
        // C5^-1: exchanges in reverse order, then clear from the restored counters.
        Self::exchange_counter(&mut frame.slots[m + 2], &mut self.state.depth)?;
        Self::exchange_counter(&mut frame.slots[m + 1], &mut self.state.sp)?;
        Self::exchange_counter(&mut frame.slots[m], &mut self.state.fp)?;
        let sp = number(&self.state.sp)?;
        let depth = number(&self.state.depth)?;
        let cells = callee.frame_cells() as u128;
        frame.slots[m + 2]
            .word
            .xor_low(&counter(self.widths.dc, depth + 1)?)?;
        frame.slots[m + 1]
            .word
            .xor_low(&counter(self.widths.sc, sp + cells)?)?;
        frame.slots[m].word.xor_low(&counter(self.widths.sc, sp)?)?;
        for (j, p) in callee.params[0].iter().enumerate() {
            let value = frame.slots[j]
                .word
                .resize(p.ty.bits(self.object.ptr_width)?, false)?;
            self.location_mut(&callee.name, &p.name)?.xor_low(&value)?; // C4^-1
        }
        let fp = number(&self.state.fp)?;
        let id = self.destination_id(f, &i.lhs[0])?;
        self.stack_xor(sp + 2, &self.word(id)?)?; // C3^-1 at the restored sp, fp
        self.stack_xor(sp + 1, &self.word(fp)?)?;
        self.stack_xor(sp, &self.word(k as u128 + 1)?)?;
        self.push_pop_locations(callee, sp)?; // C2^-1
        for (j, a) in args.iter().enumerate() {
            frame.slots[j].word.xor_low(&self.read(f, a)?)?; // C1^-1
        }
        Ok(())
    }
    /// R4: under (stack[fp+2] == id(l)), exchange disp_1 with l.
    fn deliver(&mut self, slot: &mut Slot) -> Result<()> {
        let fp = number(&self.state.fp)?;
        let id = number(&self.stack_read(fp + 2)?)?;
        let (f, n) = usize::try_from(id)
            .ok()
            .and_then(|id| self.ids.get(id))
            .cloned()
            .ok_or_else(|| Error("§3.7: delivery id names no location".into()))?;
        self.exchange_location(&f, &n, &mut slot.word)
    }
    fn forward_ret(&mut self, f: &Function, i: &Instruction, frame: &mut Frame) -> Result<()> {
        if frame.flag {
            return Ok(());
        }
        let fp = number(&self.state.fp)?;
        frame.succ ^= number(&self.stack_read(fp)?)? as usize; // R1
        frame.slots[0]
            .word
            .xor_low(&self.read(f, &i.operands[0])?)?; // R2
        self.push_pop_locations(f, fp)?; // R3
        let mut delivery = frame.slots[0].clone();
        self.deliver(&mut delivery)?; // R4
        frame.slots[0] = delivery;
        let saved_fp = self.stack_read(fp + 1)?.resize(self.widths.sc, false)?;
        let depth = number(&self.state.depth)?;
        frame.slots[1].word.xor_low(&saved_fp)?; // R5
        frame.slots[2]
            .word
            .xor_low(&counter(self.widths.dc, depth - 1)?)?;
        Self::exchange_counter(&mut frame.slots[1], &mut self.state.fp)?;
        Self::exchange_counter(&mut frame.slots[2], &mut self.state.depth)?;
        Ok(())
    }
    fn inverse_ret(&mut self, f: &Function, i: &Instruction, frame: &mut Frame) -> Result<()> {
        if frame.flag {
            return Ok(());
        }
        Self::exchange_counter(&mut frame.slots[2], &mut self.state.depth)?; // R5^-1
        Self::exchange_counter(&mut frame.slots[1], &mut self.state.fp)?;
        let fp = number(&self.state.fp)?;
        let depth = number(&self.state.depth)?;
        frame.slots[2]
            .word
            .xor_low(&counter(self.widths.dc, depth - 1)?)?;
        let saved_fp = self.stack_read(fp + 1)?.resize(self.widths.sc, false)?;
        frame.slots[1].word.xor_low(&saved_fp)?;
        let mut delivery = frame.slots[0].clone();
        self.deliver(&mut delivery)?; // R4^-1 at the restored fp
        frame.slots[0] = delivery;
        self.push_pop_locations(f, fp)?; // R3^-1
        frame.slots[0]
            .word
            .xor_low(&self.read(f, &i.operands[0])?)?; // R2^-1
        frame.succ ^= number(&self.stack_read(fp)?)? as usize; // R1^-1
        Ok(())
    }
    fn nested_oracle(&mut self, f: &Function, i: &Instruction) -> Result<()> {
        let a = &i.operands;
        let callee = &self.object.functions[a[0].trim_start_matches('@')];
        let Some(Argument::Token(budget)) = self.arguments.get(&a[1]).cloned() else {
            return err("§4.2", "tk must be a token argument of the calling oracle");
        };
        let mut input = OracleVectorInput {
            ins: list(&a[2])?
                .iter()
                .map(|n| self.read(f, n))
                .collect::<Result<_>>()?,
            ..Default::default()
        };
        for (p, n) in callee.params[3].iter().zip(list(&a[5])?) {
            let Type::Region(aw, _) = p.ty else {
                return err("§5.1", "mem slot requires region type");
            };
            input
                .regions
                .insert(p.name.clone(), region_bytes(&self.state.memory, &n, aw)?);
        }
        for n in list(&a[6])? {
            input.args.push(
                self.arguments
                    .get(&n)
                    .cloned()
                    .ok_or_else(|| Error(format!("§4.2: {n} is not a classical argument")))?,
            );
        }
        // The nested invocation is itself compute / commit / prefix reversal,
        // executed as a reversible machine (§5.2).
        let mut child = Self::new(self.object, &callee.name, &input, budget)?;
        let entry = child.snapshot();
        for _ in 0..budget.steps {
            child.forward_step()?;
        }
        let outcome = child.outcome()?;
        for _ in 0..budget.steps {
            child.inverse_step()?;
        }
        if child.snapshot() != entry || !child.frames_clean() {
            return err(
                "§5.2",
                "nested oracle prefix reversal did not restore its workspace",
            );
        }
        let target = self.location_mut(&f.name, &a[3])?;
        *target = target.binary("xor", &outcome.payload)?;
        let fault = self.location_mut(&f.name, &a[4])?;
        *fault = fault.binary("xor", &outcome.code)?;
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
            let Type::Region(aw, _) = p.ty else {
                return err("§5.1", "mem slot requires region type");
            };
            regions.insert(p.name.clone(), region_bytes(&self.state.memory, &n, aw)?);
        }
        let args = list(&a[4])?
            .iter()
            .map(|n| {
                self.arguments
                    .get(n)
                    .cloned()
                    .ok_or_else(|| Error(format!("§4.2: {n} is not a classical argument")))
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
            *self.location_mut(&f.name, n)? = v;
        }
        Ok(())
    }
}
fn xor_slot(slot: &mut Slot, v: &Slot) -> Result<()> {
    slot.word.xor_low(&v.word)?;
    slot.size ^= v.size;
    slot.live ^= v.live;
    slot.pinned ^= v.pinned;
    Ok(())
}

/// Small-integer adapter over `OracleVectorMachine`. Interfaces whose target
/// or fault width exceeds 128 bits are rejected loudly at construction, so
/// `outcome` and `commit` never truncate.
pub struct OracleMachine<'a>(OracleVectorMachine<'a>);
impl<'a> OracleMachine<'a> {
    pub fn new(
        object: &'a Object,
        name: &str,
        input: &OracleInput,
        budget: Budget,
    ) -> Result<Self> {
        let f = object
            .functions
            .get(name)
            .ok_or_else(|| Error(format!("§5.2: unknown oracle @{name}")))?;
        if f.kind == "oracle"
            && (f.params[1][0].ty.width(object.ptr_width)? > 128
                || f.params[2][0].ty.width(object.ptr_width)? > 128)
        {
            return err(
                "§3.3",
                "target/fault wider than 128 bits: use OracleVectorMachine for this interface",
            );
        }
        let input = OracleVectorInput::from_input(object, name, input)?;
        Ok(Self(OracleVectorMachine::new(
            object, name, &input, budget,
        )?))
    }
    pub fn snapshot(&self) -> BasisConfiguration {
        self.0.snapshot()
    }
    pub fn frames_clean(&self) -> bool {
        self.0.frames_clean()
    }
    pub fn widths(&self) -> &DerivedWidths {
        self.0.widths()
    }
    pub fn forward_step(&mut self) -> Result<()> {
        self.0.forward_step()
    }
    pub fn inverse_step(&mut self) -> Result<()> {
        self.0.inverse_step()
    }
    pub fn outcome(&self) -> Result<OracleOutcome> {
        self.0.outcome()?.narrow()
    }
    pub fn commit(&self, target: u128, fault: u128) -> Result<(u128, u128)> {
        let tw = self.0.state.result_val.width();
        let fw = self.0.state.code.width();
        let (t, f) = self.0.commit(
            &BitVector::from_u128_exact(tw, target)?,
            &BitVector::from_u128_exact(fw, fault)?,
        )?;
        Ok((t.to_u128()?, f.to_u128()?))
    }
}

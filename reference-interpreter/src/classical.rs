//! CC/R dynamic SSA, retained history and reversible device models (§7).
use crate::{
    err,
    object::{is_binop, is_div, list, mask, parse_bits},
    oracle::{binop, compare, divide, signed, Memory},
    Budget, Error, Object, OracleInput, Result, Type,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClassicalValue {
    Int(u16, u128),
    Ptr(u128),
    F64(u64),
    Token(Budget),
    Qreg(u64),
}
impl ClassicalValue {
    pub fn int(width: u16, value: u128) -> Result<Self> {
        if !(1..=128).contains(&width) || value & !mask(width) != 0 {
            return err("§3.3", "classical value does not fit integer width");
        }
        Ok(Self::Int(width, value))
    }
    fn bits(&self) -> Result<u128> {
        match self {
            Self::Int(_, v) | Self::Ptr(v) => Ok(*v),
            _ => err("§3.3", "expected classical bit-vector value"),
        }
    }
    fn width(&self, ptr: u16) -> Result<u16> {
        match self {
            Self::Int(w, _) => Ok(*w),
            Self::Ptr(_) => Ok(ptr),
            _ => err("§3.3", "value has no integer width"),
        }
    }
    fn matches(&self, t: &Type, ptr: u16) -> bool {
        match (self, t) {
            (Self::Int(w, v), Type::Int(n)) => w == n && *v & !mask(*w) == 0,
            (Self::Ptr(v), Type::Ptr) => *v & !mask(ptr) == 0,
            (Self::F64(v), Type::F64) => f64::from_bits(*v).is_finite(),
            (Self::Token(_), Type::Token) | (Self::Qreg(_), Type::Qreg(_)) => true,
            _ => false,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Activation {
    id: u64,
    function: String,
    block: usize,
    ip: usize,
    bindings: Vec<String>,
    segment: Option<usize>,
}
type Key = (u64, String);
#[derive(Clone, Debug, PartialEq, Eq, Default)]
struct Devices {
    console: Vec<u8>,
    clock: u64,
    entropy_position: usize,
    blocks: BTreeMap<u128, Vec<u8>>,
    tape: Vec<(String, String, Vec<u128>)>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassicalSnapshot {
    instances: BTreeMap<Key, Vec<ClassicalValue>>,
    memory: Memory,
    devices: Devices,
    control: Vec<Activation>,
    next_id: u64,
    fault: Option<u128>,
    finished: Option<Vec<ClassicalValue>>,
}
#[derive(Clone, Debug)]
struct Undo {
    control: Vec<Activation>,
    next_id: u64,
    definitions: Vec<Key>,
    bytes: Vec<(u128, u8)>,
    allocation: Option<(u128, usize)>,
    free: Option<(usize, bool)>,
    device: Option<DeviceUndo>,
    tape: bool,
    fault: Option<u128>,
    finished: Option<Vec<ClassicalValue>>,
}
#[derive(Clone, Debug)]
enum DeviceUndo {
    Console(usize),
    Clock(u64),
    Entropy(usize),
    Block(u128, Option<Vec<u8>>),
}
#[derive(Clone, Debug)]
struct HistoryFrame {
    undo: Undo,
    reversed: bool,
    marker: bool,
    pure: bool,
    irreversible: bool,
}
#[derive(Clone, Debug)]
struct Segment {
    start: usize,
    end: Option<usize>,
    callee: String,
    args: Vec<ClassicalValue>,
    results: Vec<ClassicalValue>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassicalOutcome {
    pub values: Vec<ClassicalValue>,
    pub fault: Option<u128>,
    pub trapped: bool,
    pub steps: u64,
}
pub struct ClassicalMachine<'a> {
    object: &'a Object,
    state: ClassicalSnapshot,
    history: Vec<HistoryFrame>,
    segments: Vec<Segment>,
    entropy: Vec<u8>,
    initial: ClassicalSnapshot,
}

impl<'a> ClassicalMachine<'a> {
    pub fn new(
        object: &'a Object,
        entry: &str,
        args: &[ClassicalValue],
        entropy: &[u8],
    ) -> Result<Self> {
        let f = object
            .functions
            .get(entry)
            .filter(|f| matches!(f.kind.as_str(), "cc" | "r"))
            .ok_or_else(|| Error("§3.4: classical entry not found".into()))?;
        if args.len() != f.params[0].len()
            || args
                .iter()
                .zip(&f.params[0])
                .any(|(a, p)| !a.matches(&p.ty, object.ptr_width))
        {
            return err("§10", "classical entry argument type/count mismatch");
        }
        let instances = f.params[0]
            .iter()
            .zip(args)
            .map(|(p, v)| ((0, p.name.clone()), vec![v.clone()]))
            .collect();
        let mut devices = Devices::default();
        if let Some(l) = object.section("layout") {
            if let Some(image) = l.object()?.get("block_image") {
                let data = &object.data[image.string()?];
                if data.len() % 512 != 0 {
                    return err("§7.3", "block image must have complete 512-byte sectors");
                }
                for (i, sector) in data.chunks_exact(512).enumerate() {
                    devices.blocks.insert(i as u128, sector.into());
                }
            }
        }
        let state = ClassicalSnapshot {
            instances,
            memory: Memory::initial(object, f, &OracleInput::default())?,
            devices,
            control: vec![Activation {
                id: 0,
                function: entry.into(),
                block: 0,
                ip: 0,
                bindings: Vec::new(),
                segment: None,
            }],
            next_id: 1,
            fault: None,
            finished: None,
        };
        Ok(Self {
            object,
            state: state.clone(),
            history: Vec::new(),
            segments: Vec::new(),
            entropy: entropy.into(),
            initial: state,
        })
    }
    pub fn snapshot(&self) -> ClassicalSnapshot {
        self.state.clone()
    }
    pub fn clock(&self) -> u64 {
        self.state.devices.clock
    }
    pub fn console(&self) -> &[u8] {
        &self.state.devices.console
    }
    pub fn effect_tape_len(&self) -> usize {
        self.state.devices.tape.len()
    }
    pub fn run(&mut self, max_steps: u64) -> Result<ClassicalOutcome> {
        let mut steps = 0;
        while self.state.fault.is_none() && self.state.finished.is_none() {
            if steps == max_steps {
                return err(
                    "§3.4",
                    "reference execution fuel exhausted; CC/R has no ISA step budget",
                );
            }
            self.step()?;
            steps += 1;
        }
        Ok(ClassicalOutcome {
            values: self.state.finished.clone().unwrap_or_default(),
            fault: self.state.fault,
            trapped: self.state.fault.is_some()
                && self
                    .state
                    .control
                    .last()
                    .is_some_and(|a| self.object.functions[&a.function].kind == "cc"),
            steps,
        })
    }
    pub fn reverse(&mut self) -> Result<()> {
        if self.history.iter().any(|f| f.irreversible) {
            return err(
                "§10",
                "E_NOT_REVERSIBLE: measurement or unverified release in history",
            );
        }
        for n in (0..self.history.len()).rev() {
            self.undo_frame(n)?;
        }
        self.history.clear();
        self.segments.clear();
        if self.state != self.initial {
            return err("§7.4", "whole-run reversal failed to restore entry tuple");
        }
        Ok(())
    }
    fn read(&self, name: &str) -> Result<ClassicalValue> {
        let id = self
            .state
            .control
            .last()
            .ok_or_else(|| Error("§3.4: empty activation stack".into()))?
            .id;
        self.state
            .instances
            .get(&(id, name.into()))
            .and_then(|v| v.last())
            .cloned()
            .ok_or_else(|| Error(format!("§7.4: instance {name} no longer exists")))
    }
    fn bits(&self, name: &str) -> Result<u128> {
        self.read(name)?.bits()
    }
    fn define(&mut self, name: &str, value: ClassicalValue, undo: &mut Undo) -> Result<()> {
        let id = self
            .state
            .control
            .last()
            .ok_or_else(|| Error("§3.4: define without activation".into()))?
            .id;
        self.define_at(id, name, value, undo);
        Ok(())
    }
    fn define_at(&mut self, id: u64, name: &str, value: ClassicalValue, undo: &mut Undo) {
        let key = (id, name.into());
        self.state
            .instances
            .entry(key.clone())
            .or_default()
            .push(value);
        undo.definitions.push(key);
    }
    fn save_bytes(&self, p: u128, len: usize, undo: &mut Undo) {
        for n in 0..len {
            undo.bytes
                .push((p + n as u128, self.state.memory.byte(p + n as u128)));
        }
    }
    fn undo_record(&mut self, undo: Undo) -> Result<()> {
        for key in undo.definitions.into_iter().rev() {
            let stack = self
                .state
                .instances
                .get_mut(&key)
                .ok_or_else(|| Error("§7.4: missing retained dynamic instance".into()))?;
            stack
                .pop()
                .ok_or_else(|| Error("§7.4: empty retained dynamic instance".into()))?;
            if stack.is_empty() {
                self.state.instances.remove(&key);
            }
        }
        for (p, b) in undo.bytes.into_iter().rev() {
            self.state.memory.set_byte(p, b);
        }
        if let Some((used, len)) = undo.allocation {
            self.state.memory.heap_used = used;
            self.state.memory.allocations.truncate(len);
        }
        if let Some((n, live)) = undo.free {
            self.state.memory.allocations[n].live = live;
        }
        if undo.tape {
            self.state
                .devices
                .tape
                .pop()
                .ok_or_else(|| Error("§7.3: empty effect tape on reversal".into()))?;
        }
        if let Some(d) = undo.device {
            match d {
                DeviceUndo::Console(n) => self.state.devices.console.truncate(n),
                DeviceUndo::Clock(c) => self.state.devices.clock = c,
                DeviceUndo::Entropy(p) => self.state.devices.entropy_position = p,
                DeviceUndo::Block(n, Some(v)) => {
                    self.state.devices.blocks.insert(n, v);
                }
                DeviceUndo::Block(n, None) => {
                    self.state.devices.blocks.remove(&n);
                }
            }
        }
        self.state.control = undo.control;
        self.state.next_id = undo.next_id;
        self.state.fault = undo.fault;
        self.state.finished = undo.finished;
        Ok(())
    }
    fn undo_frame(&mut self, n: usize) -> Result<()> {
        if self.history[n].reversed || self.history[n].marker {
            return Ok(());
        }
        let undo = self.history[n].undo.clone();
        self.undo_record(undo)?;
        self.history[n].reversed = true;
        Ok(())
    }
    fn effect(&mut self, op: &str, args: &[String], undo: &mut Undo) -> Result<Option<u128>> {
        let Some((dev, operation)) = op.split_once('.') else {
            return Ok(Some(8));
        };
        let values = args
            .iter()
            .map(|s| self.bits(s))
            .collect::<Result<Vec<_>>>()?;
        let arity = match (dev, operation) {
            ("console", "write") | ("entropy", "read") | ("block", "read" | "write") => 2,
            ("clock", "read") => 1,
            _ => return Ok(Some(8)),
        };
        if values.len() != arity {
            return err("§7.3", "device operation arity mismatch");
        }
        let (p, len) = match dev {
            "clock" => (values[0], 8),
            "block" => (values[1], 512),
            _ => (values[0], values[1]),
        };
        if !self.state.memory.valid(p, len) {
            return Ok(Some(7));
        }
        let len = usize::try_from(len)
            .map_err(|_| Error("§7.3: device length exceeds host address space".into()))?;
        if dev == "entropy"
            && self
                .state
                .devices
                .entropy_position
                .checked_add(len)
                .is_none_or(|end| end > self.entropy.len())
        {
            return Ok(Some(10));
        }
        match (dev, operation) {
            ("console", "write") => {
                undo.device = Some(DeviceUndo::Console(self.state.devices.console.len()));
                let bytes: Vec<_> = (0..len)
                    .map(|i| self.state.memory.byte(p + i as u128))
                    .collect();
                self.state.devices.console.extend(bytes);
            }
            ("clock", "read") => {
                self.save_bytes(p, 8, undo);
                let c = self.state.devices.clock;
                undo.device = Some(DeviceUndo::Clock(c));
                self.state.memory.store(p, 64, c as u128);
                self.state.devices.clock = c.wrapping_add(1);
            }
            ("entropy", "read") => {
                self.save_bytes(p, len, undo);
                let start = self.state.devices.entropy_position;
                undo.device = Some(DeviceUndo::Entropy(start));
                for n in 0..len {
                    self.state
                        .memory
                        .set_byte(p + n as u128, self.entropy[start + n]);
                }
                self.state.devices.entropy_position += len;
            }
            ("block", "read") => {
                self.save_bytes(p, 512, undo);
                let bytes = self
                    .state
                    .devices
                    .blocks
                    .get(&values[0])
                    .cloned()
                    .unwrap_or_else(|| vec![0; 512]);
                for (n, b) in bytes.into_iter().enumerate() {
                    self.state.memory.set_byte(p + n as u128, b);
                }
            }
            ("block", "write") => {
                let sector = values[0];
                let old = self.state.devices.blocks.get(&sector).cloned();
                undo.device = Some(DeviceUndo::Block(sector, old));
                let bytes = (0..512).map(|n| self.state.memory.byte(p + n)).collect();
                self.state.devices.blocks.insert(sector, bytes);
            }
            _ => unreachable!("device operation validated"),
        }
        self.state
            .devices
            .tape
            .push((dev.into(), operation.into(), values));
        undo.tape = true;
        Ok(None)
    }
    fn step(&mut self) -> Result<()> {
        let object = self.object;
        let current = self
            .state
            .control
            .last()
            .ok_or_else(|| Error("§3.4: no active function".into()))?
            .clone();
        let f = &object.functions[&current.function];
        let i = f.blocks[current.block]
            .instructions
            .get(current.ip)
            .ok_or_else(|| Error("§3.4: fell off classical block".into()))?
            .clone();
        let op = i.op.as_str();
        let a = &i.operands;
        let mut undo = Undo {
            control: self.state.control.clone(),
            next_id: self.state.next_id,
            definitions: Vec::new(),
            bytes: Vec::new(),
            allocation: None,
            free: None,
            device: None,
            tape: false,
            fault: self.state.fault,
            finished: self.state.finished.clone(),
        };
        self.state.control.last_mut().expect("activation").ip += 1;
        let mut fault = None;
        let mut output = None;
        let mut marker = false;
        let pure = op.starts_with("const.")
            || is_binop(op)
            || is_div(op)
            || op.starts_with("icmp.")
            || op == "select"
            || ["zext.", "sext.", "trunc.", "ptrtoint.", "load."]
                .iter()
                .any(|p| op.starts_with(p))
            || op == "inttoptr"
            || op == "br";
        let execute = (|| -> Result<()> {
            if let Some(t) = op.strip_prefix("const.") {
                let Type::Int(w) = Type::parse(t)? else {
                    return err("§4.8", "const type");
                };
                output = Some(ClassicalValue::Int(w, parse_bits(&a[0], w)?));
            } else if is_binop(op) || is_div(op) {
                let av = self.read(&a[0])?;
                let w = av.width(object.ptr_width)?;
                let left = av.bits()?;
                let right = self.bits(&a[1])?;
                let value = if is_binop(op) {
                    Some(binop(op, left, right, w))
                } else {
                    match divide(op, left, right, w) {
                        Ok(v) => Some(v),
                        Err(c) => {
                            fault = Some(c);
                            None
                        }
                    }
                };
                output = value.map(|v| ClassicalValue::Int(w, v));
            } else if let Some(pred) = op.strip_prefix("icmp.") {
                let av = self.read(&a[0])?;
                output = Some(ClassicalValue::Int(
                    1,
                    compare(
                        pred,
                        av.bits()?,
                        self.bits(&a[1])?,
                        av.width(object.ptr_width)?,
                    ) as u128,
                ));
            } else if let Some((cast, t)) = op
                .split_once('.')
                .filter(|(c, _)| ["zext", "sext", "trunc", "ptrtoint"].contains(c))
            {
                let Type::Int(w) = Type::parse(t)? else {
                    return err("§4.8", "cast target type");
                };
                let from = self.read(&a[0])?;
                let value = if cast == "sext" {
                    signed(from.bits()?, from.width(object.ptr_width)?) as u128
                } else {
                    from.bits()?
                };
                output = Some(ClassicalValue::Int(w, value & mask(w)));
            } else if let Some(t) = op.strip_prefix("load.") {
                let w = Type::parse(t)?.width(object.ptr_width)?;
                let p = self.bits(&a[0])?;
                if !self.state.memory.valid(p, (w / 8) as u128) {
                    fault = Some(7);
                } else {
                    output = Some(ClassicalValue::Int(w, self.state.memory.load(p, w)));
                }
            } else if let Some(t) = op.strip_prefix("store.") {
                let w = Type::parse(t)?.width(object.ptr_width)?;
                let p = self.bits(&a[0])?;
                let v = self.bits(&a[1])?;
                if !self.state.memory.valid(p, (w / 8) as u128) {
                    fault = Some(7);
                } else {
                    self.save_bytes(p, (w / 8) as usize, &mut undo);
                    self.state.memory.store(p, w, v);
                }
            } else {
                match op {
                    "select" => {
                        output =
                            Some(self.read(if self.bits(&a[0])? != 0 { &a[1] } else { &a[2] })?)
                    }
                    "inttoptr" => {
                        output = Some(ClassicalValue::Ptr(
                            self.bits(&a[0])? & mask(object.ptr_width),
                        ))
                    }
                    "alloc" => {
                        let n = self.bits(&a[0])?;
                        let b = object.default_budget(0)?;
                        if self.state.memory.allocation_fails(n, b) {
                            fault = Some(2);
                        } else {
                            undo.allocation = Some((
                                self.state.memory.heap_used,
                                self.state.memory.allocations.len(),
                            ));
                            output = Some(ClassicalValue::Ptr(self.state.memory.allocate(n)));
                        }
                    }
                    "free" => {
                        let p = self.bits(&a[0])?;
                        if let Some(n) = self
                            .state
                            .memory
                            .allocations
                            .iter()
                            .position(|e| e.base == p && e.live && !e.pinned)
                        {
                            undo.free = Some((n, true));
                            self.state.memory.allocations[n].live = false;
                        } else {
                            fault = Some(7);
                        }
                    }
                    "br" => {
                        let target = if a.len() == 1 {
                            &a[0]
                        } else if self.bits(&a[0])? != 0 {
                            &a[1]
                        } else {
                            &a[2]
                        };
                        let (n, args) = target
                            .trim_start_matches('^')
                            .split_once('(')
                            .ok_or_else(|| Error("§4.1: malformed branch".into()))?;
                        let values = list(&format!("({args}"))?
                            .iter()
                            .map(|n| self.read(n))
                            .collect::<Result<Vec<_>>>()?;
                        let block = f
                            .blocks
                            .iter()
                            .position(|b| b.name == n)
                            .expect("validated branch target");
                        for (p, v) in f.blocks[block].params.iter().zip(values) {
                            self.define(&p.name, v, &mut undo)?;
                        }
                        let active = self.state.control.last_mut().expect("activation");
                        active.block = block;
                        active.ip = 0;
                    }
                    "call" => {
                        let n = a[0].trim_start_matches('@');
                        let callee = &object.functions[n];
                        let args = if a.len() == 2 && a[1].starts_with('(') {
                            list(&a[1])?
                        } else {
                            a[1..].to_vec()
                        };
                        let values = args
                            .iter()
                            .map(|n| self.read(n))
                            .collect::<Result<Vec<_>>>()?;
                        let segment = self.segments.len();
                        self.segments.push(Segment {
                            start: self.history.len(),
                            end: None,
                            callee: n.into(),
                            args: values.clone(),
                            results: Vec::new(),
                        });
                        let id = self.state.next_id;
                        self.state.next_id += 1;
                        self.state.control.push(Activation {
                            id,
                            function: n.into(),
                            block: 0,
                            ip: 0,
                            bindings: i.lhs.clone(),
                            segment: Some(segment),
                        });
                        for (p, v) in callee.params[0].iter().zip(values) {
                            self.define(&p.name, v, &mut undo)?;
                        }
                    }
                    "ret" => {
                        let values = a.iter().map(|n| self.read(n)).collect::<Result<Vec<_>>>()?;
                        let active = self.state.control.pop().expect("return activation");
                        if let Some(segment) = active.segment {
                            self.segments[segment].end = Some(self.history.len());
                            self.segments[segment].results = values.clone();
                        }
                        if self.state.control.is_empty() {
                            self.state.finished = Some(values);
                        } else {
                            for (n, v) in active.bindings.iter().zip(values) {
                                self.define(n, v, &mut undo)?;
                            }
                        }
                    }
                    "uncall" => {
                        let name = a[0].trim_start_matches('@');
                        let args = list(&a[1])?
                            .iter()
                            .map(|n| self.read(n))
                            .collect::<Result<Vec<_>>>()?;
                        let results = list(&a[2])?
                            .iter()
                            .map(|n| self.read(n))
                            .collect::<Result<Vec<_>>>()?;
                        let segment = self
                            .segments
                            .iter()
                            .filter(|s| s.end.is_some() && !self.history[s.start].reversed)
                            .max_by_key(|s| s.end)
                            .cloned();
                        if let Some(s) = segment {
                            let end = s.end.expect("completed segment");
                            if s.callee != name
                                || s.args != args
                                || s.results != results
                                || self.history[end + 1..].iter().any(|f| !f.pure)
                                || self.history[s.start..=end].iter().any(|f| f.irreversible)
                            {
                                fault = Some(4);
                            } else {
                                let control = self.state.control.clone();
                                let next_id = self.state.next_id;
                                for n in (s.start..=end).rev() {
                                    self.undo_frame(n)?;
                                }
                                self.state.control = control;
                                self.state.next_id = next_id;
                                marker = true;
                            }
                        } else {
                            fault = Some(4);
                        }
                    }
                    "effect" => fault = self.effect(&a[0], &a[1..], &mut undo)?,
                    "trap" => fault = Some(4),
                    "hostcall" => {
                        return err("§10", "HOST_REQUEST/reply execution is not implemented yet")
                    }
                    "qreg.new" | "qreg.free" | "qreg.slice" | "qreg.concat" | "measure"
                    | "oracle_call" | "action_call" | "adj" | "ctl" => {
                        return err(
                            "§4.1",
                            "Q integration in classical history is not implemented yet",
                        )
                    }
                    _ => return err("§4.8", format!("unimplemented classical opcode {op}")),
                }
            }
            Ok(())
        })();
        if let Err(e) = execute {
            if e.0.starts_with("§7.4: instance") {
                fault = Some(4);
            } else {
                self.undo_record(undo)?;
                return Err(e);
            }
        }
        if let Some(code) = fault {
            self.state.fault = Some(code);
        } else if let Some(v) = output {
            self.define(&i.lhs[0], v, &mut undo)?;
        }
        self.history.push(HistoryFrame {
            undo,
            reversed: false,
            marker,
            pure: pure && fault.is_none(),
            irreversible: false,
        });
        Ok(())
    }
}

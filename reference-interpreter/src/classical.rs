//! CC/R dynamic SSA, retained history and reversible device models (§7).
use crate::{
    err,
    object::{is_binop, is_div, list, mask, parse_bits},
    oracle::Memory,
    registers::RegisterUndo,
    validation::normalize_call,
    Argument, Budget, Error, Instruction, Object, QuantumBindings, QuantumRegion, QuantumRegisters,
    Result, Retained, Type,
};
use std::collections::{BTreeMap, VecDeque};

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
/// One §7.3 effect-tape entry: `(dev, op, args)` and the byte string the
/// operation transferred (console output, clock bytes, entropy bytes, or a
/// 512-byte sector), recorded as the effect executed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TapeEntry {
    pub dev: String,
    pub op: String,
    pub args: Vec<u128>,
    pub bytes: Retained<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq, Default)]
struct Devices {
    console: Retained<u8>,
    clock: u64,
    entropy_position: usize,
    blocks: BTreeMap<u128, Vec<u8>>,
    tape: Retained<TapeEntry>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ClassicalSnapshot {
    instances: BTreeMap<Key, Vec<ClassicalValue>>,
    memory: Memory,
    devices: Devices,
    control: Vec<Activation>,
    next_id: u64,
    fault: Option<u128>,
    finished: Option<Retained<ClassicalValue>>,
    pending: Option<HostRequest>,
    quantum: QuantumRegisters,
}
/// One changed dynamic instance between two snapshots: the top instance of
/// `(activation, name)` before and after (either side absent when the
/// instance does not exist there, §7.4).
#[derive(Clone, Debug, PartialEq)]
pub struct RegisterDelta {
    pub activation: u64,
    pub name: String,
    pub pre: Option<ClassicalValue>,
    pub post: Option<ClassicalValue>,
}
/// The register and memory-byte changes between two snapshots; allocation,
/// device and handle changes are reported by `first_difference`, not here.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct SnapshotDelta {
    pub registers: Vec<RegisterDelta>,
    /// (address, before, after) for every byte whose value changed.
    pub memory: Vec<(u128, u8, u8)>,
}
impl ClassicalSnapshot {
    /// SHA-256 (hex) of the complete §7.1 memory: allocation table, heap
    /// bump and every nonzero byte, in sorted order.
    pub fn memory_digest(&self) -> Result<String> {
        let m = &self.memory;
        let mut bytes = String::new();
        let mut append = |line: String| -> Result<()> {
            bytes.try_reserve(line.len()).map_err(|e| {
                crate::resource_error("§7.1/§7.4", format!("memory digest bytes: {e}"))
            })?;
            bytes.push_str(&line);
            Ok(())
        };
        append(format!(
            "heap_base={};heap_used={};table_capacity={:?};",
            m.heap_base, m.heap_used, m.table_capacity
        ))?;
        for (name, base) in &m.addresses {
            append(format!("address {name}={base};"))?;
        }
        for a in &m.allocations {
            append(format!(
                "alloc base={} size={} live={} pinned={};",
                a.base, a.size, a.live, a.pinned
            ))?;
        }
        for (address, byte) in &m.bytes {
            append(format!("byte {address}={byte};"))?;
        }
        Ok(crate::sha256_hex(bytes.as_bytes()))
    }
    /// True iff the §7.1 memory components (bytes, allocations, heap bump)
    /// of the two snapshots are identical.
    pub fn memory_equals(&self, other: &Self) -> bool {
        self.memory == other.memory
    }
    /// The changes from `self` to `after`: every instance stack that differs
    /// and every memory byte that differs, in sorted order.
    pub fn delta(&self, after: &Self) -> Result<SnapshotDelta> {
        let mut registers = Vec::new();
        let keys = ordered_union(self.instances.keys(), after.instances.keys());
        for key in keys {
            let (pre_stack, post_stack) = (self.instances.get(key), after.instances.get(key));
            if pre_stack != post_stack {
                crate::reserve_more(&mut registers, 1, "§7.4", "snapshot register deltas")?;
                registers.push(RegisterDelta {
                    activation: key.0,
                    name: key.1.clone(),
                    pre: pre_stack.and_then(|s| s.last()).cloned(),
                    post: post_stack.and_then(|s| s.last()).cloned(),
                });
            }
        }
        let mut memory = Vec::new();
        let addresses = ordered_union(self.memory.bytes.keys(), after.memory.bytes.keys()).copied();
        for address in addresses {
            let (pre, post) = (self.memory.byte(address), after.memory.byte(address));
            if pre != post {
                crate::reserve_more(&mut memory, 1, "§7.4", "snapshot memory deltas")?;
                memory.push((address, pre, post));
            }
        }
        Ok(SnapshotDelta { registers, memory })
    }
    /// A description of the first component in which `other` differs from
    /// `self`, or None when the snapshots are equal. Components are compared
    /// in §3.2 order: instances, memory, devices, control, fault/finished,
    /// pending request, quantum registers.
    pub fn first_difference(&self, other: &Self) -> Option<String> {
        let keys = ordered_union(self.instances.keys(), other.instances.keys());
        for key in keys {
            if self.instances.get(key) != other.instances.get(key) {
                return Some(format!(
                    "instances of ({}, {:?}): {:?} differs from {:?}",
                    key.0,
                    key.1,
                    self.instances.get(key).and_then(|s| s.last()),
                    other.instances.get(key).and_then(|s| s.last())
                ));
            }
        }
        if self.memory != other.memory {
            let (a, b) = (&self.memory, &other.memory);
            let addresses = ordered_union(a.bytes.keys(), b.bytes.keys()).copied();
            for address in addresses {
                if a.byte(address) != b.byte(address) {
                    return Some(format!(
                        "memory byte at address {address}: {} differs from {}",
                        a.byte(address),
                        b.byte(address)
                    ));
                }
            }
            return Some(format!(
                "memory allocation state: heap_used {} / {} allocations differs from heap_used {} / {} allocations",
                a.heap_used,
                a.allocations.len(),
                b.heap_used,
                b.allocations.len()
            ));
        }
        if self.devices != other.devices {
            let (a, b) = (&self.devices, &other.devices);
            return Some(if a.console != b.console {
                format!(
                    "device console: {} bytes differs from {} bytes",
                    a.console.len(),
                    b.console.len()
                )
            } else if a.clock != b.clock {
                format!("device clock: {} differs from {}", a.clock, b.clock)
            } else if a.entropy_position != b.entropy_position {
                format!(
                    "device entropy position: {} differs from {}",
                    a.entropy_position, b.entropy_position
                )
            } else if a.tape != b.tape {
                format!(
                    "effect tape: {} entries differs from {} entries",
                    a.tape.len(),
                    b.tape.len()
                )
            } else {
                "device block sectors differ".to_string()
            });
        }
        if self.control != other.control || self.next_id != other.next_id {
            return Some(format!(
                "control: {} activations / next_id {} differs from {} activations / next_id {}",
                self.control.len(),
                self.next_id,
                other.control.len(),
                other.next_id
            ));
        }
        if self.fault != other.fault {
            return Some(format!(
                "fault: {:?} differs from {:?}",
                self.fault, other.fault
            ));
        }
        if self.finished != other.finished {
            return Some("finished results differ".to_string());
        }
        if self.pending != other.pending {
            return Some("pending host request differs".to_string());
        }
        if self.quantum != other.quantum {
            return Some(format!(
                "quantum registers: {} live wires differ",
                self.quantum.quantum_state().wire_count()
            ));
        }
        None
    }
}
#[derive(Clone, Debug)]
struct Undo {
    control: Vec<Activation>,
    next_id: u64,
    definitions: Vec<Key>,
    bytes: Retained<(u128, u8)>,
    allocation: Option<(u128, usize)>,
    free: Option<(usize, bool)>,
    device: Option<DeviceUndo>,
    tape: bool,
    fault: Option<u128>,
    finished: Option<Retained<ClassicalValue>>,
    pending: Option<HostRequest>,
    registers: Vec<RegisterUndo>,
    region: Option<RegionRecord>,
}
#[derive(Clone, Debug)]
struct RegionRecord {
    name: String,
    bindings: QuantumBindings,
    budget: Option<Budget>,
    adjoint: bool,
    controls: Vec<u64>,
    wire_next: u64,
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
    host: Option<HostExchange>,
    measurement: Option<(u64, Vec<bool>)>,
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
    pub values: Retained<ClassicalValue>,
    pub fault: Option<u128>,
    pub trapped: bool,
    pub steps: u64,
    pub request: Option<HostRequest>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostRequest(std::sync::Arc<HostRequestData>);
impl std::ops::Deref for HostRequest {
    type Target = HostRequestData;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct HostRequestData {
    pub service: String,
    pub payload: Retained<u8>,
    pub address: u128,
    pub capacity: u128,
    result_name: String,
}
/// The program point of one executed instruction: function, block, its
/// index within the block, and the instruction itself (§3.4, §9.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepPoint {
    pub function: String,
    pub block: String,
    pub index: usize,
    pub instruction: Instruction,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostExchange {
    pub request: HostRequest,
    pub reply: Retained<u8>,
}
pub struct ClassicalMachine<'a> {
    object: &'a Object,
    state: ClassicalSnapshot,
    history: Vec<HistoryFrame>,
    segments: Vec<Segment>,
    entropy: Vec<u8>,
    initial: ClassicalSnapshot,
    measurement_draws: VecDeque<f64>,
    measurement_rng: [u64; 4],
    idle_registers: Vec<u64>,
}

impl<'a> ClassicalMachine<'a> {
    pub fn new(
        object: &'a Object,
        entry: &str,
        args: &[ClassicalValue],
        entropy: &[u8],
    ) -> Result<Self> {
        Self::construct(object, entry, args, entropy, QuantumRegisters::new(), &[])
    }
    fn construct(
        object: &'a Object,
        entry: &str,
        args: &[ClassicalValue],
        entropy: &[u8],
        registers: QuantumRegisters,
        globals: &[(String, Vec<u8>)],
    ) -> Result<Self> {
        object.ensure_linked()?;
        let f = object
            .functions
            .get(entry)
            .filter(|f| matches!(f.kind.as_str(), "cc" | "r"))
            .ok_or_else(|| Error::new("§3.4: classical entry not found".into()))?;
        if args.len() != f.params[0].len()
            || args
                .iter()
                .zip(&f.params[0])
                .any(|(a, p)| !a.matches(&p.ty, object.ptr_width))
        {
            return err("§10", "classical entry argument type/count mismatch");
        }
        for (p, a) in f.params[0].iter().zip(args) {
            if let (Type::Qreg(n), ClassicalValue::Qreg(id)) = (&p.ty, a) {
                if registers.bundle(*id)?.len() as u64 != *n {
                    return err("§10", "E_ARG: qreg argument width differs");
                }
            }
        }
        let instances = f.params[0]
            .iter()
            .zip(args)
            .map(|(p, v)| ((0, p.name.clone()), vec![v.clone()]))
            .collect();
        let mut devices = Devices::default();
        if let Some(l) = object.section("layout") {
            if let Some(image) = l.object()?.get("block_image") {
                let data = object.data.get(image.string()?).ok_or_else(|| {
                    Error::new("§7.3/§9.1: block_image names undefined data".into())
                })?;
                if !data.len().is_multiple_of(512) {
                    return err("§7.3", "block image must have complete 512-byte sectors");
                }
                for (i, sector) in data.as_chunks::<512>().0.iter().enumerate() {
                    devices.blocks.insert(i as u128, sector.to_vec());
                }
            }
        }
        let mut memory = Memory::initial(object, f, &BTreeMap::new())?;
        // Caller-seeded global images (§7.1 entry state): each named global's
        // bytes replace its layout image, at exactly its declared size. The
        // seeded state is the run's entry state for reversal.
        let mut seeded = std::collections::BTreeSet::new();
        for (name, bytes) in globals {
            if !seeded.insert(name) {
                return err("§7.1", format!("global {name} seeded twice"));
            }
            let layout = object
                .section("layout")
                .ok_or_else(|| Error::new("§7.1: seeded globals require a layout".into()))?;
            let global = layout
                .get("globals")?
                .object()?
                .get(name)
                .ok_or_else(|| Error::new(format!("§7.1: unknown global {name}")))?;
            let base = global.get("base")?.uint()?;
            let size = global.get("size")?.uint()?;
            if size != bytes.len() as u128 {
                return err(
                    "§7.1",
                    format!(
                        "global {name} has size {size}, seeded with {} bytes",
                        bytes.len()
                    ),
                );
            }
            for (i, b) in bytes.iter().enumerate() {
                let address = base
                    .checked_add(i as u128)
                    .ok_or_else(|| Error::new("§7.1: global address overflow".into()))?;
                memory.set_byte(address, *b)?;
            }
        }
        let state = ClassicalSnapshot {
            instances,
            memory,
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
            pending: None,
            quantum: registers,
        };
        Ok(Self {
            object,
            state: state.clone(),
            history: Vec::new(),
            segments: Vec::new(),
            entropy: entropy.into(),
            initial: state,
            measurement_draws: VecDeque::new(),
            measurement_rng: [1, 2, 3, 4],
            idle_registers: Vec::new(),
        })
    }
    pub fn with_quantum_registers(
        object: &'a Object,
        entry: &str,
        args: &[ClassicalValue],
        entropy: &[u8],
        registers: QuantumRegisters,
    ) -> Result<Self> {
        Self::construct(object, entry, args, entropy, registers, &[])
    }
    /// Like `with_quantum_registers`, with §7.1 global images seeded by the
    /// caller (name, exact bytes) as part of the entry state.
    pub fn with_quantum_registers_and_globals(
        object: &'a Object,
        entry: &str,
        args: &[ClassicalValue],
        entropy: &[u8],
        registers: QuantumRegisters,
        globals: &[(String, Vec<u8>)],
    ) -> Result<Self> {
        Self::construct(object, entry, args, entropy, registers, globals)
    }
    pub fn quantum_registers(&self) -> &QuantumRegisters {
        &self.state.quantum
    }
    pub fn designate_idle_registers(&mut self, ids: &[u64]) -> Result<()> {
        if !self.history.is_empty() {
            return err("§9.4", "idle set must be designated before execution");
        }
        let mut wires = std::collections::BTreeSet::new();
        for id in ids {
            for wire in self.state.quantum.idle_bundle(*id)? {
                if !wires.insert(*wire) {
                    return err("§9.4", "designated idle registers overlap");
                }
            }
        }
        self.idle_registers = ids.to_vec();
        Ok(())
    }
    pub fn push_measurement_draw(&mut self, draw: f64) -> Result<()> {
        if !(0.0..1.0).contains(&draw) {
            return err("§14.2", "measurement draw must be finite in [0,1)");
        }
        self.measurement_draws.push_back(draw);
        Ok(())
    }
    pub fn measurements(&self) -> Vec<(u64, Vec<bool>)> {
        self.history
            .iter()
            .filter_map(|h| h.measurement.clone())
            .collect()
    }
    fn measurement_draw(&mut self) -> f64 {
        if let Some(draw) = self.measurement_draws.pop_front() {
            return draw;
        }
        let s = &mut self.measurement_rng;
        let result = s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = s[1] << 17;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);
        ((result >> 11) as f64) * (1.0 / 9_007_199_254_740_992.0)
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
    /// The §7.3 effect tape in execution order.
    pub fn effect_tape(&self) -> &[TapeEntry] {
        &self.state.devices.tape
    }
    /// SHA-256 (hex) of the current §7.1 memory (see
    /// `ClassicalSnapshot::memory_digest`).
    pub fn memory_digest(&self) -> Result<String> {
        self.state.memory_digest()
    }
    /// True while another instruction can execute: no fault record, no
    /// finished results and no unanswered host request (§10 run states).
    pub fn is_running(&self) -> bool {
        self.state.fault.is_none()
            && self.state.finished.is_none()
            && self.state.pending.is_none()
            && !self.state.control.is_empty()
    }
    /// The outcome so far: finished values, the fault record if any, the
    /// pending host request if any, and the executed-instruction count
    /// (one retained history frame per step, §7.4).
    pub fn outcome(&self) -> ClassicalOutcome {
        ClassicalOutcome {
            values: self.state.finished.clone().unwrap_or_default(),
            fault: self.state.fault,
            trapped: self.state.fault.is_some()
                && self
                    .state
                    .control
                    .last()
                    .is_some_and(|a| self.object.functions[&a.function].kind == "cc"),
            steps: self.history.len() as u64,
            request: self.state.pending.clone(),
        }
    }
    /// Executes one instruction (§3.4) and returns the point it occupied,
    /// or None when the machine is not running (faulted, finished, or
    /// suspended at a host request).
    pub fn step_once(&mut self) -> Result<Option<StepPoint>> {
        if !self.is_running() {
            return Ok(None);
        }
        let current = self.state.control.last().expect("running control").clone();
        let function = &self.object.functions[&current.function];
        let block = &function.blocks[current.block];
        let instruction = block
            .instructions
            .get(current.ip)
            .ok_or_else(|| Error::new("§3.4: fell off classical block".into()))?
            .clone();
        let point = StepPoint {
            function: current.function.clone(),
            block: block.name.clone(),
            index: current.ip,
            instruction,
        };
        self.step()?;
        Ok(Some(point))
    }
    pub fn memory_bytes(&self, address: u128, len: usize) -> Result<Vec<u8>> {
        self.transfer_bytes(address, len, "§7.1")
    }
    fn transfer_bytes(&self, address: u128, len: usize, rule: &str) -> Result<Vec<u8>> {
        if !self.state.memory.valid(address, len as u128) {
            return err("§7.1", "memory query outside live allocation");
        }
        let mut bytes = crate::reserved(len, rule, "§7.1 transfer bytes")?;
        bytes.extend((0..len).map(|n| self.state.memory.byte(address + n as u128)));
        Ok(bytes)
    }
    pub fn host_exchanges(&self) -> impl Iterator<Item = &HostExchange> {
        self.history.iter().filter_map(|h| h.host.as_ref())
    }
    pub fn host_transcript(&self) -> Result<Vec<HostExchange>> {
        let mut out = crate::reserved(
            self.host_exchanges().count(),
            "§10/§7.4",
            "host transcript entries",
        )?;
        out.extend(self.host_exchanges().cloned());
        Ok(out)
    }
    pub fn reply(&mut self, payload: &[u8]) -> Result<()> {
        let request = self
            .state
            .pending
            .clone()
            .ok_or_else(|| Error::new("§10: E_INVALID_STATE: no host request".into()))?;
        if payload.len() as u128 & !mask(self.object.ptr_width) != 0 {
            return err("§10", "E_ARG: reply length exceeds ptr_width");
        }
        let count = (payload.len() as u128).min(request.capacity) as usize;
        if self.history.last().and_then(|f| f.host.as_ref()).is_none() {
            return err("§10/§7.4", "missing retained host request");
        }
        let mut undo = self
            .history
            .last()
            .ok_or_else(|| Error::new("§10: missing host frame".into()))?
            .undo
            .clone();
        let mut reply = crate::reserved(payload.len(), "§10", "retained host reply bytes")?;
        reply.extend_from_slice(payload);
        self.save_bytes(request.address, count, &mut undo)?;
        let mut memory = self.state.memory.clone();
        memory
            .bytes
            .prepare(payload[..count].iter().filter(|b| **b != 0).count())?;
        for (n, b) in payload[..count].iter().enumerate() {
            memory.set_byte(request.address + n as u128, *b)?;
        }
        let reply = Retained::from(reply);
        crate::reserve_more(
            &mut undo.definitions,
            1,
            "§10/§7.4",
            "host result retention",
        )?;
        self.define(
            &request.result_name,
            ClassicalValue::Int(self.object.ptr_width, payload.len() as u128),
            &mut undo,
        )?;
        let frame = self
            .history
            .last_mut()
            .ok_or_else(|| Error::new("§10: missing retained host frame".into()))?;
        self.state.memory = memory;
        frame.undo = undo;
        frame
            .host
            .as_mut()
            .ok_or_else(|| Error::new("§10: missing retained host request".into()))?
            .reply = reply;
        self.state.pending = None;
        Ok(())
    }
    pub fn run(&mut self, max_steps: u64) -> Result<ClassicalOutcome> {
        let mut steps = 0;
        while self.state.fault.is_none()
            && self.state.finished.is_none()
            && self.state.pending.is_none()
        {
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
            request: self.state.pending.clone(),
        })
    }
    pub fn reverse(&mut self) -> Result<()> {
        if self.state.pending.is_some() {
            return err("§10", "E_INVALID_STATE: host request is pending");
        }
        if self.history.iter().any(|f| f.irreversible) {
            return err(
                "§10",
                "E_NOT_REVERSIBLE: measurement or unverified release in history",
            );
        }
        for n in (0..self.history.len()).rev() {
            self.undo_frame(n)?;
        }
        // History traversal undoes each frame's effects; reversed frames and
        // uncall markers are skipped (§7.4). Control is administrative, not an
        // effect: whole-run reversal ends at the run's entry point, i.e. the
        // control recorded before the first frame, even when that frame opened
        // a segment cancelled by a later uncall (codex B09).
        if let Some(first) = self.history.first() {
            let entry = first.undo.clone();
            self.state.control = entry.control;
            self.state.next_id = entry.next_id;
            self.state.fault = entry.fault;
            self.state.finished = entry.finished;
            self.state.pending = entry.pending;
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
            .ok_or_else(|| Error::new("§3.4: empty activation stack".into()))?
            .id;
        self.state
            .instances
            .get(&(id, name.into()))
            .and_then(|v| v.last())
            .cloned()
            .ok_or_else(|| {
                trap(Error::new(format!(
                    "§7.4: instance {name} does not exist (uncalled or not yet defined)"
                )))
            })
    }
    fn bits(&self, name: &str) -> Result<u128> {
        self.read(name)?.bits()
    }
    fn qreg(&self, name: &str) -> Result<u64> {
        match self.read(name)? {
            ClassicalValue::Qreg(id) => {
                // §3.3: a handle precondition that fails is a CC trap.
                self.state.quantum.bundle(id).map_err(trap)?;
                Ok(id)
            }
            _ => err("§3.3", "region operand is not a qreg"),
        }
    }
    fn region_record(&self, instruction: &Instruction) -> Result<(RegionRecord, Vec<u64>)> {
        let call = normalize_call(instruction)?;
        let a = &call.instruction.operands;
        let name = a[0].trim_start_matches('@');
        let callee = self
            .object
            .functions
            .get(name)
            .ok_or_else(|| Error::new(format!("§9.3: unresolved region @{name}")))?;
        let mut bindings = QuantumBindings::default();
        let mut ids = Vec::new();
        let mut controls = Vec::new();
        if let Some(control) = call.control {
            let id = self.qreg(&control)?;
            let wires = self.state.quantum.bundle(id)?;
            if wires.len() != 1 {
                return err("§3.3", "ctl requires qreg[1]");
            }
            controls.push(wires[0]);
            ids.push(id);
        }
        let oracle = call.instruction.op == "oracle_call";
        let (groups, memory, args, budget) = if oracle {
            let token = match self.read(&a[1])? {
                ClassicalValue::Token(b) => b,
                _ => return err("§3.3", "oracle_call needs a budget token"),
            };
            (
                vec![
                    (0, list(&a[2])?),
                    (1, vec![a[3].clone()]),
                    (2, vec![a[4].clone()]),
                ],
                (3, list(&a[5])?),
                (4, list(&a[6])?),
                Some(token),
            )
        } else {
            (
                vec![(0, list(&a[1])?), (1, list(&a[2])?)],
                (2, list(&a[3])?),
                (3, list(&a[4])?),
                None,
            )
        };
        // Check the physical interface BEFORE deduplicating lifecycle handles.
        // Distinct SSA names (e.g. select results or block parameters) can bind
        // the very same register. Controls may share read-only inputs, so they
        // are checked separately against actual targets below.
        let mut interface_wires = std::collections::BTreeSet::new();
        let mut bind_wires = |wires: &[crate::quantum::Wire]| -> Result<()> {
            for w in wires {
                if !interface_wires.insert(*w) {
                    return Err(trap(Error::new(
                        "§3.3: region interface bundles and regions overlap".into(),
                    )));
                }
            }
            Ok(())
        };
        for (group, names) in groups {
            for (p, n) in callee.params[group].iter().zip(names) {
                let id = self.qreg(&n)?;
                let wires = self.state.quantum.bundle(id)?.to_vec();
                bind_wires(&wires)?;
                bindings.bundles.insert(p.name.clone(), wires);
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
        }
        for (p, n) in callee.params[memory.0].iter().zip(memory.1) {
            let id = self.qreg(&n)?;
            let wires = self.state.quantum.bundle(id)?;
            let Type::Region(a, w) = p.ty else {
                return err("§5.1", "mem slot requires region type");
            };
            if crate::oracle::power_of_two(a as u128).and_then(|cells| cells.checked_mul(w as u128))
                != Some(wires.len() as u128)
            {
                return err("§3.3", "qreg does not fill region dimensions");
            }
            bind_wires(wires)?;
            bindings.regions.insert(
                p.name.clone(),
                QuantumRegion {
                    address_width: a,
                    cell_width: w,
                    cells: wires.chunks_exact(w as usize).map(|c| c.to_vec()).collect(),
                },
            );
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
        let mut targets = std::collections::BTreeSet::new();
        if oracle {
            for p in callee.params[1].iter().chain(&callee.params[2]) {
                targets.extend(bindings.bundles[&p.name].iter().copied());
            }
        } else if !controls.is_empty() {
            // The same transitive bit-level target analysis used at load,
            // now bound to physical wire ids. This includes diagonal targets
            // and targets reached through nested action calls.
            for (param, index) in self
                .object
                .interface_targets(name, &mut Default::default())?
            {
                targets.insert(bindings.bundles[&param][index as usize]);
            }
            for p in &callee.params[memory.0] {
                if p.role.as_deref() == Some("mut") {
                    targets.extend(bindings.regions[&p.name].cells.iter().flatten().copied());
                }
            }
        }
        if controls.iter().any(|w| targets.contains(w)) {
            return Err(trap(Error::new(
                "§3.3: effective control aliases region target".into(),
            )));
        }
        for (p, n) in callee.params[args.0].iter().zip(args.1) {
            let arg = match self.read(&n)? {
                ClassicalValue::Int(_, v) | ClassicalValue::Ptr(v) => Argument::Bits(v),
                ClassicalValue::F64(bits) => Argument::Float(f64::from_bits(bits)),
                ClassicalValue::Token(b) => Argument::Token(b),
                _ => return err("§3.3", "qreg is not a classical region argument"),
            };
            bindings.args.insert(p.name.clone(), arg);
        }
        let used = ids
            .iter()
            .map(|id| self.state.quantum.bundle(*id))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        for id in &self.idle_registers {
            let wires = self.state.quantum.idle_bundle(*id)?.to_vec();
            if wires.iter().any(|w| used.contains(w)) {
                return err(
                    "§9.4",
                    "designated idle wires overlap the region interface or control",
                );
            }
            bindings.borrowed.push(wires);
        }
        Ok((
            RegionRecord {
                name: name.into(),
                bindings,
                budget,
                adjoint: call.adjoint,
                controls,
                wire_next: self.state.quantum.state.next_wire_id(),
            },
            ids,
        ))
    }
    fn execute_region(&mut self, record: &RegionRecord, inverse: bool) -> Result<()> {
        let callee = &self.object.functions[&record.name];
        let mut state = self.state.quantum.state.clone();
        if let Some(budget) = record.budget {
            let ins = callee.params[0]
                .iter()
                .map(|p| record.bindings.bundles[&p.name].clone())
                .collect::<Vec<_>>();
            let target = &record.bindings.bundles[&callee.params[1][0].name];
            let fault = &record.bindings.bundles[&callee.params[2][0].name];
            let args = callee.params[4]
                .iter()
                .map(|p| record.bindings.args[&p.name].clone())
                .collect::<Vec<_>>();
            self.object.oracle_quantum(
                callee,
                &mut state,
                &ins,
                target,
                fault,
                &record.bindings.regions,
                &args,
                budget,
                &record
                    .controls
                    .iter()
                    .map(|w| (*w, true))
                    .collect::<Vec<_>>(),
            )?;
        } else {
            self.object.execute_action(
                &record.name,
                &mut state,
                &record.bindings,
                record.adjoint ^ inverse,
                &record.controls,
            )?;
        }
        self.state.quantum.state = state;
        Ok(())
    }
    fn define(&mut self, name: &str, value: ClassicalValue, undo: &mut Undo) -> Result<()> {
        let id = self
            .state
            .control
            .last()
            .ok_or_else(|| Error::new("§3.4: define without activation".into()))?
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
    fn save_bytes(&self, p: u128, len: usize, undo: &mut Undo) -> Result<()> {
        let bytes = undo
            .bytes
            .prepare(len, "§10/§7.1/§7.4", "retained memory bytes")?;
        for n in 0..len {
            bytes.push((p + n as u128, self.state.memory.byte(p + n as u128)));
        }
        Ok(())
    }
    fn undo_record(&mut self, undo: Undo) -> Result<()> {
        if !undo.bytes.is_empty() {
            self.state
                .memory
                .bytes
                .prepare(undo.bytes.iter().filter(|(_, b)| *b != 0).count())?;
        }
        for record in undo.registers.into_iter().rev() {
            self.state.quantum.undo(record)?;
        }
        if let Some(record) = undo.region {
            self.execute_region(&record, true)?;
            self.state
                .quantum
                .state
                .restore_next_wire_id(record.wire_next)?;
        }
        for key in undo.definitions.into_iter().rev() {
            let stack = self
                .state
                .instances
                .get_mut(&key)
                .ok_or_else(|| Error::new("§7.4: missing retained dynamic instance".into()))?;
            stack
                .pop()
                .ok_or_else(|| Error::new("§7.4: empty retained dynamic instance".into()))?;
            if stack.is_empty() {
                self.state.instances.remove(&key);
            }
        }
        for &(p, b) in undo.bytes.iter().rev() {
            self.state.memory.set_byte(p, b)?;
        }
        if let Some((used, len)) = undo.allocation {
            self.state.memory.heap_used = used;
            self.state.memory.allocations.truncate(len);
        }
        if let Some((n, live)) = undo.free {
            self.state.memory.allocations[n].live = live;
        }
        if undo.tape {
            let len = self.state.devices.tape.len();
            if len == 0 {
                return err("§7.3", "empty effect tape on reversal");
            }
            self.state.devices.tape.truncate(len - 1);
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
        self.state.pending = undo.pending;
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
        let len = usize::try_from(len).map_err(|_| {
            Error::host_resource(
                "§7.3: host limit: device length exceeds host address space".into(),
            )
        })?;
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
        self.state
            .devices
            .tape
            .prepare(0, "§7.3/§7.4", "effect tape")?
            .try_reserve_exact(1)
            .map_err(|e| crate::resource_error("§7.3/§7.4", format!("effect tape: {e}")))?;
        let transferred = match (dev, operation) {
            ("console", "write") => {
                let bytes = self.transfer_bytes(p, len, "§7.3")?;
                let old_len = self.state.devices.console.len();
                self.state
                    .devices
                    .console
                    .prepare(len, "§7.3", "console bytes")?
                    .extend_from_slice(&bytes);
                undo.device = Some(DeviceUndo::Console(old_len));
                bytes
            }
            ("clock", "read") => {
                self.save_bytes(p, 8, undo)?;
                let c = self.state.devices.clock;
                undo.device = Some(DeviceUndo::Clock(c));
                self.state.memory.store_u128(p, 64, c as u128)?;
                self.state.devices.clock = c.wrapping_add(1);
                c.to_le_bytes().to_vec()
            }
            ("entropy", "read") => {
                self.save_bytes(p, len, undo)?;
                let start = self.state.devices.entropy_position;
                let mut bytes = crate::reserved(len, "§7.3", "entropy transfer bytes")?;
                bytes.extend_from_slice(&self.entropy[start..start + len]);
                self.state
                    .memory
                    .bytes
                    .prepare(bytes.iter().filter(|b| **b != 0).count())?;
                undo.device = Some(DeviceUndo::Entropy(start));
                for n in 0..len {
                    self.state
                        .memory
                        .set_byte(p + n as u128, self.entropy[start + n])?;
                }
                self.state.devices.entropy_position += len;
                bytes
            }
            ("block", "read") => {
                self.save_bytes(p, 512, undo)?;
                let bytes = self
                    .state
                    .devices
                    .blocks
                    .get(&values[0])
                    .cloned()
                    .unwrap_or_else(|| vec![0; 512]);
                self.state
                    .memory
                    .bytes
                    .prepare(bytes.iter().filter(|b| **b != 0).count())?;
                for (n, b) in bytes.iter().enumerate() {
                    self.state.memory.set_byte(p + n as u128, *b)?;
                }
                bytes
            }
            ("block", "write") => {
                let sector = values[0];
                let old = self.state.devices.blocks.get(&sector).cloned();
                undo.device = Some(DeviceUndo::Block(sector, old));
                let bytes: Vec<_> = (0..512).map(|n| self.state.memory.byte(p + n)).collect();
                self.state.devices.blocks.insert(sector, bytes.clone());
                bytes
            }
            _ => {
                return err(
                    "§7.3",
                    format!("unknown device operation {dev}.{operation}"),
                )
            }
        };
        self.state
            .devices
            .tape
            .prepare(1, "§7.3/§7.4", "effect tape")?
            .push(TapeEntry {
                dev: dev.into(),
                op: operation.into(),
                args: values,
                bytes: transferred.into(),
            });
        undo.tape = true;
        Ok(None)
    }
    fn step(&mut self) -> Result<()> {
        let object = self.object;
        let current = self
            .state
            .control
            .last()
            .ok_or_else(|| Error::new("§3.4: no active function".into()))?
            .clone();
        let f = &object.functions[&current.function];
        let i = f.blocks[current.block]
            .instructions
            .get(current.ip)
            .ok_or_else(|| Error::new("§3.4: fell off classical block".into()))?
            .clone();
        let op = i.op.as_str();
        let a = &i.operands;
        let mut undo = Undo {
            control: self.state.control.clone(),
            next_id: self.state.next_id,
            definitions: Vec::new(),
            bytes: Retained::default(),
            allocation: None,
            free: None,
            device: None,
            tape: false,
            fault: self.state.fault,
            finished: self.state.finished.clone(),
            pending: self.state.pending.clone(),
            registers: Vec::new(),
            region: None,
        };
        crate::reserve_more(&mut self.history, 1, "§10/§7.4", "retained history frames")?;
        let mut pending = None;
        self.state
            .control
            .last_mut()
            .ok_or_else(|| Error::new("§3.4: no active function".into()))?
            .ip += 1;
        let mut fault = None;
        let mut output = None;
        let mut marker = false;
        let mut measurement = None;
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
                    Some(binop(op, left, right, w)?)
                } else {
                    match divide(op, left, right, w)? {
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
                    )? as u128,
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
                let w = Type::parse(t)?.width(object.ptr_width)? as u16;
                let p = self.bits(&a[0])?;
                if !self.state.memory.valid(p, (w / 8) as u128) {
                    fault = Some(7);
                } else {
                    output = Some(ClassicalValue::Int(
                        w,
                        self.state.memory.load_u128(p, w as u64)?,
                    ));
                }
            } else if let Some(t) = op.strip_prefix("store.") {
                let w = Type::parse(t)?.width(object.ptr_width)? as u16;
                let p = self.bits(&a[0])?;
                let v = self.bits(&a[1])?;
                if !self.state.memory.valid(p, (w / 8) as u128) {
                    fault = Some(7);
                } else {
                    self.save_bytes(p, (w / 8) as usize, &mut undo)?;
                    self.state.memory.store_u128(p, w as u64, v)?;
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
                        if self.state.memory.allocation_fails(n, b.heap) {
                            fault = Some(2);
                        } else {
                            undo.allocation = Some((
                                self.state.memory.heap_used,
                                self.state.memory.allocations.len(),
                            ));
                            output = Some(ClassicalValue::Ptr(self.state.memory.allocate(n)?));
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
                            .ok_or_else(|| Error::new("§4.1: malformed branch".into()))?;
                        let values = list(&format!("({args}"))?
                            .iter()
                            .map(|n| self.read(n))
                            .collect::<Result<Vec<_>>>()?;
                        let block = f.blocks.iter().position(|b| b.name == n).ok_or_else(|| {
                            Error::new(format!("§3.4: unknown branch target {n}"))
                        })?;
                        for (p, v) in f.blocks[block].params.iter().zip(values) {
                            self.define(&p.name, v, &mut undo)?;
                        }
                        let active = self
                            .state
                            .control
                            .last_mut()
                            .ok_or_else(|| Error::new("§3.4: no active function".into()))?;
                        active.block = block;
                        active.ip = 0;
                    }
                    "call" => {
                        let n = a[0].trim_start_matches('@');
                        let callee = &object.functions[n];
                        let args = a[1..].to_vec();
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
                        let active =
                            self.state.control.pop().ok_or_else(|| {
                                Error::new("§3.4: ret without an activation".into())
                            })?;
                        if let Some(segment) = active.segment {
                            self.segments[segment].end = Some(self.history.len());
                            self.segments[segment].results = values.clone();
                        }
                        if self.state.control.is_empty() {
                            self.state.finished = Some(values.into());
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
                            let end = s.end.ok_or_else(|| {
                                Error::new("§7.4: segment is not complete".into())
                            })?;
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
                        let address = self.bits(&a[1])?;
                        let capacity = self.bits(&a[2])?;
                        if !self.state.memory.valid(address, capacity) {
                            fault = Some(7);
                        } else {
                            let count = usize::try_from(capacity).map_err(|_| {
                                Error::host_resource(
                                    "§10: host limit: request exceeds host address space".into(),
                                )
                            })?;
                            let payload = self.transfer_bytes(address, count, "§10")?;
                            pending = Some(HostRequest(std::sync::Arc::new(HostRequestData {
                                service: crate::object::literal_name(&a[0])?,
                                payload: payload.into(),
                                address,
                                capacity,
                                result_name: i.lhs[0].clone(),
                            })));
                        }
                    }
                    "qreg.new" => {
                        let n = a[1].parse::<usize>().map_err(|_| {
                            Error::new("§3.3: qreg size exceeds host storage".into())
                        })?;
                        let (id, record) = self.state.quantum.allocate_recorded(n)?;
                        undo.registers.push(record);
                        output = Some(ClassicalValue::Qreg(id));
                    }
                    "qreg.free" => {
                        let id = self.qreg(&a[0])?;
                        undo.registers
                            .push(self.state.quantum.free_recorded(id).map_err(trap)?);
                    }
                    "qreg.slice" => {
                        let id = self.qreg(&a[0])?;
                        let lo = a[1].parse::<usize>().map_err(|_| {
                            Error::new("§3.3: slice offset exceeds host storage".into())
                        })?;
                        let len = a[2].parse::<usize>().map_err(|_| {
                            Error::new("§3.3: slice length exceeds host storage".into())
                        })?;
                        let (id, record) = self
                            .state
                            .quantum
                            .slice_recorded(id, lo, len)
                            .map_err(trap)?;
                        undo.registers.push(record);
                        output = Some(ClassicalValue::Qreg(id));
                    }
                    "qreg.concat" => {
                        let a = self.qreg(&a[0])?;
                        let b = self.qreg(&i.operands[1])?;
                        let (id, record) =
                            self.state.quantum.concat_recorded(a, b).map_err(trap)?;
                        undo.registers.push(record);
                        output = Some(ClassicalValue::Qreg(id));
                    }
                    "measure" => {
                        let id = self.qreg(&a[0])?;
                        let draw = self.measurement_draw();
                        let bits = self.state.quantum.measure(id, draw).map_err(trap)?;
                        let value = bits
                            .iter()
                            .enumerate()
                            .fold(0, |v, (i, b)| v | ((*b as u128) << i));
                        output = Some(ClassicalValue::Int(bits.len() as u16, value));
                        measurement = Some((id, bits));
                    }
                    "oracle_call" | "action_call" | "adj" | "ctl" => {
                        let (record, ids) = self.region_record(&i)?;
                        let mut trial = self.state.quantum.clone();
                        trial.complete_call(&ids).map_err(trap)?;
                        self.execute_region(&record, false)?;
                        undo.registers
                            .push(self.state.quantum.complete_call_recorded(&ids)?);
                        undo.region = Some(record);
                    }
                    _ => return err("§4.8", format!("unimplemented classical opcode {op}")),
                }
            }
            Ok(())
        })();
        if let Err(e) = execute {
            // Only conditions the specification makes a CC trap / R fault
            // FAULT_UB_TRAP are marked by `trap`; every other error is an
            // internal failure and propagates (fail loud, fable C5).
            if e.0.starts_with(TRAP) {
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
        let host = pending.clone().map(|request| HostExchange {
            request,
            reply: Retained::default(),
        });
        self.state.pending = pending;
        self.history.push(HistoryFrame {
            undo,
            reversed: false,
            marker,
            pure: pure && fault.is_none(),
            irreversible: measurement.is_some(),
            measurement,
            host,
        });
        Ok(())
    }
}

/// CC/R integer arithmetic on iN (N <= 128) and ptr (ptr_width <= 128) values (§4.1).
pub(crate) fn signed(v: u128, width: u16) -> i128 {
    let shift = 128 - width as u32;
    ((v << shift) as i128) >> shift
}
pub(crate) fn binop(op: &str, a: u128, b: u128, w: u16) -> Result<u128> {
    let shift_out = b >= w as u128;
    let v = match op {
        "add" => a.wrapping_add(b),
        "sub" => a.wrapping_sub(b),
        "mul" => a.wrapping_mul(b),
        "and" => a & b,
        "or" => a | b,
        "xor" => a ^ b,
        // §4.1: "wrap; shift >= N -> 0" for all three shifts.
        "shl" if shift_out => 0,
        "lshr" if shift_out => 0,
        "ashr" if shift_out => 0,
        "shl" => a << (b as u32),
        "lshr" => a >> (b as u32),
        "ashr" => (signed(a, w) >> (b as u32)) as u128,
        _ => return err("§4.8", format!("unknown binop {op}")),
    };
    Ok(v & mask(w))
}
/// Ok(Err(code)) is the trap / fault record of §4.1.
pub(crate) fn divide(
    op: &str,
    a: u128,
    b: u128,
    w: u16,
) -> Result<std::result::Result<u128, u128>> {
    if b == 0 {
        return Ok(Err(5));
    }
    if op.starts_with('s') && a == 1u128 << (w - 1) && b == mask(w) {
        return Ok(Err(6));
    }
    Ok(Ok(match op {
        "udiv" => a / b,
        "urem" => a % b,
        "sdiv" => (signed(a, w) / signed(b, w)) as u128,
        "srem" => (signed(a, w) % signed(b, w)) as u128,
        _ => return err("§4.8", format!("unknown division {op}")),
    } & mask(w)))
}
pub(crate) fn compare(pred: &str, a: u128, b: u128, w: u16) -> Result<bool> {
    Ok(match pred {
        "eq" => a == b,
        "ne" => a != b,
        "ult" => a < b,
        "ule" => a <= b,
        "slt" => signed(a, w) < signed(b, w),
        "sle" => signed(a, w) <= signed(b, w),
        _ => return err("§4.8", format!("unknown comparison {pred}")),
    })
}

/// Marks an error as the CC trap / R fault FAULT_UB_TRAP the specification
/// defines for it (§3.3 handle preconditions, §7.4 dead instances).
const TRAP: &str = "trap ";
fn trap(e: Error) -> Error {
    if e.kind() == crate::ErrorKind::HostResource {
        e
    } else {
        Error::new(format!("{TRAP}{}", e.0))
    }
}

#[cfg(test)]
pub(crate) fn r21_resource_cases() {
    let o = Object::parse(include_str!("../tests/fixtures/r21/cc-length.bisa")).unwrap();
    let mut m = ClassicalMachine::new(&o, "f", &[], &[]).unwrap();
    m.step_once().unwrap();
    m.step_once().unwrap();
    for len in [0, 1, 8, (1usize << 63) - 1, 1 << 63, usize::MAX] {
        let mut undo = m.history.last().unwrap().undo.clone();
        undo.bytes = Retained::default();
        let result = m.save_bytes(1, len, &mut undo);
        if len <= 8 {
            result.unwrap();
            assert_eq!(undo.bytes.len(), len);
        } else {
            let e = result.unwrap_err().to_string();
            assert!(e.contains("§7.1") && e.contains("host limit"), "{e}");
            assert!(undo.bytes.is_empty());
        }
    }
    eprintln!("R21 retained memory undo bytes: 1 path / 6 cases");
}

fn ordered_union<T: Ord>(
    a: impl Iterator<Item = T>,
    b: impl Iterator<Item = T>,
) -> impl Iterator<Item = T> {
    let (mut a, mut b) = (a.peekable(), b.peekable());
    std::iter::from_fn(move || match (a.peek(), b.peek()) {
        (Some(x), Some(y)) => match x.cmp(y) {
            std::cmp::Ordering::Less => a.next(),
            std::cmp::Ordering::Greater => b.next(),
            std::cmp::Ordering::Equal => {
                b.next();
                a.next()
            }
        },
        (Some(_), None) => a.next(),
        (None, _) => b.next(),
    })
}

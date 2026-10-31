//! §14.1 sequential reference interpreter. It uses independent dynamic
//! activations rather than the reversible machine's global location file and
//! frame-slot exchanges, so the two engines check each other (deliverable 4).
//!
//! Every location, pseudo-location and cell is a `BitVector` of its declared
//! width: §3.3 bounds only `iN` (1..=128), never `wire[N]` or `reg[N]`.
//! Addresses are `reg[ptr_width]` values, ptr_width <= 128 (SPEC-QUESTIONS 14),
//! and memory/allocator arithmetic is exact (no wrap, §3.3, §3.7).
use crate::{
    err,
    object::{integer, is_binop, is_div, list},
    BitVector, Error, Function, Instruction, Object, Result, Type,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Budget {
    pub steps: u64,
    pub heap: u128,
    pub stack: u128,
    pub depth: u128,
}
impl Budget {
    pub fn steps(steps: u64) -> Self {
        Self {
            steps,
            heap: 0,
            stack: 0,
            depth: 0,
        }
    }
}
/// A classical argument: iN (N <= 128) and ptr (ptr_width <= 128) values,
/// f64 angles and budget tokens (§3.3).
#[derive(Clone, Debug, PartialEq)]
pub enum Argument {
    Bits(u128),
    Float(f64),
    Token(Budget),
}
/// Small-integer convenience form of an oracle input. Each value must fit its
/// declared bundle; wider interfaces use `OracleVectorInput` directly.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OracleInput {
    pub ins: Vec<u128>,
    pub args: Vec<Argument>,
    pub regions: BTreeMap<String, Vec<u8>>,
}
impl OracleInput {
    pub fn bits(ins: &[u128]) -> Self {
        Self {
            ins: ins.into(),
            ..Self::default()
        }
    }
}
/// Small-integer convenience form of F_B. Conversion fails loudly when the
/// payload or code does not fit u128; `OracleVectorOutcome` is exact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OracleOutcome {
    pub payload: u128,
    pub code: u128,
    pub steps: u64,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OracleVectorInput {
    pub ins: Vec<BitVector>,
    pub args: Vec<Argument>,
    pub regions: BTreeMap<String, Vec<u8>>,
}
impl OracleVectorInput {
    pub fn from_input(object: &Object, name: &str, input: &OracleInput) -> Result<Self> {
        let f = object
            .functions
            .get(name)
            .filter(|f| f.kind == "oracle")
            .ok_or_else(|| Error::new(format!("§5.2: unknown oracle @{name}")))?;
        if input.ins.len() != f.params[0].len() {
            return err("§5.1", "oracle input count mismatch");
        }
        let ins = input
            .ins
            .iter()
            .zip(&f.params[0])
            .map(|(v, p)| {
                let width = p.ty.bits(object.ptr_width)?;
                if width < 128 && *v >> width != 0 {
                    return err("§3.3", "input value exceeds declared width");
                }
                BitVector::from_u128(width, *v)
            })
            .collect::<Result<_>>()?;
        Ok(Self {
            ins,
            args: input.args.clone(),
            regions: input.regions.clone(),
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OracleVectorOutcome {
    pub payload: BitVector,
    pub code: BitVector,
    pub steps: u64,
}
impl OracleVectorOutcome {
    pub fn narrow(&self) -> Result<OracleOutcome> {
        let narrow = |v: &BitVector| {
            v.to_u128().map_err(|_| {
                Error::new(
                    "§3.3: value exceeds the u128 convenience API; use the vector API".into(),
                )
            })
        };
        Ok(OracleOutcome {
            payload: narrow(&self.payload)?,
            code: narrow(&self.code)?,
            steps: self.steps,
        })
    }
}

/// One `alloctab` entry (§7.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Allocation {
    pub base: u128,
    pub size: u128,
    pub live: bool,
    pub pinned: bool,
}
/// Byte memory and allocation table, shared by the lifted engines and CC/R.
/// Addresses are below 2^ptr_width <= 2^128; every sum is checked, never wrapped.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub(crate) struct Memory {
    pub bytes: crate::retained::SparseBytes,
    pub allocations: Vec<Allocation>,
    pub heap_base: u128,
    pub heap_used: u128,
    /// 2^A_tab, or None when 2^A_tab >= 2^128 (no reachable table index).
    pub table_capacity: Option<u128>,
    pub addresses: BTreeMap<String, u128>,
}
pub(crate) fn power_of_two(exponent: u128) -> Option<u128> {
    (exponent < 128).then(|| 1u128 << exponent)
}
#[cfg(test)]
pub(crate) fn r21_resource_cases() {
    let mut m = Memory {
        bytes: Default::default(),
        allocations: Vec::new(),
        heap_base: 1,
        heap_used: 0,
        table_capacity: None,
        addresses: BTreeMap::from([("r".into(), 1)]),
    };
    for a in [0, 1, 3, 62, 63, 64, 127, 128] {
        let r = region_bytes(&m, "r", a);
        if a < 4 {
            assert_eq!(r.unwrap(), vec![0; 1 << a]);
        } else {
            let e = r.unwrap_err().to_string();
            assert!(e.contains("§4.2") && e.contains("host limit"), "{e}");
        }
        // The table capacity is symbolic, not 2^A eagerly allocated entries.
        m.table_capacity = power_of_two(a as u128);
        assert!(!m.allocation_fails(1, 1));
    }
    for width in [8, 64, (1u64 << 63) - 8, 1 << 63, u64::MAX - 7] {
        let r = m.load(1, width);
        if width <= 64 {
            assert!(r.unwrap().is_zero());
        } else {
            let e = r.unwrap_err().to_string();
            assert!(e.contains("§7.1") && e.contains("host limit"), "{e}");
        }
    }
    for len in [0, 1, 8, (1u128 << 63) - 1, 1 << 63, u64::MAX as u128] {
        let base = m.allocate(len).unwrap();
        assert!(m.valid(base, len));
        assert!(m.bytes.is_empty());
    }
    eprintln!("R21 nested region transfer / symbolic table capacity / load storage / sparse allocation: 4 paths / 27 cases");
}
impl Memory {
    pub fn initial(
        object: &Object,
        oracle: &Function,
        regions: &BTreeMap<String, Vec<u8>>,
    ) -> Result<Self> {
        let expected = if oracle.kind == "oracle" {
            oracle.params[3].len()
        } else {
            0
        };
        if regions.len() != expected {
            return err("§5.1", "interface region count mismatch");
        }
        let Some(layout) = object.section("layout") else {
            return Ok(Self::default());
        };
        let mut m = Self {
            heap_base: layout.get("heap_base")?.uint()?,
            table_capacity: power_of_two(layout.get("A_tab")?.uint()?),
            ..Self::default()
        };
        // Pre-entries in layout order: globals, then this oracle's regions (§3.7).
        for (n, g) in layout.get("globals")?.object()? {
            let base = g.get("base")?.uint()?;
            let size = g.get("size")?.uint()?;
            m.addresses.insert(n.clone(), base);
            m.allocations.push(Allocation {
                base,
                size,
                live: true,
                pinned: true,
            });
            let image = object
                .data
                .get(n)
                .ok_or_else(|| Error::new(format!("§9.2: global {n} has no data image")))?;
            for (i, b) in image.iter().enumerate() {
                m.set_byte(offset(base, i)?, *b)?;
            }
        }
        if oracle.kind == "oracle" && !oracle.params[3].is_empty() {
            let mapping = layout.get("interface_regions")?.get(&oracle.name)?;
            for (n, r) in mapping.object()? {
                let p = oracle.params[3]
                    .iter()
                    .find(|p| p.name == *n)
                    .ok_or_else(|| Error::new(format!("§7.2: layout maps unknown region {n}")))?;
                let Type::Region(a, _) = p.ty else {
                    return err("§5.1", "mem slot requires region type");
                };
                let base = r.get("base")?.uint()?;
                let size = power_of_two(a as u128)
                    .ok_or_else(|| Error::new("§7.2: interface region exceeds 2^H".into()))?;
                let bytes = regions
                    .get(n)
                    .ok_or_else(|| Error::new(format!("§5.1: missing interface region {n}")))?;
                if bytes.len() as u128 != size {
                    return err("§5.1", format!("interface region {n} has wrong byte count"));
                }
                m.addresses.insert(n.clone(), base);
                m.allocations.push(Allocation {
                    base,
                    size,
                    live: true,
                    pinned: true,
                });
                for (i, b) in bytes.iter().enumerate() {
                    m.set_byte(offset(base, i)?, *b)?;
                }
            }
        }
        Ok(m)
    }
    /// inrange(p, n) of §4.2: some live entry i < atop has base_i <= p and
    /// p + n <= base_i + size_i, computed without wrap.
    pub fn valid(&self, p: u128, n: u128) -> bool {
        self.allocations
            .iter()
            .any(|e| e.live && e.base <= p && n <= e.size && p - e.base <= e.size - n)
    }
    /// inrange for a length that may exceed 128 bits: such a length exceeds
    /// every entry size (sizes are below 2^128), so the range is never valid.
    pub fn valid_vector(&self, p: u128, n: &BitVector) -> bool {
        n.to_u128().is_ok_and(|n| self.valid(p, n))
    }
    pub fn byte(&self, p: u128) -> u8 {
        self.bytes.get(&p).copied().unwrap_or(0)
    }
    pub fn set_byte(&mut self, p: u128, b: u8) -> Result<()> {
        self.bytes.set(p, b)
    }
    /// N/8 bytes little-endian at p; the caller has established inrange.
    pub fn load(&self, p: u128, width: u64) -> Result<BitVector> {
        let count = (width / 8) as usize;
        let mut bytes = crate::reserved(count, "§7.1", "memory load bytes")?;
        for i in 0..count {
            bytes.push(self.byte(offset(p, i)?));
        }
        BitVector::from_le_bytes(width as usize, &bytes)
    }
    pub fn store(&mut self, p: u128, value: &BitVector) -> Result<()> {
        if !value.width().is_multiple_of(8) {
            return err("§3.3", "store width must be a byte multiple");
        }
        let bytes = value.to_le_bytes();
        self.bytes
            .prepare(bytes.iter().filter(|b| **b != 0).count())?;
        for (i, b) in bytes.into_iter().enumerate() {
            self.set_byte(offset(p, i)?, b)?;
        }
        Ok(())
    }
    pub fn load_u128(&self, p: u128, width: u64) -> Result<u128> {
        self.load(p, width)?.to_u128()
    }
    pub fn store_u128(&mut self, p: u128, width: u64, v: u128) -> Result<()> {
        let value = BitVector::from_u128(width as usize, v)?;
        self.store(p, &value)
    }
    /// The lift.alloc / alloc flag: heap_used + max(n,1) > heap_limit, or the
    /// table is full (atop >= 2^A_tab). The sum is exact (§3.7).
    pub fn allocation_fails(&self, n: u128, heap_limit: u128) -> bool {
        self.heap_used
            .checked_add(n.max(1))
            .is_none_or(|total| total > heap_limit)
            || self
                .table_capacity
                .is_some_and(|cap| self.allocations.len() as u128 >= cap)
    }
    pub fn allocate(&mut self, n: u128) -> Result<u128> {
        let base = self
            .heap_base
            .checked_add(self.heap_used)
            .ok_or_else(|| Error::new("§7.1: heap bump exceeds address space".into()))?;
        crate::reserve_more(
            &mut self.allocations,
            1,
            "§7.1",
            "sparse allocation entries",
        )?;
        self.allocations.push(Allocation {
            base,
            size: n,
            live: true,
            pinned: false,
        });
        self.heap_used += n.max(1);
        Ok(base)
    }
    /// The lift.free row of §4.2: flag = no i < atop with base_i = p, live and
    /// unpinned; otherwise live_i ^= match_i for EVERY i < atop with base_i = p
    /// (SPEC-QUESTIONS 16). Returns false when the row faults.
    pub fn lifted_free(&mut self, p: u128) -> bool {
        if !self
            .allocations
            .iter()
            .any(|e| e.base == p && e.live && !e.pinned)
        {
            return false;
        }
        for e in &mut self.allocations {
            if e.base == p {
                e.live ^= true;
            }
        }
        true
    }
}
pub(crate) fn offset(base: u128, i: usize) -> Result<u128> {
    base.checked_add(i as u128)
        .ok_or_else(|| Error::new("§7.2: byte address exceeds 2^128".into()))
}

pub(crate) struct Program<'a> {
    pub rows: Vec<(&'a Function, &'a Instruction)>,
    pub labels: BTreeMap<(String, String), usize>,
    pub ranges: BTreeMap<String, (usize, usize)>,
    pub halt: usize,
}
impl<'a> Program<'a> {
    /// §3.6 index space: the oracle's blocks in text order, then its lproc
    /// closure ordered by closure hash (multi-member SCCs: SPEC-QUESTIONS 7).
    pub fn new(object: &'a Object, root: &str) -> Result<Self> {
        object.ensure_linked()?;
        let oracle = object
            .functions
            .get(root)
            .filter(|f| f.kind == "oracle")
            .ok_or_else(|| Error::new(format!("§5.2: unknown oracle @{root}")))?;
        let mut reachable = BTreeSet::new();
        let mut pending = vec![root.to_string()];
        while let Some(n) = pending.pop() {
            let function = object.functions.get(&n).ok_or_else(|| {
                Error::new(format!(
                    "§9.3: oracle closure has unresolved lproc @{n}; link before execution"
                ))
            })?;
            for i in function.instructions().filter(|i| i.op == "lift.call") {
                let n = i.operands[0].trim_start_matches('@').to_string();
                if reachable.insert(n.clone()) {
                    pending.push(n);
                }
            }
        }
        let mut closure: Vec<_> = reachable.into_iter().collect();
        if !closure.is_empty() {
            let identity = object.identity()?;
            let mut keys = BTreeMap::new();
            for n in &closure {
                let key = if identity.scc_members[n].len() > 1 {
                    identity.scc_hashes[n].clone()
                } else {
                    identity.closure_hashes[n].clone().ok_or_else(|| {
                        Error::new(format!("§9.3: lproc @{n} has no closure hash"))
                    })?
                };
                keys.insert(n.clone(), key);
            }
            closure.sort_by(|a, b| keys[a].cmp(&keys[b]).then(a.cmp(b)));
        }
        let order = std::iter::once(&oracle.name).chain(closure.iter());
        let mut rows = Vec::new();
        let mut labels = BTreeMap::new();
        let mut ranges = BTreeMap::new();
        for n in order {
            let f = &object.functions[n];
            let start = rows.len();
            for b in &f.blocks {
                labels.insert((n.clone(), b.name.clone()), rows.len());
                for i in &b.instructions {
                    rows.push((f, i));
                }
            }
            ranges.insert(n.clone(), (start, rows.len()));
        }
        let halt = rows.len();
        Ok(Self {
            rows,
            labels,
            ranges,
            halt,
        })
    }
    pub fn label(&self, f: &str, label: &str) -> Result<usize> {
        self.labels
            .get(&(f.into(), label.trim_start_matches('^').into()))
            .copied()
            .ok_or_else(|| Error::new(format!("§3.6: unknown label {label} in @{f}")))
    }
}

/// Width of a readable operand of `f` (§3.7 name resolution).
pub(crate) fn operand_width(object: &Object, f: &Function, n: &str) -> Result<u64> {
    let pseudo: Vec<_> = if f.kind == "oracle" {
        f.params[0].iter().chain(&f.params[4]).collect()
    } else {
        Vec::new()
    };
    f.locations()
        .into_iter()
        .chain(pseudo)
        .find(|p| p.name == n)
        .ok_or_else(|| Error::new(format!("§3.7: unknown operand {n} in @{}", f.name)))?
        .ty
        .width(object.ptr_width)
}
/// A length operand: an INT literal in [0, 2^ptr_width) or a location.
pub(crate) fn literal_u128(s: &str) -> Result<u128> {
    s.parse::<u128>()
        .map_err(|_| Error::new(format!("§3.3: literal {s} exceeds 2^ptr_width")))
}

struct Activation {
    function: String,
    values: BTreeMap<String, BitVector>,
    ret_pc: usize,
    destination: String,
}
impl Activation {
    fn new(f: &Function, ptr: u16) -> Result<Self> {
        Ok(Self {
            function: f.name.clone(),
            values: f
                .locations()
                .iter()
                .map(|p| Ok((p.name.clone(), BitVector::zero(p.ty.bits(ptr)?)?)))
                .collect::<Result<_>>()?,
            ret_pc: 0,
            destination: String::new(),
        })
    }
    fn read(&self, n: &str) -> Result<BitVector> {
        self.values.get(n).cloned().ok_or_else(|| {
            Error::new(format!(
                "§3.7: unreadable operand {n} in @{}",
                self.function
            ))
        })
    }
    fn write(&mut self, f: &Function, n: &str, value: BitVector, ptr: u16) -> Result<()> {
        let p = f
            .locations()
            .into_iter()
            .find(|p| p.name == n)
            .ok_or_else(|| Error::new(format!("§3.7: unwritable location {n}")))?;
        if value.width() != p.ty.bits(ptr)? {
            return err(
                "§3.3",
                format!("internal width mismatch writing {n}: lifted writes are never resized"),
            );
        }
        self.values.insert(n.into(), value);
        Ok(())
    }
}
fn token(arguments: &BTreeMap<String, Argument>, n: &str) -> Result<Budget> {
    match arguments.get(n) {
        Some(Argument::Token(b)) => Ok(*b),
        _ => err(
            "§4.2",
            format!("tk operand {n} is not a token argument of the oracle"),
        ),
    }
}
/// Bytes of a mapped interface region passed by name to a nested region call.
pub(crate) fn region_bytes(memory: &Memory, n: &str, address_width: u64) -> Result<Vec<u8>> {
    let base = *memory
        .addresses
        .get(n)
        .ok_or_else(|| Error::new(format!("§4.2: mem operand {n} is not a mapped region")))?;
    let len = power_of_two(address_width as u128)
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| {
            Error::host_resource("§4.2: host limit: region exceeds host address space".into())
        })?;
    let mut bytes = crate::reserved(len, "§4.2/§7.1", "nested region transfer bytes")?;
    for i in 0..len {
        bytes.push(memory.byte(offset(base, i)?));
    }
    Ok(bytes)
}

impl Object {
    pub fn validate_budget(&self, b: Budget) -> Result<()> {
        if let Some(l) = self.section("layout") {
            let s = l.get("S_cells")?.uint()?;
            if b.heap > l.get("heap_limit_max")?.uint()?
                || power_of_two(s).is_some_and(|cells| b.stack > cells)
            {
                return err("§5.4", "budget heap/stack limit exceeds the layout");
            }
            // depth_limit < 2^Dc - 1 holds for the derived Dc = ceil(log2(d + 2))
            // whenever d + 2 is representable (§3.7, §5.4).
            if b.depth.checked_add(2).is_none() {
                return Err(crate::resource_error("§5.4", "depth plus two exceeds u128"));
            }
        } else if b.heap != 0 || b.stack != 0 || b.depth != 0 {
            return err(
                "§5.4",
                "memoryless budgets require zero heap, stack and depth",
            );
        }
        Ok(())
    }
    pub fn default_budget(&self, steps: u64) -> Result<Budget> {
        if let Some(l) = self.section("layout") {
            Ok(Budget {
                steps,
                heap: l.get("heap_limit_max")?.uint()?,
                stack: power_of_two(l.get("S_cells")?.uint()?).unwrap_or(u128::MAX),
                depth: 64,
            })
        } else {
            Ok(Budget::steps(steps))
        }
    }
    /// F_B on small-integer inputs: a checked adapter over `run_oracle_vectors`.
    pub fn run_oracle(
        &self,
        name: &str,
        input: &OracleInput,
        budget: Budget,
    ) -> Result<OracleOutcome> {
        let vector = OracleVectorInput::from_input(self, name, input)?;
        self.run_oracle_vectors(name, &vector, budget)?.narrow()
    }
    /// The §14.1 reference interpreter: defines F_B and the budget unit.
    pub fn run_oracle_vectors(
        &self,
        name: &str,
        input: &OracleVectorInput,
        budget: Budget,
    ) -> Result<OracleVectorOutcome> {
        self.validate_budget(budget)?;
        self.check_schedule_matches_for_run(name, budget, &input.args)?;
        let program = Program::new(self, name)?;
        let root = &self.functions[name];
        if input.ins.len() != root.params[0].len() || input.args.len() != root.params[4].len() {
            return err("§5.1", "oracle interface argument count mismatch");
        }
        let mut activation = Activation::new(root, self.ptr_width)?;
        for (p, v) in root.params[0].iter().zip(&input.ins) {
            if v.width() != p.ty.bits(self.ptr_width)? {
                return err("§3.3", "input value width differs from its bundle");
            }
            activation.values.insert(p.name.clone(), v.clone());
        }
        let arguments = self.bind_arguments(root, &input.args)?;
        for (p, v) in root.params[4].iter().zip(&input.args) {
            if let (Type::Int(_) | Type::Ptr, Argument::Bits(v)) = (&p.ty, v) {
                activation.values.insert(
                    p.name.clone(),
                    BitVector::from_u128(p.ty.bits(self.ptr_width)?, *v)?,
                );
            }
        }
        let mut memory = Memory::initial(self, root, &input.regions)?;
        let mut stack: Vec<Activation> = Vec::new();
        let mut sp = 0u128;
        let mut pc = 0;
        let mut steps = 0;
        let target_width = root.params[1][0].ty.bits(self.ptr_width)?;
        let fault_width = root.params[2][0].ty.bits(self.ptr_width)?;
        let mut result = BitVector::zero(target_width)?;
        macro_rules! fault {
            ($code:expr) => {
                return Ok(OracleVectorOutcome {
                    payload: BitVector::zero(target_width)?,
                    code: $code,
                    steps,
                })
            };
        }
        let code = |n: u128| BitVector::from_u128(fault_width, n);
        while steps < budget.steps {
            if pc == program.halt {
                return err("§3.6", "only lift.retval reaches HALT with success");
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
            let mut output = None;
            if let Some(n) = op.strip_prefix("lift.const.i") {
                let w = crate::object::host_count(n, "const width")? as usize;
                output = Some(BitVector::parse_decimal(w, &a[0])?);
            } else if let Some(bin) = op.strip_prefix("lift.").filter(|s| is_binop(s)) {
                output = Some(rd(&a[0])?.binary(bin, &rd(&a[1])?)?);
            } else if let Some(div) = op.strip_prefix("lift.").filter(|s| is_div(s)) {
                let (x, y) = (rd(&a[0])?, rd(&a[1])?);
                match x.division_fault(&y, div.starts_with('s'))? {
                    Some(c) => fault!(code(c)?),
                    None => output = Some(x.binary(div, &y)?),
                }
            } else if let Some(pred) = op.strip_prefix("lift.icmp.") {
                output = Some(BitVector::from_u128(
                    1,
                    u128::from(rd(&a[0])?.compare(pred, &rd(&a[1])?)?),
                )?);
            } else if let Some((cast, target)) = op
                .strip_prefix("lift.")
                .and_then(|s| s.split_once(".i"))
                .filter(|(c, _)| ["zext", "sext", "trunc"].contains(c))
            {
                let w = crate::object::host_count(target, "cast width")? as usize;
                output = Some(rd(&a[0])?.resize(w, cast == "sext")?);
            } else if let Some(n) = op.strip_prefix("lift.load.i") {
                let w = crate::object::host_count(n, "load width")?;
                let p = rd(&a[0])?.to_u128()?;
                if !memory.valid(p, (w / 8) as u128) {
                    fault!(code(7)?);
                }
                output = Some(memory.load(p, w)?);
            } else if op.starts_with("lift.store.i") {
                let p = rd(&a[0])?.to_u128()?;
                let v = rd(&a[1])?;
                if !memory.valid(p, (v.width() / 8) as u128) {
                    fault!(code(7)?);
                }
                memory.store(p, &v)?;
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
                    "lift.label" => {
                        let w = operand_width(self, f, &i.lhs[0])? as usize;
                        output = Some(BitVector::from_u128(
                            w,
                            program.label(&f.name, &a[0])? as u128,
                        )?)
                    }
                    "lift.addr" => {
                        output = Some(BitVector::from_u128(
                            self.ptr_width as usize,
                            *memory
                                .addresses
                                .get(a[0].trim_start_matches('@'))
                                .ok_or_else(|| Error::new("§4.2: address not mapped".into()))?,
                        )?);
                    }
                    "lift.boundscheck" => {
                        let p = rd(&a[0])?.to_u128()?;
                        let ok = if integer(&a[1]) {
                            memory.valid(p, literal_u128(&a[1])?)
                        } else {
                            memory.valid_vector(p, &rd(&a[1])?)
                        };
                        if !ok {
                            fault!(code(7)?);
                        }
                    }
                    "lift.alloc" => {
                        // A length of 2^128 or more always exceeds heap_limit < 2^128.
                        let n = if integer(&a[0]) {
                            Some(literal_u128(&a[0])?)
                        } else {
                            rd(&a[0])?.to_u128().ok()
                        };
                        match n {
                            Some(n) if !memory.allocation_fails(n, budget.heap) => {
                                output = Some(BitVector::from_u128(
                                    self.ptr_width as usize,
                                    memory.allocate(n)?,
                                )?);
                            }
                            _ => fault!(code(2)?),
                        }
                    }
                    "lift.free" => {
                        let p = rd(&a[0])?.to_u128()?;
                        if !memory.lifted_free(p) {
                            fault!(code(7)?);
                        }
                    }
                    "lift.faultif" => {
                        if !rd(&a[0])?.is_zero() {
                            fault!(BitVector::parse_unsigned_exact(fault_width, &a[1])?);
                        }
                    }
                    "fault" => fault!(BitVector::parse_unsigned_exact(fault_width, &a[0])?),
                    "qjump" => next = program.label(&f.name, &a[0])?,
                    "qbranch" => {
                        next = program
                            .label(&f.name, if !rd(&a[0])?.is_zero() { &a[1] } else { &a[2] })?
                    }
                    "qjumpr" => {
                        let (start, end) = program.ranges[&f.name];
                        match rd(&a[0])?.to_u128() {
                            Ok(t) if t >= start as u128 && t < end as u128 => next = t as usize,
                            _ => fault!(code(4)?),
                        }
                    }
                    "lift.call" => {
                        let callee = &self.functions[a[0].trim_start_matches('@')];
                        let cells = callee.frame_cells() as u128;
                        if stack.len() as u128 + 1 > budget.depth
                            || sp.checked_add(cells).is_none_or(|p| p > budget.stack)
                        {
                            fault!(code(3)?);
                        }
                        let values = list(&a[1])?
                            .iter()
                            .map(|s| rd(s))
                            .collect::<Result<Vec<_>>>()?;
                        let mut called = Activation::new(callee, self.ptr_width)?;
                        called.ret_pc = pc + 1;
                        called.destination = i.lhs[0].clone();
                        for (p, v) in callee.params[0].iter().zip(values) {
                            called.write(callee, &p.name, v, self.ptr_width)?;
                        }
                        sp += cells;
                        stack.push(activation);
                        activation = called;
                        next = program.ranges[&callee.name].0;
                    }
                    "lift.ret" => {
                        let Some(caller) = stack.pop() else {
                            fault!(code(4)?);
                        };
                        let v = rd(&a[0])?;
                        let ret = activation.ret_pc;
                        let dest = activation.destination.clone();
                        activation = caller;
                        let caller = &self.functions[&activation.function];
                        activation.write(caller, &dest, v, self.ptr_width)?;
                        next = ret;
                    }
                    "lift.retval" => {
                        result = result.binary("xor", &rd(&a[0])?)?;
                        return Ok(OracleVectorOutcome {
                            payload: result,
                            code: BitVector::zero(fault_width)?,
                            steps,
                        });
                    }
                    "lift.oracle_call" | "lift.oracle_uncall" => {
                        let callee = &self.functions[a[0].trim_start_matches('@')];
                        let tk = token(&arguments, &a[1])?;
                        let mut child = OracleVectorInput {
                            ins: list(&a[2])?
                                .iter()
                                .map(|s| rd(s))
                                .collect::<Result<Vec<_>>>()?,
                            ..Default::default()
                        };
                        for (p, n) in callee.params[3].iter().zip(list(&a[5])?) {
                            let Type::Region(aw, _) = p.ty else {
                                return err("§5.1", "mem slot requires region type");
                            };
                            child
                                .regions
                                .insert(p.name.clone(), region_bytes(&memory, &n, aw)?);
                        }
                        for n in list(&a[6])? {
                            child.args.push(self.forward_argument(&arguments, &n)?);
                        }
                        // The matched inverse is the identical permutation (§4.2, §5.5).
                        let r = self.run_oracle_vectors(&callee.name, &child, tk)?;
                        let tgt = rd(&a[3])?.binary("xor", &r.payload)?;
                        let fr = rd(&a[4])?.binary("xor", &r.code)?;
                        activation.write(f, &a[3], tgt, self.ptr_width)?;
                        activation.write(f, &a[4], fr, self.ptr_width)?;
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
                            let Type::Region(aw, _) = p.ty else {
                                return err("§5.1", "mem slot requires region type");
                            };
                            regions.insert(p.name.clone(), region_bytes(&memory, &n, aw)?);
                        }
                        let args = list(&a[4])?
                            .iter()
                            .map(|n| self.forward_argument(&arguments, n))
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
                    _ => return err("§4.8", format!("unknown lifted row {op}")),
                }
            }
            if let Some(v) = output {
                activation.write(f, &i.lhs[0], v, self.ptr_width)?;
            }
            pc = next;
        }
        Ok(OracleVectorOutcome {
            payload: BitVector::zero(target_width)?,
            code: code(1)?,
            steps,
        })
    }
    /// Validates and indexes an oracle's classical arguments (§3.3, §4.2).
    pub(crate) fn bind_arguments(
        &self,
        f: &Function,
        args: &[Argument],
    ) -> Result<BTreeMap<String, Argument>> {
        let mut out = BTreeMap::new();
        for (p, v) in f.params[4].iter().zip(args) {
            check_argument(self, &p.ty, v)?;
            out.insert(p.name.clone(), v.clone());
        }
        Ok(out)
    }
    /// A classical argument forwarded to a nested region call: one of the
    /// calling oracle's own classical arguments (validate_region_call).
    fn forward_argument(
        &self,
        arguments: &BTreeMap<String, Argument>,
        n: &str,
    ) -> Result<Argument> {
        arguments
            .get(n)
            .cloned()
            .ok_or_else(|| Error::new(format!("§4.2: classical operand {n} is not an argument")))
    }
}
/// §3.3 classical argument typing, shared by oracles, actions and CC region calls.
pub(crate) fn check_argument(object: &Object, ty: &Type, v: &Argument) -> Result<()> {
    let fits = |w: u64, n: u128| w >= 128 || n >> w == 0;
    match (ty, v) {
        (Type::Int(w), Argument::Bits(n)) if fits(*w as u64, *n) => Ok(()),
        (Type::Ptr, Argument::Bits(n)) if fits(object.ptr_width as u64, *n) => Ok(()),
        (Type::F64, Argument::Float(x)) if x.is_finite() => Ok(()),
        (Type::Token, Argument::Token(b)) => object.validate_budget(*b),
        _ => err(
            "§3.3",
            format!("classical argument does not have type {}", ty.text()),
        ),
    }
}

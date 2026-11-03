//! CC quantum-register ownership and the reversible handle transitions of §3.3.
use crate::{err, quantum::Wire, Error, QuantumState, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Status {
    Live,
    Viewed,
    Consumed,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Handle {
    wires: Vec<Wire>,
    classes: Vec<String>,
    status: Status,
    parent: Option<u64>,
}
#[derive(Clone, Debug)]
pub struct QuantumRegisters {
    pub(crate) state: QuantumState,
    handles: BTreeMap<u64, Handle>,
    next: u64,
    runtime_ids: Option<Arc<AtomicU64>>,
}
impl PartialEq for QuantumRegisters {
    fn eq(&self, other: &Self) -> bool {
        // The ever-used id namespace is not reversible machine state. A
        // snapshot owns exact quantum/ledger state and its historical cursor.
        self.state == other.state && self.handles == other.handles && self.next == other.next
    }
}
impl Default for QuantumRegisters {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Clone, Debug)]
enum KernelUndo {
    None,
    Allocation(Vec<Wire>),
    Release(Vec<(usize, Wire)>),
}
#[derive(Clone, Debug)]
pub(crate) struct RegisterUndo {
    handles: Vec<(u64, Option<Handle>)>,
    next: u64,
    wire_next: u64,
    kernel: KernelUndo,
}

impl QuantumRegisters {
    pub fn new() -> Self {
        Self {
            state: QuantumState::new(),
            handles: BTreeMap::new(),
            next: 1,
            runtime_ids: None,
        }
    }
    pub(crate) fn runtime_ids(&mut self, ids: Arc<AtomicU64>) {
        ids.fetch_max(self.next, Ordering::Relaxed);
        self.runtime_ids = Some(ids);
    }
    fn fresh_id(&self) -> Result<u64> {
        if let Some(ids) = &self.runtime_ids {
            ids.try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
                .map_err(|_| crate::resource_error("§3.3", "qreg id namespace exhausted"))
        } else {
            self.next
                .checked_add(1)
                .ok_or_else(|| crate::resource_error("§3.3", "qreg id exhausted"))?;
            Ok(self.next)
        }
    }
    pub fn quantum_state(&self) -> &QuantumState {
        &self.state
    }
    pub fn prepare(&mut self, id: u64, bits: &[bool]) -> Result<()> {
        let wires = self.ordinary(id)?.wires.clone();
        if wires.len() != bits.len() {
            return err("§10", "E_ARG: preparation width mismatch");
        }
        let mut check = self.state.clone();
        check.release(&wires)?;
        for (wire, bit) in wires.iter().zip(bits) {
            if *bit {
                self.state.gate("x", &[*wire], None, &[])?;
            }
        }
        Ok(())
    }
    pub fn bundle(&self, id: u64) -> Result<&[Wire]> {
        let h = self
            .handles
            .get(&id)
            .filter(|h| h.status == Status::Live)
            .ok_or_else(|| Error::new(format!("§3.3: qreg {id} is not live")))?;
        Ok(&h.wires)
    }
    pub fn width(&self, id: u64) -> Result<u64> {
        u64::try_from(self.ordinary(id)?.wires.len())
            .map_err(|_| crate::resource_error("§3.3", "qreg width exceeds u64"))
    }
    pub fn classes(&self, id: u64) -> Result<&[String]> {
        self.bundle(id)?;
        Ok(&self.handles[&id].classes)
    }
    pub fn from_id(&self, id: u64) -> Result<u64> {
        self.ordinary(id)?;
        Ok(id)
    }
    fn ordinary(&self, id: u64) -> Result<&Handle> {
        self.bundle(id)?;
        let h = &self.handles[&id];
        if h.parent.is_some() {
            return err(
                "§3.3",
                "a qreg view may only be used by its one region call",
            );
        }
        Ok(h)
    }
    pub(crate) fn idle_bundle(&self, id: u64) -> Result<&[Wire]> {
        Ok(&self.ordinary(id)?.wires)
    }
    fn set_status(&mut self, id: u64, status: Status) -> Result<()> {
        self.handles
            .get_mut(&id)
            .ok_or_else(|| {
                Error::new(format!(
                    "§3.3: missing qreg handle {id} during ownership transition"
                ))
            })?
            .status = status;
        Ok(())
    }
    fn record(&self, ids: &[u64]) -> RegisterUndo {
        RegisterUndo {
            handles: ids
                .iter()
                .map(|id| (*id, self.handles.get(id).cloned()))
                .collect(),
            next: self.next,
            wire_next: self.state.next_wire_id(),
            kernel: KernelUndo::None,
        }
    }
    fn create(
        &mut self,
        id: u64,
        wires: Vec<Wire>,
        classes: Vec<String>,
        parent: Option<u64>,
    ) -> Result<u64> {
        let next = id
            .checked_add(1)
            .ok_or_else(|| Error::new("§3.3: qreg identity overflow".into()))?;
        self.handles.insert(
            id,
            Handle {
                wires,
                classes,
                status: Status::Live,
                parent,
            },
        );
        self.next = next;
        Ok(id)
    }
    pub fn allocate(&mut self, n: usize) -> Result<u64> {
        Ok(self.allocate_recorded(n)?.0)
    }
    pub(crate) fn allocate_recorded(&mut self, n: usize) -> Result<(u64, RegisterUndo)> {
        self.allocate_class_recorded(n, "any")
    }
    pub fn allocate_in_class(&mut self, n: usize, class: &str) -> Result<u64> {
        self.allocate_class_recorded(n, class).map(|r| r.0)
    }
    pub(crate) fn allocate_class_recorded(
        &mut self,
        n: usize,
        class: &str,
    ) -> Result<(u64, RegisterUndo)> {
        let mut classes = crate::reserved(n, "§3.3", "qreg class vector")?;
        classes.resize(n, class.into());
        let id = self.fresh_id()?;
        let mut undo = self.record(&[id]);
        let wires = self.state.allocate(n)?;
        undo.kernel = KernelUndo::Allocation(wires.clone());
        let id = self.create(id, wires, classes, None)?;
        Ok((id, undo))
    }
    pub fn free(&mut self, id: u64) -> Result<()> {
        self.free_recorded(id).map(|_| ())
    }
    pub(crate) fn free_recorded(&mut self, id: u64) -> Result<RegisterUndo> {
        let wires = self.ordinary(id)?.wires.clone();
        let mut undo = self.record(&[id]);
        let positions = wires
            .iter()
            .map(|w| Ok((self.state.position(*w)?, *w)))
            .collect::<Result<Vec<_>>>()?;
        self.state.release(&wires)?;
        self.set_status(id, Status::Consumed)?;
        undo.kernel = KernelUndo::Release(positions);
        Ok(undo)
    }
    pub fn slice(&mut self, id: u64, lo: usize, len: usize) -> Result<u64> {
        Ok(self.slice_recorded(id, lo, len)?.0)
    }
    pub(crate) fn slice_recorded(
        &mut self,
        id: u64,
        lo: usize,
        len: usize,
    ) -> Result<(u64, RegisterUndo)> {
        let h = self.ordinary(id)?;
        let end = lo
            .checked_add(len)
            .filter(|end| *end <= h.wires.len())
            .ok_or_else(|| Error::new("§3.3: qreg slice out of bounds".into()))?;
        let mut wires = crate::reserved(len, "§3.3", "qreg slice wires")?;
        wires.extend_from_slice(&h.wires[lo..end]);
        let mut classes = crate::reserved(len, "§3.3", "qreg slice class vector")?;
        classes.extend_from_slice(&h.classes[lo..end]);
        let view_id = self.fresh_id()?;
        let undo = self.record(&[id, view_id]);
        self.set_status(id, Status::Viewed)?;
        let view = self.create(view_id, wires, classes, Some(id))?;
        Ok((view, undo))
    }
    pub fn concat(&mut self, a: u64, b: u64) -> Result<u64> {
        Ok(self.concat_recorded(a, b)?.0)
    }
    pub(crate) fn concat_recorded(&mut self, a: u64, b: u64) -> Result<(u64, RegisterUndo)> {
        if a == b {
            return err("§3.3", "qreg concat operands alias");
        }
        let a_wires = &self.ordinary(a)?.wires;
        let b_wires = &self.ordinary(b)?.wires;
        let count = a_wires.len().checked_add(b_wires.len()).ok_or_else(|| {
            Error::host_resource("§3.3: host limit: qreg concat count exceeds usize".into())
        })?;
        let mut wires = crate::reserved(count, "§3.3", "qreg concat wires")?;
        wires.extend_from_slice(a_wires);
        wires.extend_from_slice(b_wires);
        if wires.iter().collect::<BTreeSet<_>>().len() != wires.len() {
            return err("§3.3", "qreg concat wires overlap");
        }
        let mut classes = crate::reserved(count, "§3.3", "qreg concat class vector")?;
        classes.extend_from_slice(&self.handles[&a].classes);
        classes.extend_from_slice(&self.handles[&b].classes);
        let joined = self.fresh_id()?;
        let undo = self.record(&[a, b, joined]);
        self.set_status(a, Status::Consumed)?;
        self.set_status(b, Status::Consumed)?;
        let id = self.create(joined, wires, classes, None)?;
        Ok((id, undo))
    }
    pub fn complete_call(&mut self, ids: &[u64]) -> Result<()> {
        self.complete_call_recorded(ids).map(|_| ())
    }
    pub(crate) fn complete_call_recorded(&mut self, ids: &[u64]) -> Result<RegisterUndo> {
        let mut affected = Vec::new();
        let mut wires = BTreeSet::new();
        for id in ids {
            for wire in self.bundle(*id)? {
                if !wires.insert(*wire) {
                    return err("§3.3", "region interface qregs overlap");
                }
            }
            affected.push(*id);
            if let Some(parent) = self.handles[id].parent {
                affected.push(parent);
            }
        }
        let undo = self.record(&affected);
        for id in ids {
            if let Some(parent) = self.handles[id].parent {
                self.set_status(*id, Status::Consumed)?;
                self.set_status(parent, Status::Live)?;
            }
        }
        Ok(undo)
    }
    pub(crate) fn measure(&mut self, id: u64, draw: f64) -> Result<Vec<bool>> {
        if !(0.0..1.0).contains(&draw) {
            return err("§14.2", "measurement draw outside [0,1)");
        }
        let wires = self.ordinary(id)?.wires.clone();
        let branches = self.state.measurement_branches(&wires)?;
        let total = branches
            .iter()
            .fold(crate::exact::Amplitude::zero(), |s, b| s.add(&b.weight));
        let negative_threshold = total.times_f64(draw)?.times_f64(-1.0)?;
        let mut cumulative = crate::exact::Amplitude::zero();
        let mut selected = None;
        for branch in branches {
            cumulative = cumulative.add(&branch.weight);
            if cumulative.add(&negative_threshold).real_sign() == std::cmp::Ordering::Greater {
                selected = Some(branch);
                break;
            }
        }
        let branch = selected
            .ok_or_else(|| Error::new("§14.2: measurement has no nonzero outcome".into()))?;
        self.state = branch.state;
        self.set_status(id, Status::Consumed)?;
        Ok(branch.bits)
    }
    pub(crate) fn undo(&mut self, undo: RegisterUndo) -> Result<()> {
        match undo.kernel {
            KernelUndo::None => {}
            KernelUndo::Allocation(wires) => self.state.release(&wires)?,
            KernelUndo::Release(positions) => self.state.restore_clean(&positions)?,
        };
        for (id, prior) in undo.handles {
            if let Some(prior) = prior {
                self.handles.insert(id, prior);
            } else {
                self.handles.remove(&id);
            }
        }
        self.next = undo.next;
        self.state.restore_next_wire_id(undo.wire_next)?;
        Ok(())
    }
}

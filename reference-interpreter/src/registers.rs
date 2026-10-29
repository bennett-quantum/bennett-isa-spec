//! CC quantum-register ownership and the reversible handle transitions of §3.3.
use crate::{err, quantum::Wire, Error, QuantumState, Result};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Status {
    Live,
    Viewed,
    Consumed,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Handle {
    wires: Vec<Wire>,
    status: Status,
    parent: Option<u64>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct QuantumRegisters {
    pub(crate) state: QuantumState,
    handles: BTreeMap<u64, Handle>,
    next: u64,
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
            .ok_or_else(|| Error(format!("§3.3: qreg {id} is not live")))?;
        Ok(&h.wires)
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
    fn create(&mut self, wires: Vec<Wire>, parent: Option<u64>) -> Result<u64> {
        let id = self.next;
        let next = id
            .checked_add(1)
            .ok_or_else(|| Error("§3.3: qreg identity overflow".into()))?;
        self.handles.insert(
            id,
            Handle {
                wires,
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
        let mut undo = self.record(&[self.next]);
        let wires = self.state.allocate(n)?;
        undo.kernel = KernelUndo::Allocation(wires.clone());
        let id = self.create(wires, None)?;
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
        self.handles.get_mut(&id).expect("live handle").status = Status::Consumed;
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
            .ok_or_else(|| Error("§3.3: qreg slice out of bounds".into()))?;
        let wires = h.wires[lo..end].to_vec();
        let undo = self.record(&[id, self.next]);
        self.handles.get_mut(&id).expect("live parent").status = Status::Viewed;
        let view = self.create(wires, Some(id))?;
        Ok((view, undo))
    }
    pub fn concat(&mut self, a: u64, b: u64) -> Result<u64> {
        Ok(self.concat_recorded(a, b)?.0)
    }
    pub(crate) fn concat_recorded(&mut self, a: u64, b: u64) -> Result<(u64, RegisterUndo)> {
        if a == b {
            return err("§3.3", "qreg concat operands alias");
        }
        let mut wires = self.ordinary(a)?.wires.clone();
        wires.extend(&self.ordinary(b)?.wires);
        if wires.iter().collect::<BTreeSet<_>>().len() != wires.len() {
            return err("§3.3", "qreg concat wires overlap");
        }
        let undo = self.record(&[a, b, self.next]);
        self.handles.get_mut(&a).expect("handle").status = Status::Consumed;
        self.handles.get_mut(&b).expect("handle").status = Status::Consumed;
        let id = self.create(wires, None)?;
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
                self.handles.get_mut(id).expect("view").status = Status::Consumed;
                self.handles.get_mut(&parent).expect("parent").status = Status::Live;
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
        let mut cursor = draw;
        let mut selected = None;
        for branch in branches {
            cursor -= branch.probability;
            let last = branch.probability > 0.0;
            if cursor < 0.0 {
                selected = Some(branch);
                break;
            }
            if last {
                selected = Some(branch);
            }
        }
        let branch =
            selected.ok_or_else(|| Error("§14.2: measurement has no nonzero outcome".into()))?;
        self.state = branch.state;
        self.handles.get_mut(&id).expect("handle").status = Status::Consumed;
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

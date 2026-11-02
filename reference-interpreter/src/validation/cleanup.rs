//! K81 occurrence partition. Observe the same atomic version transitions as
//! admission; never infer an inverse from equality of composed circuits.
use super::*;
pub(crate) type Key = (usize, Vec<usize>);
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Occurrence {
    pub key: Key,
    pub instruction: Instruction,
    pub outputs: BTreeSet<String>,
    pub reads: BTreeSet<String>,
    /// Persistent stack heads in Recorder::versions, not copied histories.
    pub before: BTreeMap<String, usize>,
    pub births: BTreeMap<String, usize>,
    pub inverse: Option<usize>,
    pub perm: bool,
    popped: Vec<usize>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Recorder {
    rows: Vec<Occurrence>,
    stacks: BTreeMap<String, usize>,
    versions: Vec<(usize, Option<usize>)>,
    births: BTreeMap<String, usize>,
    storage_units: usize,
}
impl Recorder {
    pub(super) fn read(&mut self, wires: impl IntoIterator<Item = String>) {
        self.rows
            .last_mut()
            .expect("recorded source row")
            .reads
            .extend(wires);
    }
    pub(super) fn write(&mut self, wire: &str, cancels: bool) {
        let id = self.rows.len() - 1;
        self.rows[id].outputs.insert(wire.into());
        if cancels {
            let head = self.stacks[wire];
            let (computing, prior) = self.versions[head];
            self.rows[id].popped.push(computing);
            if let Some(prior) = prior {
                self.stacks.insert(wire.into(), prior);
            } else {
                self.stacks.remove(wire);
            }
        } else {
            let next = self.versions.len();
            self.versions.push((id, self.stacks.get(wire).copied()));
            self.stacks.insert(wire.into(), next);
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) struct Pair {
    pub carrier: usize,
    pub cleanup: Vec<usize>,
    pub computation: Vec<usize>,
    pub targets: BTreeSet<String>,
    pub history: Vec<usize>,
    pub history_available: bool,
    pub perm: bool,
    pub release: bool,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct Plan {
    pub rows: Vec<Occurrence>,
    pub pairs: Vec<Pair>,
}
impl Object {
    pub(crate) fn source_cleanup_plan(&self, f: &Function) -> Result<Plan> {
        let types = f
            .params
            .iter()
            .flatten()
            .chain(&f.regs)
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect();
        let mut state = Discipline::new(f, types);
        for i in region_declarations(f) {
            self.declare_region(i, &mut state)?;
        }
        state.recording = Some(Recorder::default());
        fn visit(
            o: &Object,
            f: &Function,
            seq: &[Instruction],
            s: &mut Discipline,
            iteration: &mut Vec<usize>,
        ) -> Result<()> {
            for i in seq {
                if i.op == "qif" {
                    let outer = s.predicates.clone();
                    for (arm, body) in i.bodies.iter().enumerate() {
                        s.predicates = outer.clone();
                        s.predicates.push(if arm == 0 {
                            i.operands[0].clone()
                        } else {
                            format!("!{}", i.operands[0])
                        });
                        visit(o, f, body, s, iteration)?;
                    }
                    s.predicates = outer;
                } else if i.op == "qloop" {
                    let count = usize::try_from(crate::object::host_count(
                        &i.operands[0],
                        "source loop count",
                    )?)
                    .map_err(|_| crate::resource_error("§11", "loop count exceeds host"))?;
                    if count > bennett_isa_conformance::ref_language::DEFAULT_SOURCE_LEAF_LIMIT {
                        return Err(crate::resource_error(
                            "§11 K81",
                            "source iteration plan exceeds host capacity",
                        ));
                    }
                    let outer = s.predicates.clone();
                    for n in 0..count {
                        iteration.push(n);
                        if i.operands.len() == 3 {
                            s.predicates
                                .extend((0..=n).map(|k| format!("{}[{k}]", i.operands[2])));
                        }
                        visit(o, f, &i.bodies[0], s, iteration)?;
                        iteration.pop();
                        s.predicates = outer.clone();
                    }
                } else {
                    let r = s.recording.as_mut().expect("recording enabled");
                    r.storage_units = r
                        .storage_units
                        .checked_add(r.stacks.len() + r.births.len() + 1)
                        .ok_or_else(|| {
                            crate::resource_error("§11 K81", "source plan size overflow")
                        })?;
                    if r.storage_units
                        > 4 * bennett_isa_conformance::ref_language::DEFAULT_SOURCE_LEAF_LIMIT
                    {
                        return Err(crate::resource_error(
                            "§11 K81",
                            "source version plan exceeds host capacity",
                        ));
                    }
                    if r.rows.len()
                        >= bennett_isa_conformance::ref_language::DEFAULT_SOURCE_LEAF_LIMIT
                    {
                        return Err(crate::resource_error(
                            "§11 K81",
                            "source occurrence plan exceeds host capacity",
                        ));
                    }
                    r.rows.push(Occurrence {
                        key: (i.line, iteration.clone()),
                        instruction: i.clone(),
                        outputs: BTreeSet::new(),
                        reads: BTreeSet::new(),
                        before: r.stacks.clone(),
                        births: r.births.clone(),
                        inverse: None,
                        perm: true,
                        popped: Vec::new(),
                    });
                    o.validate_action_sequence(f, std::slice::from_ref(i), s)?;
                    let outputs = s
                        .recording
                        .as_ref()
                        .unwrap()
                        .rows
                        .last()
                        .unwrap()
                        .outputs
                        .clone();
                    let perm = !outputs.iter().any(|w| s.wire_is_other(w));
                    let r = s.recording.as_mut().unwrap();
                    let id = r.rows.len() - 1;
                    let row = &mut r.rows[id];
                    row.perm = perm;
                    if !row.outputs.is_empty()
                        && row.popped.len() == row.outputs.len()
                        && row.popped.iter().all(|n| *n == row.popped[0])
                    {
                        row.inverse = Some(row.popped[0]);
                    }
                    if matches!(i.op.as_str(), "qalloc" | "qborrow" | "qcopy" | "qload") {
                        r.births.insert(i.lhs[0].clone(), id);
                    }
                    if matches!(i.op.as_str(), "qfree" | "qreturn" | "quncopy" | "qunload") {
                        let name = i.operands.last().expect("release operand");
                        r.births.remove(name);
                        r.stacks
                            .retain(|w, _| w.split('[').next() != Some(name.as_str()));
                    }
                }
            }
            Ok(())
        }
        for block in &f.blocks {
            visit(self, f, &block.instructions, &mut state, &mut Vec::new())?;
        }
        let Recorder { rows, versions, .. } = state.recording.unwrap();
        let mut pairs = Vec::new();
        let mut owned = BTreeSet::new();
        for (id, row) in rows.iter().enumerate() {
            let op = row.instruction.op.as_str();
            let release = matches!(op, "qfree" | "qreturn");
            let intrinsic = matches!(op, "quncopy" | "qunload");
            if !release && !intrinsic {
                continue;
            }
            let name = row.instruction.operands.last().unwrap();
            let birth = row.births[name];
            let mut cleanup = Vec::new();
            let mut targets = BTreeSet::new();
            if release {
                let mut intervening = BTreeSet::new();
                for n in (birth + 1..id).rev() {
                    let previous = &rows[n];
                    if !previous
                        .outputs
                        .iter()
                        .any(|w| w.split('[').next() == Some(name.as_str()))
                    {
                        if previous
                            .reads
                            .iter()
                            .any(|w| w.split('[').next() == Some(name.as_str()))
                        {
                            break;
                        }
                        intervening.extend(previous.outputs.iter().chain(&previous.reads).cloned());
                        continue;
                    }
                    if !previous.outputs.is_disjoint(&intervening) {
                        break;
                    }
                    if previous.inverse.is_none() {
                        break;
                    }
                    if !owned.contains(&n) {
                        cleanup.push(n);
                        targets.extend(previous.outputs.clone());
                    }
                }
            }
            // A released bundle includes its unchanged bits: they can be
            // internal controls of the paired computation and carry arbitrary
            // entry data when borrowed.
            let width = produced_width(self, f, &rows, birth)?;
            targets.extend((0..width).map(|b| format!("{name}[{b}]")));
            cleanup.push(id);
            let mut computation = cleanup
                .iter()
                .filter_map(|n| rows[*n].inverse)
                .collect::<Vec<_>>();
            if intrinsic {
                computation.push(birth);
            }
            computation.sort_unstable();
            computation.dedup();
            for n in &cleanup {
                owned.insert(*n);
            }
            let perm = computation.iter().all(|n| rows[*n].perm);
            pairs.push(Pair {
                carrier: id,
                cleanup,
                computation,
                targets,
                history: Vec::new(),
                history_available: true,
                perm,
                release,
            });
        }
        for (id, row) in rows.iter().enumerate() {
            if owned.contains(&id) || row.inverse.is_none() {
                continue;
            }
            if !matches!(
                row.instruction.op.as_str(),
                "action_call" | "oracle_call" | "adj" | "ctl"
            ) {
                continue;
            }
            let computing = row.inverse.unwrap();
            pairs.push(Pair {
                carrier: id,
                cleanup: vec![id],
                computation: vec![computing],
                targets: row.outputs.clone(),
                history: Vec::new(),
                history_available: true,
                perm: rows[computing].perm,
                release: false,
            });
        }
        for pair in &mut pairs {
            // Replay uses complete bundle actuals. Include spectator bits as
            // identity components of the delta, preserving their exact entry
            // values when they also control a changed bit.
            let names = pair
                .targets
                .iter()
                .filter(|w| !w.ends_with("[*]"))
                .map(|w| w.split('[').next().unwrap().to_string())
                .collect::<BTreeSet<_>>();
            for name in names {
                let width = if let Some(p) = f.params.iter().flatten().find(|p| p.name == name) {
                    p.ty.bits(self.ptr_width)?
                } else {
                    produced_width(self, f, &rows, rows[pair.carrier].births[&name])?
                };
                pair.targets
                    .extend((0..width).map(|b| format!("{name}[{b}]")));
            }
            let Some(computing) = pair.computation.first().copied() else {
                continue;
            };
            let mut history = BTreeSet::new();
            for w in &pair.targets {
                if let Some(birth) = rows[computing].births.get(w.split('[').next().unwrap()) {
                    history.insert(*birth);
                }
                let mut head = rows[computing].before.get(w).copied();
                while let Some(n) = head {
                    history.insert(versions[n].0);
                    head = versions[n].1;
                }
            }
            pair.history = history
                .into_iter()
                .filter(|n| !pair.computation.contains(n))
                .collect();
            pair.history_available = pair.history.iter().all(|n| {
                rows[*n]
                    .reads
                    .iter()
                    .filter(|w| !pair.targets.contains(*w))
                    .all(|w| rows[*n].before.get(w) == rows[pair.carrier].before.get(w))
            });
        }
        Ok(Plan { rows, pairs })
    }
}
fn produced_width(o: &Object, f: &Function, rows: &[Occurrence], birth: usize) -> Result<usize> {
    let i = &rows[birth].instruction;
    if matches!(i.op.as_str(), "qalloc" | "qborrow") {
        return Ok(crate::object::host_count(&i.operands[1], "cleanup width")? as usize);
    }
    if i.op == "qload" {
        let name = &i.operands[0];
        if let Some(p) = f.params.iter().flatten().find(|p| &p.name == name) {
            if let Type::Region(_, w) = p.ty {
                return Ok(w as usize);
            }
        }
        if let Some(d) = region_declarations(f)
            .into_iter()
            .find(|i| &i.operands[0] == name)
        {
            return Ok(crate::object::host_count(&d.operands[2], "cell width")? as usize);
        }
    }
    let source = &i.operands[0];
    if source.contains('[') {
        return Ok(1);
    }
    if let Some(p) = f.params.iter().flatten().find(|p| &p.name == source) {
        return p.ty.bits(o.ptr_width);
    }
    let previous = rows[birth]
        .births
        .get(source)
        .ok_or_else(|| Error::new("§11 K81: missing produced source lifetime".into()))?;
    produced_width(o, f, rows, *previous)
}

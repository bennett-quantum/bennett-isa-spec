//! One fixed source instance for every reachable (lproc, binding key).
use super::*;
use std::sync::Arc;
#[derive(Clone)]
struct Instance {
    source: String,
    key: Vec<usize>,
    args: BTreeMap<String, usize>,
    regions: BTreeMap<String, usize>,
}
impl Instance {
    fn namespace(&self) -> String {
        if self.key.is_empty() {
            self.source.clone()
        } else {
            format!(
                "{}#{}",
                self.source,
                self.key
                    .iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join("_")
            )
        }
    }
}
pub(crate) struct Program<'a> {
    pub rows: Vec<(Arc<Function>, &'a Instruction)>,
    pub labels: BTreeMap<(String, String), usize>,
    pub ranges: BTreeMap<String, (usize, usize)>,
    pub halt: usize,
    functions: BTreeMap<String, Arc<Function>>,
    instances: BTreeMap<String, Instance>,
    calls: BTreeMap<(String, usize), String>,
    root_args: Vec<String>,
    root_regions: Vec<String>,
}
fn bind(caller: &Instance, callee: &Function, i: &Instruction) -> Result<Instance> {
    let mut result = Instance {
        source: callee.name.clone(),
        key: Vec::new(),
        args: BTreeMap::new(),
        regions: BTreeMap::new(),
    };
    for group in 2..5 {
        let operands = if i.operands.len() == 2 {
            Vec::new()
        } else {
            list(
                i.operands
                    .get(group)
                    .ok_or_else(|| Error::new("§3.7: incomplete fixed binding lists".into()))?,
            )?
        };
        if operands.len() != callee.params[group].len() {
            return err("§3.7", "fixed binding count differs from callee");
        }
        for (p, n) in callee.params[group].iter().zip(operands) {
            let source = if group == 4 {
                &caller.regions
            } else {
                &caller.args
            };
            let index = *source.get(&n).ok_or_else(|| {
                Error::new(format!(
                    "§3.7: {n} is not an entry argument or mapped region"
                ))
            })?;
            result.key.push(index);
            if group == 4 {
                result.regions.insert(p.name.clone(), index);
            } else {
                result.args.insert(p.name.clone(), index);
            }
        }
    }
    Ok(result)
}
fn close_instances(
    object: &Object,
    root: &str,
) -> Result<BTreeMap<(String, Vec<usize>), Instance>> {
    let oracle = object
        .functions
        .get(root)
        .ok_or_else(|| Error::new("§3.7: unknown oracle entry".into()))?;
    let entry = Instance {
        source: root.into(),
        key: Vec::new(),
        args: oracle.params[4]
            .iter()
            .enumerate()
            .map(|(i, p)| (p.name.clone(), i))
            .collect(),
        regions: oracle.params[3]
            .iter()
            .enumerate()
            .map(|(i, p)| (p.name.clone(), i))
            .collect(),
    };
    let mut closed = BTreeMap::from([((root.to_string(), Vec::new()), entry.clone())]);
    let mut pending = vec![entry];
    while let Some(caller) = pending.pop() {
        let function = object.declaration(&caller.source)?;
        for i in function.instructions().filter(|i| i.op == "lift.call") {
            let target = i
                .operands
                .first()
                .and_then(|s| s.strip_prefix('@'))
                .ok_or_else(|| Error::new("§3.7: lift.call requires @symbol".into()))?;
            let callee = object.declaration(target)?;
            let instance = bind(&caller, &callee, i)?;
            let key = (instance.source.clone(), instance.key.clone());
            if let std::collections::btree_map::Entry::Vacant(entry) = closed.entry(key) {
                crate::preparation("§3.7", "fixed binding instance")?;
                crate::reserve_more(&mut pending, 1, "§3.7", "pending fixed bindings")?;
                entry.insert(instance.clone());
                pending.push(instance);
            }
        }
    }
    Ok(closed)
}
impl<'a> Program<'a> {
    pub fn new(object: &'a Object, root: &str) -> Result<Self> {
        object.ensure_linked()?;
        let oracle = object
            .functions
            .get(root)
            .filter(|f| f.kind == "oracle")
            .ok_or_else(|| Error::new(format!("§5.2: unknown oracle @{root}")))?;
        let closed = close_instances(object, root)?;
        let mut order = closed
            .values()
            .filter(|i| i.source != root)
            .cloned()
            .collect::<Vec<_>>();
        if !order.is_empty() {
            let identity = object.identity()?;
            let mut hashes = BTreeMap::new();
            for instance in &order {
                let name = &instance.source;
                let hash = if identity.scc_members[name].len() > 1 {
                    identity.scc_hashes[name].clone()
                } else {
                    identity.closure_hashes[name]
                        .clone()
                        .ok_or_else(|| Error::new("§9.3: unlinked instance".into()))?
                };
                hashes.insert(name.clone(), hash);
            }
            order.sort_by(|a, b| {
                hashes[&a.source]
                    .cmp(&hashes[&b.source])
                    .then(a.source.cmp(&b.source))
                    .then(a.key.cmp(&b.key))
            });
        }
        let mut ordered = vec![closed[&(root.to_string(), Vec::new())].clone()];
        ordered.extend(order);
        let mut functions = BTreeMap::new();
        let mut instances = BTreeMap::new();
        let mut rows = Vec::new();
        let mut labels = BTreeMap::new();
        let mut ranges = BTreeMap::new();
        let mut calls = BTreeMap::new();
        for instance in ordered {
            let original = &object.functions[&instance.source];
            let name = instance.namespace();
            let mut function = original.clone();
            function.name = name.clone();
            let function = Arc::new(function);
            let start = rows.len();
            for block in &original.blocks {
                labels.insert((name.clone(), block.name.clone()), rows.len());
                for i in &block.instructions {
                    rows.push((function.clone(), i));
                }
            }
            ranges.insert(name.clone(), (start, rows.len()));
            for i in original.instructions().filter(|i| i.op == "lift.call") {
                let callee = &object.functions[i.operands[0].trim_start_matches('@')];
                let binding = bind(&instance, callee, i)?;
                calls.insert((name.clone(), i.line), binding.namespace());
            }
            instances.insert(name.clone(), instance);
            functions.insert(name, function);
        }
        let halt = rows.len();
        Ok(Self {
            rows,
            labels,
            ranges,
            halt,
            functions,
            instances,
            calls,
            root_args: oracle.params[4].iter().map(|p| p.name.clone()).collect(),
            root_regions: oracle.params[3].iter().map(|p| p.name.clone()).collect(),
        })
    }
    pub(crate) fn binding_keys(
        object: &Object,
        root: &str,
        source: &str,
    ) -> Result<Vec<Vec<usize>>> {
        Ok(close_instances(object, root)?
            .values()
            .filter(|instance| instance.source == source)
            .map(|instance| instance.key.clone())
            .collect())
    }
    pub(crate) fn index_space_size(
        object: &Object,
        root: &str,
        containing: &str,
    ) -> Result<Option<u128>> {
        let instances = close_instances(object, root)?;
        if !instances.values().any(|i| i.source == containing) {
            return Ok(None);
        }
        instances
            .values()
            .try_fold(0u128, |count, instance| {
                count
                    .checked_add(
                        object.declaration(&instance.source)?.instructions().count() as u128
                    )
                    .ok_or_else(|| {
                        crate::resource_error(
                            "§3.7",
                            "fixed binding index space exceeds host capacity",
                        )
                    })
            })
            .map(Some)
    }
    pub fn label(&self, f: &str, label: &str) -> Result<usize> {
        self.labels
            .get(&(f.into(), label.trim_start_matches('^').into()))
            .copied()
            .ok_or_else(|| Error::new(format!("§3.6: unknown label {label} in @{f}")))
    }
    pub fn call(&self, f: &Function, i: &Instruction) -> Result<Arc<Function>> {
        let name = self
            .calls
            .get(&(f.name.clone(), i.line))
            .ok_or_else(|| Error::new("§3.7: unresolved fixed call instance".into()))?;
        self.functions
            .get(name)
            .cloned()
            .ok_or_else(|| Error::new("§3.7: fixed instance is absent".into()))
    }
    pub fn function(&self, name: &str) -> Result<&Function> {
        self.functions
            .get(name)
            .map(Arc::as_ref)
            .ok_or_else(|| Error::new("§3.7: unknown activation instance".into()))
    }
    pub fn source_name(&self, f: &Function) -> &str {
        &self.instances[&f.name].source
    }
    pub fn instance_key(&self, f: &Function) -> &[usize] {
        &self.instances[&f.name].key
    }
    pub fn arguments<T: Clone>(
        &self,
        f: &Function,
        root: &BTreeMap<String, T>,
    ) -> Result<BTreeMap<String, T>> {
        self.instances[&f.name]
            .args
            .iter()
            .map(|(name, index)| {
                let root_name = &self.root_args[*index];
                Ok((
                    name.clone(),
                    root.get(root_name).cloned().ok_or_else(|| {
                        Error::new(format!("§3.7: missing fixed argument {root_name}"))
                    })?,
                ))
            })
            .collect()
    }
    pub fn region_name(&self, f: &Function, name: &str) -> String {
        self.instances[&f.name].regions.get(name).map_or_else(
            || name.to_string(),
            |index| self.root_regions[*index].clone(),
        )
    }
}

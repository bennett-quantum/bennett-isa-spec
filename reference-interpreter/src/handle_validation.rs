use crate::{err, object::list, validation::normalize_call, Error, Function, Object, Result, Type};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Views {
    live: BTreeMap<String, String>,
    dead: BTreeSet<String>,
}
impl Object {
    pub(crate) fn validate_handle_views(
        &self,
        f: &Function,
        types: &BTreeMap<String, Type>,
    ) -> Result<()> {
        let mut work = vec![(0usize, Views::default())];
        let mut seen = BTreeSet::new();
        while let Some((block, mut views)) = work.pop() {
            if !seen.insert((block, views.clone())) {
                continue;
            }
            for instruction in &f.blocks[block].instructions {
                let op = instruction.op.as_str();
                let a = &instruction.operands;
                let region = matches!(op, "oracle_call" | "action_call" | "ctl" | "adj");
                let operands: Vec<String> = match op {
                    "qreg.new" => Vec::new(),
                    "qreg.slice" | "qreg.free" | "measure" => vec![a[0].clone()],
                    "qreg.concat" => a.clone(),
                    _ => {
                        let mut values = Vec::new();
                        for operand in a {
                            if operand.starts_with('(') {
                                values.extend(list(operand)?);
                            } else if operand.starts_with('^') {
                                if let Some((_, suffix)) = operand.split_once('(') {
                                    values.extend(list(&format!("({suffix}"))?);
                                }
                            } else {
                                values.push(operand.clone());
                            }
                        }
                        values
                    }
                };
                let handles: Vec<_> = operands
                    .into_iter()
                    .filter(|n| matches!(types.get(n), Some(Type::Qreg(_))))
                    .collect();
                for handle in &handles {
                    if views.live.values().any(|parent| parent == handle) {
                        return err("§3.3", format!("use of viewed parent {handle}"));
                    }
                    if views.dead.contains(handle) {
                        return err("§3.3", format!("view {handle} used after its region call"));
                    }
                    if views.live.contains_key(handle) && !region {
                        return err(
                            "§3.3",
                            format!("view {handle} may only be an operand of one region call"),
                        );
                    }
                }
                if op == "qreg.slice" {
                    let view = instruction.lhs[0].clone();
                    views.dead.remove(&view);
                    views.live.insert(view, a[0].clone());
                }
                if region {
                    normalize_call(instruction)?;
                    for handle in handles {
                        if views.live.remove(&handle).is_some() {
                            views.dead.insert(handle);
                        }
                    }
                }
                if op == "br" {
                    for branch in if a.len() == 1 { &a[..] } else { &a[1..] } {
                        let (label, _) = branch
                            .trim_start_matches('^')
                            .split_once('(')
                            .ok_or_else(|| Error::new("§3.4: malformed branch".into()))?;
                        let target = f
                            .blocks
                            .iter()
                            .position(|b| b.name == label)
                            .expect("validated block");
                        work.push((target, views.clone()));
                    }
                }
                if matches!(op, "ret" | "trap") && !views.live.is_empty() {
                    return err(
                        "§3.3",
                        "view is not consumed by exactly one region call on every exit path",
                    );
                }
            }
        }
        Ok(())
    }
}

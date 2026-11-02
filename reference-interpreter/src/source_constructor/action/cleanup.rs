//! Source-owned K65/K80 replacements, before catalog instantiation.
use super::*;
use crate::validation::cleanup::{Pair, Plan};

fn substitute(value: &mut Value, names: &BTreeMap<String, String>) {
    match value {
        Value::Str(s) => {
            let (base, suffix) = s
                .split_once('[')
                .map_or((s.as_str(), None), |(b, s)| (b, Some(s)));
            if let Some(name) = names.get(base) {
                *s = suffix.map_or_else(|| name.clone(), |suffix| format!("{name}[{suffix}"));
            }
        }
        Value::Arr(a) => {
            for value in a {
                substitute(value, names);
            }
        }
        Value::Obj(m) => {
            for value in m.values_mut() {
                substitute(value, names);
            }
        }
        _ => {}
    }
}
fn fresh_calls(c: &mut Compiler<'_>, value: &mut Value) -> Result<()> {
    match value {
        Value::Arr(a) => {
            for v in a {
                fresh_calls(c, v)?;
            }
        }
        Value::Obj(m) => {
            if matches!(
                m.get("invoke").and_then(Value::as_str),
                Some("action_call" | "oracle_call")
            ) {
                let Some(Value::Arr(args)) = m.get_mut("args") else {
                    return Err(Error::new("§11 K80: call arguments".into()));
                };
                let old = args[0]
                    .as_str()
                    .ok_or_else(|| Error::new("§11 K80: call alias".into()))?;
                let mut symbol = c
                    .symbols
                    .iter()
                    .find(|s| s.name == old)
                    .cloned()
                    .ok_or_else(|| Error::new("§11 K80: missing captured call".into()))?;
                symbol.name = format!("$source-call/{}", c.counter);
                c.counter += 1;
                args[0] = text(&symbol.name);
                c.symbols.push(symbol);
            }
            for v in m.values_mut() {
                fresh_calls(c, v)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn replay(
    c: &mut Compiler<'_>,
    plan: &Plan,
    rows: &[usize],
    names: &BTreeMap<String, String>,
    adjoint: bool,
) -> Result<Vec<Value>> {
    let mut nodes = Vec::new();
    for n in rows {
        let nodes_for_row = c.compiled[&plan.rows[*n].key].nodes.clone();
        for node in &nodes_for_row {
            // Fresh lifetimes are acquired once by the replacement, never by
            // a replayed qcopy/qload's original declaration.
            if node.get("acquire").is_some() || node.get("relinquish").is_some() {
                continue;
            }
            let mut node = node.clone();
            substitute(&mut node, names);
            fresh_calls(c, &mut node)?;
            nodes.push(node);
        }
    }
    if adjoint {
        source_adjoint(nodes, &mut c.symbols)
    } else {
        Ok(nodes)
    }
}
fn entry_history(
    c: &mut Compiler<'_>,
    plan: &Plan,
    pair: &Pair,
    carrier: &Compiled,
    aliases: &BTreeMap<String, usize>,
    targets: &[(String, usize, Value)],
) -> Result<Vec<Value>> {
    // An interface/borrow entry is an arbitrary quantum input. Recover its
    // exact pre-call versions coherently from the still-current outputs of
    // the matched source computation, retaining h until delta is uncomputed.
    let mut h = BTreeMap::new();
    let mut delta = BTreeMap::new();
    for (alias, width) in aliases {
        h.insert(alias.clone(), c.declare(*width, "any", "clean"));
        delta.insert(alias.clone(), c.declare(*width, "any", "clean"));
    }
    let copy = |from: &BTreeMap<String, String>, to: &BTreeMap<String, String>| -> Vec<Value> {
        targets
            .iter()
            .map(|(alias, index, original)| {
                gate(
                    "cx",
                    [
                        from.get(alias)
                            .map_or_else(|| original.clone(), |name| bit(name, *index)),
                        to.get(alias)
                            .map_or_else(|| original.clone(), |name| bit(name, *index)),
                    ],
                )
            })
            .collect()
    };
    let original = BTreeMap::new();
    let mut out = h
        .values()
        .map(|n| obj([("acquire", text(n))]))
        .collect::<Vec<_>>();
    out.extend(copy(&original, &h));
    out.extend(replay(c, plan, &pair.computation, &h, true)?);
    out.extend(delta.values().map(|n| obj([("acquire", text(n))])));
    out.extend(copy(&h, &delta));
    out.extend(replay(c, plan, &pair.computation, &delta, false)?);
    out.extend(copy(&h, &delta));
    out.extend(controlled(&carrier.before, copy(&delta, &original)));
    out.extend(copy(&h, &delta));
    out.extend(replay(c, plan, &pair.computation, &delta, true)?);
    out.extend(copy(&h, &delta));
    out.extend(delta.values().rev().map(|n| obj([("relinquish", text(n))])));
    // The original input to the history recovery was y=f(h). Temporarily
    // restore that version while applying the exact adjoint of its recovery,
    // then restore the desired h. All these occurrences belong to the carrier.
    out.extend(replay(c, plan, &pair.computation, &original, false)?);
    out.extend(replay(c, plan, &pair.computation, &h, false)?);
    out.extend(copy(&original, &h));
    out.extend(replay(c, plan, &pair.computation, &original, true)?);
    out.extend(h.values().rev().map(|n| obj([("relinquish", text(n))])));
    if pair.release {
        out.extend(carrier.nodes.clone());
    }
    Ok(out)
}
fn replacement(c: &mut Compiler<'_>, plan: &Plan, pair: &Pair) -> Result<Vec<Value>> {
    let carrier = c.compiled[&plan.rows[pair.carrier].key].clone();
    let mut aliases = BTreeMap::new();
    let mut targets = Vec::new();
    for wire in &pair.targets {
        let (name, index) = wire
            .split_once('[')
            .ok_or_else(|| Error::new("§11 K81: target bit".into()))?;
        let index = index
            .strip_suffix(']')
            .and_then(|s| s.parse::<usize>().ok())
            .ok_or_else(|| Error::new("§11 K81: target index".into()))?;
        let (alias, width) = carrier
            .before
            .wires
            .get(name)
            .cloned()
            .ok_or_else(|| Error::new("§11 K81: released target lifetime missing".into()))?;
        aliases.insert(alias.clone(), width);
        targets.push((alias, index, carrier.before.indexed(name, index)?));
    }
    let arbitrary_entry = pair.targets.iter().any(|w| {
        let name = w.split('[').next().unwrap();
        let entry = pair.computation.first().map_or(pair.carrier, |n| *n);
        if plan.rows[entry].instruction.lhs.iter().any(|n| n == name)
            && matches!(
                plan.rows[entry].instruction.op.as_str(),
                "qcopy" | "qload" | "qalloc"
            )
        {
            return false;
        }
        plan.rows[entry]
            .births
            .get(name)
            .is_none_or(|birth| plan.rows[*birth].instruction.op == "qborrow")
    });
    if arbitrary_entry || !pair.history_available {
        return entry_history(c, plan, pair, &carrier, &aliases, &targets);
    }
    let mut fresh = BTreeMap::new();
    for (alias, width) in &aliases {
        fresh.insert(alias.clone(), c.declare(*width, "any", "clean"));
    }
    let mut out = fresh
        .values()
        .map(|n| obj([("acquire", text(n))]))
        .collect::<Vec<_>>();
    // A copy's entry history h is already available in its unchanged source.
    // The common K81 witness therefore needs just one fresh bundle.
    let direct_history = if pair.history.len() == 1 {
        let i = &plan.rows[pair.history[0]].instruction;
        if i.op == "qcopy" {
            Some((
                i.clone(),
                c.compiled[&plan.rows[pair.history[0]].key].clone(),
            ))
        } else {
            None
        }
    } else {
        None
    };
    let nonzero_history = pair
        .history
        .iter()
        .any(|n| !matches!(plan.rows[*n].instruction.op.as_str(), "qalloc" | "qborrow"));
    let mut history_xor = Vec::new();
    let mut history_fresh = BTreeMap::new();
    if nonzero_history && direct_history.is_none() {
        for (alias, width) in &aliases {
            history_fresh.insert(alias.clone(), c.declare(*width, "any", "clean"));
        }
        out.extend(history_fresh.values().map(|n| obj([("acquire", text(n))])));
        out.extend(replay(c, plan, &pair.history, &history_fresh, false)?);
        for (alias, index, _) in &targets {
            history_xor.push(gate(
                "cx",
                [
                    bit(&history_fresh[alias], *index),
                    bit(&fresh[alias], *index),
                ],
            ));
        }
    } else if let Some((i, row)) = &direct_history {
        let target = &row.after.wires[&i.lhs[0]].0;
        let width = row.after.wires[&i.lhs[0]].1;
        for b in 0..width {
            history_xor.push(gate(
                "cx",
                [
                    row.before.indexed(&i.operands[0], b)?,
                    bit(&fresh[target], b),
                ],
            ));
        }
        history_xor = controlled(&row.before, history_xor);
    }
    out.extend(replay(c, plan, &pair.history, &fresh, false)?);
    out.extend(replay(c, plan, &pair.computation, &fresh, false)?);
    out.extend(history_xor.clone());
    let xor = targets
        .iter()
        .map(|(alias, index, original)| gate("cx", [bit(&fresh[alias], *index), original.clone()]))
        .collect();
    out.extend(controlled(&carrier.before, xor));
    out.extend(source_adjoint(history_xor, &mut c.symbols)?);
    out.extend(replay(c, plan, &pair.computation, &fresh, true)?);
    out.extend(replay(c, plan, &pair.history, &fresh, true)?);
    out.extend(fresh.values().rev().map(|n| obj([("relinquish", text(n))])));
    if !history_fresh.is_empty() {
        out.extend(replay(c, plan, &pair.history, &history_fresh, true)?);
        out.extend(
            history_fresh
                .values()
                .rev()
                .map(|n| obj([("relinquish", text(n))])),
        );
    }
    if pair.release {
        out.extend(carrier.nodes);
    } else if matches!(
        plan.rows[pair.carrier].instruction.op.as_str(),
        "quncopy" | "qunload"
    ) {
        out.extend(
            carrier
                .nodes
                .into_iter()
                .filter(|n| n.get("relinquish").is_some()),
        );
    }
    Ok(out)
}
pub(super) fn apply(c: &mut Compiler<'_>, plan: &Plan, body: Vec<Value>) -> Result<Vec<Value>> {
    let mut changes = BTreeMap::new();
    for pair in &plan.pairs {
        let tag = c.compiled[&plan.rows[pair.carrier].key].tag.clone();
        // Non-wire targets retain their instruction-defined inverse (K81).
        let direct = pair.targets.iter().any(|w| w.ends_with("[*]"));
        let fresh = if direct || !pair.perm {
            Vec::new()
        } else {
            replacement(c, plan, pair)?
        };
        for row in &pair.cleanup {
            let compiled = &c.compiled[&plan.rows[*row].key];
            let selected = if direct {
                compiled.nodes.clone()
            } else if *row == pair.carrier {
                fresh.clone()
            } else {
                Vec::new()
            };
            changes.insert(
                compiled.tag.clone(),
                vec![obj([
                    ("source_cleanup", text(&tag)),
                    ("reverse", array(compiled.nodes.clone())),
                    ("recompute", array(selected)),
                    ("perm", Value::Bool(pair.perm)),
                ])],
            );
        }
    }
    fn rewrite(mut v: Value, changes: &BTreeMap<String, Vec<Value>>) -> Value {
        match &mut v {
            Value::Obj(m) => {
                if let Some(Value::Str(tag)) = m.get("for") {
                    if let Some(body) = changes.get(tag) {
                        m.insert("body".into(), array(body.clone()));
                        return v;
                    }
                }
                for value in m.values_mut() {
                    *value = rewrite(value.clone(), changes);
                }
            }
            Value::Arr(a) => {
                for value in a {
                    *value = rewrite(value.clone(), changes);
                }
            }
            _ => {}
        }
        v
    }
    Ok(body.into_iter().map(|v| rewrite(v, &changes)).collect())
}

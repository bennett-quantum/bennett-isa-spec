//! Object-wide checks that run before any per-function or interprocedural
//! analysis: the fixed operand schema of §4.8 (so later passes may index
//! operands without panicking, suite F) and the §5.1 region call graph.
use crate::{err, object::list, validation::normalize_call, Error, Instruction, Object, Result};
use std::collections::{BTreeMap, BTreeSet};

/// (results, operand counts, structured bodies) of one opcode; None = any count.
fn arity(op: &str, structured: bool) -> Option<(Option<usize>, &'static [usize], usize)> {
    use crate::object::{is_binop, is_div};
    let fixed = |lhs: usize, ops: &'static [usize]| Some((Some(lhs), ops, 0));
    if structured {
        return match op {
            "x" | "h" | "s" | "sdg" | "t" | "tdg" => fixed(0, &[1]),
            "cx" | "swap" | "cz" | "mcx" | "rx" | "ry" | "rz" => fixed(0, &[2]),
            "ccx" => fixed(0, &[3]),
            "qalloc" | "qborrow" | "qload" => fixed(1, &[2]),
            "qfree" | "qreturn" | "qpred" | "qunpred" => fixed(0, &[1]),
            "qcopy" => fixed(1, &[1]),
            "quncopy" | "qxor" | "qexch" => fixed(0, &[2]),
            "qinplace" => fixed(0, &[4]),
            "qregion" => fixed(0, &[3, 4]),
            "qunload" | "qstore" | "qexchm" => fixed(0, &[3]),
            "qif" => Some((Some(0), &[1], 2)),
            "qloop" => Some((Some(0), &[1, 3], 1)),
            "oracle_call" => fixed(0, &[7]),
            "action_call" => fixed(0, &[5]),
            _ => None,
        };
    }
    if op.starts_with("const.") || op == "inttoptr" || op == "alloc" || op == "measure" {
        return fixed(1, &[1]);
    }
    if is_binop(op) || is_div(op) || op.starts_with("icmp.") {
        return fixed(1, &[2]);
    }
    if ["zext.", "sext.", "trunc.", "ptrtoint.", "load."]
        .iter()
        .any(|p| op.starts_with(p))
    {
        return fixed(1, &[1]);
    }
    if op.starts_with("store.") {
        return fixed(0, &[2]);
    }
    match op {
        "select" => fixed(1, &[3]),
        "free" | "qreg.free" => fixed(0, &[1]),
        "br" => fixed(0, &[1, 3]),
        "uncall" => fixed(0, &[3]),
        "trap" => fixed(0, &[0]),
        "qreg.new" | "qreg.concat" => fixed(1, &[2]),
        "qreg.slice" | "hostcall" => fixed(1, &[3]),
        "oracle_call" => fixed(0, &[7]),
        "action_call" => fixed(0, &[5]),
        // call: any results, the callee plus any arguments; ret: any values;
        // effect: the device operation plus any operands.
        "call" | "ret" | "effect" => Some((None, &[], 0)),
        _ => None,
    }
}
fn check(i: &Instruction, structured: bool) -> Result<()> {
    let bad = |what: &str| err("§4.8", format!("{} at line {}: {what}", i.op, i.line));
    let instruction = if matches!(i.op.as_str(), "adj" | "ctl") {
        normalize_call(i)?.instruction
    } else {
        i.clone()
    };
    // Unknown or tier-foreign opcodes are reported by the per-function
    // validator with their specific rule (§4.8, §5.1).
    let Some((lhs, operands, bodies)) = arity(&instruction.op, structured) else {
        return Ok(());
    };
    if lhs.is_some_and(|n| n != instruction.lhs.len())
        || (!operands.is_empty() && !operands.contains(&instruction.operands.len()))
    {
        return bad("result/operand count differs from the §4.8 schema");
    }
    if matches!(instruction.op.as_str(), "call" | "effect") && instruction.operands.is_empty() {
        return bad("missing callee or device operation");
    }
    if i.bodies.len() != bodies {
        return bad("structured body count differs from the schema");
    }
    for body in &i.bodies {
        for child in body {
            check(child, structured)?;
        }
    }
    Ok(())
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Edge {
    LiftCall,
    Region,
}
fn symbol(s: Option<&String>) -> Result<&str> {
    s.and_then(|s| s.strip_prefix('@'))
        .ok_or_else(|| Error::new("§4.8: call requires an @symbol operand".into()))
}
fn edges(i: &Instruction, out: &mut Vec<(String, Edge)>) -> Result<()> {
    match i.op.as_str() {
        "lift.call" => out.push((symbol(i.operands.first())?.into(), Edge::LiftCall)),
        "lift.oracle_call" | "lift.oracle_uncall" | "lift.perm_call" => {
            out.push((symbol(i.operands.first())?.into(), Edge::Region))
        }
        "qinplace" => {
            for operand in i.operands.iter().take(2) {
                out.push((symbol(Some(operand))?.into(), Edge::Region));
            }
        }
        "oracle_call" | "action_call" | "adj" | "ctl" => {
            let call = normalize_call(i)?;
            out.push((
                symbol(call.instruction.operands.first())?.into(),
                Edge::Region,
            ));
        }
        _ => {}
    }
    for body in &i.bodies {
        for child in body {
            edges(child, out)?;
        }
    }
    Ok(())
}
impl Object {
    /// §4.8 operand schema for every structured and classical instruction.
    /// Lifted rows are checked by `validate_row`, which reads only callee
    /// signatures; every pass that reads callee bodies runs after this one.
    pub(crate) fn validate_local_schemas(&self) -> Result<()> {
        for f in self.functions.values() {
            if matches!(f.kind.as_str(), "oracle" | "lproc") {
                continue;
            }
            let structured = matches!(f.kind.as_str(), "action" | "perm_action");
            for i in f.instructions() {
                check(i, structured)?;
                if !structured {
                    for operand in &i.operands {
                        if operand.starts_with('(') {
                            list(operand)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
    /// §5.1 "Action call graphs are acyclic; recursion exists only through
    /// lift.call": every cycle of the region call graph (structured region
    /// calls, lifted oracle / permutation calls and qinplace pairs, through
    /// lproc closures) must consist of lift.call edges only.
    pub(crate) fn validate_call_graph(&self) -> Result<()> {
        let mut graph: BTreeMap<&str, Vec<(String, Edge)>> = BTreeMap::new();
        for (name, f) in &self.functions {
            if matches!(f.kind.as_str(), "cc" | "r") {
                continue;
            }
            let mut out = Vec::new();
            for i in f.instructions() {
                edges(i, &mut out)?;
            }
            graph.insert(name, out);
        }
        // Tarjan's strongly connected components.
        struct State<'a> {
            graph: &'a BTreeMap<&'a str, Vec<(String, Edge)>>,
            index: BTreeMap<String, usize>,
            low: BTreeMap<String, usize>,
            stack: Vec<String>,
            on_stack: BTreeSet<String>,
            components: Vec<Vec<String>>,
        }
        fn visit(s: &mut State, v: &str) {
            let n = s.index.len();
            s.index.insert(v.into(), n);
            s.low.insert(v.into(), n);
            s.stack.push(v.into());
            s.on_stack.insert(v.into());
            for (w, _) in s.graph.get(v).into_iter().flatten() {
                if !s.graph.contains_key(w.as_str()) {
                    continue; // an import: resolved and rechecked at link time
                }
                if !s.index.contains_key(w) {
                    visit(s, w);
                    let low = s.low[v].min(s.low[w]);
                    s.low.insert(v.into(), low);
                } else if s.on_stack.contains(w) {
                    let low = s.low[v].min(s.index[w]);
                    s.low.insert(v.into(), low);
                }
            }
            if s.low[v] == s.index[v] {
                let mut component = Vec::new();
                while let Some(w) = s.stack.pop() {
                    s.on_stack.remove(&w);
                    let done = w == v;
                    component.push(w);
                    if done {
                        break;
                    }
                }
                s.components.push(component);
            }
        }
        let mut state = State {
            graph: &graph,
            index: BTreeMap::new(),
            low: BTreeMap::new(),
            stack: Vec::new(),
            on_stack: BTreeSet::new(),
            components: Vec::new(),
        };
        for v in graph.keys() {
            if !state.index.contains_key(*v) {
                visit(&mut state, v);
            }
        }
        for component in &state.components {
            let members: BTreeSet<_> = component.iter().map(String::as_str).collect();
            for v in component {
                for (w, edge) in &graph[v.as_str()] {
                    if members.contains(w.as_str()) && *edge == Edge::Region {
                        return err(
                            "§5.1",
                            format!("region call cycle through @{v} -> @{w}: recursion exists only through lift.call"),
                        );
                    }
                }
            }
        }
        Ok(())
    }
}

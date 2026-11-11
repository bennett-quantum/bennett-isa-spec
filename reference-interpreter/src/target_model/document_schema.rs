//! Closed target productions (§§8.1–8.3). Dynamic matching, capacity and
//! schedule timing are not established by syntactic admission.
use super::{CostExpression, Rational};
use crate::{err, json::Json, Result};
use std::collections::BTreeMap;

pub(super) fn number(v: &Json) -> Result<Rational> {
    let Json::Number(n) = v else {
        return err("§8", "expected target JSON number");
    };
    Rational::decimal(n)
}
fn nonnegative(v: &Json) -> Result<()> {
    if number(v)? < Rational::integer(0) {
        return err("§8", "negative target constant");
    }
    Ok(())
}
fn integer(v: &Json) -> Result<()> {
    nonnegative(v)?;
    if number(v)?.pair().1 != "1" {
        return err("§8", "target count must be an integer");
    }
    Ok(())
}
pub(super) fn unbounded(v: &Json) -> bool {
    matches!(v,Json::String(s) if s=="unbounded")
}
fn count(v: &Json) -> Result<()> {
    if unbounded(v) {
        Ok(())
    } else {
        integer(v)
    }
}
fn boolean(v: &Json) -> Result<()> {
    if matches!(v, Json::Bool(_)) {
        Ok(())
    } else {
        err("§8", "expected target boolean")
    }
}
fn class(v: &Json, classes: &BTreeMap<String, Json>, wildcard: bool) -> Result<()> {
    let name = v.string()?;
    if !classes.contains_key(name) && !(wildcard && name == "*") {
        return err("§8.3", format!("unknown target class {name}"));
    }
    Ok(())
}
fn controls(v: &Json) -> Result<()> {
    v.fields(&["min", "max"], &[])?;
    integer(v.get("min")?)?;
    count(v.get("max")?)?;
    if !unbounded(v.get("max")?) && number(v.get("min")?)? > number(v.get("max")?)? {
        return err("§8.3", "reversed controls range");
    }
    Ok(())
}
fn params(v: &Json) -> Result<()> {
    let args = v.array()?;
    if args.len() != 1 || args[0].string()? != "theta" {
        return err("§8.3", "gate params must declare theta");
    }
    Ok(())
}
fn cost(v: &Json, resources: &BTreeMap<String, Json>) -> Result<()> {
    v.get("time")?;
    for (name, expr) in v.object()? {
        if !resources.contains_key(name) {
            return err("§8.2", format!("unknown cost resource {name}"));
        }
        CostExpression::parse(expr)?;
    }
    Ok(())
}

pub(super) fn validate(t: &Json) -> Result<()> {
    t.fields(
        &[
            "ptr_widths_supported",
            "classes",
            "gates",
            "schemes",
            "qram",
            "capabilities",
            "resources",
            "objective",
            "tables",
            "imports",
        ],
        &["topology"],
    )?;
    for width in t.get("ptr_widths_supported")?.array()? {
        integer(width)?;
    }
    let classes = t.get("classes")?.object()?;
    for c in classes.values() {
        c.fields(&["qubit_second", "dirty_rate", "capacity"], &[])?;
        nonnegative(c.get("qubit_second")?)?;
        nonnegative(c.get("dirty_rate")?)?;
        count(c.get("capacity")?)?;
    }
    let resources = t.get("resources")?.object()?;
    for required in ["qubit_second", "time", "peak_qubits"] {
        t.get("resources")?.get(required)?;
    }
    for resource in resources.values() {
        resource.fields(&["kind", "unit"], &["rate"])?;
        if !["count", "time", "occupancy", "peak"].contains(&resource.get("kind")?.string()?) {
            return err("§8.2", "unknown resource aggregation kind");
        }
        resource.get("unit")?.string()?;
        if let Some(rate) = resource.object()?.get("rate") {
            for (name, value) in rate.object()? {
                class(&Json::String(name.clone()), classes, false)?;
                nonnegative(value)?;
            }
        }
    }
    for gate in t.get("gates")?.array()? {
        gate.fields(
            &["op", "classes", "controls", "epsilon", "cost"],
            &["params", "realisation_model"],
        )?;
        gate.get("op")?.string()?;
        controls(gate.get("controls")?)?;
        for c in gate.get("classes")?.array()? {
            class(c, classes, true)?;
        }
        if let Some(p) = gate.object()?.get("params") {
            params(p)?;
        }
        CostExpression::parse(gate.get("epsilon")?)?;
        cost(gate.get("cost")?, resources)?;
        if let Some(model) = gate.object()?.get("realisation_model") {
            model.get("version")?.string()?;
        }
    }
    for scheme in t.get("schemes")?.array()? {
        validate_scheme(scheme, classes)?;
    }
    let qram = t.get("qram")?;
    if !matches!(qram, Json::Null) {
        qram.fields(&["load", "store", "exch"], &[])?;
        for entry in qram.object()?.values() {
            entry.fields(&["cost"], &[])?;
            cost(entry.get("cost")?, resources)?;
        }
    }
    capabilities(t.get("capabilities")?)?;
    objective(t.get("objective")?, resources)?;
    for table in t.get("tables")?.object()?.values() {
        for n in table.array()? {
            number(n)?;
        }
    }
    for import in t.get("imports")?.object()?.values() {
        import.fields(&["version", "map"], &[])?;
        import.get("version")?.string()?;
        for mapping in import.get("map")?.object()?.values() {
            let pair = mapping.array()?;
            if pair.len() != 2 {
                return err("§8.3", "import map requires resource and factor");
            }
            resource_name(&pair[0], resources)?;
            nonnegative(&pair[1])?;
        }
    }
    if let Some(topology) = t.object()?.get("topology") {
        validate_topology(topology, classes)?;
    }
    Ok(())
}

fn capabilities(v: &Json) -> Result<()> {
    v.fields(
        &[
            "measurement",
            "mbu",
            "max_parallel",
            "max_predicate_depth",
            "classical_step_time",
        ],
        &[],
    )?;
    let m = v.get("measurement")?;
    m.fields(&["mid_circuit", "feedforward_latency"], &[])?;
    boolean(m.get("mid_circuit")?)?;
    nonnegative(m.get("feedforward_latency")?)?;
    if v.get("mbu")? != &Json::Bool(false) {
        return err("§8.3/§6.4", "mbu must be false");
    }
    count(v.get("max_parallel")?)?;
    count(v.get("max_predicate_depth")?)?;
    nonnegative(v.get("classical_step_time")?)
}
fn resource_name(v: &Json, resources: &BTreeMap<String, Json>) -> Result<()> {
    if !resources.contains_key(v.string()?) {
        return err("§8.2", "unknown resource name");
    }
    Ok(())
}
fn objective(v: &Json, resources: &BTreeMap<String, Json>) -> Result<()> {
    if v.object()?.contains_key("weighted") {
        v.fields(&["weighted"], &[])?;
        for (r, w) in v.get("weighted")?.object()? {
            resource_name(&Json::String(r.clone()), resources)?;
            nonnegative(w)?;
        }
    } else {
        v.fields(&["lexicographic"], &[])?;
        for r in v.get("lexicographic")?.array()? {
            resource_name(r, resources)?;
        }
    }
    Ok(())
}
fn validate_scheme(s: &Json, classes: &BTreeMap<String, Json>) -> Result<()> {
    s.fields(&["name", "realises", "ancillae", "template"], &[])?;
    s.get("name")?.string()?;
    let r = s.get("realises")?;
    r.fields(&["op", "controls"], &["params"])?;
    r.get("op")?.string()?;
    controls(r.get("controls")?)?;
    if let Some(p) = r.object()?.get("params") {
        params(p)?;
    }
    let a = s.get("ancillae")?;
    a.fields(&["count", "state", "class"], &[])?;
    CostExpression::parse(a.get("count")?)?;
    class(a.get("class")?, classes, false)?;
    if !["clean", "dirty"].contains(&a.get("state")?.string()?) {
        return err("§8.3", "unknown ancilla state");
    }
    for leaf in s.get("template")?.array()? {
        leaf.fields(&["gate", "wires"], &["params", "repeat"])?;
        leaf.get("gate")?.string()?;
        for wire in leaf.get("wires")?.array()? {
            wire.string()?;
        }
        if let Some(p) = leaf.object()?.get("params") {
            for expr in p.array()? {
                CostExpression::parse(expr)?;
            }
        }
        if let Some(r) = leaf.object()?.get("repeat") {
            r.fields(&["var", "from", "to"], &[])?;
            if r.get("var")?.string()? != "i" {
                return err("§8.3", "repeat variable must be i");
            }
            CostExpression::parse(r.get("from")?)?;
            CostExpression::parse(r.get("to")?)?;
        }
    }
    Ok(())
}
fn validate_topology(v: &Json, classes: &BTreeMap<String, Json>) -> Result<()> {
    v.fields(&["vertices", "edges"], &[])?;
    let vertices = v.get("vertices")?.array()?;
    for vertex in vertices {
        vertex.fields(&["class"], &[])?;
        class(vertex.get("class")?, classes, false)?;
    }
    let mut previous = None;
    for edge in v.get("edges")?.array()? {
        let pair = edge.array()?;
        if pair.len() != 2 {
            return err("§8.3", "topology edge must be a pair");
        }
        integer(&pair[0])?;
        integer(&pair[1])?;
        let u = number(&pair[0])?;
        let w = number(&pair[1])?;
        if u >= w || w >= Rational::integer(vertices.len() as i128) {
            return err("§8.3", "invalid topology vertex indices");
        }
        let edge = (u, w);
        if previous.as_ref().is_some_and(|p| p >= &edge) {
            return err("§8.3", "topology edges must be sorted without duplicates");
        }
        previous = Some(edge);
    }
    Ok(())
}

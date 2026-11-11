//! Ordered B5 slot classes (§9.4), shared by admission and runtime preflight.
use super::{expression::host_index, CostExpression, ExpressionBindings, Rational};
use crate::{err, json::Json, Result};
use std::collections::BTreeMap;

pub fn frame_slot_classes(slot: &Json, inherited_controls: u128) -> Result<Vec<String>> {
    slot.fields(&["id", "role", "width"], &["class", "fragments"])?;
    crate::certificate::expression(slot.get("width")?, &std::collections::BTreeSet::from(["c"]))?;
    let bindings = ExpressionBindings::from([(
        "c".into(),
        Rational::decimal(&inherited_controls.to_string())?,
    )]);
    let width = CostExpression::parse(slot.get("width")?)?.evaluate(&bindings, &BTreeMap::new())?;
    let width = host_index(&width)?;
    if width == 0 {
        return err("§9.4", "zero-width frame slot");
    }
    let obj = slot.object()?;
    let mut classes = crate::reserved(width, "§9.4", "slot class vector")?;
    match (obj.get("class"), obj.get("fragments")) {
        (Some(class), None) => {
            let class = class.string()?;
            if class.is_empty() {
                return err("§9.4", "empty resolved class");
            }
            classes.resize(width, class.into());
        }
        (None, Some(fragments)) => {
            let fragments = fragments.array()?;
            if fragments.len() < 2 {
                return err("§9.4", "one class run must use the class form");
            }
            let mut end = 0u128;
            let mut previous = None;
            for fragment in fragments {
                fragment.fields(&["offset", "width", "class"], &[])?;
                let offset = fragment.get("offset")?.uint()?;
                let count = fragment.get("width")?.uint()?;
                let class = fragment.get("class")?.string()?;
                if offset != end || count == 0 || class.is_empty() || previous == Some(class) {
                    return err(
                        "§9.4",
                        "fragments must be contiguous, positive and maximally merged",
                    );
                }
                end = end.checked_add(count).ok_or_else(|| {
                    crate::resource_error("§9.4", "fragment end exceeds host range")
                })?;
                if end > width as u128 {
                    return err("§9.4", "fragment exceeds slot width");
                }
                classes.resize(end as usize, class.into());
                previous = Some(class);
            }
            if end != width as u128 {
                return err("§9.4", "fragments do not cover the complete slot");
            }
        }
        _ => return err("§9.4", "exactly one of class/fragments is required"),
    }
    Ok(classes)
}

impl super::NativeTarget {
    /// Validate this invocation's interface classes, capacities and placement.
    /// Scratch/history/borrow allocation lifetimes require scheduled accounting
    /// separately; they are not inferred from these interface-only arrays.
    pub fn frame_interface_vertices(
        &self,
        frame: &Json,
        control_classes: &[String],
        supplied: Option<&[Vec<u128>]>,
    ) -> Result<Vec<Vec<u128>>> {
        let declared = self.document.get("classes")?.object()?;
        let mut groups = Vec::new();
        for class in control_classes {
            groups.push(vec![class.clone()]);
        }
        for slot in frame.get("slots")?.array()? {
            groups.push(frame_slot_classes(slot, control_classes.len() as u128)?);
        }
        let mut counts = BTreeMap::<&str, usize>::new();
        for class in groups.iter().flatten() {
            let entry = declared
                .get(class)
                .ok_or_else(|| crate::Error::new(format!("§9.4: undeclared slot class {class}")))?;
            let count = counts.entry(class).or_default();
            *count += 1;
            let cap = entry.get("capacity")?;
            if !super::document_schema::unbounded(cap)
                && Rational::integer(*count as i128) > super::document_schema::number(cap)?
            {
                return err("§8.4/§9.4", "interface exceeds per-class capacity");
            }
        }
        let Some(topology) = self.document.object()?.get("topology") else {
            if supplied.is_some() {
                return err("§11", "placements must be absent without topology");
            }
            return Ok(Vec::new());
        };
        if supplied.is_some_and(|vertices| vertices.len() != groups.len()) {
            return err("§11", "interface placement group count mismatch");
        }
        let vertices = topology.get("vertices")?.array()?;
        let mut used = std::collections::BTreeSet::new();
        let mut result = Vec::new();
        for (index, classes) in groups.iter().enumerate() {
            let requested = supplied.map(|v| &v[index]);
            if requested.is_some_and(|v| v.len() != classes.len()) {
                return err("§11", "interface placement width mismatch");
            }
            let mut resolved = Vec::new();
            for (bit, class) in classes.iter().enumerate() {
                let vertex = if let Some(requested) = requested {
                    requested[bit]
                } else {
                    vertices
                        .iter()
                        .enumerate()
                        .find_map(|(i, v)| {
                            (v.get("class").and_then(Json::string) == Ok(class.as_str())
                                && !used.contains(&(i as u128)))
                            .then_some(i as u128)
                        })
                        .ok_or_else(|| {
                            crate::Error::new("§11: no free vertex of interface class".into())
                        })?
                };
                let actual = usize::try_from(vertex)
                    .ok()
                    .and_then(|i| vertices.get(i))
                    .ok_or_else(|| {
                        crate::Error::new("§11: interface vertex outside target topology".into())
                    })?;
                if actual.get("class")?.string()? != class || !used.insert(vertex) {
                    return err(
                        "§11",
                        "interface vertex class mismatch or overlapping live wires",
                    );
                }
                resolved.push(vertex);
            }
            result.push(resolved);
        }
        Ok(result)
    }
}

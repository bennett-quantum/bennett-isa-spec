use super::{ResolvedSourceInvocation, Value};
use crate::{err, json::Json, Error, Object, Result, Type};
use std::collections::BTreeMap;

pub(crate) fn validate_context_key(key: &Json) -> Result<()> {
    crate::metadata_schema::validate_for(crate::IsaVersion::V09, "resolved_context09", key)?;
    for edge in key.array()? {
        let fields = edge.object()?;
        match (fields.get("step"), fields.get("phase")) {
            (None, None) => {}
            (Some(step), Some(_)) if step.uint()? > 0 => {}
            _ => {
                return err(
                    "§12.1",
                    "context step/phase must be paired with positive step",
                )
            }
        }
    }
    Ok(())
}

enum FrameScope {
    Complete,
    Provided,
}
#[derive(Clone, Debug)]
pub struct SelectedInvocationFrame {
    /// False when a source `any` class still requires target realisation.
    pub interface_classes_established: bool,
    pub context_key: Json,
    pub closure_hash: String,
    pub frame: Json,
}

pub(super) fn select<'a>(region: &'a Json, key: &Json) -> Result<&'a Json> {
    match (
        region.object()?.get("frame"),
        region.object()?.get("frame_contexts"),
    ) {
        (Some(frame), None) => Ok(frame),
        (None, Some(records)) => records
            .array()?
            .iter()
            .find(|r| r.get("context_key") == Ok(key))
            .ok_or_else(|| Error::new("§12.3: missing frame for resolved context".into()))?
            .get("frame"),
        _ => err("§12.3", "exactly one of frame/frame_contexts is required"),
    }
}

impl Object {
    /// Independently expand the complete action scope, select descriptors and
    /// check interface classes for every invocation, including empty bodies.
    /// Non-slot allocation/lifetime accounting remains a separate obligation.
    pub fn selected_action_frames(
        &self,
        name: &str,
        args: &[Value],
        bundles: &[(&str, usize)],
        adjoint: bool,
        aliases: Option<&[u64]>,
        root_classes: &[Vec<String>],
    ) -> Result<Vec<SelectedInvocationFrame>> {
        let contexts = self.source_action_invocations(name, args, bundles, adjoint, aliases)?;
        self.select_invocation_frames(&contexts, root_classes)
    }
    pub(crate) fn select_invocation_frames(
        &self,
        contexts: &[ResolvedSourceInvocation],
        root_classes: &[Vec<String>],
    ) -> Result<Vec<SelectedInvocationFrame>> {
        self.check_invocation_frame_scope(contexts, root_classes, FrameScope::Complete)
    }
    pub(crate) fn select_provided_invocation_frames(
        &self,
        contexts: &[ResolvedSourceInvocation],
        root_classes: &[Vec<String>],
    ) -> Result<Vec<SelectedInvocationFrame>> {
        self.check_invocation_frame_scope(contexts, root_classes, FrameScope::Provided)
    }
    fn check_invocation_frame_scope(
        &self,
        contexts: &[ResolvedSourceInvocation],
        root_classes: &[Vec<String>],
        scope: FrameScope,
    ) -> Result<Vec<SelectedInvocationFrame>> {
        let certificate = self.section("certificate").ok_or_else(|| {
            Error::new("§12.3: frame checking requires certificate records".into())
        })?;
        let regions = certificate.get("regions")?.object()?;
        let identities = self.identity()?;
        let root = contexts
            .iter()
            .find(|c| c.context_key == Json::Array(vec![]))
            .ok_or_else(|| Error::new("§12.1: missing root invocation".into()))?;
        let root_name = identities
            .closure_hashes
            .iter()
            .find_map(|(n, h)| (h.as_deref() == Some(&root.closure_hash)).then_some(n))
            .ok_or_else(|| Error::new("§12.3: unresolved frame root".into()))?;
        let root_function = &self.functions[root_name];
        let params = root_function
            .params
            .iter()
            .flatten()
            .filter(|p| matches!(p.ty, Type::Wire(_) | Type::Region(..)))
            .collect::<Vec<_>>();
        if params.len() != root_classes.len() {
            return err("§9.4", "runtime root slot class count differs");
        }
        let mut classes = BTreeMap::new();
        for (p, actual) in params.iter().zip(root_classes) {
            let wires = root.wires.get(&p.name).ok_or_else(|| {
                Error::new("§9.4: region wire flattening is unestablished".into())
            })?;
            if wires.len() != actual.len() {
                return err("§9.4", "runtime slot class width differs");
            }
            for (wire, class) in wires.iter().zip(actual) {
                if classes.insert(wire.clone(), class.clone()).is_some() {
                    return err("§9.4", "root slots alias");
                }
            }
        }
        // Compare record coverage to the source tree, never to emitted leaves.
        for (closure, region) in regions {
            for field in ["frame_contexts", "resource_contexts"] {
                if let Some(records) = region.object()?.get(field) {
                    let mut expected = contexts
                        .iter()
                        .filter(|c| c.closure_hash == *closure)
                        .map(|c| c.context_key.canonical())
                        .collect::<Vec<_>>();
                    expected.sort();
                    let actual = records
                        .array()?
                        .iter()
                        .map(|r| r.get("context_key").map(Json::canonical))
                        .collect::<Result<Vec<_>>>()?;
                    if actual != expected {
                        return err(
                            "§12.3",
                            "frame contexts do not equal complete reached context set",
                        );
                    }
                }
            }
        }
        let mut selected = Vec::new();
        for context in contexts {
            let region = match regions.get(&context.closure_hash) {
                Some(region) => region,
                None if matches!(scope, FrameScope::Provided) => continue,
                None => return err("§12.3", "reached invocation has no region frame"),
            };
            let frame = select(region, &context.context_key)?;
            let name = identities
                .closure_hashes
                .iter()
                .find_map(|(n, h)| (h.as_deref() == Some(&context.closure_hash)).then_some(n))
                .ok_or_else(|| Error::new("§12.3: unresolved frame closure".into()))?;
            let function = &self.functions[name];
            crate::certificate::frame09(
                frame,
                function,
                &[context.binding.inherited_controls as u128],
            )?;
            let params = function
                .params
                .iter()
                .flatten()
                .filter(|p| matches!(p.ty, Type::Wire(_) | Type::Region(..)));
            let mut interface_classes_established = true;
            for (slot, p) in frame.get("slots")?.array()?.iter().zip(params) {
                let wires = context.wires.get(&p.name).ok_or_else(|| {
                    Error::new(
                        "§9.4: interface region class correspondence is unestablished".into(),
                    )
                })?;
                let expected = crate::target::frame_slot_classes(
                    slot,
                    context.binding.inherited_controls as u128,
                )?;
                if wires.len() != expected.len() {
                    return err(
                        "§9.4",
                        "selected invocation slot width differs from actual wires",
                    );
                }
                for (wire, class) in wires.iter().zip(expected) {
                    let actual = classes
                        .get(wire)
                        .or_else(|| context.allocation_classes.get(wire))
                        .ok_or_else(|| {
                            Error::new("§9.4: missing source allocation class".into())
                        })?;
                    if actual == "any" {
                        interface_classes_established = false;
                        continue;
                    }
                    if *actual != class {
                        return err(
                            "§9.4",
                            "selected invocation frame class differs from inherited wire class",
                        );
                    }
                }
            }
            selected.push(SelectedInvocationFrame {
                interface_classes_established,
                context_key: context.context_key.clone(),
                closure_hash: context.closure_hash.clone(),
                frame: frame.clone(),
            });
        }
        Ok(selected)
    }
}

//! Typed schedule/coverage validation and canonical program-point resolution.
use crate::{
    err, json::Json, validation::normalize_call, Error, Function, Instruction, Object, Result,
};

pub(crate) fn hash(value: &Json) -> Result<&str> {
    crate::metadata_schema::validate("hash", value)?;
    value.string()
}
pub(crate) fn text(value: &Json) -> Result<&str> {
    crate::metadata_schema::validate("text", value)?;
    value.string()
}
/// A variant identifier: §6.4/§12.3 key cleanup, error and multiplicity entries
/// by "<variant>:<inherited_controls>" (and "<hash>:<variant>:<controls>"), so a
/// variant is a nonempty string without ':' (SPEC-QUESTIONS 11). The
/// specification does not enumerate variant names.
pub(crate) fn variant(value: &Json) -> Result<&str> {
    let s = text(value)?;
    if s.contains(':') {
        return err(
            "§12.3",
            "a variant contains no ':' (keys are <variant>:<controls>)",
        );
    }
    Ok(s)
}
pub(crate) fn nonnegative(value: &Json) -> Result<()> {
    crate::metadata_schema::validate("decimal", value)
}

fn selector(value: &Json, positive: bool) -> Result<()> {
    if value == &Json::String("*".into()) {
        return Ok(());
    }
    if positive && value.uint()? == 0 {
        return err("§11", "step numbers start at one");
    }
    value.uint()?;
    Ok(())
}
fn contextual<T>(rule: &str, f: impl FnOnce() -> Result<T>) -> Result<T> {
    f().map_err(|e| Error(format!("{rule}: {e}")))
}
pub(crate) struct Point<'a> {
    pub(crate) function: &'a Function,
    pub(crate) instruction: &'a Instruction,
    loops: Vec<u64>,
}
impl Object {
    pub(crate) fn point<'a>(
        &'a self,
        symbol: &str,
        block: &str,
        ordinal: u128,
    ) -> Result<Point<'a>> {
        hash(&Json::String(symbol.into()))?;
        let candidates = self
            .symbols()?
            .values()
            .filter(|s| s.get("interface_hash").and_then(Json::string) == Ok(symbol));
        fn find<'a>(
            seq: &'a [Instruction],
            ordinal: u128,
            cursor: &mut u128,
            loops: &mut Vec<u64>,
        ) -> Result<Option<(&'a Instruction, Vec<u64>)>> {
            for i in seq {
                let current = *cursor;
                *cursor += 1;
                if current == ordinal {
                    return Ok(Some((i, loops.clone())));
                }
                if i.op == "qloop" {
                    loops.push(crate::object::host_count(
                        &i.operands[0],
                        "qloop trip count",
                    )?);
                }
                for body in &i.bodies {
                    if let Some(p) = find(body, ordinal, cursor, loops)? {
                        return Ok(Some(p));
                    }
                }
                if i.op == "qloop" {
                    loops.pop();
                }
            }
            Ok(None)
        }
        let mut found = None;
        for candidate in candidates {
            let name = candidate.get("name")?.string()?;
            if let Some(f) = self.functions.get(name) {
                if let Some(b) = f.blocks.iter().find(|b| b.name == block) {
                    if let Some((instruction, loops)) =
                        find(&b.instructions, ordinal, &mut 0, &mut Vec::new())?
                    {
                        if found.is_some() {
                            return err("§9.2", "ambiguous program-point symbol hash");
                        }
                        found = Some(Point {
                            function: f,
                            instruction,
                            loops,
                        });
                    }
                }
            }
        }
        found.ok_or_else(|| Error("§9.2: program point does not identify an instruction".into()))
    }
    pub(crate) fn validate_path(&self, path: &Json, root: Option<&str>) -> Result<()> {
        let mut expected = root.map(str::to_string);
        for element in path.array()? {
            let (block, ordinal) = text(element.get("point")?)?
                .rsplit_once(':')
                .ok_or_else(|| Error("§11: point must be block:ordinal".into()))?;
            let ordinal = ordinal
                .parse::<u128>()
                .map_err(|_| Error("§11: invalid instruction ordinal".into()))?;
            let point = self.point(hash(element.get("symbol")?)?, block, ordinal)?;
            if expected.as_ref().is_some_and(|n| n != &point.function.name) {
                return err("§11", "path is not a region invocation chain");
            }
            let iteration = element.get("iteration")?;
            if iteration != &Json::String("*".into()) {
                let indices = iteration.array()?;
                if indices.len() != point.loops.len() {
                    return err("§11", "iteration vector differs from loop nesting depth");
                }
                for (index, bound) in indices.iter().zip(&point.loops) {
                    if index.uint()? >= *bound as u128 {
                        return err("§11", "loop iteration selector out of bounds");
                    }
                }
            }
            expected = if point.instruction.op == "lift.call"
                || matches!(
                    point.instruction.op.as_str(),
                    "lift.oracle_call" | "lift.oracle_uncall" | "lift.perm_call"
                ) {
                Some(point.instruction.operands[0].trim_start_matches('@').into())
            } else if let Ok(call) = normalize_call(point.instruction) {
                Some(call.instruction.operands[0].trim_start_matches('@').into())
            } else {
                Some(String::new())
            };
        }
        Ok(())
    }
    pub(crate) fn validate_schedule_details(&self, s: &Json) -> Result<()> {
        contextual("§11", || {
            let b = s.get("bindings")?;

            for key in ["closure_hash", "target_hash", "params_hash"] {
                hash(b.get(key)?)?;
            }
            let region = text(b.get("region")?)?.trim_start_matches('@');
            let f = self
                .functions
                .get(region)
                .filter(|f| matches!(f.kind.as_str(), "oracle" | "action" | "perm_action"))
                .ok_or_else(|| Error("§11: schedule region is not defined".into()))?;
            let identity = self.identity()?;
            if identity.closure_hashes[region].as_deref() != Some(hash(b.get("closure_hash")?)?) {
                return err("§11", "schedule closure binding mismatch");
            }
            variant(b.get("variant")?)?;
            b.get("inherited_controls")?.uint()?;
            nonnegative(b.get("epsilon")?)?;
            let budget = b.get("budget")?.array()?;
            let values = budget.iter().map(Json::uint).collect::<Result<Vec<_>>>()?;
            if f.kind == "oracle" {
                self.validate_budget(crate::Budget {
                    steps: u64::try_from(values[0])
                        .map_err(|_| Error("§11: step budget exceeds u64".into()))?,
                    heap: values[1],
                    stack: values[2],
                    depth: values[3],
                })?;
                // Exact permutation regions have no approximation budget.
                if b.get("epsilon")?.canonical() != "0"
                    && b.get("epsilon")? != &Json::String("0".into())
                {
                    return err("§5.2", "oracle schedules must be exact");
                }
            }
            let entries = s.get("entries")?.array()?;
            for e in entries {
                self.validate_path(e.get("path")?, Some(region))?;
                selector(e.get("step")?, true)?;
                selector(e.get("generated")?, false)?;
                let k = e.get("kind")?.string()?;
                for key in ["cleanup", "gate", "alloc", "start"] {
                    if key != k && e.object()?.contains_key(key) {
                        return err("§11", "entry has a payload for another kind");
                    }
                }
                match k {
                    "cleanup" | "gate" | "start" => {
                        e.get(k)?;
                    }
                    "alloc" => {
                        let p = e.get(k)?;

                        if text(p.get("class")?)? == "any" {
                            return err("§11", "allocation class must be concrete");
                        }
                    }
                    _ => return err("§11", "invalid entry kind"),
                }
            }
            for (i, a) in entries.iter().enumerate() {
                for other in &entries[i + 1..] {
                    if a.get("kind")? == other.get("kind")?
                        && specificity(a)? == specificity(other)?
                        && overlaps(a, other)?
                    {
                        return err("§11", "multiple maximal matching schedule entries");
                    }
                }
            }
            Ok(())
        })
    }
    pub(crate) fn validate_coverage_details(&self, c: &Json) -> Result<()> {
        contextual("§13", || {
            for e in c.get("entries")?.array()? {
                let point = e.get("point")?.array()?;
                self.point(hash(&point[0])?, text(&point[1])?, point[2].uint()?)?;
                text(e.get("source")?)?;
                e.get("reason")?.string()?;
                e.get("count")?.uint()?;
                let fields = e.object()?;
                if let Some(h) = fields.get("shim_hash") {
                    hash(h)?;
                }
                if let Some(code) = fields.get("fault_code") {
                    if code.uint()? == 0 {
                        return err("§13", "trapped coverage requires a nonzero fault code");
                    }
                }
                if e.get("classification")?.string()? == "trapped"
                    && !fields.contains_key("fault_code")
                {
                    return err("§13", "trapped coverage is missing fault_code");
                }
                if e.get("classification")?.string()? == "shimmed"
                    && !fields.contains_key("shim_hash")
                {
                    return err("§13", "shimmed coverage is missing shim_hash");
                }
            }
            Ok(())
        })
    }
}
fn wildcard(v: &Json) -> bool {
    v == &Json::String("*".into())
}
fn specificity(e: &Json) -> Result<(usize, usize, bool, bool, bool)> {
    let path = e.get("path")?.array()?;
    Ok((
        path.len(),
        path.iter()
            .filter(|p| p.get("iteration").is_ok_and(|i| !wildcard(i)))
            .count(),
        !wildcard(e.get("step")?),
        !wildcard(e.get("phase")?),
        !wildcard(e.get("generated")?),
    ))
}
fn overlaps(a: &Json, b: &Json) -> Result<bool> {
    for key in ["step", "phase", "generated"] {
        let x = a.get(key)?;
        let y = b.get(key)?;
        if !wildcard(x) && !wildcard(y) && x != y {
            return Ok(false);
        }
    }
    for (x, y) in a.get("path")?.array()?.iter().zip(b.get("path")?.array()?) {
        if x.get("symbol")? != y.get("symbol")? || x.get("point")? != y.get("point")? {
            return Ok(false);
        }
        let x = x.get("iteration")?;
        let y = y.get("iteration")?;
        if !wildcard(x) && !wildcard(y) && x != y {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Select one schedule entry by §11 specificity; the object was validated at parse.
impl Object {
    pub fn resolve_schedule(&self, operation: &Json, kind: &str) -> Result<Option<&Json>> {
        let Some(schedule) = self.section("schedule") else {
            return Ok(None);
        };
        let mut result = None;
        for entry in schedule.get("entries")?.array()? {
            if entry.get("kind")?.string()? != kind
                || entry.get("path")?.array()?.len() > operation.get("path")?.array()?.len()
                || !overlaps(entry, operation)?
            {
                continue;
            }
            let better = match result {
                None => true,
                Some(prior) => specificity(entry)? > specificity(prior)?,
            };
            if better {
                result = Some(entry);
            }
        }
        Ok(result)
    }
}

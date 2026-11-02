//! Matching against bound source events, with §11 K52 invocation-level selection.
use super::*;
use bennett_isa_conformance::ref_language::{Leaf,SourceLocation};
fn vector_matches(selector:&Json,actual:&[usize])->Result<bool> {
    if wildcard(selector) {return Ok(true);}
    let values=selector.array()?;
    Ok(values.len()==actual.len() && values.iter().zip(actual).all(|(a,b)|a.uint()==Ok(*b as u128)))
}
fn matches(entry:&Json,leaf:&Leaf,source:&SourceLocation)->Result<bool> {
    let path=entry.get("path")?.array()?;
    if path.len()>source.path.len()+1 {return Ok(false);}
    for (index,selector) in path.iter().enumerate() {
        let edge=source.path.get(index);
        let point=edge.map_or(&source.terminal,|edge|&edge.point);
        if selector.get("symbol")?.string()?!=point.symbol || selector.get("point")?.string()?!=point.point
            || !vector_matches(selector.get("iteration")?,&point.iteration)? {return Ok(false);}
        if let Some(instance)=selector.object()?.get("instance") {
            if !vector_matches(instance,&point.instance)? {return Ok(false);}
        }
        if let Some(index)=selector.object()?.get("call_index").filter(|index|!wildcard(index)) {
            if edge.is_none_or(|edge|index.uint()!=Ok(edge.call_index as u128)) {return Ok(false);}
        }
    }
    // A terminal call point does not enter its callee; a proper prefix does.
    let depth=path.len().min(source.path.len())+1;
    let owner=source.invocations.iter().filter(|scope|scope.oracle && scope.invocation.len()<=depth).max_by_key(|scope|scope.invocation.len());
    let action_support=source.invocations.last().is_some_and(|scope|!scope.oracle && depth>=scope.invocation.len())
        && matches!(source.terminal.point.as_str(),"$construct"|"$release");
    let step=if action_support {None} else {owner.and_then(|owner|source.steps.iter().find(|step|step.invocation==owner.invocation))};
    if !wildcard(entry.get("step")?) && step.is_none_or(|step|entry.get("step").and_then(Json::uint)!=Ok(step.step as u128)) {return Ok(false);}
    if !wildcard(entry.get("phase")?) && step.is_none_or(|step|entry.get("phase").and_then(Json::string)!=Ok(if step.reverse {"rev"}else{"fwd"})) {return Ok(false);}
    Ok(wildcard(entry.get("generated")?) || entry.get("generated")?.uint()?==leaf.ordinal as u128)
}
impl Object {
    /// Select a decision on an actual source-derived event. This is the
    /// matching boundary; target realisation and charging are separate.
    pub fn resolve_source_leaf(&self,leaf:&Leaf,kind:&str)->Result<Option<&Json>> {
        let Some(schedule)=self.section("schedule") else {return Ok(None);};
        let source=leaf.source.as_deref().ok_or_else(||Error::new("§11: generated operation lacks its source address".into()))?;
        let applicable=match kind {
            "gate"=>leaf.allocation.is_none() && leaf.event.is_none(),
            "alloc"=>leaf.gate=="alloc",
            "start"=>true,
            "cleanup"=>false,
            _=>return err("§11","unknown decision kind"),
        };
        if !applicable {return Ok(None);}
        let mut selected=None;let mut maxima=0;
        for entry in schedule.get("entries")?.array()? {
            if entry.get("kind")?.string()?!=kind || !matches(entry,leaf,source)? {continue;}
            let score=specificity(entry)?;
            let better=match selected {None=>true,Some(previous)=>score>specificity(previous)?};
            if better {selected=Some(entry);maxima=1;}
            else if let Some(previous)=selected {if score==specificity(previous)? {maxima+=1;}}
        }
        if maxima>1 {return err("§11","multiple maximal matching schedule entries");}
        Ok(selected)
    }
}

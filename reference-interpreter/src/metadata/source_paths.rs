//! Versioned §11 source paths. Lifted procedure instances share their oracle's
//! invocation; only semantic region calls introduce another path element.
use super::*;

pub(super) fn support_point(point: &str) -> bool {
    matches!(point, "$construct" | "$step" | "$commit" | "$release")
}
fn calls(instruction: &Instruction) -> Result<Vec<String>> {
    if instruction.op == "qinplace" {
        return Ok(instruction.operands[..2].iter().map(|name| name.trim_start_matches('@').to_string()).collect());
    }
    if matches!(instruction.op.as_str(), "lift.oracle_call" | "lift.oracle_uncall" | "lift.perm_call") {
        return Ok(vec![instruction.operands[0].trim_start_matches('@').into()]);
    }
    Ok(normalize_call(instruction).ok().map(|call| vec![call.instruction.operands[0].trim_start_matches('@').into()]).unwrap_or_default())
}
fn instance_matches(element: &Json, keys: &[Vec<usize>]) -> Result<bool> {
    let selector=element.object()?.get("instance");
    if selector.is_none_or(wildcard) {return Ok(!keys.is_empty());}
    let key=selector.unwrap().array()?.iter().map(|value| {
        usize::try_from(value.uint()?).map_err(|_| Error::new("§11: fixed binding selector exceeds the entry position space".into()))
    }).collect::<Result<Vec<_>>>()?;
    Ok(keys.contains(&key))
}
impl Object {
    pub(super) fn validate_source_path(&self, path: &Json, root: &str) -> Result<()> {
        let identities=self.identity()?;
        let mut scopes=vec![root.to_string()];
        for element in path.array()? {
            let spelling=text(element.get("point")?)?;
            let symbol=hash(element.get("symbol")?)?;
            if support_point(spelling) {
                let valid=scopes.iter().any(|name| self.functions.get(name).is_some_and(|f|f.kind=="oracle" || (matches!(f.kind.as_str(),"action"|"perm_action") && matches!(spelling,"$construct"|"$release"))) && identities.interface_hashes.get(name).is_some_and(|hash|hash==symbol));
                if !valid || !instance_matches(element,&[vec![]])? {return err("§11","support point is not in the selected source invocation");}
                if !wildcard(element.get("iteration")?) && !element.get("iteration")?.array()?.is_empty() {return err("§11","support points have no source-loop iterations");}
                if element.object()?.get("call_index").is_some_and(|value|!wildcard(value)) {return err("§11","support point has no semantic call occurrence");}
                scopes.clear();
                continue;
            }
            let (block,ordinal)=spelling.rsplit_once(':').ok_or_else(||Error::new("§11: point must be block:ordinal or a reserved oracle support point".into()))?;
            let point=self.point(symbol,block,ordinal.parse().map_err(|_|Error::new("§11: invalid instruction ordinal".into()))?)?;
            let mut keys=Vec::new();
            for scope in &scopes {
                let function=self.functions.get(scope).ok_or_else(||Error::new("§11: unresolved source invocation".into()))?;
                if function.kind=="oracle" {keys.extend(crate::oracle::Program::binding_keys(self,scope,&point.function.name)?);}
                else if scope==&point.function.name {keys.push(vec![]);}
            }
            if !instance_matches(element,&keys)? {return err("§11","point or fixed binding instance is outside the selected invocation");}
            let iteration=element.get("iteration")?;
            if !wildcard(iteration) {
                let indices=iteration.array()?;
                if indices.len()!=point.loops.len() {return err("§11","iteration vector differs from loop nesting depth");}
                for (index,bound) in indices.iter().zip(&point.loops) {
                    if index.uint()?>=u128::from(*bound) {return err("§11","loop iteration selector out of bounds");}
                }
            }
            let targets=calls(point.instruction)?;
            scopes=if let Some(selector)=element.object()?.get("call_index").filter(|value|!wildcard(value)) {
                let index=usize::try_from(selector.uint()?).map_err(|_|Error::new("§11: semantic call occurrence is out of bounds".into()))?;
                vec![targets.get(index).cloned().ok_or_else(||Error::new("§11: semantic call occurrence is out of bounds".into()))?]
            } else {targets};
        }
        Ok(())
    }
}

impl Object {
    /// Keep fixed-binding alternatives separate when subtracting higher-entry
    /// coverage. Covering one instance cannot cover a wildcard's other keys.
    pub(super) fn source_chains(&self,path:&Json,root:&str)->Result<Vec<Vec<Json>>> {
        fn key_json(key:&[usize])->Json {Json::Array(key.iter().map(|n|Json::Number(n.to_string())).collect())}
        fn ordinals(sequence:&[Instruction],cursor:&mut usize,out:&mut Vec<usize>) {
            for instruction in sequence {*cursor+=1;out.push(*cursor-1);for body in &instruction.bodies {ordinals(body,cursor,out);}}
        }
        let identities=self.identity()?;
        let root_function=&self.functions[root];
        if path.array()?.is_empty() {
            let mut out=Vec::new();
            for function in self.functions.values() {
                let keys=if root_function.kind=="oracle" {crate::oracle::Program::binding_keys(self,root,&function.name)?} else if function.name==root {vec![vec![]]} else {vec![]};
                for key in keys {
                    for block in &function.blocks {
                        let mut points=Vec::new();ordinals(&block.instructions,&mut 0,&mut points);
                        for ordinal in points {
                            out.push(vec![Json::Object(std::collections::BTreeMap::from([
                                ("symbol".into(),Json::String(identities.interface_hashes[&function.name].clone())),
                                ("point".into(),Json::String(format!("{}:{ordinal}",block.name))),
                                ("iteration".into(),Json::String("*".into())),("instance".into(),key_json(&key)),
                            ]))]);
                        }
                    }
                }
            }
            let support: &[&str]=if root_function.kind=="oracle" {&["$construct","$step","$commit","$release"]}
                else if !crate::validation::region_declarations(root_function).is_empty() {&["$construct","$release"]} else {&[]};
            {
                for point in support {
                    out.push(vec![Json::Object(std::collections::BTreeMap::from([
                        ("symbol".into(),Json::String(identities.interface_hashes[root].clone())),("point".into(),Json::String((*point).into())),
                        ("iteration".into(),Json::Array(vec![])),("instance".into(),Json::Array(vec![])),
                    ]))]);
                }
            }
            return Ok(out);
        }
        let mut alternatives=vec![(Vec::new(),vec![root.to_string()])];
        for (position,element) in path.array()?.iter().enumerate() {
            let spelling=element.get("point")?.string()?;
            let mut next=Vec::new();
            for (prefix,scopes) in alternatives {
                let (keys,targets)=if support_point(spelling) {(vec![vec![]],Vec::new())} else {
                    let (block,ordinal)=spelling.rsplit_once(':').ok_or_else(||Error::new("§11: invalid point".into()))?;
                    let point=self.point(element.get("symbol")?.string()?,block,ordinal.parse().map_err(|_|Error::new("§11: invalid ordinal".into()))?)?;
                    let mut keys=Vec::new();
                    for scope in &scopes {
                        if self.functions[scope].kind=="oracle" {keys.extend(crate::oracle::Program::binding_keys(self,scope,&point.function.name)?);}
                        else if scope==&point.function.name {keys.push(vec![]);}
                    }
                    (keys,calls(point.instruction)?)
                };
                for key in keys {
                    if !instance_matches(element,std::slice::from_ref(&key))? {continue;}
                    let mut concrete=element.clone();
                    if let Json::Object(fields)=&mut concrete {fields.insert("instance".into(),key_json(&key));}
                    if position+1==path.array()?.len() {
                        let mut chain=prefix.clone();chain.push(concrete);next.push((chain,Vec::new()));
                    } else {
                        for (index,target) in targets.iter().enumerate() {
                            if element.object()?.get("call_index").filter(|value|!wildcard(value)).is_some_and(|value|value.uint()!=Ok(index as u128)) {continue;}
                            let mut edge=concrete.clone();if let Json::Object(fields)=&mut edge {fields.insert("call_index".into(),Json::Number(index.to_string()));}
                            let mut chain=prefix.clone();chain.push(edge);next.push((chain,vec![target.clone()]));
                        }
                    }
                }
            }
            alternatives=next;
        }
        Ok(alternatives.into_iter().map(|(chain,_)|chain).collect())
    }
}

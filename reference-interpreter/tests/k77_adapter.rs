//! §15 H / K77 finite source-builder adapter witness, not a general ISA expander.
//! Context is out-of-band: the catalog and inc8's [x,t,f] interface stay intact.
mod support;
use bennett_isa_conformance::{
    json::{self, Json},
    ref_language::{Expander, Leaf, Registry},
};
use bennett_isa_ref::{Budget, Object, OracleInput, OracleMachine, QuantumState};

fn j(s: &str) -> Json {
    json::parse(s).unwrap()
}

#[derive(Clone)]
struct CapturedCall {
    arguments: Json,
    oracle_invocation: Option<Json>,
    source_token: Budget,
    call_index: usize,
}
impl CapturedCall {
    fn new(b: u64, call_index: usize) -> Self {
        Self {
            arguments: j(r#"["x","t","f"]"#),
            oracle_invocation: Some(j(&format!(r#"{{"token":["{b}","0","0","0"]}}"#))),
            source_token: Budget::steps(b),
            call_index,
        }
    }
    fn budget(&self) -> Result<Budget, String> {
        let record = self
            .oracle_invocation
            .as_ref()
            .ok_or("missing oracle_invocation")?;
        let Json::Obj(fields) = record else {
            return Err("malformed record".into());
        };
        if fields.len() != 1 {
            return Err("malformed record fields".into());
        }
        let parts = record
            .get("token")
            .and_then(Json::as_arr)
            .ok_or("malformed token")?;
        if parts.len() != 4 {
            return Err("malformed token length".into());
        }
        let mut values = Vec::new();
        for part in parts {
            let s = part.as_str().ok_or("malformed token component")?;
            if s.is_empty()
                || !s.bytes().all(|b| b.is_ascii_digit())
                || (s.len() > 1 && s.starts_with('0'))
            {
                return Err("noncanonical token component".into());
            }
            values.push(
                s.parse::<u128>()
                    .map_err(|_| "token exceeds this finite witness")?,
            );
        }
        let b = Budget {
            steps: u64::try_from(values[0]).map_err(|_| "B exceeds this finite witness")?,
            heap: values[1],
            stack: values[2],
            depth: values[3],
        };
        if b != self.source_token {
            return Err("source-inconsistent oracle_invocation".into());
        }
        Ok(b)
    }
}

// A finite linked-source builder for the printed const/add/retval inc8 only.
// The generated gates are derived from increment's carry equations. The test
// checks them against the independent literal frame machine over every x.
fn inc8_body(call: &CapturedCall) -> Result<Json, String> {
    let b = call.budget()?;
    match b.steps {
        2 => Ok(j(r#"[{"gate":"x","wires":["f[0]"]}]"#)),
        3 => {
            let mut body = vec![j(r#"{"gate":"x","wires":["t[0]"]}"#)];
            for bit in 0..8 {
                body.push(j(&format!(
                    r#"{{"gate":"cx","wires":["x[{bit}]","t[{bit}]"]}}"#
                )));
                if bit > 0 {
                    let controls = (0..bit)
                        .map(|i| format!("\"x[{i}]\""))
                        .collect::<Vec<_>>()
                        .join(",");
                    body.push(j(&format!(
                        r#"{{"gate":"mcx","wires":[{controls},"t[{bit}]"]}}"#
                    )));
                }
            }
            Ok(Json::Arr(body))
        }
        _ => Err("finite inc8 witness needs B=2 or B=3".into()),
    }
}

// Explicit source-binding scope, independent of positional catalog arguments.
// Captures own the complete source call. Applying under another scope cannot
// substitute its tuple. Nested builders restore even when the child rejects.
#[derive(Default)]
struct SourceBuilder {
    current: Option<CapturedCall>,
}
impl SourceBuilder {
    fn within<T>(
        &mut self,
        call: &CapturedCall,
        f: impl FnOnce(&mut Self) -> Result<T, String>,
    ) -> Result<T, String> {
        call.budget()?;
        let outer = self.current.replace(call.clone());
        let result = f(self);
        self.current = outer;
        result
    }
    fn expand(
        &mut self,
        call: &CapturedCall,
        hash: &str,
        route: &str,
    ) -> Result<Vec<Leaf>, String> {
        self.within(call, |scope| {
            let body = inc8_body(scope.current.as_ref().unwrap())?;
            let parameters = j(r#"[{"name":"x","type":"wire","width":8},{"name":"t","type":"wire","width":8},{"name":"f","type":"wire","width":4}]"#);
            let mut registry = Registry::new();
            registry.insert(hash.into(), j(&format!(r#"{{"parameters":{parameters},"body":{body}}}"#)));
            // Actual argument/body/template capture and forwarding use the
            // catalog interpreter, including conflicting apply-site wire names.
            let recipes = r#"[
              {"kind":"apply_body","parameters":[{"name":"B","type":"body"},{"name":"x","type":"wire","width":8}],"body":[{"splice":"B"}]},
              {"kind":"apply_template","parameters":[{"name":"T","type":"templates","length":1},{"name":"x","type":"wire","width":8}],"body":[{"apply":"T","index":0,"args":[]}]},
              {"kind":"forward","parameters":[{"name":"S","type":"symbol"},{"name":"A","type":"arguments","symbol":"S"}],"body":[{"call":"S","args":"A"}]},
              {"kind":"root","parameters":[{"name":"S","type":"symbol"},{"name":"A","type":"arguments","symbol":"S"},{"name":"P","type":"wire","width":8}],"body":BODY}
            ]"#;
            let body = match route {
                "arguments" => r#"[{"invoke":"forward","args":["S","A"]}]"#,
                "body" => r#"[{"invoke":"apply_body","args":[[{"invoke":"forward","args":["S","A"]}],"P"]}]"#,
                "template" => r#"[{"invoke":"apply_template","args":[[{"parameters":[],"body":[{"invoke":"forward","args":["S","A"]}]}],"P"]}]"#,
                _ => return Err("unknown witness route".into()),
            };
            let recipes = j(&recipes.replace("BODY", body));
            let Json::Arr(mut recipes) = recipes else { unreachable!() };
            for recipe in &mut recipes {
                let Json::Obj(m) = recipe else { unreachable!() };
                m.insert("scratch".into(), j("[]"));
                m.insert("removable".into(), j("[]"));
                m.insert("source".into(), j(r#""§15 H / K77 inc8 adapter witness""#));
            }
            let doc = format!(r#"{{"isa":"0.9","name":"ref","version":"0.9","recipes":{}}}"#, Json::Arr(recipes));
            Expander::new(&doc, registry)?.expand("root", &[Json::Str(hash.into()), call.arguments.clone(), Json::Str("poison".into())], &[("x",8),("t",8),("f",4),("poison",8)])
        })
    }
}

#[test]
fn suite_h_k77_captures_forwarding_nesting_and_inverse() {
    let object = Object::parse(include_str!("fixtures/inc8.bisa")).unwrap();
    let hash = object.identity().unwrap().closure_hashes["inc8"]
        .clone()
        .unwrap();
    let calls = [CapturedCall::new(2, 0), CapturedCall::new(3, 1)];
    let mut builder = SourceBuilder::default();
    let unrelated = CapturedCall::new(99, 99);
    for route in ["arguments", "body", "template"] {
        let instantiated = builder
            .within(&unrelated, |builder| {
                let first = builder.expand(&calls[0], &hash, route)?;
                assert_eq!(
                    builder.current.as_ref().unwrap().budget()?,
                    Budget::steps(99)
                );
                let second = builder.within(&calls[0], |builder| {
                    let child = builder.expand(&calls[1], &hash, route)?;
                    assert_eq!(
                        builder.current.as_ref().unwrap().budget()?,
                        Budget::steps(2)
                    );
                    Ok(child)
                })?;
                assert_eq!(
                    builder.current.as_ref().unwrap().budget()?,
                    Budget::steps(99)
                );
                Ok(vec![(calls[0].clone(), first), (calls[1].clone(), second)])
            })
            .unwrap();
        assert!(builder.current.is_none());
        for x in 0..256 {
            let mut q = QuantumState::new();
            let wires = [
                q.allocate(8).unwrap(),
                q.allocate(8).unwrap(),
                q.allocate(4).unwrap(),
                q.allocate(8).unwrap(),
            ];
            for bit in 0..8 {
                if x >> bit & 1 != 0 {
                    q.gate("x", &[wires[0][bit]], None, &[]).unwrap();
                }
            }
            let before = q.clone();
            let apply = |q: &mut QuantumState, leaf: &Leaf| {
                let resolve = |(name, bit): &(String, usize)| {
                    wires[match name.as_str() {
                        "x" => 0,
                        "t" => 1,
                        "f" => 2,
                        "poison" => 3,
                        _ => panic!("unexpected wire"),
                    }][*bit]
                };
                let operands = leaf.wires.iter().map(resolve).collect::<Vec<_>>();
                assert!(leaf.controls.is_empty());
                q.gate(&leaf.gate, &operands, None, &[]).unwrap();
            };
            for (call, leaves) in &instantiated {
                let budget = call.budget().unwrap();
                let mut machine =
                    OracleMachine::new(&object, "inc8", &OracleInput::bits(&[x]), budget).unwrap();
                let entry = machine.snapshot();
                for _ in 0..budget.steps {
                    machine.forward_step().unwrap();
                }
                assert!(
                    machine.forward_step().is_err(),
                    "exactly B frames: {}",
                    budget.steps
                );
                let expected = machine
                    .commit(
                        q.read_basis(&wires[1]).unwrap(),
                        q.read_basis(&wires[2]).unwrap(),
                    )
                    .unwrap();
                for leaf in leaves {
                    apply(&mut q, leaf);
                }
                assert_eq!(
                    (
                        q.read_basis(&wires[1]).unwrap(),
                        q.read_basis(&wires[2]).unwrap()
                    ),
                    expected
                );
                for _ in 0..budget.steps {
                    machine.inverse_step().unwrap();
                }
                assert_eq!(machine.snapshot(), entry);
                assert!(machine.frames_clean());
            }
            assert_eq!(
                (
                    q.read_basis(&wires[1]).unwrap(),
                    q.read_basis(&wires[2]).unwrap()
                ),
                ((x + 1) & 255, 1)
            );
            let mut inverse_indices = Vec::new();
            // Raw inverse traverses instantiated forward captures and leaves;
            // it does not reinstantiate or renumber calls in traversal order.
            for (call, leaves) in instantiated.iter().rev() {
                inverse_indices.push(call.call_index);
                for leaf in leaves.iter().rev() {
                    apply(&mut q, leaf);
                }
            }
            assert_eq!(inverse_indices, [1, 0]);
            assert_eq!(q, before, "{route}, x={x}: complete inverse restoration");
        }
    }
}

#[test]
fn suite_h_k77_rejects_absent_malformed_and_source_inconsistent_records() {
    let mut builder = SourceBuilder::default();
    let outer = CapturedCall::new(3, 0);
    let bad = [
        None,
        Some(j("[]")),
        Some(j(r#"{}"#)),
        Some(j(r#"{"token":["2","0","0"]}"#)),
        Some(j(r#"{"token":[2,"0","0","0"]}"#)),
        Some(j(r#"{"token":["02","0","0","0"]}"#)),
        Some(j(r#"{"token":["2","0","0","0"],"extra":0}"#)),
        Some(j(r#"{"token":["3","0","0","0"]}"#)),
    ];
    builder
        .within(&outer, |builder| {
            for record in bad {
                let mut call = CapturedCall::new(2, 1);
                call.oracle_invocation = record;
                assert!(builder.expand(&call, &"a".repeat(64), "arguments").is_err());
                assert_eq!(
                    builder.current.as_ref().unwrap().budget()?,
                    Budget::steps(3)
                );
            }
            Ok(())
        })
        .unwrap();
    assert!(builder.current.is_none());
}

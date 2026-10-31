//! Independent invocation shapes and §11 oracle; no production domain helpers.
use super::{json, object, rehash};
use bennett_isa_ref::{json::Json, Object};

pub const TWO: &str = include_str!("r18-nested-budget-2.bisa");
pub const THREE: &str = include_str!("r18-nested-budget-3.bisa");
pub const OH: &str = "8e6e852c4717ad55763ec2162b0582896a948c073b649c40002a8aa64dd0f815";
pub const IH: &str = "9687f21af62158ff87ab8a974e056774ee75b531e158cc4538a11d4631f89012";
pub const ORDERS: [[usize; 3]; 6] = [
    [0, 1, 2],
    [0, 2, 1],
    [1, 0, 2],
    [1, 2, 0],
    [2, 0, 1],
    [2, 1, 0],
];
pub fn set(v: &mut Json, k: &str, next: Json) {
    let Json::Object(m) = v else {
        panic!("expected object")
    };
    m.insert(k.into(), next);
}
pub fn point(symbol: &str, ordinal: usize) -> Json {
    json(&format!(
        r#"{{"symbol":"{symbol}","point":"entry:{ordinal}","iteration":[]}}"#
    ))
}
pub fn gate(path: &[Json], step: Json) -> Json {
    let mut e = json(
        r#"{"path":[],"step":"*","phase":"*","generated":"*","kind":"gate","gate":{"realisation":"native"}}"#,
    );
    set(&mut e, "path", Json::Array(path.to_vec()));
    set(&mut e, "step", step);
    e
}
pub fn score(e: &Json) -> (usize, usize, bool, bool, bool) {
    let p = e.get("path").unwrap().array().unwrap();
    let specific = |v: &Json| v != &json(r#""*""#);
    (
        p.len(),
        p.iter()
            .filter(|p| specific(p.get("iteration").unwrap()))
            .count(),
        specific(e.get("step").unwrap()),
        specific(e.get("phase").unwrap()),
        specific(e.get("generated").unwrap()),
    )
}
pub fn winners(entries: &[Json], op: &Json) -> Vec<usize> {
    let path = op.get("path").unwrap().array().unwrap();
    let matching: Vec<_> = entries
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            let ep = e.get("path").unwrap().array().unwrap();
            ep.len() <= path.len()
                && ep.iter().zip(path).all(|(a, b)| {
                    ["symbol", "point"]
                        .iter()
                        .all(|k| a.get(k).unwrap() == b.get(k).unwrap())
                        && (a.get("iteration").unwrap() == &json(r#""*""#)
                            || a.get("iteration").unwrap() == b.get("iteration").unwrap())
                })
                && ["step", "phase", "generated"].iter().all(|k| {
                    e.get(k).unwrap() == &json(r#""*""#) || e.get(k).unwrap() == op.get(k).unwrap()
                })
        })
        .collect();
    let best = matching.iter().map(|(_, e)| score(e)).max();
    matching
        .into_iter()
        .filter(|(_, e)| Some(score(e)) == best)
        .map(|(i, _)| i)
        .collect()
}

/// O edges start a token-governed machine; L edges share the current machine.
/// Valid mixed chains have an oracle prefix and lproc suffix (lprocs have no
/// token parameters). The leaf has const/add/ret; all other nodes call/ret.
pub fn chain(edges: &[char], action: bool) -> Object {
    let mut owned = Vec::new();
    let tokens = edges.iter().filter(|&&edge| edge == 'O').count();
    let parameters = (0..tokens)
        .map(|i| format!("tk{i}:token"))
        .collect::<Vec<_>>()
        .join(", ");
    let arguments = (0..tokens)
        .map(|i| format!("tk{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let types = Json::Array((0..tokens).map(|_| Json::String("token".into())).collect());
    for index in 0..=edges.len() {
        let name = format!("n{index}");
        let callee = format!("n{}", index + 1);
        let lproc = index > 0 && edges[index - 1] == 'L';
        let sig = if index == 0 && action {
            r#"{"abi":"0.8","args":["token"],"bounds":[],"in":[8],"kind":"action","mem":[],"mut":[8,4],"tier":"q"}"#
        } else if lproc {
            r#"{"abi":"0.8","kind":"lproc","params":[8],"result":8,"tier":"q"}"#
        } else {
            r#"{"abi":"0.8","args":["token"],"fault":4,"in":[8],"kind":"oracle","mem":[],"tgt":8,"tier":"q"}"#
        };
        let mut signature = json(sig);
        if !lproc {
            set(&mut signature, "args", types.clone());
        }
        let token = format!(
            "tk{}",
            edges[..index].iter().filter(|&&edge| edge == 'O').count()
        );
        let body = if index == 0 && action {
            assert_eq!(edges[0], 'O');
            format!("action @{name} (t:wire[8] mut, f:wire[4] mut) (x:wire[8] in) () ({parameters}) ()\n^entry():\noracle_call @{callee} {token} (x) t f () ({arguments})")
        } else {
            let header = if lproc {
                format!("lproc @{name} (x:reg[8]) (r:reg[8]) () () ()")
            } else {
                format!("oracle @{name} (x:wire[8] in) (t:wire[8]) (f:wire[4]) () ({parameters})")
            };
            let ret = if lproc { "lift.ret" } else { "lift.retval" };
            let rows = if index == edges.len() {
                format!("one = lift.const.i8 1\nr = lift.add x one\n{ret} r")
            } else if edges[index] == 'O' {
                format!("lift.oracle_call @{callee} {token} (x) r fr () ({arguments})\n{ret} r")
            } else {
                format!("r = lift.call @{callee} (x)\n{ret} r")
            };
            // r is already a declared result location in an lproc.
            let regs = if lproc {
                "one:reg[8]"
            } else {
                "one:reg[8], r:reg[8], fr:reg[4]"
            };
            format!("{header}\nregs {{ {regs} }}\n^entry():\n{rows}")
        };
        owned.push((name, signature.canonical(), body));
    }
    let refs: Vec<_> = owned
        .iter()
        .map(|(n, s, b)| (n.as_str(), s.as_str(), b.as_str()))
        .collect();
    rehash(&object(&refs, "{}", "\nsection layout\n{\"A_tab\":2,\"H\":5,\"S_cells\":8,\"globals\":{},\"heap_base\":16,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":16,\"ptr_width\":32}\n"))
}

pub fn schedule(o: &Object, budget: u64, token_budgets: &[u64]) -> Json {
    let id = o.identity().unwrap();
    // Hash the supplied concrete tokens independently as JCS values. Production
    // admission sees ONLY this hash, so it cannot recover their step bounds.
    let params = Json::Array(
        token_budgets
            .iter()
            .map(|b| {
                json(&format!(
                    r#"{{"steps":{b},"heap":0,"stack":256,"depth":16}}"#
                ))
            })
            .collect(),
    );
    json(&format!(
        r#"{{"bindings":{{"budget":[{budget},0,256,16],"closure_hash":"{}","epsilon":0,"inherited_controls":0,"params_hash":"{}","region":"n0","scheduler":{{"name":"test","version":"1"}},"target_hash":"{}","variant":"forward"}},"entries":[]}}"#,
        id.closure_hashes["n0"].as_ref().unwrap(),
        bennett_isa_ref::sha256_hex(params.canonical().as_bytes()),
        "3".repeat(64)
    ))
}
pub fn document(o: &Object, s: &Json) -> String {
    format!("{}\nsection schedule\n{}\n", o.text, s.canonical())
}

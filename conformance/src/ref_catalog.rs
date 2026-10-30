//! Independent ref constructor expander and basis permutation machine.
//! Reads catalog bytes only. No reference interpreter code or subprocess calls.
use crate::json::{self, Json};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) type Env = BTreeMap<String, i64>;
fn field<'a>(v: &'a Json, k: &str) -> Result<&'a Json, String> {
    v.get(k).ok_or_else(|| format!("missing {k}: {v}"))
}
fn string(v: &Json) -> Result<&str, String> {
    v.as_str().ok_or_else(|| format!("expected string: {v}"))
}
fn array(v: &Json) -> Result<&[Json], String> {
    v.as_arr().ok_or_else(|| format!("expected array: {v}"))
}
fn keys(v: &Json, expected: &[&str]) -> Result<(), String> {
    let Json::Obj(m) = v else {
        return Err("constructor must be object".into());
    };
    let a: BTreeSet<_> = m.keys().map(String::as_str).collect();
    let b: BTreeSet<_> = expected.iter().copied().collect();
    if a != b {
        return Err(format!("closed constructor keys: {a:?} != {b:?}"));
    }
    Ok(())
}
pub(crate) fn scalar(v: &Json, env: &Env) -> Result<i64, String> {
    match v {
        Json::Num(n) if n.fract() == 0.0 && n.abs() < (1u64 << 53) as f64 => Ok(*n as i64),
        Json::Str(s) => env
            .get(s)
            .copied()
            .ok_or_else(|| format!("unbound integer {s}")),
        Json::Arr(a) if a.len() == 3 => {
            let x = scalar(&a[1], env)?;
            let y = scalar(&a[2], env)?;
            match string(&a[0])? {
                "+" => x.checked_add(y),
                "-" => x.checked_sub(y),
                "*" => x.checked_mul(y),
                "div" if y > 0 => Some(x.div_euclid(y)),
                "mod" if y > 0 => Some(x.rem_euclid(y)),
                _ => None,
            }
            .ok_or_else(|| "invalid/overflowed integer expression".into())
        }
        _ => Err(format!("invalid scalar {v}")),
    }
}
pub(crate) fn index(s: &str, env: &Env) -> Result<(String, usize), String> {
    let (name, expr) = s.split_once('[').ok_or("wire needs a subscript")?;
    let expr = expr.strip_suffix(']').ok_or("unclosed subscript")?;
    let v = if env.contains_key(expr) {
        Json::Str(expr.into())
    } else {
        json::parse(expr)?
    };
    let i = usize::try_from(scalar(&v, env)?).map_err(|_| "negative wire index")?;
    Ok((name.into(), i))
}
#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
    Gate(String, Vec<(String, usize)>),
    Alloc(String, usize),
    Release(String, usize),
    Classical(String, String),
}
impl Operation {
    fn adjoint(&self) -> Result<Self, String> {
        Ok(match self {
            Self::Alloc(n, w) => Self::Release(n.clone(), *w),
            Self::Release(n, w) => Self::Alloc(n.clone(), *w),
            Self::Gate(g, w) => Self::Gate(
                match g.as_str() {
                    "s" => "sdg",
                    "sdg" => "s",
                    "t" => "tdg",
                    "tdg" => "t",
                    "x" | "cx" | "ccx" | "mcx" | "swap" | "h" | "cz" => g,
                    _ => return Err(format!("unsupported adjoint {g}")),
                }
                .into(),
                w.clone(),
            ),
            Self::Classical(..) => return Err("classical inverse requires history contract".into()),
        })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Leaf {
    pub ordinal: usize,
    pub operation: Operation,
}
pub struct Catalog {
    document: Json,
}
impl Catalog {
    pub fn parse(bytes: &str) -> Result<Self, String> {
        let document = json::parse(bytes)?;
        keys(&document, &["isa", "name", "version", "recipes"])?;
        for (k, v) in [("isa", "0.9"), ("name", "ref"), ("version", "0.9")] {
            if string(field(&document, k)?)? != v {
                return Err(format!("wrong {k}"));
            }
        }
        let mut previous = None;
        for r in array(field(&document, "recipes")?)? {
            keys(
                r,
                &[
                    "kind",
                    "parameters",
                    "body",
                    "scratch",
                    "removable",
                    "source",
                ],
            )?;
            let kind = string(field(r, "kind")?)?;
            if previous.is_some_and(|p| p >= kind) {
                return Err("recipe order/duplicate kind".into());
            }
            previous = Some(kind);
            if !array(field(r, "removable")?)?.is_empty() {
                return Err("removable must be empty".into());
            }
        }
        Ok(Self { document })
    }
    pub fn kinds(&self) -> Vec<&str> {
        self.document
            .get("recipes")
            .unwrap()
            .as_arr()
            .unwrap()
            .iter()
            .map(|r| r.get("kind").unwrap().as_str().unwrap())
            .collect()
    }
    fn recipe(&self, kind: &str) -> Result<&Json, String> {
        array(field(&self.document, "recipes")?)?
            .iter()
            .find(|r| r.get("kind").and_then(Json::as_str) == Some(kind))
            .ok_or_else(|| format!("unknown recipe {kind}"))
    }
    pub fn expand(&self, kind: &str, args: &[i64]) -> Result<Vec<Leaf>, String> {
        let mut ops = Vec::new();
        self.invoke(kind, args, &mut Vec::new(), &mut ops)?;
        Ok(ops
            .into_iter()
            .enumerate()
            .map(|(ordinal, operation)| Leaf { ordinal, operation })
            .collect())
    }
    fn invoke(
        &self,
        kind: &str,
        args: &[i64],
        chain: &mut Vec<String>,
        out: &mut Vec<Operation>,
    ) -> Result<(), String> {
        if chain.iter().any(|s| s == kind) {
            return Err("cyclic recipe dependency".into());
        }
        let r = self.recipe(kind)?;
        let params = array(field(r, "parameters")?)?;
        if params.len() != args.len() {
            return Err("parameter arity".into());
        }
        let env = params
            .iter()
            .zip(args)
            .map(|(p, v)| Ok((string(p)?.to_owned(), *v)))
            .collect::<Result<Env, String>>()?;
        chain.push(kind.into());
        self.body(
            array(field(r, "body")?)?,
            &env,
            chain,
            &mut BTreeMap::new(),
            out,
        )?;
        chain.pop();
        Ok(())
    }
    fn body(
        &self,
        b: &[Json],
        env: &Env,
        chain: &mut Vec<String>,
        widths: &mut BTreeMap<String, usize>,
        out: &mut Vec<Operation>,
    ) -> Result<(), String> {
        for c in b {
            if let Some(g) = c.get("gate") {
                keys(c, &["gate", "wires"])?;
                let wires = array(field(c, "wires")?)?
                    .iter()
                    .map(|s| index(string(s)?, env))
                    .collect::<Result<_, String>>()?;
                out.push(Operation::Gate(string(g)?.into(), wires));
            } else if let Some(n) = c.get("alloc") {
                keys(c, &["alloc", "width", "class", "state"])?;
                if string(field(c, "class")?)? != "any" || string(field(c, "state")?)? != "clean" {
                    return Err("unsupported allocation state/class".into());
                }
                let width = usize::try_from(scalar(field(c, "width")?, env)?)
                    .map_err(|_| "negative allocation width")?;
                widths.insert(string(n)?.into(), width);
                out.push(Operation::Alloc(string(n)?.into(), width));
            } else if let Some(n) = c.get("release") {
                keys(c, &["release"])?;
                let n = string(n)?;
                let w = *widths
                    .get(n)
                    .ok_or_else(|| format!("release {n} has no allocation metadata"))?;
                out.push(Operation::Release(n.into(), w));
            } else if let Some(i) = c.get("for") {
                keys(c, &["for", "from", "to", "body"])?;
                let from = scalar(field(c, "from")?, env)?;
                let to = scalar(field(c, "to")?, env)?;
                if to < from {
                    return Err("reversed loop range".into());
                }
                for j in from..to {
                    let mut inner = env.clone();
                    inner.insert(string(i)?.into(), j);
                    self.body(array(field(c, "body")?)?, &inner, chain, widths, out)?;
                }
            } else if let Some(v) = c.get("inverse") {
                keys(c, &["inverse", "body"])?;
                if v.as_bool() != Some(true) {
                    return Err("inverse marker must be true".into());
                }
                let mut inner = Vec::new();
                self.body(array(field(c, "body")?)?, env, chain, widths, &mut inner)?;
                for op in inner.iter().rev() {
                    out.push(op.adjoint()?);
                }
            } else if let Some(k) = c.get("invoke") {
                keys(c, &["invoke", "args"])?;
                let args = array(field(c, "args")?)?
                    .iter()
                    .map(|a| scalar(a, env))
                    .collect::<Result<Vec<_>, _>>()?;
                // Integer-only invocation; no invented wire/body substitution.
                self.invoke(string(k)?, &args, chain, out)?;
            } else if c.get("classical").is_some() {
                keys(c, &["classical", "point"])?;
                out.push(Operation::Classical(
                    string(field(c, "classical")?)?.into(),
                    string(field(c, "point")?)?.into(),
                ));
            } else {
                return Err(format!("unknown closed constructor {c}"));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
enum Instruction {
    Gate(String, Vec<usize>),
    Alloc(Vec<usize>),
    Release(Vec<usize>),
}
/// Compile wire names to bit indices once, then enumerate all input states.
/// Each allocation occurrence receives new indices, including reused local names.
pub struct Machine {
    instructions: Vec<(usize, Instruction)>,
    pub inputs: BTreeMap<String, Vec<usize>>,
    pub outputs: BTreeMap<String, Vec<usize>>,
    bits: usize,
}
impl Machine {
    pub fn compile(leaves: &[Leaf], bundles: &[(&str, usize)]) -> Result<Self, String> {
        let mut live = BTreeMap::new();
        let mut bits = 0;
        for (name, n) in bundles {
            let ids = (bits..bits + n).collect::<Vec<_>>();
            bits += n;
            if live.insert((*name).to_string(), ids).is_some() {
                return Err("duplicate bundle".into());
            }
        }
        let inputs = live.clone();
        let mut instructions = Vec::new();
        for l in leaves {
            let ins = match &l.operation {
                Operation::Alloc(n, w) => {
                    let ids = (bits..bits + w).collect::<Vec<_>>();
                    bits += w;
                    if live.insert(n.clone(), ids.clone()).is_some() {
                        return Err(format!("allocation already live {n}"));
                    }
                    Instruction::Alloc(ids)
                }
                Operation::Release(n, w) => {
                    let ids = live
                        .remove(n)
                        .ok_or_else(|| format!("release not live {n}"))?;
                    if ids.len() != *w {
                        return Err("release width mismatch".into());
                    }
                    Instruction::Release(ids)
                }
                Operation::Gate(g, w) => {
                    let ids = w
                        .iter()
                        .map(|(n, i)| {
                            live.get(n)
                                .and_then(|v| v.get(*i))
                                .copied()
                                .ok_or_else(|| format!("wire not live/in range {n}[{i}]"))
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    if ids.iter().collect::<BTreeSet<_>>().len() != ids.len() {
                        return Err("coincident gate operands".into());
                    }
                    let arity = match g.as_str() {
                        "x" => 1,
                        "cx" | "swap" => 2,
                        "ccx" => 3,
                        "mcx" => ids.len(),
                        _ => return Err(format!("{g} is not a basis permutation")),
                    };
                    if arity != ids.len() || ids.is_empty() {
                        return Err("gate arity".into());
                    }
                    Instruction::Gate(g.clone(), ids)
                }
                Operation::Classical(..) => {
                    return Err("classical events are not quantum permutations".into())
                }
            };
            instructions.push((l.ordinal, ins));
        }
        Ok(Self {
            instructions,
            inputs,
            outputs: live,
            bits,
        })
    }
    pub fn state(&self) -> Vec<bool> {
        vec![false; self.bits]
    }
    pub fn set(&self, state: &mut [bool], name: &str, value: u64) {
        for (i, b) in self.inputs[name].iter().enumerate() {
            state[*b] = value >> i & 1 != 0;
        }
    }
    pub fn get(&self, state: &[bool], name: &str) -> u64 {
        self.inputs
            .get(name)
            .or_else(|| self.outputs.get(name))
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, b)| u64::from(state[*b]) << i)
            .sum()
    }
    pub fn run(&self, state: &mut [bool], enabled: bool, reverse: bool) -> Result<(), String> {
        let apply =
            |(_, ins): &(usize, Instruction), s: &mut [bool], rev: bool| -> Result<(), String> {
                match ins {
                    Instruction::Gate(g, w) if enabled => {
                        if g == "swap" {
                            s.swap(w[0], w[1]);
                        } else if w[..w.len() - 1].iter().all(|i| s[*i]) {
                            s[w[w.len() - 1]] ^= true;
                        }
                    }
                    Instruction::Alloc(w) | Instruction::Release(w) if w.iter().any(|i| s[*i]) => {
                        return Err(format!(
                            "dirty {}",
                            if matches!(ins, Instruction::Release(_)) ^ rev {
                                "release"
                            } else {
                                "allocation"
                            }
                        ));
                    }
                    _ => {}
                }
                Ok(())
            };
        if reverse {
            for i in self.instructions.iter().rev() {
                apply(i, state, true)?;
            }
        } else {
            for i in &self.instructions {
                apply(i, state, false)?;
            }
        }
        Ok(())
    }
}
impl Machine {
    /// Sixty-four independent basis tuples; each bit position is one lane.
    pub fn packed_state(&self) -> Vec<u64> {
        vec![0; self.bits]
    }
    pub fn set_lane(&self, s: &mut [u64], name: &str, value: u64, lane: usize) {
        assert!(lane < 64);
        for (i, b) in self.inputs[name].iter().enumerate() {
            s[*b] |= ((value >> i) & 1) << lane;
        }
    }
    pub fn get_lane(&self, s: &[u64], name: &str, lane: usize) -> u64 {
        self.inputs
            .get(name)
            .or_else(|| self.outputs.get(name))
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, b)| ((s[*b] >> lane) & 1) << i)
            .sum()
    }
    pub fn run_packed(&self, s: &mut [u64], enabled: u64, reverse: bool) -> Result<(), String> {
        let apply = |(_, ins): &(usize, Instruction), s: &mut [u64]| -> Result<(), String> {
            match ins {
                Instruction::Gate(g, w) => {
                    if g == "swap" {
                        let d = (s[w[0]] ^ s[w[1]]) & enabled;
                        s[w[0]] ^= d;
                        s[w[1]] ^= d;
                    } else {
                        let mask = w[..w.len() - 1].iter().fold(enabled, |v, i| v & s[*i]);
                        s[w[w.len() - 1]] ^= mask;
                    }
                }
                Instruction::Alloc(w) | Instruction::Release(w) if w.iter().any(|i| s[*i] != 0) => {
                    return Err("dirty packed lifetime boundary".into())
                }
                _ => {}
            }
            Ok(())
        };
        if reverse {
            for i in self.instructions.iter().rev() {
                apply(i, s)?;
            }
        } else {
            for i in &self.instructions {
                apply(i, s)?;
            }
        }
        Ok(())
    }
}

impl Machine {
    pub fn symbolic_state(&self) -> Vec<usize> {
        vec![0; self.bits]
    }
    /// A reduced ordered BDD denotes every input tuple simultaneously. Equality
    /// under domain is exact: a nonzero difference has a concrete counterexample.
    pub fn run_symbolic(
        &self,
        b: &mut crate::ref_bdd::Bdd,
        s: &mut [usize],
        enabled: usize,
        reverse: bool,
        domain: usize,
    ) -> Result<(), String> {
        let apply = |(_, ins): &(usize, Instruction),
                     b: &mut crate::ref_bdd::Bdd,
                     s: &mut [usize]|
         -> Result<(), String> {
            match ins {
                Instruction::Gate(g, w) => {
                    if g == "swap" {
                        let d = b.xor(s[w[0]], s[w[1]]);
                        let d = b.and(d, enabled);
                        s[w[0]] = b.xor(s[w[0]], d);
                        s[w[1]] = b.xor(s[w[1]], d);
                    } else {
                        let mut mask = enabled;
                        for i in &w[..w.len() - 1] {
                            mask = b.and(mask, s[*i]);
                        }
                        let t = *w.last().unwrap();
                        s[t] = b.xor(s[t], mask);
                    }
                }
                Instruction::Alloc(w) | Instruction::Release(w) => {
                    for i in w {
                        if b.and(domain, s[*i]) != 0 {
                            return Err("symbolically dirty lifetime boundary".into());
                        }
                    }
                }
            }
            Ok(())
        };
        if reverse {
            for i in self.instructions.iter().rev() {
                apply(i, b, s)?;
            }
        } else {
            for i in &self.instructions {
                apply(i, b, s)?;
            }
        }
        Ok(())
    }
}

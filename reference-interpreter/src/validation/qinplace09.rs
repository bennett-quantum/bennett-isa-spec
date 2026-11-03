//! K4 signature, read-dependency and identity checks; old ISA uses its own arm.
use super::*;
impl Object {
    pub(super) fn validate_qinplace09(&self, i: &Instruction, s: &mut Discipline) -> Result<()> {
        let a = &i.operands;
        if !i.lhs.is_empty() || !matches!(a.len(), 4 | 7) {
            return err(
                "§4.5 K4",
                "qinplace requires four operands or all three lists",
            );
        }
        let inputs = if a.len() == 7 {
            list(&a[4])?
        } else {
            Vec::new()
        };
        let regions = if a.len() == 7 {
            list(&a[5])?
        } else {
            Vec::new()
        };
        let args = if a.len() == 7 {
            list(&a[6])?
        } else {
            Vec::new()
        };
        if s.types.get(&a[2]) != Some(&Type::Token) {
            return err("§4.5 K4", "qinplace token operand");
        }
        let mut operands = vec![a[3].as_str()];
        operands.extend(inputs.iter().map(String::as_str));
        s.disjoint(&operands)?;
        let wires = operands
            .iter()
            .map(|n| s.wires(n))
            .collect::<Result<Vec<_>>>()?;
        let context = s
            .effective_controls()?
            .into_iter()
            .map(|(w, _, _)| w)
            .collect::<BTreeSet<_>>();
        if wires.iter().flatten().any(|w| context.contains(w)) {
            return err(
                "§4.5 K4",
                "qinplace inputs/target overlap effective controls",
            );
        }
        let n = wires[0].len();
        let mut fault = None;
        for name in &a[..2] {
            let f = self
                .declaration(name.strip_prefix('@').ok_or_else(|| {
                    Error::new("§4.5 K4: qinplace requires @oracle names".into())
                })?)?;
            if f.kind != "oracle"
                || f.params[0].len() != wires.len()
                || f.params[1][0].ty != Type::Wire(n as u64)
                || f.params[3].len() != regions.len()
                || f.params[4].len() != args.len()
            {
                return err("§4.5 K4", "qinplace signature/list arity mismatch");
            }
            for (p, w) in f.params[0].iter().zip(&wires) {
                if p.ty != Type::Wire(w.len() as u64) {
                    return err("§4.5 K4", "qinplace input width mismatch");
                }
            }
            if fault.get_or_insert(f.params[2][0].ty.clone()) != &f.params[2][0].ty {
                return err("§4.5 K4", "qinplace fault widths differ");
            }
            let mut unique = BTreeSet::new();
            for (p, name) in f.params[3].iter().zip(&regions) {
                if p.role.as_deref() != Some("in")
                    || s.types.get(name) != Some(&p.ty)
                    || !unique.insert(name)
                {
                    return err("§4.5 K4", "qinplace memory signature/alias mismatch");
                }
            }
            for (p, name) in f.params[4].iter().zip(&args) {
                if s.types.get(name) != Some(&p.ty) {
                    return err("§4.5 K4", "qinplace classical argument type mismatch");
                }
            }
        }
        let mut reads = wires[1..].concat();
        reads.extend(regions.iter().map(|n| format!("{n}[*]")));
        let mut matching = a.clone();
        if !s.token_identities.is_empty() {
            let key = |n: &str| {
                s.token_identities
                    .get(n)
                    .map_or_else(|| n.to_string(), |id| format!("$token-identity/{id}"))
            };
            matching[2] = key(&a[2]);
            if a.len() == 7 {
                matching[6] = format!(
                    "({})",
                    args.iter()
                        .map(|n| if s.types[n] == Type::Token {
                            key(n)
                        } else {
                            n.clone()
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }
        let version = format!(
            "qinplace:{}:{:?}:{:?}",
            matching.join(" "),
            s.read_snapshot(&reads)?,
            s.effective_controls()?
        );
        s.note_reads(&version, &reads)?;
        s.change(&a[3], version, false)
    }
}

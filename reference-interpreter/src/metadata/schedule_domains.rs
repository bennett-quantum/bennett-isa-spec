//! §11 maximal-match admission over selector domains, without loop expansion.
use super::*;

type Domain = Vec<(u128, u128)>;

#[derive(Clone, Copy)]
enum InvocationSteps {
    Root(u128),
    // A token is a classical argument; its preimage is absent at admission.
    // Still respect the ABI's u64 step limit, independently of the root B.
    Nested,
    Action,
}
impl InvocationSteps {
    fn upper(self) -> u128 {
        match self {
            Self::Root(budget) => budget,
            Self::Nested => u128::from(u64::MAX),
            Self::Action => u128::MAX,
        }
    }
}

fn intersection(a: &Domain, b: &Domain) -> Option<Domain> {
    let out: Domain = a
        .iter()
        .zip(b)
        .map(|(x, y)| (x.0.max(y.0), x.1.min(y.1)))
        .collect();
    out.iter().all(|(lo, hi)| lo <= hi).then_some(out)
}

/// Partition A \ B into disjoint boxes. Boundaries come only from selectors;
/// cost is independent of numeric loop/step bounds. Repeating subtraction
/// checks coverage by the UNION of all higher entries, not one overlapping C.
fn subtract(mut a: Domain, b: &Domain) -> Vec<Domain> {
    let Some(common) = intersection(&a, b) else {
        return vec![a];
    };
    let mut out = Vec::new();
    for (i, &(lo, hi)) in common.iter().enumerate() {
        if a[i].0 < lo {
            let mut part = a.clone();
            part[i].1 = lo - 1;
            out.push(part);
            a[i].0 = lo;
        }
        if hi < a[i].1 {
            let mut part = a.clone();
            part[i].0 = hi + 1;
            out.push(part);
            a[i].1 = hi;
        }
    }
    out
}

impl Object {
    /// Each point has its own generated operations (§11), including call/loop
    /// rows. Start at the shortest chains; a new oracle invocation can broaden
    /// their step domain, so has_uncovered_tie also visits those extensions.
    /// The empty prefix is split over every root instruction (not a fictitious
    /// operation with an empty chain). validate_path already checked call edges.
    fn shortest_chains(&self, path: &Json, region: &str) -> Result<Vec<Vec<Json>>> {
        if !path.array()?.is_empty() {
            return Ok(vec![path.array()?.to_vec()]);
        }
        fn points(seq: &[Instruction], ordinal: &mut u128, out: &mut Vec<u128>) {
            for i in seq {
                out.push(*ordinal);
                *ordinal += 1;
                for body in &i.bodies {
                    points(body, ordinal, out);
                }
            }
        }
        let symbol = &self.identity()?.interface_hashes[region];
        let mut chains = Vec::new();
        for block in &self.functions[region].blocks {
            let mut ordinals = Vec::new();
            points(&block.instructions, &mut 0, &mut ordinals);
            for ordinal in ordinals {
                chains.push(vec![Json::Object(std::collections::BTreeMap::from([
                    ("symbol".into(), Json::String(symbol.clone())),
                    (
                        "point".into(),
                        Json::String(format!("{}:{ordinal}", block.name)),
                    ),
                    ("iteration".into(), Json::String("*".into())),
                ]))]);
            }
        }
        Ok(chains)
    }

    fn chain_domain(
        &self,
        chain: &[Json],
        root_budget: Option<u128>,
    ) -> Result<Option<(Domain, InvocationSteps)>> {
        let mut domain = Vec::new();
        // The step owner persists through lift.call and lifted action gates.
        // Only an oracle call opens a new token-governed invocation (§11).
        let mut steps = root_budget.map_or(InvocationSteps::Action, InvocationSteps::Root);
        let mut opens_oracle = false;
        if root_budget == Some(0) {
            return Ok(None);
        }
        for element in chain {
            let (block, ordinal) = element
                .get("point")?
                .string()?
                .rsplit_once(':')
                .ok_or_else(|| Error("§11: invalid point".into()))?;
            let point = self.point(
                element.get("symbol")?.string()?,
                block,
                ordinal
                    .parse()
                    .map_err(|_| Error("§11: invalid point ordinal".into()))?,
            )?;
            if opens_oracle {
                steps = InvocationSteps::Nested;
            }
            opens_oracle = matches!(
                point.instruction.op.as_str(),
                "lift.oracle_call" | "lift.oracle_uncall"
            ) || normalize_call(point.instruction)
                .is_ok_and(|call| call.instruction.op == "oracle_call");
            for bound in point.loops {
                if bound == 0 {
                    return Ok(None);
                }
                domain.push((0, u128::from(bound) - 1));
            }
        }
        // Generated ordinals belong to the bound expansion library. Admission
        // does not execute it; retain the whole ordinal selector domain.
        domain.extend([(1, steps.upper()), (0, 1), (0, u128::MAX)]);
        Ok(Some((domain, steps)))
    }

    fn entry_domain(&self, entry: &Json, chain: &[Json], base: &Domain) -> Result<Option<Domain>> {
        let path = entry.get("path")?.array()?;
        if path.len() > chain.len() {
            return Ok(None);
        }
        let mut domain = base.clone();
        let mut offset = 0;
        for (i, element) in chain.iter().enumerate() {
            let (block, ordinal) = element
                .get("point")?
                .string()?
                .rsplit_once(':')
                .ok_or_else(|| Error("§11: invalid point".into()))?;
            let depth = self
                .point(
                    element.get("symbol")?.string()?,
                    block,
                    ordinal
                        .parse()
                        .map_err(|_| Error("§11: invalid point ordinal".into()))?,
                )?
                .loops
                .len();
            if let Some(selector) = path.get(i) {
                if selector.get("symbol")? != element.get("symbol")?
                    || selector.get("point")? != element.get("point")?
                {
                    return Ok(None);
                }
                let iterations = selector.get("iteration")?;
                if !wildcard(iterations) {
                    for (j, value) in iterations.array()?.iter().enumerate() {
                        let value = value.uint()?;
                        domain[offset + j] = (value, value);
                    }
                }
            }
            offset += depth;
        }
        for (i, key) in ["step", "phase", "generated"].iter().enumerate() {
            let value = entry.get(key)?;
            if !wildcard(value) {
                let n = if *key == "phase" {
                    u128::from(value.string()? == "rev")
                } else {
                    value.uint()?
                };
                domain[offset + i] = (n, n);
            }
        }
        Ok(intersection(&domain, base))
    }

    pub(super) fn has_uncovered_tie(
        &self,
        a: &Json,
        b: &Json,
        entries: &[Json],
        region: &str,
        root_budget: Option<u128>,
    ) -> Result<bool> {
        let mut chains = self.shortest_chains(a.get("path")?, region)?;
        let max_path = entries
            .iter()
            .map(|e| e.get("path")?.array().map(<[Json]>::len))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .max()
            .unwrap_or(0);
        // Beyond every selector's path, only the reachable call graph matters.
        // Memoizing suffix points per boundary chain terminates recursive
        // lprocs without a call-depth, step, or loop expansion cap.
        let mut suffixes = std::collections::BTreeSet::new();
        while let Some(chain) = chains.pop() {
            let Some((base, steps)) = self.chain_domain(&chain, root_budget)? else {
                continue;
            };
            if matches!(steps, InvocationSteps::Root(_)) {
                let last = chain
                    .last()
                    .ok_or_else(|| Error("§11: empty operation chain".into()))?;
                let (block, ordinal) = last
                    .get("point")?
                    .string()?
                    .rsplit_once(':')
                    .ok_or_else(|| Error("§11: invalid point".into()))?;
                let point = self.point(
                    last.get("symbol")?.string()?,
                    block,
                    ordinal
                        .parse()
                        .map_err(|_| Error("§11: invalid point ordinal".into()))?,
                )?;
                let callee = if matches!(
                    point.instruction.op.as_str(),
                    "lift.call" | "lift.oracle_call" | "lift.oracle_uncall" | "lift.perm_call"
                ) {
                    Some(
                        point.instruction.operands[0]
                            .trim_start_matches('@')
                            .to_string(),
                    )
                } else {
                    normalize_call(point.instruction).ok().map(|call| {
                        call.instruction.operands[0]
                            .trim_start_matches('@')
                            .to_string()
                    })
                };
                let boundary = chain[..chain.len().min(max_path)].to_vec();
                let key = (Json::Array(boundary).canonical(), last.canonical());
                if chain.len() < max_path || suffixes.insert(key) {
                    if let Some(callee) = callee.filter(|name| self.functions.contains_key(name)) {
                        for suffix in self.shortest_chains(&Json::Array(vec![]), &callee)? {
                            let mut extension = chain.clone();
                            extension.extend(suffix);
                            chains.push(extension);
                        }
                    }
                }
            }
            let Some(ad) = self.entry_domain(a, &chain, &base)? else {
                continue;
            };
            let Some(bd) = self.entry_domain(b, &chain, &base)? else {
                continue;
            };
            let Some(common) = intersection(&ad, &bd) else {
                continue;
            };
            let mut remaining = vec![common];
            for higher in entries {
                if higher.get("kind")? != a.get("kind")? || specificity(higher)? <= specificity(a)?
                {
                    continue;
                }
                if let Some(cover) = self.entry_domain(higher, &chain, &base)? {
                    remaining = remaining
                        .into_iter()
                        .flat_map(|box_| subtract(box_, &cover))
                        .collect();
                    if remaining.is_empty() {
                        break;
                    }
                }
            }
            if !remaining.is_empty() {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

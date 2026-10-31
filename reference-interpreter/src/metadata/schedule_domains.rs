//! §11 maximal-match admission over selector domains, without loop expansion.
use super::*;

type Domain = Vec<(u128, u128)>;

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
    /// The shortest invocation chains in a prefix are sufficient: each point
    /// has its own generated operations (§11), including call/loop rows. A
    /// longer path cannot cover those operations. If these shortest chains
    /// are covered, every extension is covered by the same covering prefixes.
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

    fn chain_domain(&self, chain: &[Json], budget: u128) -> Result<Option<Domain>> {
        let mut domain = Vec::new();
        let mut oracle = false;
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
            oracle |= point.function.kind == "oracle";
            for bound in point.loops {
                if bound == 0 {
                    return Ok(None);
                }
                domain.push((0, u128::from(bound) - 1));
            }
        }
        if oracle && budget == 0 {
            return Ok(None);
        }
        // Generated ordinals belong to the bound expansion library. Admission
        // does not execute it; retain the whole ordinal selector domain.
        domain.extend([
            (1, if oracle { budget } else { u128::MAX }),
            (0, 1),
            (0, u128::MAX),
        ]);
        Ok(Some(domain))
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
        budget: u128,
    ) -> Result<bool> {
        for chain in self.shortest_chains(a.get("path")?, region)? {
            let Some(base) = self.chain_domain(&chain, budget)? else {
                continue;
            };
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

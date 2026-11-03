//! ISA 0.9 full-tuple replay. The legacy 0.8 table encoding stays in claims.rs.
use super::*;
use crate::Argument;

#[derive(Clone)]
pub(super) struct Invocation {
    pub args: Vec<Argument>,
    pub controls: Vec<BitVector>,
    pub adjoint: bool,
    budget: Budget,
}

struct ResolvedDomain {
    domain: Domain,
    relative_to_image: bool,
    images: Vec<FixedRegionBinding>,
}
pub(super) struct Outcome {
    pub verdict: ClaimVerdict,
    pub coverage: Option<ClaimCoverage>,
}

fn coverage(
    f: &Function,
    controls: usize,
    domain: &ResolvedDomain,
    elements: usize,
) -> Result<ClaimCoverage> {
    let mut bits = controls;
    for p in f.params.iter().flatten() {
        let width = match p.ty {
            Type::Wire(w) => Some(w as u128),
            Type::Region(a, w) => Some(
                crate::oracle::power_of_two(a as u128)
                    .and_then(|n| n.checked_mul(w as u128))
                    .ok_or_else(|| {
                        crate::resource_error("§12.2 K74", "full region width exceeds u128")
                    })?,
            ),
            _ => None,
        };
        if let Some(width) = width {
            bits = usize::try_from(width)
                .ok()
                .and_then(|w| bits.checked_add(w))
                .ok_or_else(|| {
                    crate::resource_error("§12.2 K74", "full interface width exceeds usize")
                })?;
        }
    }
    for (c, width, ..) in &domain.domain.components {
        if matches!(c, Component::Borrowed(..)) {
            bits = bits.checked_add(*width).ok_or_else(|| {
                crate::resource_error("§12.2 K74", "borrowed width exceeds usize")
            })?;
        }
    }
    let power = |exponent: usize| -> Result<String> {
        let width = exponent
            .checked_add(1)
            .ok_or_else(|| crate::resource_error("§12.2 K74", "coverage storage exceeds usize"))?;
        let mut n = BitVector::zero(width)?;
        n.set(exponent, true)?;
        Ok(n.to_decimal())
    };
    let full_domain_elements = power(bits)?;
    let mut numerator = elements as u128;
    let covered_fraction = if numerator == 0 {
        "0/1".into()
    } else {
        let reduce = (numerator.trailing_zeros() as usize).min(bits);
        numerator >>= reduce;
        format!("{numerator}/{}", power(bits - reduce)?)
    };
    Ok(ClaimCoverage {
        elements: elements as u128,
        full_domain_elements,
        covered_fraction,
        fixed_regions: domain.images.clone(),
    })
}

fn budget(value: &Json) -> Result<Budget> {
    let values = value
        .array()?
        .iter()
        .map(|v| {
            v.string()?
                .parse::<u128>()
                .map_err(|_| crate::resource_error("§12.1 K13", "invocation tuple exceeds u128"))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Budget {
        steps: u64::try_from(values[0])
            .map_err(|_| crate::resource_error("§12.1 K13", "invocation steps exceed u64"))?,
        heap: values[1],
        stack: values[2],
        depth: values[3],
    })
}

fn decimal(value: &BitVector) -> Json {
    Json::String(value.to_decimal())
}

impl Object {
    fn invocation09(&self, claim: &Json, f: &Function, closure: &str) -> Result<Invocation> {
        let subject = claim.get("subject")?;
        if subject.get("closure_hash")?.string()? != closure {
            return err("§12.1 K13", "claim subject differs from its region closure");
        }
        // Variant is an open identifier selecting a realisation. It does not
        // change the region's reference semantics or imply an adjoint.
        let adjoint = false;
        let invocation = if let Some(invocation) = claim.object()?.get("invocation") {
            invocation
        } else {
            let bindings = self.section("certificate").unwrap().get("bindings")?;
            if bindings.get("closure_hash")?.string()? != closure {
                return err(
                    "§12.1 K13",
                    "nested scheduled invocation needs concrete context resolution",
                );
            }
            bindings.get("invocation")?
        };
        let group = if f.kind == "oracle" { 4 } else { 3 };
        let args = invocation
            .get("args")?
            .array()?
            .iter()
            .zip(&f.params[group])
            .map(|(value, p)| {
                Ok(match p.ty {
                    Type::Int(_) | Type::Ptr => {
                        Argument::Bits(value.string()?.parse::<u128>().map_err(|_| {
                            crate::resource_error("§12.1 K13", "classical value exceeds u128")
                        })?)
                    }
                    Type::F64 => Argument::Float(f64::from_bits(
                        u64::from_str_radix(&value.string()?[2..], 16)
                            .map_err(|_| Error::new("§12.1 K13: f64 bits".into()))?,
                    )),
                    Type::Token => Argument::Token(budget(value)?),
                    _ => return err("§12.1 K13", "invalid classical invocation type"),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        if args.len() != f.params[group].len() {
            return err("§12.1 K13", "invocation argument count mismatch");
        }
        Ok(Invocation {
            args,
            controls: Vec::new(),
            adjoint,
            budget: budget(invocation.get("budget")?)?,
        })
    }

    fn domain09(&self, claim: &Json, f: &Function) -> Result<ResolvedDomain> {
        let controls = usize::try_from(claim.get("subject")?.get("inherited_controls")?.uint()?)
            .map_err(|_| {
                crate::resource_error("§12.2 K72", "inherited control count exceeds usize")
            })?;
        let mut source = claim.get("domain")?.clone();
        let Json::Object(fields) = &mut source else {
            unreachable!()
        };
        let Json::Array(bundles) = fields.get_mut("bundles").unwrap() else {
            unreachable!()
        };
        if bundles.len() < controls {
            return err("§12.2 K72", "domain omits inherited controls");
        }
        let inherited = bundles.drain(..controls).collect::<Vec<_>>();
        let images = self.image_domains09(claim.get("domain")?, f)?;
        let mut unrestricted = source.clone();
        for (value, full) in [(&mut source, false), (&mut unrestricted, true)] {
            let Json::Object(fields) = value else {
                unreachable!()
            };
            let Json::Array(regions) = fields.get_mut("regions").unwrap() else {
                unreachable!()
            };
            for image in &images {
                regions.push(image.entry(full)?);
            }
        }
        let mut domain = self.domain(f, &source)?;
        let mut relative_to_image = !images.is_empty() && self.domain(f, &unrestricted)?.covers_d;
        let mut components = Vec::new();
        for (i, entry) in inherited.iter().enumerate() {
            if entry.get("width")?.uint()? != 1 {
                return err("§12.2 K72", "inherited control width differs from one");
            }
            let lo = json_bits(entry.get("lo")?, 1)?;
            let hi = json_bits(entry.get("hi")?, 1)?;
            domain.covers_d &= lo.is_zero() && hi == ones(1)?;
            relative_to_image &= lo.is_zero() && hi == ones(1)?;
            components.push((Component::Control(i), 1, lo, hi));
        }
        components.append(&mut domain.components);
        domain.components = components;
        Ok(ResolvedDomain {
            domain,
            relative_to_image,
            images: images.into_iter().map(|i| i.binding).collect(),
        })
    }

    /// Convert the flat executor output to K13's complete typed tuple; the
    /// region/cell boundaries come from the subject signature, never the table.
    fn tuple09(
        &self,
        f: &Function,
        flat: &[BitVector],
        controls: usize,
        borrowed: usize,
    ) -> Result<Json> {
        let bundle_count = controls
            + f.params
                .iter()
                .flatten()
                .filter(|p| matches!(p.ty, Type::Wire(_)))
                .count();
        let mut offset = bundle_count;
        let mut regions = Vec::new();
        for p in f.params.iter().flatten() {
            if let Type::Region(a, _) = p.ty {
                let cells = crate::oracle::power_of_two(a as u128)
                    .and_then(|n| usize::try_from(n).ok())
                    .ok_or_else(|| {
                        crate::resource_error("§12.1 K13", "region tuple exceeds usize")
                    })?;
                let end = offset.checked_add(cells).ok_or_else(|| {
                    crate::resource_error("§12.1 K13", "region tuple size overflow")
                })?;
                let values = flat.get(offset..end).ok_or_else(|| {
                    Error::new("§12.1 K13: incomplete executor region tuple".into())
                })?;
                regions.push(Json::Array(values.iter().map(decimal).collect()));
                offset = end;
            }
        }
        if flat.len() != offset + borrowed {
            return err("§12.1 K13", "executor tuple length differs from interface");
        }
        Ok(Json::Object(BTreeMap::from([
            (
                "bundles".into(),
                Json::Array(flat[..bundle_count].iter().map(decimal).collect()),
            ),
            ("regions".into(), Json::Array(regions)),
            (
                "borrowed".into(),
                Json::Array(flat[offset..].iter().map(decimal).collect()),
            ),
        ])))
    }

    fn entry09(
        &self,
        f: &Function,
        element: &[(Component, BitVector)],
        controls: usize,
    ) -> Result<Vec<BitVector>> {
        let find = |component: Component, width| {
            element
                .iter()
                .find_map(|(c, v)| {
                    (match (&component, c) {
                        (Component::Control(a), Component::Control(b)) => a == b,
                        (Component::Bundle(a, b), Component::Bundle(c, d)) => (a, b) == (c, d),
                        (Component::Cell(a, b, c), Component::Cell(d, e, f)) => {
                            (a, b, c) == (d, e, f)
                        }
                        _ => false,
                    })
                    .then_some(v.clone())
                })
                .map_or_else(|| BitVector::zero(width), Ok)
        };
        let mut out = (0..controls)
            .map(|i| find(Component::Control(i), 1))
            .collect::<Result<Vec<_>>>()?;
        for (group, params) in f.params.iter().enumerate() {
            for (slot, p) in params.iter().enumerate() {
                if matches!(p.ty, Type::Wire(_)) {
                    out.push(find(
                        Component::Bundle(group, slot),
                        p.ty.bits(self.ptr_width)?,
                    )?);
                }
            }
        }
        for (group, params) in f.params.iter().enumerate() {
            for (slot, p) in params.iter().enumerate() {
                if let Type::Region(a, w) = p.ty {
                    let count = crate::oracle::power_of_two(a as u128)
                        .and_then(|n| usize::try_from(n).ok())
                        .ok_or_else(|| {
                            crate::resource_error("§12.1 K13", "region tuple exceeds usize")
                        })?;
                    crate::reserve_more(&mut out, count, "§12.1 K13", "entry region cells")?;
                    for index in 0..count {
                        out.push(find(Component::Cell(group, slot, index), w as usize)?);
                    }
                }
            }
        }
        out.extend(
            element
                .iter()
                .filter_map(|(c, v)| matches!(c, Component::Borrowed(..)).then_some(v.clone())),
        );
        Ok(out)
    }

    pub(super) fn verify_replay09(
        &self,
        closure: &str,
        index: usize,
        claim: &Json,
        f: &Function,
    ) -> Result<Outcome> {
        if claim.get("expected")? == &Json::String("inverse_pair".into()) {
            return err(
                "§12.1 K4",
                "inverse-pair claims require their entry binding",
            );
        }
        if claim.get("expected")? == &Json::String("oracle".into()) {
            return Ok(Outcome {
                verdict: ClaimVerdict::Unverified(
                    "expected oracle needs realised target/schedule execution (deliverables 9–11)"
                        .into(),
                ),
                coverage: None,
            });
        }
        let mut invocation = self.invocation09(claim, f, closure)?;
        let controls = usize::try_from(claim.get("subject")?.get("inherited_controls")?.uint()?)
            .map_err(|_| crate::resource_error("§12.1 K13", "control count exceeds usize"))?;
        let resolved = self.domain09(claim, f)?;
        let domain = &resolved.domain;
        let sampled = claim.get("kind")?.string()? == "sampled";
        let elements = if sampled {
            let sampler = claim.get("sampler")?;
            let seed = sampler
                .get("seed")?
                .array()?
                .iter()
                .map(|v| {
                    v.string()?
                        .parse::<u64>()
                        .map_err(|_| Error::new("§12.1: sampler seed".into()))
                })
                .collect::<Result<Vec<_>>>()?;
            let mut rng = Xoshiro256([seed[0], seed[1], seed[2], seed[3]]);
            let count = sample_count(sampler)?;
            // Retain the existing fallible host-capacity check before a
            // metadata-controlled sampling loop or allocation can run away.
            let _capacity = sample_elements(count)?;
            let mut distinct = BTreeMap::new();
            for _ in 0..count {
                let row = domain
                    .components
                    .iter()
                    .map(|(_, w, lo, hi)| rng.draw(*w, lo, hi))
                    .collect::<Result<Vec<_>>>()?;
                // Fixed-width unsigned decimal magnitude order equals domain
                // enumeration order, independent of draw order and duplicates.
                let key = row
                    .iter()
                    .map(|v| {
                        let s = v.to_decimal();
                        (s.len(), s)
                    })
                    .collect::<Vec<_>>();
                distinct.insert(key, row);
            }
            distinct.into_values().collect::<Vec<_>>()
        } else {
            enumerate(domain)?
        };
        let expected = claim.get("expected")?;
        let table = expected
            .object()?
            .get("table")
            .map(Json::array)
            .transpose()?;
        let program = expected
            .object()?
            .get("program")
            .map(|p| self.region_function(p.string()?))
            .transpose()?;
        if table.is_some_and(|table| table.len() != elements.len()) {
            return err(
                "§12.1 K13",
                "table must contain one row per distinct domain element",
            );
        }
        if program.is_some_and(|p| p.signature() != f.signature()) {
            return err(
                "§12.1 K13",
                "expected program has an incompatible signature",
            );
        }
        let borrowed = domain
            .components
            .iter()
            .filter(|(c, ..)| matches!(c, Component::Borrowed(..)))
            .count();
        for (row, values) in elements.iter().enumerate() {
            let element = domain
                .components
                .iter()
                .map(|c| c.0.clone())
                .zip(values.iter().cloned())
                .collect::<Vec<_>>();
            let before = self.entry09(f, &element, controls)?;
            invocation.controls = before[..controls].to_vec();
            let after = self.execute_element_bound(
                f,
                &element,
                Some(invocation.budget),
                None,
                Some(&invocation),
            )?;
            if let Some(table) = table {
                let entry = &table[row];
                if entry.get("before")? != &self.tuple09(f, &before, controls, borrowed)?
                    || entry.get("after")? != &self.tuple09(f, &after, controls, borrowed)?
                {
                    return err(
                        "§12.1 K13",
                        format!("claim {index} of {closure}: full tuple mismatch at element {row}"),
                    );
                }
            }
            if let Some(program) = program {
                let other = self.execute_element_bound(
                    program,
                    &element,
                    Some(invocation.budget),
                    None,
                    Some(&invocation),
                )?;
                if other != after {
                    return err("§12.1 K13", "expected program full tuple mismatch");
                }
            }
        }
        let verdict = if !sampled && domain.covers_d {
            ClaimVerdict::Proof
        } else if !sampled && resolved.relative_to_image {
            ClaimVerdict::ProofRelativeToImage
        } else {
            ClaimVerdict::Evidence {
                elements: elements.len() as u128,
            }
        };
        if claim.object()?.get("label") == Some(&Json::String("proof".into()))
            && verdict != ClaimVerdict::Proof
        {
            return err(
                "§12.2 K74",
                "requested proof but replay covers only a slice of D",
            );
        }
        if claim.object()?.get("label") == Some(&Json::String("proof_relative_to_image".into()))
            && verdict != ClaimVerdict::ProofRelativeToImage
        {
            return err(
                "§12.2 K85",
                "requested proof_relative_to_image without complete remaining domain",
            );
        }
        Ok(Outcome {
            verdict,
            coverage: Some(coverage(f, controls, &resolved, elements.len())?),
        })
    }
}

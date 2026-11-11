//! K87 arithmetic on independently resolved accounting values. These helpers
//! do not reconstruct a target event list or certify its completeness.
use super::Rational;
use crate::{err, json::Json, Error, Result};
use std::collections::BTreeMap;

const RESOURCE_DIGITS: i64 = 34; // §10.1/§12.3 canonical upward resource display.

pub fn resource_display(value: &Rational) -> Result<String> {
    if *value < Rational::integer(0) {
        return err("§12.3", "negative resource quantity");
    }
    if *value == Rational::integer(0) {
        return Ok("0".into());
    }
    let (n, d) = value.pair();
    let mut exponent = i64::try_from(n.len())
        .and_then(|n| i64::try_from(d.len()).map(|d| n - d))
        .map_err(|_| crate::resource_error("§12.3", "decimal exponent exceeds host range"))?;
    if *value < Rational::decimal(&format!("1e{exponent}"))? {
        exponent -= 1;
    }
    let scale = exponent - (RESOURCE_DIGITS - 1);
    let digits = value
        .div(&Rational::decimal(&format!("1e{scale}"))?)?
        .ceil()
        .pair()
        .0;
    let mut out = String::new();
    let extra = usize::try_from(scale.unsigned_abs()).map_err(|_| {
        crate::resource_error("§12.3", "resource display length exceeds host range")
    })?;
    out.try_reserve(
        digits
            .len()
            .checked_add(extra)
            .and_then(|n| n.checked_add(2))
            .ok_or_else(|| {
                crate::resource_error("§12.3", "resource display length exceeds host range")
            })?,
    )
    .map_err(|e| crate::resource_error("§12.3", e))?;
    if scale >= 0 {
        out.push_str(&digits);
        out.extend(std::iter::repeat_n('0', extra));
    } else if extra >= digits.len() {
        out.push_str("0.");
        out.extend(std::iter::repeat_n('0', extra - digits.len()));
        out.push_str(&digits);
    } else {
        let point = digits.len() - extra;
        out.push_str(&digits[..point]);
        out.push('.');
        out.push_str(&digits[point..]);
    }
    if out.contains('.') {
        while out.ends_with('0') {
            out.pop();
        }
        if out.ends_with('.') {
            out.pop();
        }
    }
    Ok(out)
}

pub(crate) fn canonical_rows(rows: &Json) -> Result<()> {
    for value in rows.object()?.values() {
        let text = value.string()?;
        if resource_display(&Rational::decimal(text)?)? != text {
            return err("§12.3", "noncanonical resource display");
        }
    }
    Ok(())
}

pub fn verify_resource_rows(
    rows: &Json,
    estimate: Option<&Json>,
    exact: &BTreeMap<String, Rational>,
) -> Result<()> {
    canonical_rows(rows)?;
    if rows.object()?.keys().ne(exact.keys()) {
        return err(
            "§12.3",
            "resource row keys differ from independently computed accounting",
        );
    }
    for (key, value) in exact {
        if rows.get(key)?.string()? != resource_display(value)? {
            return err("§12.3", format!("resource display differs for {key}"));
        }
    }
    if let Some(estimate) = estimate {
        estimate.fields(&["precision_digits", "exact_resources"], &[])?;
        if estimate.get("precision_digits")?.uint()? != RESOURCE_DIGITS as u128 {
            return err("§12.3", "resource precision_digits must be 34");
        }
        let fractions = estimate.get("exact_resources")?;
        if fractions.object()?.keys().ne(exact.keys()) {
            return err("§12.3", "exact resource keys differ from accounting");
        }
        for (key, value) in exact {
            let fraction = fractions.get(key)?;
            fraction.fields(&["numerator", "denominator"], &[])?;
            let provided = Rational::leaf(
                fraction.get("numerator")?.string()?,
                fraction.get("denominator")?.string()?,
            )?;
            if provided != *value {
                return err(
                    "§12.3",
                    format!("exact resource fraction differs for {key}"),
                );
            }
        }
    }
    Ok(())
}

/// One wire's occupancy, with disjoint half-open borrow intervals replacing
/// the owner's charge. Rates have already been resolved from the target.
pub fn charged_wire_occupancy(
    start: &Rational,
    end: &Rational,
    owner_rate: &Rational,
    borrows: &[(Rational, Rational, Rational)],
) -> Result<Rational> {
    if start < &Rational::integer(0) || end < start || owner_rate < &Rational::integer(0) {
        return err("§8.2", "invalid wire interval or owner rate");
    }
    let mut ordered = borrows.iter().collect::<Vec<_>>();
    ordered.sort_by(|a, b| a.0.cmp(&b.0));
    let mut previous = start;
    let mut charge = end.sub(start).mul(owner_rate);
    for (lo, hi, rate) in ordered {
        if lo < previous || hi <= lo || hi > end || rate < &Rational::integer(0) {
            return err("§8.2", "borrow intervals overlap or exceed wire lifetime");
        }
        charge = charge
            .sub(&hi.sub(lo).mul(owner_rate))
            .add(&hi.sub(lo).mul(rate));
        previous = hi;
    }
    Ok(charge)
}

/// Trusted accounting input, computed independently from the complete resolved
/// context tree/event list. Never populate this from the producer's rows.
pub struct ResolvedResourceContext {
    pub context_key: Json,
    pub resources: BTreeMap<String, Rational>,
    pub gates_used: Json,
}

/// Compare producer rows/projection against a complete independently computed
/// context set. Target reconstruction and scope completeness are prerequisites,
/// not consequences of this comparison helper.
pub fn verify_resource_contexts(region: &Json, expected: &[ResolvedResourceContext]) -> Result<()> {
    if expected.is_empty() {
        return err("§12.3", "resource report has no reached contexts");
    }
    let mut expected = expected.iter().collect::<Vec<_>>();
    expected.sort_by_key(|r| r.context_key.canonical());
    for pair in expected.windows(2) {
        if pair[0].context_key == pair[1].context_key {
            return err("§12.3", "duplicate independently computed context");
        }
    }
    let supplied = region.object()?.get("resource_contexts");
    if let Some(supplied) = supplied {
        let records = supplied.array()?;
        if records.len() != expected.len() {
            return err("§12.3", "resource context coverage differs");
        }
        for (record, computed) in records.iter().zip(&expected) {
            record.fields(&["context_key", "resources", "gates_used"], &["estimate"])?;
            crate::ref_library::validate_context_key(record.get("context_key")?)?;
            if record.get("context_key")? != &computed.context_key {
                return err(
                    "§12.3",
                    "resource contexts differ or are out of canonical order",
                );
            }
            verify_resource_rows(
                record.get("resources")?,
                record.object()?.get("estimate"),
                &computed.resources,
            )?;
            if record.get("gates_used")? != &computed.gates_used {
                return err(
                    "§12.3",
                    "context gate counts/attribution differ from accounting",
                );
            }
        }
        let first = &records[0];
        for key in ["resources", "gates_used"] {
            if region.get(key)? != first.get(key)? {
                return err(
                    "§12.3",
                    "unkeyed resource projection differs from first canonical context",
                );
            }
        }
        if region.object()?.get("estimate") != first.object()?.get("estimate") {
            return err(
                "§12.3",
                "estimate projection differs from first canonical context",
            );
        }
    } else if expected.len() != 1 {
        return err(
            "§12.3",
            "repeated-context resource report requires resource_contexts",
        );
    }
    verify_resource_rows(
        region.get("resources")?,
        region.object()?.get("estimate"),
        &expected[0].resources,
    )?;
    if region.get("gates_used")? != &expected[0].gates_used {
        return Err(Error::new(
            "§12.3: unkeyed gates differ from accounting".into(),
        ));
    }
    Ok(())
}

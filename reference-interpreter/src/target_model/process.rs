//! Low-level §12.1 acceptance procedures. Callers must independently establish
//! that each matrix encloses the bound operation in every invocation context;
//! these routines alone never establish a certificate claim or native model.
use super::{ComplexInterval, Interval, IntervalMatrix, Rational};
use crate::{err, Result};

/// §12.1 square-unitary procedure: outward enclose V†U-I, then its
/// Frobenius norm. This is not a test of the operators' normalization.
pub fn verify_numerical_bound(
    realised: &IntervalMatrix,
    reference: &IntervalMatrix,
    bound: &Rational,
) -> Result<()> {
    realised.compatible(reference)?;
    if realised.rows != realised.cols {
        return err("§12.1", "unrestricted numerical operators must be square");
    }
    let mut difference = reference.dagger()?.multiply(realised)?;
    let one = ComplexInterval::real(Rational::integer(1), realised.precision)?;
    for index in 0..difference.rows {
        difference.put(index, index, difference.at(index, index).sub(&one)?);
    }
    frobenius_bound(&difference, bound)
}

/// K85: callers supply the independently reconstructed UJ_I and VJ_I,
/// preserving all output rows. No certificate/preparation encoding is inferred.
pub fn verify_restricted_numerical_bound(
    realised_image: &IntervalMatrix,
    reference_image: &IntervalMatrix,
    bound: &Rational,
) -> Result<()> {
    frobenius_bound(&realised_image.combine(reference_image, true)?, bound)
}

fn frobenius_bound(difference: &IntervalMatrix, bound: &Rational) -> Result<()> {
    if *bound < Rational::integer(0) {
        return err("§12.1", "negative numerical bound");
    }
    let mut sum = Interval::point(Rational::integer(0), difference.precision)?;
    for entry in &difference.entries {
        sum = sum.add(&entry.norm_squared()?)?;
    }
    let norm = sum.sqrt()?;
    if norm.upper() > bound {
        return err(
            "§12.1",
            "interval Frobenius upper bound exceeds numerical claim",
        );
    }
    Ok(())
}

/// §12.1 input-first unnormalised Choi dual, including K85 rectangular
/// isometries (reduced input, complete output). Every LDL† pivot is strict.
pub fn verify_channel_bound(
    realised: &IntervalMatrix,
    reference: &IntervalMatrix,
    y: &IntervalMatrix,
    mu: &Rational,
    bound: &Rational,
) -> Result<()> {
    realised.compatible(reference)?;
    if mu < &Rational::integer(0) || mu > bound {
        return err("§12.1", "channel mu is negative or exceeds bound");
    }
    let choi = choi_difference(realised, reference)?;
    choi.compatible(y)?;
    y.hermitian()?;
    strict_positive(&y.combine(&choi, true)?)?;
    strict_positive(&y.combine(&choi, false)?)?;
    let mut slack = IntervalMatrix::zeros(realised.cols, realised.cols, realised.precision)?;
    let mu = ComplexInterval::real(mu.clone(), realised.precision)?;
    for i in 0..realised.cols {
        for j in 0..realised.cols {
            let mut trace = ComplexInterval::real(Rational::integer(0), realised.precision)?;
            for output in 0..realised.rows {
                trace = trace.add(y.at(i * realised.rows + output, j * realised.rows + output))?;
            }
            let scalar = if i == j {
                mu.clone()
            } else {
                ComplexInterval::real(Rational::integer(0), realised.precision)?
            };
            slack.put(i, j, scalar.sub(&trace)?);
        }
    }
    strict_positive(&slack)
}

fn choi_difference(u: &IntervalMatrix, v: &IntervalMatrix) -> Result<IntervalMatrix> {
    let dimension = u
        .rows
        .checked_mul(u.cols)
        .ok_or_else(|| crate::resource_error("§12.1", "Choi dimension exceeds host range"))?;
    let mut out = IntervalMatrix::zeros(dimension, dimension, u.precision)?;
    for row in 0..dimension {
        let (i, a) = (row / u.rows, row % u.rows);
        for col in 0..=row {
            let (j, b) = (col / u.rows, col % u.rows);
            let entry = if row == col {
                ComplexInterval::new(
                    u.at(a, i)
                        .norm_squared()?
                        .sub(&v.at(a, i).norm_squared()?)?,
                    Interval::point(Rational::integer(0), u.precision)?,
                )?
            } else {
                u.at(a, i)
                    .mul(&u.at(b, j).conjugate())?
                    .sub(&v.at(a, i).mul(&v.at(b, j).conjugate())?)?
            };
            out.put(row, col, entry.clone());
            out.put(col, row, entry.conjugate());
        }
    }
    Ok(out)
}

fn strict_positive(matrix: &IntervalMatrix) -> Result<()> {
    matrix.hermitian()?;
    let mut l = IntervalMatrix::zeros(matrix.rows, matrix.cols, matrix.precision)?;
    let mut pivots: Vec<Interval> = crate::reserved(matrix.rows, "§12.1", "LDL pivots")?;
    for i in 0..matrix.rows {
        let mut pivot = matrix.at(i, i).re.clone();
        for (k, d) in pivots.iter().enumerate() {
            pivot = pivot.sub(&l.at(i, k).norm_squared()?.mul(d)?)?;
        }
        if pivot.lower() <= &Rational::integer(0) {
            return err(
                "§12.1",
                format!("interval LDL pivot {i} is not strictly positive"),
            );
        }
        for j in i + 1..matrix.rows {
            let mut entry = matrix.at(j, i).clone();
            for (k, d) in pivots.iter().enumerate() {
                entry = entry.sub(&l.at(j, k).mul(&l.at(i, k).conjugate())?.scale(d)?)?;
            }
            l.put(j, i, entry.divide_real(&pivot)?);
        }
        pivots.push(pivot);
    }
    Ok(())
}

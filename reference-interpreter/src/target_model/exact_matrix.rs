use super::{ComplexInterval, ExactComplex, Interval, IntervalMatrix, Rational};
use crate::{err, Result};

pub(super) struct ExactMatrix {
    pub size: usize,
    pub entries: Vec<ExactComplex>,
}

impl ExactMatrix {
    pub fn zeros(size: usize) -> Result<Self> {
        let count = size.checked_mul(size).ok_or_else(|| {
            crate::resource_error("§8.3", "native matrix dimensions exceed host range")
        })?;
        let mut entries = crate::reserved(count, "§8.3", "native model matrix")?;
        entries.resize(count, ExactComplex::real_value(Rational::integer(0)));
        Ok(Self { size, entries })
    }
    pub fn identity(size: usize) -> Result<Self> {
        let mut out = Self::zeros(size)?;
        for i in 0..size {
            out.entries[i * size + i] = ExactComplex::real_value(Rational::integer(1));
        }
        Ok(out)
    }
    pub fn at(&self, row: usize, col: usize) -> &ExactComplex {
        &self.entries[row * self.size + col]
    }
    pub fn controlled(&self, controls: usize) -> Result<Self> {
        let groups = dimension(controls)?;
        let size = self
            .size
            .checked_mul(groups)
            .ok_or_else(|| crate::resource_error("§8.3", "controlled matrix exceeds host range"))?;
        let mut out = Self::identity(size)?;
        let offset = size - self.size;
        for r in 0..self.size {
            for c in 0..self.size {
                out.entries[(offset + r) * size + offset + c] = self.at(r, c).clone();
            }
        }
        Ok(out)
    }
    pub fn permute_basis(&self, map: &[usize]) -> Result<Self> {
        let mut out = Self::zeros(self.size)?;
        let index = |bits: usize| {
            map.iter().enumerate().fold(0usize, |out, (j, allocation)| {
                out | (((bits >> (map.len() - 1 - j)) & 1) << (map.len() - 1 - allocation))
            })
        };
        for r in 0..self.size {
            for c in 0..self.size {
                out.entries[index(r) * self.size + index(c)] = self.at(r, c).clone();
            }
        }
        Ok(out)
    }
    pub fn unitary(&self) -> Result<()> {
        for r in 0..self.size {
            for c in 0..self.size {
                let mut sum = ExactComplex::real_value(Rational::integer(0));
                for k in 0..self.size {
                    sum = sum.add(&self.at(k, r).conjugate().mul(self.at(k, c)));
                }
                if sum != ExactComplex::real_value(Rational::integer(i128::from(r == c))) {
                    return err("§8.3", "native model is not unitary (U-dagger U != I)");
                }
            }
        }
        Ok(())
    }
    /// Exact Hermitian LDL with semidefinite zero-pivot handling proves
    /// eps² I - (U-V)†(U-V) >= 0, including equality at the spectral boundary.
    /// §8.3 uses operator norm; §12.1's Frobenius claim test is separate.
    pub fn accuracy(&self, ideal: &Self, epsilon: &Rational) -> Result<()> {
        if self.size != ideal.size {
            return err("§8.3", "ideal/native dimensions disagree");
        }
        let mut slack = Self::zeros(self.size)?;
        for r in 0..self.size {
            for c in 0..self.size {
                let mut sum = ExactComplex::real_value(Rational::integer(0));
                for k in 0..self.size {
                    sum = sum.add(
                        &self
                            .at(k, r)
                            .sub(ideal.at(k, r))
                            .conjugate()
                            .mul(&self.at(k, c).sub(ideal.at(k, c))),
                    );
                }
                let diagonal = ExactComplex::real_value(if r == c {
                    epsilon.mul(epsilon)
                } else {
                    Rational::integer(0)
                });
                slack.entries[r * self.size + c] = diagonal.sub(&sum);
            }
        }
        slack.positive_semidefinite()
    }
    fn positive_semidefinite(mut self) -> Result<()> {
        let zero = Rational::integer(0);
        for i in 0..self.size {
            let pivot = self.at(i, i).clone();
            if pivot.im != zero || pivot.re < zero {
                return err(
                    "§8.3",
                    "native model exceeds advertised operator-norm bound",
                );
            }
            if pivot.re == zero {
                if (i + 1..self.size)
                    .any(|r| *self.at(r, i) != ExactComplex::real_value(zero.clone()))
                {
                    return err(
                        "§8.3",
                        "native accuracy matrix has nonzero column at zero pivot",
                    );
                }
                continue;
            }
            for r in i + 1..self.size {
                for c in i + 1..self.size {
                    let correction = self.at(r, i).mul(self.at(i, c)).div(&pivot)?;
                    self.entries[r * self.size + c] = self.at(r, c).sub(&correction);
                }
            }
        }
        Ok(())
    }
    pub fn enclose(&self, precision: usize) -> Result<IntervalMatrix> {
        let mut entries = crate::reserved(self.entries.len(), "§12.1", "native enclosures")?;
        for e in &self.entries {
            entries.push(ComplexInterval::new(
                Interval::point(e.re.clone(), precision)?,
                Interval::point(e.im.clone(), precision)?,
            )?);
        }
        IntervalMatrix::new(self.size, self.size, entries)
    }
}

pub(super) fn dimension(bits: usize) -> Result<usize> {
    u32::try_from(bits)
        .ok()
        .and_then(|bits| 1usize.checked_shl(bits))
        .ok_or_else(|| crate::resource_error("§8.3", "native matrix basis exceeds host range"))
}

use super::{Interval, Rational};
use crate::{err, Result};

/// Complex entry enclosure. A box is a checking input, never an operation
/// identity or a native-model normalization certificate (§8.3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComplexInterval {
    pub re: Interval,
    pub im: Interval,
}

impl ComplexInterval {
    pub fn new(re: Interval, im: Interval) -> Result<Self> {
        if re.precision_bits() != im.precision_bits() {
            return err("§12.1", "complex interval precision mismatch");
        }
        Ok(Self { re, im })
    }
    pub fn real(value: Rational, precision_bits: usize) -> Result<Self> {
        Self::new(
            Interval::point(value, precision_bits)?,
            Interval::point(Rational::integer(0), precision_bits)?,
        )
    }
    pub fn conjugate(&self) -> Self {
        Self {
            re: self.re.clone(),
            im: self.im.neg(),
        }
    }
    pub fn add(&self, other: &Self) -> Result<Self> {
        Self::new(self.re.add(&other.re)?, self.im.add(&other.im)?)
    }
    pub fn sub(&self, other: &Self) -> Result<Self> {
        Self::new(self.re.sub(&other.re)?, self.im.sub(&other.im)?)
    }
    pub fn mul(&self, other: &Self) -> Result<Self> {
        Self::new(
            self.re.mul(&other.re)?.sub(&self.im.mul(&other.im)?)?,
            self.re.mul(&other.im)?.add(&self.im.mul(&other.re)?)?,
        )
    }
    pub(super) fn scale(&self, factor: &Interval) -> Result<Self> {
        Self::new(self.re.mul(factor)?, self.im.mul(factor)?)
    }
    pub(super) fn divide_real(&self, divisor: &Interval) -> Result<Self> {
        Self::new(self.re.div(divisor)?, self.im.div(divisor)?)
    }
    pub(super) fn norm_squared(&self) -> Result<Interval> {
        self.re.square()?.add(&self.im.square()?)
    }
    pub(super) fn is_real(&self) -> bool {
        *self.im.lower() == Rational::integer(0) && *self.im.upper() == Rational::integer(0)
    }
}

/// Entrywise enclosures of a linear map. Constructing this value does not
/// certify the map's model identity, unitarity, binding or accuracy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntervalMatrix {
    pub(super) rows: usize,
    pub(super) cols: usize,
    pub(super) entries: Vec<ComplexInterval>,
    pub(super) precision: usize,
}

impl IntervalMatrix {
    pub fn new(rows: usize, cols: usize, entries: Vec<ComplexInterval>) -> Result<Self> {
        let count = rows
            .checked_mul(cols)
            .ok_or_else(|| crate::resource_error("§12.1", "matrix size exceeds host range"))?;
        if rows == 0 || cols == 0 || entries.len() != count {
            return err("§12.1", "matrix dimensions disagree with entries");
        }
        let precision = entries[0].re.precision_bits();
        if entries
            .iter()
            .any(|e| e.re.precision_bits() != precision || e.im.precision_bits() != precision)
        {
            return err("§12.1", "matrix entry precision mismatch");
        }
        Ok(Self {
            rows,
            cols,
            entries,
            precision,
        })
    }
    pub fn rows(&self) -> usize {
        self.rows
    }
    pub fn columns(&self) -> usize {
        self.cols
    }
    pub fn entry(&self, row: usize, col: usize) -> Result<&ComplexInterval> {
        if row >= self.rows || col >= self.cols {
            return err("§12.1", "matrix entry outside dimensions");
        }
        Ok(self.at(row, col))
    }
    pub(super) fn zeros(rows: usize, cols: usize, precision: usize) -> Result<Self> {
        let count = rows
            .checked_mul(cols)
            .ok_or_else(|| crate::resource_error("§12.1", "matrix size exceeds host range"))?;
        let zero = ComplexInterval::real(Rational::integer(0), precision)?;
        let mut entries = crate::reserved(count, "§12.1", "interval matrix")?;
        entries.resize(count, zero);
        Self::new(rows, cols, entries)
    }
    pub(super) fn at(&self, row: usize, col: usize) -> &ComplexInterval {
        &self.entries[row * self.cols + col]
    }
    pub(super) fn put(&mut self, row: usize, col: usize, value: ComplexInterval) {
        self.entries[row * self.cols + col] = value;
    }
    pub(super) fn compatible(&self, other: &Self) -> Result<()> {
        if (self.rows, self.cols, self.precision) != (other.rows, other.cols, other.precision) {
            return err("§12.1", "process dimensions or precisions disagree");
        }
        Ok(())
    }
    pub fn dagger(&self) -> Result<Self> {
        let mut out = Self::zeros(self.cols, self.rows, self.precision)?;
        for row in 0..self.rows {
            for col in 0..self.cols {
                out.put(col, row, self.at(row, col).conjugate());
            }
        }
        Ok(out)
    }
    pub fn multiply(&self, other: &Self) -> Result<Self> {
        if self.cols != other.rows || self.precision != other.precision {
            return err("§12.1", "matrix product dimensions or precisions disagree");
        }
        let mut out = Self::zeros(self.rows, other.cols, self.precision)?;
        for row in 0..self.rows {
            for col in 0..other.cols {
                let mut sum = ComplexInterval::real(Rational::integer(0), self.precision)?;
                for inner in 0..self.cols {
                    sum = sum.add(&self.at(row, inner).mul(other.at(inner, col))?)?;
                }
                out.put(row, col, sum);
            }
        }
        Ok(out)
    }
    pub(super) fn combine(&self, other: &Self, subtract: bool) -> Result<Self> {
        self.compatible(other)?;
        let mut entries = crate::reserved(self.entries.len(), "§12.1", "matrix difference")?;
        for (a, b) in self.entries.iter().zip(&other.entries) {
            entries.push(if subtract { a.sub(b)? } else { a.add(b)? });
        }
        Self::new(self.rows, self.cols, entries)
    }
    pub(super) fn hermitian(&self) -> Result<()> {
        if self.rows != self.cols {
            return err("§12.1", "Hermitian matrix must be square");
        }
        for row in 0..self.rows {
            if !self.at(row, row).is_real() {
                return err("§12.1", "Hermitian diagonal must be real");
            }
            for col in 0..row {
                if *self.at(row, col) != self.at(col, row).conjugate() {
                    return err("§12.1", "matrix intervals are not Hermitian");
                }
            }
        }
        Ok(())
    }
}

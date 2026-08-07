use num::{BigUint, One, Zero};
use std::vec::Vec;

/// Binomial coefficients, cached with good ol' Pascal's triangle.
pub struct MemoizedBinomialCoefficient {
  t: Vec<BigUint>,
  // Number of complete rows in t, i.e. rows 0..rows.
  rows: usize,
  // Zero value to reference for k > n.
  zero: BigUint,
}

impl MemoizedBinomialCoefficient {
  pub fn new() -> MemoizedBinomialCoefficient {
    return MemoizedBinomialCoefficient {
      t: vec![],
      rows: 0,
      zero: BigUint::zero(),
    };
  }

  /// Returns n choose k, computing any rows not cached yet. None when k > n.
  pub fn comb(&mut self, n: usize, k: usize) -> &BigUint {
    // Return zero when k > n, matching num::integer::binomial.
    if k > n {
      return &self.zero;
    }
    self.grow_to(n);
    return &self.t[Self::index(n, k)];
  }

  /// Computes every row up to and including row n.
  fn grow_to(&mut self, n: usize) {
    if n < self.rows {
      return;
    }
    self.t.reserve(Self::index(n, n) + 1 - self.t.len());
    for row in self.rows..=n {
      // The edges of the triangle are 1, everything between is the sum of the two entries above.
      self.t.push(BigUint::one());
      for k in 1..row {
        let value = &self.t[Self::index(row - 1, k - 1)] + &self.t[Self::index(row - 1, k)];
        self.t.push(value);
      }
      if row > 0 {
        self.t.push(BigUint::one());
      }
    }
    self.rows = n + 1;
  }

  /// Index of n choose k in t.
  fn index(n: usize, k: usize) -> usize {
    ((n * (n + 1)) / 2) + k
  }
}

#[cfg(test)]
mod tests {
  use num::integer::binomial;
  use rstest::rstest;

  use super::*;

  #[test]
  fn matches_the_definition_for_small_rows() {
    let mut c = MemoizedBinomialCoefficient::new();
    for n in 0..20usize {
      for k in 0..=n {
        let expected = BigUint::from(binomial(n as u64, k as u64));
        assert_eq!(*c.comb(n, k), expected, "comb({n}, {k})");
      }
    }
  }

  #[rstest]
  #[case(0, 0, &BigUint::from(1u64))]
  #[case(5, 2, &BigUint::from(10u64))]
  #[case(10, 5, &BigUint::from(252u64))]
  fn known_values(#[case] n: usize, #[case] k: usize, #[case] expected: &BigUint) {
    let mut c = MemoizedBinomialCoefficient::new();
    assert_eq!(c.comb(n, k), expected);
  }

  #[test]
  fn works_for_larger_than_u64() {
    let mut c = MemoizedBinomialCoefficient::new();
    assert_eq!(
      c.comb(100, 50).to_string(),
      "100891344545564193334812497256"
    );
  }

  #[test]
  fn is_zero_above_the_diagonal() {
    let mut c = MemoizedBinomialCoefficient::new();
    assert_eq!(*c.comb(3, 4), BigUint::ZERO);
    assert_eq!(*c.comb(0, 1), BigUint::ZERO);
  }

  #[test]
  fn caches_across_calls_in_any_order() {
    let mut c = MemoizedBinomialCoefficient::new();
    assert_eq!(*c.comb(30, 15), BigUint::from(binomial(30u64, 15u64)));
    assert_eq!(*c.comb(4, 2), BigUint::from(binomial(4u64, 2u64)));
    assert_eq!(*c.comb(30, 15), BigUint::from(binomial(30u64, 15u64)));
    assert_eq!(*c.comb(40, 20), BigUint::from(binomial(40u64, 20u64)));
    assert_eq!(c.t.len(), MemoizedBinomialCoefficient::index(40, 40) + 1);
  }
}

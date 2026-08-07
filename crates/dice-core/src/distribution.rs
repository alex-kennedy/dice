use crate::sum_distributions;

/// Distribution represents a probability distribution of integer types for dice. It's stored as a
/// PMF with an offset for the minimum.
#[derive(Clone, Debug)]
pub struct Distribution {
  pmf: Vec<f64>,
  minimum: i32,
}

impl Distribution {
  /// Creates a new distribution where P(X = constant) = 1.
  pub fn new_constant(constant: i32) -> Distribution {
    return Distribution {
      pmf: vec![1.0],
      minimum: constant,
    };
  }

  /// Creates a new uniform distribution from 1..=faces.
  pub fn new_uniform(faces: u32) -> Distribution {
    let probability = 1.0 / (faces as f64);
    return Distribution {
      pmf: vec![probability; faces as usize],
      minimum: 1,
    };
  }

  /// Creates a new distribution directly from a PMF where index zero represents the given minimum
  /// value.
  pub fn new(pmf: Vec<f64>, minimum: i32) -> Distribution {
    Distribution {
      pmf: pmf,
      minimum: minimum,
    }
  }

  /// Length of the PMF, i.e. the total number of possibilities.
  pub fn len(&self) -> usize {
    self.pmf.len()
  }

  /// Returns true iff the distribution has a probability of 1.0 for a single integer.
  pub fn is_constant(&self) -> bool {
    self.len() == 1
  }

  /// Returns the direct underlying PMF.
  pub fn pmf(&self) -> &Vec<f64> {
    &self.pmf
  }

  /// Returns a mutable reference for the direct underlying PMG.
  pub fn mut_pmf(&mut self) -> &mut Vec<f64> {
    &mut self.pmf
  }

  // Returns the minimum value with a non-zero probability.
  pub fn min(&self) -> i32 {
    self.minimum
  }

  // Returns the maximum value with a non-zero probability.
  pub fn max(&self) -> i32 {
    self.minimum + self.len() as i32 - 1
  }

  // Returns the probability mass for the given outcome.
  pub fn pmf_at(&self, x: i32) -> f64 {
    if x < self.min() || x > self.max() {
      return 0.0;
    }
    self.pmf[(x - self.min()) as usize]
  }

  /// The value paired with its probabilities, ascending.
  pub fn iter(&self) -> impl Iterator<Item = (i32, f64)> + '_ {
    self
      .pmf
      .iter()
      .enumerate()
      .map(|(i, p)| (self.minimum + i as i32, *p))
  }

  /// Highest probability for any outcome.
  pub fn pmf_max(&self) -> f64 {
    self.pmf().iter().fold(-f64::INFINITY, |a, &b| a.max(b))
  }

  /// Returns the cumulative distribution, beginning at the minimum value.
  pub fn cdf(&self) -> Vec<f64> {
    let mut cdf = vec![0.0; self.len()];
    cdf[0] = self.pmf[0];
    for i in 1..self.len() {
      cdf[i] = cdf[i - 1] + self.pmf[i];
    }
    cdf
  }

  /// Negates all the outcomes, keeping probabilities.
  pub fn negate(&mut self) {
    self.minimum = -self.minimum - self.len() as i32 + 1;
    self.pmf.reverse();
  }

  /// Convolution of this distribution and the given one.
  pub fn add(&self, rhs: Distribution) -> Distribution {
    // Two constants
    if self.len() == 1 && rhs.len() == 1 {
      return Distribution::new_constant(self.minimum + rhs.minimum);
    }

    if self.len() == 1 {
      let mut added = rhs.clone();
      added.minimum += self.minimum;
      return added;
    }
    if rhs.len() == 1 {
      let mut added = self.clone();
      added.minimum += rhs.minimum;
      return added;
    }
    sum_distributions(vec![self, &rhs], None)
  }

  /// Distribution obtained by summing this distribution n times.
  pub fn multiply(&self, n: i32) -> Distribution {
    if self.is_constant() {
      return Distribution::new_constant(self.minimum * n);
    }
    sum_distributions(vec![&self], Some(n))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_constant_distribution() {
    let d = Distribution::new_constant(5);

    assert_eq!(d.is_constant(), true);
    assert_eq!(d.min(), 5);
    assert_eq!(d.max(), 5);
    assert_eq!(d.len(), 1);
    assert_eq!(*d.pmf(), vec![1.0f64]);
    assert_eq!(*d.cdf(), vec![1.0f64]);
  }

  #[test]
  fn test_uniform() {
    let d = Distribution::new_uniform(4);

    assert_eq!(d.is_constant(), false);
    assert_eq!(d.min(), 1);
    assert_eq!(d.max(), 4);
    assert_eq!(d.len(), 4);
    assert_eq!(d.pmf_at(-100), 0.0);
    assert_eq!(d.pmf_at(1), 0.25);
    assert_eq!(*d.pmf(), vec![0.25f64, 0.25f64, 0.25f64, 0.25f64]);
    assert_eq!(*d.cdf(), vec![0.25f64, 0.5f64, 0.75f64, 1.0f64]);
  }

  #[test]
  fn test_multiply_has_correct_bounds() {
    let d = Distribution::new_uniform(4).multiply(4);

    assert_eq!(d.min(), 4);
    assert_eq!(d.max(), 16);
  }

  #[test]
  fn test_add_has_correct_bounds() {
    let d = Distribution::new_uniform(4).add(Distribution::new_uniform(4));

    assert_eq!(d.min(), 2);
    assert_eq!(d.max(), 8);
  }

  #[test]
  fn test_negation_has_correct_bounds() {
    let mut d = Distribution::new_uniform(4);
    d.negate();

    assert_eq!(d.pmf_at(-1), 0.25);
    assert_eq!(d.min(), -4);
    assert_eq!(d.max(), -1);
  }
}

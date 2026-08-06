use crate::distribution::Distribution;
use num::{One, Zero};
use rustfft::{FftPlanner, num_complex::Complex64};

/// Add (convolution) one or more probability distributions, optionally repeating the result by a
/// multiplier. E.g. 2d4, could be computed by passing a uniform distribution from 1..4 with a
/// multiplier of 2.
pub fn sum_distributions(
  distributions: Vec<&Distribution>,
  multiplier: Option<i32>,
) -> Distribution {
  if distributions.is_empty() {
    return Distribution::new_constant(0);
  }
  if distributions.len() == 1 && multiplier == Some(1) {
    return distributions[0].clone();
  }
  let negative_multiplier = multiplier.unwrap_or(1).is_negative();
  let multiplier_value = multiplier.unwrap_or(1).abs();

  // Shift all the values down by the minimum value so arrays can begin at zero.
  let min_individual_roll: i32 = distributions.iter().map(|d| d.min()).min().unwrap();
  let total_shift = min_individual_roll * distributions.len() as i32 * multiplier_value;
  let min_total_roll: i32 = distributions.iter().map(|d| d.min()).sum::<i32>() * multiplier_value;
  let max_total_roll: i32 = distributions
    .iter()
    .map(|d| d.max() - min_individual_roll)
    .sum::<i32>()
    * multiplier_value;
  let fft_length = max_total_roll as usize + 1;

  let mut planner = FftPlanner::<f64>::new();
  let fft = planner.plan_fft_forward(fft_length);
  let ifft = planner.plan_fft_inverse(fft_length);

  let mut distributions_transformed: Vec<Vec<Complex64>> =
    vec![vec![Complex64::zero(); fft_length]; distributions.len()];
  for (i, d) in distributions.iter().enumerate() {
    for x in d.min()..d.max() + 1 {
      distributions_transformed[i][(x - min_individual_roll) as usize] =
        Complex64::new(d.pdf_at(x), 0.0);
    }
    fft.process(distributions_transformed[i].as_mut_slice());
  }

  let mut output_complex = vec![Complex64::one(); fft_length];
  for i in 0..distributions.len() {
    for x in 0..fft_length {
      output_complex[x] *= distributions_transformed[i][x];
    }
  }
  if multiplier_value != 1 {
    for x in 0..fft_length {
      output_complex[x] = output_complex[x].powi(multiplier_value);
    }
  }

  ifft.process(&mut output_complex);
  let normalisation_factor = fft_length as f64;
  let output = output_complex[(min_total_roll - total_shift) as usize..]
    .iter()
    .map(|x| (x.re / normalisation_factor).abs())
    .collect();

  let mut dist = Distribution::new(output, min_total_roll);
  if negative_multiplier {
    dist.negate();
  }
  dist
}

#[cfg(test)]
mod tests {
  use rstest::rstest;

  use super::*;

  /// Direct O(n*m) convolution, i.e. the definition of a sum of two independent variables.
  fn naive_sum(a: &Distribution, b: &Distribution) -> Distribution {
    let mut pdf = vec![0.0; a.len() + b.len() - 1];
    for (i, p) in a.pdf().iter().enumerate() {
      for (j, q) in b.pdf().iter().enumerate() {
        pdf[i + j] += p * q;
      }
    }
    Distribution::new(pdf, a.min() + b.min())
  }

  fn assert_close(actual: &Distribution, expected: &Distribution) {
    assert_eq!(actual.min(), expected.min(), "minimum");
    assert_eq!(actual.len(), expected.len(), "length");
    for (i, (got, want)) in actual.pdf().iter().zip(expected.pdf()).enumerate() {
      assert!(
        (got - want).abs() < 1e-9,
        "index {i}: got {got}, want {want}"
      );
    }
  }

  /// A distribution over `min..=min+weights.len()-1`, normalised.
  fn skewed(weights: &[f64], min: i32) -> Distribution {
    let total: f64 = weights.iter().sum();
    Distribution::new(weights.iter().map(|w| w / total).collect(), min)
  }

  #[rstest]
  #[case(Distribution::new_uniform(6), Distribution::new_uniform(6))]
  #[case(Distribution::new_uniform(6), Distribution::new_uniform(4))]
  #[case(Distribution::new_uniform(20), Distribution::new_uniform(3))]
  // Distributions that don't start at 1, so the shift is doing real work.
  #[case(skewed(&[1.0, 2.0, 3.0], 3), skewed(&[5.0, 1.0], 7))]
  #[case(skewed(&[1.0, 2.0, 3.0], -4), skewed(&[5.0, 1.0, 1.0], 2))]
  #[case(skewed(&[1.0, 2.0], -9), Distribution::new_uniform(6))]
  fn test_sum_of_two(#[case] a: Distribution, #[case] b: Distribution) {
    assert_close(&sum_distributions(vec![&a, &b], None), &naive_sum(&a, &b));
  }

  #[rstest]
  #[case(2)]
  #[case(3)]
  #[case(5)]
  fn test_multiplier_matches_repeated_summing(#[case] copies: i32) {
    let d = skewed(&[1.0, 2.0, 3.0, 1.0], 2);

    let mut expected = d.clone();
    for _ in 1..copies {
      expected = naive_sum(&expected, &d);
    }

    assert_close(&sum_distributions(vec![&d], Some(copies)), &expected);
  }

  #[test]
  fn test_sum_of_three() {
    let a = Distribution::new_uniform(6);
    let b = skewed(&[1.0, 2.0, 3.0], 3);
    let c = skewed(&[5.0, 1.0], -2);

    let expected = naive_sum(&naive_sum(&a, &b), &c);
    assert_close(&sum_distributions(vec![&a, &b, &c], None), &expected);
  }

  /// A negative multiplier means "negate the sum", i.e. 1 * -1 == -1. Constants are the degenerate
  /// case: the transform is length 1, so only the shift arithmetic is under test here.
  #[test]
  fn test_negative_multiplier_on_constant() {
    let result = sum_distributions(vec![&Distribution::new_constant(1)], Some(-1));
    assert_eq!(result.min(), -1);
    assert_eq!(result.pdf(), &vec![1.0]);
  }

  /// -n copies is the negation of n copies, e.g. -2 copies of a d4 is -(2d4).
  #[rstest]
  #[case(1)]
  #[case(2)]
  #[case(3)]
  fn test_negative_multiplier_negates_the_sum(#[case] copies: i32) {
    let d = skewed(&[1.0, 2.0, 3.0, 1.0], 2);

    let mut expected = d.clone();
    for _ in 1..copies {
      expected = naive_sum(&expected, &d);
    }
    expected.negate();

    assert_close(&sum_distributions(vec![&d], Some(-copies)), &expected);
  }

  #[test]
  fn test_no_distributions_is_zero() {
    let result = sum_distributions(vec![], None);
    assert_eq!(result.min(), 0);
    assert_eq!(result.pdf(), &vec![1.0]);
  }
}

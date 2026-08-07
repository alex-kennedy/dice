// Calculates a BestK or WorstK style distribution.
// Implements the calculation on page 9 of:
//
// Evans, Diane L., Lawrence M. Leemis, and John H. Drew. "The distribution of order statistics for
// discrete random variables with applications to bootstrapping." INFORMS Journal on Computing 18.1
// (2006): 19-30. https://doi.org/10.1287/ijoc.1040.0105

use crate::distribution::Distribution;

use num::integer::{binomial, multinomial};

// Decides which order statistic to return, the best or worst k of n
#[derive(PartialEq)]
pub enum Order {
  Best,
  Worst,
}

struct MemoizedMultinomialCoefficient {
  map: std::collections::HashMap<(i32, i32, i32), i32>,
}

impl MemoizedMultinomialCoefficient {
  pub fn new() -> MemoizedMultinomialCoefficient {
    return MemoizedMultinomialCoefficient {
      map: std::collections::HashMap::new(),
    };
  }

  pub fn get(&mut self, a: i32, b: i32, c: i32) -> i32 {
    let key = (a, b, c);
    match self.map.get(&key) {
      Some(value) => return *value,
      None => {
        let value = multinomial(&[a, b, c]);
        self.map.insert(key, value);
        return value;
      }
    }
  }
}

/// Calculates order statistics for a given distribution. I.e. the distribution obtained by
/// realising the distribution n times and taking the k-Order values. Returns a vector of
/// distributions in the given order (e.g best->worst if order = Best).
pub fn calculate_order_distributions(
  d: &Distribution,
  n: i32,
  k: i32,
  order: Order,
) -> Vec<Distribution> {
  if n == 0 || k == 0 {
    return vec![];
  }

  let mut output_distributions = vec![d.clone(); k as usize];
  let r: Vec<i32> = match order {
    Order::Best => (n - k..n).collect(),
    Order::Worst => (0..k).collect(),
  };

  let mut mc_calculator = MemoizedMultinomialCoefficient::new();
  let pmf = d.pmf();
  let cdf = d.cdf();
  for i in 0..r.len() {
    // x = 0
    let mut sum = 0.0;
    for w in 0..n - r[i] {
      sum += binomial(n, w) as f64 * pmf[0].powi((n - w) as i32) * (1.0 - cdf[0]).powi(w as i32);
    }
    output_distributions[i].mut_pmf()[0] = sum;

    // x = 1, 2, ..., len(d) - 2
    for x in 1..d.len() - 1 {
      let mut sum = 0.0;
      for u in 0..r[i] + 1 {
        for w in 0..n - r[i] {
          sum += mc_calculator.get(u, n - u - w, w) as f64
            * cdf[x - 1].powi(u as i32)
            * pmf[x].powi((n - u - w) as i32)
            * (1.0 - cdf[x]).powi(w as i32);
        }
      }
      output_distributions[i].mut_pmf()[x] = sum;
    }

    // x = len(d) - 1
    let mut sum = 0.0;
    for u in 0..r[i] + 1 {
      sum += binomial(n, u) as f64 * cdf[d.len() - 2].powi(u as i32) * pmf[d.len() - 1].powi(n - u);
    }
    output_distributions[i].mut_pmf()[d.len() - 1] = sum;
  }
  // Currently in worst -> best order, reverse that if Order::Best is requested.
  if order == Order::Best {
    output_distributions.reverse();
  }
  return output_distributions;
}

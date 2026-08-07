/// Quantiles to calculate, quartiles.
const QUANTILES: [u8; 3] = [25, 50, 75];

/// Summary statistics of a distribution, in the order they're worth reading.
pub struct Stats {
  pub mean: f64,
  pub sd: f64,
  pub min: i32,
  pub max: i32,
  /// The likeliest single outcome's probability.
  pub peak: f64,
  /// Quantiles as (label percent, value) pairs, e.g. `(25, 9)`.
  pub quantiles: Vec<(u8, i32)>,
}

/// Summarises a pmf given as ascending (outcome, probability) pairs.
pub fn summarize(pmf: &[(i32, f64)]) -> Stats {
  let mean = pmf.iter().map(|(v, p)| *v as f64 * p).sum::<f64>();
  let variance = pmf
    .iter()
    .map(|(v, p)| p * (*v as f64 - mean).powi(2))
    .sum::<f64>();

  Stats {
    mean,
    sd: variance.sqrt(),
    min: pmf.first().map_or(0, |(v, _)| *v),
    max: pmf.last().map_or(0, |(v, _)| *v),
    peak: pmf.iter().map(|(_, p)| *p).fold(0.0, f64::max),
    quantiles: QUANTILES
      .iter()
      .map(|q| (*q, quantile(pmf, *q as f64 / 100.0)))
      .collect(),
  }
}

/// The smallest outcome whose cumulative probability reaches `q`.
fn quantile(pmf: &[(i32, f64)], q: f64) -> i32 {
  let mut cumulative = 0.0;
  for (value, p) in pmf {
    cumulative += p;
    if cumulative >= q {
      return *value;
    }
  }
  pmf.last().map_or(0, |(v, _)| *v)
}

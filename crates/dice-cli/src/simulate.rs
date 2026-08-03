use std::collections::HashMap;
use std::error::Error;
use std::time::Instant;

use dice_core::parser::{self, Expr, ParseError};
use dice_core::simulate_expression;

use crate::render;

/// Quantiles to calculate, quartiles.
const QUANTILES: [u8; 3] = [25, 50, 75];

/// Summary statistics of a simulated distribution, in the order they're worth reading.
pub struct Stats {
  pub runs: usize,
  pub mean: f64,
  pub sd: f64,
  pub min: i32,
  pub max: i32,
  /// Quantiles as (label percent, value) pairs, e.g. `(25, 9)`.
  pub quantiles: Vec<(u8, i32)>,
}

/// How long a single roll of the expression takes.
pub struct Timing {
  pub mean_nanos: f64,
  pub sd_nanos: f64,
}

/// Runs a simulation for the expression `runs` times, rendering the results.
pub fn run(expression: &str, runs: usize) -> Result<(), Box<dyn Error>> {
  let expr = parser::parse(expression).map_err(|err| describe(expression, &err))?;

  let (counts, timing) = roll_many(&expr, runs);
  let mut pmf: Vec<(i32, f64)> = counts
    .into_iter()
    .map(|(value, count)| (value, count as f64 / runs as f64))
    .collect();
  pmf.sort_by_key(|(value, _)| *value);

  let stats = summarize(&pmf, runs);
  render::show(expression, &pmf, &stats, &timing)?;
  Ok(())
}

/// Rolls the expression `runs` times, recording how long each roll took alongside its
/// outcome.
fn roll_many(expr: &Expr, runs: usize) -> (HashMap<i32, u64>, Timing) {
  let mut counts: HashMap<i32, u64> = HashMap::new();
  let mut total = 0.0;
  let mut total_squares = 0.0;

  for _ in 0..runs {
    let started = Instant::now();
    let roll = simulate_expression(expr);
    let nanos = started.elapsed().as_nanos() as f64;

    *counts.entry(roll).or_default() += 1;
    total += nanos;
    total_squares += nanos * nanos;
  }

  let mean = total / runs as f64;
  // Clamped because catastrophic cancellation can push this a hair below zero when every
  // roll takes the same time.
  let variance = (total_squares / runs as f64 - mean * mean).max(0.0);
  (
    counts,
    Timing {
      mean_nanos: mean,
      sd_nanos: variance.sqrt(),
    },
  )
}

fn summarize(pmf: &[(i32, f64)], runs: usize) -> Stats {
  let mean = pmf.iter().map(|(v, p)| *v as f64 * p).sum::<f64>();
  let variance = pmf
    .iter()
    .map(|(v, p)| p * (*v as f64 - mean).powi(2))
    .sum::<f64>();

  Stats {
    runs,
    mean,
    sd: variance.sqrt(),
    min: pmf.first().map_or(0, |(v, _)| *v),
    max: pmf.last().map_or(0, |(v, _)| *v),
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

/// Renders a parse failure with the offending span underlined, e.g.
///
/// ```text
/// error: unexpected `+`, expected one of: ...
///   2d6 ++ 4
///       ^
/// ```
fn describe(input: &str, err: &ParseError) -> String {
  // The error carries byte offsets, but the caret has to line up with printed columns.
  let chars_before = |offset: usize| {
    input
      .get(..offset)
      .map_or(offset, |prefix| prefix.chars().count())
  };
  let start = chars_before(err.range.start);
  let width = chars_before(err.range.end).saturating_sub(start).max(1);

  format!(
    "error: {}\n  {input}\n  {}{}",
    err.message,
    " ".repeat(start),
    "^".repeat(width)
  )
}

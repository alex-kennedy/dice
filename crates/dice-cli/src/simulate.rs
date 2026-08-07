use std::collections::HashMap;
use std::error::Error;
use std::time::Instant;

use dice_core::parser::Expr;
use dice_core::simulate_expression;

use crate::parse::parse;
use crate::render::{self, Timing};

/// Runs a simulation for the expression runs times, rendering the results.
pub fn run(expression: &str, runs: usize) -> Result<(), Box<dyn Error>> {
  let expr = parse(expression)?;

  let (counts, mean_nanos, sd_nanos) = roll_many(&expr, runs);
  let mut pmf: Vec<(i32, f64)> = counts
    .into_iter()
    .map(|(value, count)| (value, count as f64 / runs as f64))
    .collect();
  pmf.sort_by_key(|(value, _)| *value);

  let timing = Timing::Simulated {
    runs,
    mean_nanos,
    sd_nanos,
  };
  render::show(expression, &pmf, &timing)?;
  Ok(())
}

/// Rolls the expression `runs` times, returning the outcomes alongside the mean and
/// standard deviation of how long a single roll took, in nanoseconds.
fn roll_many(expr: &Expr, runs: usize) -> (HashMap<i32, u64>, f64, f64) {
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
  // Clamped because cancellation can push this a hair below zero when every roll takes the same
  // time.
  let variance = (total_squares / runs as f64 - mean * mean).max(0.0);
  (counts, mean, variance.sqrt())
}

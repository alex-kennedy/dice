use std::error::Error;
use std::time::Instant;

use dice_core::calculate_expression;

use crate::parse::{describe, parse};
use crate::render::{self, Timing};

/// Calculates the exact distribution of the expression, rendering the results.
pub fn run(expression: &str) -> Result<(), Box<dyn Error>> {
  let expr = parse(expression)?;

  let started = Instant::now();
  let distribution = calculate_expression(&expr)
    .map_err(|err| describe(expression, &err.message, &err.range))?;
  let nanos = started.elapsed().as_nanos() as f64;

  let pmf: Vec<(i32, f64)> = distribution.iter().collect();
  render::show(expression, &pmf, &Timing::Calculated { nanos })?;
  Ok(())
}

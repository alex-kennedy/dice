use std::fmt;
use std::ops::Range;

use crate::{
  Distribution,
  parser::{BinOpKind, Expr, ExprKind, UnOpKind},
};

/// A dice expression that parses, but has no distribution that can be calculated exactly.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CalculateError {
  /// Human-readable description of what went wrong.
  pub message: String,
  /// Byte offsets into the input string where the problem was found.
  pub range: Range<usize>,
}

impl fmt::Display for CalculateError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{} (at {}..{})",
      self.message, self.range.start, self.range.end
    )
  }
}

impl std::error::Error for CalculateError {}

/// Calculates the probability distribution for a dice rolling expression.
pub fn calculate_expression(expr: &Expr) -> Result<Distribution, CalculateError> {
  match &expr.kind {
    ExprKind::Constant(n) => return Ok(Distribution::new_constant(*n)),
    ExprKind::Dice(d) => return Ok(d.distribution()),
    ExprKind::BinOp(lhs, op, rhs) => {
      let op_range = lhs.range.end..rhs.range.start;
      let lhs = calculate_expression(lhs)?;
      let rhs = calculate_expression(rhs)?;
      match op {
        BinOpKind::Add => return Ok(lhs.add(rhs)),
        BinOpKind::Mul => match (lhs.is_constant(), rhs.is_constant()) {
          (true, _) => return Ok(rhs.multiply(lhs.min())),
          (_, true) => return Ok(lhs.multiply(rhs.min())),
          _ => {
            return Err(CalculateError {
              message: "can't multiply two dice terms exactly, one side must be a constant"
                .to_string(),
              range: op_range,
            });
          }
        },
      };
    }
    ExprKind::UnOp(op, rhs) => {
      let mut rhs = calculate_expression(rhs)?;
      match op {
        UnOpKind::Neg => rhs.negate(),
      }
      return Ok(rhs);
    }
  };
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::parser::parse;
  use rstest::rstest;

  #[rstest]
  #[case("d6*d6", 2..3)]
  #[case("2d6 * 3d6", 3..6)]
  #[case("1 + d6*d6", 6..7)]
  #[case("(d6+1)*(d6+1)", 6..7)]
  #[case("2*d6*d6", 4..5)]
  #[case("d6*(d6-d6)", 2..3)]
  #[case("2(d6*d6)", 4..5)]
  fn test_calculate_multiplies_two_dice_terms(
    #[case] query: &str,
    #[case] expected_range: Range<usize>,
  ) {
    let expr = parse(query).expect("expected the expression to parse");
    let err = calculate_expression(&expr).expect_err("expected a calculate error");
    assert_eq!(err.range, expected_range);
    assert!(
      err.message.contains("can't multiply two dice terms"),
      "{}",
      err.message
    );
  }

  #[rstest]
  #[case("d6*2")]
  #[case("2*d6")]
  #[case("2(d6+1)")]
  #[case("d6*(1+1)")]
  #[case("(d6-d6)*3")]
  #[case("-2*d6")]
  #[case("-d6")]
  fn test_calculate_allows_constant_factors(#[case] query: &str) {
    let expr = parse(query).expect("expected the expression to parse");
    assert!(calculate_expression(&expr).is_ok());
  }
}

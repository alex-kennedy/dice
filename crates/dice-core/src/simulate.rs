use crate::parser::{BinOpKind, Expr, ExprKind, UnOpKind};

// Simulates the given expression once, rolling relevant dice.
pub fn simulate_expression(expr: &Expr) -> i32 {
  match &expr.kind {
    ExprKind::Constant(n) => *n,
    // Use the thread local rng for dice rolling.
    ExprKind::Dice(d) => d.roll(&mut rand::rng()) as i32,
    ExprKind::BinOp(lhs, op, rhs) => {
      let lhs = simulate_expression(&lhs);
      let rhs = simulate_expression(&rhs);
      match op {
        BinOpKind::Add => lhs + rhs,
        BinOpKind::Mul => lhs * rhs,
      }
    }
    ExprKind::UnOp(op, rhs) => {
      let rhs = simulate_expression(&rhs);
      match op {
        UnOpKind::Neg => -rhs,
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::parser::parse;
  use rstest::rstest;

  use super::*;

  #[rstest]
  #[case("1+1", 2)]
  #[case("1*1", 1)]
  #[case("1*-1", -1)]
  #[case("-1*-1", 1)]
  #[case("(2*2)+1", 5)]
  #[case("2*2+1", 5)]
  #[case("2*(2+1)", 6)]
  #[case("2*(((((((((((2+1)))))))))))", 6)]
  #[case("1*1*1*1*1*1*1*1*1*1", 1)]
  #[case("-1", -1)]
  #[case("((((1+(2+(3+4+5))))+((+6+--7)+(8+9))))", 1+2+3+4+5+6+7+8+9)]
  fn evaluates_constant_expression_correctly(#[case] expr: &str, #[case] result: i32) {
    let parsed = parse(expr).expect("failed to parse");
    assert_eq!(simulate_expression(&parsed), result);
  }

  #[rstest]
  #[case("2*d6", 2, 12)]
  #[case("-d6", -6, -1)]
  #[case("d2+d2+d2+d2+d2+(d2+d2)", 7, 14)]
  fn dice_term_is_within_bounds(#[case] expr: &str, #[case] min: i32, #[case] max: i32) {
    let parsed = parse(expr).expect("failed to parse");
    assert!(simulate_expression(&parsed) >= min);
    assert!(simulate_expression(&parsed) <= max);
  }
}

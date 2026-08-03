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

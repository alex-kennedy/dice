use wasm_bindgen::prelude::*;

use dice_core::parser::{Expr, ExprKind};

/// Parses an expression and tags ranges of the input query with their types. Does not guarantee
/// that all parts of the query will be matched to a range.
#[wasm_bindgen]
pub fn parse(query: String) -> Vec<Range> {
  match dice_core::parser::parse(&query) {
    Ok(expr) => {
      let mut flattened = vec![];
      flatten_expr(&expr, &mut flattened);
      flattened
    }
    Err(e) => {
      vec![Range {
        start: e.range.start,
        end: e.range.end,
        kind: ExpressionKind::Error,
        dice_faces: None,
      }]
    }
  }
}

/// Ranges of the input expression tagged by their type for syntax highlighting.
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Range {
  pub start: usize,
  pub end: usize,
  pub kind: ExpressionKind,
  pub dice_faces: Option<usize>,
}

/// The type of the expression range.
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExpressionKind {
  /// Operator, like +, -, *.
  Operator,

  /// Constant, like 8.
  Constant,

  /// Dice term, like d6.
  Dice,

  /// An error in the expression.
  Error,
}

/// Flattens an expression into the ranges for syntax highlighting.
fn flatten_expr(expr: &Expr, flat: &mut Vec<Range>) {
  match &expr.kind {
    ExprKind::Constant(_) => {
      flat.push(Range {
        start: expr.range.start,
        end: expr.range.end,
        kind: ExpressionKind::Constant,
        dice_faces: None,
      });
    }
    ExprKind::Dice(d) => {
      flat.push(Range {
        start: expr.range.start,
        end: expr.range.end,
        kind: ExpressionKind::Dice,
        dice_faces: Some(d.faces as usize),
      });
    }
    ExprKind::UnOp(_, rhs) => {
      flat.push(Range {
        start: expr.range.start,
        end: rhs.range.start,
        kind: ExpressionKind::Operator,
        dice_faces: None,
      });
      flatten_expr(rhs, flat);
    }
    ExprKind::BinOp(lhs, _, rhs) => {
      flatten_expr(lhs, flat);
      flat.push(Range {
        start: lhs.range.end,
        end: rhs.range.start,
        kind: ExpressionKind::Operator,
        dice_faces: None,
      });
      flatten_expr(rhs, flat);
    }
  };
}

#[cfg(test)]
mod tests {
  use super::*;
  use rstest::rstest;

  #[rstest]
  #[case("3", vec![
    Range { start: 0, end: 1, kind: ExpressionKind::Constant, dice_faces: None },
  ])]
  #[case("d6", vec![
    Range { start: 0, end: 2, kind: ExpressionKind::Dice, dice_faces: Some(6) },
  ])]
  #[case("2d6", vec![
    Range { start: 0, end: 3, kind: ExpressionKind::Dice, dice_faces: Some(6) },
  ])]
  #[case("-3", vec![
    Range { start: 0, end: 1, kind: ExpressionKind::Operator, dice_faces: None },
    Range { start: 1, end: 2, kind: ExpressionKind::Constant, dice_faces: None },
  ])]
  #[case("1+2", vec![
    Range { start: 0, end: 1, kind: ExpressionKind::Constant, dice_faces: None },
    Range { start: 1, end: 2, kind: ExpressionKind::Operator, dice_faces: None },
    Range { start: 2, end: 3, kind: ExpressionKind::Constant, dice_faces: None },
  ])]
  #[case("2d6 + 3", vec![
    Range { start: 0, end: 3, kind: ExpressionKind::Dice, dice_faces: Some(6) },
    Range { start: 3, end: 6, kind: ExpressionKind::Operator, dice_faces: None },
    Range { start: 6, end: 7, kind: ExpressionKind::Constant, dice_faces: None },
  ])]
  fn test_parse(#[case] query: &str, #[case] expected: Vec<Range>) {
    assert_eq!(parse(query.to_string()), expected);
  }

  #[test]
  fn test_parse_error() {
    let result = parse("+".to_string());
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].kind, ExpressionKind::Error);
  }
}

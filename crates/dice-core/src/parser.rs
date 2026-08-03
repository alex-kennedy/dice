use std::fmt;
use std::ops::Range;
use std::unreachable;

use crate::dice::{DicePool, Keep};

use lalrpop_util::lalrpop_mod;
use pratt::{Affix, Associativity, PrattError, PrattParser, Precedence};

lalrpop_mod!(grammar, "grammar.rs");

/// Parses a dice expression, e.g. `2d20d5 + 3`, into its expression tree.
pub fn parse(input: &str) -> std::result::Result<Expr, ParseError> {
  let tokens = grammar::TokenTreeParser::new()
    .parse(input)
    .map_err(ParseError::from)?;
  let mut expr_parser = ExprParser;
  expr_parser
    .parse(&mut tokens.into_iter())
    .map_err(ExprError::from)
    .map_err(ParseError::from)
}

/// A failure to parse a dice expression.
#[derive(Debug)]
pub struct ParseError {
  /// Human-readable description of what went wrong.
  pub message: String,
  /// Byte offsets into the input string where the problem was found.
  pub range: Range<usize>,
}

impl fmt::Display for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{} (at {}..{})",
      self.message, self.range.start, self.range.end
    )
  }
}

impl std::error::Error for ParseError {}

/// A problem converting a matched token into its value, e.g. a number too large to
/// represent. Raised by the grammar's fallible actions, which is why it carries its own
/// byte range: it's reported against the token, not the whole input.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TokenError {
  pub message: String,
  pub range: Range<usize>,
}

impl fmt::Display for TokenError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&self.message)
  }
}

impl<T: fmt::Display> From<lalrpop_util::ParseError<usize, T, TokenError>> for ParseError {
  fn from(err: lalrpop_util::ParseError<usize, T, TokenError>) -> Self {
    use lalrpop_util::ParseError as E;
    match err {
      E::InvalidToken { location } => ParseError {
        message: "invalid token".to_string(),
        range: location..location,
      },
      E::UnrecognizedEof { location, expected } => ParseError {
        message: format!(
          "unexpected end of input, expected one of: {}",
          expected.join(", ")
        ),
        range: location..location,
      },
      E::UnrecognizedToken {
        token: (start, token, end),
        expected,
      } => ParseError {
        message: format!(
          "unexpected `{token}`, expected one of: {}",
          expected.join(", ")
        ),
        range: start..end,
      },
      E::ExtraToken {
        token: (start, token, end),
      } => ParseError {
        message: format!("unexpected extra token `{token}`"),
        range: start..end,
      },
      E::User { error } => ParseError {
        message: error.message,
        range: error.range,
      },
    }
  }
}

/// A parsed dice expression node, along with the byte range of the input it was parsed from.
#[derive(Debug, Eq, PartialEq)]
pub struct Expr {
  pub range: Range<usize>,
  pub kind: ExprKind,
}

/// The shape of a parsed dice expression, ignoring where it came from in the input.
#[derive(Debug, Eq, PartialEq)]
pub enum ExprKind {
  BinOp(Box<Expr>, BinOpKind, Box<Expr>),
  UnOp(UnOpKind, Box<Expr>),
  Dice(DicePool),
  Constant(i32),
}

#[derive(Debug, Eq, PartialEq)]
pub enum BinOpKind {
  Add,
  Mul,
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnOpKind {
  Neg,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TokenTree {
  pub range: Range<usize>,
  pub kind: TokenTreeKind,
}

#[derive(Debug, Eq, PartialEq)]
pub enum TokenTreeKind {
  Group(Vec<TokenTree>),
  Sign(char),
  Infix(char),
  Dice(DicePool),
  Constant(i32),
}

/// Parses a constant, reporting an error rather than panicking when it doesn't fit.
///
/// `offset` is where the token starts in the whole input, so the error can point at it.
fn parse_constant(s: &str, offset: usize) -> std::result::Result<i32, TokenError> {
  s.parse().map_err(|_| TokenError {
    message: format!("`{s}` is too large, constants must be at most {}", i32::MAX),
    range: offset..offset + s.len(),
  })
}

/// Parses one numeric field of a die term, reporting an error rather than panicking when
/// it doesn't fit. `what` names the field for the error message.
fn parse_dice_field(s: &str, offset: usize, what: &str) -> std::result::Result<u32, TokenError> {
  s.parse().map_err(|_| TokenError {
    message: format!("{what} `{s}` is too large, must be at most {}", u32::MAX),
    range: offset..offset + s.len(),
  })
}

/// Splits a die-term token such as `2d20d5`, `d20d` or `4d6a3` into its parts.
///
/// The grammar guarantees the shape `[0-9]*d[0-9]+([ad][0-9]*)?`, but says nothing about
/// how big those digit runs are, so each one can still overflow. `offset` is where the
/// token starts in the whole input, so errors can point at the offending field.
fn parse_dice_term(s: &str, offset: usize) -> std::result::Result<DicePool, TokenError> {
  let separator = s.find('d').expect("die term always contains a `d`");
  let count: u32 = if separator == 0 {
    1
  } else {
    parse_dice_field(&s[..separator], offset, "dice count")?
  };

  let rest = &s[separator + 1..];
  let rest_offset = offset + separator + 1;
  let faces_end = rest
    .find(|c: char| !c.is_ascii_digit())
    .unwrap_or(rest.len());
  let faces: u32 = parse_dice_field(&rest[..faces_end], rest_offset, "face count")?;

  let (keep, pool) = match rest[faces_end..].split_at_checked(1) {
    None => (Keep::All, count),
    Some((marker, size)) => {
      let keep = if marker == "a" {
        Keep::Best
      } else {
        Keep::Worst
      };
      // A bare `a`/`d` means one extra die, e.g. `d20d` rolls two and keeps the
      // worse. An explicit size gives the pool directly.
      let pool = if size.is_empty() {
        count.checked_add(1).ok_or_else(|| TokenError {
          message: format!(
            "dice pool of `{s}` is too large, must be at most {}",
            u32::MAX
          ),
          range: offset..offset + s.len(),
        })?
      } else {
        parse_dice_field(size, rest_offset + faces_end + 1, "dice pool")?
      };
      (keep, pool)
    }
  };

  // Keeping more dice than are rolled is invalid.
  if count > pool {
    return Err(TokenError {
      message: format!("{s} keeps {count} dice from a pool of only {pool}"),
      range: offset..offset + s.len(),
    });
  }

  Ok(DicePool {
    count,
    faces,
    pool,
    keep,
  })
}

#[derive(Debug)]
struct ExprError(Box<PrattError<TokenTree, ExprError>>);

impl fmt::Display for ExprError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<PrattError<TokenTree, ExprError>> for ExprError {
  fn from(err: PrattError<TokenTree, ExprError>) -> Self {
    ExprError(Box::new(err))
  }
}

impl From<ExprError> for ParseError {
  fn from(err: ExprError) -> Self {
    ParseError {
      message: err.to_string(),
      range: 0..0,
    }
  }
}

type Result<T> = std::result::Result<T, ExprError>;

struct ExprParser;

impl<I> PrattParser<I> for ExprParser
where
  I: Iterator<Item = TokenTree>,
{
  type Error = ExprError;
  type Input = TokenTree;
  type Output = Expr;

  // Query information about an operator (Affix, Precedence, Associativity)
  fn query(&mut self, tree: &TokenTree) -> Result<Affix> {
    let affix = match tree.kind {
      TokenTreeKind::Infix('+') => Affix::Infix(Precedence(2), Associativity::Left),
      TokenTreeKind::Infix('-') => Affix::Infix(Precedence(2), Associativity::Left),
      TokenTreeKind::Infix('*') => Affix::Infix(Precedence(3), Associativity::Left),
      TokenTreeKind::Sign('-') => Affix::Prefix(Precedence(4)),
      TokenTreeKind::Sign('+') => Affix::Prefix(Precedence(4)),
      TokenTreeKind::Group(_) => Affix::Nilfix,
      TokenTreeKind::Dice(_) => Affix::Nilfix,
      TokenTreeKind::Constant(_) => Affix::Nilfix,
      _ => unreachable!(),
    };
    Ok(affix)
  }

  // Constructs a primary expression, i.e. a die, group, or constant
  fn primary(&mut self, tree: TokenTree) -> Result<Expr> {
    let TokenTree { range, kind } = tree;
    match kind {
      // A group's own range spans its delimiters (parens, or the digits and `(` of an
      // implicit multiplication), which is wider than the union of its children's ranges,
      // so it replaces whatever range the recursive parse computed.
      TokenTreeKind::Group(group) => {
        let mut expr = self.parse(&mut group.into_iter())?;
        expr.range = range;
        Ok(expr)
      }
      TokenTreeKind::Dice(term) => Ok(Expr {
        range,
        kind: ExprKind::Dice(term),
      }),
      TokenTreeKind::Constant(num) => Ok(Expr {
        range,
        kind: ExprKind::Constant(num),
      }),
      _ => unreachable!(),
    }
  }

  // Constructs a binary infix expression, e.g. 1+1
  fn infix(&mut self, lhs: Expr, tree: TokenTree, rhs: Expr) -> Result<Expr> {
    let range = lhs.range.start..rhs.range.end;
    let kind = match tree.kind {
      TokenTreeKind::Infix('+') => ExprKind::BinOp(Box::new(lhs), BinOpKind::Add, Box::new(rhs)),
      TokenTreeKind::Infix('-') => {
        let neg_range = tree.range.start..rhs.range.end;
        ExprKind::BinOp(
          Box::new(lhs),
          BinOpKind::Add,
          Box::new(Expr {
            range: neg_range,
            kind: ExprKind::UnOp(UnOpKind::Neg, Box::new(rhs)),
          }),
        )
      }
      TokenTreeKind::Infix('*') => ExprKind::BinOp(Box::new(lhs), BinOpKind::Mul, Box::new(rhs)),
      _ => unreachable!(),
    };
    Ok(Expr { range, kind })
  }

  fn prefix(&mut self, tree: TokenTree, rhs: Expr) -> Result<Expr> {
    let range = tree.range.start..rhs.range.end;
    match tree.kind {
      TokenTreeKind::Sign('+') => Ok(Expr {
        range,
        kind: rhs.kind,
      }),
      TokenTreeKind::Sign('-') => Ok(Expr {
        range,
        kind: ExprKind::UnOp(UnOpKind::Neg, Box::new(rhs)),
      }),
      _ => unreachable!(),
    }
  }

  // Postfixes are folded into the die term by the grammar, unreachable.
  fn postfix(&mut self, _lhs: Expr, _tree: TokenTree) -> Result<Expr> {
    unreachable!("Postfixes are folded into the die term by the grammar, unreachable.");
  }
}

#[cfg(test)]
mod tests {
  use std::assert_eq;

  use super::*;
  use rstest::rstest;

  fn expr(range: Range<usize>, kind: ExprKind) -> Expr {
    Expr { range, kind }
  }

  fn constant(range: Range<usize>, n: i32) -> Expr {
    expr(range, ExprKind::Constant(n))
  }

  fn add(range: Range<usize>, l: Expr, r: Expr) -> Expr {
    expr(
      range,
      ExprKind::BinOp(Box::new(l), BinOpKind::Add, Box::new(r)),
    )
  }

  fn mul(range: Range<usize>, l: Expr, r: Expr) -> Expr {
    expr(
      range,
      ExprKind::BinOp(Box::new(l), BinOpKind::Mul, Box::new(r)),
    )
  }

  fn neg(range: Range<usize>, e: Expr) -> Expr {
    expr(range, ExprKind::UnOp(UnOpKind::Neg, Box::new(e)))
  }

  fn dice(range: Range<usize>, count: u32, faces: u32, pool: u32, keep: Keep) -> Expr {
    expr(
      range,
      ExprKind::Dice(DicePool {
        count,
        faces,
        pool,
        keep,
      }),
    )
  }

  #[rstest]
  #[case("d3", dice(0..2, 1, 3, 1, Keep::All))]
  #[case("2d6", dice(0..3, 2, 6, 2, Keep::All))]
  #[case("2d6a10", dice(0..6, 2, 6, 10, Keep::Best))]
  #[case("1", constant(0..1, 1))]
  #[case("0", constant(0..1, 0))]
  #[case("1+1", add(0..3, constant(0..1, 1), constant(2..3, 1)))]
  // A leading `+` adds no node to the tree, but still counts towards the range.
  #[case("+1", constant(0..2, 1))]
  #[case("-1", neg(0..2, constant(1..2, 1)))]
  #[case("+d6", dice(0..3, 1, 6, 1, Keep::All))]
  #[case("-d6", neg(0..3, dice(1..3, 1, 6, 1, Keep::All)))]
  #[case("+2d6", dice(0..4, 2, 6, 2, Keep::All))]
  #[case("-3d6", neg(0..4, dice(1..4, 3, 6, 3, Keep::All)))]
  // Multi-digit / leading-zero constants.
  #[case("42", constant(0..2, 42))]
  #[case("007", constant(0..3, 7))]
  // The largest values that still fit, one below the overflow cases in
  // `test_parse_overflow_error`.
  #[case("2147483647", constant(0..10, i32::MAX))]
  #[case("4294967295d6", dice(0..12, u32::MAX, 6, u32::MAX, Keep::All))]
  // Stacked prefix signs: both widen the range one character per layer, but only `-` adds
  // a node.
  #[case("++1", constant(0..3, 1))]
  #[case("--1", neg(0..3, neg(1..3, constant(2..3, 1))))]
  #[case("+-1", neg(0..3, constant(2..3, 1)))]
  #[case("-+d6", neg(0..4, dice(1..4, 1, 6, 1, Keep::All)))]
  // Two-term binary ops. The synthetic `neg` from `-` starts at the operator, not the rhs.
  #[case("1-1", add(0..3, constant(0..1, 1), neg(1..3, constant(2..3, 1))))]
  #[case("1*1", mul(0..3, constant(0..1, 1), constant(2..3, 1)))]
  #[case("2d6+3", add(0..5, dice(0..3, 2, 6, 2, Keep::All), constant(4..5, 3)))]
  #[case("3+2d6", add(0..5, constant(0..1, 3), dice(2..5, 2, 6, 2, Keep::All)))]
  #[case("2d6*3", mul(0..5, dice(0..3, 2, 6, 2, Keep::All), constant(4..5, 3)))]
  // Chained and mixed-precedence sequences.
  #[case(
    "1+2+3",
    add(0..5, add(0..3, constant(0..1, 1), constant(2..3, 2)), constant(4..5, 3))
  )]
  #[case(
    "2+3*4",
    add(0..5, constant(0..1, 2), mul(2..5, constant(2..3, 3), constant(4..5, 4)))
  )]
  #[case(
    "2*3+4",
    add(0..5, mul(0..3, constant(0..1, 2), constant(2..3, 3)), constant(4..5, 4))
  )]
  #[case(
    "2d6a4-1*3",
    add(
      0..9,
      dice(0..5, 2, 6, 4, Keep::Best),
      neg(5..9, mul(6..9, constant(6..7, 1), constant(8..9, 3))),
    )
  )]
  #[case(
    "1d6d4+2*3-1",
    add(
      0..11,
      add(
        0..9,
        dice(0..5, 1, 6, 4, Keep::Worst),
        mul(6..9, constant(6..7, 2), constant(8..9, 3)),
      ),
      neg(9..11, constant(10..11, 1)),
    )
  )]
  // Expressions with parentheses.
  #[case("(1)", constant(0..3, 1))]
  #[case("(1+1)", add(0..5, constant(1..2, 1), constant(3..4, 1)))]
  #[case("(d6)", dice(0..4, 1, 6, 1, Keep::All))]
  #[case("-(1+1)", neg(0..6, add(1..6, constant(2..3, 1), constant(4..5, 1))))]
  #[case("(1)+(2)", add(0..7, constant(0..3, 1), constant(4..7, 2)))]
  // Nested parens collapse away in the tree just the same, however deep, but each layer
  // widens the range by one paren on each side.
  #[case("((1))", constant(0..5, 1))]
  #[case("(((1)))", constant(0..7, 1))]
  #[case(
    "((1+1)*2)",
    mul(0..9, add(1..6, constant(2..3, 1), constant(4..5, 1)), constant(7..8, 2))
  )]
  // Implicit multiplication, e.g. `12(d6+1)`, behaves like an explicit `*`.
  #[case("2(3)", mul(0..4, constant(0..1, 2), constant(2..3, 3)))]
  #[case(
    "2(1+1)",
    mul(0..6, constant(0..1, 2), add(2..5, constant(2..3, 1), constant(4..5, 1)))
  )]
  #[case(
    "12(d6+1)",
    mul(
      0..8,
      constant(0..2, 12),
      add(3..7, dice(3..5, 1, 6, 1, Keep::All), constant(6..7, 1)),
    )
  )]
  #[case(
    "2(3)*4",
    mul(0..6, mul(0..4, constant(0..1, 2), constant(2..3, 3)), constant(5..6, 4))
  )]
  // Whitespace between tokens (relies on lalrpop's default whitespace skipping) shifts
  // ranges outward but doesn't otherwise change anything.
  #[case("1 + 1", add(0..5, constant(0..1, 1), constant(4..5, 1)))]
  #[case(" d6 ", dice(1..3, 1, 6, 1, Keep::All))]
  #[case(
    "2d6 + 3 * 4",
    add(0..11, dice(0..3, 2, 6, 2, Keep::All), mul(6..11, constant(6..7, 3), constant(10..11, 4)))
  )]
  fn test_parse(#[case] query: &str, #[case] expected: Expr) {
    let result = parse(query).unwrap();
    assert_eq!(result, expected);
  }

  #[rstest]
  #[case("d20", DicePool { count: 1, faces: 20, pool: 1, keep: Keep::All })]
  #[case("2d20", DicePool { count: 2, faces: 20, pool: 2, keep: Keep::All })]
  #[case("5d1", DicePool { count: 5, faces: 1, pool: 5, keep: Keep::All })]
  #[case("d20a", DicePool { count: 1, faces: 20, pool: 2, keep: Keep::Best })]
  #[case("d20d", DicePool { count: 1, faces: 20, pool: 2, keep: Keep::Worst })]
  #[case("2d20d", DicePool { count: 2, faces: 20, pool: 3, keep: Keep::Worst })]
  #[case("5d20d", DicePool { count: 5, faces: 20, pool: 6, keep: Keep::Worst })]
  #[case("d20d5", DicePool { count: 1, faces: 20, pool: 5, keep: Keep::Worst })]
  #[case("5d20d7", DicePool { count: 5, faces: 20, pool: 7, keep: Keep::Worst })]
  #[case("5d20a7", DicePool { count: 5, faces: 20, pool: 7, keep: Keep::Best })]
  #[case("5d20a", DicePool { count: 5, faces: 20, pool: 6, keep: Keep::Best })]
  fn test_square(#[case] input: &str, #[case] expected: DicePool) {
    assert_eq!(parse_dice_term(input, 0).unwrap(), expected);
  }

  #[rstest]
  #[case("7d20d5", 0..6)]
  #[case("2d6a1", 0..5)]
  #[case("4d6a3", 0..5)]
  fn test_parse_keeps_more_than_it_rolls(
    #[case] query: &str,
    #[case] expected_range: Range<usize>,
  ) {
    let err = parse(query).expect_err("expected a parse error");
    assert_eq!(err.range, expected_range);
    assert!(
      err.message.contains("from a pool of only"),
      "{}",
      err.message
    );
  }

  #[rstest]
  #[case("", 0..0)]
  #[case("+", 1..1)]
  #[case("1+", 2..2)]
  #[case("(1", 2..2)]
  #[case("1)", 1..2)]
  #[case("1 2", 2..3)]
  #[case("()", 1..2)]
  #[case("@", 0..0)]
  #[case("😎", 0..0)]
  #[case("5d", 1..1)]
  #[case("1+*2", 2..3)]
  #[case("@1@", 0..0)]
  #[case("1@2@", 1..1)]
  #[case("1+@", 2..2)]
  #[case("(1+2)*@", 6..6)]
  #[case("(1+@)+2", 3..3)]
  #[case("5d+6d", 1..1)]
  #[case("1+2*3 4", 6..7)]
  fn test_parse_error(#[case] query: &str, #[case] expected_range: Range<usize>) {
    let err = parse(query).expect_err("expected a parse error");
    assert_eq!(err.range, expected_range);
  }

  #[rstest]
  #[case("9999999999", 0..10, "constants must be at most")]
  #[case("2147483648", 0..10, "constants must be at most")]
  #[case("1+9999999999", 2..12, "constants must be at most")]
  #[case("(2*9999999999)", 3..13, "constants must be at most")]
  #[case("9999999999(2)", 0..10, "constants must be at most")]
  #[case("9999999999d6", 0..10, "dice count")]
  #[case("d99999999999", 1..12, "face count")]
  #[case("2d6a99999999999", 4..15, "dice pool")]
  #[case("4294967295d6d", 0..13, "dice pool of")]
  fn test_parse_overflow_error(
    #[case] query: &str,
    #[case] expected_range: Range<usize>,
    #[case] expected_message: &str,
  ) {
    let err = parse(query).expect_err("expected a parse error");
    assert_eq!(err.range, expected_range);
    assert!(
      err.message.contains(expected_message),
      "message `{}` should contain `{expected_message}`",
      err.message
    );
  }
}

use std::ops::Range;

use dice_core::parser::{self, Expr};

/// Parses an expression, rendering any failure in a form fit to print.
pub fn parse(input: &str) -> Result<Expr, String> {
  parser::parse(input).map_err(|err| describe(input, &err.message, &err.range))
}

/// Renders a failure with the offending span of the input underlined, e.g.
///
/// ```text
/// error: unexpected `+`, expected one of: ...
///   2d6 ++ 4
///       ^
/// ```
pub fn describe(input: &str, message: &str, range: &Range<usize>) -> String {
  // The error carries byte offsets, but the caret has to line up with printed columns.
  let chars_before = |offset: usize| {
    input
      .get(..offset)
      .map_or(offset, |prefix| prefix.chars().count())
  };
  let start = chars_before(range.start);
  let width = chars_before(range.end).saturating_sub(start).max(1);

  format!(
    "error: {message}\n  {input}\n  {}{}",
    " ".repeat(start),
    "^".repeat(width)
  )
}

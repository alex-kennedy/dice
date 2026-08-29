use dice_core::calculate_expression;
use wasm_bindgen::prelude::*;

mod parse;

pub use parse::{ExpressionKind, Range, parse};

#[wasm_bindgen(start)]
pub fn on_start() {
  console_error_panic_hook::set_once();
}

/// A probability distribution of the rolling outcomes.
#[wasm_bindgen]
pub struct Distribution(dice_core::Distribution);

#[wasm_bindgen]
impl Distribution {
  /// The minimum possible dice roll, which maps to the the first element of the pmf array.
  #[wasm_bindgen(getter)]
  pub fn minimum(&self) -> i32 {
    self.0.min()
  }

  /// The maximum possible dice roll.
  #[wasm_bindgen(getter)]
  pub fn maximum(&self) -> i32 {
    self.0.max()
  }

  /// The outcome with the highest probability.
  #[wasm_bindgen(getter)]
  pub fn mode(&self) -> i32 {
    self.0.mode()
  }

  /// Probability mass function, starting from minimum.
  #[wasm_bindgen(getter)]
  pub fn pmf(&self) -> Vec<f64> {
    self.0.pmf().clone()
  }
}

/// Parses and calculates the probability distribution for a dice rolling expression.
#[wasm_bindgen]
pub fn calculate_distribution(query: String) -> Result<Distribution, String> {
  let expr = dice_core::parser::parse(&query).map_err(|e| e.message)?;
  let dist = calculate_expression(&expr).map_err(|e| e.message)?;
  Ok(Distribution(dist))
}

#[cfg(test)]
mod tests {
  use super::*;
  use rstest::rstest;

  #[rstest]
  #[case("3", 3, vec![1.0])]
  #[case("d6", 1, vec![1.0 / 6.0; 6])]
  #[case("1+2", 3, vec![1.0])]
  fn test_calculate_distribution(#[case] query: &str, #[case] minimum: i32, #[case] pmf: Vec<f64>) {
    let dist = calculate_distribution(query.to_string()).unwrap();
    assert_eq!(dist.minimum(), minimum);
    assert_eq!(dist.pmf(), pmf);
  }

  #[test]
  fn test_calculate_distribution_sums_dice_pool() {
    let dist = calculate_distribution("2d6".to_string()).unwrap();
    assert_eq!(dist.minimum(), 2);
    assert_eq!(dist.maximum(), 12);
    assert_eq!(dist.mode(), 7);
    assert_eq!(dist.pmf().len(), 11);
  }

  #[test]
  fn test_calculate_distribution_parse_error() {
    assert!(calculate_distribution("+".to_string()).is_err());
  }
}

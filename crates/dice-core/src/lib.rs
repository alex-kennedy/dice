pub mod binomial;
pub mod dice;
pub mod parser;

mod simulate;
mod sum_distributions;

pub use simulate::simulate_expression;
pub use sum_distributions::sum_distributions;

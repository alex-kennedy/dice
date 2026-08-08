use rand::RngExt;

use crate::{Distribution, sum_distributions};

/// A dice term means roll `pool` dice of `faces` sides and sum the `count` of them selected by
/// `keep`.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct DicePool {
  pub count: u32,
  pub faces: u32,
  pub pool: u32,
  pub keep: Keep,
}

/// Which dice from a rolled pool contribute to the sum.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Keep {
  /// The whole pool, i.e. pool == count.
  All,
  /// The highest count of the pool, written a for "advantage".
  Best,
  /// The lowest count of the pool, written disadvantage.
  Worst,
}

impl DicePool {
  pub fn new(count: u32, faces: u32, pool: u32, keep: Keep) -> Self {
    DicePool {
      count,
      faces,
      pool,
      keep,
    }
  }

  /// Rolls `pool` dice of `faces` sides, and returns the sum of the `count` best or worst (as
  /// determined by `keep`).
  pub fn roll<T: rand::Rng>(&self, rng: &mut T) -> u32 {
    if self.faces == 0 {
      return 0;
    }
    let mut pool: Vec<u32> = (0..self.pool)
      .map(|_| rng.random_range(1..self.faces + 1))
      .collect();
    pool.sort();
    match self.keep {
      Keep::All => pool.iter().sum(),
      Keep::Best => pool[(self.pool - self.count) as usize..].iter().sum(),
      Keep::Worst => pool[0..(self.count as usize)].iter().sum(),
    }
  }

  /// Returns the minimum possible value this dice pool can roll.
  pub fn min(&self) -> u32 {
    self.count
  }

  /// Returns the maximum possible value this dice pool can roll.
  pub fn max(&self) -> u32 {
    self.count * self.faces
  }

  // Computes and returns a probability distribution of the possible values from this dice pool.
  pub fn distribution(&self) -> Distribution {
    if self.count == 0 || self.faces == 0 {
      return Distribution::new_constant(0);
    }
    if self.count == 1 && self.pool == 1 {
      return Distribution::new_uniform(self.faces);
    }
    if self.count == self.pool {
      let d = Distribution::new_uniform(self.faces);
      return sum_distributions(vec![&d], Some(self.count as i32));
    }
    crate::keep::calculate_keep_distribution(&self)
  }
}

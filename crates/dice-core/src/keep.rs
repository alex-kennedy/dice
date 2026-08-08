//! Implements the algorithm described in "Icepool: Efficient Computation of Dice Pool
//! Probabilities" for best/worst sums of a pool of dice.

use std::collections::HashMap;

use num::{BigUint, One, ToPrimitive, Zero};

use crate::{
  Distribution,
  binomial::MemoizedBinomialCoefficient,
  dice::{DicePool, Keep},
};

// Key type used to cache results of solve calls.
type CacheKey = (usize, usize, usize);

// Calculates a unique key for the state of a single run of the algorithm. The state of the die and
// count list uniquely identify the inputs, because the others are constant.
fn cache_key(die_state: usize, count_list: &CountList) -> CacheKey {
  (die_state, count_list.len, count_list.sum())
}

/// Calculates a distribution for rolling and summing a pool of dice where only an amount of the
/// best or worst rolls are kept. The classic example is rolling a d20 with advantage.
pub fn calculate_keep_distribution(dice: &DicePool) -> Distribution {
  // Builds the count list, which is basically a bitmask over the pool of dice throws assuming
  // they've been ordered appropriately. We always form the count_list in the same order, and flip
  // the order in which we iterate the dice faces to suit best/worst ordering. This allows an
  // optimization where we stop processing once the whole count pool is zero.
  let count_list = CountList {
    len: dice.pool as usize,
    idx: (dice.pool - dice.count) as usize,
  };

  let range = match dice.keep {
    Keep::Best => Range::new(dice.faces as usize, 1),
    Keep::Worst | Keep::All => Range::new(1, dice.faces as usize),
  };

  let mut b = MemoizedBinomialCoefficient::new();
  let mut c: HashMap<CacheKey, HashMap<usize, BigUint>> = HashMap::new();
  let results = solve(range, &count_list, 0, &mut b, &mut c);

  let (min, max) = results
    .keys()
    .into_iter()
    .fold((usize::MAX, usize::MIN), |a, &b| (a.0.min(b), a.1.max(b)));
  let mut pmf = vec![0.0; (max - min) + 1];
  let sum: f64 = results
    .values()
    .into_iter()
    .sum::<BigUint>()
    .to_f64()
    .unwrap();
  for x in min..=max {
    pmf[x - min] = results
      .get(&x)
      .unwrap_or(&BigUint::zero())
      .to_f64()
      .unwrap()
      / sum;
  }
  return Distribution::new(pmf, min as i32);
}

/// State transition function for addition.
fn next_state(state: Option<usize>, outcome: usize, count: usize) -> usize {
  match state {
    Some(state) => state + outcome * count,
    None => outcome * count,
  }
}

/// Solves the sub problems for rolling a pool of dice and summing according to a count list.
fn solve<'a>(
  mut die: Range,
  count_list: &CountList,
  depth: u64,
  b: &mut MemoizedBinomialCoefficient,
  c: &'a mut HashMap<CacheKey, HashMap<usize, BigUint>>,
) -> &'a HashMap<usize, BigUint> {
  let key = cache_key(die.state(), count_list);

  // All branches here set the value in the cache for these inputs, then we return a reference from
  // the cache.
  if !c.contains_key(&key) {
    let n = count_list.len;
    let outcome = die.next();

    if die.is_empty() {
      let state = next_state(None, outcome, count_list.sum());
      let mut m = HashMap::new();
      m.insert(state, BigUint::one());
      c.insert(key, m);
    } else {
      let mut m = HashMap::new();
      for k in 0..=n {
        let (new_count_list, count) = count_list.cut_at((n - k) as usize);
        let tail = solve(die.clone(), &new_count_list, depth + 1, b, c);
        for (&state, weight) in tail.iter() {
          let state = next_state(Some(state), outcome, count);
          let weight = weight * b.comb(n as usize, k as usize);
          m.entry(state)
            .and_modify(|v| *v += &weight)
            .or_insert(weight);
        }
      }
      c.insert(key, m);
    }
  }
  return &c[&key];
}

/// Range represents the faces of a die, and are iterated through either up or down. Only `state`
/// differs between function calls, so it's the only thing that makes it into the cache key.
#[derive(Clone)]
struct Range {
  state: usize,
  stop: usize,
  step: Step,
}

impl Range {
  /// Creates a new range going from start to stop in increments of +1 or -1, whichever order is
  /// right.
  fn new(start: usize, stop: usize) -> Self {
    return Self {
      state: start,
      stop: stop,
      step: if start < stop { Step::Up } else { Step::Down },
    };
  }

  /// Returns the state of the range.
  fn state(&self) -> usize {
    self.state
  }

  /// Returns the next value in the range. Note this shouldn't be made public because it is unsafe -
  /// it doesn't check that there indeed _is_ a next value, and would just keep growing.
  fn next(&mut self) -> usize {
    let result = self.state;
    match self.step {
      Step::Up => self.state += 1,
      Step::Down => self.state -= 1,
    };
    result
  }

  /// Returns true if there are no values left to return in the range. `is_empty` being true will
  /// not prevent next() from returning an invalid value.
  fn is_empty(&self) -> bool {
    match self.step {
      Step::Up => self.state > self.stop,
      Step::Down => self.state < self.stop,
    }
  }
}

/// Range step direction.
#[derive(Clone)]
enum Step {
  /// Goes from low to high, in increments of +1.
  Up,

  /// Goes from high to low, in increments of -1.
  Down,
}

/// A special form of count list that is a number of zeros followed by a number of ones. The length
/// is `len`, and the point the zeros switch to ones is `idx`. E.g. 0001111 would be represented by
/// `CountList {len: 7, idx: 3}`.
struct CountList {
  /// Total length of the count list.
  len: usize,

  /// Index at which zeros switch to ones.
  idx: usize,
}

impl CountList {
  /// Sum, or the total number of ones in the list.
  fn sum(&self) -> usize {
    self.len - self.idx
  }

  /// Cuts the count list at the given index, returning the new count list, and the sum of the
  /// elements chopped off the end.
  fn cut_at(&self, i: usize) -> (Self, usize) {
    (
      Self {
        len: i,
        idx: self.idx.min(i),
      },
      (self.len - self.idx) - (i - self.idx.min(i)),
    )
  }
}

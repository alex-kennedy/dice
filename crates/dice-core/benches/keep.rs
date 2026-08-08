//! Benchmarks for the best/worst keep algorithm.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use dice_core::dice::{DicePool, Keep};

/// Realistic dice pools for keep.rs.
fn keep_dice_expressions(c: &mut Criterion) {
  let cases = [
    ("d20a", DicePool::new(1, 20, 2, Keep::Best)),
    ("d20d", DicePool::new(1, 20, 2, Keep::Worst)),
    ("3d6a4", DicePool::new(3, 6, 4, Keep::Best)),
    ("5d10a8", DicePool::new(5, 10, 8, Keep::Best)),
    ("10d10a20", DicePool::new(10, 10, 20, Keep::Best)),
    ("10d20a20", DicePool::new(10, 20, 20, Keep::Best)),
    ("d100a", DicePool::new(1, 100, 2, Keep::Best)),
  ];

  let mut group = c.benchmark_group("keep");
  for (name, pool) in cases {
    group.bench_function(name, |b| b.iter(|| black_box(&pool).distribution()));
  }
  group.finish();
}

criterion_group!(benches, keep_dice_expressions,);

criterion_main!(benches);

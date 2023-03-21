use std::collections::{HashMap, HashSet};

use criterion::{
    criterion_group, criterion_main,
    measurement::{Measurement, ValueFormatter},
    BenchmarkId, Criterion,
};
use rand::{thread_rng, Rng};
use succinct_neo::rolling_hash::{CyclicPolynomial, HashedByteSet, HashedBytes, RabinKarp, RollingHash};

const STRING_SIZE: usize = 500_000_000;
const WINDOW_SIZE: usize = 32;

struct HashCollisions;

impl Measurement for HashCollisions {
    type Intermediate = usize;

    type Value = usize;

    fn start(&self) -> Self::Intermediate {
        0
    }

    fn end(&self, i: Self::Intermediate) -> Self::Value {
        i
    }

    fn add(&self, v1: &Self::Value, v2: &Self::Value) -> Self::Value {
        v1 + v2
    }

    fn zero(&self) -> Self::Value {
        0
    }

    fn to_f64(&self, value: &Self::Value) -> f64 {
        *value as f64
    }

    fn formatter(&self) -> &dyn criterion::measurement::ValueFormatter {
        &HashCollisionFormatter
    }
}

struct HashCollisionFormatter;
impl ValueFormatter for HashCollisionFormatter {
    fn scale_values(&self, _typical_value: f64, _values: &mut [f64]) -> &'static str {
        "collisions"
    }

    fn scale_throughputs(
        &self,
        _typical_value: f64,
        _throughput: &criterion::Throughput,
        _values: &mut [f64],
    ) -> &'static str {
        "b/collision"
    }

    fn scale_for_machines(&self, _values: &mut [f64]) -> &'static str {
        "collisions"
    }
}

fn setup_string() -> String {
    let mut s = String::with_capacity(STRING_SIZE);
    let mut rng = thread_rng();

    for _ in 0..STRING_SIZE {
        s.push(rng.gen());
    }

    s
}

fn bench_rolling_hash(c: &mut Criterion) {
    let s = setup_string();

    let mut group = c.benchmark_group("rolling_hash");
    group.sample_size(250);

    group.bench_function(BenchmarkId::new("insert", "default_hash"), |b| {
        b.iter_batched_ref(
            || (HashSet::<&[u8]>::new(), 0..STRING_SIZE - WINDOW_SIZE),
            |(map, iter)| {
                let i = iter.next().unwrap();
                map.insert(&s.as_bytes()[i..i + WINDOW_SIZE]);
            },
            criterion::BatchSize::NumIterations((STRING_SIZE - WINDOW_SIZE) as u64),
        )
    });

    group.bench_function(BenchmarkId::new("insert", "rk_hash"), |b| {
        b.iter_batched_ref(
            || {
                (
                    HashedByteSet::default(),
                    RabinKarp::new(&s, WINDOW_SIZE),
                )
            },
            |(map, rk)| {
                map.insert(rk.hashed_bytes());
                rk.advance();
            },
            criterion::BatchSize::NumIterations((STRING_SIZE - WINDOW_SIZE) as u64),
        )
    });
    let cc = CyclicPolynomial::new(&s, WINDOW_SIZE);
    let seed = cc.seed();
    let char_table = *cc.char_table();
    group.bench_function(BenchmarkId::new("insert", "cc_hash"), |b| {
        b.iter_batched_ref(
            || {
                (
                    HashedByteSet::default(),
                    CyclicPolynomial::with_table(&s, WINDOW_SIZE, seed, &char_table),
                )
            },
            |(map, cc)| {
                map.insert(cc.hashed_bytes());
                cc.advance();
            },
            criterion::BatchSize::NumIterations((STRING_SIZE - WINDOW_SIZE) as u64),
        )
    });
}

fn bench_rolling_hash_collisions(c: &mut Criterion<HashCollisions>) {
    let s = setup_string();

    let mut group = c.benchmark_group("rolling_hash");

    group.bench_function(BenchmarkId::new("collisions", "cc_hash"), |b| {
        b.iter_batched_ref(
            HashSet::<HashedBytes<'_>>::new,
            |map| {
                let cc = CyclicPolynomial::new(&s, WINDOW_SIZE);
                for hash in cc {
                    map.insert(hash);
                }
            },
            criterion::BatchSize::LargeInput,
        )
    });

    let mut cc = CyclicPolynomial::new(&s, WINDOW_SIZE);
    let a = cc.next();
    println!("{a:?}");
    group.bench_function(BenchmarkId::new("collisions", "cc_hash2"), |b| {
        b.iter_custom(|iters| {
            let mut collisions = 0;
            for _ in 0..iters {
                let mut map = HashMap::<HashedBytes<'_>, HashedBytes<'_>>::new();
                let cc = CyclicPolynomial::new(&s, WINDOW_SIZE);
                for hash in cc {
                    if let Some(found) = map.get(&hash) {
                        if found.bytes() != hash.bytes() {
                            collisions += 1;
                        }
                    }
                    map.insert(hash, hash);
                }
            }

            collisions
        })
    });
}

fn hash_collision_measurement() -> Criterion<HashCollisions> {
    Criterion::default().with_measurement(HashCollisions)
}

criterion_group!(rolling_hash_benches, bench_rolling_hash,);

criterion_group! {
    name = rolling_hash_collision_benches;
    config = hash_collision_measurement();
    targets = bench_rolling_hash_collisions
}

criterion_main!(rolling_hash_benches);

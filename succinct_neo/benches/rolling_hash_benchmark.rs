use std::collections::HashSet;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{thread_rng, Rng};
use succinct_neo::{
    bit_vec::BitVec,
    util::rolling_hash::{HashedBytes, RabinKarp},
};

const STRING_SIZE: usize = 50_000_000;
const WINDOW_SIZE: usize = 32;

fn setup_string() -> String {
    let mut s = String::with_capacity(STRING_SIZE);
    let mut rng = thread_rng();

    for i in 0..STRING_SIZE {
        s.push(rng.gen());
    }

    s
}

fn bench_rolling_hash(c: &mut Criterion) {
    let s = setup_string();

    let mut group = c.benchmark_group("rabin_karp");
    group.sample_size(250);

    let mut map = HashSet::<&[u8]>::new();
    let mut i = 0;
    group.bench_function(BenchmarkId::new("insert", "default_hash"), |b| {
        b.iter(|| {
            map.insert(&s.as_bytes()[i..i + WINDOW_SIZE]);
            i = (i + 1) % (STRING_SIZE - WINDOW_SIZE - 1);
        })
    });

    let mut map = HashSet::<HashedBytes<'_>>::new();
    let mut rk = RabinKarp::new(&s, WINDOW_SIZE, 7919);
    group.bench_function(BenchmarkId::new("insert", "rk_hash"), |b| {
        b.iter(|| {
            map.insert(rk.hashed_bytes());
            match rk.next() {
                Some(_) => {}
                None => {
                    rk = RabinKarp::new(&s, WINDOW_SIZE, 7919);
                }
            };
        })
    });
}

criterion_group!(bv_benches, bench_rolling_hash);
criterion_main!(bv_benches);

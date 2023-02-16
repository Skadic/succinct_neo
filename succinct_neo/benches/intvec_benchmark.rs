use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{thread_rng, Rng};
use succinct_neo::int_vec::{IntAccess, IntVec};

#[allow(non_upper_case_globals)]
const KiB: usize = 1024;
#[allow(non_upper_case_globals)]
const MiB: usize = 1024 * KiB;
const IV_BITS: usize = 50 * MiB;
const IV_WIDTH: usize = 17;
const IV_ELEMS: usize = IV_BITS / IV_WIDTH;
const IV_MAX_INT: usize = (1 << IV_WIDTH) - 1;

fn setup_iv() -> IntVec {
    let mut iv = IntVec::with_capacity(17, IV_ELEMS);
    let mut rng = rand::thread_rng();

    for _ in 0..IV_ELEMS {
        iv.push(rng.gen_range(0..=IV_MAX_INT));
    }

    iv
}

fn bench_iv_ops(c: &mut Criterion) {
    let mut iv = setup_iv();
    let mut rng = thread_rng();
    let n = iv.len();

    let mut group = c.benchmark_group("iv_ops");
    group.sample_size(250);

    group.bench_function("get", |b| {
        b.iter_batched(
            || rng.gen_range(0..n),
            |i| {
                iv.get(black_box(i));
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("set", |b| {
        b.iter_batched(
            || (rng.gen_range(0..n), rng.gen_range(0..=IV_MAX_INT)),
            |(i, v)| {
                iv.set(black_box(i), v);
            },
            criterion::BatchSize::SmallInput,
        )
    });

    let mut iv = IntVec::with_capacity(IV_WIDTH,1);
    group.bench_function("push_no_reserve", |b| {
        b.iter(
            || {
                iv.push(black_box(0));
            },
        )
    });

    let mut iv = IntVec::with_capacity(IV_WIDTH,2_000_000_000);
    group.bench_function("push_with_reserve", |b| {
        b.iter(
            || {
                iv.push(black_box(0));
            },
        )
    });
}

criterion_group!(iv_benches, bench_iv_ops);
criterion_main!(iv_benches);

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rand::{thread_rng, Rng};
use succinct_neo::int_vec::{DynamicIntVec, FixedIntVec, IntVector};

#[allow(non_upper_case_globals)]
const KiB: usize = 1024;
#[allow(non_upper_case_globals)]
const MiB: usize = 1024 * KiB;
const IV_BITS: usize = 50 * MiB;
const IV_WIDTH: usize = 17;
const IV_ELEMS: usize = IV_BITS / IV_WIDTH;
const IV_MAX_INT: usize = (1 << IV_WIDTH) - 1;

fn setup_iv_dyn(w: usize) -> DynamicIntVec {
    let mut iv = DynamicIntVec::with_capacity(w, IV_ELEMS);
    let mut rng = rand::thread_rng();
    let max_int = (1 << w) - 1;
    for _ in 0..IV_ELEMS {
        iv.push(rng.gen_range(0..=max_int));
    }

    iv
}

fn setup_iv_fix<const WIDTH: usize>() -> FixedIntVec<WIDTH> {
    let mut iv = FixedIntVec::<WIDTH>::with_capacity(IV_ELEMS);
    let mut rng = rand::thread_rng();
    let max: usize = (1 << WIDTH) - 1;

    for _ in 0..IV_ELEMS {
        iv.push(rng.gen_range(0..=max));
    }

    iv
}

fn bench_iv_ops(c: &mut Criterion) {
    let mut ivd = setup_iv_dyn(17);
    let mut ivf = setup_iv_fix::<17>();
    let mut rng = thread_rng();
    let n = ivd.len();

    let mut group = c.benchmark_group("iv_ops");
    group.sample_size(250);

    group.bench_function(BenchmarkId::new("get", "dyn"), |b| {
        b.iter_batched(
            || rng.gen_range(0..n),
            |i| {
                ivd.get(black_box(i));
            },
            criterion::BatchSize::SmallInput,
        )
    });
    group.bench_function(BenchmarkId::new("get", "fixed"), |b| {
        b.iter_batched(
            || rng.gen_range(0..n),
            |i| {
                ivf.get(black_box(i));
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function(BenchmarkId::new("set", "dyn"), |b| {
        b.iter_batched(
            || (rng.gen_range(0..n), rng.gen_range(0..=IV_MAX_INT)),
            |(i, v)| {
                ivd.set(black_box(i), v);
            },
            criterion::BatchSize::SmallInput,
        )
    });
    group.bench_function(BenchmarkId::new("set", "fixed"), |b| {
        b.iter_batched(
            || (rng.gen_range(0..n), rng.gen_range(0..=IV_MAX_INT)),
            |(i, v)| {
                ivf.set(black_box(i), v);
            },
            criterion::BatchSize::SmallInput,
        )
    });

    let mut iv = DynamicIntVec::with_capacity(IV_WIDTH, 1);
    group.bench_function(BenchmarkId::new("push_no_reserve", "dyn"), |b| {
        b.iter(|| {
            iv.push(black_box(0));
        })
    });
    let mut iv = FixedIntVec::<IV_WIDTH>::with_capacity(1);
    group.bench_function(BenchmarkId::new("push_no_reserve", "fixed"), |b| {
        b.iter(|| {
            iv.push(black_box(0));
        })
    });

    let mut iv = DynamicIntVec::with_capacity(IV_WIDTH, 2_000_000_000);
    group.bench_function(BenchmarkId::new("push_with_reserve", "dyn"), |b| {
        b.iter(|| {
            iv.push(black_box(0));
        })
    });
    let mut iv = FixedIntVec::<IV_WIDTH>::with_capacity(2_000_000_000);
    group.bench_function(BenchmarkId::new("push_with_reserve", "fixed"), |b| {
        b.iter(|| {
            iv.push(black_box(0));
        })
    });
}

fn bench_iv_16_ops(c: &mut Criterion) {
    let mut ivd = setup_iv_dyn(16);
    let mut ivf = setup_iv_fix::<16>();
    let mut rng = thread_rng();
    let n = ivd.len();

    let mut group = c.benchmark_group("iv_16_ops");
    group.sample_size(250);

    group.bench_function(BenchmarkId::new("get", "dyn"), |b| {
        b.iter_batched(
            || rng.gen_range(0..n),
            |i| {
                ivd.get(black_box(i));
            },
            criterion::BatchSize::SmallInput,
        )
    });
    group.bench_function(BenchmarkId::new("get", "fixed"), |b| {
        b.iter_batched(
            || rng.gen_range(0..n),
            |i| {
                ivf.get(black_box(i));
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function(BenchmarkId::new("set", "dyn"), |b| {
        b.iter_batched(
            || (rng.gen_range(0..n), rng.gen_range(0..=u16::MAX as usize)),
            |(i, v)| {
                ivd.set(black_box(i), v);
            },
            criterion::BatchSize::SmallInput,
        )
    });
    group.bench_function(BenchmarkId::new("set", "fixed"), |b| {
        b.iter_batched(
            || (rng.gen_range(0..n), rng.gen_range(0..=u16::MAX as usize)),
            |(i, v)| {
                ivf.set(black_box(i), v);
            },
            criterion::BatchSize::SmallInput,
        )
    });

    let mut iv = DynamicIntVec::with_capacity(16, 1024);
    group.bench_function(BenchmarkId::new("push", "dyn"), |b| {
        b.iter(|| {
            iv.push(black_box(0));
        })
    });
    let mut iv = FixedIntVec::<16>::with_capacity(1024);
    group.bench_function(BenchmarkId::new("push", "fixed"), |b| {
        b.iter(|| {
            iv.push(black_box(0));
        })
    });
}

criterion_group!(iv_benches, bench_iv_ops, bench_iv_16_ops);
criterion_main!(iv_benches);

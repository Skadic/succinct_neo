use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use rand::{thread_rng, Rng};

use succinct_neo::{
    bit_vec::BitVec,
    bit_vec::rank_select::{
        flat_popcount::{BinarySearch, LinearSearch},
        FlatPopcount, BitRankSupport, BitSelectSupport,
    },
};

#[allow(non_upper_case_globals)]
const KiB: usize = 1024;
#[allow(non_upper_case_globals)]
const MiB: usize = 1024 * KiB;
#[allow(non_upper_case_globals, unused)]
const GiB: usize = 1024 * MiB;
#[allow(clippy::identity_op)]
const BV_SIZE_BYTES: usize = 20 * MiB;

fn setup_bv() -> BitVec {
    let mut bv = BitVec::new(BV_SIZE_BYTES * 8);

    for i in 0..bv.len() {
        bv.set(i, (i / 2) % 2 == 0);
    }

    bv
}

fn bench_construction(c: &mut Criterion) {
    let bv = setup_bv();
    let mut group = c.benchmark_group("flat_popcount");
    group.sample_size(50);
    group.throughput(criterion::Throughput::Bytes(BV_SIZE_BYTES as u64));

    group.bench_function("bench_construction", |b| {
        b.iter_with_large_drop(|| FlatPopcount::<()>::new(black_box(&bv)))
    });

    group.finish();
}

fn bench_rank(c: &mut Criterion) {
    let bv = setup_bv();
    let mut rng = thread_rng();
    let n = bv.len();

    let mut group = c.benchmark_group("flat_popcount");

    let rs_linear = FlatPopcount::<LinearSearch>::new(&bv);
    group.bench_function("rank_0", |b| {
        b.iter_batched(
            || rng.gen_range(0..n),
            |i| rs_linear.rank::<false>(i),
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("rank_1", |b| {
        b.iter_batched(
            || rng.gen_range(0..n),
            |i| rs_linear.rank::<true>(i),
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("rank_dyn_0", |b| {
        b.iter_batched(
            || rng.gen_range(0..n),
            |i| rs_linear.rank_dyn(i, false),
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("rank_dyn_1", |b| {
        b.iter_batched(
            || rng.gen_range(0..n),
            |i| rs_linear.rank_dyn(i, false),
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_select(c: &mut Criterion) {
    let bv = setup_bv();
    let mut rng = thread_rng();

    let mut group = c.benchmark_group("flat_popcount");

    let rs_linear = FlatPopcount::<LinearSearch>::new(&bv);
    let num_ones = rs_linear.num_ones();
    group.bench_function("select_1_linear", |b| {
        b.iter_batched(
            || rng.gen_range(0..num_ones),
            |i| {
                rs_linear.select(i);
            },
            criterion::BatchSize::SmallInput,
        )
    });

    let rs_binary = FlatPopcount::<BinarySearch>::new(&bv);
    group.bench_function("select_1_binary", |b| {
        b.iter_batched(
            || rng.gen_range(0..num_ones),
            |i| {
                rs_binary.select(i);
            },
            criterion::BatchSize::SmallInput,
        )
    });

    #[cfg(all(
        target_arch = "x86_64",
        target_feature = "sse2",
        target_feature = "ssse3",
        target_feature = "sse4.1"
    ))]
    {
        use succinct_neo::bit_vec::rank_select::flat_popcount::SimdSearch;
        let rs_simd = FlatPopcount::<SimdSearch>::new(&bv);
        group.bench_function("select_1_simd", |b| {
            b.iter_batched(
                || rng.gen_range(0..num_ones),
                |i| {
                    rs_simd.select(i);
                },
                criterion::BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

criterion_group!(
    name = flat_popcout_benches;
    config = Criterion::default();//.with_measurement(Perf::new(Builder::from_hardware_event(Hardware::Instructions)));
    targets = bench_construction, bench_rank, bench_select,
);
criterion_main!(flat_popcout_benches);

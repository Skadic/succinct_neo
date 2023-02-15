use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{thread_rng, Rng};
use succinct_neo::bit_vec::BitVec;

#[allow(non_upper_case_globals)]
const KiB: usize = 1024;
#[allow(non_upper_case_globals)]
const MiB: usize = 1024 * KiB;
const BV_SIZE: usize = 50 * MiB;

fn setup_bv() -> BitVec {
    let mut bv = BitVec::new(BV_SIZE);

    for i in 0..bv.len() {
        bv.set(i, (i / 2) % 2 == 0);
    }

    bv
}

fn bench_bv_ops(c: &mut Criterion) {
    let mut bv = setup_bv();
    let mut rng = thread_rng();
    let n = bv.len();

    let mut group = c.benchmark_group("bv_ops");
    group.sample_size(250);

    group.bench_function("bv_get", |b| {
        b.iter_batched(
            || rng.gen_range(0..n),
            |i| {
                bv.get(black_box(i));
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("bv_set", |b| {
        b.iter_batched(
            || (rng.gen_range(0..n), rng.gen_bool(0.5)),
            |(i, v)| {
                bv.set(black_box(i), v);
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(bv_benches, bench_bv_ops);
criterion_main!(bv_benches);

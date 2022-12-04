use bit_set::BitSet;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use tree_bitset::TreeBitSet;

pub fn criterion_benchmark(c: &mut Criterion) {
    let n_bits: u64 = 10000;
    let n_bits_usize: usize = n_bits as usize;

    c.bench_with_input(
        BenchmarkId::new("tree-bitset-set-bits", n_bits),
        &n_bits,
        |b, n_bits| {
            let mut tree = TreeBitSet::<u64>::new();
            b.iter(|| {
                for bit in 0..*n_bits {
                    tree.insert(black_box(bit));
                }
            });
        },
    );

    c.bench_with_input(
        BenchmarkId::new("bitset-set-bits", n_bits_usize),
        &n_bits_usize,
        |b, n_bits| {
            let mut bitset = BitSet::new();
            b.iter(|| {
                for bit in 0..*n_bits {
                    bitset.insert(black_box(bit));
                }
            });
        },
    );

    c.bench_with_input(
        BenchmarkId::new("tree-bitset-set-bits-sparse", n_bits),
        &n_bits,
        |b, n_bits| {
            let mut tree = TreeBitSet::<u64>::new();
            b.iter(|| {
                for bit in 0..*n_bits {
                    tree.insert(black_box(bit * 193));
                }
            });
        },
    );

    c.bench_with_input(
        BenchmarkId::new("bitset-set-bits-sparse", n_bits_usize),
        &n_bits_usize,
        |b, n_bits| {
            let mut bitset = BitSet::new();
            b.iter(|| {
                for bit in 0..*n_bits {
                    bitset.insert(black_box(bit * 193));
                }
            });
        },
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

use crate::{
    iter::{next_bucket, Bucket, BucketIterator, RawRefIter},
    op::CombineOp,
    Bits,
};

#[derive(Debug, Eq, PartialEq)]
pub enum Combination<T: Bits> {
    PQ(T, T),
    P(T),
    Q(T),
}

#[derive(Debug, Eq, PartialEq)]
pub struct BucketCombination<T: Bits> {
    base: T,
    combination: Combination<T>,
}

impl<T: Bits> BucketCombination<T> {
    pub fn new(base: T, combination: Combination<T>) -> Self {
        Self { base, combination }
    }

    pub fn combine<O: CombineOp<T>>(self) -> Bucket<T> {
        Bucket {
            base: self.base,
            bits: O::combine(self.combination),
        }
    }
}

pub struct BucketCombinationIterator<'a, 'b, T: Bits> {
    p: RawRefIter<'a, T>,
    q: RawRefIter<'b, T>,
    bucket_p: Option<Bucket<T>>,
    bucket_q: Option<Bucket<T>>,
}

impl<'a, 'b, T: Bits> BucketCombinationIterator<'a, 'b, T> {
    pub fn new(mut p: RawRefIter<'a, T>, mut q: RawRefIter<'b, T>) -> Self {
        let bucket_p = next_bucket(&mut p);
        let bucket_q = next_bucket(&mut q);

        Self {
            p,
            q,
            bucket_p,
            bucket_q,
        }
    }
}

impl<'a, 'b, T: Bits> Iterator for BucketCombinationIterator<'a, 'b, T> {
    type Item = BucketCombination<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match (&mut self.bucket_p, &mut self.bucket_q) {
            (Some(p), Some(q)) if p.base == q.base => {
                let item = BucketCombination::new(p.base, Combination::PQ(p.bits, q.bits));

                self.bucket_p = next_bucket(&mut self.p);
                self.bucket_q = next_bucket(&mut self.q);

                Some(item)
            }
            (Some(p), Some(q)) if p.base < q.base => {
                let item = BucketCombination::new(p.base, Combination::P(p.bits));

                self.bucket_p = next_bucket(&mut self.p);
                Some(item)
            }
            (Some(p), None) => {
                let item = BucketCombination::new(p.base, Combination::P(p.bits));

                self.bucket_p = next_bucket(&mut self.p);
                Some(item)
            }
            (_, Some(q)) => {
                let item = BucketCombination::new(q.base, Combination::Q(q.bits));

                self.bucket_q = next_bucket(&mut self.q);
                Some(item)
            }
            (None, None) => None,
        }
    }
}

pub struct BitCombinationIterator<'a, 'b, T: Bits, O: CombineOp<T>> {
    bucket_combination_iter: BucketCombinationIterator<'a, 'b, T>,
    bucket_iter: Option<BucketIterator<T>>,
    _op: O,
}

impl<'a, 'b, T: Bits, O: CombineOp<T>> BitCombinationIterator<'a, 'b, T, O> {
    pub fn new(mut bucket_combination_iter: BucketCombinationIterator<'a, 'b, T>, op: O) -> Self {
        let bucket_iter = bucket_combination_iter
            .next()
            .map(BucketCombination::combine::<O>)
            .map(BucketIterator::new);

        Self {
            bucket_combination_iter,
            bucket_iter,
            _op: op,
        }
    }
}

impl<'a, 'b, T: Bits, O: CombineOp<T>> Iterator for BitCombinationIterator<'a, 'b, T, O> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(bucket_iter) = &mut self.bucket_iter {
            match bucket_iter.next() {
                Some(bit) => return Some(bit),
                None => {
                    self.bucket_iter = self
                        .bucket_combination_iter
                        .next()
                        .map(BucketCombination::combine::<O>)
                        .map(BucketIterator::new)
                }
            }
        }
        None
    }
}

#[test]
fn combines_correctly() {
    use crate::*;

    let p = TreeBitSet::from([1, 2, 64, 666]);
    let q = TreeBitSet::from([3, 4, 333]);

    let iter = BucketCombinationIterator::new(p.buckets.iter(), q.buckets.iter());
    let combinations = iter.collect::<Vec<_>>();

    assert_eq!(
        vec![
            BucketCombination::new(0_u64, Combination::PQ(1 << 1 | 1 << 2, 1 << 3 | 1 << 4)),
            BucketCombination::new(64, Combination::P(1 << 0)),
            BucketCombination::new(320, Combination::Q(1 << 13)),
            BucketCombination::new(640, Combination::P(1 << 26))
        ],
        combinations
    );
}

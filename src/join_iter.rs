use crate::{
    iter::{next_bucket, Bucket, BucketIterator, RawRefIter},
    op::JoinOp,
    Bits,
};

pub struct BucketJoinIterator<'a, 'b, T: Bits> {
    p: RawRefIter<'a, T>,
    q: RawRefIter<'b, T>,
    bucket_p: Option<Bucket<T>>,
    bucket_q: Option<Bucket<T>>,
}

impl<'a, 'b, T: Bits> BucketJoinIterator<'a, 'b, T> {
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

#[derive(Debug, Eq, PartialEq)]
pub enum Join<T: Bits> {
    PQ(T, T),
    P(T),
    Q(T),
}

impl<'a, 'b, T: Bits> Iterator for BucketJoinIterator<'a, 'b, T> {
    type Item = (T, Join<T>);

    fn next(&mut self) -> Option<Self::Item> {
        match (&mut self.bucket_p, &mut self.bucket_q) {
            (Some(p), Some(q)) if p.base == q.base => {
                let item = (p.base, Join::PQ(p.bits, q.bits));

                self.bucket_p = next_bucket(&mut self.p);
                self.bucket_q = next_bucket(&mut self.q);

                Some(item)
            }
            (Some(p), Some(q)) if p.base < q.base => {
                let item = (p.base, Join::P(p.bits));

                self.bucket_p = next_bucket(&mut self.p);
                Some(item)
            }
            (Some(p), None) => {
                let item = (p.base, Join::P(p.bits));

                self.bucket_p = next_bucket(&mut self.p);
                Some(item)
            }
            (_, Some(q)) => {
                let item = (q.base, Join::Q(q.bits));

                self.bucket_q = next_bucket(&mut self.q);
                Some(item)
            }
            (None, None) => None,
        }
    }
}

pub struct BitJoinIterator<'a, 'b, T: Bits, O: JoinOp<T>> {
    bucket_join_iter: BucketJoinIterator<'a, 'b, T>,
    bucket_iter: Option<BucketIterator<T>>,
    _op: O,
}

impl<'a, 'b, T: Bits, O: JoinOp<T>> BitJoinIterator<'a, 'b, T, O> {
    pub fn new(mut bucket_join_iter: BucketJoinIterator<'a, 'b, T>, op: O) -> Self {
        let bucket_iter = bucket_join_iter.next().map(|(base, join)| {
            BucketIterator::new(Bucket {
                base,
                bits: O::join(join),
            })
        });

        Self {
            bucket_join_iter,
            bucket_iter,
            _op: op,
        }
    }
}

impl<'a, 'b, T: Bits, O: JoinOp<T>> Iterator for BitJoinIterator<'a, 'b, T, O> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.bucket_iter {
            None => None,
            Some(bucket_iter) => match bucket_iter.next() {
                None => {
                    self.bucket_iter = self.bucket_join_iter.next().map(|(base, join)| {
                        BucketIterator::new(Bucket {
                            base,
                            bits: O::join(join),
                        })
                    });

                    self.next()
                }
                Some(next) => Some(next),
            },
        }
    }
}

#[test]
fn joins_correctly() {
    use crate::*;

    let p = TreeBitSet::from([1, 2, 64, 666]);
    let q = TreeBitSet::from([3, 4, 333]);

    let iter = BucketJoinIterator::new(p.buckets.iter(), q.buckets.iter());
    let joins = iter.collect::<Vec<_>>();

    assert_eq!(
        vec![
            (0_u64, Join::PQ(1 << 1 | 1 << 2, 1 << 3 | 1 << 4)),
            (64, Join::P(1 << 0)),
            (320, Join::Q(1 << 13)),
            (640, Join::P(1 << 26))
        ],
        joins
    );
}

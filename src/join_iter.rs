use crate::{
    iter::{next_bucket, Bucket, BucketIterator, RawRefIter},
    op::JoinOp,
    Bits,
};

pub struct BucketJoinIterator<'a, 'b, T: Bits> {
    a: RawRefIter<'a, T>,
    b: RawRefIter<'b, T>,
    bucket_a: Option<Bucket<T>>,
    bucket_b: Option<Bucket<T>>,
}

impl<'a, 'b, T: Bits> BucketJoinIterator<'a, 'b, T> {
    pub fn new(mut a: RawRefIter<'a, T>, mut b: RawRefIter<'b, T>) -> Self {
        let bucket_a = next_bucket(&mut a);
        let bucket_b = next_bucket(&mut b);

        Self {
            a,
            b,
            bucket_a,
            bucket_b,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Join<T: Bits> {
    A(T),
    B(T),
    AB(T, T),
}

impl<'a, 'b, T: Bits> Iterator for BucketJoinIterator<'a, 'b, T> {
    type Item = (T, Join<T>);

    fn next(&mut self) -> Option<Self::Item> {
        match (&mut self.bucket_a, &mut self.bucket_b) {
            (Some(a), Some(b)) if a.base == b.base => {
                let item = (a.base, Join::AB(a.bits, b.bits));

                self.bucket_a = next_bucket(&mut self.a);
                self.bucket_b = next_bucket(&mut self.b);

                Some(item)
            }
            (Some(a), Some(b)) if a.base < b.base => {
                let item = (a.base, Join::A(a.bits));

                self.bucket_a = next_bucket(&mut self.a);
                Some(item)
            }
            (Some(a), None) => {
                let item = (a.base, Join::A(a.bits));

                self.bucket_a = next_bucket(&mut self.a);
                Some(item)
            }
            (_, Some(b)) => {
                let item = (b.base, Join::B(b.bits));

                self.bucket_b = next_bucket(&mut self.b);
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

    let a = TreeBitSet::from([1, 2, 64, 666]);
    let b = TreeBitSet::from([3, 4, 333]);

    let iter = BucketJoinIterator::new(a.buckets.iter(), b.buckets.iter());
    let joins = iter.collect::<Vec<_>>();

    assert_eq!(
        vec![
            (0_u64, Join::AB(1 << 1 | 1 << 2, 1 << 3 | 1 << 4)),
            (64, Join::A(1 << 0)),
            (320, Join::B(1 << 13)),
            (640, Join::A(1 << 26))
        ],
        joins
    );
}

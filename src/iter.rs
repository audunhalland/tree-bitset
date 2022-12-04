use std::collections::BTreeMap;

use crate::Bits;

pub type RawRefIter<'b, T> = <&'b BTreeMap<T, T> as IntoIterator>::IntoIter;

pub fn next_bucket<'b, T: Bits>(raw: &mut RawRefIter<'b, T>) -> Option<Bucket<T>> {
    raw.next().map(|(base, bits)| Bucket {
        base: *base,
        bits: *bits,
    })
}

pub fn next_bucket_iterator<'b, T: Bits>(raw: &mut RawRefIter<'b, T>) -> Option<BucketIterator<T>> {
    let bucket = next_bucket(raw)?;
    Some(BucketIterator {
        bucket,
        index: T::ZERO,
    })
}

pub struct RefIterator<'b, T: Bits> {
    pub raw: RawRefIter<'b, T>,
    pub bucket_iter: Option<BucketIterator<T>>,
}

impl<'b, T: Bits> Iterator for RefIterator<'b, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.bucket_iter {
            Some(bucket_iter) => match bucket_iter.next() {
                Some(next) => Some(next),
                None => {
                    self.bucket_iter = next_bucket_iterator(&mut self.raw);
                    self.next()
                }
            },
            None => None,
        }
    }
}

pub struct Bucket<T> {
    pub base: T,
    pub bits: T,
}

pub struct BucketIterator<T> {
    bucket: Bucket<T>,
    index: T,
}

impl<T: Bits> BucketIterator<T> {
    pub fn new(bucket: Bucket<T>) -> Self {
        Self {
            bucket,
            index: T::ZERO,
        }
    }
}

impl<T: Bits> Iterator for BucketIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: Optimize:
        let mut i = self.index;

        while i < T::MASK {
            if self.bucket.bits & (T::ONE << i) > T::ZERO {
                self.index = i + T::ONE;
                return Some(self.bucket.base + i);
            }

            i += T::ONE;
        }

        None
    }
}

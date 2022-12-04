use std::collections::BTreeMap;

use crate::Bits;

pub type RawRefIter<'b, T> = <&'b BTreeMap<T, T> as IntoIterator>::IntoIter;

#[inline]
pub fn next_bucket<'b, T: Bits>(raw: &mut RawRefIter<'b, T>) -> Option<Bucket<T>> {
    raw.next().map(|(base, bits)| Bucket::new(*base, *bits))
}

pub fn next_bucket_iterator<'b, T: Bits>(raw: &mut RawRefIter<'b, T>) -> Option<BucketIterator<T>> {
    Some(BucketIterator::new(next_bucket(raw)?))
}

pub struct RefIterator<'b, T: Bits> {
    pub raw: RawRefIter<'b, T>,
    pub bucket_iter: Option<BucketIterator<T>>,
}

impl<'b, T: Bits> Iterator for RefIterator<'b, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(bucket_iter) = &mut self.bucket_iter {
            match bucket_iter.next() {
                Some(bit) => return Some(bit),
                None => {
                    self.bucket_iter = next_bucket_iterator(&mut self.raw);
                }
            }
        }
        None
    }
}

pub struct Bucket<T> {
    pub base: T,
    pub bits: T,
}

impl<T> Bucket<T> {
    pub fn new(base: T, bits: T) -> Self {
        Self { base, bits }
    }
}

pub struct BucketIterator<T> {
    cursor: T,
    remaining_bits: T,
}

impl<T: Bits> BucketIterator<T> {
    pub fn new(bucket: Bucket<T>) -> Self {
        Self {
            cursor: bucket.base,
            remaining_bits: bucket.bits,
        }
    }
}

impl<T: Bits> Iterator for BucketIterator<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_bits == T::ZERO {
            return None;
        }

        let cursor = self.cursor;
        let trailing = self.remaining_bits.trailing_zeros();
        let shift = trailing + T::ONE;
        self.remaining_bits = self.remaining_bits >> shift;
        self.cursor += shift;

        Some(cursor + trailing)
    }
}

#[test]
fn iterate_empty_bucket() {
    let mut iterator = BucketIterator::<u64>::new(Bucket::new(0, 0));
    assert_eq!(None, iterator.next());
}

#[test]
fn iterate_first_bits() {
    let mut iterator = BucketIterator::<u64>::new(Bucket::new(0, 0x7));
    assert_eq!(Some(0), iterator.next());
    assert_eq!(Some(1), iterator.next());
    assert_eq!(Some(2), iterator.next());
    assert_eq!(None, iterator.next());
}

#[test]
fn iterate_skip_bits() {
    let mut iterator = BucketIterator::<u64>::new(Bucket::new(0, 0xa));
    assert_eq!(Some(1), iterator.next());
    assert_eq!(Some(3), iterator.next());
    assert_eq!(None, iterator.next());
}

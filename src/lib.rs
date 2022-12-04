use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt::Debug,
};

use combination_iter::{BitCombinationIterator, BucketCombinationIterator};
use iter::{next_bucket_iterator, RefIterator};

mod combination_iter;
mod iter;
mod op;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct TreeBitSet<T: Bits = u64> {
    buckets: BTreeMap<T, T>,
}

impl<T: Bits> TreeBitSet<T> {
    pub fn new() -> Self {
        Self {
            buckets: BTreeMap::new(),
        }
    }

    pub fn contains(&self, bit: T) -> bool {
        let bucket_key = bit & !T::MASK;

        self.buckets
            .get(&bucket_key)
            .map(|bits| (*bits & (T::ONE << (bit - bucket_key))) > T::ZERO)
            .unwrap_or(false)
    }

    pub fn insert(&mut self, bit: T) {
        let bucket_key = bit & !T::MASK;

        let bits = self.buckets.entry(bucket_key).or_default();
        *bits |= T::ONE << (bit - bucket_key);
    }

    pub fn remove(&mut self, bit: T) {
        let bucket_key = bit & !T::MASK;

        match self.buckets.entry(bucket_key) {
            Entry::Vacant(_) => {}
            Entry::Occupied(mut entry) => {
                let local_bit = bit - bucket_key;
                *entry.get_mut() &= !(T::ONE << local_bit);

                if *entry.get() == T::ZERO {
                    entry.remove();
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.buckets.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        let mut raw = self.buckets.iter();
        let bucket_iter = next_bucket_iterator(&mut raw);

        RefIterator { raw, bucket_iter }
    }

    pub fn union<'q>(&self, q: &'q TreeBitSet<T>) -> Union<'_, 'q, T> {
        Union {
            bit_iter: BitCombinationIterator::new(
                BucketCombinationIterator::new(self.buckets.iter(), q.buckets.iter()),
                op::Union,
            ),
        }
    }

    pub fn intersection<'q>(&self, q: &'q TreeBitSet<T>) -> Intersection<'_, 'q, T> {
        Intersection {
            bit_iter: BitCombinationIterator::new(
                BucketCombinationIterator::new(self.buckets.iter(), q.buckets.iter()),
                op::Intersection,
            ),
        }
    }

    pub fn difference<'q>(&self, q: &'q TreeBitSet<T>) -> Difference<'_, 'q, T> {
        Difference {
            bit_iter: BitCombinationIterator::new(
                BucketCombinationIterator::new(self.buckets.iter(), q.buckets.iter()),
                op::Difference,
            ),
        }
    }
}

impl<I, T: Bits> From<I> for TreeBitSet<T>
where
    I: IntoIterator<Item = T>,
{
    fn from(into_iter: I) -> Self {
        let mut bitset = TreeBitSet::new();

        for bit in into_iter.into_iter() {
            bitset.insert(bit);
        }

        bitset
    }
}

impl<T: Bits> Debug for TreeBitSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_set = f.debug_set();
        for bit in self.iter() {
            debug_set.entry(&bit);
        }
        debug_set.finish()
    }
}

impl<T: Bits> Extend<T> for TreeBitSet<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for bit in iter {
            self.insert(bit);
        }
    }
}

pub struct Union<'p, 'q, T: Bits> {
    bit_iter: BitCombinationIterator<'p, 'q, T, op::Union>,
}

impl<'p, 'q, T: Bits> Iterator for Union<'p, 'q, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.bit_iter.next()
    }
}

pub struct Intersection<'p, 'q, T: Bits> {
    bit_iter: BitCombinationIterator<'p, 'q, T, op::Intersection>,
}

impl<'p, 'q, T: Bits> Iterator for Intersection<'p, 'q, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.bit_iter.next()
    }
}

pub struct Difference<'p, 'q, T: Bits> {
    bit_iter: BitCombinationIterator<'p, 'q, T, op::Difference>,
}

impl<'p, 'q, T: Bits> Iterator for Difference<'p, 'q, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.bit_iter.next()
    }
}

pub trait Bits:
    Sized
    + Copy
    + Debug
    + Ord
    + std::ops::BitAnd<Self, Output = Self>
    + std::ops::Shl<Output = Self>
    + std::ops::Shr<Output = Self>
    + std::ops::BitOr<Output = Self>
    + std::ops::BitOrAssign
    + std::ops::BitAndAssign
    + std::ops::Not<Output = Self>
    + std::ops::Add<Output = Self>
    + std::ops::AddAssign
    + std::ops::Sub<Output = Self>
    + Default
{
    const ZERO: Self;
    const ONE: Self;
    const MASK: Self;

    fn trailing_zeros(self) -> Self;
}

impl Bits for u32 {
    const ZERO: u32 = 0;
    const ONE: u32 = 1;
    const MASK: u32 = 0x1f;

    fn trailing_zeros(self) -> Self {
        self.trailing_zeros()
    }
}

impl Bits for u64 {
    const ZERO: u64 = 0;
    const ONE: u64 = 1;
    const MASK: u64 = 0x3f;

    fn trailing_zeros(self) -> Self {
        self.trailing_zeros() as Self
    }
}

impl Bits for u128 {
    const ZERO: u128 = 0;
    const ONE: u128 = 1;
    const MASK: u128 = 0x7f;

    fn trailing_zeros(self) -> Self {
        self.trailing_zeros() as Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_u64() {
        let mut bits: TreeBitSet = TreeBitSet::new();

        assert_eq!("{}", format!("{bits:?}"));

        bits.insert(42);
        assert_eq!("{42}", format!("{bits:?}"));

        bits.insert(43);
        assert_eq!("{42, 43}", format!("{bits:?}"));

        bits.insert(6666);
        assert_eq!("{42, 43, 6666}", format!("{bits:?}"));

        bits.insert(887);
        assert_eq!("{42, 43, 887, 6666}", format!("{bits:?}"));

        println!("{:#?}", bits.buckets);

        bits.remove(887);
        assert_eq!("{42, 43, 6666}", format!("{bits:?}"));

        let cloned = bits.clone();
        assert_eq!(cloned, bits);

        assert!(!bits.contains(0));
        assert!(!bits.contains(1));
        assert!(bits.contains(42));
        assert!(bits.contains(6666));
        assert!(!bits.contains(6667));
    }

    #[test]
    fn basic_u32() {
        let mut bits: TreeBitSet<u32> = TreeBitSet::new();

        assert_eq!("{}", format!("{bits:?}"));

        bits.insert(42);
        assert_eq!("{42}", format!("{bits:?}"));

        bits.insert(43);
        assert_eq!("{42, 43}", format!("{bits:?}"));

        bits.insert(6666);
        assert_eq!("{42, 43, 6666}", format!("{bits:?}"));

        bits.insert(887);
        assert_eq!("{42, 43, 887, 6666}", format!("{bits:?}"));

        println!("{:#?}", bits.buckets);

        bits.remove(887);
        assert_eq!("{42, 43, 6666}", format!("{bits:?}"));

        assert!(!bits.contains(0));
        assert!(!bits.contains(1));
        assert!(bits.contains(42));
        assert!(bits.contains(6666));
        assert!(!bits.contains(6667));
    }

    #[test]
    fn union() {
        let a = TreeBitSet::<u64>::from([100, 200, 300, 400]);
        let b = TreeBitSet::<u64>::from([400, 500, 600]);

        let union: TreeBitSet = a.union(&b).into();

        assert_eq!(TreeBitSet::from([100, 200, 300, 400, 500, 600]), union);

        println!("union: {:#?}", union.buckets);
    }

    #[test]
    fn intersection() {
        let a = TreeBitSet::<u64>::from([100, 200, 300, 400, 500]);
        let b = TreeBitSet::<u64>::from([400, 500, 600, 700]);

        let intersection: TreeBitSet = a.intersection(&b).into();

        assert_eq!(TreeBitSet::from([400, 500]), intersection);
    }

    #[test]
    fn difference() {
        let a = TreeBitSet::<u64>::from([100, 200, 300, 400, 500]);
        let b = TreeBitSet::<u64>::from([400, 500, 600, 700]);

        let intersection: TreeBitSet = a.difference(&b).into();

        assert_eq!(TreeBitSet::from([100, 200, 300]), intersection);
    }
}

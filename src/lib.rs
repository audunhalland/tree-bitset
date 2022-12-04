use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt::Debug,
};

use iter::{next_bucket_iterator, RefIterator};
use join_iter::{BitJoinIterator, BucketJoinIterator};

mod iter;
mod join_iter;
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
        let local_bit = bit - bucket_key;

        self.buckets
            .get(&bucket_key)
            .map(|bits| (*bits & (T::ONE << local_bit)) > T::ZERO)
            .unwrap_or(false)
    }

    pub fn insert(&mut self, bit: T) {
        let bucket_key = bit & !T::MASK;
        let local_bit = bit - bucket_key;

        let bits = self.buckets.entry(bucket_key).or_default();
        *bits |= T::ONE << local_bit;
    }

    pub fn remove(&mut self, bit: T) {
        let bucket_key = bit & !T::MASK;
        let local_bit = bit - bucket_key;

        match self.buckets.entry(bucket_key) {
            Entry::Vacant(_) => {}
            Entry::Occupied(mut entry) => {
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

    pub fn union<'o>(&self, other: &'o TreeBitSet<T>) -> Union<'_, 'o, T> {
        Union {
            join_iter: BitJoinIterator::new(
                BucketJoinIterator::new(self.buckets.iter(), other.buckets.iter()),
                op::Union,
            ),
        }
    }

    pub fn intersection<'o>(&self, other: &'o TreeBitSet<T>) -> Intersection<'_, 'o, T> {
        Intersection {
            join_iter: BitJoinIterator::new(
                BucketJoinIterator::new(self.buckets.iter(), other.buckets.iter()),
                op::Intersection,
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
        write!(f, "{{")?;

        let mut iterator = self.iter().peekable();
        while let Some(bit) = iterator.next() {
            write!(f, "{bit:?}")?;
            if iterator.peek().is_some() {
                write!(f, ", ")?;
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}

pub struct Union<'a, 'b, T: Bits> {
    join_iter: BitJoinIterator<'a, 'b, T, op::Union>,
}

impl<'a, 'b, T: Bits> Iterator for Union<'a, 'b, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.join_iter.next()
    }
}

pub struct Intersection<'a, 'b, T: Bits> {
    join_iter: BitJoinIterator<'a, 'b, T, op::Intersection>,
}

impl<'a, 'b, T: Bits> Iterator for Intersection<'a, 'b, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.join_iter.next()
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
    fn join() {
        let a = TreeBitSet::<u64>::from([100, 200, 300]);
        let b = TreeBitSet::<u64>::from([400, 500, 600]);

        let union = TreeBitSet::from(a.union(&b));

        assert_eq!(TreeBitSet::from([100, 200, 300, 400, 500, 600]), union);
    }

    #[test]
    fn intersection() {
        let a = TreeBitSet::<u64>::from([100, 200, 300, 400, 500]);
        let b = TreeBitSet::<u64>::from([400, 500, 600, 700]);

        let intersection = TreeBitSet::from(a.intersection(&b));

        assert_eq!(TreeBitSet::from([400, 500]), intersection);
    }
}

use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt::Debug,
};

use iter::{next_group_iter, RefIterator};

mod iter;
mod join_iter;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BitTree<T: Bits = u64> {
    groups: BTreeMap<T, T>,
}

impl<T: Bits> BitTree<T> {
    pub fn new() -> Self {
        Self {
            groups: BTreeMap::new(),
        }
    }

    pub fn contains(&self, bit: T) -> bool {
        let group = bit & !T::MASK;
        let local_bit = bit - group;
        self.groups
            .get(&group)
            .map(|bits| (*bits & (T::ONE << local_bit)) > T::ZERO)
            .unwrap_or(false)
    }

    pub fn insert(&mut self, bit: T) {
        let group = bit & !T::MASK;
        let local_bit = bit - group;

        let bits = self.groups.entry(group).or_default();
        *bits |= T::ONE << local_bit;
    }

    pub fn remove(&mut self, bit: T) {
        let group = bit & !T::MASK;
        let local_bit = bit - group;

        match self.groups.entry(group) {
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
        self.groups.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        let mut raw = self.groups.iter();
        let group_iter = next_group_iter(&mut raw);

        RefIterator { raw, group_iter }
    }
}

impl<I, T: Bits> From<I> for BitTree<T>
where
    I: IntoIterator<Item = T>,
{
    fn from(into_iter: I) -> Self {
        let mut bitset = BitTree::new();

        for bit in into_iter.into_iter() {
            bitset.insert(bit);
        }

        bitset
    }
}

impl<T: Bits> Debug for BitTree<T> {
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

pub trait Bits:
    Sized
    + Copy
    + Debug
    + Ord
    + std::ops::BitAnd<Self, Output = Self>
    + std::ops::Shl<Output = Self>
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
}

impl Bits for u32 {
    const ZERO: u32 = 0;
    const ONE: u32 = 1;
    const MASK: u32 = 0x1f;
}

impl Bits for u64 {
    const ZERO: u64 = 0;
    const ONE: u64 = 1;
    const MASK: u64 = 0x3f;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_u64() {
        let mut bits: BitTree<u64> = BitTree::new();

        assert_eq!("{}", format!("{bits:?}"));

        bits.insert(42);
        assert_eq!("{42}", format!("{bits:?}"));

        bits.insert(43);
        assert_eq!("{42, 43}", format!("{bits:?}"));

        bits.insert(6666);
        assert_eq!("{42, 43, 6666}", format!("{bits:?}"));

        bits.insert(887);
        assert_eq!("{42, 43, 887, 6666}", format!("{bits:?}"));

        println!("{:#?}", bits.groups);

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
        let mut bits: BitTree<u32> = BitTree::new();

        assert_eq!("{}", format!("{bits:?}"));

        bits.insert(42);
        assert_eq!("{42}", format!("{bits:?}"));

        bits.insert(43);
        assert_eq!("{42, 43}", format!("{bits:?}"));

        bits.insert(6666);
        assert_eq!("{42, 43, 6666}", format!("{bits:?}"));

        bits.insert(887);
        assert_eq!("{42, 43, 887, 6666}", format!("{bits:?}"));

        println!("{:#?}", bits.groups);

        bits.remove(887);
        assert_eq!("{42, 43, 6666}", format!("{bits:?}"));

        assert!(!bits.contains(0));
        assert!(!bits.contains(1));
        assert!(bits.contains(42));
        assert!(bits.contains(6666));
        assert!(!bits.contains(6667));
    }
}

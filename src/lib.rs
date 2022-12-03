use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt::Debug,
};

use iter::{next_group_iter, RefIterator};

mod iter;
mod join_iter;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BitTree {
    groups: BTreeMap<u64, u64>,
}

impl BitTree {
    pub fn new() -> Self {
        Self {
            groups: BTreeMap::new(),
        }
    }

    pub fn contains(&self, bit: u64) -> bool {
        let group = bit & !0x3f;
        let local_bit = bit - group;
        self.groups
            .get(&group)
            .map(|bits| (bits & (0x1 << local_bit)) > 0)
            .unwrap_or(false)
    }

    pub fn insert(&mut self, bit: u64) {
        let group = bit & !0x3f;
        let local_bit = bit - group;

        let bits = self.groups.entry(group).or_default();
        *bits |= 0x1 << local_bit;
    }

    pub fn remove(&mut self, bit: u64) {
        let group = bit & !0x3f;
        let local_bit = bit - group;

        match self.groups.entry(group) {
            Entry::Vacant(_) => {}
            Entry::Occupied(mut entry) => {
                *entry.get_mut() &= !(0x1 << local_bit);

                if *entry.get() == 0 {
                    entry.remove();
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.groups.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = u64> + '_ {
        let mut raw = self.groups.iter();
        let group_iter = next_group_iter(&mut raw);

        RefIterator { raw, group_iter }
    }
}

impl<T> From<T> for BitTree
where
    T: IntoIterator<Item = u64>,
{
    fn from(into_iter: T) -> Self {
        let mut bitset = BitTree::new();

        for bit in into_iter.into_iter() {
            bitset.insert(bit);
        }

        bitset
    }
}

impl Debug for BitTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;

        let mut iterator = self.iter().peekable();
        while let Some(bit) = iterator.next() {
            write!(f, "{bit}")?;
            if iterator.peek().is_some() {
                write!(f, ", ")?;
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mut bits = BitTree::new();

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

use std::collections::BTreeMap;

use crate::Bits;

pub type RawRefIter<'b, T: Bits> = <&'b BTreeMap<T, T> as IntoIterator>::IntoIter;

pub fn next_group<'b, T: Bits>(raw: &mut RawRefIter<'b, T>) -> Option<Group<T>> {
    raw.next().map(|(base, bits)| Group {
        base: *base,
        bits: *bits,
    })
}

pub fn next_group_iter<'b, T: Bits>(raw: &mut RawRefIter<'b, T>) -> Option<GroupIterator<T>> {
    let group = next_group(raw)?;
    Some(GroupIterator {
        group,
        index: T::ZERO,
    })
}

pub struct RefIterator<'b, T: Bits> {
    pub raw: RawRefIter<'b, T>,
    pub group_iter: Option<GroupIterator<T>>,
}

impl<'b, T: Bits> Iterator for RefIterator<'b, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.group_iter {
            Some(group_iter) => match group_iter.next() {
                Some(next) => Some(next),
                None => {
                    self.group_iter = next_group_iter(&mut self.raw);
                    self.next()
                }
            },
            None => None,
        }
    }
}

pub struct Group<T> {
    pub base: T,
    pub bits: T,
}

pub struct GroupIterator<T> {
    group: Group<T>,
    index: T,
}

impl<T: Bits> Iterator for GroupIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: Optimize:
        let mut i = self.index;

        while i < T::MASK {
            if self.group.bits & (T::ONE << i) > T::ZERO {
                self.index = i + T::ONE;
                return Some(self.group.base + i);
            }

            i += T::ONE;
        }

        None
    }
}

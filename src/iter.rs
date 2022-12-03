use std::collections::BTreeMap;

pub type RawRefIter<'b> = <&'b BTreeMap<u64, u64> as IntoIterator>::IntoIter;

pub fn next_group<'b>(raw: &mut RawRefIter<'b>) -> Option<Group> {
    raw.next().map(|(base, bits)| Group {
        base: *base,
        bits: *bits,
    })
}

pub fn next_group_iter<'b>(raw: &mut RawRefIter<'b>) -> Option<GroupIterator> {
    let group = next_group(raw)?;
    Some(GroupIterator { group, index: 0 })
}

pub struct RefIterator<'b> {
    pub raw: RawRefIter<'b>,
    pub group_iter: Option<GroupIterator>,
}

impl<'b> Iterator for RefIterator<'b> {
    type Item = u64;

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

pub struct Group {
    pub base: u64,
    pub bits: u64,
}

pub struct GroupIterator {
    group: Group,
    index: u8,
}

impl Iterator for GroupIterator {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: Optimize:
        for i in self.index..63 {
            if self.group.bits & (1 << i) > 0 {
                self.index = i + 1;
                return Some(self.group.base + i as u64);
            }
        }

        None
    }
}

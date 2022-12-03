use crate::iter::{next_group, Group, RawRefIter};

pub struct JoinGroupIter<'a, 'b> {
    a: RawRefIter<'a>,
    b: RawRefIter<'b>,
    a_group: Option<Group>,
    b_group: Option<Group>,
}

impl<'a, 'b> JoinGroupIter<'a, 'b> {
    fn new(mut a: RawRefIter<'a>, mut b: RawRefIter<'b>) -> Self {
        let a_group = next_group(&mut a);
        let b_group = next_group(&mut b);

        Self {
            a,
            b,
            a_group,
            b_group,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Join {
    A(u64),
    B(u64),
    AB(u64, u64),
}

impl<'a, 'b> Iterator for JoinGroupIter<'a, 'b> {
    type Item = (u64, Join);

    fn next(&mut self) -> Option<Self::Item> {
        match (&mut self.a_group, &mut self.b_group) {
            (Some(a), Some(b)) if a.base == b.base => {
                let item = (a.base, Join::AB(a.bits, b.bits));

                self.a_group = next_group(&mut self.a);
                self.b_group = next_group(&mut self.b);

                Some(item)
            }
            (Some(a), Some(b)) if a.base < b.base => {
                let item = (a.base, Join::A(a.bits));

                self.a_group = next_group(&mut self.a);
                Some(item)
            }
            (Some(a), None) => {
                let item = (a.base, Join::A(a.bits));

                self.a_group = next_group(&mut self.a);
                Some(item)
            }
            (_, Some(b)) => {
                let item = (b.base, Join::B(b.bits));

                self.b_group = next_group(&mut self.b);
                Some(item)
            }
            (None, None) => None,
        }
    }
}

#[test]
fn joins_correctly() {
    use crate::*;

    let a = BitTree::from([1, 2, 64, 666]);
    let b = BitTree::from([3, 4, 333]);

    let iter = JoinGroupIter::new(a.groups.iter(), b.groups.iter());
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

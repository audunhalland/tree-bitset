use crate::{join_iter::Join, Bits};

pub trait JoinOp<T: Bits> {
    fn join(input: Join<T>) -> T;
}

pub struct Union;

impl<T: Bits> JoinOp<T> for Union {
    #[inline]
    fn join(input: Join<T>) -> T {
        match input {
            Join::PQ(p, q) => p | q,
            Join::P(p) => p,
            Join::Q(q) => q,
        }
    }
}

pub struct Intersection;

impl<T: Bits> JoinOp<T> for Intersection {
    #[inline]
    fn join(input: Join<T>) -> T {
        match input {
            Join::PQ(p, q) => p & q,
            Join::P(_) | Join::Q(_) => T::ZERO,
        }
    }
}

pub struct Difference;

impl<T: Bits> JoinOp<T> for Difference {
    #[inline]
    fn join(input: Join<T>) -> T {
        match input {
            Join::PQ(p, q) => p & !q,
            Join::P(p) => p,
            Join::Q(_) => T::ZERO,
        }
    }
}

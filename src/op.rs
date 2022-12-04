use crate::{join_iter::Join, Bits};

pub trait JoinOp<T: Bits> {
    fn join(input: Join<T>) -> T;
}

pub struct Union;

impl<T: Bits> JoinOp<T> for Union {
    fn join(input: Join<T>) -> T {
        match input {
            Join::AB(a, b) => a | b,
            Join::A(a) => a,
            Join::B(b) => b,
        }
    }
}

pub struct Intersection;

impl<T: Bits> JoinOp<T> for Intersection {
    fn join(input: Join<T>) -> T {
        match input {
            Join::AB(a, b) => a & b,
            Join::A(_) | Join::B(_) => T::ZERO,
        }
    }
}

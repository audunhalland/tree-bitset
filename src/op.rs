use crate::{combination_iter::Combination, Bits};

pub trait CombineOp<T: Bits> {
    fn combine(input: Combination<T>) -> T;
}

pub struct Union;

impl<T: Bits> CombineOp<T> for Union {
    #[inline]
    fn combine(input: Combination<T>) -> T {
        match input {
            Combination::PQ(p, q) => p | q,
            Combination::P(p) => p,
            Combination::Q(q) => q,
        }
    }
}

pub struct Intersection;

impl<T: Bits> CombineOp<T> for Intersection {
    #[inline]
    fn combine(input: Combination<T>) -> T {
        match input {
            Combination::PQ(p, q) => p & q,
            Combination::P(_) | Combination::Q(_) => T::ZERO,
        }
    }
}

pub struct Difference;

impl<T: Bits> CombineOp<T> for Difference {
    #[inline]
    fn combine(input: Combination<T>) -> T {
        match input {
            Combination::PQ(p, q) => p & !q,
            Combination::P(p) => p,
            Combination::Q(_) => T::ZERO,
        }
    }
}

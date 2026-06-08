use crate::typeclasses::Semigroup;

pub trait Monoid: Semigroup {
    fn empty() -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Monoid for i32 {
        fn empty() -> Self {
            0
        }
    }

    fn empty<A: Monoid>() -> A {
        A::empty()
    }

    #[test]
    fn test_empty_instance() {
        let empty = empty::<i32>();
        assert_eq!(empty, 0);
    }
}

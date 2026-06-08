pub trait Semigroup {
    fn combine(x: &Self, y: &Self) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Semigroup for i32 {
        fn combine(x: &i32, y: &i32) -> i32 {
            x + y
        }
    }

    fn combine<A: Semigroup>(a: &A, b: &A) -> A {
        A::combine(a, b)
    }

    #[test]
    fn test_combine_instance() {
        let combined = combine(&1, &2);
        assert_eq!(combined, 3);
    }
}

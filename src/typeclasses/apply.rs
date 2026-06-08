pub trait Apply<A>: Functor<A> {
    fn ap<B, F: Fn(&A) -> B>(self, ff: Self::Target<F>) -> Self::Target<B>;
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<A> Apply<A> for Option<A> {
        fn ap<B, F: Fn(&A) -> B>(self, ff: Option<F>) -> Self::Target<B> {
            match (self, ff) {
                (Some(a), Some(f)) => Some(f(&a)),
                _ => None,
            }
        }
    }

    #[test]
    fn test_apply() {}
}

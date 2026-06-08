pub trait Apply<A>: Functor<A> {
    fn ap<B, F: Fn(&A) -> B>(self, ff: Self::Target<F>) -> Self::Target<B>;
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<A> Apply<A> for Option<A> {

    }

    #[test]
    fn test_apply() {

    }
}

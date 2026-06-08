pub trait Functor<A> {
    type Target<B>;

    fn fmap<B, F: Fn(&A) -> B>(self, f: F) -> Self::Target<B>;
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<A> Functor<A> for Option<A> {
        type Target<B> = Option<B>;

        fn fmap<B, F: Fn(&A) -> B>(self, f: F) -> Self::Target<B> {
            self.map(|a| f(&a))
        }
    }

    #[test]
    fn test_map() {
        let opt = Some(1);
        let mapped = opt.fmap(|a| a.to_string());
        assert_eq!(Some("2".to_string()), mapped);
    }
}

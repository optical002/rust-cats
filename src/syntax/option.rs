use crate::core::OptionH;
use crate::typeclasses::{Applicative, Apply, FlatMap, Functor};

pub trait OptionExt<A> {
    fn fmap<B, F: Fn(&A) -> B>(self, f: F) -> Option<B>;
    fn ap<B, F: Fn(&A) -> B>(self, ff: Option<F>) -> Option<B>;
    fn flat_map<B, F: Fn(A) -> Option<B>>(self, f: F) -> Option<B>;
}

impl<A> OptionExt<A> for Option<A> {
    fn fmap<B, F: Fn(&A) -> B>(self, f: F) -> Option<B> {
        <OptionH as Functor<OptionH>>::fmap(self, f)
    }

    fn ap<B, F: Fn(&A) -> B>(self, ff: Option<F>) -> Option<B> {
        <OptionH as Apply<OptionH>>::ap(self, ff)
    }

    fn flat_map<B, F: Fn(A) -> Option<B>>(self, f: F) -> Option<B> {
        <OptionH as FlatMap<OptionH>>::flat_map(self, f)
    }
}

impl OptionH {
    pub fn pure<A>(a: A) -> Option<A> {
        <OptionH as Applicative<OptionH>>::pure(a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmap_some() {
        let mapped = Some(1).fmap(|a| a.to_string());
        assert_eq!(Some("1".to_string()), mapped);
    }

    #[test]
    fn test_fmap_none() {
        let opt: Option<i32> = None;
        let result = opt.fmap(|a| a.to_string());
        assert_eq!(result, None);
    }

    #[test]
    fn test_apply_some() {
        let applied = Some(1).ap(Some(|a: &i32| a.to_string()));
        assert_eq!(applied, Some("1".to_string()));
    }

    #[test]
    fn test_apply_none_value() {
        let fa: Option<i32> = None;
        let applied = fa.ap(Some(|a: &i32| a.to_string()));
        assert_eq!(applied, None);
    }

    #[test]
    fn test_apply_none_fn() {
        let ff: Option<fn(&i32) -> String> = None;
        let applied = Some(1).ap(ff);
        assert_eq!(applied, None);
    }

    #[test]
    fn test_pure() {
        let pured: Option<i32> = OptionH::pure(1);
        assert_eq!(pured, Some(1));
    }

    #[test]
    fn test_flat_map_some() {
        let result = Some(1).flat_map(|a| Some(a + 1));
        assert_eq!(result, Some(2));
    }

    #[test]
    fn test_flat_map_none_value() {
        let fa: Option<i32> = None;
        let result = fa.flat_map(|a| Some(a + 1));
        assert_eq!(result, None);
    }

    #[test]
    fn test_flat_map_none_result() {
        let result = Some(1).flat_map(|_: i32| Option::<i32>::None);
        assert_eq!(result, None);
    }

    #[test]
    fn test_monad_pure_then_flat_map() {
        let result = OptionH::pure(1).flat_map(|a| Some(a + 1));
        assert_eq!(result, Some(2));
    }

}

use crate::core::ResultH;
use crate::typeclasses::{Applicative, ApplicativeError, Apply, FlatMap, Functor};

pub trait ResultExt<A, E> {
    fn fmap<B, F: Fn(&A) -> B>(self, f: F) -> Result<B, E>;
    fn ap<B, F: Fn(&A) -> B>(self, ff: Result<F, E>) -> Result<B, E>;
    fn flat_map<B, F: Fn(A) -> Result<B, E>>(self, f: F) -> Result<B, E>;
    fn handle_error_with<F: Fn(E) -> Result<A, E>>(self, f: F) -> Result<A, E>;
}

impl<A, E> ResultExt<A, E> for Result<A, E> {
    fn fmap<B, F: Fn(&A) -> B>(self, f: F) -> Result<B, E> {
        <ResultH<E> as Functor<ResultH<E>>>::fmap(self, f)
    }

    fn ap<B, F: Fn(&A) -> B>(self, ff: Result<F, E>) -> Result<B, E> {
        <ResultH<E> as Apply<ResultH<E>>>::ap(self, ff)
    }

    fn flat_map<B, F: Fn(A) -> Result<B, E>>(self, f: F) -> Result<B, E> {
        <ResultH<E> as FlatMap<ResultH<E>>>::flat_map(self, f)
    }

    fn handle_error_with<F: Fn(E) -> Result<A, E>>(self, f: F) -> Result<A, E> {
        <ResultH<E> as ApplicativeError<ResultH<E>, E>>::handle_error_with(self, f)
    }
}

impl<E> ResultH<E> {
    pub fn raise_error<A>(e: E) -> Result<A, E> {
        <ResultH<E> as ApplicativeError<ResultH<E>, E>>::raise_error(e)
    }

    pub fn pure<A>(a: A) -> Result<A, E> {
        <ResultH<E> as Applicative<ResultH<E>>>::pure(a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmap_ok() {
        let r: Result<i32, &str> = Ok(1);
        let mapped = r.fmap(|a| a.to_string());
        assert_eq!(mapped, Ok("1".to_string()));
    }

    #[test]
    fn test_fmap_err() {
        let r: Result<i32, &str> = Err("boom");
        let mapped = r.fmap(|a| a.to_string());
        assert_eq!(mapped, Err("boom"));
    }

    #[test]
    fn test_apply_ok() {
        let r: Result<i32, &str> = Ok(1);
        let ff: Result<fn(&i32) -> String, &str> = Ok(|a| a.to_string());
        let applied = r.ap(ff);
        assert_eq!(applied, Ok("1".to_string()));
    }

    #[test]
    fn test_apply_err_value() {
        let r: Result<i32, &str> = Err("boom");
        let ff: Result<fn(&i32) -> String, &str> = Ok(|a| a.to_string());
        let applied = r.ap(ff);
        assert_eq!(applied, Err("boom"));
    }

    #[test]
    fn test_apply_err_fn() {
        let r: Result<i32, &str> = Ok(1);
        let ff: Result<fn(&i32) -> String, &str> = Err("nope");
        let applied = r.ap(ff);
        assert_eq!(applied, Err("nope"));
    }

    #[test]
    fn test_pure() {
        let pured: Result<i32, &str> = ResultH::<&str>::pure(1);
        assert_eq!(pured, Ok(1));
    }

    #[test]
    fn test_flat_map_ok() {
        let r: Result<i32, &str> = Ok(1);
        let result = r.flat_map(|a| Ok(a + 1));
        assert_eq!(result, Ok(2));
    }

    #[test]
    fn test_flat_map_err_value() {
        let r: Result<i32, &str> = Err("boom");
        let result = r.flat_map(|a| Ok(a + 1));
        assert_eq!(result, Err("boom"));
    }

    #[test]
    fn test_flat_map_err_result() {
        let r: Result<i32, &str> = Ok(1);
        let result = r.flat_map(|_| Err::<i32, _>("inner"));
        assert_eq!(result, Err("inner"));
    }

    #[test]
    fn test_monad_pure_then_flat_map() {
        let result: Result<i32, &str> = ResultH::<&str>::pure(1).flat_map(|a| Ok(a + 1));
        assert_eq!(result, Ok(2));
    }

    #[test]
    fn test_raise_error() {
        let result: Result<i32, &str> = ResultH::<&str>::raise_error("boom");
        assert_eq!(result, Err("boom"));
    }

    #[test]
    fn test_handle_error_with_ok_passes_through() {
        let r: Result<i32, &str> = Ok(1);
        let result = r.handle_error_with(|_| Ok(99));
        assert_eq!(result, Ok(1));
    }

    #[test]
    fn test_handle_error_with_err_recovers() {
        let r: Result<i32, &str> = Err("boom");
        let result = r.handle_error_with(|_| Ok(99));
        assert_eq!(result, Ok(99));
    }

    #[test]
    fn test_handle_error_with_err_stays_err() {
        let r: Result<i32, &str> = Err("boom");
        let result = r.handle_error_with(|e| Err(e));
        assert_eq!(result, Err("boom"));
    }

    #[test]
    fn test_result_is_monad_error() {
        use crate::core::HKT;
        use crate::typeclasses::MonadError;
        fn _requires_monad_error<F: HKT, E, M: MonadError<F, E>>() {}
        _requires_monad_error::<ResultH<&str>, &str, ResultH<&str>>();
    }
}

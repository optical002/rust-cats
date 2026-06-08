use crate::core::ResultH;
use crate::typeclasses::{
    Applicative, ApplicativeError, Apply, FlatMap, Functor, Invariant, Monad, MonadError,
};

impl<E> Invariant<ResultH<E>> for ResultH<E> {
    fn imap<A, B, Fwd: Fn(A) -> B, Bwd: Fn(B) -> A>(
        fa: Result<A, E>,
        f: Fwd,
        _g: Bwd,
    ) -> Result<B, E> {
        fa.map(f)
    }
}

impl<E> Functor<ResultH<E>> for ResultH<E> {
    fn fmap<A, B, Func: Fn(&A) -> B>(fa: Result<A, E>, f: Func) -> Result<B, E> {
        fa.map(|a| f(&a))
    }
}

impl<E> Apply<ResultH<E>> for ResultH<E> {
    fn ap<A, B, Func: Fn(&A) -> B>(
        fa: Result<A, E>,
        ff: Result<Func, E>,
    ) -> Result<B, E> {
        match (fa, ff) {
            (Ok(a), Ok(f)) => Ok(f(&a)),
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e),
        }
    }
}

impl<E> Applicative<ResultH<E>> for ResultH<E> {
    fn pure<A>(a: A) -> Result<A, E> {
        Ok(a)
    }
}

impl<E> FlatMap<ResultH<E>> for ResultH<E> {
    fn flat_map<A, B, Func: Fn(A) -> Result<B, E>>(
        fa: Result<A, E>,
        f: Func,
    ) -> Result<B, E> {
        fa.and_then(f)
    }
}

impl<E> Monad<ResultH<E>> for ResultH<E> {}

impl<E> ApplicativeError<ResultH<E>, E> for ResultH<E> {
    fn raise_error<A>(e: E) -> Result<A, E> {
        Err(e)
    }

    fn handle_error_with<A, Handler: Fn(E) -> Result<A, E>>(
        fa: Result<A, E>,
        f: Handler,
    ) -> Result<A, E> {
        match fa {
            Ok(a) => Ok(a),
            Err(e) => f(e),
        }
    }
}

impl<E> MonadError<ResultH<E>, E> for ResultH<E> {}

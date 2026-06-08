use crate::core::EvalH;
use crate::data::eval::Eval;
use crate::typeclasses::{Applicative, Apply, Defer, FlatMap, Functor, Invariant, Monad};

impl Invariant<EvalH> for EvalH {
    fn imap<A, B, Fwd: Fn(A) -> B, Bwd: Fn(B) -> A>(
        fa: Eval<A>,
        f: Fwd,
        _g: Bwd,
    ) -> Eval<B> {
        Eval::now(f(fa.into_value()))
    }
}

impl Functor<EvalH> for EvalH {
    fn fmap<A, B, Func: Fn(&A) -> B>(fa: Eval<A>, f: Func) -> Eval<B> {
        let a = fa.into_value();
        Eval::now(f(&a))
    }
}

impl Apply<EvalH> for EvalH {
    fn ap<A, B, Func: Fn(&A) -> B>(fa: Eval<A>, ff: Eval<Func>) -> Eval<B> {
        let a = fa.into_value();
        let f = ff.into_value();
        Eval::now(f(&a))
    }
}

impl Applicative<EvalH> for EvalH {
    fn pure<A>(a: A) -> Eval<A> {
        Eval::now(a)
    }
}

impl FlatMap<EvalH> for EvalH {
    fn flat_map<A, B, Func: Fn(A) -> Eval<B>>(fa: Eval<A>, f: Func) -> Eval<B> {
        f(fa.into_value())
    }
}

impl Monad<EvalH> for EvalH {}

impl Defer<EvalH> for EvalH {
    fn defer<A, Thunk: Fn() -> Eval<A> + 'static>(fa: Thunk) -> Eval<A> {
        Eval::always(move || fa().into_value())
    }
}

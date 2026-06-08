use crate::core::OptionH;
use crate::typeclasses::{Applicative, Apply, FlatMap, Functor, Invariant, Monad};

impl Invariant<OptionH> for OptionH {
    fn imap<A, B, Fwd: Fn(A) -> B, Bwd: Fn(B) -> A>(
        fa: Option<A>,
        f: Fwd,
        _g: Bwd,
    ) -> Option<B> {
        fa.map(f)
    }
}

impl Functor<OptionH> for OptionH {
    fn fmap<A, B, Func: Fn(&A) -> B>(fa: Option<A>, f: Func) -> Option<B> {
        fa.map(|a| f(&a))
    }
}

impl Apply<OptionH> for OptionH {
    fn ap<A, B, Func: Fn(&A) -> B>(fa: Option<A>, ff: Option<Func>) -> Option<B> {
        match (fa, ff) {
            (Some(a), Some(f)) => Some(f(&a)),
            _ => None,
        }
    }
}

impl Applicative<OptionH> for OptionH {
    fn pure<A>(a: A) -> Option<A> {
        Some(a)
    }
}

impl FlatMap<OptionH> for OptionH {
    fn flat_map<A, B, Func: Fn(A) -> Option<B>>(fa: Option<A>, f: Func) -> Option<B> {
        fa.and_then(f)
    }
}

impl Monad<OptionH> for OptionH {}

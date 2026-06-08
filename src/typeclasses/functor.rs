use crate::core::HKT;
use crate::typeclasses::Invariant;

pub trait Functor<F: HKT>: Invariant<F> {
    fn fmap<A, B, Func: Fn(&A) -> B>(fa: F::Applied<A>, f: Func) -> F::Applied<B>;
}

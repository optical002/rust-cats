use crate::core::HKT;
use crate::typeclasses::Functor;

pub trait Apply<F: HKT>: Functor<F> {
    fn ap<A, B, Func: Fn(&A) -> B>(
        fa: F::Applied<A>,
        ff: F::Applied<Func>,
    ) -> F::Applied<B>;
}

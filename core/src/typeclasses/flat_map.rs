use crate::core::HKT;
use crate::typeclasses::Apply;

pub trait FlatMap<F: HKT>: Apply<F> {
    fn flat_map<A, B, Func: Fn(A) -> F::Applied<B>>(
        fa: F::Applied<A>,
        f: Func,
    ) -> F::Applied<B>;
}

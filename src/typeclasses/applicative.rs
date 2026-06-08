use crate::core::HKT;
use crate::typeclasses::Apply;

pub trait Applicative<F: HKT>: Apply<F> {
    fn pure<A>(a: A) -> F::Applied<A>;
}

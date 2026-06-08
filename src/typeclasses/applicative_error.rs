use crate::core::HKT;
use crate::typeclasses::Applicative;

pub trait ApplicativeError<F: HKT, E>: Applicative<F> {
    fn raise_error<A>(e: E) -> F::Applied<A>;
    fn handle_error_with<A, Handler: Fn(E) -> F::Applied<A>>(
        fa: F::Applied<A>,
        f: Handler,
    ) -> F::Applied<A>;
}

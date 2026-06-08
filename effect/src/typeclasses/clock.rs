use rust_cats_core::core::HKT;
use rust_cats_core::typeclasses::Applicative;
use std::time::Duration;

pub trait Clock<F: HKT>: Applicative<F> {
    fn monotonic() -> F::Applied<Duration>;

    fn realtime() -> F::Applied<Duration>;
}

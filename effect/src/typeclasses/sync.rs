use crate::typeclasses::{Clock, MonadCancel, Unique};
use rust_cats_core::core::HKT;
use rust_cats_core::typeclasses::Defer;

pub trait Sync<F: HKT, E>: MonadCancel<F, E> + Clock<F> + Defer<F> + Unique<F, E> {
    fn delay<A: Send + 'static, Thunk: Fn() -> A + Send + std::marker::Sync + 'static>(
        thunk: Thunk,
    ) -> F::Applied<A>;
}

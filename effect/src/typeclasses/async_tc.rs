use crate::typeclasses::{Sync, Temporal};
use rust_cats_core::core::HKT;

pub trait AsyncCallback<A> {
    fn complete(self, result: Result<A, String>);
}

pub trait Async<F: HKT, E>: Sync<F, E> + Temporal<F, E> {
    fn async_io<A: 'static, K: Fn(Box<dyn Fn(Result<A, String>)>) -> F::Applied<Option<F::Applied<()>>>
        + 'static>(
        k: K,
    ) -> F::Applied<A>;

    fn execution_context() -> F::Applied<()>;
}

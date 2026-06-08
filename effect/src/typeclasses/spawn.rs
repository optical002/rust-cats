use crate::typeclasses::{MonadCancel, Unique};
use rust_cats_core::core::HKT;

pub enum Outcome<F: HKT, E, A> {
    Succeeded(F::Applied<A>),
    Errored(E),
    Canceled,
}

pub trait Fiber<F: HKT, E, A> {
    fn cancel(&self) -> F::Applied<()>;
    fn join(&self) -> F::Applied<Outcome<F, E, A>>;
}

pub trait Spawn<F: HKT, E>: MonadCancel<F, E> + Unique<F, E> {
    type FiberT<A: Clone + Send + Sync + 'static>: Fiber<F, E, A>;

    fn start<A: Clone + Send + Sync + 'static>(fa: F::Applied<A>) -> F::Applied<Self::FiberT<A>>;

    fn never<A: 'static>() -> F::Applied<A>;

    fn cede() -> F::Applied<()>;
}

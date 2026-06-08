use crate::typeclasses::Spawn;
use rust_cats_core::core::HKT;

pub trait Ref<F: HKT, A> {
    fn get(&self) -> F::Applied<A>;
    fn set(&self, a: A) -> F::Applied<()>;
    fn update<U: Fn(A) -> A + 'static>(&self, f: U) -> F::Applied<()>;
    fn modify<B: 'static, U: Fn(A) -> (A, B) + 'static>(&self, f: U) -> F::Applied<B>;
}

pub trait Deferred<F: HKT, A> {
    fn get(&self) -> F::Applied<A>;
    fn complete(&self, a: A) -> F::Applied<bool>;
    fn try_get(&self) -> F::Applied<Option<A>>;
}

pub trait Concurrent<F: HKT, E>: Spawn<F, E> {
    type RefT<A: Clone + Send + Sync + 'static>: Ref<F, A>;
    type DeferredT<A: Clone + Send + Sync + 'static>: Deferred<F, A>;

    fn ref_of<A: Clone + Send + Sync + 'static>(a: A) -> F::Applied<Self::RefT<A>>;

    fn deferred<A: Clone + Send + Sync + 'static>() -> F::Applied<Self::DeferredT<A>>;
}

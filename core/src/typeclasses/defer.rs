use crate::core::HKT;

pub trait Defer<F: HKT> {
    fn defer<A, Thunk: Fn() -> F::Applied<A> + 'static>(fa: Thunk) -> F::Applied<A>;
}

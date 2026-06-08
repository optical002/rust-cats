use rust_cats_core::core::HKT;
use rust_cats_core::typeclasses::MonadError;

pub trait Poll<F: HKT> {
    fn apply<A: 'static>(&self, fa: F::Applied<A>) -> F::Applied<A>;
}

pub trait MonadCancel<F: HKT, E>: MonadError<F, E> {
    fn uncancelable<A: 'static, P: Poll<F>, Body: Fn(&P) -> F::Applied<A>>(
        body: Body,
    ) -> F::Applied<A>;

    fn canceled() -> F::Applied<()>;

    fn on_cancel<A: 'static>(fa: F::Applied<A>, fin: F::Applied<()>) -> F::Applied<A>;

    fn force_r<A: 'static, B: 'static>(fa: F::Applied<A>, fb: F::Applied<B>) -> F::Applied<B>;
}

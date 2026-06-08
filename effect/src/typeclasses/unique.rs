use crate::typeclasses::MonadCancel;
use rust_cats_core::core::HKT;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Token(pub u64);

pub trait Unique<F: HKT, E>: MonadCancel<F, E> {
    fn unique() -> F::Applied<Token>;
}

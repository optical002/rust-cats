use crate::core::HKT;
use crate::typeclasses::{ApplicativeError, Monad};

pub trait MonadError<F: HKT, E>: ApplicativeError<F, E> + Monad<F> {}

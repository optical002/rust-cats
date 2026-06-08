use crate::core::HKT;
use crate::typeclasses::{Applicative, FlatMap};

pub trait Monad<F: HKT>: Applicative<F> + FlatMap<F> {}

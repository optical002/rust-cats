use crate::typeclasses::{Clock, Concurrent};
use rust_cats_core::core::HKT;
use std::time::Duration;

pub trait Temporal<F: HKT, E>: Concurrent<F, E> + Clock<F> {
    fn sleep(d: Duration) -> F::Applied<()>;
}

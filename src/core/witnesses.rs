use crate::core::HKT;
use std::marker::PhantomData;

pub struct OptionH;
impl HKT for OptionH {
    type Applied<A> = Option<A>;
}

pub struct ResultH<E>(PhantomData<E>);
impl<E> HKT for ResultH<E> {
    type Applied<A> = Result<A, E>;
}

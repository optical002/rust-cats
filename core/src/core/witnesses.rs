use crate::core::HKT;
use crate::data::eval::Eval;
use std::marker::PhantomData;

pub struct OptionH;
impl HKT for OptionH {
    type Applied<A> = Option<A>;
}

pub struct ResultH<E>(PhantomData<E>);
impl<E> HKT for ResultH<E> {
    type Applied<A> = Result<A, E>;
}

pub struct EvalH;
impl HKT for EvalH {
    type Applied<A> = Eval<A>;
}

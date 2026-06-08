use crate::data::io::IO;
use rust_cats_core::core::HKT;

pub struct IoH;
impl HKT for IoH {
    type Applied<A> = IO<A>;
}

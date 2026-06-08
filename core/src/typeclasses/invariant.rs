use crate::core::HKT;

pub trait Invariant<F: HKT> {
    fn imap<A, B, Fwd: Fn(A) -> B, Bwd: Fn(B) -> A>(
        fa: F::Applied<A>,
        f: Fwd,
        g: Bwd,
    ) -> F::Applied<B>;
}

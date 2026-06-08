use crate::core::EvalH;
use crate::data::eval::Eval;
use crate::typeclasses::{Applicative, Apply, Defer, FlatMap, Functor};

pub trait EvalExt<A> {
    fn fmap<B, F: Fn(&A) -> B>(self, f: F) -> Eval<B>;
    fn ap<B, F: Fn(&A) -> B>(self, ff: Eval<F>) -> Eval<B>;
    fn flat_map<B, F: Fn(A) -> Eval<B>>(self, f: F) -> Eval<B>;
}

impl<A> EvalExt<A> for Eval<A> {
    fn fmap<B, F: Fn(&A) -> B>(self, f: F) -> Eval<B> {
        <EvalH as Functor<EvalH>>::fmap(self, f)
    }

    fn ap<B, F: Fn(&A) -> B>(self, ff: Eval<F>) -> Eval<B> {
        <EvalH as Apply<EvalH>>::ap(self, ff)
    }

    fn flat_map<B, F: Fn(A) -> Eval<B>>(self, f: F) -> Eval<B> {
        <EvalH as FlatMap<EvalH>>::flat_map(self, f)
    }
}

impl EvalH {
    pub fn pure<A>(a: A) -> Eval<A> {
        <EvalH as Applicative<EvalH>>::pure(a)
    }

    pub fn defer<A, Thunk: Fn() -> Eval<A> + 'static>(thunk: Thunk) -> Eval<A> {
        <EvalH as Defer<EvalH>>::defer(thunk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmap_now() {
        let e = Eval::now(1);
        let mapped = e.fmap(|a| a + 10);
        assert_eq!(mapped.value(), 11);
    }

    #[test]
    fn test_fmap_later() {
        let e = Eval::later(|| 1);
        let mapped = e.fmap(|a| a + 10);
        assert_eq!(mapped.value(), 11);
    }

    #[test]
    fn test_fmap_always() {
        let e = Eval::always(|| 2);
        let mapped = e.fmap(|a| a * 3);
        assert_eq!(mapped.value(), 6);
    }

    #[test]
    fn test_apply() {
        let fa = Eval::now(1);
        let ff: Eval<fn(&i32) -> i32> = Eval::now(|a| a + 100);
        let applied = fa.ap(ff);
        assert_eq!(applied.value(), 101);
    }

    #[test]
    fn test_pure() {
        let pured: Eval<i32> = EvalH::pure(42);
        assert_eq!(pured.value(), 42);
    }

    #[test]
    fn test_flat_map_chain() {
        let e = Eval::now(1)
            .flat_map(|a| Eval::now(a + 1))
            .flat_map(|a| Eval::now(a * 10));
        assert_eq!(e.value(), 20);
    }

    #[test]
    fn test_monad_pure_then_flat_map() {
        let e = EvalH::pure(1).flat_map(|a| Eval::now(a + 1));
        assert_eq!(e.value(), 2);
    }

    #[test]
    fn test_defer_is_lazy() {
        use std::cell::Cell;
        use std::rc::Rc;
        let counter = Rc::new(Cell::new(0));
        let c = counter.clone();
        let deferred = EvalH::defer(move || {
            c.set(c.get() + 1);
            Eval::now(42)
        });
        assert_eq!(counter.get(), 0);
        assert_eq!(deferred.value(), 42);
        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn test_defer_recomputes() {
        use std::cell::Cell;
        use std::rc::Rc;
        let counter = Rc::new(Cell::new(0));
        let c = counter.clone();
        let deferred = EvalH::defer(move || {
            c.set(c.get() + 1);
            Eval::now(1)
        });
        let _ = deferred.value();
        let _ = deferred.value();
        let _ = deferred.value();
        assert_eq!(counter.get(), 3);
    }
}

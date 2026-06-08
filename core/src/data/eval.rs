use std::cell::OnceCell;

pub enum Eval<A> {
    Now(A),
    Later(OnceCell<A>, Box<dyn Fn() -> A>),
    Always(Box<dyn Fn() -> A>),
}

impl<A> Eval<A> {
    pub fn now(a: A) -> Self {
        Eval::Now(a)
    }

    pub fn later<F: Fn() -> A + 'static>(f: F) -> Self {
        Eval::Later(OnceCell::new(), Box::new(f))
    }

    pub fn always<F: Fn() -> A + 'static>(f: F) -> Self {
        Eval::Always(Box::new(f))
    }
}

impl<A: Clone> Eval<A> {
    pub fn value(&self) -> A {
        match self {
            Eval::Now(a) => a.clone(),
            Eval::Later(cell, f) => cell.get_or_init(f).clone(),
            Eval::Always(f) => f(),
        }
    }
}

impl<A> Eval<A> {
    pub fn into_value(self) -> A {
        match self {
            Eval::Now(a) => a,
            Eval::Later(_, f) => f(),
            Eval::Always(f) => f(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn test_now_returns_value() {
        let e = Eval::now(42);
        assert_eq!(e.value(), 42);
    }

    #[test]
    fn test_now_is_eager() {
        let counter = Rc::new(Cell::new(0));
        let c = counter.clone();
        let _ = Eval::now({
            c.set(c.get() + 1);
            1
        });
        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn test_later_is_lazy_and_memoized() {
        let counter = Rc::new(Cell::new(0));
        let c = counter.clone();
        let e = Eval::later(move || {
            c.set(c.get() + 1);
            7
        });
        assert_eq!(counter.get(), 0);
        assert_eq!(e.value(), 7);
        assert_eq!(e.value(), 7);
        assert_eq!(e.value(), 7);
        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn test_always_is_lazy_and_recomputed() {
        let counter = Rc::new(Cell::new(0));
        let c = counter.clone();
        let e = Eval::always(move || {
            c.set(c.get() + 1);
            9
        });
        assert_eq!(counter.get(), 0);
        assert_eq!(e.value(), 9);
        assert_eq!(e.value(), 9);
        assert_eq!(e.value(), 9);
        assert_eq!(counter.get(), 3);
    }
}

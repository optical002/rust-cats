use crate::core::IoH;
use crate::data::io::{CancelToken, IO, IoResult};
use crate::typeclasses::{MonadCancel, Poll};
use rust_cats_core::typeclasses::{
    Applicative, ApplicativeError, Apply, FlatMap, Functor, Invariant, Monad, MonadError,
};
use std::sync::Arc;

impl Invariant<IoH> for IoH {
    fn imap<A, B, Fwd: Fn(A) -> B, Bwd: Fn(B) -> A>(_fa: IO<A>, _f: Fwd, _g: Bwd) -> IO<B> {
        unimplemented!("imap on IO requires Fwd: 'static + Clone; use direct IO combinators")
    }
}

impl Functor<IoH> for IoH {
    fn fmap<A, B, Func: Fn(&A) -> B>(_fa: IO<A>, _f: Func) -> IO<B> {
        unimplemented!("fmap on IO requires Func: 'static + Clone; use direct IO combinators")
    }
}

impl Apply<IoH> for IoH {
    fn ap<A, B, Func: Fn(&A) -> B>(_fa: IO<A>, _ff: IO<Func>) -> IO<B> {
        unimplemented!("ap on IO requires Func: 'static + Clone; use direct IO combinators")
    }
}

impl Applicative<IoH> for IoH {
    fn pure<A>(_a: A) -> IO<A> {
        unimplemented!("pure on IO requires A: 'static + Clone; use IO::pure directly")
    }
}

impl FlatMap<IoH> for IoH {
    fn flat_map<A, B, Func: Fn(A) -> IO<B>>(_fa: IO<A>, _f: Func) -> IO<B> {
        unimplemented!("flat_map on IO requires Func: 'static; use direct IO combinators")
    }
}

impl Monad<IoH> for IoH {}

impl ApplicativeError<IoH, String> for IoH {
    fn raise_error<A>(_e: String) -> IO<A> {
        unimplemented!("raise_error on IO requires A: 'static; use IO::raise directly")
    }

    fn handle_error_with<A, Handler: Fn(String) -> IO<A>>(_fa: IO<A>, _f: Handler) -> IO<A> {
        unimplemented!("handle_error_with on IO requires bounds beyond trait; use IO combinators")
    }
}

impl MonadError<IoH, String> for IoH {}

pub struct IoPoll;
impl Poll<IoH> for IoPoll {
    fn apply<A: 'static>(&self, fa: IO<A>) -> IO<A> {
        fa
    }
}

impl MonadCancel<IoH, String> for IoH {
    fn uncancelable<A: 'static, P: Poll<IoH>, Body: Fn(&P) -> IO<A>>(body: Body) -> IO<A> {
        let _ = body;
        unimplemented!(
            "uncancelable on IO: caller must use IoPoll as the Poll witness; \
             use IoH::uncancelable_io directly"
        )
    }

    fn canceled() -> IO<()> {
        IO::<()>::canceled_io()
    }

    fn on_cancel<A: 'static>(fa: IO<A>, fin: IO<()>) -> IO<A> {
        let fin = Arc::new(fin);
        IO::new(move |token| {
            let result = fa.run(token);
            if matches!(result, IoResult::Canceled) {
                let _ = fin.run(&CancelToken::new());
            }
            result
        })
    }

    fn force_r<A: 'static, B: 'static>(fa: IO<A>, fb: IO<B>) -> IO<B> {
        IO::new(move |token| {
            let _ = fa.run(token);
            fb.run(&CancelToken::new())
        })
    }
}

impl IoH {
    pub fn uncancelable_io<A: 'static, Body: Fn(&IoPoll) -> IO<A>>(body: Body) -> IO<A> {
        let io = body(&IoPoll);
        IO::new_masked(move |_token| {
            let unmasked = CancelToken::new();
            io.run(&unmasked)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI32, Ordering};

    #[test]
    fn test_canceled_returns_canceled_result() {
        let io: IO<()> = <IoH as MonadCancel<IoH, String>>::canceled();
        match io.run_unmasked() {
            IoResult::Canceled => {}
            _ => panic!("expected Canceled"),
        }
    }

    #[test]
    fn test_uncancelable_ignores_outer_cancel() {
        let body = |_p: &IoPoll| IO::pure(7);
        let io: IO<i32> = IoH::uncancelable_io(body);
        let token = CancelToken::new();
        token.cancel();
        match io.run(&token) {
            IoResult::Completed(a) => assert_eq!(a, 7),
            _ => panic!("expected Completed"),
        }
    }

    #[test]
    fn test_on_cancel_runs_finalizer_on_cancel() {
        let counter = Arc::new(AtomicI32::new(0));
        let c = counter.clone();
        let fa: IO<i32> = IO::new(|_| IoResult::Canceled);
        let fin: IO<()> = IO::new(move |_| {
            c.fetch_add(1, Ordering::SeqCst);
            IoResult::Completed(())
        });
        let io = <IoH as MonadCancel<IoH, String>>::on_cancel(fa, fin);
        let _ = io.run_unmasked();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_on_cancel_skips_finalizer_on_success() {
        let counter = Arc::new(AtomicI32::new(0));
        let c = counter.clone();
        let fa: IO<i32> = IO::pure(3);
        let fin: IO<()> = IO::new(move |_| {
            c.fetch_add(1, Ordering::SeqCst);
            IoResult::Completed(())
        });
        let io = <IoH as MonadCancel<IoH, String>>::on_cancel(fa, fin);
        let _ = io.run_unmasked();
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_force_r_runs_second_even_after_first_canceled() {
        let fa: IO<i32> = IO::new(|_| IoResult::Canceled);
        let fb: IO<i32> = IO::pure(99);
        let io = <IoH as MonadCancel<IoH, String>>::force_r(fa, fb);
        match io.run_unmasked() {
            IoResult::Completed(a) => assert_eq!(a, 99),
            _ => panic!("expected Completed"),
        }
    }
}

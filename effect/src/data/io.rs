use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Default)]
pub struct CancelToken {
    flag: Arc<AtomicBool>,
}

impl CancelToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    pub fn is_canceled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
}

pub enum IoResult<A> {
    Completed(A),
    Canceled,
    Errored(String),
}

pub struct IO<A> {
    run: Box<dyn Fn(&CancelToken) -> IoResult<A> + Send + Sync>,
    masked: bool,
}

impl<A: 'static> IO<A> {
    pub fn new<F: Fn(&CancelToken) -> IoResult<A> + Send + Sync + 'static>(f: F) -> Self {
        IO {
            run: Box::new(f),
            masked: false,
        }
    }

    pub fn new_masked<F: Fn(&CancelToken) -> IoResult<A> + Send + Sync + 'static>(f: F) -> Self {
        IO {
            run: Box::new(f),
            masked: true,
        }
    }

    pub fn pure(a: A) -> Self
    where
        A: Clone + Send + Sync,
    {
        IO::new(move |_| IoResult::Completed(a.clone()))
    }

    pub fn delay<F>(thunk: F) -> Self
    where
        F: Fn() -> A + Send + ::std::marker::Sync + 'static,
    {
        IO::new(move |_| IoResult::Completed(thunk()))
    }

    pub fn raise(msg: String) -> Self {
        IO::new(move |_| IoResult::Errored(msg.clone()))
    }

    pub fn canceled_io() -> IO<()> {
        IO::new(|_| IoResult::Canceled)
    }

    pub fn run(&self, token: &CancelToken) -> IoResult<A> {
        if !self.masked && token.is_canceled() {
            return IoResult::Canceled;
        }
        (self.run)(token)
    }

    pub fn run_unmasked(&self) -> IoResult<A> {
        let token = CancelToken::new();
        self.run(&token)
    }

    pub fn map<B, F>(self, f: F) -> IO<B>
    where
        A: Send + Sync,
        B: 'static,
        F: Fn(A) -> B + Send + Sync + 'static,
    {
        IO::new(move |token| match self.run(token) {
            IoResult::Completed(a) => IoResult::Completed(f(a)),
            IoResult::Errored(e) => IoResult::Errored(e),
            IoResult::Canceled => IoResult::Canceled,
        })
    }

    pub fn flat_map<B, F>(self, f: F) -> IO<B>
    where
        A: Send + Sync,
        B: 'static,
        F: Fn(A) -> IO<B> + Send + Sync + 'static,
    {
        IO::new(move |token| match self.run(token) {
            IoResult::Completed(a) => f(a).run(token),
            IoResult::Errored(e) => IoResult::Errored(e),
            IoResult::Canceled => IoResult::Canceled,
        })
    }

    pub fn as_unit(self) -> IO<()>
    where
        A: Send + Sync,
    {
        self.map(|_| ())
    }

}

impl IO<()> {
    pub fn println(s: String) -> IO<()> {
        IO::new(move |_| {
            println!("{}", s);
            IoResult::Completed(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_completes() {
        let io = IO::pure(42);
        match io.run_unmasked() {
            IoResult::Completed(a) => assert_eq!(a, 42),
            _ => panic!("expected Completed"),
        }
    }

    #[test]
    fn test_canceled_token_short_circuits() {
        let io = IO::pure(1);
        let token = CancelToken::new();
        token.cancel();
        match io.run(&token) {
            IoResult::Canceled => {}
            _ => panic!("expected Canceled"),
        }
    }

    #[test]
    fn test_canceled_io_returns_canceled() {
        let io = IO::<()>::canceled_io();
        match io.run_unmasked() {
            IoResult::Canceled => {}
            _ => panic!("expected Canceled"),
        }
    }

    #[test]
    fn test_raise_errored() {
        let io: IO<i32> = IO::raise("boom".to_string());
        match io.run_unmasked() {
            IoResult::Errored(e) => assert_eq!(e, "boom"),
            _ => panic!("expected Errored"),
        }
    }
}

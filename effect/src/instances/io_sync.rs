use crate::core::IoH;
use crate::data::io::{IO, IoResult};
use crate::typeclasses::Sync;

impl Sync<IoH, String> for IoH {
    fn delay<A: Send + 'static, Thunk: Fn() -> A + Send + std::marker::Sync + 'static>(
        thunk: Thunk,
    ) -> IO<A> {
        IO::new(move |_| IoResult::Completed(thunk()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::runtime::unsafe_run_sync;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicI32, Ordering};

    #[test]
    fn test_delay_runs_thunk_on_run() {
        let counter = Arc::new(AtomicI32::new(0));
        let c = counter.clone();
        let io = <IoH as Sync<IoH, String>>::delay(move || {
            c.fetch_add(1, Ordering::SeqCst);
            c.load(Ordering::SeqCst)
        });
        assert_eq!(counter.load(Ordering::SeqCst), 0);
        match unsafe_run_sync(io) {
            IoResult::Completed(v) => assert_eq!(v, 1),
            _ => panic!("expected Completed"),
        }
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}

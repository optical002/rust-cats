use crate::core::IoH;
use crate::data::fiber::{FiberState, IoFiber};
use crate::data::io::{CancelToken, IO, IoResult};
use crate::data::runtime;
use crate::typeclasses::Spawn;
use std::sync::Arc;

impl Spawn<IoH, String> for IoH {
    type FiberT<A: Clone + Send + Sync + 'static> = IoFiber<A>;

    fn start<A: Clone + Send + Sync + 'static>(fa: IO<A>) -> IO<IoFiber<A>> {
        IoH::start_io(fa)
    }

    fn never<A: 'static>() -> IO<A> {
        IO::new(|token| {
            // Park on a condvar; check cancellation periodically.
            loop {
                if token.is_canceled() {
                    return IoResult::Canceled;
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        })
    }

    fn cede() -> IO<()> {
        IO::new(|_| {
            std::thread::yield_now();
            IoResult::Completed(())
        })
    }
}

impl IoH {
    pub fn start_io<A: Clone + Send + Sync + 'static>(fa: IO<A>) -> IO<IoFiber<A>> {
        let fa_arc = Arc::new(fa);
        IO::new(move |_outer_token| {
            let token = CancelToken::new();
            let fiber: IoFiber<A> = IoFiber::new(token.clone());
            let state = fiber.state.clone();
            let joiners = fiber.joiners.clone();
            let fa_for_task = fa_arc.clone();

            let token_for_task = token.clone();
            runtime::enqueue(Box::new(move || {
                let r = fa_for_task.run(&token_for_task);
                let next = match r {
                    IoResult::Completed(a) => FiberState::Succeeded(a),
                    IoResult::Errored(e) => FiberState::Errored(e),
                    IoResult::Canceled => FiberState::Canceled,
                };
                *state.lock().unwrap() = next;
                let drained: Vec<_> = std::mem::take(&mut *joiners.lock().unwrap());
                for j in drained {
                    runtime::enqueue(j);
                }
            }));

            IoResult::Completed(fiber)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::runtime::unsafe_run_sync;
    use crate::typeclasses::{Fiber as FiberTC, Outcome};

    #[test]
    fn test_start_then_join_succeeded() {
        let prog: IO<i32> = IO::new(|_| {
            let f = IoH::start_io(IO::pure(42));
            match f.run_unmasked() {
                IoResult::Completed(fiber) => match fiber.join().run_unmasked() {
                    IoResult::Completed(Outcome::Succeeded(inner)) => inner.run_unmasked(),
                    _ => IoResult::Errored("expected Succeeded".to_string()),
                },
                _ => IoResult::Errored("start failed".to_string()),
            }
        });
        match unsafe_run_sync(prog) {
            IoResult::Completed(v) => assert_eq!(v, 42),
            _ => panic!("expected Completed"),
        }
    }

    #[test]
    fn test_fiber_cancel_yields_canceled_outcome() {
        let prog: IO<()> = IO::new(|_| {
            let f = IoH::start_io::<i32>(<IoH as Spawn<IoH, String>>::never::<i32>());
            match f.run_unmasked() {
                IoResult::Completed(fiber) => {
                    let _ = fiber.cancel().run_unmasked();
                    match fiber.join().run_unmasked() {
                        IoResult::Completed(Outcome::Canceled) => IoResult::Completed(()),
                        _ => IoResult::Errored("expected Canceled outcome".to_string()),
                    }
                }
                _ => IoResult::Errored("start failed".to_string()),
            }
        });
        match unsafe_run_sync(prog) {
            IoResult::Completed(()) => {}
            IoResult::Errored(e) => panic!("err: {}", e),
            _ => panic!("expected Completed(())"),
        }
    }
}

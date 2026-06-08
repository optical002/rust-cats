use crate::core::IoH;
use crate::data::io::{IO, IoResult};
use crate::typeclasses::{Fiber as FiberTC, Outcome, Spawn};
use std::sync::{Arc, Mutex};

/// Scala's `parTraverse`: start one fiber per input, then join them all and
/// collect results in order.
pub fn par_traverse<T, A, F>(items: Vec<T>, f: F) -> IO<Vec<A>>
where
    T: Send + Sync + 'static,
    A: Clone + Send + Sync + 'static,
    F: Fn(T) -> IO<A> + Send + Sync + 'static,
{
    // `items` and `f` are captured by a `Fn` closure (IO::new requires Fn), so
    // we wrap `items` in an Arc<Mutex<Option>> to allow take-on-first-run.
    let slot: Arc<Mutex<Option<Vec<T>>>> = Arc::new(Mutex::new(Some(items)));
    let f = Arc::new(f);
    IO::new(move |_| {
        let items = match slot.lock().unwrap().take() {
            Some(v) => v,
            None => return IoResult::Errored("par_traverse run more than once".to_string()),
        };
        let mut fibers = Vec::with_capacity(items.len());
        for item in items {
            let io = (f)(item);
            let start = <IoH as Spawn<IoH, String>>::start::<A>(io);
            match start.run_unmasked() {
                IoResult::Completed(fiber) => fibers.push(fiber),
                IoResult::Errored(e) => return IoResult::Errored(e),
                IoResult::Canceled => return IoResult::Canceled,
            }
        }
        let mut out = Vec::with_capacity(fibers.len());
        for fiber in &fibers {
            match fiber.join().run_unmasked() {
                IoResult::Completed(Outcome::Succeeded(inner)) => match inner.run_unmasked() {
                    IoResult::Completed(a) => out.push(a),
                    IoResult::Errored(e) => return IoResult::Errored(e),
                    IoResult::Canceled => return IoResult::Canceled,
                },
                IoResult::Completed(Outcome::Errored(e)) => return IoResult::Errored(e),
                IoResult::Completed(Outcome::Canceled) => return IoResult::Canceled,
                IoResult::Errored(e) => return IoResult::Errored(e),
                IoResult::Canceled => return IoResult::Canceled,
            }
        }
        IoResult::Completed(out)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::runtime::unsafe_run_sync;

    #[test]
    fn test_par_traverse_collects_results() {
        let items: Vec<i32> = (1..=10).collect();
        let prog = par_traverse(items, |i| IO::pure(i * 2));
        match unsafe_run_sync(prog) {
            IoResult::Completed(v) => {
                assert_eq!(v, (1..=10).map(|i| i * 2).collect::<Vec<_>>());
            }
            _ => panic!("expected Completed"),
        }
    }
}

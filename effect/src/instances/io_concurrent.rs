use crate::core::IoH;
use crate::data::deferred::IoDeferred;
use crate::data::io::{IO, IoResult};
use crate::data::io_ref::IoRef;
use crate::typeclasses::Concurrent;
use std::sync::{Arc, Mutex};

impl Concurrent<IoH, String> for IoH {
    type RefT<A: Clone + Send + Sync + 'static> = IoRef<A>;
    type DeferredT<A: Clone + Send + Sync + 'static> = IoDeferred<A>;

    fn ref_of<A: Clone + Send + Sync + 'static>(a: A) -> IO<IoRef<A>> {
        IoH::ref_of_io(a)
    }

    fn deferred<A: Clone + Send + Sync + 'static>() -> IO<IoDeferred<A>> {
        IoH::deferred_io()
    }
}

impl IoH {
    pub fn ref_of_io<A: Clone + Send + Sync + 'static>(a: A) -> IO<IoRef<A>> {
        let a = Arc::new(a);
        IO::new(move |_| {
            IoResult::Completed(IoRef {
                cell: Arc::new(Mutex::new((*a).clone())),
            })
        })
    }

    pub fn deferred_io<A: Clone + Send + Sync + 'static>() -> IO<IoDeferred<A>> {
        IO::new(|_| IoResult::Completed(IoDeferred::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::runtime::unsafe_run_sync;
    use crate::typeclasses::Deferred;

    #[test]
    fn test_ref_get_set() {
        use crate::typeclasses::Ref;
        let prog: IO<i32> = IO::new(|_| {
            let r = match IoH::ref_of_io(0i32).run_unmasked() {
                IoResult::Completed(r) => r,
                _ => return IoResult::Errored("ref_of failed".to_string()),
            };
            let _ = r.set(7).run_unmasked();
            r.get().run_unmasked()
        });
        match unsafe_run_sync(prog) {
            IoResult::Completed(v) => assert_eq!(v, 7),
            _ => panic!("expected Completed"),
        }
    }

    #[test]
    fn test_ref_update_io_and_modify_io() {
        use crate::typeclasses::Ref;
        let prog: IO<(i32, i32)> = IO::new(|_| {
            let r = match IoH::ref_of_io(10i32).run_unmasked() {
                IoResult::Completed(r) => r,
                _ => return IoResult::Errored("ref_of failed".to_string()),
            };
            let _ = r.update_io(|x| x + 1).run_unmasked();
            let out = match r.modify_io(|x| (x * 2, x)).run_unmasked() {
                IoResult::Completed(v) => v,
                _ => return IoResult::Errored("modify failed".to_string()),
            };
            match r.get().run_unmasked() {
                IoResult::Completed(cur) => IoResult::Completed((out, cur)),
                _ => IoResult::Errored("get failed".to_string()),
            }
        });
        match unsafe_run_sync(prog) {
            IoResult::Completed((returned, current)) => {
                assert_eq!(returned, 11);
                assert_eq!(current, 22);
            }
            _ => panic!("expected Completed"),
        }
    }

    #[test]
    fn test_deferred_complete_and_try_get() {
        let prog: IO<(bool, bool, Option<i32>)> = IO::new(|_| {
            let d = match IoH::deferred_io::<i32>().run_unmasked() {
                IoResult::Completed(d) => d,
                _ => return IoResult::Errored("deferred failed".to_string()),
            };
            let first = match d.complete(5).run_unmasked() {
                IoResult::Completed(b) => b,
                _ => return IoResult::Errored("complete1 failed".to_string()),
            };
            let second = match d.complete(6).run_unmasked() {
                IoResult::Completed(b) => b,
                _ => return IoResult::Errored("complete2 failed".to_string()),
            };
            match d.try_get().run_unmasked() {
                IoResult::Completed(v) => IoResult::Completed((first, second, v)),
                _ => IoResult::Errored("try_get failed".to_string()),
            }
        });
        match unsafe_run_sync(prog) {
            IoResult::Completed((first, second, v)) => {
                assert!(first);
                assert!(!second);
                assert_eq!(v, Some(5));
            }
            _ => panic!("expected Completed"),
        }
    }
}

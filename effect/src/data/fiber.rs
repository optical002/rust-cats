use crate::core::IoH;
use crate::data::io::{CancelToken, IO, IoResult};
use crate::data::runtime::{self, ParkSlot};
use crate::typeclasses::{Fiber as FiberTC, Outcome};
use std::sync::{Arc, Mutex};

pub enum FiberState<A> {
    Pending,
    Succeeded(A),
    Errored(String),
    Canceled,
}

pub struct IoFiber<A> {
    pub(crate) state: Arc<Mutex<FiberState<A>>>,
    pub(crate) token: CancelToken,
    pub(crate) joiners: Arc<Mutex<Vec<Box<dyn FnOnce() + Send>>>>,
}

impl<A: Clone + Send + Sync + 'static> IoFiber<A> {
    pub(crate) fn new(token: CancelToken) -> Self {
        IoFiber {
            state: Arc::new(Mutex::new(FiberState::Pending)),
            token,
            joiners: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl<A: Clone + Send + Sync + 'static> FiberTC<IoH, String, A> for IoFiber<A> {
    fn cancel(&self) -> IO<()> {
        let token = self.token.clone();
        let state = self.state.clone();
        let joiners = self.joiners.clone();
        IO::new(move |_| {
            token.cancel();
            let mut s = state.lock().unwrap();
            let is_pending = matches!(*s, FiberState::Pending);
            if is_pending {
                *s = FiberState::Canceled;
                drop(s);
                let drained: Vec<_> = std::mem::take(&mut *joiners.lock().unwrap());
                for j in drained {
                    runtime::enqueue(j);
                }
            }
            IoResult::Completed(())
        })
    }

    fn join(&self) -> IO<Outcome<IoH, String, A>> {
        let state = self.state.clone();
        let joiners = self.joiners.clone();
        IO::new(move |_| {
            // Snapshot fast path.
            {
                let s = state.lock().unwrap();
                match &*s {
                    FiberState::Succeeded(a) => {
                        return IoResult::Completed(Outcome::Succeeded(IO::pure(a.clone())));
                    }
                    FiberState::Errored(e) => {
                        return IoResult::Completed(Outcome::Errored(e.clone()));
                    }
                    FiberState::Canceled => {
                        return IoResult::Completed(Outcome::<IoH, String, A>::Canceled);
                    }
                    FiberState::Pending => {}
                }
            }

            let park = Arc::new(ParkSlot::new());
            let park_for_waker = park.clone();
            joiners.lock().unwrap().push(Box::new(move || {
                park_for_waker.signal();
            }));
            // Re-check after registering, in case completion happened between
            // our snapshot and the joiner push.
            {
                let s = state.lock().unwrap();
                let already = !matches!(*s, FiberState::Pending);
                if already {
                    // Pop our joiner — easier to just signal park and skip waiting.
                    drop(s);
                    park.signal();
                }
            }
            runtime::block_until(park);

            let s = state.lock().unwrap();
            match &*s {
                FiberState::Succeeded(a) => {
                    IoResult::Completed(Outcome::Succeeded(IO::pure(a.clone())))
                }
                FiberState::Errored(e) => IoResult::Completed(Outcome::Errored(e.clone())),
                FiberState::Canceled => IoResult::Completed(Outcome::<IoH, String, A>::Canceled),
                FiberState::Pending => {
                    // Spurious wake; shouldn't happen with our signaling, but be safe.
                    IoResult::Errored("join woke without completion".to_string())
                }
            }
        })
    }
}

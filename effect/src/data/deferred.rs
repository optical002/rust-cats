use crate::core::IoH;
use crate::data::io::{IO, IoResult};
use crate::data::runtime::{self, ParkSlot};
use crate::typeclasses::Deferred;
use std::sync::{Arc, Mutex};

pub struct IoDeferred<A> {
    pub(crate) slot: Arc<Mutex<Option<A>>>,
    pub(crate) waiters: Arc<Mutex<Vec<Box<dyn FnOnce() + Send>>>>,
}

impl<A: Clone + Send + Sync + 'static> IoDeferred<A> {
    pub fn new() -> Self {
        IoDeferred {
            slot: Arc::new(Mutex::new(None)),
            waiters: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl<A: Clone + Send + Sync + 'static> Default for IoDeferred<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Clone + Send + Sync + 'static> Deferred<IoH, A> for IoDeferred<A> {
    fn get(&self) -> IO<A> {
        let slot = self.slot.clone();
        let waiters = self.waiters.clone();
        IO::new(move |_| {
            if let Some(a) = slot.lock().unwrap().as_ref() {
                return IoResult::Completed(a.clone());
            }
            let park = Arc::new(ParkSlot::new());
            let park_for_waker = park.clone();
            waiters.lock().unwrap().push(Box::new(move || {
                park_for_waker.signal();
            }));
            // Re-check after registering.
            if slot.lock().unwrap().is_some() {
                park.signal();
            }
            runtime::block_until(park);
            let g = slot.lock().unwrap();
            match g.as_ref() {
                Some(a) => IoResult::Completed(a.clone()),
                None => IoResult::Errored("deferred get woke without completion".to_string()),
            }
        })
    }

    fn complete(&self, a: A) -> IO<bool> {
        let slot = self.slot.clone();
        let waiters = self.waiters.clone();
        let a = Arc::new(a);
        IO::new(move |_| {
            let mut g = slot.lock().unwrap();
            if g.is_some() {
                return IoResult::Completed(false);
            }
            *g = Some((*a).clone());
            drop(g);
            let drained: Vec<_> = std::mem::take(&mut *waiters.lock().unwrap());
            for w in drained {
                runtime::enqueue(w);
            }
            IoResult::Completed(true)
        })
    }

    fn try_get(&self) -> IO<Option<A>> {
        let slot = self.slot.clone();
        IO::new(move |_| IoResult::Completed(slot.lock().unwrap().clone()))
    }
}

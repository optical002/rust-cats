use crate::core::IoH;
use crate::data::io::{IO, IoResult};
use crate::data::runtime::{self, ParkSlot};
use crate::typeclasses::Async;
use std::sync::{Arc, Mutex};

impl Async<IoH, String> for IoH {
    fn async_io<
        A: 'static,
        K: Fn(Box<dyn Fn(Result<A, String>)>) -> IO<Option<IO<()>>> + 'static,
    >(
        k: K,
    ) -> IO<A> {
        let _ = k;
        unimplemented!(
            "Async::async_io on IO requires A: Send + Sync, K: Send + Sync; \
             use IoH::async_io_send directly"
        )
    }

    fn execution_context() -> IO<()> {
        IO::new(|_| IoResult::Completed(()))
    }
}

impl IoH {
    pub fn async_io_send<
        A: Send + 'static,
        K: Fn(Box<dyn Fn(Result<A, String>) + Send + Sync>) -> IO<Option<IO<()>>>
            + Send
            + Sync
            + 'static,
    >(
        k: K,
    ) -> IO<A> {
        IO::new(move |_token| {
            let slot: Arc<Mutex<Option<Result<A, String>>>> = Arc::new(Mutex::new(None));
            let park = Arc::new(ParkSlot::new());

            let slot_for_cb = slot.clone();
            let park_for_cb = park.clone();
            let callback: Box<dyn Fn(Result<A, String>) + Send + Sync> = Box::new(move |r| {
                let mut g = slot_for_cb.lock().unwrap();
                if g.is_none() {
                    *g = Some(r);
                    drop(g);
                    park_for_cb.signal();
                }
            });

            let _finalizer_io = k(callback);
            let _ = _finalizer_io.run_unmasked();

            runtime::block_until(park);

            match slot.lock().unwrap().take() {
                Some(Ok(a)) => IoResult::Completed(a),
                Some(Err(e)) => IoResult::Errored(e),
                None => IoResult::Canceled,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::runtime::unsafe_run_sync;

    #[test]
    fn test_async_io_completes_via_callback() {
        let io: IO<i32> = IoH::async_io_send(|cb| {
            runtime::enqueue(Box::new(move || cb(Ok(123))));
            IO::new(|_| IoResult::Completed(None))
        });
        match unsafe_run_sync(io) {
            IoResult::Completed(v) => assert_eq!(v, 123),
            _ => panic!("expected Completed"),
        }
    }
}

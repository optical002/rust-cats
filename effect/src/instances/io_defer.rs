use crate::core::IoH;
use crate::data::io::IO;
use rust_cats_core::typeclasses::Defer;

impl Defer<IoH> for IoH {
    fn defer<A, Thunk: Fn() -> IO<A> + 'static>(_thunk: Thunk) -> IO<A> {
        unimplemented!(
            "Defer::defer on IO requires A: 'static + Send and Thunk: Send + Sync; \
             use IoH::defer_io directly"
        )
    }
}

impl IoH {
    pub fn defer_io<A: 'static, Thunk: Fn() -> IO<A> + Send + Sync + 'static>(
        thunk: Thunk,
    ) -> IO<A> {
        IO::new(move |token| thunk().run(token))
    }
}

use crate::core::IoH;
use crate::data::io::{IO, IoResult};
use crate::typeclasses::Ref;
use std::sync::{Arc, Mutex};

pub struct IoRef<A> {
    pub(crate) cell: Arc<Mutex<A>>,
}

impl<A: Clone + Send + Sync + 'static> Ref<IoH, A> for IoRef<A> {
    fn get(&self) -> IO<A> {
        let cell = self.cell.clone();
        IO::new(move |_| IoResult::Completed(cell.lock().unwrap().clone()))
    }

    fn set(&self, a: A) -> IO<()> {
        let cell = self.cell.clone();
        let a = Arc::new(a);
        IO::new(move |_| {
            *cell.lock().unwrap() = (*a).clone();
            IoResult::Completed(())
        })
    }

    fn update<U: Fn(A) -> A + 'static>(&self, _f: U) -> IO<()> {
        unimplemented!("Ref::update on IoRef requires U: Send + Sync; use IoRef::update_io")
    }

    fn modify<B: 'static, U: Fn(A) -> (A, B) + 'static>(&self, _f: U) -> IO<B> {
        unimplemented!("Ref::modify on IoRef requires U: Send + Sync; use IoRef::modify_io")
    }
}

impl<A: Clone + Send + Sync + 'static> IoRef<A> {
    pub fn update_io<U: Fn(A) -> A + Send + Sync + 'static>(&self, f: U) -> IO<()> {
        let cell = self.cell.clone();
        IO::new(move |_| {
            let mut g = cell.lock().unwrap();
            let cur = g.clone();
            *g = f(cur);
            IoResult::Completed(())
        })
    }

    pub fn modify_io<B: Send + 'static, U: Fn(A) -> (A, B) + Send + Sync + 'static>(
        &self,
        f: U,
    ) -> IO<B> {
        let cell = self.cell.clone();
        IO::new(move |_| {
            let mut g = cell.lock().unwrap();
            let cur = g.clone();
            let (next, out) = f(cur);
            *g = next;
            IoResult::Completed(out)
        })
    }
}

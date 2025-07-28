
use core::{future::Future, pin::Pin, task::{Context, Poll}};
use alloc::boxed::Box;

pub mod simple_executor;

pub struct Task {
    // task to run without outputting, while not moving the future around in memory by pinning it
    // through placing it on the heap
    // allows safety against self-referential futures if a future is moved, to not reference old
    // memory
    future: Pin<Box<dyn Future<Output = ()>>>
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            future: Box::pin(future)
        }
    }
    
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

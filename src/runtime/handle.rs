use core::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use alloc::sync::Arc;
use crossbeam_queue::ArrayQueue;

/// An owned permission to wait until a [Task](crate::task::Task) is driven to completion
pub struct JoinHandle<T> {
    rx: Arc<ArrayQueue<T>>,
    _phantom_data: PhantomData<fn() -> T>,
}

impl<T> JoinHandle<T> {
    pub fn new(rx: Arc<ArrayQueue<T>>) -> Self {
        Self {
            rx,
            _phantom_data: PhantomData,
        }
    }

    /// waits for the [Task](crate::task::Task) to be driven to completion and returns its result
    pub fn join(&self) -> T {
        loop {
            match self.rx.pop() {
                Some(x) => return x,
                None => continue,
            }
        }
    }
}

/// see [JoinHandle::join](struct.JoinHandle.html#method.join)
impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        return Poll::Ready(self.join());
    }
}

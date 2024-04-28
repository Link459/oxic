use core::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use alloc::sync::Arc;
use crossbeam_queue::ArrayQueue;

use crate::task::TaskId;

pub struct JoinHandle<T> {
    id: TaskId,
    rx: Arc<ArrayQueue<T>>,
    _phantom_data: PhantomData<fn() -> T>,
}

impl<T> JoinHandle<T> {
    pub fn new(id: TaskId, rx: Arc<ArrayQueue<T>>) -> Self {
        Self {
            id,
            rx,
            _phantom_data: PhantomData,
        }
    }

    pub fn join(&self) -> T {
        loop {
            match self.rx.pop() {
                Some(x) => return x,
                None => continue,
            }
        }
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        return Poll::Ready(self.join());
    }
}

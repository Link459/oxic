use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(u64);

impl TaskId {
    pub fn new() -> Self {
        static TASK_ID: AtomicU64 = AtomicU64::new(0);
        return TaskId(TASK_ID.fetch_add(1, Ordering::Relaxed));
    }
}

/// A asynchronos path of execution, the fundamental building block of the Runtime.
pub struct Task {
    pub(crate) id: TaskId,
    pub(crate) future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(fut: impl Future<Output = ()> + 'static) -> Self {
        return Self {
            id: TaskId::new(),
            future: Box::pin(fut),
        };
    }

    /// polls the asynchronos task
    pub fn poll(&mut self, cx: &mut Context) -> Poll<()> {
        return self.future.as_mut().poll(cx);
    }
}

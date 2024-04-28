use core::task::Waker;

use alloc::{sync::Arc, task::Wake};
use crossbeam_queue::SegQueue;

use crate::task::TaskId;

pub struct TaskWaker {
    id: TaskId,
    task_queue: Arc<SegQueue<TaskId>>,
}

impl TaskWaker {
    pub fn new(id: TaskId, task_queue: Arc<SegQueue<TaskId>>) -> Waker {
        return Waker::from(Arc::new(Self { id, task_queue }));
    }

    pub(crate) fn wake_task(&self) -> () {
        self.task_queue.push(self.id);
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}

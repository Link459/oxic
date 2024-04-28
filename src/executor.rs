use core::{
    future::Future,
    panic,
    task::{Context, Poll, Waker},
};

use alloc::{collections::BTreeMap, sync::Arc};

use crate::{
    handle::JoinHandle,
    task::{Task, TaskId},
    waker::TaskWaker,
};
use crossbeam_queue::{ArrayQueue, SegQueue};

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<SegQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new() -> Self {
        return Self {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(SegQueue::new()),
            waker_cache: BTreeMap::new(),
        };
    }

    pub fn spawn<Fut, T>(&mut self, f: Fut) -> JoinHandle<T>
    where
        Fut: Future<Output = T> + 'static,
        T: 'static,
    {
        let queue = Arc::new(ArrayQueue::new(1));
        let queue2 = queue.clone();
        let fut = async move {
            let v = f.await;
            let _ = queue2.push(v);
            //.expect("unexpected error, queue should not be full");
        };

        let task = Task::new(fut);
        let id = task.id;
        if self.tasks.insert(id, task).is_some() {
            panic!("task already exists");
        }

        self.task_queue.push(id);
        self.run_task(id);
        return JoinHandle::new(id, queue);
    }

    pub fn block_on<Fut, T>(&mut self, f: Fut) -> T
    where
        Fut: Future<Output = T> + 'static,
        T: 'static,
    {
        return self.spawn(f).join();
    }

    pub fn run(&mut self) -> ! {
        loop {
            while let Some(id) = self.task_queue.pop() {
                self.run_task(id);
            }
        }
    }

    fn run_task(&mut self, id: TaskId) {
        let task = match self.tasks.get_mut(&id) {
            Some(task) => task,
            None => return,
        };

        let waker = self
            .waker_cache
            .entry(id)
            .or_insert_with(|| TaskWaker::new(id, self.task_queue.clone()));

        let mut cx = Context::from_waker(waker);
        match task.poll(&mut cx) {
            Poll::Ready(_) => {
                self.tasks.remove(&id);
                self.waker_cache.remove(&id);
            }
            Poll::Pending => (),
        }
    }

}

#[cfg(test)]
mod tests {
    use alloc::sync::Arc;

    use crossbeam_queue::SegQueue;

    use super::Executor;

    #[test]
    fn run() {
        let q = Arc::new(SegQueue::new());
        let q1 = q.clone();
        let hello = || async move {
            q1.push("Hello");
        };

        let mut ex = Executor::new();
        ex.spawn(hello());
        //ex.run();
        assert_eq!(q.pop().unwrap(), "Hello");
    }
}

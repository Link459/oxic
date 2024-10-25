use core::{
    future::Future,
    panic,
    task::{Context, Poll, Waker},
};
use std::sync::Mutex;

use alloc::{collections::BTreeMap, sync::Arc};

use crossbeam_queue::{ArrayQueue, SegQueue};

use crate::runtime::{
    handle::JoinHandle,
    task::task::{Task, TaskId},
    waker::TaskWaker,
};

/// A Executor
pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    pub(crate) task_queue: Arc<SegQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

//should be safe since it's always in a Arc<Mutex<T>> and there is only one thread polling it
unsafe impl Send for Executor {}

impl Executor {
    pub fn new() -> Self {
        return Self {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(SegQueue::new()),
            waker_cache: BTreeMap::new(),
        };
    }

    /// spawns a [Task] onto the executor and polls it once. After that it will continue
    /// execution asynchronously
    pub fn spawn<Fut, T>(&mut self, f: Fut) -> JoinHandle<T>
    where
        Fut: Future<Output = T> + 'static,
        T: 'static,
    {
        let result_reciever = Arc::new(ArrayQueue::new(1));
        let result_sender = result_reciever.clone();
        let fut = async move {
            let v = f.await;
            match result_sender.push(v) {
                Ok(_) => (),
                Err(_) => panic!("queue should never be full at this point"),
            };
        };

        let task = Task::new(fut);
        let id = task.id;
        if self.tasks.insert(id, task).is_some() {
            // this should never be reached since the TaskId is atomically incremented with each
            // call so getting the same twice here is impossible
            unreachable!();
        }

        self.task_queue.push(id);
        //self.task_queue.push(id);
        return JoinHandle::new(result_reciever);
    }

    /// [spawn](Self::spawn<Fut,T>()) a [Task] in a blocking manner
    pub fn block_on<Fut, T>(&mut self, f: Fut) -> T
    where
        Fut: Future<Output = T> + 'static,
        T: 'static,
    {
        return self.spawn(f).join();
    }

    pub(crate) fn run_task(&mut self, id: TaskId) {
        let task = match self.tasks.get_mut(&id) {
            Some(task) => task,
            None => {
                return;
            }
        };

        assert_eq!(task.id, id);

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
            Poll::Pending => return,
        }
    }
}

impl Default for Executor {
    fn default() -> Self {
        return Self::new();
    }
}

#[inline(always)]
pub(crate) fn run_executor(ex: Arc<Mutex<Executor>>) -> impl Fn() {
    println!("running executor");
    move || loop {
        let mut ex = ex.lock().unwrap();
        if let Some(id) = ex.task_queue.pop() {
            ex.run_task(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{string::String, sync::Arc};

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

    #[test]
    fn with_return() {
        async fn hello() -> String {
            return String::from("Hello");
        }

        let mut ex = Executor::new();
        let res = ex.block_on(hello());
        assert_eq!(res, "Hello");
    }

    #[test]
    fn nested() {
        async fn bottom() -> u32 {
            7
        }

        async fn middle() -> u32 {
            bottom().await
        }
        async fn top() -> u32 {
            middle().await
        }

        let mut ex = Executor::new();
        let res = ex.block_on(top());
        assert_eq!(res, 7);
    }
}

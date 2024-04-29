use core::future::Future;
use std::{sync::Mutex, thread};

use alloc::sync::Arc;

use crate::{
    executor::{self, Executor},
    handle::JoinHandle,
};

pub struct Runtime {
    executor: Arc<Mutex<Executor>>,
}

impl Runtime {
    pub fn new() -> Self {
        let rt = Self {
            executor: Arc::new(Mutex::new(Executor::new())),
        };

        let ex = rt.executor.clone();

        thread::spawn(executor::run_executor(ex));
        return rt;
    }

    pub fn spawn<Fut, T>(&mut self, f: Fut) -> JoinHandle<T>
    where
        Fut: Future<Output = T> + 'static,
        T: 'static,
    {
        return self.executor.lock().unwrap().spawn(f);
    }

    pub fn block_on<Fut, T>(&mut self, f: Fut) -> T
    where
        Fut: Future<Output = T> + 'static,
        T: 'static,
    {
        return self.spawn(f).join();
    }
}

impl Default for Runtime {
    fn default() -> Self {
        return Self::new();
    }
}

#[cfg(test)]
mod tests {
    use core::{assert_eq, future::Future, task::Poll};

    use crate::runtime::Runtime;

    struct TestFuture {
        pub counter: u32,
    }

    impl Future for TestFuture {
        type Output = u32;
        fn poll(
            mut self: core::pin::Pin<&mut Self>,
            cx: &mut core::task::Context<'_>,
        ) -> core::task::Poll<Self::Output> {
            if self.counter == 4 {
                return Poll::Ready(1);
            }

            cx.waker().wake_by_ref();

            self.counter += 1;
            return Poll::Pending;
        }
    }

    #[test]
    fn test_custom_future() {
        async fn two_times() -> bool {
            let tf = TestFuture { counter: 0 };
            let x = tf.await;
            if x == 0 {
                return true;
            }
            return false;
        }

        let mut rt = Runtime::new();
        let b = rt.block_on(two_times());
        assert_eq!(b, false);
    }
}

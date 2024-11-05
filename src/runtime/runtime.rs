use core::future::Future;
use std::{
    sync::Mutex,
    thread::{self},
};

use alloc::sync::Arc;

use crate::runtime::{
    executor::executor::{self, Executor},
    handle::JoinHandle,
};

use super::reactor::reactor::Reactor;

pub struct RuntimeBuilder {
    num_threads: usize,
}

impl RuntimeBuilder {
    pub fn new() -> Self {
        return Self {
            num_threads: 1, //std::thread::available_parallelism().unwrap().into(),
        };
    }

    pub fn threads(mut self, num_threads: usize) -> Self {
        self.num_threads = num_threads;
        return self;
    }

    pub fn build(self) -> Runtime {
        return Runtime::from_builder(self);
    }
}

pub struct Runtime {
    executor: Arc<Mutex<Executor>>,
}

impl Runtime {
    pub fn new() -> Self {
        let builder = RuntimeBuilder::new();
        return builder.build();
    }

    pub fn from_builder(builder: RuntimeBuilder) -> Self {
        let rt = Self {
            executor: Arc::new(Mutex::new(Executor::new())),
        };

        let _ = Reactor::get();

        for _ in 0..builder.num_threads {
            println!("spawning executor loop");
            let _ = thread::spawn(executor::run_executor(rt.executor.clone()));
        }

        return rt;
    }

    #[inline(always)]
    pub fn spawn<Fut, T>(&mut self, f: Fut) -> JoinHandle<T>
    where
        Fut: Future<Output = T> + 'static,
        T: 'static,
    {
        return self.executor.lock().unwrap().spawn(f);
    }

    #[inline(always)]
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

    use crate::prelude::Runtime;

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

    #[test]
    fn panic() {
        async fn does_panic() {
            panic!("panic happened");
        }

        let mut rt = Runtime::new();
        let err = std::panic::catch_unwind(move || rt.spawn(does_panic()));
        //assert!(err.is_err());
    }
}

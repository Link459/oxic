
use alloc::sync::Arc;

use crate::executor::Executor;

pub struct Runtime {
    executor: Arc<Executor>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            executor: Arc::new(Executor::new()),
        }
    }

    pub fn run() -> () {}
}

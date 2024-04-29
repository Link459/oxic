//! A tiny async rust runtime
//!
//! ## Example
//! ```
//! use oxic::prelude::*;
//!
//! fn main() {
//! 	let mut rt = Runtime::new();
//! 	rt.block_on(async_hello());
//! }
//!
//! async fn async_hello() {
//! 	println!("Hello, from async");
//! }
//! ```
extern crate alloc;

pub mod executor;
pub mod handle;
pub mod runtime;
pub mod task;
pub mod waker;

pub mod prelude {
    pub use crate::{executor::Executor, handle::JoinHandle, runtime::Runtime, task::Task};
}

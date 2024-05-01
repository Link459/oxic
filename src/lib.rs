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

pub mod runtime;

pub mod io;
pub mod net;
pub mod fs;

pub mod prelude {
    pub use crate::runtime::runtime::Runtime;
    pub use crate::{
        runtime::executor::executor::Executor, runtime::handle::JoinHandle,
        runtime::task::task::Task,
    };
}

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use oxic::prelude::*;

struct GiveNumberFuture {
    number_to_give: u32,
    give_after_tries: u32,
    current_tries: u32,
}

impl Future for GiveNumberFuture {
    type Output = u32;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        println!(
            "polled {} time(s) - now on {:?}",
            this.current_tries + 1,
            std::thread::current().id()
        );
        if this.give_after_tries > this.current_tries + 1 {
            this.current_tries += 1;
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(this.number_to_give)
        }
    }
}

async fn main_thought(number: u32, tries: u32) {
    let future = GiveNumberFuture {
        number_to_give: number,
        give_after_tries: tries,
        current_tries: 0,
    };

    let number = future.await;
    println!("waited for {}", number);
}

fn spawn(mut rt: Runtime) {
    let j = rt.spawn(main_thought(10, 10));
    rt.spawn(main_thought(30, 6));
    rt.spawn(main_thought(20, 5));
    rt.spawn(main_thought(40, 3));
    j.join();
}

fn main() {
    let rt = Runtime::new();
    spawn(rt);
}

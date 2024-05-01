use std::{os::fd::RawFd, sync::OnceLock, task::Waker, thread};

use super::{
    epoll::Epoll,
    interest::{epoll_interest, Interest},
};
use epoll::Event;
use lockfree::map::Map;

struct Subscription {
    waker: Waker,
    interest: Interest,
}

impl Subscription {
    fn new(waker: Waker, interest: Interest) -> Self {
        Self { waker, interest }
    }
}

pub struct Reactor {
    subscribtions: Map<i32, Subscription>,
    epoll: Epoll,
}

impl Reactor {
    pub fn new() -> Reactor {
        return Self {
            subscribtions: Map::new(),
            epoll: Epoll::new(),
        };
    }

    pub fn get() -> &'static Reactor {
        static REACTOR: OnceLock<Reactor> = OnceLock::new();
        return REACTOR.get_or_init(|| {
            thread::spawn(Self::reactor_loop);
            Self::new()
        });
    }

    pub fn register(&self, fd: RawFd, waker: Waker, interest: Interest) {
        let sub = Subscription::new(waker, interest);
        self.subscribtions.insert(fd, sub);
        self.epoll
            .add_interest(fd, epoll_interest(fd as u64, interest))
            .expect("failed to add interest to epoll queue");
    }

    pub fn remove(&self, fd: RawFd) {
        let rem = self
            .subscribtions
            .remove(&fd)
            .expect("failed to remove from epoll queue");
        self.epoll
            .remove_interest(fd, Event::new(rem.1.interest.into(), fd as u64))
            .expect("failed to remove from epoll queue");
    }

    fn wait(&self, mut buf: &mut Vec<Event>) -> Vec<Waker> {
        buf.clear();
        let mut wakers = Vec::new();
        loop {
            let n = self.epoll.wait(None, &mut buf).unwrap();

            assert_eq!(n, buf.len());
            for event in buf.iter() {
                let fd = event.data;
                let sub = self
                    .subscribtions
                    .remove(&(fd as i32))
                    .expect("subscription should exist");
                self.remove(fd as i32);

                wakers.push(sub.1.waker.clone());
            }
            return wakers;
        }
    }

    pub fn reactor_loop() {
        let reactor = Self::get();
        let mut buf = Vec::with_capacity(1024);
        loop {
            let wakers = reactor.wait(&mut buf);
            wakers.iter().for_each(Waker::wake_by_ref);
        }
    }
}

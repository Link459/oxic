use std::{os::fd::RawFd, sync::OnceLock, task::Waker, thread};

use super::{
    interest::{Event, Interest},
    poller::Poller,
};
use lockfree::map::Map;

#[derive(Debug, Clone)]
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
    poller: Poller,
}

impl Reactor {
    pub fn new() -> Reactor {
        return Self {
            subscribtions: Map::new(),
            poller: Poller::new(),
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
        // we need to clone this subsription here otherwise we get problem registering the fd for
        // some weird reason, cannot be bothered to find why it actually fails
        let sub = Subscription::new(waker.clone(), interest);
        let _ = self.subscribtions.insert(fd, sub);
        let event = Event::new(fd, interest);
        self.poller
            .add_interest(event)
            .expect("failed to add interest to epoll queue");
    }

    pub fn remove(&self, fd: RawFd) {
        let rem = self
            .subscribtions
            .remove(&fd)
            .expect("failed to remove from epoll queue");
        self.poller
            .remove_interest(Event::new(fd, rem.1.interest))
            .expect("failed to remove from epoll queue");
    }

    fn wait(&self, mut buf: &mut Vec<Event>) {
        let n = self
            .poller
            .wait(None, &mut buf)
            .expect("failed to wait for events");

        for i in 0..n.0 {
            let event = n.1[i];
            let fd = event.fd;
            let sub = self
                .subscribtions
                .remove(&fd)
                .expect("subscription should exist");
            self.poller
                .remove_interest(Event::new(fd, sub.1.interest))
                .expect("failed to remove from epoll queue");
            println!("notifing waker");
            sub.1.waker.wake_by_ref();
        }
    }

    pub fn run(&self) {
        let mut buf = Vec::with_capacity(1024);
        buf.resize(1024, Event::default());
        dbg!(buf.len());
        loop {
            self.wait(&mut buf);
        }
    }

    pub fn reactor_loop() {
        let reactor = Self::get();
        let mut buf = Vec::with_capacity(1024);
        buf.resize(1024, Event::default());
        loop {
            reactor.wait(&mut buf);
        }
    }
}

mod tests {
    use std::{
        net::UdpSocket,
        os::fd::{AsRawFd, RawFd},
        sync::Arc,
        task::{Wake, Waker},
    };

    use crate::runtime::reactor::interest::Interest;

    use super::Reactor;

    struct TestWaker;
    impl Wake for TestWaker {
        fn wake(self: Arc<Self>) {
            println!("woken up")
        }

        fn wake_by_ref(self: &Arc<Self>) {
            println!("woken up")
        }
    }

    #[test]
    pub fn register_fd() {
        let reactor = Reactor::new();

        let socket = UdpSocket::bind("127.0.0.1:3006").unwrap();
        let fd = socket.as_raw_fd();
        reactor.register(fd, Waker::from(Arc::new(TestWaker {})), Interest::Read);
    }

    pub fn run_reactor() {
        let reactor = Reactor::new();

        let socket = UdpSocket::bind("127.0.0.1:3006").unwrap();
        let fd = socket.as_raw_fd();
        reactor.register(fd, Waker::from(Arc::new(TestWaker {})), Interest::Read);
    }
}

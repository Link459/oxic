use std::io;

use super::epoll::Epoll;
use super::interest::Event;

pub enum Poller {
    Epoll(Epoll),
}

impl Poller {
    pub fn new() -> Self {
        let poller = Epoll::new();
        return Self::Epoll(poller);
    }

    pub fn add_interest(&self, event: Event) -> Result<(), std::io::Error> {
        match self {
            Poller::Epoll(epoll) => epoll.add_interest(event.fd, event.into()),
        }
    }
    pub fn remove_interest(&self, event: Event) -> Result<(), std::io::Error> {
        match self {
            Poller::Epoll(epoll) => epoll.remove_interest(event.fd, event.into()),
        }
    }
    pub fn modify_interest(&self, event: Event) -> Result<(), std::io::Error> {
        match self {
            Poller::Epoll(epoll) => epoll.modify_interest(event.fd, event.into()),
        }
    }

    pub fn wait(
        &self,
        timeout: Option<u32>,
        events: &mut [Event],
    ) -> io::Result<(usize, Vec<Event>)> {
        match self {
            Poller::Epoll(epoll) => {
                let mut epoll_buf = Vec::with_capacity(events.len());
                epoll_buf.resize(events.len(), epoll::Event::new(epoll::Events::empty(), 0));
                let size = epoll.wait(timeout, &mut epoll_buf)?;
                let res: Vec<_> = epoll_buf.iter().map(Event::from).collect();
                return Ok((size, res));
            }
        };
    }
}

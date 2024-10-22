use std::os::fd::RawFd;

use epoll::Events;

#[derive(Default, Debug, Clone, Copy)]
pub enum Interest {
    Read,
    Write,
    #[default]
    ReadWrite,
}

impl From<epoll::Events> for Interest {
    fn from(value: epoll::Events) -> Self {
        if value.contains(epoll::Events::EPOLLIN) && value.contains(epoll::Events::EPOLLOUT) {
            return Interest::ReadWrite;
        }
        if value.contains(epoll::Events::EPOLLIN) {
            return Interest::Read;
        }
        if value.contains(epoll::Events::EPOLLOUT) {
            return Interest::Write;
        }
        return Interest::ReadWrite;
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Event {
    pub fd: RawFd,
    pub interest: Interest,
}

impl Event {
    pub fn new(fd: RawFd, interest: Interest) -> Self {
        Self { fd, interest }
    }
}

impl Into<epoll::Event> for Event {
    fn into(self) -> epoll::Event {
        return epoll::Event::new(self.interest.into(), self.fd as u64);
    }
}

impl From<&epoll::Event> for Event {
    fn from(value: &epoll::Event) -> Self {
        Self {
            fd: value.data as i32,
            interest: Interest::from(epoll::Events::from_bits_truncate(value.events)),
        }
    }
}

impl Into<Events> for Interest {
    fn into(self) -> Events {
        return match self {
            Interest::Read => Events::EPOLLONESHOT | Events::EPOLLIN,
            Interest::Write => Events::EPOLLONESHOT | Events::EPOLLOUT,
            Interest::ReadWrite => Events::EPOLLONESHOT | Events::EPOLLOUT | Events::EPOLLIN,
        };
    }
}

/*pub(crate) fn epoll_interest(data: u64, interest: Interest) -> epoll::Event {
    let events = match interest {
        Interest::Read => Events::EPOLLONESHOT | Events::EPOLLIN,
        Interest::Write => Events::EPOLLONESHOT | Events::EPOLLOUT,
        Interest::ReadWrite => todo!(),
    };

    return epoll::Event::new(events, data);
}*/

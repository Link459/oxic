use epoll::{Event, Events};

#[derive(Clone, Copy)]
pub enum Interest {
    Read,
    Write,
}

impl Into<Events> for Interest {
    fn into(self) -> Events {
        return match self {
            Interest::Read => Events::EPOLLONESHOT | Events::EPOLLIN,
            Interest::Write => Events::EPOLLONESHOT | Events::EPOLLOUT,
        };
    }
}

pub(crate) fn epoll_interest(data: u64, interest: Interest) -> Event {
    let events = match interest {
        Interest::Read => Events::EPOLLONESHOT | Events::EPOLLIN,
        Interest::Write => Events::EPOLLONESHOT | Events::EPOLLOUT,
    };

    return Event::new(events, data);
}

use std::{io, os::fd::RawFd};

use epoll::{ControlOptions, Event};

pub struct Epoll(pub RawFd);

//read flags =  Events::EPOLLONESHOT | Events::EPOLLIN;
//write flags =  Events::EPOLLONESHOT | Events::EPOLLOUT;

impl Epoll {
    pub fn new() -> Self {
        return Self(epoll::create(true).expect("can't create a epoll queue"));
    }

    pub fn add_interest(&self, fd: RawFd, event: Event) -> io::Result<()> {
        return epoll::ctl(self.0, ControlOptions::EPOLL_CTL_ADD, fd, event);
    }

    pub fn modify_interest(&self, fd: RawFd, event: Event) -> io::Result<()> {
        return epoll::ctl(self.0, ControlOptions::EPOLL_CTL_MOD, fd, event);
    }

    pub fn remove_interest(&self, fd: RawFd, event: Event) -> io::Result<()> {
        return epoll::ctl(self.0, ControlOptions::EPOLL_CTL_DEL, fd, event);
    }

    pub fn wait(&self, timeout: Option<u32>, events: &mut [Event]) -> io::Result<usize> {
        let timeout = match timeout {
            None => -1,
            Some(x) => x as i32,
        };

        dbg!(timeout);
        //let events =
        dbg!(events.len());

        return epoll::wait(self.0, timeout, events);
    }

    pub fn close(&self) -> io::Result<()> {
        return epoll::close(self.0);
    }
}

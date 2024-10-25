use std::future::Future;
use std::net::{SocketAddr, UdpSocket as StdUdpSocket};
use std::os::fd::{AsRawFd, FromRawFd};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::{io, net::ToSocketAddrs};

use crate::runtime::reactor::interest::Interest;
use crate::runtime::reactor::reactor::Reactor;

pub struct UdpSocket {
    socket: Arc<StdUdpSocket>,
}

impl UdpSocket {
    pub fn bind<A>(addr: A) -> io::Result<Self>
    where
        A: ToSocketAddrs,
    {
        let sock = StdUdpSocket::bind(addr)?;
        sock.set_nonblocking(true)?;

        return Ok(UdpSocket {
            socket: Arc::new(sock),
        });
    }

    pub fn from_std(sock: StdUdpSocket) -> Self {
        return Self {
            socket: Arc::new(sock),
        };
    }

    //TODO: somehow implement this
    pub fn into_std(self) -> StdUdpSocket {
        let fd = self.socket.as_raw_fd();
        return unsafe { StdUdpSocket::from_raw_fd(fd) };
    }

    pub fn connect<A: ToSocketAddrs>(&self, addr: A) -> Connect<A> {
        let connect = Connect {
            socket: self.socket.clone(),
            addr,
        };

        return connect;
    }

    pub fn recv<'a>(&self, buf: &'a mut [u8]) -> Recv<'a> {
        let recv = Recv {
            socket: self.socket.clone(),
            buf,
        };
        return recv;
    }

    pub fn recv_from<'a>(&self, buf: &'a mut [u8]) -> RecvFrom<'a> {
        let recv_from = RecvFrom {
            socket: self.socket.clone(),
            buf,
        };
        return recv_from;
    }

    pub fn send<'a>(&self, buf: &'a [u8]) -> Send<'a> {
        let send = Send {
            socket: self.socket.clone(),
            buf,
        };
        return send;
    }

    pub fn send_to<'a, A>(&self, buf: &'a [u8], addr: A) -> SendTo<'a, A>
    where
        A: ToSocketAddrs,
    {
        let send_to = SendTo {
            socket: self.socket.clone(),
            addr,
            buf,
        };
        return send_to;
    }
}

pub struct Connect<A> {
    socket: Arc<StdUdpSocket>,
    addr: A,
}

impl<A> Future for Connect<A>
where
    A: ToSocketAddrs,
{
    type Output = io::Result<()>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        return match self.socket.connect(&self.addr) {
            Ok(x) => Poll::Ready(Ok(x)),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                Reactor::get().register(
                    self.socket.as_raw_fd(),
                    cx.waker().clone(),
                    Interest::ReadWrite,
                );
                return Poll::Pending;
            }
            Err(e) => Poll::Ready(Err(e)),
        };
    }
}

pub struct Recv<'a> {
    socket: Arc<StdUdpSocket>,
    buf: &'a mut [u8],
}

impl Future for Recv<'_> {
    type Output = io::Result<usize>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("polling of stuff");
        return match self.socket.clone().recv(&mut self.buf) {
            Ok(x) => Poll::Ready(Ok(x)),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                Reactor::get().register(
                    self.socket.as_raw_fd(),
                    cx.waker().clone(),
                    Interest::Read,
                );
                return Poll::Pending;
            }
            Err(e) => Poll::Ready(Err(e)),
        };
    }
}

pub struct RecvFrom<'a> {
    socket: Arc<StdUdpSocket>,
    buf: &'a mut [u8],
}

impl Future for RecvFrom<'_> {
    type Output = io::Result<(usize, SocketAddr)>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        return match self.socket.clone().recv_from(&mut self.buf) {
            Ok(x) => Poll::Ready(Ok(x)),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                Reactor::get().register(
                    self.socket.as_raw_fd(),
                    cx.waker().clone(),
                    Interest::Read,
                );
                return Poll::Pending;
            }
            Err(e) => Poll::Ready(Err(e)),
        };
    }
}

pub struct Send<'a> {
    socket: Arc<StdUdpSocket>,
    buf: &'a [u8],
}

impl Future for Send<'_> {
    type Output = io::Result<usize>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        return match self.socket.clone().send(&self.buf) {
            Ok(x) => Poll::Ready(Ok(x)),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                Reactor::get().register(
                    self.socket.as_raw_fd(),
                    cx.waker().clone(),
                    Interest::Write,
                );
                return Poll::Pending;
            }
            Err(e) => Poll::Ready(Err(e)),
        };
    }
}

pub struct SendTo<'a, A> {
    socket: Arc<StdUdpSocket>,
    addr: A,
    buf: &'a [u8],
}

impl<A> Future for SendTo<'_, A>
where
    A: ToSocketAddrs,
{
    type Output = io::Result<usize>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        return match self.socket.clone().send_to(&self.buf, &self.addr) {
            Ok(x) => Poll::Ready(Ok(x)),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                Reactor::get().register(
                    self.socket.as_raw_fd(),
                    cx.waker().clone(),
                    Interest::Write,
                );
                return Poll::Pending;
            }
            Err(e) => Poll::Ready(Err(e)),
        };
    }
}

#[cfg(test)]
mod test {
    use std::assert_eq;

    use crate::prelude::{Executor, Runtime};

    use super::UdpSocket;
    use std::net::UdpSocket as StdUdpSocket;

    #[test]
    pub fn bind() {
        let _ = UdpSocket::bind("127.0.0.1:3000").unwrap();
    }

    #[test]
    pub fn send() {
        let mut ex = Executor::new();
        ex.block_on(async {
            let sock1 = StdUdpSocket::bind("127.0.0.1:3000").unwrap();
            let sock2 = UdpSocket::bind("127.0.0.1:3001").unwrap();
            sock2.connect("127.0.0.1:3000").await.unwrap();
            let buf = "hello".as_bytes();
            let n = sock2.send(buf).await.unwrap();
            assert_eq!(n, 5);
            let mut buf2 = [0; 5];
            let n = sock1.recv(&mut buf2).unwrap();
            assert_eq!(n, 5);
            assert_eq!(buf, buf2);
        });
    }

    #[test]
    pub fn send_to() {
        let mut ex = Executor::new();
        ex.block_on(async {
            let sock1 = StdUdpSocket::bind("127.0.0.1:3002").unwrap();
            let sock2 = UdpSocket::bind("127.0.0.1:3003").unwrap();
            let buf = "hello".as_bytes();
            let n = sock2.send_to(buf, "127.0.0.1:3002").await.unwrap();
            assert_eq!(n, 5);
            let mut buf2 = [0; 5];
            let n = sock1.recv(&mut buf2).unwrap();
            assert_eq!(n, 5);
            assert_eq!(buf, buf2);
        });
    }

    #[test]
    pub fn recv() {
        let mut rt = Runtime::new();

        rt.block_on(async {
            let sock = UdpSocket::bind("127.0.0.1:3004").unwrap();
            let mut buf = [0; 5];

            let sock2 = std::net::UdpSocket::bind("127.0.0.1:3005").unwrap();
            let n = sock2.send_to("hello".as_bytes(), "127.0.0.1:3004").unwrap();
            assert_eq!(n, 5);

            let n = sock.recv(&mut buf).await.unwrap();
            assert_eq!(n, 5);
            assert_eq!(buf, "hello".as_bytes());
        });
    }
}

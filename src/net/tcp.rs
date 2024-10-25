use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::net::TcpListener as StdTcpListener;
use std::net::TcpStream as StdTcpStream;
use std::net::ToSocketAddrs;
use std::os::fd::AsRawFd;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;

use crate::runtime::reactor::interest::Interest;
use crate::runtime::reactor::reactor::Reactor;

pub struct TcpStream {
    stream: Arc<StdTcpStream>,
}

impl TcpStream {
    pub fn from_std(stream: StdTcpStream) -> Self {
        todo!();
    }
}

pub struct TcpListener {
    listener: Arc<StdTcpListener>,
}

impl TcpListener {
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<TcpListener> {
        let listener = StdTcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        return Ok(Self {
            listener: Arc::new(listener),
        });
    }

    pub async fn accept(&self) -> Accept {
        let accept = Accept {
            listener: self.listener.clone(),
        };
        return accept;
    }
}

pub struct Accept {
    listener: Arc<StdTcpListener>,
}

impl Future for Accept {
    type Output = io::Result<(TcpStream, SocketAddr)>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.listener.accept() {
            Ok(res) => Poll::Ready(Ok((TcpStream::from_std(res.0), res.1))),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                Reactor::get().register(
                    self.listener.as_raw_fd(),
                    cx.waker().clone(),
                    Interest::ReadWrite,
                );
                return Poll::Pending;
            }
            Err(_) => todo!(),
        }
    }
}

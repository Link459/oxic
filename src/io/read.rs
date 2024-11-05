use std::{
    future::Future,
    io,
    ops::DerefMut,
    pin::Pin,
    task::{Context, Poll},
};

pub trait AsyncRead {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>>;
}

pub trait AsyncReadExt {
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Read<'a, Self>
    where
        Self: Sized,
    {
        return Read { reader: self, buf };
    }
}

impl<T> AsyncReadExt for T where T: AsyncRead {}

pub struct Read<'a, T: ?Sized> {
    reader: &'a mut T,
    buf: &'a mut [u8],
}

impl<'a, T> Future for Read<'a, T>
where
    T: AsyncRead + Unpin + ?Sized,
{
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self { reader, buf } = self.deref_mut();
        let reader = reader.deref_mut();
        let buf = buf.deref_mut();
        return Pin::new(reader).poll_read(cx, buf);
    }
}

#[cfg(test)]
mod test {
    use std::{
        io,
        pin::Pin,
        task::{Context, Poll},
    };

    use crate::prelude::Runtime;

    use super::{AsyncRead, AsyncReadExt};

    struct TestReader;
    impl AsyncRead for TestReader {
        fn poll_read(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &mut [u8],
        ) -> Poll<io::Result<usize>> {
            if buf.len() != 5 {
                return Poll::Ready(Err(io::ErrorKind::InvalidInput.into()));
            }
            buf.copy_from_slice("Hello".as_bytes());
            return Poll::Ready(Ok(buf.len()));
        }
    }

    #[test]
    fn test_reader() {
        let mut rt = Runtime::new();
        rt.block_on(async {
            let mut reader = TestReader {};
            let mut buf = [0; 5];
            let res = reader.read(&mut buf).await;
            assert_eq!(res.unwrap(), 5);
            assert_eq!(buf.as_slice(), "Hello".as_bytes());
        });
    }
}

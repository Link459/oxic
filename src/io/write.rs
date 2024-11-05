use std::{
    future::Future,
    io,
    ops::DerefMut,
    pin::Pin,
    task::{Context, Poll},
};

pub trait AsyncWrite {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>>;
}

pub trait AsyncWriteExt {
    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Write<'a, Self>
    where
        Self: Sized,
    {
        return Write { writer: self, buf };
    }
}

impl<T> AsyncWriteExt for T where T: AsyncWrite {}

pub struct Write<'a, T: ?Sized> {
    writer: &'a mut T,
    buf: &'a [u8],
}

impl<'a, T> Future for Write<'a, T>
where
    T: AsyncWrite + Unpin + ?Sized,
{
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self { writer, buf } = self.deref_mut();
        let writer = writer.deref_mut();
        return Pin::new(writer).poll_write(cx, buf);
    }
}

#[cfg(test)]
mod test {
    use std::{
        io,
        pin::Pin,
        task::{Context, Poll},
    };

    use crate::{io::write::AsyncWriteExt, prelude::Runtime};

    use super::AsyncWrite;

    struct TestWriter;
    impl AsyncWrite for TestWriter {
        fn poll_write(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<io::Result<usize>> {
            assert_eq!(buf.len(), 5);
            assert_eq!(buf, "Hello".as_bytes());
            return Poll::Ready(Ok(buf.len()));
        }
    }

    #[test]
    fn test_writer() {
        let mut rt = Runtime::new();
        rt.block_on(async {
            let mut reader = TestWriter {};
            let res = reader.write("Hello".as_bytes()).await;
            assert_eq!(res.unwrap(), 5);
        });
    }
}

use std::{
    future::Future,
    marker::PhantomData,
    ops::DerefMut,
    pin::Pin,
    task::{Context, Poll},
};

pub trait Stream {
    type Item;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;
}

pub trait StreamExt {
    type Item;
    fn next<'a>(&'a mut self) -> Next<'a, Self, Self::Item> {
        return Next {
            stream: self,
            _phantom_data: PhantomData,
        };
    }
}

impl<T, I> StreamExt for T
where
    T: Stream<Item = I>,
{
    type Item = I;
}

pub struct Next<'a, S: ?Sized, I> {
    stream: &'a mut S,
    _phantom_data: PhantomData<fn() -> I>,
}

impl<'a, T, I> Future for Next<'a, T, I>
where
    T: Stream<Item = I> + Unpin + ?Sized,
{
    type Output = Option<I>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Next {
            stream,
            _phantom_data,
        } = self.deref_mut();
        let stream = stream.deref_mut();
        return Pin::new(stream).poll_next(cx);
    }
}

#[cfg(test)]
mod test {
    use std::{
        pin::Pin,
        task::{Context, Poll},
    };

    use crate::prelude::Runtime;

    use super::Stream;
    use super::StreamExt;

    struct TestStream(u32);

    impl Stream for TestStream {
        type Item = u32;
        fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            self.0 += 1;
            return Poll::Ready(Some(self.0));
        }
    }

    #[test]
    fn test_stream() {
        let mut rt = Runtime::new();
        rt.block_on(async {
            let mut stream = TestStream(0);
            let first = stream.next().await.unwrap();
            assert_eq!(first, 1);
            let second = stream.next().await.unwrap();
            assert_eq!(second, 2);
            let third = stream.next().await.unwrap();
            assert_eq!(third, 3);
        });
    }
}

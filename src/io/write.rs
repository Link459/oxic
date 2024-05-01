use std::{future::Future, io, pin::Pin, task::Context};

pub trait AsyncWrite {
    type Future: Future<Output = io::Result<usize>>;
    fn read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Self::Future;
}

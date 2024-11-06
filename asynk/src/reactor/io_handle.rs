use super::{direction::WakerMap, Direction, Reactor};
use mio::{event::Source, Interest, Token};
use std::{
    io::{self, ErrorKind, Read, Write},
    ops::Deref,
    pin::Pin,
    task::{Context, Poll, Waker},
};

/// Wrapper for an I/O source with event tracking capabilities for reading/writing
pub struct IoHandle<S>
where
    S: Source,
{
    /// Tracked source
    source: S,
    /// Currently registered mio token
    token: Token,
}

impl<S> Unpin for IoHandle<S> where S: Source {}

impl<S> Deref for IoHandle<S>
where
    S: Source,
{
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.source
    }
}

impl<S> IoHandle<S>
where
    S: Source,
{
    pub fn try_new(mut source: S, interests: Interest) -> io::Result<Self> {
        let token = Reactor::get().register(&mut source, interests, WakerMap::new())?;
        Ok(Self { source, token })
    }

    pub fn token(&self) -> Token {
        self.token
    }

    /// Set waker for direction
    pub fn set_waker(&self, direction: Direction, waker: Waker) -> io::Result<()> {
        Reactor::get().set_waker(self.token, direction, waker)?;
        Ok(())
    }

    /// Deregister source
    fn deregister(&mut self) -> io::Result<()> {
        Reactor::get().deregister(self.token, &mut self.source)
    }
}

pub fn poll_io<T>(
    token: Token,
    cx: &mut Context<'_>,
    direction: Direction,
    mut f: impl FnMut() -> io::Result<T>,
) -> Poll<io::Result<T>> {
    match f() {
        Ok(n) => Poll::Ready(Ok(n)),
        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
            Reactor::get().set_waker(token, direction, cx.waker().clone())?;
            Poll::Pending
        }
        Err(e) => Poll::Ready(Err(e)),
    }
}

impl<S> IoHandle<S>
where
    S: Source + Read,
{
    pub fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        poll_io(self.token, cx, Direction::Read, || self.source.read(buf))
    }
}

impl<S> IoHandle<S>
where
    S: Source + Write,
{
    pub fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        poll_io(self.token, cx, Direction::Write, || self.source.write(buf))
    }

    pub fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        poll_io(self.token, cx, Direction::Write, || self.source.flush())
    }
}

impl<S> Drop for IoHandle<S>
where
    S: Source,
{
    fn drop(&mut self) {
        self.deregister().ok();
    }
}

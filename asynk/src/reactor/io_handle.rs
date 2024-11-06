use super::{Direction, Reactor};
use mio::{event::Source, Token};
use std::{
    io::{ErrorKind, Read, Result, Write},
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
    /// Read waiting flag
    waiting_read: bool,
    /// Write waiting flag
    waiting_write: bool,
    /// Currently registered mio token
    token: Option<Token>,
}

impl<S> Unpin for IoHandle<S> where S: Source {}

impl<S> IoHandle<S>
where
    S: Source,
{
    pub fn new(source: S) -> Self {
        Self {
            source,
            waiting_read: false,
            waiting_write: false,
            token: None,
        }
    }

    /// Reference to the tracked source
    pub fn source(&self) -> &S {
        &self.source
    }

    /// Register tracking for changes in the specified direction
    pub fn register_direction(&mut self, direction: Direction, waker: Waker) -> Result<()> {
        let token = match self.token {
            Some(token) => {
                Reactor::get().add_direction_for_token(token, &mut self.source, direction, waker)?
            }
            None => Reactor::get().register(&mut self.source, direction, waker)?,
        };

        self.token = Some(token);

        Ok(())
    }

    /// Remove tracking for changes in the specified direction
    pub fn deregister_direction(&mut self, direction: Direction) -> Result<()> {
        match self.token {
            Some(token) => {
                self.token = Reactor::get().remove_direction_for_token(
                    token,
                    &mut self.source,
                    direction,
                )?;
                Ok(())
            }
            // If no token exists, there's nothing to deregister
            None => Ok(()),
        }
    }

    /// Remove tracking for all directions
    pub fn deregister_all_directions(&mut self) -> Result<()> {
        match self.token {
            Some(token) => {
                Reactor::get().deregister(token, &mut self.source)?;
                self.token = None;
                Ok(())
            }
            // If no token exists, there's nothing to deregister
            None => Ok(()),
        }
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
    ) -> Poll<Result<usize>> {
        if !self.waiting_read {
            self.register_direction(Direction::Read, cx.waker().clone())?;
            self.waiting_read = true;
            return Poll::Pending;
        }

        match self.source.read(buf) {
            Ok(n) => {
                if n == 0 {
                    self.deregister_direction(Direction::Read)?;
                    self.waiting_read = false;
                }

                Poll::Ready(Ok(n))
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => Poll::Pending,
            Err(e) => Poll::Ready(Err(e)),
        }
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
    ) -> Poll<Result<usize>> {
        println!("{:?}: poll_write", self.token);

        if !self.waiting_write {
            self.register_direction(Direction::Write, cx.waker().clone())?;
            self.waiting_write = true;
            return Poll::Pending;
        }

        match self.source.write(buf) {
            Ok(n) => {
                if n == 0 {
                    self.deregister_direction(Direction::Write)?;
                    self.waiting_write = false;
                }

                Poll::Ready(Ok(n))
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => Poll::Pending,
            Err(e) => Poll::Ready(Err(e)),
        }
    }

    pub fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        if !self.waiting_write {
            self.register_direction(Direction::Write, cx.waker().clone())?;
            self.waiting_write = true;
            return Poll::Pending;
        }

        match self.source.flush() {
            Ok(()) => {
                self.deregister_direction(Direction::Write)?;
                self.waiting_write = false;
                Poll::Ready(Ok(()))
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => Poll::Pending,
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl<S> Drop for IoHandle<S>
where
    S: Source,
{
    fn drop(&mut self) {
        self.deregister_all_directions().ok();
    }
}

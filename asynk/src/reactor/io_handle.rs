use super::{Direction, Reactor};
use mio::{event::Source, Token};
use parking_lot::Mutex;
use std::{
    future::Future,
    io::{ErrorKind, Read, Result, Write},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
};

/// Wrapper for an I/O source with event tracking capabilities for reading/writing
pub struct IoHandle<S>
where
    S: Source,
{
    /// Tracked source
    source: S,
    registered_directions: [bool; 2],
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
            registered_directions: [false, false],
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

        self.registered_directions[direction as usize] = true;

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

                self.registered_directions[direction as usize] = false;
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

    pub fn check_direction(&self, direction: Direction) -> bool {
        self.registered_directions[direction as usize]
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
        if !self.check_direction(Direction::Read) {
            self.register_direction(Direction::Read, cx.waker().clone())?;
            return Poll::Pending;
        }

        match self.source.read(buf) {
            Ok(n) => {
                if n == 0 {
                    self.deregister_direction(Direction::Read)?;
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
        if !self.check_direction(Direction::Write) {
            self.register_direction(Direction::Write, cx.waker().clone())?;
            return Poll::Pending;
        }

        match self.source.write(buf) {
            Ok(n) => {
                if n == 0 {
                    self.deregister_direction(Direction::Write)?;
                }

                Poll::Ready(Ok(n))
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => Poll::Pending,
            Err(e) => Poll::Ready(Err(e)),
        }
    }

    pub fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        if !self.check_direction(Direction::Write) {
            self.register_direction(Direction::Write, cx.waker().clone())?;
            return Poll::Pending;
        }

        match self.source.flush() {
            Ok(()) => {
                self.deregister_direction(Direction::Write)?;
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

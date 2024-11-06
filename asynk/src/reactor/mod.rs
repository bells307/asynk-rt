pub mod io_handle;

pub(crate) mod direction;

use direction::Direction;
use mio::{event::Source, Events, Interest, Poll, Registry, Token};
use sharded_slab::Slab;
use std::{
    io::{self, Error, Result},
    sync::{Arc, OnceLock},
    task::Waker,
    thread::{self},
};

/// Reactor polls events from mio and calls wakers interested
/// by these events
pub struct Reactor {
    wakers: Arc<Slab<[Option<Waker>; 2]>>,
    registry: Registry,
}

static REACTOR: OnceLock<Reactor> = OnceLock::new();

impl Reactor {
    pub fn new() -> Result<Self> {
        let poll = Poll::new()?;
        let registry = poll.registry().try_clone()?;

        let wakers = Arc::new(Slab::new());

        // Spawn poll events thread
        thread::Builder::new().name("reactor".into()).spawn({
            let wakers = Arc::clone(&wakers);
            move || poll_events_loop(wakers, poll)
        })?;

        Ok(Self { registry, wakers })
    }

    pub fn get() -> &'static Reactor {
        REACTOR.get().expect("reactor is not set")
    }

    pub fn set_global(self) {
        REACTOR.set(self).ok();
    }

    /// Register interested events for the given source
    pub fn register<S>(
        &self,
        source: &mut S,
        direction: Direction,
        waker: Waker,
    ) -> io::Result<Token>
    where
        S: Source,
    {
        let mut wakers = [None, None];
        wakers[direction as usize] = Some(waker);

        let token = self
            .wakers
            .insert(wakers)
            .ok_or(Error::other("slab queue is full"))?;

        let token = Token(token);

        self.registry.register(source, token, direction.into())?;

        Ok(token)
    }

    /// Add a waker to track an event in one of the directions for an existing
    /// registration.
    pub fn add_direction_for_token<S>(
        &self,
        token: Token,
        source: &mut S,
        direction: Direction,
        waker: Waker,
    ) -> io::Result<Token>
    where
        S: Source,
    {
        let mut prev = self
            .wakers
            .take(token.into())
            .ok_or_else(|| Error::other(format!("token {:?} not found", token)))?;

        prev[direction as usize] = Some(waker);

        let interests = match (
            &prev[Direction::Read as usize],
            &prev[Direction::Write as usize],
        ) {
            (None, Some(_)) => Interest::WRITABLE,
            (Some(_), None) => Interest::READABLE,
            (Some(_), Some(_)) => Interest::READABLE.add(Interest::WRITABLE),
            // We can't reach here in theory, as the function will always receive a waker
            (None, None) => unreachable!(),
        };

        let new_token = Token(
            self.wakers
                .insert(prev)
                .ok_or(Error::other("slab queue is full"))?,
        );

        self.registry.reregister(source, new_token, interests)?;

        Ok(new_token)
    }

    /// Remove a waker from tracking by direction for an existing registration. Returns
    /// `Ok(None)` if no tracking directions are left for the registration and it has been removed.
    pub fn remove_direction_for_token<S>(
        &self,
        token: Token,
        source: &mut S,
        direction: Direction,
    ) -> io::Result<Option<Token>>
    where
        S: Source,
    {
        let mut prev = self
            .wakers
            .take(token.into())
            .ok_or_else(|| Error::other(format!("token {:?} not found", token)))?;

        prev[direction as usize] = None;

        let interests = match (
            &prev[Direction::Read as usize],
            &prev[Direction::Write as usize],
        ) {
            (None, Some(_)) => Interest::WRITABLE,
            (Some(_), None) => Interest::READABLE,
            (Some(_), Some(_)) => Interest::READABLE.add(Interest::WRITABLE),
            (None, None) => {
                self.registry.deregister(source)?;
                return Ok(None);
            }
        };

        let new_token = Token(
            self.wakers
                .insert(prev)
                .ok_or(Error::other("slab queue is full"))?,
        );

        self.registry.reregister(source, new_token, interests)?;

        Ok(Some(new_token))
    }

    /// Remove the interests for the given source
    pub fn deregister<S>(&self, token: Token, source: &mut S) -> io::Result<()>
    where
        S: Source,
    {
        self.registry.deregister(source)?;
        self.wakers.remove(token.0);
        Ok(())
    }
}

fn poll_events_loop(wakers: Arc<Slab<[Option<Waker>; 2]>>, mut poll: Poll) {
    let mut events = Events::with_capacity(1024);

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.into_iter() {
            if let Some(wakers) = wakers.get(event.token().into()) {
                // Call waker interested by this event
                if event.is_readable() {
                    if let Some(w) = &wakers[0] {
                        w.wake_by_ref();
                    }
                } else if event.is_writable() {
                    if let Some(w) = &wakers[1] {
                        w.wake_by_ref();
                    }
                }
            }
        }
    }
}

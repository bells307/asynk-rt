pub mod non_blocking;

pub(crate) mod waker_map;

use mio::{event::Source, Events, Interest, Poll, Registry, Token};
use parking_lot::Mutex;
use slab::Slab;
use std::{
    io::{self, Error, Result},
    sync::{Arc, OnceLock},
    task::Waker,
    thread::{self},
};
use waker_map::WakerMap;

const READ_INTEREST_IDX: usize = 0;
const WRITE_INTEREST_IDX: usize = 1;

/// Reactor polls events from mio and calls wakers interested
/// by these events
pub struct Reactor {
    wakers: Arc<Mutex<Slab<WakerMap>>>,
    registry: Registry,
}

static REACTOR: OnceLock<Reactor> = OnceLock::new();

impl Reactor {
    pub fn new() -> Result<Self> {
        let poll = Poll::new()?;
        let registry = poll.registry().try_clone()?;

        let wakers = Arc::new(Mutex::new(Slab::new()));

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
        interests: Interest,
        waker_map: WakerMap,
    ) -> io::Result<Token>
    where
        S: Source,
    {
        let token = Token(self.wakers.lock().insert(waker_map));
        self.registry.register(source, token, interests)?;
        Ok(token)
    }

    /// Add a waker to track an event in one of the directions for an existing
    /// registration.
    pub fn set_waker(&self, token: Token, interests: Interest, waker: Waker) -> io::Result<()> {
        let mut lock = self.wakers.lock();

        let prev = lock
            .get_mut(token.into())
            .ok_or_else(|| Error::other(format!("token {:?} not found", token)))?;

        prev.set_waker(interests, waker);

        Ok(())
    }

    /// Deregister source
    pub fn deregister<S>(&self, token: Token, source: &mut S) -> io::Result<()>
    where
        S: Source,
    {
        self.registry.deregister(source)?;
        self.wakers.lock().remove(token.0);
        Ok(())
    }
}

fn poll_events_loop(waker_map: Arc<Mutex<Slab<WakerMap>>>, mut poll: Poll) {
    let mut events = Events::with_capacity(1024);

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.into_iter() {
            if let Some(directions) = waker_map.lock().get(event.token().into()) {
                let wakers = directions.wakers();

                // Call waker interested by this event
                if event.is_readable() {
                    if let Some(w) = &wakers[READ_INTEREST_IDX] {
                        w.wake_by_ref();
                    }
                };

                if event.is_writable() {
                    if let Some(w) = &wakers[WRITE_INTEREST_IDX] {
                        w.wake_by_ref();
                    }
                }
            }
        }
    }
}

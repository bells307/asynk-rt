use super::{READ_INTEREST_IDX, WRITE_INTEREST_IDX};
use mio::Interest;
use std::task::Waker;

pub struct WakerMap([Option<Waker>; 2]);

impl WakerMap {
    pub fn new() -> Self {
        Self([None, None])
    }

    pub fn wakers(&self) -> &[Option<Waker>] {
        &self.0
    }

    pub fn set_waker(&mut self, interests: Interest, waker: Waker) {
        if interests.is_writable() && interests.is_readable() {
            self.0[READ_INTEREST_IDX] = Some(waker.clone());
            self.0[WRITE_INTEREST_IDX] = Some(waker);
        } else if interests.is_readable() {
            self.0[READ_INTEREST_IDX] = Some(waker);
        } else if interests.is_writable() {
            self.0[WRITE_INTEREST_IDX] = Some(waker);
        }
    }
}

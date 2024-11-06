use std::task::Waker;

/// Direction of interest tracking - for reading or writing
#[derive(Clone, Copy)]
pub enum Direction {
    Read = 0,
    Write = 1,
}

pub struct WakerMap([Option<Waker>; 2]);

impl WakerMap {
    pub fn new() -> Self {
        Self([None, None])
    }

    pub fn wakers(&self) -> &[Option<Waker>] {
        &self.0
    }

    pub fn set_waker(&mut self, direction: Direction, waker: Waker) {
        self.0[direction as usize] = Some(waker);
    }
}

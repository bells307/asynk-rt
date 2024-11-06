use mio::Interest;

/// Direction of interest tracking - for reading or writing
#[derive(Clone, Copy)]
pub(crate) enum Direction {
    Read = 0,
    Write = 1,
}

impl From<Direction> for Interest {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Read => Interest::READABLE,
            Direction::Write => Interest::WRITABLE,
        }
    }
}

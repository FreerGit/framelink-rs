#[derive(Debug)]
pub enum FrameError {
    CobsDecodeError,
    CrcMismatch,
    BufferTooSmall,
    InvalidLength,
}

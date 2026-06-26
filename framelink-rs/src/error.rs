#[derive(Debug)]
pub enum FrameError {
    CobsDecodeError,
    CrcMismatch,
    BufferTooSmall,
    PayloadTooLarge,
    InvalidLength,
}

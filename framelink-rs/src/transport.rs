#[derive(Debug)]
pub enum TransportError {
    FailedRead,
    FailedWrite,
    FailedFlush,
}

pub trait Transport {
    /// Writes all bytes in `data` to the transport.
    ///
    /// # Errors
    /// Returns [`TransportError::FailedWrite`] if the underlying transport fails to write.
    fn write_bytes(&mut self, data: &[u8]) -> Result<(), TransportError>;

    /// Reads a single byte from the transport.
    ///
    /// # Errors
    /// Returns [`TransportError::FailedRead`] if no data is available or the transport fails.
    fn read_byte(&mut self) -> Result<u8, TransportError>;

    /// Flushes any buffered output to the transport.
    ///
    /// # Errors
    /// Returns [`TransportError::FailedFlush`] if the flush fails.
    fn flush(&mut self) -> Result<(), TransportError>;
}

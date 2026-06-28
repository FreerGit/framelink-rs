pub trait Transport {
    type Error;

    /// Writes all bytes in `data` to the transport.
    ///
    /// # Errors
    /// Returns `Self::Error` if the underlying transport fails to write.
    fn write_bytes(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// Reads a single byte from the transport.
    ///
    /// # Errors
    /// Returns `Self::Error` if no data is available or the transport fails.
    fn read_byte(&mut self) -> Result<u8, Self::Error>;

    /// Flushes any buffered output to the transport.
    ///
    /// # Errors
    /// Returns `Self::Error` if the flush fails.
    fn flush(&mut self) -> Result<(), Self::Error>;
}

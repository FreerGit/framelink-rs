use heapless::Vec;

use crate::{
    frame::{self, Frame, FrameError},
    protocol::State::Idle,
    transport::{self, Transport, TransportError},
};

pub enum ProtocolError {
    MaxRetriesExceeded,

    FrameError(FrameError),
    Transport,
}

impl From<FrameError> for ProtocolError {
    fn from(e: FrameError) -> Self {
        ProtocolError::FrameError(e)
    }
}

pub enum State {
    Idle,
    Sending {
        retries: u8,
    },
    WaitingAck {
        retries: u8,
        last_encoded: heapless::Vec<u8, 256>,
    },
    Error(ProtocolError),
}

pub enum ControlByte {
    Ack = 0xAA,
    Nak = 0xFF,
}

pub struct Protocol<T: Transport, const N: usize, const RETRIES: u8 = 3> {
    transport: T,
    state: State,
    rx_buf: heapless::Vec<u8, 256>,
}

impl<T: Transport, const N: usize, const RETRIES: u8> Protocol<T, N, RETRIES> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            state: State::Idle,
            rx_buf: heapless::Vec::new(),
        }
    }

    pub fn send(&mut self, payload: &[u8]) -> Result<(), ProtocolError> {
        let frame = Frame::<N>::new(payload)?;

        let mut buf = Vec::<u8, 256>::new();
        let n = frame.encode(&mut buf)?;

        self.transport
            .write_bytes(&buf[..n])
            .map_err(|_| ProtocolError::Transport)?;

        self.state = State::WaitingAck {
            retries: RETRIES,
            last_encoded: buf,
        };

        Ok(())
    }

    pub fn poll(&mut self) -> Result<Option<Vec<u8, N>>, ProtocolError> {
        while let Ok(byte) = self.transport.read_byte() {
            self.rx_buf
                .push(byte)
                .map_err(|_| FrameError::BufferTooSmall)?;

            if byte == 0x00 {
                match self.rx_buf.len() {
                    1 if self.rx_buf[0] == 0xAA => {
                        self.rx_buf.clear();
                        self.state = Idle;
                    }
                    1 if self.rx_buf[0] == 0xFF => {
                        self.rx_buf.clear();
                        let state = core::mem::replace(&mut self.state, State::Idle);
                        match state {
                            State::WaitingAck {
                                retries,
                                last_encoded,
                            } if retries > 0 => {
                                self.transport
                                    .write_bytes(&last_encoded)
                                    .map_err(|_| ProtocolError::Transport)?;
                                self.state = State::WaitingAck {
                                    retries: retries - 1,
                                    last_encoded,
                                };
                            }
                            _ => return Err(ProtocolError::MaxRetriesExceeded),
                        }
                    }
                    _ => {
                        match Frame::<N>::decode(&self.rx_buf) {
                            Ok(frame) => {
                                self.rx_buf.clear();
                                self.send_ack()?;
                                return Ok(Some(frame.payload()));
                            }
                            Err(FrameError::CrcMismatch) => {
                                self.rx_buf.clear();
                                self.send_nak()?;
                                // don't return — keep polling
                            }
                            Err(e) => {
                                self.rx_buf.clear();
                                return Err(ProtocolError::FrameError(e));
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    fn send_ack(&mut self) -> Result<(), ProtocolError> {
        self.transport
            .write_bytes(&[0xAA, 0x00])
            .map_err(|_| ProtocolError::Transport)
    }

    fn send_nak(&mut self) -> Result<(), ProtocolError> {
        self.transport
            .write_bytes(&[0xFF, 0x00])
            .map_err(|_| ProtocolError::Transport)
    }
}

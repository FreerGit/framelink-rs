use heapless::Vec;

use crate::error::FrameError;

pub struct Frame<const N: usize> {
    payload: heapless::Vec<u8, N>,
}

impl<const N: usize> Frame<N> {
    pub fn new(payload: &[u8]) -> Result<Self, FrameError> {
        match Vec::from_slice(payload) {
            Ok(payload) => Ok(Frame { payload }),
            Err(_) => Err(FrameError::BufferTooSmall),
        }
    }

    pub fn encode(&self, out: &mut [u8]) -> Result<usize, FrameError> {
        todo!()
    }

    pub fn decode(raw: &[u8]) -> Result<Self, FrameError> {
        todo!()
    }
}

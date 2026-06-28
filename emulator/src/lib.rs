use core::iter::Iterator;
use core::option::Option::{self, None, Some};
use core::result::Result;
use core::result::Result::Ok;

use framelink_rs::transport::Transport;
use heapless::spsc::{Consumer, Producer, Queue};

#[derive(Default)]
pub struct UartEmulator {
    a_to_b: Queue<u8, 256>,
    b_to_a: Queue<u8, 256>,
}

impl UartEmulator {
    pub fn split(&mut self) -> (UartSide<'_>, UartSide<'_>) {
        let (a_tx, b_rx) = self.a_to_b.split();
        let (b_tx, a_rx) = self.b_to_a.split();
        (
            UartSide {
                tx: a_tx,
                rx: a_rx,
                corrupt_at: None,
            },
            UartSide {
                tx: b_tx,
                rx: b_rx,
                corrupt_at: None,
            },
        )
    }
}

pub struct UartSide<'a> {
    tx: Producer<'a, u8, 256>, // writes into a_to_b
    rx: Consumer<'a, u8, 256>, // reads from b_to_a
    corrupt_at: Option<usize>,
}

#[derive(Debug)]
pub enum TransportError {
    FailedRead,
    FailedWrite,
}

impl UartSide<'_> {
    pub fn corrupt_byte_at(&mut self, offset: usize) {
        self.corrupt_at = Some(offset);
    }
}

impl Transport for UartSide<'_> {
    type Error = TransportError;

    fn write_bytes(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        for (i, &b) in data.iter().enumerate() {
            let byte = match self.corrupt_at {
                Some(pos) if pos == i => {
                    self.corrupt_at = None;
                    b ^ 0xFF // flip all bits
                }
                _ => b,
            };
            self.tx
                .enqueue(byte)
                .map_err(|_| TransportError::FailedWrite)?;
        }
        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8, Self::Error> {
        self.rx.dequeue().ok_or(TransportError::FailedRead)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use framelink_rs::transport::Transport;

    #[test]
    fn side_a_sends_side_b_receives() {
        let mut emulator = UartEmulator::default();
        let (mut a, mut b) = emulator.split();
        a.write_bytes(&[0x01, 0x02, 0x03]).unwrap();
        assert_eq!(b.read_byte().unwrap(), 0x01);
        assert_eq!(b.read_byte().unwrap(), 0x02);
        assert_eq!(b.read_byte().unwrap(), 0x03);
    }

    #[test]
    fn corruption_flips_byte() {
        let mut emulator = UartEmulator::default();
        let (mut a, mut b) = emulator.split();
        a.corrupt_byte_at(1);
        a.write_bytes(&[0x01, 0x02, 0x03]).unwrap();
        assert_eq!(b.read_byte().unwrap(), 0x01);
        assert_eq!(b.read_byte().unwrap(), 0x02 ^ 0xFF); // corrupted
        assert_eq!(b.read_byte().unwrap(), 0x03);
    }
}

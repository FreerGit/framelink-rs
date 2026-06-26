use heapless::Vec;

use crate::error::FrameError;

pub struct Frame<const N: usize> {
    payload: heapless::Vec<u8, N>,
}

impl<const N: usize> Frame<N> {
    /// # Errors
    /// Returns [`FrameError::PayloadTooLarge`] if `payload.len() > N`.
    pub fn new(payload: &[u8]) -> Result<Self, FrameError> {
        Vec::from_slice(payload).map_or_else(
            |_| Err(FrameError::PayloadTooLarge),
            |payload| Ok(Self { payload }),
        )
    }

    /// # Errors
    /// Returns [`FrameError::BufferTooSmall`] if the `out` buffer is too small
    pub fn encode(&self, out: &mut [u8]) -> Result<usize, FrameError> {
        let mut write_pos = 1;
        let mut header_pos = 0;
        let mut distance = 1;

        for &b in &self.payload {
            if b == 0 {
                // write distance into current overhead slot
                *out.get_mut(header_pos).ok_or(FrameError::BufferTooSmall)? = distance;
                header_pos = write_pos;
                distance = 1; // reset, counting the new overhead byte itself
            } else {
                *out.get_mut(write_pos).ok_or(FrameError::BufferTooSmall)? = b;
                distance += 1;
            }

            write_pos += 1;
        }

        // close final overhead slot
        *out.get_mut(header_pos).ok_or(FrameError::BufferTooSmall)? = distance;

        // terminator
        *out.get_mut(write_pos).ok_or(FrameError::BufferTooSmall)? = 0x00;
        write_pos += 1;

        Ok(write_pos)
    }
    /// # Errors
    ///
    /// Will return `FrameError` if the `raw` buffer is inherently incorrect.
    /// For example being much smaller than the inital payload
    pub fn decode(raw: &[u8]) -> Result<Self, FrameError> {
        let mut payload: heapless::Vec<u8, N> = heapless::Vec::new();
        let mut i = 0;

        loop {
            let code = *raw.get(i).ok_or(FrameError::InvalidLength)?;
            i += 1;

            if code == 0x00 {
                return Err(FrameError::InvalidLength);
            }

            for _ in 0..(code - 1) {
                let b = *raw.get(i).ok_or(FrameError::InvalidLength)?;
                i += 1;
                payload.push(b).map_err(|_| FrameError::BufferTooSmall)?;
            }

            match raw.get(i) {
                Some(&0x00) => break,
                Some(_) => payload.push(0x00).map_err(|_| FrameError::BufferTooSmall)?,
                None => return Err(FrameError::InvalidLength),
            }
        }

        // todo: strip and verify CRC from end of payload

        Ok(Self { payload })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // helper — creates a Frame and encodes it, panics if either step fails
    fn encode_payload<const N: usize>(payload: &[u8], out: &mut [u8]) -> usize {
        Frame::<N>::new(payload)
            .expect("Frame::new failed")
            .encode(out)
            .expect("encode failed")
    }

    #[test]
    fn encoded_output_ends_with_zero() {
        let mut out = [0u8; 32];
        let n = encode_payload::<16>(&[0x11, 0x22, 0x33], &mut out);
        assert_eq!(out[n - 1], 0x00, "last byte must be the COBS terminator");
    }

    #[test]
    fn encoded_output_has_no_zero_before_terminator() {
        let mut out = [0u8; 32];
        let n = encode_payload::<16>(&[0x11, 0x00, 0x22], &mut out);
        // every byte except the last must be non-zero
        for (i, &b) in out[..n - 1].iter().enumerate() {
            assert_ne!(b, 0x00, "unexpected 0x00 at position {i} before terminator");
        }
    }

    #[test]
    fn empty_payload_encodes() {
        // empty payload + 2 CRC bytes, all non-zero after COBS
        let mut out = [0u8; 16];
        let n = encode_payload::<16>(&[], &mut out);
        assert!(n > 0);
        assert_eq!(out[n - 1], 0x00);
    }

    #[test]
    fn encoded_length_no_zeros_in_payload() {
        // payload with no zeros: COBS overhead = 1 byte, CRC = 2 bytes, terminator = 1 byte
        // total = payload.len() + 4
        let payload = [0x11u8; 8];
        let mut out = [0u8; 32];
        let n = encode_payload::<16>(&payload, &mut out);
        assert_eq!(n, payload.len() + 4);
    }

    #[test]
    fn encode_returns_err_when_out_too_small() {
        let frame = Frame::<16>::new(&[0x11, 0x22, 0x33]).unwrap();
        let mut tiny = [0u8; 2]; // nowhere near large enough
        assert!(matches!(
            frame.encode(&mut tiny),
            Err(FrameError::BufferTooSmall)
        ));
    }

    // --- round-trip tests ---

    #[test]
    fn roundtrip_no_zeros() {
        let payload = &[0x11u8, 0x22, 0x33];
        let mut out = [0u8; 32];
        let n = encode_payload::<16>(payload, &mut out);
        let frame = Frame::<16>::decode(&out[..n]).expect("decode failed");
        assert_eq!(frame.payload.as_slice(), payload);
    }

    #[test]
    fn roundtrip_payload_containing_zeros() {
        let payload = &[0x11u8, 0x00, 0x22, 0x00, 0x33];
        let mut out = [0u8; 32];
        let n = encode_payload::<16>(payload, &mut out);
        let frame = Frame::<16>::decode(&out[..n]).expect("decode failed");
        assert_eq!(frame.payload.as_slice(), payload);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    extern crate std;
    proptest! {
        #[test]
        fn roundtrip_any_payload(payload in proptest::collection::vec(any::<u8>(), 0..64)) {
            let mut out = [0u8; 256];
            let frame = Frame::<64>::new(&payload).unwrap();
            let n = frame.encode(&mut out).unwrap();
            let decoded = Frame::<64>::decode(&out[..n]).unwrap();
            prop_assert_eq!(decoded.payload.as_slice(), payload.as_slice());
        }

        #[test]
        fn roundtrip_no_zeros_in_payload(payload in proptest::collection::vec(1u8..=255u8, 0..64)) {
            let mut out = [0u8; 256];
            let frame = Frame::<64>::new(&payload).unwrap();
            let n = frame.encode(&mut out).unwrap();
            let decoded = Frame::<64>::decode(&out[..n]).unwrap();
            prop_assert_eq!(decoded.payload.as_slice(), payload.as_slice());
        }

        #[test]
        fn roundtrip_all_zeros(len in 0usize..64) {
            let payload: std::vec::Vec<u8> = std::vec![0u8; len];
            let mut out = [0u8; 256];
            let frame = Frame::<64>::new(&payload).unwrap();
            let n = frame.encode(&mut out).unwrap();
            let decoded = Frame::<64>::decode(&out[..n]).unwrap();
            prop_assert_eq!(decoded.payload.as_slice(), payload.as_slice());
        }

        #[test]
        fn encoded_output_never_contains_zero_before_terminator(
            payload in proptest::collection::vec(any::<u8>(), 0..64)
        ) {
            let mut out = [0u8; 256];
            let frame = Frame::<64>::new(&payload).unwrap();
            let n = frame.encode(&mut out).unwrap();
            let encoded = &out[..n];
            // every byte except the last must be non-zero
            for (i, &b) in encoded[..n-1].iter().enumerate() {
                prop_assert_ne!(b, 0x00, "zero found at position {} before terminator", i);
            }
        }

        #[test]
        fn encoded_length_is_bounded(payload in proptest::collection::vec(any::<u8>(), 0..64)) {
            let mut out = [0u8; 256];
            let frame = Frame::<64>::new(&payload).unwrap();
            let n = frame.encode(&mut out).unwrap();
            // max encoded length: payload + 2 CRC bytes + ceil(n/254) overhead + 1 terminator
            let max_len = payload.len() + 2 + 1 + 1;
            prop_assert!(n <= max_len, "encoded length {n} exceeded bound {max_len}");
        }
    }
}

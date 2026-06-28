use heapless::Vec;

use crate::error::FrameError;

pub struct Frame<const N: usize> {
    payload: heapless::Vec<u8, N>,
}

/// <https://www.sput.nl/internet/crc.html>
const CRC16_TABLE: [u16; 256] = [
    0x0000, 0x1021, 0x2042, 0x3063, 0x4084, 0x50A5, 0x60C6, 0x70E7, 0x8108, 0x9129, 0xA14A, 0xB16B,
    0xC18C, 0xD1AD, 0xE1CE, 0xF1EF, 0x1231, 0x0210, 0x3273, 0x2252, 0x52B5, 0x4294, 0x72F7, 0x62D6,
    0x9339, 0x8318, 0xB37B, 0xA35A, 0xD3BD, 0xC39C, 0xF3FF, 0xE3DE, 0x2462, 0x3443, 0x0420, 0x1401,
    0x64E6, 0x74C7, 0x44A4, 0x5485, 0xA56A, 0xB54B, 0x8528, 0x9509, 0xE5EE, 0xF5CF, 0xC5AC, 0xD58D,
    0x3653, 0x2672, 0x1611, 0x0630, 0x76D7, 0x66F6, 0x5695, 0x46B4, 0xB75B, 0xA77A, 0x9719, 0x8738,
    0xF7DF, 0xE7FE, 0xD79D, 0xC7BC, 0x48C4, 0x58E5, 0x6886, 0x78A7, 0x0840, 0x1861, 0x2802, 0x3823,
    0xC9CC, 0xD9ED, 0xE98E, 0xF9AF, 0x8948, 0x9969, 0xA90A, 0xB92B, 0x5AF5, 0x4AD4, 0x7AB7, 0x6A96,
    0x1A71, 0x0A50, 0x3A33, 0x2A12, 0xDBFD, 0xCBDC, 0xFBBF, 0xEB9E, 0x9B79, 0x8B58, 0xBB3B, 0xAB1A,
    0x6CA6, 0x7C87, 0x4CE4, 0x5CC5, 0x2C22, 0x3C03, 0x0C60, 0x1C41, 0xEDAE, 0xFD8F, 0xCDEC, 0xDDCD,
    0xAD2A, 0xBD0B, 0x8D68, 0x9D49, 0x7E97, 0x6EB6, 0x5ED5, 0x4EF4, 0x3E13, 0x2E32, 0x1E51, 0x0E70,
    0xFF9F, 0xEFBE, 0xDFDD, 0xCFFC, 0xBF1B, 0xAF3A, 0x9F59, 0x8F78, 0x9188, 0x81A9, 0xB1CA, 0xA1EB,
    0xD10C, 0xC12D, 0xF14E, 0xE16F, 0x1080, 0x00A1, 0x30C2, 0x20E3, 0x5004, 0x4025, 0x7046, 0x6067,
    0x83B9, 0x9398, 0xA3FB, 0xB3DA, 0xC33D, 0xD31C, 0xE37F, 0xF35E, 0x02B1, 0x1290, 0x22F3, 0x32D2,
    0x4235, 0x5214, 0x6277, 0x7256, 0xB5EA, 0xA5CB, 0x95A8, 0x8589, 0xF56E, 0xE54F, 0xD52C, 0xC50D,
    0x34E2, 0x24C3, 0x14A0, 0x0481, 0x7466, 0x6447, 0x5424, 0x4405, 0xA7DB, 0xB7FA, 0x8799, 0x97B8,
    0xE75F, 0xF77E, 0xC71D, 0xD73C, 0x26D3, 0x36F2, 0x0691, 0x16B0, 0x6657, 0x7676, 0x4615, 0x5634,
    0xD94C, 0xC96D, 0xF90E, 0xE92F, 0x99C8, 0x89E9, 0xB98A, 0xA9AB, 0x5844, 0x4865, 0x7806, 0x6827,
    0x18C0, 0x08E1, 0x3882, 0x28A3, 0xCB7D, 0xDB5C, 0xEB3F, 0xFB1E, 0x8BF9, 0x9BD8, 0xABBB, 0xBB9A,
    0x4A75, 0x5A54, 0x6A37, 0x7A16, 0x0AF1, 0x1AD0, 0x2AB3, 0x3A92, 0xFD2E, 0xED0F, 0xDD6C, 0xCD4D,
    0xBDAA, 0xAD8B, 0x9DE8, 0x8DC9, 0x7C26, 0x6C07, 0x5C64, 0x4C45, 0x3CA2, 0x2C83, 0x1CE0, 0x0CC1,
    0xEF1F, 0xFF3E, 0xCF5D, 0xDF7C, 0xAF9B, 0xBFBA, 0x8FD9, 0x9FF8, 0x6E17, 0x7E36, 0x4E55, 0x5E74,
    0x2E93, 0x3EB2, 0x0ED1, 0x1EF0,
];

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

        let crc = Self::crc16(&self.payload);
        let crc_hi_lo = [(crc >> 8) as u8, (crc & 0xFF) as u8];

        for &b in self.payload.iter().chain(crc_hi_lo.iter()) {
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

        // sliding window — holds the last 2 decoded bytes
        // at end of frame these will be [CRC_HI, CRC_LO]
        let mut window = [0u8; 2];
        let mut window_len = 0usize; // how many bytes are in the window (0, 1, or 2)

        let mut push = |payload: &mut heapless::Vec<u8, N>, byte: u8| -> Result<(), FrameError> {
            if window_len == 2 {
                // window is full — flush oldest byte to payload
                payload
                    .push(window[0])
                    .map_err(|_| FrameError::BufferTooSmall)?;
                window[0] = window[1];
                window[1] = byte;
            } else {
                window[window_len] = byte;
                window_len += 1;
            }
            Ok(())
        };

        loop {
            let code = *raw.get(i).ok_or(FrameError::InvalidLength)?;
            i += 1;

            if code == 0x00 {
                return Err(FrameError::InvalidLength);
            }

            for _ in 0..(code - 1) {
                let b = *raw.get(i).ok_or(FrameError::InvalidLength)?;
                i += 1;
                push(&mut payload, b)?;
            }

            match raw.get(i) {
                Some(&0x00) => break,
                Some(_) => push(&mut payload, 0x00)?,
                None => return Err(FrameError::InvalidLength),
            }
        }

        let crc_hi = window[0];
        let crc_lo = window[1];

        let received_crc = (u16::from(crc_hi)) << 8 | u16::from(crc_lo);
        let computed_crc = Self::crc16(&payload);
        if received_crc != computed_crc {
            return Err(FrameError::CrcMismatch);
        }

        let payload =
            heapless::Vec::from_slice(&payload).map_err(|_| FrameError::PayloadTooLarge)?;

        Ok(Self { payload })
    }

    #[must_use]
    pub fn crc16(data: &[u8]) -> u16 {
        let mut crc: u16 = 0xFFFF;
        for &b in data {
            let pos = (((crc >> 8) as u8) ^ b) as usize;
            crc = (crc << 8) ^ CRC16_TABLE[pos];
        }
        crc
    }
}

#[cfg(test)]
mod encode_decode_tests {
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
        let n = encode_payload::<3>(&[0x11, 0x00, 0x22], &mut out);
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
mod encode_decode_proptests {
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

        #[test]
        fn corrupted_byte_returns_crc_mismatch(payload in proptest::collection::vec(any::<u8>(), 0..64)) {
            let mut out = [0u8; 256];
            let frame = Frame::<64>::new(&payload).unwrap();
            let n = frame.encode(&mut out).unwrap();

            // flip a bit in the middle of the frame (not the terminator)
            out[n / 2] ^= 0xFF;

            assert!(matches!(
                Frame::<64>::decode(&out[..n]),
                Err(FrameError::CrcMismatch) | Err(FrameError::InvalidLength)
            ));
        }
    }
}

#[cfg(test)]
mod crc16_tests {
    use super::*;

    #[test]
    fn crc16_table_is_correct() {
        // generate the table from scratch using bit-by-bit method
        // and verify it matches our lookup table
        for (i, &entry) in CRC16_TABLE.iter().enumerate() {
            let mut crc = (i as u16) << 8;
            for _ in 0..8 {
                if crc & 0x8000 != 0 {
                    crc = (crc << 1) ^ 0x1021; // CCITT polynomial
                } else {
                    crc <<= 1;
                }
            }
            assert_eq!(entry, crc, "table entry {i} is wrong -> {:#X?}", crc);
        }
    }

    #[test]
    fn crc16_known_vector() {
        // "123456789" in ASCII = 9 bytes
        // standard check value for CRC-16/CCITT-FALSE
        let input = b"123456789";
        assert_eq!(Frame::<9>::crc16(input), 0x29B1);
    }
}

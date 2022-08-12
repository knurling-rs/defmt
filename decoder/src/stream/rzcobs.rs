use super::StreamDecoder;
use crate::{DecodeError, Frame, Table};

/// Decode a full message.
///
/// `data` must be a full rzCOBS encoded message. Decoding partial
/// messages is not possible. `data` must NOT include any `0x00` separator byte.
fn rzcobs_decode(data: &[u8]) -> Result<Vec<u8>, DecodeError> {
    let mut res = vec![];
    let mut data = data.iter().rev().cloned();
    while let Some(x) = data.next() {
        match x {
            0 => return Err(DecodeError::Malformed),
            0x01..=0x7f => {
                for i in 0..7 {
                    if x & (1 << (6 - i)) == 0 {
                        res.push(data.next().ok_or(DecodeError::Malformed)?);
                    } else {
                        res.push(0);
                    }
                }
            }
            0x80..=0xfe => {
                let n = (x & 0x7f) + 7;
                res.push(0);
                for _ in 0..n {
                    res.push(data.next().ok_or(DecodeError::Malformed)?);
                }
            }
            0xff => {
                for _ in 0..134 {
                    res.push(data.next().ok_or(DecodeError::Malformed)?);
                }
            }
        }
    }

    res.reverse();
    Ok(res)
}

pub struct Rzcobs<'a> {
    table: &'a Table,
    raw: Vec<u8>,
}

impl<'a> Rzcobs<'a> {
    pub fn new(table: &'a Table) -> Self {
        Self {
            table,
            raw: Vec::new(),
        }
    }
}

impl<'a> StreamDecoder for Rzcobs<'a> {
    fn received(&mut self, mut data: &[u8]) {
        // Trim zeros from the left, start storing at first non-zero byte.
        if self.raw.is_empty() {
            while data.first() == Some(&0) {
                data = &data[1..]
            }
        }

        self.raw.extend_from_slice(data);
    }

    fn decode(&mut self) -> Result<Frame<'_>, DecodeError> {
        // Find frame separator. If not found, we don't have enough data yet.
        let zero = self
            .raw
            .iter()
            .position(|&x| x == 0)
            .ok_or(DecodeError::UnexpectedEof)?;

        let frame = rzcobs_decode(&self.raw[..zero]);

        // Even if it failed, pop the data off so we don't get stuck.
        // Pop off the frame + 1 or more separator zero-bytes
        if let Some(nonzero) = self.raw[zero..].iter().position(|&x| x != 0) {
            self.raw.drain(0..zero + nonzero);
        } else {
            self.raw.clear();
        }

        assert!(self.raw.is_empty() || self.raw[0] != 0);

        let frame: Vec<u8> = frame?;
        match self.table.decode(&frame) {
            Ok((frame, _consumed)) => Ok(frame),
            Err(DecodeError::UnexpectedEof) => Err(DecodeError::Malformed),
            Err(DecodeError::Malformed) => Err(DecodeError::Malformed),
        }
    }
}

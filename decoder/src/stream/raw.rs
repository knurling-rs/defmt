use super::StreamDecoder;
use crate::{DecodeError, Frame, Table};

pub struct Raw<'a> {
    table: &'a Table,
    data: Vec<u8>,
}

impl<'a> Raw<'a> {
    pub fn new(table: &'a Table) -> Self {
        Self {
            table,
            data: Vec::new(),
        }
    }
}

impl<'a> StreamDecoder for Raw<'a> {
    fn received(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    fn decode(&mut self) -> Result<Frame<'_>, DecodeError> {
        match self.table.decode(&self.data) {
            Ok((frame, consumed)) => {
                self.data.drain(0..consumed);
                Ok(frame)
            }
            Err(e) => Err(e),
        }
    }
}

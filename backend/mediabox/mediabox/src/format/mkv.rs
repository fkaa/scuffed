use async_trait::async_trait;

use crate::{Stream, Packet, format::{Demuxer}, io::{Io}};

#[derive(thiserror::Error, Debug)]
pub enum MkvError {
    #[error("Not enough data")]
    NotEnoughData,

    #[error("Unsupported variable integer size: {0}")]
    UnsupportedVint(u8),

    #[error("Unsupported variable integer ID: {0}")]
    UnsupportedVid(u8),A

    #[error("Expected 0x{0:08x} but found 0x{1:08x}")]
    UnexpectedId(u32, u32),

    #[error("{0}")]
    Io(#[from] crate::io::IoError),

    #[error("{0}")]
    StdIo(#[from] std::io::Error),
}

const EBML_HEADER: u32 = 0x1a45dfa3;

async fn vint(io: &mut Io) -> Result<u64, MkvError> {
    use tokio::io::AsyncReadExt;

    let reader = io.reader()?;

    let byte = reader.read_u8().await?;
    let extra_bytes = byte.leading_zeros() as u8;
    let len = 1 + extra_bytes as usize;

    if extra_bytes > 7 {
        return Err(MkvError::UnsupportedVint(extra_bytes));
    }

    let mut bytes = [0u8; 7];
    if extra_bytes > 0 {
        reader.read_exact(&mut bytes[..extra_bytes as usize]).await?;
    }

    let mut value = byte as u64 & ((1 << (8 - len)) - 1) as u64;

    for i in 0..extra_bytes {
        value <<= 8;
        value |= bytes[i as usize] as u64;
    }

    Ok(value)
}

async fn vid(io: &mut Io) -> Result<u32, MkvError> {
    use tokio::io::AsyncReadExt;

    let reader = io.reader()?;

    let byte = reader.read_u8().await?;
    let extra_bytes = byte.leading_zeros() as u8;
    let len = 1 + extra_bytes as usize;

    if extra_bytes > 3 {
        return Err(MkvError::UnsupportedVid(extra_bytes));
    }

    let mut bytes = [0u8; 3];
    if extra_bytes > 0 {
        reader.read_exact(&mut bytes[..extra_bytes as usize]).await?;
    }

    let mut value = byte as u32;

    for i in 0..extra_bytes {
        value <<= 8;
        value |= bytes[i as usize] as u32;
    }

    Ok(value)
}

pub struct MatroskaDemuxer {
    io: Io,
}

impl MatroskaDemuxer {
    pub fn new(io: Io) -> Self {
        MatroskaDemuxer {
            io
        }
    }

    async fn parse_ebml_header(&mut self) -> Result<(), MkvError> {
        let id = vid(&mut self.io).await?;
        let size = vint(&mut self.io).await?;

        if id != EBML_HEADER {
            return Err(MkvError::UnexpectedId(EBML_HEADER, id));
        }
        

        Ok(())
    }
}

#[async_trait]
impl Demuxer for MatroskaDemuxer {
    async fn start(&mut self) -> anyhow::Result<Vec<Stream>> {
        loop {
            let id = vid(&mut self.io).await?;
            let size = vint(&mut self.io).await?;

            println!("0x{id:08x} ({size} B)");

            self.io.skip(size).await?;
        }
    }

    async fn read(&mut self) -> anyhow::Result<Packet> {
        todo!()
    }

    async fn stop(&mut self) -> anyhow::Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use assert_matches::assert_matches;
    use test_case::test_case;
    use std::io::Cursor;

    #[test_case(&[0b1000_0010], 2)]
    #[test_case(&[0b0100_0000, 0b0000_0010], 2)]
    #[test_case(&[0b0010_0000, 0b0000_0000, 0b0000_0010], 2)]
    #[test_case(&[0b0001_0000, 0b0000_0000, 0b0000_0000, 0b0000_0010], 2)]
    #[tokio::test]
    async fn vint(bytes: &[u8], expected: u64) {
        let cursor = Cursor::new(bytes.to_vec());
        let mut io = Io::from_reader(Box::new(cursor));

        let value = super::vint(&mut io).await;

        assert_matches!(value, Ok(expected));
    }
}

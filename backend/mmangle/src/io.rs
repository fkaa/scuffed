use tokio::{
    fs::File,
    io::{AsyncSeek, AsyncWrite},
};

use downcast::{downcast, Any};

use std::{io::SeekFrom, path::Path};

use crate::Span;

pub trait WriteSeek: Any + AsyncWrite + AsyncSeek + Unpin + Sync + Send + 'static {}
pub trait Write: Any + AsyncWrite + Unpin + Sync + Send {}

downcast!(dyn WriteSeek);
downcast!(dyn Write);

impl<T> WriteSeek for T where T: AsyncWrite + AsyncSeek + Unpin + Sync + Send + 'static {}
impl<T> Write for T where T: AsyncWrite + Unpin + Sync + Send + 'static {}

pub enum Writer {
    Seekable(Box<dyn WriteSeek>),
    Stream(Box<dyn Write>),
}

#[derive(Debug, thiserror::Error)]
pub enum IoError {
    #[error("Stream is not writeable")]
    NotWriteable,

    #[error("Stream is not seekable")]
    NotSeekable,

    #[error("{0}")]
    Io(#[from] std::io::Error),
}

pub struct Io {
    writer: Option<Writer>,
}

impl Io {
    pub fn null() -> Self {
        Io {
            writer: None,
        }
    }

    pub async fn open_file<P: AsRef<Path>>(path: P) -> Result<Self, IoError> {
        let file = File::create(path).await?;

        Ok(Io {
            writer: Some(Writer::Seekable(Box::new(file))),
        })
    }
    pub fn from_stream(writer: Box<dyn Write>) -> Self {
        Io {
            writer: Some(Writer::Stream(writer)),
        }
    }

    pub async fn write_span(&mut self, span: Span) -> Result<(), IoError> {
        use tokio::io::AsyncWriteExt;

        let writer = self.writer.as_mut().ok_or(IoError::NotWriteable)?;

        match writer {
            Writer::Seekable(writer) => {
                // TODO: replace with write_vectored
                for span in span.spans() {
                    writer.write_all(&span).await?
                }
            }
            Writer::Stream(writer) => {
                // TODO: replace with write_vectored
                for span in span.spans() {
                    writer.write_all(&span).await?
                }
            }
        };

        Ok(())
    }

    pub async fn write(&mut self, bytes: &[u8]) -> Result<(), IoError> {
        use tokio::io::AsyncWriteExt;

        let writer = self.writer.as_mut().ok_or(IoError::NotWriteable)?;

        match writer {
            Writer::Seekable(writer) => writer.write_all(bytes).await?,
            Writer::Stream(writer) => writer.write_all(bytes).await?,
        }

        Ok(())
    }

    pub async fn seek(&mut self, pos: SeekFrom) -> Result<u64, IoError> {
        use tokio::io::AsyncSeekExt;

        let writer = self.writer.as_mut().ok_or(IoError::NotWriteable)?;

        let pos = match writer {
            Writer::Seekable(writer) => writer.seek(pos).await?,
            _ => return Err(IoError::NotSeekable)?,
        };

        Ok(pos)
    }

    pub fn seekable(&self) -> bool {
        matches!(self.writer, Some(Writer::Seekable(_)))
    }

    pub fn into_writer<T: 'static>(&mut self) -> Result<Box<T>, IoError> {
        let writer = self.writer.take().ok_or(IoError::NotWriteable)?;

        let writer = match writer {
            Writer::Seekable(writer) => writer
                .downcast::<T>()
                .expect("Invalid write type requested"),
            Writer::Stream(writer) => writer
                .downcast::<T>()
                .expect("Invalid write type requested"),
        };

        Ok(writer)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;

    #[test_case(&[b"abc"], b"abc")]
    #[test_case(&[b"a", b"b", b"c"], b"abc")]
    #[tokio::test]
    async fn io_write(spans: &[&[u8]], expected: &[u8]) {
        let buf: Vec<u8> = Vec::new();

        let mut io = Io::from_stream(Box::new(buf));
        for span in spans {
            io.write(span).await.unwrap();
        }

        let buf: Box<Vec<u8>> = io.into_writer().unwrap();

        assert_eq!(expected, *buf);
    }
}

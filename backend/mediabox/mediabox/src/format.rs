use async_trait::async_trait;

use crate::{io::Io, Packet, Stream};

pub mod mkv;
pub mod mp4;
pub mod rtmp;

pub trait MuxerMetadata {
    fn create(io: Io) -> Self;
    fn name() -> &'static str;
}

/// A trait for exposing functionality related to muxing together multiple streams into a container
/// format.
#[async_trait]
pub trait Muxer {
    /// Starts the muxer with the given streams.
    async fn start(&mut self, streams: Vec<Stream>) -> anyhow::Result<()>;

    /// Writes a packet to the muxer.
    ///
    /// Note that this does not ensure something will be written to the output, as it may buffer
    /// packets internally in order to write its output correctly.
    async fn write(&mut self, packet: Packet) -> anyhow::Result<()>;

    /// Stops the muxer. This will flush any buffered packets and finalize the output if
    /// appropriate.
    async fn stop(&mut self) -> anyhow::Result<()>;
}

#[async_trait]
pub trait Demuxer {
    async fn start(&mut self) -> anyhow::Result<Vec<Stream>>;
    async fn read(&mut self) -> anyhow::Result<Packet>;
    async fn stop(&mut self) -> anyhow::Result<()>;
}

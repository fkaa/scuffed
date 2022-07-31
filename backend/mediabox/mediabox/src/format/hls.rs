use std::time::Duration;

use async_trait::async_trait;

use crate::{io::Io, Packet, Track};

use super::{Movie, Muxer};

/// A muxer for the *HTTP Live Streaming* (HLS) format/protocol.
///
/// *Note* that HLS is not just one file, but consists of several playlist files and multiple
/// media segment files.
pub struct HlsMuxer {
    master_playlist: Io,
}

impl HlsMuxer {
    pub fn new_stream(&mut self, movie: Movie) -> HlsStreamMuxer {
        todo!()
    }
}

pub struct HlsStreamMuxer {
    playlist: Io,
    segment_idx: u32,
    segment_duration: Duration,
}

#[async_trait]
impl Muxer for HlsStreamMuxer {
    async fn start(&mut self, streams: Vec<Track>) -> anyhow::Result<()> {
        todo!()
    }

    async fn write(&mut self, packet: Packet) -> anyhow::Result<()> {
        todo!()
    }

    async fn stop(&mut self) -> anyhow::Result<()> {
        todo!()
    }
}

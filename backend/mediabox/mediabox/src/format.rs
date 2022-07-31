use std::cmp::Ordering;

use async_trait::async_trait;

use crate::{io::Io, Packet, Span, Track};

pub mod hls;
pub mod mkv;
pub mod mp4;
pub mod rtmp;
pub mod webvtt;

/// Registers a demuxer with mediabox
#[macro_export]
macro_rules! demuxer {
    ($name:literal, $create:expr, $probe:expr) => {
        const META: $crate::format::DemuxerMetadata = $crate::format::DemuxerMetadata {
            name: $name,
            create: $create,
            probe: $probe,
        };

        inventory::submit!(META);
    };
}

/// Registers a muxer with mediabox
#[macro_export]
macro_rules! muxer {
    ($name:literal, $create:expr) => {
        const META: $crate::format::MuxerMetadata = $crate::format::MuxerMetadata {
            name: $name,
            create: $create,
        };

        inventory::submit!(META);
    };
}

inventory::collect!(DemuxerMetadata);
inventory::collect!(MuxerMetadata);

#[async_trait]
pub trait Demuxer {
    async fn start(&mut self) -> anyhow::Result<Movie>;
    async fn read(&mut self) -> anyhow::Result<Packet>;
    async fn stop(&mut self) -> anyhow::Result<()>;

    fn create(io: Io) -> Box<dyn Demuxer>
    where
        Self: Sized;

    fn probe(data: &[u8]) -> ProbeResult
    where
        Self: Sized,
    {
        ProbeResult::Unsure
    }
}

/// A trait for exposing functionality related to muxing together multiple streams into a container
/// format.
#[async_trait]
pub trait Muxer {
    /// Starts the muxer with the given tracks.
    async fn start(&mut self, tracks: Vec<Track>) -> anyhow::Result<()>;

    /// Writes a packet to the muxer.
    ///
    /// Note that this does not ensure something will be written to the output, as it may buffer
    /// packets internally in order to write its output correctly.
    async fn write(&mut self, packet: Packet) -> anyhow::Result<()>;

    /// Stops the muxer. This will flush any buffered packets and finalize the output if
    /// appropriate.
    async fn stop(&mut self) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct DemuxerMetadata {
    name: &'static str,
    create: fn(Io) -> Box<dyn Demuxer>,
    probe: fn(&[u8]) -> ProbeResult,
}

impl DemuxerMetadata {
    pub fn create(&self, io: Io) -> Box<dyn Demuxer> {
        (self.create)(io)
    }

    pub fn probe(&self, data: &[u8]) -> ProbeResult {
        (self.probe)(data)
    }
}

#[derive(Clone)]
pub struct MuxerMetadata {
    name: &'static str,
    create: fn(Io) -> Box<dyn Muxer>,
}

impl MuxerMetadata {
    pub fn create(&self, io: Io) -> Box<dyn Muxer> {
        (self.create)(io)
    }
}

/// A muxer that can handle splitting up the output into multiple segments.
pub struct SegmentMuxer {
    muxer: Box<dyn Muxer>,
}

#[derive(Copy, Clone, PartialEq)]
pub enum ProbeResult {
    Yup,
    Maybe(f32),
    Unsure,
}

impl PartialOrd for ProbeResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use ProbeResult::*;

        let ordering = match (self, other) {
            (Yup, Yup) => Ordering::Equal,
            (Yup, _) => Ordering::Greater,
            (_, Yup) => Ordering::Less,
            (Maybe(p1), Maybe(p2)) => p1.partial_cmp(p2)?,
            (Unsure, _) => Ordering::Less,
            (_, Unsure) => Ordering::Greater,
        };

        Some(ordering)
    }
}

pub struct Movie {
    pub tracks: Vec<Track>,
    pub attachments: Vec<Attachment>,
}

impl Movie {
    pub fn subtitles(&self) -> impl Iterator<Item = &Track> + '_ {
        self.tracks.iter().filter(|t| t.info.subtitle().is_some())
    }
}

pub struct Attachment {
    pub name: String,
    pub mime: String,
    pub data: Span,
}

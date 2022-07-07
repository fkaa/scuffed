use std::time::Instant;

use axum::{
    body::StreamBody, extract::Path, http::HeaderValue, response::Response, routing::get,
    Extension, Json, Router,
};
use bytes::Bytes;
use futures::stream::{self, Stream};
use log::*;
use mediabox::{
    format::{
        mp4::FragmentedMp4Muxer,
        rtmp::{RtmpListener, RtmpRequest},
    },
    Span,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc::Sender, RwLock};

use std::{collections::HashMap, io, sync::Arc};

use crate::Error;

pub fn api_route() -> Router {
    Router::new()
        .route("/", get(get_streams))
        // .route("/:stream", get(stream::get_streams))
        .route("/:stream/snapshot", get(get_snapshot))
}

async fn handle_rtmp_request(
    LiveStreams(live_streams): LiveStreams,
    request: RtmpRequest,
) -> anyhow::Result<()> {
    let app = request.app().to_string();
    let mut session = request.authenticate().await?;

    let streams = session.streams().await?;
    for stream in &streams {
        eprintln!("{}: {:?}", stream.id, stream.info);
    }

    let live_stream = LiveStream::new(streams);
    let splitter = live_stream.splitter.clone();
    let snapshot = live_stream.snapshot.clone();

    {
        let mut streams = live_streams.write().await;
        dbg!(&app);
        dbg!(&streams.keys());
        if streams.contains_key(&app) {
            return Err(anyhow::anyhow!("Tried to stream to an existing session"));
        }

        streams.insert(app.clone(), live_stream);
    }

    loop {
        match session.read_frame().await {
            Ok(pkt) => {
                if pkt.key && pkt.track.is_video() {
                    *snapshot.write().await = Some(pkt.clone());
                }
                splitter.write_packet(pkt).await
            }
            Err(e) => {
                live_streams.write().await.remove(&app);
                return Err(e);
            }
        }
    }
}

pub async fn listen(live_streams: LiveStreams) -> anyhow::Result<()> {
    let mut listener = RtmpListener::bind("127.0.0.1:1935").await?;

    loop {
        let request = listener.accept().await?;

        let live_streams = live_streams.clone();

        tokio::spawn(async {
            eprintln!("Got RTMP request from {}", request.addr());

            if let Err(e) = handle_rtmp_request(live_streams, request).await {
                error!("{}", e);
            }
        });
    }
}

#[derive(Clone, Default)]
pub struct LiveStreams(pub Arc<RwLock<HashMap<String, LiveStream>>>);

pub struct LiveStream {
    started: Instant,
    splitter: PacketSplitter,
    snapshot: Arc<RwLock<Option<mediabox::Packet>>>,
}

impl LiveStream {
    pub fn new(streams: Vec<mediabox::Track>) -> Self {
        LiveStream {
            started: Instant::now(),
            splitter: PacketSplitter::new(streams),
            snapshot: Arc::new(RwLock::new(None)),
        }
    }
}

#[derive(Clone)]
pub struct PacketSplitter {
    targets: Arc<RwLock<Vec<Sender<mediabox::Packet>>>>,
    streams: Vec<mediabox::Track>,
}

impl PacketSplitter {
    fn new(streams: Vec<mediabox::Track>) -> Self {
        PacketSplitter {
            targets: Arc::new(RwLock::new(Vec::new())),
            streams,
        }
    }

    pub async fn write_packet(&self, packet: mediabox::Packet) {
        let mut targets = self.targets.write().await;

        #[allow(clippy::needless_collect)]
        let targets_to_remove = targets
            .iter()
            .map(|send| send.try_send(packet.clone()))
            .enumerate()
            .filter_map(|(idx, res)| res.err().map(|e| (idx, e)))
            .collect::<Vec<_>>();

        for (idx, result) in targets_to_remove.into_iter().rev() {
            use tokio::sync::mpsc::error::TrySendError;

            match result {
                TrySendError::Full(_) => {
                    debug!("Closing splitter output due to channel overflow")
                }
                TrySendError::Closed(_) => {
                    debug!("Closing splitter output due to channel disconnection.")
                }
            }

            targets.remove(idx);
        }
    }

    async fn viewer_count(&self) -> usize {
        self.targets.read().await.len()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LiveStreamInfo {
    name: String,
    viewers: usize,
}

pub async fn get_streams(
    Extension(LiveStreams(map)): Extension<LiveStreams>,
) -> Result<Json<Vec<LiveStreamInfo>>, Error> {
    let streams = map.read().await;

    let mut live_streams = Vec::new();
    for (name, stream) in streams.iter() {
        live_streams.push(LiveStreamInfo {
            name: name.clone(),
            viewers: stream.splitter.viewer_count().await,
        });
    }

    Ok(Json(live_streams))
}

pub async fn get_snapshot(
    Path(stream): Path<String>,
    Extension(LiveStreams(map)): Extension<LiveStreams>,
) -> Result<Response<StreamBody<impl Stream<Item = io::Result<Bytes>>>>, Error> {
    let streams = map.read().await;

    let stream = streams.get(&stream).ok_or(Error::NotFound)?;
    let snapshot = stream.snapshot.read().await;
    let snapshot = snapshot.as_ref().ok_or(Error::NotFound)?;

    let mp4 = snapshot_mp4(&stream.splitter.streams, snapshot.clone())?;
    let chunks = mp4
        .to_byte_spans()
        .into_iter()
        .map(|s| Ok(s))
        .collect::<Vec<_>>();

    let stream = stream::iter(chunks);

    let mut response = Response::new(StreamBody::new(stream));
    response
        .headers_mut()
        .insert("Content-Type", HeaderValue::from_static("video/mp4"));

    Ok(response)
}

fn snapshot_mp4(streams: &[mediabox::Track], packet: mediabox::Packet) -> anyhow::Result<Span> {
    let mut fragger = FragmentedMp4Muxer::with_streams(streams);

    let span = [
        fragger.initialization_segment()?,
        fragger.write_media_segment(packet)?,
    ]
    .into_iter()
    .collect::<Span>();

    Ok(span)
}

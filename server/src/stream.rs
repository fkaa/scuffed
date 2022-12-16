use anyhow::Context;
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
        Movie,
    },
    Packet, Span,
};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    RwLock,
};
use tokio_rusqlite::Connection;
use tracing::{debug_span, Instrument};
use utoipa::ToSchema;

use std::{collections::HashMap, io, sync::Arc};

use crate::{
    notification::{self, WebPushKeys},
    Error,
};

pub fn api_route() -> Router {
    Router::new()
        .route("/", get(get_streams))
        .route("/:stream/preview", get(get_preview))
}

async fn handle_rtmp_request(
    db: Connection,
    svc: LiveStreamService,
    keys: Option<WebPushKeys>,
    request: RtmpRequest,
) -> anyhow::Result<()> {
    let key = request.key().to_string();

    let account = get_account_by_stream_key(&db, key).await?;

    let span = debug_span!("stream", name = %account.username);

    let fut = async move {
        debug!("Got RTMP request from {}", request.addr());

        let mut session = request.authenticate().await?;

        let tracks = session.streams().await?;
        let movie = Movie {
            tracks,
            attachments: Vec::new(),
        };

        let (mut splitter, gop) = svc.new_stream(account.username.clone(), movie).await?;

        if let Some(keys) = keys {
            let db = db.clone();
            let keys = keys.clone();
            let username = account.username.clone();
            tokio::spawn(async move {
                if let Err(e) = notification::on_stream_started(
                    db,
                    keys,
                    username,
                ).await {
                    error!("Failed to send stream started notifications: {e:?}");
                }
            });
        }

        let mut new_gop = Vec::new();
        loop {
            match session.read_frame().await {
                Ok(pkt) => {
                    if pkt.track.is_video() {
                        if pkt.key {
                            let mut gop = gop.write().await;
                            *gop = new_gop.clone();
                            new_gop.clear();
                        }
                        new_gop.push(pkt.clone());
                    }

                    splitter.write_packet(pkt).await
                }
                Err(e) => {
                    warn!("Encountered error while ingesting stream: {e:?}");
                    svc.stop_stream(account.username).await;

                    return Err(e);
                }
            }
        }
    };

    fut.instrument(span).await
}

struct Account {
    username: String,
    stream_key: String,
}

async fn get_account_by_stream_key(db: &Connection, key: String) -> anyhow::Result<Account> {
    db.call(move |conn| {
        conn.query_row(
            "SELECT username, stream_key FROM users WHERE stream_key = ?1",
            params![key],
            |r| {
                Ok(Account {
                    username: r.get(0).unwrap(),
                    stream_key: r.get(1).unwrap(),
                })
            },
        )
        .context("Failed to find account by stream key")
    })
    .await
}

pub async fn listen(
    db: Connection,
    keys: Option<WebPushKeys>,
    svc: LiveStreamService,
) -> anyhow::Result<()> {
    let mut listener = RtmpListener::bind("127.0.0.1:1935").await?;

    loop {
        let request = listener.accept().await?;

        let db = db.clone();
        let svc = svc.clone();
        let keys = keys.clone();

        let future = async move {
            if let Err(e) = handle_rtmp_request(db, svc, keys, request).await {
                error!("{}", e);
            }
        };

        tokio::spawn(future.instrument(debug_span!("ingest",)));
    }
}

#[derive(Clone, Default)]
pub struct LiveStreams(pub Arc<RwLock<HashMap<String, LiveStream>>>);

#[derive(Clone)]
pub struct LiveStreamService {
    streams: Arc<RwLock<HashMap<String, LiveStream>>>,
}

impl LiveStreamService {
    pub fn new() -> Self {
        LiveStreamService {
            streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_splitter_for_stream(&self, stream: &str) -> Option<PacketSplitter> {
        if let Some(stream) = self.streams.read().await.get(stream) {
            stream.splitter.read().await.clone()
        } else {
            None
        }
    }

    pub async fn new_stream(
        &self,
        username: String,
        movie: Movie,
    ) -> anyhow::Result<(PacketSplitter, Arc<RwLock<Vec<mediabox::Packet>>>)> {
        let mut streams = self.streams.write().await;

        let stream = streams
            .entry(username.clone())
            .or_insert(LiveStream::new(username.clone()));
        if stream.is_live {
            anyhow::bail!("Stream for {username:?} is already live");
        }

        let splitter = stream.start_stream(movie).await;
        let gop = stream.gop.clone();

        Ok((splitter, gop))
    }

    pub async fn stop_stream(&self, username: String) {
        let mut streams = self.streams.write().await;

        if let Some(stream) = streams.get_mut(&username) {
            stream.stop_stream().await;
        } else {
            warn!("Did not find stream for {username:?} when stopping");
        }
    }

    pub async fn get_stream(&self, username: &str) -> Option<LiveStream> {
        self.streams.read().await.get(username).cloned()
    }

    pub async fn get_all_streams(&self) -> Vec<LiveStream> {
        self.streams.read().await.values().cloned().collect()
    }
}

#[derive(Clone)]
pub struct LiveStream {
    name: String,
    started: OffsetDateTime,
    stopped_streaming: Option<OffsetDateTime>,
    is_live: bool,
    splitter: Arc<RwLock<Option<PacketSplitter>>>,
    gop: Arc<RwLock<Vec<mediabox::Packet>>>,
}

impl LiveStream {
    pub fn new(name: String) -> Self {
        LiveStream {
            name,
            started: OffsetDateTime::now_utc(),
            stopped_streaming: None,
            is_live: false,
            splitter: Arc::new(RwLock::new(None)),
            gop: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn start_stream(&mut self, movie: Movie) -> PacketSplitter {
        info!("Starting stream for {:?}", self.name);

        self.is_live = true;
        self.started = OffsetDateTime::now_utc();

        let splitter = PacketSplitter::new(movie);
        *self.splitter.write().await = Some(splitter.clone());

        splitter
    }

    pub async fn stop_stream(&mut self) {
        info!("Stopping stream for {:?}", self.name);

        self.is_live = false;
        self.stopped_streaming = Some(OffsetDateTime::now_utc());
    }
}

#[derive(Clone)]
pub struct PacketSplitter {
    targets: Arc<RwLock<Vec<Sender<mediabox::Packet>>>>,
    movie: Movie,
}

impl PacketSplitter {
    fn new(movie: Movie) -> Self {
        PacketSplitter {
            targets: Arc::new(RwLock::new(Vec::new())),
            movie,
        }
    }

    pub async fn attach(&mut self) -> (Movie, Receiver<Packet>) {
        let (send, recv) = mpsc::channel(512);

        self.targets.write().await.push(send);

        (self.movie.clone(), recv)
    }

    pub async fn write_packet(&mut self, packet: mediabox::Packet) {
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

/// Information about a livestream
///
/// ### Remarks
///
/// This can be used to represent a livestream that has taken place as well.
#[derive(ToSchema, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LiveStreamInfo {
    /// The name of the stream.
    name: String,

    /// How many viewers the stream has.
    viewers: usize,

    /// Whether the stream is currently live.
    is_live: bool,

    /// When the stream was started.
    started: i64,

    /// When the stream was stopped, if it's not live.
    stopped: Option<i64>,
}

/// Gets a list of all public livestreams.
#[utoipa::path(
    get,
    path = "/api/stream",
    responses(
        (status = 200, description = "List all livestreams successfully", body = [LiveStreamInfo]),
    )
)]
pub async fn get_streams(
    Extension(svc): Extension<LiveStreamService>,
) -> Result<Json<Vec<LiveStreamInfo>>, Error> {
    let streams = svc.get_all_streams().await;

    let mut all_streams = Vec::new();
    for stream in streams.iter() {
        let splitter = stream.splitter.read().await;
        let viewers = if let Some(splitter) = &*splitter {
            splitter.viewer_count().await
        } else {
            0
        };

        all_streams.push(LiveStreamInfo {
            name: stream.name.clone(),
            viewers,
            is_live: splitter.is_some(),
            started: stream.started.unix_timestamp(),
            stopped: stream.stopped_streaming.map(|t| t.unix_timestamp()),
        });
    }

    Ok(Json(all_streams))
}

/// Gets the most recently cached preview from a stream.
///
/// ### Remarks
///
/// The preview is returned as a short MP4 video.
///
/// This can return previews from offline streams as well.
#[utoipa::path(
    get,
    path = "/api/stream/{stream}/preview",
    responses(
        (status = 200, description = "Returned preview", content_type = "video/mp4"),
        (status = 404, description = "Did not find any previews for the given stream", content_type = "text/plain")
    ),
    params(
        ("stream" = String, Path, description = "The stream to get the preview from")
    )
)]
pub async fn get_preview(
    Path(stream): Path<String>,
    Extension(svc): Extension<LiveStreamService>,
) -> Result<Response<StreamBody<impl Stream<Item = io::Result<Bytes>>>>, Error> {
    let stream = svc.get_stream(&stream).await.ok_or(Error::NotFound)?;

    let gop = stream.gop.read().await;
    if gop.len() == 0 {
        return Err(Error::NotFound);
    }

    let movie = stream
        .splitter
        .read()
        .await
        .as_ref()
        .map(|s| s.movie.clone())
        .ok_or(Error::NotFound)?;

    let mp4 = snapshot_mp4(&movie, gop.clone())?;
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

fn snapshot_mp4(movie: &Movie, packets: Vec<mediabox::Packet>) -> anyhow::Result<Span> {
    let mut fragger = FragmentedMp4Muxer::with_streams(&movie.tracks);

    let mut spans = Vec::new();
    spans.push(fragger.initialization_segment()?);
    spans.push(fragger.write_many_media_segments(&packets)?);

    let span = spans.into_iter().collect::<Span>();

    Ok(span)
}

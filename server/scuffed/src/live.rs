


use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path,
    },
    response::Response,
    routing::get,
    Extension, Router,
};


use log::*;
use mediabox::{
    format::{
        mp4::FragmentedMp4Muxer,
        Movie,
    },
    Packet,
};



use tokio::sync::{
    mpsc::{Receiver},
};

use utoipa::ToSchema;



use crate::{Error, stream::LiveStreamService};

pub fn api_route() -> Router {
    Router::new()
        .route("/:stream", get(get_video))
}

/// Gets a MP4 video livestream through a websocket connection.
///
/// ### Messages
///
/// The WebSocket connection will only pushing video. Messages sent to the WebSocket connection
/// will be ignored.
///
/// The messages coming from the connection will always be binary and are intended to be used with
/// the [MSE](https://developer.mozilla.org/en-US/docs/Web/API/Media_Source_Extensions_API) API.
#[utoipa::path(
    get,
    path = "/live/{stream}",
    responses(
        (status = 101, description = "A livestream was found. Switching to the WebSocket protocol"),
        (status = 404, description = "There was no active livestream for the given stream", content_type = "text/plain")
    ),
    params(
        ("stream" = String, Path, description = "The stream to get the WebSocket connection from")
    )
)]
pub async fn get_video(
    Path(stream): Path<String>,
    ws: WebSocketUpgrade,
    Extension(svc): Extension<LiveStreamService>,
) -> Result<Response, Error> {
    debug!("Received video request for {stream}");

    let mut splitter = svc
        .get_splitter_for_stream(&stream)
        .await
        .ok_or(Error::NotFound)?;
    let (movie, receiver) = splitter.attach().await;

    Ok(ws.on_upgrade(move |socket| websocket_video(socket, movie, receiver)))
}

async fn websocket_video(socket: WebSocket, movie: Movie, receiver: Receiver<Packet>) {
    if let Err(e) = websocket_video_impl(socket, movie, receiver).await {
        warn!("Error while sending video over websocket: {e}");
    }
}

async fn wait_for_sync_frame(recv: &mut Receiver<Packet>) -> anyhow::Result<Packet> {
    loop {
        let pkt = recv
            .recv()
            .await
            .ok_or(anyhow::anyhow!("Packet channel closed"))?;

        if pkt.track.is_video() && pkt.key {
            return Ok(pkt);
        }
    }
}

async fn websocket_video_impl(
    mut socket: WebSocket,
    movie: Movie,
    mut receiver: Receiver<Packet>,
) -> anyhow::Result<()> {
    let mut fragger = FragmentedMp4Muxer::with_streams(&movie.tracks);

    let first_frame = wait_for_sync_frame(&mut receiver).await?;

    let codec_string = movie
        .codec_string()
        .ok_or(anyhow::anyhow!("Failed to create codec string"))?;
    let content_type = format!("video/mp4; codecs=\"{}\"", codec_string);

    debug!("Video content type: {content_type}");

    socket.send(Message::Text(content_type)).await?;

    socket
        .send(Message::Binary(
            fragger.initialization_segment()?.to_slice().into_owned(),
        ))
        .await?;

    socket
        .send(Message::Binary(
            fragger
                .write_media_segment(first_frame)?
                .to_slice()
                .into_owned(),
        ))
        .await?;

    loop {
        let pkt = receiver
            .recv()
            .await
            .ok_or(anyhow::anyhow!("Packet channel closed"))?;

        socket
            .send(Message::Binary(
                fragger.write_media_segment(pkt)?.to_slice().into_owned(),
            ))
            .await?;
    }
}

use std::time::Instant;

use anyhow::Context;
use axum::{
    body::StreamBody,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path,
    },
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use bytes::Bytes;
use futures::stream::{self, Stream};
use idlib::{AuthorizationRejection, AuthorizeCookie, NoGroups};
use log::*;
use mediabox::{
    format::{
        mp4::FragmentedMp4Muxer,
        rtmp::{RtmpListener, RtmpRequest},
        Movie,
    },
    Packet, Span,
};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    RwLock,
};
use tokio_rusqlite::Connection;
use utoipa::ToSchema;
use rand::{rngs::StdRng, SeedableRng, RngCore};

use std::{collections::HashMap, io, sync::Arc};

use crate::Error;

pub fn api_route() -> Router {
    Router::new()
        .route("/", get(get_account))
        .route("/login", get(get_login))
        .route("/key", post(post_generate_stream_key))
}

/// Information about an account.
#[derive(ToSchema, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    /// The name of the account.
    name: String,

    /// The account's stream key.
    stream_key: String,
}

/// Gets account info.
#[utoipa::path(
    get,
    path = "/account",
    responses(
        (status = 200, description = "Lists account info successfully", body = [AccountInfo]),
        (status = 404, description = "Not signed in")
    )
)]
pub async fn get_account(
    cookie: Result<AuthorizeCookie<NoGroups>, AuthorizationRejection>,
    Extension(db): Extension<Connection>,
) -> impl IntoResponse {
    let AuthorizeCookie(payload, maybe_token, ..) = if let Ok(cookie) = cookie {
        cookie
    } else {
        return Error::NotFound.into_response();
    };

    maybe_token.wrap_future(async move {
        let account = get_account_by_username(db, payload.name)
            .await;

        account.map(|a| Json(AccountInfo {
            name: a.username,
            stream_key: a.stream_key,
        })).map_err(|_| Error::NotFound)
    }).await
}

/// Logs in to the site by redirecting to hiveID.
#[utoipa::path(
    get,
    path = "/account/login",
    responses(
        (status = 302, description = "Redirects to hiveID successfully"),
    )
)]
pub async fn get_login(
    AuthorizeCookie(_payload, maybe_token, ..): AuthorizeCookie<NoGroups>,
) -> impl IntoResponse {
    maybe_token.wrap(|| {})
}

struct Account {
    username: String,
    stream_key: String,
}

async fn get_account_by_username(db: Connection, username: String) -> anyhow::Result<Account> {
    db.call(move |conn| {
        conn.query_row(
            "SELECT username, stream_key FROM users WHERE username = ?1",
            params![username],
            |r| {
                Ok(Account {
                    username: r.get(0).unwrap(),
                    stream_key: r.get(1).unwrap(),
                })
            },
        )
        .context("Failed to query users")
    })
    .await
}


/// Generates a new stream key.
#[utoipa::path(
    post,
    path = "/account/key",
    responses(
        (status = 200, description = "Succesfully changed stream key.")
    )
)]
pub async fn post_generate_stream_key(
    AuthorizeCookie(payload, maybe_token, ..): AuthorizeCookie<NoGroups>,
    Extension(db): Extension<Connection>,
) -> impl IntoResponse {
    maybe_token.wrap_future(async move {
        generate_stream_key(db, payload.name).await?;

        Ok::<_, Error>(StatusCode::OK)
    }).await
}

async fn generate_stream_key(db: Connection, username: String) -> anyhow::Result<()> {
    let new_stream_key = get_new_stream_key();

    db.call(move |conn| {
        conn.execute(
            "UPDATE users \
            SET stream_key = ?1
            WHERE username = ?2",
            params![new_stream_key, username]
        )
        .context("Failed to update stream key")?;

        Ok(())
    })
    .await
}

fn get_new_stream_key() -> String {
    let mut secret_bytes = [0u8; 32];
    StdRng::from_entropy().fill_bytes(&mut secret_bytes[..]);

    base64::encode(secret_bytes)
}

use std::{env, io::Cursor, path::PathBuf, sync::Arc};

use crate::{Connection, Error};

use anyhow::Context;
use axum::{
    response::IntoResponse,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use hyper::StatusCode;
use idlib::{AuthorizeCookie, NoGroups};
use rusqlite::{params, OptionalExtension};
use serde::Serialize;
use time::OffsetDateTime;
use tokio::fs;
use tracing::debug;
use web_push::{
    ContentEncoding, SubscriptionInfo, SubscriptionKeys, VapidSignatureBuilder, WebPushClient,
    WebPushMessageBuilder,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StreamNotification {
    name: String,
    started: i64,
}

pub async fn on_stream_started(
    db: Connection,
    keys: WebPushKeys,
    name: String,
) -> anyhow::Result<()> {
    debug!("Sending stream started notification for {name:?}");

    let subscriptions = get_all_notification_subscriptions_for_stream(db, name.clone()).await;

    let sig_builder =
        VapidSignatureBuilder::from_base64_no_sub(&keys.private_key, web_push::STANDARD)?;
    let client = WebPushClient::new()?;

    let notification = StreamNotification {
        name: name.clone(),
        started: OffsetDateTime::now_utc().unix_timestamp(),
    };
    let notification = serde_json::to_string(&notification).unwrap();

    for account_subscription in subscriptions {
        debug!("Sending notification to {}", account_subscription.username);

        let subscription = account_subscription.subscription;

        let mut sig_builder = sig_builder.clone().add_sub_info(&subscription);
        sig_builder.add_claim("sub", "tmtu+vapid@tmtu.ee");
        let sig = sig_builder.build()?;

        let mut builder = WebPushMessageBuilder::new(&subscription)?;

        let content = notification.as_bytes();
        builder.set_payload(ContentEncoding::Aes128Gcm, content);
        builder.set_vapid_signature(sig);

        client.send(builder.build()?).await?;
    }

    Ok(())
}

#[derive(Clone)]
pub struct WebPushKeys {
    pub public_key: String,
    pub private_key: String,
}

impl WebPushKeys {
    pub fn from_env() -> Option<Self> {
        let public_web_push_key = env::var("WEB_PUSH_PUBLIC_KEY_B64");
        let private_web_push_key = env::var("WEB_PUSH_PRIVATE_KEY_B64");

        if let (Ok(public_key), Ok(private_key)) = (public_web_push_key, private_web_push_key) {
            Some(WebPushKeys {
                public_key,
                private_key,
            })
        } else {
            None
        }
    }
}

pub fn api_route() -> Router {
    Router::new()
        .route("/key", get(get_public_key))
        .route("/", get(get_notification_settings))
        .route("/", post(post_notification_subscription))
        .route("/", delete(delete_notification_subscription))
}

/// Gets the notification settings.
#[utoipa::path(
    get,
    path = "/api/notification/",
    responses(
        (status = 200, description = "Lists the notification settings", content_type = "text/plain"),
        (status = 404, description = "The user has no notification settings", content_type = "text/plain"),
    )
)]
pub async fn get_notification_settings(
    AuthorizeCookie(payload, maybe_token, ..): AuthorizeCookie<NoGroups>,
    Extension(db): Extension<Connection>,
) -> impl IntoResponse {
    maybe_token
        .wrap_future(async move {
            let name = payload.name;
            match db
                .call(move |conn| has_notification_subscription(conn, &name))
                .await
            {
                true => (StatusCode::OK, ""),
                false => (StatusCode::NOT_FOUND, ""),
            }
        })
        .await
}

/// Gets the public key used for web push notifications.
#[utoipa::path(
    get,
    path = "/api/notification/key",
    responses(
        (status = 200, description = "Lists the public key successfully", content_type = "text/plain"),
        (status = 404, description = "Web push keys have not been setup correctly", content_type = "text/plain"),
    )
)]
pub async fn get_public_key(
    Extension(keys): Extension<Arc<Option<WebPushKeys>>>,
) -> impl IntoResponse {
    match &*keys {
        Some(keys) => (StatusCode::OK, keys.public_key.clone()),
        None => (StatusCode::NOT_FOUND, "".into()),
    }
}

/// Subscribes to notifications.
#[utoipa::path(
    post,
    path = "/api/notification",
    responses(
        (status = 201, description = "Notification subscription added successfully"),
    )
)]
pub async fn post_notification_subscription(
    AuthorizeCookie(payload, maybe_token, ..): AuthorizeCookie<NoGroups>,
    Extension(db): Extension<Connection>,
    Json(body): Json<SubscriptionInfo>,
) -> impl IntoResponse {
    maybe_token
        .wrap_future(add_notification_subscription(db, payload.name, body))
        .await
}

/// Removes a notification subscription.
#[utoipa::path(
    delete,
    path = "/api/notification",
    responses(
        (status = 200, description = "Notification subscription deleted successfully"),
    )
)]
pub async fn delete_notification_subscription(
    AuthorizeCookie(payload, maybe_token, ..): AuthorizeCookie<NoGroups>,
    Extension(db): Extension<Connection>,
) -> impl IntoResponse {
    maybe_token
        .wrap_future(remove_notification_subscription(db, payload.name))
        .await
}

async fn add_notification_subscription(
    db: Connection,
    name: String,
    subscription: SubscriptionInfo,
) -> Result<StatusCode, Error> {
    db.call(move |conn| {
        conn.execute(
            "UPDATE notification_subscriptions \
            SET endpoint=?2, auth=?3, p256dh=?4 \
            WHERE username=?1",
            params![
                &name,
                &subscription.endpoint,
                &subscription.keys.auth,
                &subscription.keys.p256dh
            ],
        )
        .context("Failed to update notification subscription")?;

        conn.execute(
            "INSERT OR IGNORE INTO notification_subscriptions \
            (username, endpoint, auth, p256dh) \
            VALUES (?1, ?2, ?3, ?4)",
            params![
                &name,
                &subscription.endpoint,
                &subscription.keys.auth,
                &subscription.keys.p256dh
            ],
        )
        .context("Failed to insert notification subscription")?;

        debug!("Inserted notification subscription for {name:?}");

        Ok::<_, Error>(())
    })
    .await?;

    Ok(StatusCode::CREATED)
}

async fn remove_notification_subscription(db: Connection, name: String) -> Result<(), Error> {
    db.call(move |conn| {
        conn.execute(
            "DELETE FROM notification_subscriptions \
            WHERE username = ?1",
            params![name,],
        )
        .context("Failed to delete notification subscription")?;

        Ok::<_, Error>(())
    })
    .await?;

    Ok(())
}

pub fn has_notification_subscription(conn: &rusqlite::Connection, name: &str) -> bool {
    conn.query_row(
        "SELECT * FROM notification_subscriptions \
        WHERE username=?1",
        params![name],
        |_r| Ok(()),
    )
    .optional()
    .unwrap()
    .is_some()
}

struct AccountSubscriptionInfo {
    username: String,
    subscription: SubscriptionInfo,
}

async fn get_all_notification_subscriptions_for_stream(
    db: Connection,
    name: String,
) -> Vec<AccountSubscriptionInfo> {
    db.call(|conn| {
        let mut stmt = conn
            .prepare("SELECT username, endpoint, auth, p256dh FROM notification_subscriptions")
            .unwrap();

        let rows = stmt
            .query_map(params![], |row| {
                Ok(AccountSubscriptionInfo {
                    username: row.get(0).unwrap(),
                    subscription: SubscriptionInfo {
                        endpoint: row.get(1).unwrap(),
                        keys: SubscriptionKeys {
                            auth: row.get(2).unwrap(),
                            p256dh: row.get(3).unwrap(),
                        },
                    },
                })
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        rows
    })
    .await
}

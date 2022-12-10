#![allow(dead_code)]
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use axum::body::{boxed, BoxBody, Empty};
use axum::response::Response;
use axum::routing::get;
use axum::{http::StatusCode, response::IntoResponse, Extension, Router};
use futures::FutureExt;
use idlib::{AuthCallback, IdpClient, SecretKey, Variables};

use log::*;
use rusqlite::{params, OptionalExtension};
use rusqlite_migration::{Migrations, M};

mod account;
mod error;
mod live;
mod stream;

pub use error::Error;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub type Connection = tokio_rusqlite::Connection;

const MIGRATIONS: [M; 1] = [M::up(include_str!("../migrations/0001_initial.sql"))];

async fn create_account_if_missing(db: Connection, name: String) -> anyhow::Result<()> {
    db.call(move |conn| {
        if let None = conn
            .query_row(
                "SELECT * FROM users WHERE username = ?1",
                params![&name],
                |_r| Ok(()),
            )
            .optional()
            .context("Failed to query users")?
        {
            let stream_key = "test123";

            conn.execute(
                "INSERT INTO users (username, stream_key) VALUES (?1, ?2)",
                params![&name, &stream_key],
            )
            .context("")?;

            info!("Created account for user {name}");
        } else {
            debug!("Account already existed for {name}");
        }

        Ok::<(), anyhow::Error>(())
    })
    .await?;

    Ok(())
}

pub async fn api_route(db: tokio_rusqlite::Connection, svc: stream::LiveStreamService) -> Router {
    let secret_key = SecretKey::from_env();
    let variables = Variables::from_env();

    let client = IdpClient::default();

    #[derive(OpenApi)]
    #[openapi(
        paths(
            stream::get_streams,
            stream::get_preview,
            live::get_video,
            account::get_account,
            account::get_login,
            account::post_generate_stream_key
        ),
        components(schemas(stream::LiveStreamInfo, account::AccountInfo))
    )]
    struct ApiDoc;

    let a = db.clone();
    let mut router = Router::new()
        .merge(SwaggerUi::new("/swagger").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .route("/api/health", get(health))
        .nest("/api/stream/", stream::api_route())
        .nest("/api/live/", live::api_route())
        .nest("/api/account/", account::api_route())
        .nest(
            "/auth",
            idlib::api_route(
                client,
                Some(AuthCallback(Arc::new(Box::new(move |name| {
                    let db = a.clone();

                    async move {
                        create_account_if_missing(db, name).await?;

                        Ok(())
                    }
                    .boxed()
                })))),
            ),
        )
        .layer(Extension(IdpClient::default()));

    router = router
        .layer(Extension(db))
        .layer(Extension(svc))
        .layer(Extension(secret_key))
        .layer(Extension(Arc::new(variables)));

    router
}

async fn health() -> Result<Response<BoxBody>, Error> {
    let response = Response::builder()
        .status(StatusCode::OK)
        .body(boxed(Empty::new()))
        .unwrap();

    Ok(response)
}

async fn get_stream_key(db: Connection, name: String) -> anyhow::Result<String> {
    let key = db
        .call(move |conn| {
            conn.query_row(
                "SELECT \
                stream_key \
            FROM users \
            WHERE username = ?1",
                params![name],
                |row| row.get::<_, String>(0),
            )
        })
        .await
        .context("Failed to get inviter")?;

    Ok(key)
}

async fn run() {
    let db_path: PathBuf = env::var("DB_PATH").expect("DB_PATH not set").into();

    let bind_addr: SocketAddr = env::var("BIND_ADDRESS")
        .expect("BIND_ADDRESS not set")
        .parse()
        .expect("BIND_ADDRESS could not be parsed");

    info!("Listening on {bind_addr:?}");

    let conn = tokio_rusqlite::Connection::open(&db_path)
        .await
        .expect("Failed to open database");

    // apply latest migrations
    conn.call(|mut c| {
        let migrations = Migrations::new(MIGRATIONS.to_vec());
        migrations
            .to_latest(&mut c)
            .expect("Failed to apply migrations");
    })
    .await;

    let svc = stream::LiveStreamService::new();

    {
        let db = conn.clone();
        let svc = svc.clone();
        tokio::spawn(async {
            if let Err(e) = stream::listen(db, svc).await {
                error!("{}", e);
            }
        });
    }

    let router = api_route(conn, svc).await;

    axum::Server::try_bind(&bind_addr)
        .expect("Failed to bind server")
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { run().await })
}

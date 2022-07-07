#![allow(dead_code)]
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;

use axum::{http::StatusCode, response::IntoResponse, routing::get_service, Extension, Router};
use log::*;
use rusqlite_migration::{Migrations, M};
use tower_http::services::ServeDir;

mod db;
mod error;
mod stream;

pub use error::Error;

pub type Connection = tokio_rusqlite::Connection;

const MIGRATIONS: [M; 1] = [M::up(include_str!("../migrations/0001_initial.sql"))];

pub fn api_route(
    db: tokio_rusqlite::Connection,
    streams: stream::LiveStreams,
    serve_dir: PathBuf,
) -> Router {
    Router::new()
        .nest("/api/streams/", stream::api_route())
        // .route("/api/streams/:stream", get(stream::get_streams))
        // .route("/api/streams/:stream/snapshot", get())
        .fallback(get_service(ServeDir::new(serve_dir)).handle_error(handle_error))
        .layer(Extension(db))
        .layer(Extension(streams))
}

async fn run() {
    let db_path: PathBuf = env::var("DB_PATH").expect("DB_PATH not set").into();

    let serve_dir: PathBuf = env::var("SERVE_DIR").expect("SERVE_DIR not set").into();

    let bind_addr: SocketAddr = env::var("BIND_ADDRESS")
        .expect("BIND_ADDRESS not set")
        .parse()
        .expect("BIND_ADDRESS could not be parsed");

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

    let streams = stream::LiveStreams::default();

    let live_streams = streams.clone();
    tokio::spawn(async {
        if let Err(e) = stream::listen(live_streams).await {
            error!("{}", e);
        }
    });

    let router = api_route(conn, streams, serve_dir);

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

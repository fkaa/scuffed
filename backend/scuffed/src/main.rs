use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Instant;

use axum::{Extension, Router};
use rusqlite_migration::{Migrations, M};

mod db;
mod stream;

pub type Connection = tokio_rusqlite::Connection;

const MIGRATIONS: [M; 1] = [M::up(include_str!("../migrations/0001_initial.sql"))];

pub fn api_route(db: tokio_rusqlite::Connection) -> Router {
    Router::new()
        .nest("/api/streams/", stream::api_route())
        // .route("/api/streams/:stream", get(stream::get_streams))
        // .route("/api/streams/:stream/snapshot", get())
        .layer(Extension(db))
}

async fn run() {
    let db_path: PathBuf = env::var("DB_PATH").expect("DB_PATH not set").into();

    let bind_addr: SocketAddr = env::var("BIND_ADDRESS")
        .expect("BIND_ADDRESS not set")
        .parse()
        .expect("BIND_ADDRESS could not be parsed");

    let conn = tokio_rusqlite::Connection::open(&db_path).await.expect("Failed to open database");

    // apply latest migrations
    conn.call(|mut c| {
        let migrations = Migrations::new(MIGRATIONS.to_vec());
        migrations
            .to_latest(&mut c)
            .expect("Failed to apply migrations");
    })
    .await;

    axum::Server::try_bind(&bind_addr).expect("Failed to bind server")
        .serve(api_route(conn).into_make_service())
        .await
        .unwrap();
    // .await
    // .unwrap();
}

fn main() {
    dotenv::dotenv().ok();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { run().await })
}

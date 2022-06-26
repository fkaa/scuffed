use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Instant;

use rusqlite_migration::{Migrations, M};

mod db;
mod stream;

pub struct Stream {
    started: Instant,
    viewers: u32,
}

pub struct PacketSplitter {

}

pub struct AppState {
    db: db::Connection,
}

const MIGRATIONS: [M; 1] = [M::up(include_str!("../migrations/0001_initial.sql"))];

async fn run() {
    let db_path: PathBuf = env::var("DB_PATH").expect("DB_PATH not set").into();

    let bind_addr: SocketAddr = env::var("BIND_ADDRESS")
        .expect("BIND_ADDRESS not set")
        .parse()
        .expect("BIND_ADDRESS could not be parsed");

    let conn = db::Connection::open(&db_path).expect("Failed to open database");

    // apply latest migrations
    conn.write(|mut c| {
        let migrations = Migrations::new(MIGRATIONS.to_vec());
        migrations
            .to_latest(&mut c)
            .expect("Failed to apply migrations");
    })
    .await;

    axum::Server::try_bind(&bind_addr).expect("Failed to bind server");
    // .serve(api_route(pool).into_make_service())
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

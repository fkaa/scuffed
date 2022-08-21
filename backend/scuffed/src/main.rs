#![allow(dead_code)]
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::{borrow::Cow, env};

use anyhow::Context;
use askama::Template;
use axum::body::{boxed, BoxBody, Empty, self, Full};
use axum::response::Response;
use axum::routing::get;
use axum::{http::StatusCode, response::IntoResponse, routing::get_service, Extension, Router};
use futures::FutureExt;
use idlib::{
    AuthCallback, Authorizations, Authorize, IdpClient, PermissionResponse, SecretKey, Variables, AuthorizationRejection,
};
use jwt::token::signed::SignWithKey;
use log::*;
use rusqlite::{params, OptionalExtension};
use rusqlite_migration::{Migrations, M};
use serde::Deserialize;
use tower_http::services::ServeDir;

mod db;
mod error;
mod stream;

pub use error::Error;

pub type Connection = tokio_rusqlite::Connection;

const MIGRATIONS: [M; 1] = [M::up(include_str!("../migrations/0001_initial.sql"))];

pub fn into_response<T: Template>(t: &T, ext: &str) -> Response<BoxBody> {
    match t.render() {
        Ok(body) => Response::builder()
            .status(StatusCode::OK)
            .header(
                "content-type",
                "text/html",
            )
            .body(body::boxed(Full::from(body)))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(body::boxed(Empty::new()))
            .unwrap(),
    }
}

async fn create_account_if_missing(db: Connection, name: String) -> anyhow::Result<()> {
    db.call(move |conn| {
        if let None = conn
            .query_row(
                "SELECT * FROM users WHERE username = ?1",
                params![&name],
                |r| Ok(()),
            )
            .optional()
            .context("")?
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

pub async fn api_route(
    db: tokio_rusqlite::Connection,
    streams: stream::LiveStreams,
    serve_dir: PathBuf,
    old_serve_dir: Option<PathBuf>,
) -> Router {
    let secret_key = SecretKey::from_env();
    let variables = Variables::from_env();
    let authorizations = Authorizations::in_memory().await;

    let client = IdpClient::default();

    fill_auth(&authorizations, &client, &variables, &secret_key)
        .await
        .expect("Failed to initialize permissions");

    let a = db.clone();
    let mut router = Router::new()
        .route("/api/health", get(health))
        .route("/account", get(account))
        .nest("/api/streams/", stream::api_route())
        .nest(
            "/api/auth",
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
        .fallback(get_service(ServeDir::new(serve_dir)).handle_error(handle_error));

    if let Some(old_dir) = old_serve_dir {
        router = router.nest(
            "/old/",
            Router::new()
                .route("/", get(home))
                .route("/account", get(account))
        )
        .fallback(get_service(ServeDir::new(old_dir)).handle_error(handle_error));
    }

    router = router
        .layer(Extension(db))
        .layer(Extension(streams))
        .layer(Extension(secret_key))
        .layer(Extension(authorizations))
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

#[derive(Template)]
#[template(path = "home.html")]
struct HomePageTemplate {
    username: Option<String>,
}

async fn home(
    auth: Result<Authorize, AuthorizationRejection>,
    Extension(db): Extension<Connection>,
) -> Result<Response<BoxBody>, Error> {
    let template = HomePageTemplate {
        username: auth.ok().map(|a| a.0)
    };

    Ok(into_response(&template, "html"))
}

#[derive(Template)]
#[template(path = "account.html")]
struct AccountPageTemplate<'a> {
    username: &'a str,
    stream_key: &'a str,
}

async fn account(
    Authorize(name): Authorize,
    Extension(db): Extension<Connection>,
) -> Result<Response<BoxBody>, Error> {
    let stream_key = get_stream_key(db, name.clone()).await?;

    let template = AccountPageTemplate {
        username: &name,
        stream_key: &stream_key,
    };

    Ok(into_response(&template, "html"))
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

async fn fill_auth(
    auth: &Authorizations,
    IdpClient(client): &IdpClient,
    variables: &Variables,
    key: &SecretKey,
) -> anyhow::Result<()> {
    let url = variables
        .idp_fetch_permission_address
        .as_ref()
        .expect("IDP_FETCH_PERMISSION_ADDR not set");
    let token = "yup"
        .sign_with_key(&*key.0)
        .context("Failed to sign token")?;

    let body = client
        .get(url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await?
        .text()
        .await?;

    let permissions: PermissionResponse = serde_json::de::from_str(&body)?;
    auth.replace_policy(permissions.policy, permissions.group_policy)
        .await
        .context("Failed to replace policy")?;

    Ok(())
}

async fn run() {
    let db_path: PathBuf = env::var("DB_PATH").expect("DB_PATH not set").into();

    let old_serve_dir: Option<PathBuf> = env::var("OLD_SERVE_DIR").map(|s| s.into()).ok();
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

    let router = api_route(conn, streams, serve_dir, old_serve_dir).await;

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

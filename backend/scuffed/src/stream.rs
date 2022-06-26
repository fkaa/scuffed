use std::time::Instant;

pub struct Stream {
    started: Instant,
    viewers: u32,
}

pub struct PacketSplitter {

}

pub fn api_route() -> Router {
    Router::new()
        .route("/streams/", get(get_streams))
        // .route("/api/streams/:stream", get(stream::get_streams))
        // .route("/api/streams/:stream/snapshot", get())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StreamInfo {
    author: String,
    viewers: u32,
}

pub async fn get_streams(
    // Authorize(_): Authorize,
    Extension(db): Extension<Connection>,
) -> Result<Json<Vec<StreamInfo>>, Error> {
    todo!()
}

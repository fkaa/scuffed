use mediabox::format::mkv::*;
use mediabox::format::*;
use mediabox::io::*;

use tokio::fs::File;

#[tokio::main]
async fn main() {
    env_logger::init();

    let path = "test.mkv";
    let file = File::open(path).await.unwrap();
    let io = Io::from_reader(Box::new(file));
    let mut demuxer = MatroskaDemuxer::new(io);

    let streams = demuxer.start().await.unwrap();

    loop {
        let pkt = demuxer.read().await.unwrap();
        dbg!(pkt);
    }
}

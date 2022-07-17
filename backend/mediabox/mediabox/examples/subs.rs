use mediabox::codec::webvtt::*;
use mediabox::codec::*;
use mediabox::format::mkv::*;
use mediabox::format::*;
use mediabox::io::*;
use mediabox::*;

use log::*;
use tokio::fs::File;

use std::{env, str};

#[tokio::main]
async fn main() {
    env_logger::init();

    let path = env::args().nth(1).expect("Provide a file");
    debug!("Opening {path}");

    let file = File::open(path).await.unwrap();
    let io = Io::from_reader(Box::new(file));
    let mut demuxer = MatroskaDemuxer::new(io);

    let movie = demuxer.start().await.unwrap();

    for track in &movie.tracks {
        eprintln!("#{}: {:?}", track.id, track.info);
    }

    let transcode_mapping = movie
        .subtitles()
        .filter_map(|Track { id, info, .. }| {
            (info.name != "webvtt").then(|| {
                (
                    *id,
                    Transcode::Subtitles {
                        decoder: mediabox::find_decoder(info.name).unwrap(),
                        encoder: mediabox::find_encoder_with_params("webvtt", info).unwrap(),
                    },
                )
            })
        })
        .collect();

    let mut transcoder = PacketTranscoder::new(transcode_mapping);

    loop {
        let pkt = demuxer.read().await.unwrap();

        transcoder
            .process(pkt, |pkt| {
                if pkt.track.info.name == "webvtt" {
                    eprintln!(
                        "{}",
                        str::from_utf8(&pkt.buffer.to_slice()).expect("Failed to read string")
                    );
                }
            })
            .await
            .unwrap();
    }
}

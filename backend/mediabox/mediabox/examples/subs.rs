use mediabox::codec::ass::*;
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

    let subtitle_track = movie
        .tracks
        .iter()
        .find(|t| t.info.subtitle().is_some())
        .unwrap();

    let subtitle_info = subtitle_track.info.subtitle().unwrap();

    let mut decoder = AssDecoder::new();
    decoder
        .start(subtitle_info)
        .expect("Failed to init decoder");

    let mut encoder = WebVttEncoder::new();
    encoder
        .start(SubtitleDescription::default())
        .expect("Failed to start encoder");

    loop {
        let pkt = demuxer.read().await.unwrap();

        if pkt.track.id == subtitle_track.id {
            /*eprintln!("{:?}", pkt.time);
            eprintln!(
                "{}",
                String::from_utf8(pkt.buffer.to_slice().into_owned()).unwrap()
            );*/

            decoder.feed(pkt).expect("Failed to decode");

            while let Some(cue) = decoder.receive() {
                encoder.feed(cue).expect("Failed to encode");

                while let Some(pkt) = encoder.receive() {
                    eprintln!(
                        "{}",
                        str::from_utf8(&pkt.buffer.to_slice()).expect("Failed to read string")
                    );
                }

                /*for part in &cue.text {
                    if let TextPart::Text(txt) = &part {
                        print!("{txt}");
                    }
                }

                println!("\n");*/
            }
        }
    }
}

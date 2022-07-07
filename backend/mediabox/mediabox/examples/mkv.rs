use mediabox::format::mkv::*;
use mediabox::format::*;
use mediabox::io::*;
use mediabox::*;

use tokio::fs::File;

#[tokio::main]
async fn main() {
    env_logger::init();

    let file = File::open("test.mkv").await.unwrap();
    let io = Io::from_reader(Box::new(file));
    let mut demuxer = MatroskaDemuxer::new(io);

    let movie = demuxer.start().await.unwrap();

    for track in &movie.tracks {
        eprintln!("#{}: {:?}", track.id, track.info);
    }

    let subtitle_id = movie
        .tracks
        .iter()
        .find(|t| t.info.subtitle().is_some())
        .map(|t| t.id)
        .unwrap();

    // println!("pts,dts,keyframe,stream,length");
    loop {
        let pkt = demuxer.read().await.unwrap();

        println!("{:?}", pkt.time);
        if pkt.track.id == subtitle_id {
            println!(
                "{}",
                String::from_utf8(pkt.buffer.to_slice().into_owned()).unwrap()
            );
        }

        /*print!("{},", pkt.time.pts);
        if let Some(dts) = pkt.time.dts {
            print!("{dts},");
        } else {
            print!(",");
        }
        print!("{},", pkt.key);
        print!("{},", pkt.stream.id);
        println!("{}", pkt.buffer.len());*/
    }
}

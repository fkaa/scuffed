use mediabox::format::mkv::*;
use mediabox::format::*;
use mediabox::io::*;
use mediabox::*;

use tokio::fs::File;

#[tokio::main]
async fn main() {
    env_logger::init();

    let path = "test.mkv";
    let file = File::open(path).await.unwrap();
    let io = Io::from_reader(Box::new(file));
    let mut demuxer = MatroskaDemuxer::new(io);

    let streams = demuxer.start().await.unwrap();

    for stream in &streams {
        eprintln!("#{}: {:?}", stream.id, stream.info);
    }

    let subtitle_id = streams
        .iter()
        .find(|s| s.info.subtitle().is_some())
        .map(|s| s.id)
        .unwrap();

    // println!("pts,dts,keyframe,stream,length");
    loop {
        let pkt = demuxer.read().await.unwrap();

        if pkt.stream.id == subtitle_id {
            println!("{}", String::from_utf8(pkt.buffer.to_slice().into_owned()).unwrap());
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

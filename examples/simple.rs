extern crate mpv ;

// use mpv::mpv;

use std::env;
use std::path::Path;

fn simple_example(video_path: &Path) {
    let mpv = mpv::mpv_handler::Mpv_handler::init().expect("Error while initializing MPV");
    if video_path.is_file() {
        let video_path = video_path.to_str().expect("Expected a string for Path, got None");
        mpv.command(&["loadfile", video_path as &str])
           .expect("Error loading file");
        loop {
            while let Some(_) = mpv.wait_event() {
                // do something with the events
                // but it's kind of useless
                // it's still necessary to empty the event pool
            }

        }
    } else {
        println!("A file is required; {} is not a valid file",
                 video_path.to_str().unwrap());
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    println!("MPV_API_VERSION : {}", mpv::client_api_version());
    if args.len() < 2 {
        println!("Usage: ./simple [any mp4, avi, mkv, ... file]");
    } else {
        let path: &Path = Path::new(&args[1]);
        simple_example(path);
    }
}

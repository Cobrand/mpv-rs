extern crate mpv ;

// use mpv::mpv;
use std::env;
use std::path::Path;

fn simple_example(video_path: &Path) {
    let mpv = mpv::MpvHandler::init().expect("Error while initializing MPV");
    if video_path.is_file() {
        let video_path = video_path.to_str().expect("Expected a string for Path, got None");
        mpv.command(&["loadfile", video_path as &str])
           .expect("Error loading file");

        // set option "sid" to "no" (no subtitles)
        mpv.set_option("sid","no").unwrap();

        // loop twice, send parameter as a string
        mpv.set_property("loop","2").unwrap();

        // set speed to 100%, send parameter as a f64
        mpv.set_property("speed",1.0).unwrap();

        // get how many loops are playing as an i64
        let n_loop : i64 = mpv.get_property("loop").unwrap() ;
        println!("NUMBER OF LOOPS IS {}",n_loop);

        'main: loop {
            while let Some(event) = mpv.wait_event() {
                // even if you don't do anything with the events, it is still necessary to empty
                // the event loop
                println!("RECEIVED EVENT : {:?}", event.event_id.to_str());
                match event.event_id {
                    mpv::MpvEventId::MPV_EVENT_SHUTDOWN => {
                        break 'main;
                    }
                    _ => {}
                };
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

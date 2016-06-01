extern crate mpv ;
extern crate sdl2;
extern crate sdl2_sys;
#[macro_use]
extern crate log;

use std::env;
use std::path::Path;
use std::os::raw::{c_void,c_char};
use std::ffi::CStr;

use sdl2::event::Event as SdlEvent;
use sdl2::keyboard::Keycode;

unsafe extern "C" fn get_proc_address(arg: *mut c_void,
                                      name: *const c_char) -> *mut c_void {
    let arg: &sdl2::VideoSubsystem = &*(arg as *mut sdl2::VideoSubsystem);
    let name = CStr::from_ptr(name).to_str().unwrap();
    arg.gl_get_proc_address(name) as *mut c_void
}

fn sdl_example(video_path: &Path) {
    let mut opengl_driver : Option<i32> = None ;
    info!("Detecting drivers ...");
    // SDL drivers are counted from 0
    // Typically here if we want to draw with SDL on mpv we must use the "opengl" driver,
    // and for instance not the direct3d driver (on windows), nor the opengles driver, ...
    let mut driver_index = -1 ;
    for item in sdl2::render::drivers() {
        driver_index = driver_index + 1 ;
        info!("* Found driver '{}'",item.name);
        if item.name == "opengl"{
            opengl_driver = Some(driver_index);
        }
    }
    if let Some(opengl_driver) = opengl_driver {
        let opengl_driver = opengl_driver as u32;
        // initialize SDL
        let sdl_context = sdl2::init().unwrap();
        // initialize the video subsystem for SDL
        let mut video_subsystem = sdl_context.video().unwrap();
        // Create a new window
        let window = video_subsystem.window("MPV-RS SDL2 EXAMPLE", 960, 540)
            .resizable()
            .position_centered()
            .opengl()
            .build()
            .unwrap();
        // Create an SDL renderer with vsync enabled, using the given opengl driver
        let renderer = window.renderer()
            .present_vsync()
            .index(opengl_driver)
            .build()
            .expect("Failed to create renderer with given parameters");

        // this step is necessary
        renderer.window()
                .expect("Failed to extract window from displayer")
                .gl_set_context_to_current()
                .unwrap();
        let ptr = &mut video_subsystem as *mut _ as *mut c_void;
        let mpv_builder = mpv::MpvHandlerBuilder::new().expect("Error while creating MPV builder");
        let mut mpv : Box<mpv::MpvHandlerWithGl> = mpv_builder.build_with_gl(Some(get_proc_address), ptr).expect("Error while initializing MPV with opengl");
        // observe the property "pause" with userdata 5.
        // When we will pause later, an event PropertyChange will be sent with userdata 5
        mpv.observe_property::<bool>("pause",5).unwrap();

        let video_path = video_path.to_str().expect("Expected a string for Path, got None");

        // Send a command synchronously, telling the libmpv core to load a file
        mpv.command(&["loadfile", video_path as &str])
           .expect("Error loading file");

        let mut event_pump = sdl_context.event_pump().expect("Failed to create event_pump");
        'main: loop {
            for event in event_pump.poll_iter() {
                match event {
                    SdlEvent::Quit {..} | SdlEvent::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'main
                    },
                    SdlEvent::KeyDown { keycode: Some(Keycode::Space),repeat: false, .. } => {
                        match mpv.get_property("pause").unwrap() {
                            true => {mpv.set_property_async("pause",false,1).expect("Failed to pause player");},
                            false => {mpv.set_property_async("pause",true,1).expect("Failed to unpause player");}
                        }
                    },
                    // this is useless and only here to show the possibilities of the mpv-rs API
                    SdlEvent::KeyDown { keycode: Some(Keycode::O),repeat: false, .. } => {
                        mpv.get_property_async::<&str>("speed",5).unwrap();
                    },
                    _ => {}
                }
            }
            // wait up to 0.0 seconds for an event.
            while let Some(event) = mpv.wait_event(0.0) {
                // even if you don't do anything with the events, it is still necessary to empty
                // the event loop
                println!("RECEIVED EVENT : {:?}", event);
                match event {
                    mpv::Event::Shutdown | mpv::Event::EndFile(_) => {
                        break 'main;
                    }
                    _ => {}
                };
            }
            let (width, height) = renderer.window().unwrap().size();
            if mpv.is_update_available(){
                mpv.draw(0, width as i32, -(height as i32)).expect("Failed to draw on SDL2 window");
            }
            renderer.window().unwrap().gl_swap_window();
        }
    }else{
        error!("OpenGL driver not found, aborting");
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: ./sdl [any mp4, avi, mkv, ... file]");
    } else {
        let path: &Path = Path::new(&args[1]);
        if path.is_file() {
            sdl_example(path);
        } else {
            println!("A file is required; {} is not a valid file",
                     path.to_str().unwrap());
        }
    }
}

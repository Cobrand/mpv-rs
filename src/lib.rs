#[macro_use]
extern crate enum_primitive;
extern crate num;

mod mpv_error;
mod mpv_enums;
mod mpv_gen;
mod mpv_handler;
mod mpv_gl;

pub use mpv_error::{MpvError,Result};
pub use mpv_handler::*;
pub use mpv_enums::{
    MpvEventId,
    MpvSubApi,
    MpvLogLevel,
    MpvEndFileReason
};
pub use mpv_gl::* ;

/// Returns the MPV_CLIENT_API_VERSION the mpv source has been compiled with
///

pub fn client_api_version() -> u64 {
    unsafe {
        mpv_gen::mpv_client_api_version()
    }
}

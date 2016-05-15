#[macro_use]
extern crate enum_primitive;
extern crate num;

mod mpv_error;
mod mpv_enums;
mod mpv_gen;
mod mpv_handler;

pub use mpv_error::{MpvError,Result};
pub use mpv_handler::*;
pub use mpv_enums::{
    MpvEventId,
    MpvSubApi,
    MpvLogLevel,
    MpvEndFileReason
};

/// Returns the MPV_CLIENT_API_VERSION the mpv source has been compiled with
///

pub fn client_api_version() -> u32 {
    let api_version : ::std::os::raw::c_ulong = unsafe {
        mpv_gen::mpv_client_api_version()
    };
    api_version as u32
}

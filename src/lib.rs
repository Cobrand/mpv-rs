#[macro_use]
extern crate enum_primitive;
extern crate num;

mod mpv_error;
mod mpv_gen;
mod mpv_handler;

pub mod mpv_gl;
pub use mpv_error::{MpvError,Result};
pub use mpv_handler::*;

pub use mpv_gen::{
    MpvFormat,
    MpvEventId,
    MpvSubApi,
    MpvLogLevel,
    MpvEndFileReason
};

pub fn client_api_version() -> u64 {
    unsafe {
        mpv_gen::mpv_client_api_version()
    }
}

#[macro_use]
extern crate enum_primitive;
extern crate num;

mod mpv_error;
mod mpv_gen;
mod mpv_handler;

pub mod mpv_gl;
pub use mpv_error::*;
pub use mpv_handler::*;

pub use mpv_gen::{
    Enum_mpv_format as Format,
    Enum_mpv_event_id as Event,
    Enum_mpv_sub_api as SubApi,
    Enum_mpv_log_level as LogLevel,
    Enum_mpv_end_file_reason as EndFileReason
};

pub fn client_api_version() -> u64 {
    unsafe {
        mpv_gen::mpv_client_api_version()
    }
}

#[macro_use]
extern crate enum_primitive;
extern crate num;

mod mpv_gen;
mod mpv_error;
pub mod mpv_handler;
pub mod mpv_gl;

pub fn client_api_version() -> u64 {
    unsafe {
        mpv_gen::mpv_client_api_version()
    }
}

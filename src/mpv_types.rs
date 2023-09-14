use std::os::raw::{c_void, c_char};

pub struct OsdString<'a> {
    pub string:&'a str
}

pub type GetProcAddressFn = unsafe extern "C" fn(ctx: *mut c_void, name: *const c_char) -> *mut c_void;
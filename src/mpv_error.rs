use ::std::error::Error as StdError ;
use std::{result, ffi, fmt};
pub use num::FromPrimitive;

use mpv_gen::mpv_error_string;
pub use mpv_gen::Error;

pub type Result<T> = result::Result<T, Error>;

impl StdError for Error {
    fn description(&self) -> &str {
        let str_ptr = unsafe { mpv_error_string(*self as ::std::os::raw::c_int) };
        assert!(!str_ptr.is_null());
        unsafe { ffi::CStr::from_ptr(str_ptr).to_str().unwrap() }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({:?})", self.description(), self)
    }
}

pub fn ret_to_result<T>(ret: i32, default: T) -> Result<T> {
    if ret < 0 {
        Err(Error::from_i32(ret).unwrap())
    } else {
        Ok(default)
    }
}

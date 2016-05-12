use std::{ffi, fmt};

use mpv_gen::mpv_event_name;
pub use mpv_gen::{MpvFormat, MpvEventId, MpvSubApi, MpvLogLevel, MpvEndFileReason};

impl MpvEventId {
    pub fn to_str(&self) -> &str {
        let str_ptr = unsafe { mpv_event_name(*self) };
        assert!(!str_ptr.is_null());
        unsafe { ffi::CStr::from_ptr(str_ptr).to_str().unwrap() }
    }
}

impl fmt::Display for MpvEventId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({:?})", self.to_str(), self)
    }
}

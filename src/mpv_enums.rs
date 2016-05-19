use std::{ffi, fmt};

use mpv_error::* ;
use mpv_gen::mpv_event_name;
pub use mpv_gen::{MpvEventId, MpvSubApi, MpvLogLevel, MpvEndFileReason};

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

pub enum Event<'a,'b> {
    Shutdown,
    LogMessage,//(&'a str),
    GetPropertyReply(&'a str,Result<Format<'b>>),
    SetPropertyReply(Result<()>),
    CommandReply,
    StartFile,
    EndFile,
    TrackesSwitched,
    Idle,
    Pause,
    Unpause,
    Tick,
    ClientMessage,
    VideoReconfig,
    AudioReconfig,
    MetadataUpdate,
    Seek,
    PlaybackRestart,
    PropertyChange(&'a str,Format<'b>),
    ChapterChange,
    QueueOverflow
}

pub enum Format<'a>{
    Flag(bool),
    Str(&'a str),
    Double(f64),
    Int(i64),
    OsdStr(&'a str)
}

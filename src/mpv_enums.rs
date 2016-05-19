use std::{ffi, fmt};

use mpv_error::* ;
use mpv_gen::mpv_event_name;
pub use mpv_gen::{MpvEventId, MpvSubApi, MpvLogLevel, MpvEndFileReason};
use ::std::os::raw::{c_int,c_void,c_ulong};

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
    GetPropertyReply(&'a str,Result<(Format<'b>,u32)>),
    SetPropertyReply(Result<u32>),
    CommandReply(Result<u32>),
    StartFile,
    EndFile(Result<MpvEndFileReason>),
    FileLoaded,
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
    PropertyChange(&'a str,Format<'b>,u32),
    ChapterChange,
    QueueOverflow,
    /// Unused event
    Unused
}

pub fn to_event<'a,'b>(event_id:MpvEventId,
                error: c_int,
                reply_userdata: c_ulong,
                data:*mut c_void) -> Option<Event<'a,'b>> {
    match event_id {
        MpvEventId::MPV_EVENT_NONE                  => None,
        MpvEventId::MPV_EVENT_SHUTDOWN              => Some(Event::Shutdown),
        MpvEventId::MPV_EVENT_LOG_MESSAGE           => unimplemented!(),
        MpvEventId::MPV_EVENT_GET_PROPERTY_REPLY    => unimplemented!(),
        MpvEventId::MPV_EVENT_SET_PROPERTY_REPLY    => unimplemented!(),
        MpvEventId::MPV_EVENT_COMMAND_REPLY         => unimplemented!(),
        MpvEventId::MPV_EVENT_START_FILE            => Some(Event::StartFile),
        MpvEventId::MPV_EVENT_END_FILE              => unimplemented!(),
        MpvEventId::MPV_EVENT_FILE_LOADED           => Some(Event::FileLoaded),
        MpvEventId::MPV_EVENT_TRACKS_CHANGED        => Some(Event::Unused),
        MpvEventId::MPV_EVENT_TRACK_SWITCHED        => Some(Event::TrackesSwitched),
        MpvEventId::MPV_EVENT_IDLE                  => Some(Event::Idle),
        MpvEventId::MPV_EVENT_PAUSE                 => Some(Event::Pause),
        MpvEventId::MPV_EVENT_UNPAUSE               => Some(Event::Unpause),
        MpvEventId::MPV_EVENT_TICK                  => Some(Event::Tick),
        MpvEventId::MPV_EVENT_SCRIPT_INPUT_DISPATCH => Some(Event::Unused),
        MpvEventId::MPV_EVENT_CLIENT_MESSAGE        => unimplemented!(),
        MpvEventId::MPV_EVENT_VIDEO_RECONFIG        => Some(Event::VideoReconfig),
        MpvEventId::MPV_EVENT_AUDIO_RECONFIG        => Some(Event::AudioReconfig),
        MpvEventId::MPV_EVENT_METADATA_UPDATE       => Some(Event::MetadataUpdate),
        MpvEventId::MPV_EVENT_SEEK                  => Some(Event::Seek),
        MpvEventId::MPV_EVENT_PLAYBACK_RESTART      => Some(Event::PlaybackRestart),
        MpvEventId::MPV_EVENT_PROPERTY_CHANGE       => unimplemented!(),
        MpvEventId::MPV_EVENT_CHAPTER_CHANGE        => Some(Event::ChapterChange),
        MpvEventId::MPV_EVENT_QUEUE_OVERFLOW        => Some(Event::QueueOverflow),
    }
}

pub enum Format<'a>{
    Flag(bool),
    Str(&'a str),
    Double(f64),
    Int(i64),
    OsdStr(&'a str)
}

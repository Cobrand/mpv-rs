use std::{ffi, fmt, ptr};
use std::ffi::CStr;
use std::mem;

use mpv_error::* ;
use mpv_gen::{mpv_event_name,MpvFormat as MpvInternalFormat};
pub use mpv_gen::{MpvEventId, MpvSubApi, MpvLogLevel, MpvEndFileReason};
use ::std::os::raw::{c_int,c_void,c_ulong,c_char};

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
    let userdata : u32 = reply_userdata as u32 ;
    match event_id {
        MpvEventId::MPV_EVENT_NONE                  => None,
        MpvEventId::MPV_EVENT_SHUTDOWN              => Some(Event::Shutdown),
        MpvEventId::MPV_EVENT_LOG_MESSAGE           => unimplemented!(),
        MpvEventId::MPV_EVENT_GET_PROPERTY_REPLY    => unimplemented!(),
        MpvEventId::MPV_EVENT_SET_PROPERTY_REPLY    => Some(Event::SetPropertyReply(ret_to_result(error, userdata))),
        MpvEventId::MPV_EVENT_COMMAND_REPLY         => Some(Event::CommandReply(ret_to_result(error, userdata))),
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

impl<'a> Format<'a> {
    pub fn get_mpv_format(&self) -> MpvInternalFormat {
        match self {
            &Format::Flag(_) => MpvInternalFormat::MPV_FORMAT_FLAG,
            &Format::Str(_) => MpvInternalFormat::MPV_FORMAT_STRING,
            &Format::Double(_) => MpvInternalFormat::MPV_FORMAT_DOUBLE,
            &Format::Int(_) => MpvInternalFormat::MPV_FORMAT_INT64,
            &Format::OsdStr(_) => MpvInternalFormat::MPV_FORMAT_OSD_STRING,
        }
    }
    pub fn get_from_c_void(format:MpvInternalFormat,pointer:*mut c_void) -> Self {
        match format {
            MpvInternalFormat::MPV_FORMAT_FLAG => {
                Format::Flag(unsafe { *(pointer as *mut bool) })
            },
            MpvInternalFormat::MPV_FORMAT_STRING => {
                let char_ptr : *mut c_char =unsafe{ mem::transmute(*(pointer as *mut *mut c_char))};
                Format::Str(unsafe {
                    CStr::from_ptr(char_ptr)
                         .to_str()
                         .unwrap()
                })
            },
            MpvInternalFormat::MPV_FORMAT_OSD_STRING => {
                let char_ptr : *mut c_char =unsafe{ mem::transmute(*(pointer as *mut *mut c_char))};
                Format::OsdStr(unsafe {
                    CStr::from_ptr(char_ptr)
                         .to_str()
                         .unwrap()
                })
            },
            MpvInternalFormat::MPV_FORMAT_DOUBLE => {
                Format::Double(unsafe { *(pointer as *mut f64) })
            },
            MpvInternalFormat::MPV_FORMAT_INT64 => {
                Format::Int(unsafe { *(pointer as *mut i64) })
            },
            _ => {
                Format::Flag(false)
            }
        }
    }
}

pub trait MpvFormat {
    fn call_as_c_void<F : FnMut(*mut c_void)>(&self,f:F);
    fn get_from_c_void<F : FnMut(*mut c_void)>(mut f:F) -> Self;
    fn get_mpv_format() -> MpvInternalFormat ;
}

impl MpvFormat for f64 {
    fn call_as_c_void<F : FnMut(*mut c_void)>(&self,mut f:F){
        let mut cpy : f64= *self;
        let pointer = &mut cpy as *mut _ as *mut c_void;
        f(pointer)
    }

    fn get_from_c_void<F : FnMut(*mut c_void)>(mut f:F) -> f64 {
        let mut ret_value = f64::default() ;
        let pointer = &mut ret_value as *mut _ as *mut c_void;
        f(pointer);
        ret_value
    }

    fn get_mpv_format() -> MpvInternalFormat {
        MpvInternalFormat::MPV_FORMAT_DOUBLE
    }
}

impl MpvFormat for i64 {
    fn call_as_c_void<F : FnMut(*mut c_void)>(&self,mut f:F){
        let mut cpy : i64= *self;
        let pointer = &mut cpy as *mut _ as *mut c_void;
        f(pointer)
    }

    fn get_from_c_void<F : FnMut(*mut c_void)>(mut f:F) -> i64 {
        let mut ret_value = i64::default() ;
        let pointer = &mut ret_value as *mut _ as *mut c_void;
        f(pointer);
        ret_value
    }

    fn get_mpv_format() -> MpvInternalFormat {
        MpvInternalFormat::MPV_FORMAT_INT64
    }
}

impl MpvFormat for bool {
    fn call_as_c_void<F : FnMut(*mut c_void)>(&self,mut f:F){
        let mut cpy : ::std::os::raw::c_int = if *self == true {
            1
        } else {
            0
        } ;
        let pointer = &mut cpy as *mut _ as *mut c_void;
        f(pointer)
    }

    fn get_from_c_void<F : FnMut(*mut c_void)>(mut f:F) -> bool {
        let mut temp_int = ::std::os::raw::c_int::default() ;
        let pointer = &mut temp_int as *mut _ as *mut c_void;
        f(pointer);
        match temp_int {
            0 => false,
            1 => true,
            _ => unreachable!()
        }
    }

    fn get_mpv_format() -> MpvInternalFormat {
        MpvInternalFormat::MPV_FORMAT_FLAG
    }
}

impl<'a> MpvFormat for &'a str {
    fn call_as_c_void<F : FnMut(*mut c_void)>(&self,mut f:F){
        let string = ffi::CString::new(*self).unwrap();
        let ptr = string.as_ptr();
        f(unsafe {mem::transmute(&ptr)})
    }

    fn get_from_c_void<F : FnMut(*mut c_void)>(mut f:F) -> &'a str {
        let char_ptr : *mut c_char = ptr::null_mut() as *mut c_char;
        f(unsafe {mem::transmute(&char_ptr)});
        unsafe {
            CStr::from_ptr(char_ptr)
                 .to_str()
                 .unwrap()
        }
    }

    fn get_mpv_format() -> MpvInternalFormat {
        MpvInternalFormat::MPV_FORMAT_STRING
    }
}

use std::{ffi, fmt, ptr};
use std::ffi::CStr;
use std::mem;

use mpv_error::* ;
use mpv_types::OsdString;
use mpv_gen::{mpv_event_name,mpv_format as MpvInternalFormat,mpv_event_property,mpv_event_end_file,
    mpv_event_log_message,mpv_free};
pub use mpv_gen::{mpv_event_id as MpvEventId, mpv_log_level as LogLevel, mpv_end_file_reason as EndFileReason};
use ::std::os::raw::{c_int,c_void,c_ulong,c_char};

impl MpvEventId {
    pub fn as_str(&self) -> &str {
        let str_ptr = unsafe { mpv_event_name(*self) };
        assert!(!str_ptr.is_null());
        unsafe { ffi::CStr::from_ptr(str_ptr).to_str().unwrap() }
    }
}

impl fmt::Display for MpvEventId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({:?})", self.as_str(), self)
    }
}

#[derive(Debug)]
pub enum Event<'a> {
    /// Received when the player is shutting down
    Shutdown,
    /// *Has not been tested*, received when explicitly asked to MPV
    LogMessage{prefix:&'static str,level:&'static str,text:&'static str,log_level:LogLevel},
    /// Received when using get_property_async
    GetPropertyReply{name:&'static str,result:Result<Format<'a>>,reply_userdata:u32},
    /// Received when using set_property_async
    SetPropertyReply(Result<()>,u32),
    /// Received when using command_async
    CommandReply(Result<()>,u32),
    /// Event received when a new file is playing
    StartFile,
    /// Event received when the file being played currently has stopped, for an error or not
    EndFile(Result<EndFileReason>),
    /// Event received when a file has been *loaded*, but has not been started
    FileLoaded,
    /// Received when the player has no more files to play and is in an idle state
    Idle,
    Tick,
    /// **Unimplemented**
    ClientMessage,
    VideoReconfig,
    AudioReconfig,
    /// The player changed current position
    Seek,
    PlaybackRestart,
    /// Received when used with observe_property
    PropertyChange{name:&'static str,change:Format<'a>,reply_userdata:u32},
    /// Received when the Event Queue is full
    QueueOverflow,
    /// Unused event
    Unused
}

pub fn to_event<'a>(event_id:MpvEventId,
                error: c_int,
                reply_userdata: c_ulong,
                data:*mut c_void) -> Option<Event<'a>> {
    let userdata = reply_userdata as u32 ;
    match event_id {
        MpvEventId::MPV_EVENT_NONE                  => None,
        MpvEventId::MPV_EVENT_SHUTDOWN              => Some(Event::Shutdown),
        MpvEventId::MPV_EVENT_LOG_MESSAGE           => {
            let log_message = unsafe {*(data as *mut mpv_event_log_message)};
            let prefix = unsafe { CStr::from_ptr(log_message.prefix).to_str().unwrap() };
            let level  = unsafe { CStr::from_ptr(log_message.level ).to_str().unwrap() };
            let text   = unsafe { CStr::from_ptr(log_message.text  ).to_str().unwrap() };
            Some(Event::LogMessage{prefix:prefix,level:level,text:text,log_level:log_message.log_level})
        },
        MpvEventId::MPV_EVENT_GET_PROPERTY_REPLY    => {
            let property_struct = unsafe {*(data as *mut mpv_event_property)};
            let format_result = Format::get_from_c_void(property_struct.format,property_struct.data);
            let string = unsafe {
                CStr::from_ptr(property_struct.name)
                 .to_str()
                 .unwrap()
            };
            let result = ret_to_result(error, format_result);
            Some(Event::GetPropertyReply{name:string,result:result,reply_userdata:userdata})
        },
        MpvEventId::MPV_EVENT_SET_PROPERTY_REPLY    => Some(Event::SetPropertyReply(ret_to_result(error,()), userdata)),
        MpvEventId::MPV_EVENT_COMMAND_REPLY         => Some(Event::CommandReply(ret_to_result(error,()), userdata)),
        MpvEventId::MPV_EVENT_START_FILE            => Some(Event::StartFile),
        MpvEventId::MPV_EVENT_END_FILE              => {
            let end_file = unsafe {*(data as *mut mpv_event_end_file)};
            let result = match end_file.reason {
                EndFileReason::MPV_END_FILE_REASON_ERROR => Err(Error::from_i32(end_file.error).unwrap()),
                reason => Ok(reason)
            };
            Some(Event::EndFile(result))
        }
        MpvEventId::MPV_EVENT_FILE_LOADED           => Some(Event::FileLoaded),
        MpvEventId::MPV_EVENT_IDLE                  => Some(Event::Idle),
        MpvEventId::MPV_EVENT_TICK                  => Some(Event::Tick),
        MpvEventId::MPV_EVENT_CLIENT_MESSAGE        => unimplemented!(),
        MpvEventId::MPV_EVENT_VIDEO_RECONFIG        => Some(Event::VideoReconfig),
        MpvEventId::MPV_EVENT_AUDIO_RECONFIG        => Some(Event::AudioReconfig),
        MpvEventId::MPV_EVENT_SEEK                  => Some(Event::Seek),
        MpvEventId::MPV_EVENT_PLAYBACK_RESTART      => Some(Event::PlaybackRestart),
        MpvEventId::MPV_EVENT_PROPERTY_CHANGE       => {
            let property_struct = unsafe {*(data as *mut mpv_event_property)};
            let format_result = Format::get_from_c_void(property_struct.format,property_struct.data);
            let name = unsafe {
                CStr::from_ptr(property_struct.name)
                 .to_str()
                 .unwrap()
            };
            Some(Event::PropertyChange{name:name,change:format_result,reply_userdata:userdata})
        },
        MpvEventId::MPV_EVENT_QUEUE_OVERFLOW        => Some(Event::QueueOverflow),
        MpvEventId::MPV_EVENT_HOOK                  => unimplemented!(),
    }
}

///
/// Event replies `GetPropertyReply` and `PropertyChange` will answer this object.
///
/// This list is incomplete, the current formats are missing :
///
/// * `Node`
/// * `NodeArray`
/// * `NodeMap`
/// * `ByteArray`

#[derive(Debug)]
pub enum Format<'a>{
    Flag(bool),
    Str(&'a str),
    Double(f64),
    Int(i64),
    OsdStr(&'a str)
}

impl<'a> Format<'a> {
    pub fn get_mpv_format(&self) -> MpvInternalFormat {
        match *self {
            Format::Flag(_) => MpvInternalFormat::MPV_FORMAT_FLAG,
            Format::Str(_) => MpvInternalFormat::MPV_FORMAT_STRING,
            Format::Double(_) => MpvInternalFormat::MPV_FORMAT_DOUBLE,
            Format::Int(_) => MpvInternalFormat::MPV_FORMAT_INT64,
            Format::OsdStr(_) => MpvInternalFormat::MPV_FORMAT_OSD_STRING,
        }
    }
    ///
    /// This is used internally by the mpv-rs crate, you probably should not be using this.
    ///
    pub fn get_from_c_void(format:MpvInternalFormat,pointer:*mut c_void) -> Self {
        match format {
            MpvInternalFormat::MPV_FORMAT_FLAG => {
                Format::Flag(unsafe { *(pointer as *mut bool) })
            },
            MpvInternalFormat::MPV_FORMAT_STRING => {
                let char_ptr = unsafe {*(pointer as *mut *mut c_char)};
                Format::Str(unsafe {
                    CStr::from_ptr(char_ptr)
                         .to_str()
                         .unwrap()
                })
                // TODO : mpv_free
            },
            MpvInternalFormat::MPV_FORMAT_OSD_STRING => {
                let char_ptr = unsafe{ *(pointer as *mut *mut c_char)};
                Format::OsdStr(unsafe {
                    CStr::from_ptr(char_ptr)
                         .to_str()
                         .unwrap()
                })
                // TODO : mpv_free
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

/// This trait is meant to represent which types are allowed
/// to be received and sent through diverse mpv functions,
/// such as set_option, get_property, ...
///
/// # Equivalences
///
/// * `MPV_FORMAT_DOUBLE` : `f64`
/// * `MPV_FORMAT_INT64` : `i64`
/// * `MPV_FORMAT_OSD_STRING` : [`OsdString`](struct.OsdString.html)
/// * `MPV_FORMAT_STRING` : `&'a str`
/// * `MPV_FORMAT_BOOL` : `bool`
/// * `MPV_FORMAT_NODE` : unimplemented for now
/// * `MPV_FORMAT_NODE_ARRAY` : unimplemented
/// * `MPV_FORMAT_BYTE_ARRAY` : unimplemented, expected &'a [u8]
///

pub trait MpvFormat {
    fn call_as_c_void<F : FnMut(*mut c_void)>(&self,f:F);
    fn get_from_c_void<F : FnMut(*mut c_void)>(f: F) -> Self;
    fn get_mpv_format() -> MpvInternalFormat ;
}

impl MpvFormat for f64 {
    fn call_as_c_void<F : FnMut(*mut c_void)>(&self,mut f:F){
        let mut cpy = *self;
        let pointer = &mut cpy as *mut _ as *mut c_void;
        f(pointer)
    }

    fn get_from_c_void<F : FnMut(*mut c_void)>(mut f: F) -> f64 {
        let mut ret_value = 0.0;
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
        let mut cpy = *self;
        let pointer = &mut cpy as *mut _ as *mut c_void;
        f(pointer)
    }

    fn get_from_c_void<F : FnMut(*mut c_void)>(mut f:F) -> i64 {
        let mut ret_value = 0;
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
        let mut cpy = if *self {
            1
        } else {
            0
        } ;
        let pointer = &mut cpy as *mut _ as *mut c_void;
        f(pointer)
    }

    fn get_from_c_void<F : FnMut(*mut c_void)>(mut f:F) -> bool {
        let mut temp_int = c_int::default() ;
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
        // transmute needed for *const -> *mut
        // Should be ok since mpv doesn't modify *ptr
        f(unsafe {mem::transmute(&ptr)})
    }

    fn get_from_c_void<F : FnMut(*mut c_void)>(mut f:F) -> &'a str {
        let mut char_ptr = ptr::null_mut() as *mut c_void;
        f(&mut char_ptr as *mut *mut c_void as *mut c_void);
        if char_ptr.is_null() {
            // if this is still a nullptr (like, in an error)
            // the code below will segfault
            // since it runs *before* checking for an mpv error
            return "";
        }
        let return_str = unsafe {
            CStr::from_ptr(char_ptr as *mut c_char)
                 .to_str()
                 .unwrap()
        };
        unsafe {mpv_free(char_ptr)};
        return_str
    }

    fn get_mpv_format() -> MpvInternalFormat {
        MpvInternalFormat::MPV_FORMAT_STRING
    }
}

impl<'a> MpvFormat for OsdString<'a> {
    fn call_as_c_void<F : FnMut(*mut c_void)>(&self,mut f:F){
        let string = ffi::CString::new(self.string).unwrap();
        let ptr = string.as_ptr();
        // transmute needed for *const -> *mut
        // Should be ok since mpv doesn't modify *ptr
        f(unsafe {mem::transmute(&ptr)})
    }

    fn get_from_c_void<F : FnMut(*mut c_void)>(mut f:F) -> OsdString<'a> {
        let mut char_ptr = ptr::null_mut() as *mut c_void;
        f(&mut char_ptr as *mut *mut c_void as *mut c_void);
        if char_ptr.is_null() {
            // if this is still a nullptr (like, in an error)
            // the code below will segfault
            // since it runs *before* checking for an mpv error
            return OsdString{string:""};
        }
        let return_str = unsafe {
            CStr::from_ptr(char_ptr as *mut c_char)
                 .to_str()
                 .unwrap()
        };
        unsafe {mpv_free(mem::transmute(char_ptr))};
        OsdString{string:return_str}
    }

    fn get_mpv_format() -> MpvInternalFormat {
        MpvInternalFormat::MPV_FORMAT_OSD_STRING
    }
}

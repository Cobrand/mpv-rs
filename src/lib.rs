//! libmpv bindings for Rust
//!
//! Most of the documentation in this crate is based (and even copied) from
//! [this file](https://github.com/mpv-player/mpv/blob/master/libmpv/client.h) and
//! [this file](https://github.com/mpv-player/mpv/blob/master/libmpv/opengl_cb.h)
//!
//! # License
//!
//! the libmpv client API is licensed under ISC to ease
//! interoperability with other licenses. But keep in mind that the
//! mpv core is still mostly GPLv2+. It's up to lawyers to decide
//! whether applications using this API are affected by the GPL.
//!
//! One argument against this is that proprietary applications
//! using mplayer in slave mode is apparently tolerated, and this
//! API is basically equivalent to slave mode.
//!
//! As for this crate itself, it is licensed under the zlib license
//!
//! # Additional Documentation
//!
//! Additional documentation can be found in these two files mentionned before.
//!
//! Most actual interaction with this player is done through
//! options/commands/properties, which can be accessed through this API.
//! Essentially everything is done with them, including loading a file,
//! retrieving playback progress, and so on.
//!
//! These are documented elsewhere:
//!
//! * http://mpv.io/manual/master/#options
//! * http://mpv.io/manual/master/#list-of-input-commands
//! * http://mpv.io/manual/master/#properties
//!
//!
//! # Event loop
//!
//! In general, the API user should run an event loop in order to receive events.
//! This event loop should call mpv.wait_event(...), which will return once a new
//! mpv client API is available. It is also possible to integrate client API
//! usage in other event loops (e.g. GUI toolkits) with the
//! mpv.set_wakeup_callback() function, and then polling for events by calling
//! mpv_wait_event() with a 0 timeout.
//!
//! Note that the event loop is detached from the actual player. Not calling
//! mpv.wait_event() will not stop playback. It will eventually congest the
//! event queue of your API handle, though.
//!
//! # Synchronous vs. asynchronous calls
//!
//! The libmpv API allows both synchronous and asynchronous calls. Synchronous calls
//! have to wait until the playback core is ready, which currently can take
//! an unbounded time (e.g. if network is slow or unresponsive). Asynchronous
//! calls just queue operations as requests, and return the result of the
//! operation as events.
//!
//! As for right now, asynchronous calls are not implemented in mpv-rs
//!
//! # Asynchronous calls
//!
//! The client API includes asynchronous functions. These allow you to send
//! requests instantly, and get replies as events at a later point. The
//! requests are made with functions carrying the _async suffix, and replies
//! are returned by mpv_wait_event() (interleaved with the normal event stream).
//!
//! A unsigned userdata value is used to allow the user to associate requests
//! with replies. The value is passed as reply_userdata parameter to the request
//! function. The reply to the request will have the reply
//! MpvEvent.reply_userdata field set to the same value as the
//! userdata parameter of the corresponding request.
//!
//! This userdata value is arbitrary and is never interpreted by the API. Note
//! that the userdata value 0 is also allowed, but then the client must be
//! careful not accidentally interpret the mpv_event->reply_userdata if an
//! event is not a reply. (For non-replies, this field is set to 0.)
//!
//! Currently, asynchronous calls are always strictly ordered (even with
//! synchronous calls) for each client, although that may change in the future.
//!
//! # Multithreading
//!
//! The libmpv API is generally fully thread-safe, unless otherwise noted.
//! Currently, there is no real advantage in using more than 1 thread to access
//! the client API, since everything is serialized through a single lock in the
//! playback core.
//!
//! # Basic environment requirements
//!
//! This documents basic requirements on the C environment. This is especially
//! important if mpv is used as library with mpv_create(), such as with this mpv-rs crate.
//!
//! * The LC_NUMERIC locale category must be set to "C". If your program calls
//!   setlocale(), be sure not to use LC_ALL, or if you do, reset LC_NUMERIC
//!   to its sane default: setlocale(LC_NUMERIC, "C").
//! * If a X11 based VO is used, mpv will set the xlib error handler. This error
//!   handler is process-wide, and there's no proper way to share it with other
//!   xlib users within the same process. This might confuse GUI toolkits.
//! * mpv uses some other libraries that are not library-safe, such as Fribidi
//!   (used through libass), ALSA, FFmpeg, and possibly more.
//! * The FPU precision must be set at least to double precision.
//! * On Windows, mpv will call timeBeginPeriod(1).
//! * On UNIX, every mpv_initialize() call will block SIGPIPE. This is done
//!   because FFmpeg makes unsafe use of OpenSSL and GnuTLS, which can raise
//!   this signal under certain circumstances. Once these libraries (or FFmpeg)
//!   are fixed, libmpv will not block the signal anymore.
//! * On memory exhaustion, mpv will kill the process.
//!
//! # Encoding of filenames
//!
//! Like Rust, libmpv uses UTF-8 everywhere.
//!
//!
//! On OS X, filenames and other strings taken/returned by libmpv can have
//! inconsistent unicode normalization. This can sometimes lead to problems.
//! You have to hope for the best.
//!
//! Also see the remarks for MPV_FORMAT_STRING.
//!
//!



#[macro_use]
extern crate enum_primitive;
extern crate num;

mod mpv_error;
mod mpv_enums;
mod mpv_gen;
mod mpv_handler;
mod mpv_types;

pub use mpv_error::{Error,Result};
pub use mpv_handler::*;
pub use mpv_enums::{
    SubApi,
    LogLevel,
    EndFileReason,
    Event,
    MpvFormat,
    Format
};
pub use mpv_types::* ;
pub use mpv_gen::mpv_opengl_cb_get_proc_address_fn;

/// Returns the MPV_CLIENT_API_VERSION the mpv source has been compiled with
/// as (major_v,minor_v)

pub fn client_api_version() -> (u16,u16) {
    let api_version : ::std::os::raw::c_ulong = unsafe {
        mpv_gen::mpv_client_api_version()
    };
    ((api_version >> 16) as u16, (api_version & 0xFFFF) as u16)
}

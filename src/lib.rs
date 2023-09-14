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
//! using mplayer in slave mode is apparently tolerated, and the libmpv
//! API used by this crate is basically equivalent to slave mode.
//!
//! As for this crate itself, it is licensed under the dual MIT / Apache-2.0 license.
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
//! # Differences with mpv
//!
//! This crate uses the libmpv API, which is different from the command line player mpv.
//! Event though both share the same core, some differences must be noted. Taken directly
//! from the libmpv docs :
//!
//! > Unlike the command line player, this will have initial settings suitable
//! > for embedding in applications. The following settings are different:
//! >
//! > * stdin/stdout/stderr and the terminal will never be accessed. This is
//! >   equivalent to setting the --no-terminal option.
//! >   (Technically, this also suppresses C signal handling.)
//! > * No config files will be loaded. This is roughly equivalent to using
//! >   --no-config. Since libmpv 1.15, you can actually re-enable this option,
//! >   which will make libmpv load config files during `mpv.init()`. If you
//! >   do this, you are strongly encouraged to set the "config-dir" option too.
//! >   (Otherwise it will load the mpv command line player's config.)
//! > * Idle mode is enabled, which means the playback core will enter idle mode
//! >   if there are no more files to play on the internal playlist, instead of
//! >   exiting. This is equivalent to the --idle option.
//! > * Disable parts of input handling.
//! > * Most of the different settings can be viewed with the command line player
//! >   by running `mpv --show-profile=libmpv`.
//!
//! # Event loop
//!
//! In general, the API user should run an event loop in order to receive events.
//! This event loop should call `mpv.wait_event(...)`, which will return once a new
//! mpv client API is available. It is also possible to integrate client API
//! usage in other event loops (e.g. GUI toolkits) with the
//! `mpv.set_wakeup_callback()` function, and then polling for events by calling
//! `mpv_wait_event()` with a 0 timeout.
//!
//! Note that the event loop is detached from the actual player. Not calling
//! `mpv.wait_event()` will not stop playback. It will eventually congest the
//! event queue of your API handle, though, that is why should still empty
//! the event loop, even though you do not use the events.
//!
//! # Synchronous vs. asynchronous calls
//!
//! The libmpv API allows both synchronous and asynchronous calls. Synchronous calls
//! have to wait until the playback core is ready, which currently can take
//! an unbounded time (e.g. if network is slow or unresponsive). Asynchronous
//! calls just queue operations as requests, and return the result of the
//! operation as events.
//!
//! # Asynchronous calls
//!
//! The client API includes asynchronous functions. These allow you to send
//! requests instantly, and get replies as events at a later point. The
//! requests are made with functions carrying the _async suffix, and replies
//! are returned by `mpv.wait_event(...)` (interleaved with the normal event stream).
//!
//! A unsigned userdata value is used to allow the user to associate requests
//! with replies. The value is passed as `reply_userdata` parameter to the request
//! function. The reply to the request will have the reply
//! `MpvEvent.reply_userdata` field set to the same value as the
//! userdata parameter of the corresponding request.
//!
//! This userdata value is arbitrary and is never interpreted by the libmpv API nor this crate.
//!
//! > *Note that the userdata value 0 is also allowed, but then the client must be
//! > careful not accidentally interpret the `mpv_event->reply_userdata` if an
//! > event is not a reply. (For non-replies, this field is set to 0.)*
//!
//! This comment is from the libmpv API and is not valid in this crate since the `reply_userdata`
//! field is suppressed for non-reply events
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
//! important if mpv is used as library with `mpv_create()`, such as with this mpv-rs crate.
//!
//! * The `LC_NUMERIC` locale category must be set to "C". If your program calls
//!   setlocale(), be sure not to use `LC_ALL`, or if you do, reset `LC_NUMERIC`
//!   to its sane default: `setlocale(LC_NUMERIC, "C")`.
//! * If a X11 based VO is used, mpv will set the xlib error handler. This error
//!   handler is process-wide, and there's no proper way to share it with other
//!   xlib users within the same process. This might confuse GUI toolkits.
//! * mpv uses some other libraries that are not library-safe, such as Fribidi
//!   (used through libass), ALSA, FFmpeg, and possibly more.
//! * The FPU precision must be set at least to double precision.
//! * On Windows, mpv will call `timeBeginPeriod(1)`.
//! * On UNIX, every `mpv_initialize()` call will block SIGPIPE. This is done
//!   because FFmpeg makes unsafe use of OpenSSL and GnuTLS, which can raise
//!   this signal under certain circumstances. Once these libraries (or FFmpeg)
//!   are fixed, libmpv will not block the signal anymore.
//! * On memory exhaustion, mpv will kill the process.
//!
//! # Encoding of filenames
//!
//! Like Rust, libmpv uses UTF-8 everywhere.
//!
//! On OS X, filenames and other strings taken/returned by libmpv can have
//! inconsistent unicode normalization. This can sometimes lead to problems.
//! You have to hope for the best.
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
    LogLevel,
    EndFileReason,
    Event,
    MpvFormat,
    Format
};
pub use mpv_types::* ;

/// Returns the `MPV_CLIENT_API_VERSION` the mpv source has been compiled with
/// as `(major_v,minor_v)`

pub fn client_api_version() -> (u16,u16) {
    let api_version : ::std::os::raw::c_ulong = unsafe {
        mpv_gen::mpv_client_api_version()
    };
    ((api_version >> 16) as u16, (api_version & 0xFFFF) as u16)
}

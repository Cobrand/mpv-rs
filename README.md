
# mpv-rs

safe libmpv bindings for Rust.

> mpv is a media player based on MPlayer and mplayer2.
> It supports a wide variety of video file formats, audio and video codecs, and subtitle types.

The crate is a safe libmpv API binding in Rust. For more info about mpv,
see [here](https://github.com/mpv-player/mpv).

[Documentation](https://docs.rs/mpv/)

# Installing

    [dependencies]
    mpv = "0.2"

the mpv package is needed for this to run.

## Linux

On linux, you can ask your package manager to install it for you.

### Arch

    # pacman -S mpv

### Debian-based systems

    # apt-get install libmpv1

## Windows

libmpv can be found [here](https://mpv.srsfckn.biz/) for windows.
You need to copy the library into your rust binaries folder for it to be
linked properly.

# Running

2 examples are available from the get go in this crate.

_simple.rs_ will alow you to display a standard mpv player in a new window.
Controls will be available.

    $ cargo run --example simple

_sdl2.rs_ will embed mpv in an sdl2 window. Controls will not be available and
cannot be. If you want an interface on top of this player,
you must draw your own with OpenGL calls or SDL2 calls.

    $ cargo run --example sdl2

# Contributing

Any contribution is welcome, as well as any code review !

## What is left to implement

If you want to contribute, there are quite a few less-used functions and structs of mpv that can be added to mpv-rs

### Events :

* (advanced) [ClientMessage](https://github.com/mpv-player/mpv/blob/master/libmpv/client.h#L1375)

### Formats :

* (advanced) [Node](https://github.com/mpv-player/mpv/blob/master/libmpv/client.h#L677)
* (advanced) [NodeList](https://github.com/mpv-player/mpv/blob/master/libmpv/client.h#L716)
* NodeMap
* (easy) [ByteArray](https://github.com/mpv-player/mpv/blob/master/libmpv/client.h#L716)

### MpvHandler impls :

* (easy) load_config_file
* (easy) [detach_destroy](https://github.com/mpv-player/mpv/blob/master/libmpv/client.h#L431)
(if only I knew what this was for ?)
* (easy) [client_name](https://github.com/mpv-player/mpv/blob/master/libmpv/client.h#L361)
* (advanced) set_wakeup_callback
* (advanced) get_wakeup_pipe
* (easy) request_log_messages
* (easy) request_event

### MpvHandlerWithGl impls :

* (easy) raw_opengl_ctx() : return the raw opengl context
* (easy, but hard testing) [report_flip](https://github.com/mpv-player/mpv/blob/master/libmpv/opengl_cb.h#L313)
* (very hard, probably requires a third-party library such as futures-rs) [set_update_callback](https://github.com/mpv-player/mpv/blob/master/libmpv/opengl_cb.h#L217) (partially implemented via update_available)

### Refactor

(easy, long) This crate was done on my early rust days, and as such the current code is very poorly organized. Without changing how this crate works, refactoring everything in coherent files would be a huge plus.

# Submitting an issue

Any question concerning the mpv-rs API is welcome in the issues.

If your mpv crashes, please make sure it's coming from this Rust binding and
not from libmpv itself.

# License

mpv is globally licensed under GPLv2+
(see [here](https://github.com/mpv-player/mpv#license)), but this crate is
licensed under the MIT/Apache-2.0 (at your option).


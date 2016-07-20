
# mpv-rs

safe libmpv bindings for Rust.

> mpv is a media player based on MPlayer and mplayer2.
> It supports a wide variety of video file formats, audio and video codecs, and subtitle types.

The crate is a safe libmpv API binding in Rust. For more info about mpv,
see [here](https://github.com/mpv-player/mpv).

Online documentation for version 0.2.1 (19 July 2016) of this crate is available [at this location](http://cobrand.github.io/mpv-rs/mpv/)

If the documentation is outdated and/or the crate is outdated (major changes to master), please submit an issue.

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

### Events :

* [ClientMessage](https://github.com/mpv-player/mpv/blob/master/libmpv/client.h#L1375)

### Formats :

* [Node](https://github.com/mpv-player/mpv/blob/master/libmpv/client.h#L677)
* [NodeList](https://github.com/mpv-player/mpv/blob/master/libmpv/client.h#L716)
* NodeMap
* [ByteArray](https://github.com/mpv-player/mpv/blob/master/libmpv/client.h#L716)

### MpvHandler impls :

* [resume](https://github.com/mpv-player/mpv/blob/master/libmpv/client.h#L523) / suspend core (**not** pause/unpause !)
* load_config_file
* [detach_destroy](https://github.com/mpv-player/mpv/blob/master/libmpv/client.h#L431)
(if only I knew what this was for ?)
* [client_name](https://github.com/mpv-player/mpv/blob/master/libmpv/client.h#L361)
* set_wakeup_callback
* get_wakeup_pipe
* request_log_messages
* request_event

### MpvHandlerWithGl impls :

* raw_opengl_ctx() : return the raw opengl context
* [report_flip](https://github.com/mpv-player/mpv/blob/master/libmpv/opengl_cb.h#L313)
* [set_update_callback](https://github.com/mpv-player/mpv/blob/master/libmpv/opengl_cb.h#L217) (partially implemented via update_available)

## API breaks and version numbers

* Minor beta changes (0.X.0) -> (0.X.1) are non-breaking changes.
* Major beta changes (0.1.X) -> (0.2.0) will probably break the API.
* Once this binding is finished, this crate will be released as 1.0.0

# Submitting an issue

Any question concerning the mpv-rs API is welcome in the issues.

If your mpv crashes, please make sure it's coming from this Rust binding and
not from the mpv player itself.

# License

mpv is globally licensed under GPLv2+
(see [here](https://github.com/mpv-player/mpv#license)), but this crate is
licensed under the zlib license.

Relicensing is possible if it can be properly justified for whatever reason.

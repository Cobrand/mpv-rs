
# mpv-rs

safe mpv bindings for Rust v0.2.0

# Installing

the mpv package is needed for this to run.

## Linux

On linux, you can ask your package manager to install it for you.

### Arch

    $ sudo pacman -S mpv

### Debian-based systems

    $ sudo apt-get install libmpv1

## Windows

libmpv can be found [here](https://mpv.srsfckn.biz/) for windows.
You need to copy the library into your rust binaries folder for it to be
linked properly.

# Contributing

Any contribution is welcome, as well as any code review !

## What is left to implement

### Events :

* ClientMessage

### Formats :

* Node
* NodeList
* NodeMap
* ByteArray

### MpvHandler impls :

* [get_time_us](https://github.com/mpv-player/mpv/blob/master/libmpv/client.h#L537)
* resume / suspend core (**not** pause/unpause !)
* load_config_file
* detach_destroy (if only I knew what this was for ?)
* [client_name](https://github.com/mpv-player/mpv/blob/master/libmpv/client.h#L361)
* set_wakeup_callback
* get_wakeup_pipe
* request_log_messages
* request_event

### MpvHandlerWithGl impls :

* raw_opengl_ctx() : return the raw opengl context
* report_flip
* set_update_callback (partly implemented via update_available)

# Submitting an issue

An question concerning the mpv-rs API is welcome in the issues.

If your mpv crashes, please make sure it's coming from this API and not from the
 mpv player itself.

# License

mpv is globally licensed under GPLv2+
(see [here](https://github.com/mpv-player/mpv#license)), but this crate is
licensed under the zlib license.

Relicensing is possible if it can be properly justified for whatever reason.

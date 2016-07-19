use mpv_gen::{mpv_command, mpv_command_async, mpv_wait_event, mpv_create, mpv_initialize,
              mpv_terminate_destroy, mpv_handle, mpv_set_option,
              mpv_set_property, mpv_set_property_async, mpv_get_property,
              mpv_get_property_async, mpv_opengl_cb_get_proc_address_fn, mpv_get_sub_api,
              mpv_opengl_cb_uninit_gl, mpv_opengl_cb_init_gl, mpv_opengl_cb_draw,
              mpv_opengl_cb_context, mpv_observe_property, mpv_unobserve_property,
              mpv_opengl_cb_set_update_callback, mpv_get_time_us};
use mpv_enums::*;
use mpv_error::*;

use std::os::raw::c_void;
use std::{ffi, ptr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::boxed::Box;
use std::ops::{Deref,DerefMut};
/// The main struct of the mpv-rs crate
///
/// Almost every function from the libmpv API needs a context, which is stored in this struct.
///
pub struct MpvHandler {
    handle: *mut mpv_handle,
}


///
/// This struct is a decorator of `MpvHandler`, and can use all the functions from `MpvHandler`.
/// It is only used when you must embed mpv somewhere else using openGL.
///
pub struct MpvHandlerWithGl {
    mpv_handler:     MpvHandler,
    gl_context:      *mut mpv_opengl_cb_context,
    update_available:AtomicBool
}

pub struct MpvHandlerBuilder {
    handle: *mut mpv_handle,
}

/// A must-use `MpvHandler` builder.
///
/// * **Step 1** : call `MpvHandler::new()` to create a Builder.
/// * **Step 2** : Add options to your player
/// * **Step 3** : Finish creating your `MpvHandler`, either with `build()` or `build_with_gl(...)`
///
impl MpvHandlerBuilder {

    ///
    /// Returns a std::Result that contains an MpvHandlerBuilder if successful,
    /// or an Error is the creation failed. Currently, errors can happen in the following
    /// situations :
    ///         - out of memory
    ///         - LC_NUMERIC is not set to "C" (see general remarks)
    #[must_use]
    pub fn new() -> Result<Self> {
        let handle = unsafe { mpv_create() };
        if handle == ptr::null_mut() {
            return Err(Error::MPV_ERROR_NOMEM);
        }
        ret_to_result(0,MpvHandlerBuilder { handle:     handle })
    }

    ///
    /// All options for your mpv player should be set on this step
    ///
    /// # Example
    /// ```
    /// let mut mpv_builder = mpv::MpvHandlerBuilder::new().expect("Failed to init MPV builder");
    /// mpv_builder.set_option("sid","no").expect("Failed to set option 'sid' to 'no'");
    /// // set other options
    /// // Build the MpvHandler later
    /// ```
    #[cfg_attr(feature = "clippy", allow(temporary_cstring_as_ptr))]
    pub fn set_option<T : MpvFormat>(&mut self, property: &str, option: T) -> Result<()> {
        let mut ret = 0 ;
        let format = T::get_mpv_format();
        option.call_as_c_void(|ptr:*mut c_void|{
            ret = unsafe {
                mpv_set_option(self.handle,
                               ffi::CString::new(property).unwrap().as_ptr(),
                               format,
                               ptr)
            }
        });
        ret_to_result(ret,())
    }

    /// shortcut for `set_option("hwdec","auto")`
    ///
    /// If it is available, the playing will try hardware decoding
    pub fn try_hardware_decoding(&mut self) -> Result<()> {
        self.set_option("hwdec","auto")
    }

    ///
    /// Finish creating your player. It will spawn a new window on your window manager.
    /// Note that it returns a Box of MpvHandler because it needs to be allocated on the heap;
    /// The Rust MpvHandler gives its own pointer the the C mpv API, and moving the MpvHandler
    /// within the stack is forbidden in that case.
    #[must_use]
    pub fn build(self) -> Result<MpvHandler> {
        let ret = unsafe { mpv_initialize(self.handle) };

        ret_to_result(ret,MpvHandler {
            handle:             self.handle,
        })
    }

    ///
    /// Finish creating your player, using a custom opengl instance. It will **not** spawn a new,
    /// window on your window manager, but instead use the given opengl context to draw the video.
    ///
    /// An option of an 'extern "C"' function must be passed as a parameter,
    /// which fullfills the role of get_proc_address.
    /// An arbitrary opaque user context which will be passed to the
    /// get_proc_address callback must also be sent.
    ///
    /// # Errors
    ///
    /// * MPV_ERROR_UNSUPPORTED: the OpenGL version is not supported
    ///                          (or required extensions are missing)
    ///
    /// For additional information, see examples/sdl2.rs for a basic implementation with a sdl2 opengl context
    #[must_use]
    pub fn build_with_gl(mut self,
                         get_proc_address: mpv_opengl_cb_get_proc_address_fn,
                         get_proc_address_ctx: *mut ::std::os::raw::c_void) -> Result<Box<MpvHandlerWithGl>> {
        self.set_option("vo", "opengl-cb").expect("Error setting vo option to opengl-cb");
        let mpv_handler_result = self.build();
        match mpv_handler_result {
            Ok(mpv_handler) => {
                let opengl_ctx = unsafe {
                    mpv_get_sub_api(mpv_handler.handle,
                                    SubApi::MPV_SUB_API_OPENGL_CB)
                } as *mut mpv_opengl_cb_context ;
                let ret = unsafe {
                    mpv_opengl_cb_init_gl(opengl_ctx, ptr::null(), get_proc_address, get_proc_address_ctx)
                };
                // Actually using the opengl_cb state has to be explicitly requested.
                // Otherwise, mpv will create a separate platform window.

                let mut mpv_handler_with_gl = Box::new(MpvHandlerWithGl {
                    mpv_handler:      mpv_handler,
                    gl_context:       opengl_ctx,
                    update_available: AtomicBool::new(false)
                });

                // Additional callback
                unsafe {mpv_opengl_cb_set_update_callback(opengl_ctx,
                                                          Some(MpvHandlerWithGl::update_draw),
                                                          mpv_handler_with_gl.as_mut() as *mut MpvHandlerWithGl as *mut c_void)};
                ret_to_result(ret,mpv_handler_with_gl)
            },
            Err(e) => {
                Err(e)
            }
        }
    }
}

impl Deref for MpvHandlerWithGl {
    type Target = MpvHandler;
    fn deref(&self) -> &MpvHandler {
        &self.mpv_handler
    }
}

impl DerefMut for MpvHandlerWithGl {
    fn deref_mut(&mut self) -> &mut MpvHandler {
        &mut self.mpv_handler
    }
}

impl MpvHandlerWithGl {
    /// Render video
    ///
    /// The video will use the full provided framebuffer. Options like "panscan" are
    /// applied to determine which part of the video should be visible and how the
    /// video should be scaled. You can change these options at runtime by using the
    /// mpv property API.
    ///
    /// fbo is the framebuffer object to render on. Because the renderer might
    /// manage multiple FBOs internally for the purpose of video
    /// postprocessing, it will always bind and unbind FBOs itself. If
    /// you want mpv to render on the main framebuffer, pass 0.
    ///
    /// width is the width of the framebuffer. This is either the video size if the fbo
    /// parameter is 0, or the allocated size of the texture backing the
    /// fbo. The renderer will always use the full size of the fbo.
    ///
    /// height is the height of the framebuffer. Same as with the w parameter, except
    /// that this parameter can be negative. In this case, the video
    /// frame will be rendered flipped.
    ///
    /// # Errors
    ///
    /// If the external video module has not been configured correctly, libmpv can send various
    /// errors such as MPV_ERROR_UNSUPPORTED
    ///
    pub fn draw(&mut self, fbo: i32, width: i32, heigth: i32) -> Result<()> {
        self.update_available.store(false,Ordering::Relaxed) ;
        let ret = unsafe { mpv_opengl_cb_draw(self.gl_context, fbo, width, heigth) };
        ret_to_result(ret, ())
    }

    unsafe extern "C" fn update_draw(cb_ctx: *mut ::std::os::raw::c_void) {
        let ptr = cb_ctx as *mut MpvHandlerWithGl ;
        assert!(!ptr.is_null());
        let mpv = &mut (*ptr) ;
        mpv.update_available.store(true, Ordering::Relaxed);
    }

    /// returns true if another frame is available
    pub fn is_update_available(&self) -> bool {
        self.update_available.load(Ordering::Relaxed)
    }
}

impl MpvHandler {

    /// Set a property synchronously
    #[cfg_attr(feature = "clippy", allow(temporary_cstring_as_ptr))]
    pub fn set_property<T : MpvFormat>(&mut self, property: &str, value : T) -> Result<()>{
        let mut ret = 0 ;
        let format = T::get_mpv_format();
        value.call_as_c_void(|ptr:*mut c_void|{
            ret = unsafe {
                mpv_set_property(self.handle,
                                 ffi::CString::new(property).unwrap().as_ptr(),
                                 format,
                                 ptr)
            }
        });
        ret_to_result(ret,())
    }

    /// Set a property asynchronously
    #[cfg_attr(feature = "clippy", allow(temporary_cstring_as_ptr))]
    pub fn set_property_async<T : MpvFormat>(&mut self, property: &str, value : T, userdata:u32) -> Result<()>{
        let userdata = userdata as ::std::os::raw::c_ulong;
        let mut ret = 0 ;
        let format = T::get_mpv_format();
        value.call_as_c_void(|ptr:*mut c_void|{
            ret = unsafe {
                mpv_set_property_async(self.handle,
                                       userdata,
                                       ffi::CString::new(property).unwrap().as_ptr(),
                                       format,
                                       ptr)
            }
        });
        ret_to_result(ret,())
    }

    /// Get a property synchronously
    #[cfg_attr(feature = "clippy", allow(temporary_cstring_as_ptr))]
    pub fn get_property<T : MpvFormat>(&self, property: &str) -> Result<T> {
        let mut ret = 0 ;
        let format = T::get_mpv_format();
        let result = T::get_from_c_void(|ptr:*mut c_void|{
            ret = unsafe {
                mpv_get_property(self.handle,
                                 ffi::CString::new(property).unwrap().as_ptr(),
                                 format,
                                 ptr)
            }
        });
        ret_to_result(ret,result)
    }

    /// Get a property asynchronously
    #[cfg_attr(feature = "clippy", allow(temporary_cstring_as_ptr))]
    pub fn get_property_async<T : MpvFormat>(&self, property: &str, userdata :u32) -> Result<()> {
        let userdata = userdata as ::std::os::raw::c_ulong;
        let ret = unsafe {
            mpv_get_property_async(self.handle,
                                   userdata,
                                   ffi::CString::new(property).unwrap().as_ptr(),
                                   T::get_mpv_format())
        };
        ret_to_result(ret,())
    }

    ///
    /// Set an option. Note that you can't normally set options during runtime :
    /// changing options at runtime does not always work. For some options, attempts
    /// to change them simply fails. Many other options may require reloading the
    /// file for changes to take effect. In general, you should prefer calling
    /// mpv.set_property() to change settings during playback, because the property
    /// mechanism guarantees that changes take effect immediately.
    ///
    /// It is preferred that you initialize your options with the Builder instead
    ///
    #[cfg_attr(feature = "clippy", allow(temporary_cstring_as_ptr))]
    pub fn set_option<T : MpvFormat>(&mut self, property: &str, option: T) -> Result<()> {
        let mut ret = 0 ;
        let format = T::get_mpv_format();
        option.call_as_c_void(|ptr:*mut c_void|{
            ret = unsafe {
                mpv_set_option(self.handle,
                                 ffi::CString::new(property).unwrap().as_ptr(),
                                 format,
                                 ptr)
            }
        });
        ret_to_result(ret,())
    }

    /// Send a command synchronously
    pub fn command(&mut self, command: &[&str]) -> Result<()> {
        let command_cstring: Vec<_> = command.iter()
                                             .map(|item| ffi::CString::new(*item).unwrap())
                                             .collect();
        let mut command_pointers: Vec<_> = command_cstring.iter()
                                                          .map(|item| item.as_ptr())
                                                          .collect();
        command_pointers.push(ptr::null());

        let ret = unsafe { mpv_command(self.handle, command_pointers.as_mut_ptr()) };

        ret_to_result(ret, ())
    }

    /// Send a command asynchronously
    pub fn command_async(&mut self, command: &[&str], userdata :u32) -> Result<()> {
        let userdata = userdata as ::std::os::raw::c_ulong;
        let command_cstring: Vec<_> = command.iter()
                                             .map(|item| ffi::CString::new(*item).unwrap())
                                             .collect();
        let mut command_pointers: Vec<_> = command_cstring.iter()
                                                          .map(|item| item.as_ptr())
                                                          .collect();
        command_pointers.push(ptr::null());
        let ret = unsafe { mpv_command_async(self.handle, userdata,command_pointers.as_mut_ptr())};

        ret_to_result(ret, ())
    }

    /// Returns an Event if there is an Event available. Returns None if the event pool is empty.
    ///
    /// It is still necessary to empty the event pool even if you don't use the events, since
    /// the event pool is not limited and will be full if you don't empty it.
    ///
    /// # Panics
    ///
    /// Will panic if a null pointer is received from the libmpv API (should never happen)

    pub fn wait_event<'a>(&mut self,timeout:f64) -> Option<Event<'a>> {
        let event = unsafe {
            let ptr = mpv_wait_event(self.handle, timeout);
            if ptr.is_null() {
                panic!("Unexpected null ptr from mpv_wait_event");
            }
            *ptr
        };
        to_event(event.event_id,
                 event.error,
                 event.reply_userdata,
                 event.data)
    }

    /// Observe a property change. The property change will be returned via an Event PropertyChange
    #[cfg_attr(feature = "clippy", allow(temporary_cstring_as_ptr))]
    pub fn observe_property<T:MpvFormat>(&mut self,name:&str,userdata:u32) -> Result<()>{
        let userdata = userdata as ::std::os::raw::c_ulong;
        let ret = unsafe {
            mpv_observe_property(self.handle,
                                 userdata,
                                 ffi::CString::new(name).unwrap().as_ptr(),
                                 T::get_mpv_format())
        };
        ret_to_result(ret,())
    }

    /// Unobserve a previously observed property change
    pub fn unobserve_property(&mut self,userdata:u32) -> Result<()> {
        let userdata = userdata as ::std::os::raw::c_ulong;
        let ret = unsafe {
            mpv_unobserve_property(self.handle,
                                   userdata)
        };
        ret_to_result(ret,())
    }

    /// Get the raw pointer for the mpv_handle. Use with care.
    pub fn raw(&self) -> *mut mpv_handle {
        self.handle
    }

    /// See `mpv_get_time_us`.
    pub fn get_time_us(&self) -> i64 {
        unsafe {
            mpv_get_time_us(self.handle)
        }
    }
}

impl Drop for MpvHandlerWithGl {
    fn drop(&mut self) {
        unsafe {
            // careful : always uninit gl before terminate_destroy mpv
            mpv_opengl_cb_uninit_gl(self.gl_context);
        }
    }
}

impl Drop for MpvHandler {
    fn drop(&mut self) {
        unsafe {
            mpv_terminate_destroy(self.handle);
        }
    }
}

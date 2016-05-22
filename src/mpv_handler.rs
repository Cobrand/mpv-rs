use mpv_gen::{mpv_command, mpv_command_async, mpv_wait_event, mpv_create, mpv_initialize,
              mpv_terminate_destroy, mpv_handle, mpv_set_option,
              mpv_set_property, mpv_set_property_async, mpv_get_property,
              mpv_get_property_async, mpv_opengl_cb_get_proc_address_fn, mpv_get_sub_api,
              mpv_opengl_cb_uninit_gl, mpv_opengl_cb_init_gl, mpv_opengl_cb_draw,
              mpv_opengl_cb_context, mpv_observe_property, mpv_unobserve_property,
              mpv_opengl_cb_set_update_callback};
use mpv_enums::*;
use mpv_error::*;

use std::os::raw::c_void;
use std::{ffi, ptr};
use std::sync::atomic::{AtomicBool, Ordering};

/// The main struct of the mpv-rs crate
///
/// Almost every function from the libmpv API needs a context, and sometimes an opengl context,
/// and both are stored here.
///
pub struct MpvHandler {
    handle: *mut mpv_handle,
    gl_context: Option<*mut mpv_opengl_cb_context>,
    update_available:AtomicBool
}

impl MpvHandler {

    /// Creates a mpv handler
    ///
    /// Create a new mpv instance and an associated client API handle to control
    /// the mpv instance. This instance is in a pre-initialized state,
    /// and needs to be initialized with init() or init_with_gl()
    /// to be actually used with most other API functions.
    ///
    /// Most API functions will return MPV_ERROR_UNINITIALIZED in the uninitialized
    /// state. You can call mpv.set_option() to set initial options.
    /// After this, call mpv.init() or init_with_gl() to start
    /// the player, and then use e.g. mpv.command() to start playback of a file.
    ///
    /// The point of separating handle creation and actual initialization is that
    /// you can configure things which can't be changed during runtime.
    ///
    /// Unlike the command line player, this will have initial settings suitable
    /// for embedding in applications. The following settings are different:
    /// - stdin/stdout/stderr and the terminal will never be accessed. This is
    ///   equivalent to setting the --no-terminal option.
    ///   (Technically, this also suppresses C signal handling.)
    /// - No config files will be loaded. This is roughly equivalent to using
    ///   --no-config. Since libmpv 1.15, you can actually re-enable this option,
    ///   which will make libmpv load config files during mpv.init(). If you
    ///   do this, you are strongly encouraged to set the "config-dir" option too.
    ///   (Otherwise it will load the mpv command line player's config.)
    /// - Idle mode is enabled, which means the playback core will enter idle mode
    ///   if there are no more files to play on the internal playlist, instead of
    ///   exiting. This is equivalent to the --idle option.
    /// - Disable parts of input handling.
    /// - Most of the different settings can be viewed with the command line player
    ///   by running "mpv --show-profile=libmpv".
    ///
    /// All this assumes that API users want a mpv instance that is strictly
    /// isolated from the command line player's configuration, user settings, and
    /// so on. You can re-enable disabled features by setting the appropriate
    /// options.
    ///
    /// The mpv command line parser is not available through this API, but you can
    /// set individual options with mpv_set_option(). Files for playback must be
    /// loaded with mpv_command() or others.
    ///
    /// Note that you should avoid doing concurrent accesses on the uninitialized
    /// client handle. (Whether concurrent access is definitely allowed or not has
    /// yet to be decided.)
    ///
    /// Returns a std::Result that contains an MpvHandler if successful,
    /// or an Error is the creation failed. Currently, errors can happen in the following
    /// situations :
    ///         - out of memory
    ///         - LC_NUMERIC is not set to "C" (see general remarks)
    ///
    /// You **must** init the handler afterwards using init() or init_with_gl()
    ///
    ///

    pub fn create() -> Result<MpvHandler> {
        let handle = unsafe { mpv_create() };
        if handle == ptr::null_mut() {
            return Err(Error::MPV_ERROR_NOMEM);
        }
        ret_to_result(0,MpvHandler { gl_context: None,
                                     handle: handle,
                                     update_available:AtomicBool::new(false)})
    }

    ///
    /// Inits an uninitialized player.
    /// Options should be sent to the player **before** initializing it.
    ///
    /// See set_option for more details
    ///
    /// If the mpv instance if already running, an error is returned.
    ///
    /// If everything goes well, it will return an Ok(()) (i.e. an empty Result)
    ///
    pub fn init(&mut self) -> Result<()> {
        let ret = unsafe { mpv_initialize(self.handle) };
        ret_to_result(ret, ())
    }

    ///
    /// Inits an uninitialized player with a custom opengl instance.
    ///
    /// An option of an 'extern "C"' function must be passed as a parameter,
    /// which fullfills the role of get_prox_address.
    /// An arbitrary opaque user context which will be passed to the
    /// get_proc_address callback must also be sent.
    ///
    /// # Panics
    ///
    /// It will panic if the custom get_proc_address_ctx is NULL
    ///
    /// # Errors
    ///
    /// * MPV_ERROR_UNSUPPORTED: the OpenGL version is not supported
    ///                          (or required extensions are missing)
    /// * MPV_ERROR_INVALID_PARAMETER: the OpenGL state was already initialized
    ///
    /// For additional information, see examples/sdl2.rs for a basic implementation
    pub fn init_with_gl(&mut self,
                        get_proc_address: mpv_opengl_cb_get_proc_address_fn,
                        get_proc_address_ctx: *mut ::std::os::raw::c_void)
                        -> Result<()> {
        self.set_option("vo", "opengl-cb").expect("Error setting vo option to opengl-cb");
        let result = self.init();
        if result.is_ok(){
            let opengl_ctx = unsafe {
                mpv_get_sub_api(self.handle,
                                SubApi::MPV_SUB_API_OPENGL_CB)
            } as *mut mpv_opengl_cb_context ;
            let ret = unsafe {
                mpv_opengl_cb_init_gl(opengl_ctx, ptr::null(), get_proc_address, get_proc_address_ctx)
            };
            // Actually using the opengl_cb state has to be explicitly requested.
            // Otherwise, mpv will create a separate platform window.

            // Additional callback
            unsafe {mpv_opengl_cb_set_update_callback(opengl_ctx,
                                                      Some(Self::update_draw),
                                                      self as *mut _ as *mut c_void)};

            self.gl_context = Some(opengl_ctx) ;
            ret_to_result(ret,())
        } else {
            result
        }
    }

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
    /// # Panics
    ///
    /// This function will panic if you did not initialize the object with init_with_gl(...)
    ///
    pub fn draw(&mut self, fbo: i32, width: i32, heigth: i32) -> Result<()> {
        self.update_available.store(false,Ordering::Relaxed) ;
        let opengl_ctx = self.gl_context.expect("Opengl context is required to draw");
        let ret = unsafe { mpv_opengl_cb_draw(opengl_ctx, fbo, width, heigth) };
        ret_to_result(ret, ())
    }

    /// Set a property synchronously
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
    pub fn set_property_async<T : MpvFormat>(&mut self, property: &str, value : T, userdata:u32) -> Result<()>{
        let userdata : ::std::os::raw::c_ulong = userdata as ::std::os::raw::c_ulong;
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
    pub fn get_property<T : MpvFormat>(&self, property: &str) -> Result<T> {
        let mut ret = 0 ;
        let result : T ;
        let format = T::get_mpv_format();
        result = T::get_from_c_void(|ptr:*mut c_void|{
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
    pub fn get_property_async<T : MpvFormat>(&self, property: &str, userdata :u32) -> Result<()> {
        let userdata : ::std::os::raw::c_ulong = userdata as ::std::os::raw::c_ulong;
        let ret = unsafe {
            mpv_get_property_async(self.handle,
                                   userdata,
                                   ffi::CString::new(property).unwrap().as_ptr(),
                                   T::get_mpv_format())
        };
        ret_to_result(ret,())
    }

    ///
    /// Set an option. Note that you can't normally set options during runtime.
    ///
    /// Changing options at runtime does not always work. For some options, attempts
    /// to change them simply fails. Many other options may require reloading the
    /// file for changes to take effect. In general, you should prefer calling
    /// mpv.set_property() to change settings during playback, because the property
    /// mechanism guarantees that changes take effect immediately.
    ///
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
        let userdata : ::std::os::raw::c_ulong = userdata as ::std::os::raw::c_ulong;
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
    /// It is still necessary to empty the event pool even if you don't use the events
    /// Unexpected behaviour can happen
    ///
    /// # Example
    /// ```
    /// let mpv = mpv::MpvHandler::init().expect("Error while initializing MPV");
    /// while let Some(event) = mpv.wait_event(0.0) {
    ///     println!("RECEIVED EVENT : {:?}", event.event_id.to_str());
    ///     // do something else with event
    /// }
    /// ```
    ///
    ///

    pub fn wait_event<'a,'b,'c>(&mut self,timeout:f64) -> Option<Event<'a,'b,'c>> {
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

    pub fn observe_property<T:MpvFormat>(&mut self,name:&str,userdata:u32) -> Result<()>{
        let userdata : ::std::os::raw::c_ulong = userdata as ::std::os::raw::c_ulong;
        let ret = unsafe {
            mpv_observe_property(self.handle,
                                 userdata,
                                 ffi::CString::new(name).unwrap().as_ptr(),
                                 T::get_mpv_format())
        };
        ret_to_result(ret,())
    }

    pub fn unobserve_property(&mut self,userdata:u32) -> Result<()> {
        let userdata : ::std::os::raw::c_ulong = userdata as ::std::os::raw::c_ulong;
        let ret = unsafe {
            mpv_unobserve_property(self.handle,
                                   userdata)
        };
        ret_to_result(ret,())
    }

    unsafe extern "C" fn update_draw(cb_ctx: *mut ::std::os::raw::c_void) {
        let ptr : *mut MpvHandler = cb_ctx as *mut MpvHandler ;
        let mpv : &mut MpvHandler = &mut (*ptr) ;
        mpv.update_available.store(true, Ordering::Relaxed);
    }

    pub fn is_update_available(&self) -> bool {
        self.update_available.load(Ordering::Relaxed)
    }
}

impl Drop for MpvHandler {
    fn drop(&mut self) {
        if self.gl_context.is_some(){
            unsafe {
                // careful : always uninit gl before terminate_destroy mpv
                mpv_opengl_cb_uninit_gl(self.gl_context.unwrap());
            }
        }
        unsafe {
            mpv_terminate_destroy(self.handle);
        }
    }
}

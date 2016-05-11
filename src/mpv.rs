use mpv_gen::*;
use mpv_error::*;

use std::{ffi, ptr};

pub struct Mpv {
    handle: *mut mpv_handle,
}

impl Mpv {
    pub fn init() -> Result<Mpv> {
        let handle = unsafe { mpv_create() };
        if handle == ptr::null_mut() {
            return Err(Enum_mpv_error::MPV_ERROR_NOMEM);
        }

        let ret = unsafe { mpv_initialize(handle) };

        ret_to_result(ret, Mpv { handle: handle })
    }

    // // TODO: implement this
    // pub fn get_opengl_context(&self,
    //                           get_proc_address: mpv_opengl_cb_get_proc_address_fn,
    //                           get_proc_address_ctx: *mut ::std::os::raw::c_void)
    //                           -> Result<OpenglContext> {
    //     OpenglContext::init(unsafe {
    //                             mpv_get_sub_api(self.handle,
    //                                             Enum_mpv_sub_api::MPV_SUB_API_OPENGL_CB)
    //                         } as *mut mpv_opengl_cb_context,
    //                         get_proc_address,
    //                         get_proc_address_ctx)
    // }

    pub fn command(&self, command: &[&str]) -> Result<()> {
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

    pub fn wait_event(&self) -> Option<Struct_mpv_event> {
        let event = unsafe {
            let ptr = mpv_wait_event(self.handle, 0.0);
            if ptr.is_null() {
                panic!("Unexpected null ptr from mpv_wait_event");
            }
            *ptr
        };
        match event.event_id {
            Enum_mpv_event_id::MPV_EVENT_NONE => None,
            _ => Some(event),
        }
    }
}

impl Drop for Mpv {
    fn drop(&mut self) {
        unsafe {
            mpv_terminate_destroy(self.handle);
        }
    }
}

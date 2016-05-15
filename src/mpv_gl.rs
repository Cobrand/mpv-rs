use mpv_error::*;
use mpv_gen::{mpv_opengl_cb_context,mpv_opengl_cb_draw,mpv_opengl_cb_init_gl,
    mpv_opengl_cb_uninit_gl,mpv_opengl_cb_get_proc_address_fn};

use std::os::raw::c_void;
use std::ptr;

pub struct OpenGLContext {
    handle: *mut mpv_opengl_cb_context,
}

impl OpenGLContext {
    ///
    /// Init mpv_gl
    ///
    pub fn init(ctx: *mut mpv_opengl_cb_context,
            get_proc_address: mpv_opengl_cb_get_proc_address_fn,
            get_proc_address_ctx: *mut c_void)
            -> Result<OpenGLContext> {
        assert!(!ctx.is_null());
        let ret = unsafe {
            mpv_opengl_cb_init_gl(ctx, ptr::null(), get_proc_address, get_proc_address_ctx)
        };

        ret_to_result(ret, OpenGLContext { handle: ctx })
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
    /// fbo is FRAME_BUFFER_OBJECT, do not change unless you know what you are doing !
    ///
    pub fn draw(&self, fbo: i32, width: i32, heigth: i32) -> Result<()> {
        let ret = unsafe { mpv_opengl_cb_draw(self.handle, fbo, width, heigth) };
        ret_to_result(ret, ())
    }

    // pub fn report_flip(&self, time: i64) -> Result<()> {
    //    let ret = unsafe { mpv_opengl_cb_report_flip(self.handle, time) };
    //    ret_to_result(ret, ())
    // }
}

impl Drop for OpenGLContext {
    fn drop(&mut self) {
        unsafe {
            mpv_opengl_cb_uninit_gl(self.handle);
        }
    }
}

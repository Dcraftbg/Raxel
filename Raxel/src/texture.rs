use gl33::{global_loader::*, *};

use crate::image::Image;
#[derive(Debug)]
pub struct Texture {
    pub id: u32,
    pub width: i32,
    pub height: i32,
}
impl From<Image> for Texture {
   fn from(value: Image) -> Self {
       Self { 
           id: unsafe {
              let mut tex: u32 = 0;
              glGenTextures(1, &mut tex);
              glPixelStorei(GL_UNPACK_ALIGNMENT, 1);
              glBindTexture(GL_TEXTURE_2D, tex);
	          glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST.0 as i32);
	          glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST.0 as i32);
	          glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE.0 as i32);
	          glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE.0 as i32); 
              // TODO: set format correct values
              glTexImage2D(
                  GL_TEXTURE_2D,
                  0,
                  GL_RGBA8.0 as i32,
                  value.width,
                  value.height,
                  0,
                  GL_RGBA,
                  GL_UNSIGNED_BYTE,
                  value.pixels.cast()
              );
              glGenerateMipmap(GL_TEXTURE_2D);
              tex
           },
           width: value.width,
           height: value.height
       }
   }
}
impl Texture {
   pub fn bind(&self) {
       unsafe {
       glBindTexture(GL_TEXTURE_2D, self.id);
       }
   }
   pub unsafe fn new_gl() -> Self {
       let mut tex = 0;
       glGenTextures(1, &mut tex);
       Self { id: tex, width: 0, height: 0}
   }
   pub unsafe fn buffer_sub_gl(&self, level: i32, xoff: i32, yoff: i32, width: i32, height: i32, format: GLenum, typ: GLenum, pixels: *const i8) {
        self.bind();
        glTexSubImage2D(GL_TEXTURE_2D, level, xoff, yoff, width, height, format, typ, pixels.cast());
   }
   pub unsafe fn buffer_raw_gl(&self, level: i32, inner_fmt: i32, width: i32, height: i32, fmt: GLenum, typ: GLenum, pixels: *const i8) {
       self.bind();
       glTexImage2D(GL_TEXTURE_2D, level, inner_fmt, width, height, 0, fmt, typ, pixels.cast());
   }
   pub const fn null() -> Self {
       Self { id: 0, width: 0, height: 0 }
   }
}
impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
        glDeleteTextures(1, &self.id);
        }
    }
}

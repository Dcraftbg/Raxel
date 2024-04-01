

use std::ffi::CString;
use stb_image::{stbi_image_free, stbi_load, stbi_set_flip_vertically_on_load};
pub struct Image {
    pub pixels: *mut i8,
    pub width: i32,
    pub height: i32,
    pub channels: i32
}
impl Image {
    pub fn load_file(path: &str) -> std::io::Result<Self> {
        unsafe {
        stbi_set_flip_vertically_on_load(1);
        let pathc = CString::new(path).unwrap();
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut channels: i32 = 0;
        let desired_channels: i32 = 4;
        let data = stbi_load(pathc.into_raw(), &mut x, &mut y, &mut channels, desired_channels);
        if data == 0 as *mut _ {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, path))
        } else {
            Ok(Self { pixels: data, width: x, height: y, channels})
        }
        }
    }
}
impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
        stbi_image_free(self.pixels);
        }
    }
}

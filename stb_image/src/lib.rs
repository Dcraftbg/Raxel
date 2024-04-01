use std::ffi::{c_char, c_int};

extern "C" {
    pub fn stbi_load(path: *const c_char, x: *mut c_int, y: *mut c_int, channels: *mut c_int, desired_channels: c_int) -> *mut i8;
    pub fn stbi_image_free(data: *mut i8);
    pub fn stbi_set_flip_vertically_on_load(v: i32);
}

